//! Builds the static website from a [`Site`].
//!
//! The output goes to `_site/` in the site folder:
//!
//! ```text
//! _site/
//!   index.html            the start page
//!   about/index.html      /about/
//!   style.css             one stylesheet for every site and theme
//!   media/…               copied from the site's media folder, plus smaller
//!                         copies of photos (`team.480w.jpg`, see images.rs)
//!   assets/…              copied from the site's assets folder (downloads)
//!   pdf/a.pdf, …          the site's static folder, copied as it is
//!   sitemap.xml           only if `base_url` is set
//!   blog/feed.xml         an RSS feed for each page with posts below it,
//!                         also only with `base_url`
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use askama::Template;
use serde::Serialize;

use crate::block::{self, Block, Side};
use crate::calendar::{self, Date};
use crate::deploy::MANIFEST_FILE;
use crate::fonts;
use crate::images::{self, ImageError, Images};
use crate::links::LinkContext;
use crate::markdown;
use crate::media::CLUTTER;
use crate::page::Menu;
use crate::site::{
    ASSETS_DIR, HeaderStyle, MEDIA_DIR, Navigation, Page, STATIC_DIR, Site, SiteConfig,
};
use crate::slug::slugify;

pub const OUTPUT_DIR: &str = "_site";

/// Marks a folder as built by qed, so it may be emptied on the next build.
const MARKER: &str = ".qed-build";

/// The stylesheet, compiled by `npm run build:site-css` and built into qed.
const STYLESHEET: &str = include_str!("../assets/site.css");

/// What a build produced.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct BuildReport {
    pub out_dir: PathBuf,
    pub pages: usize,
    pub drafts_skipped: usize,
    pub redirects: usize,
    /// Images in `media/` and downloads in `assets/`.
    pub media_files: usize,
    /// Smaller copies of photos, for phones and narrow columns.
    pub resized_images: usize,
    /// RSS feeds written, one per page with posts below it.
    pub feeds: usize,
    /// Files from `static/`, copied as they are.
    pub static_files: usize,
    /// Things worth knowing that didn't stop the build, such as a feed
    /// left out.
    pub notes: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("can't write {}: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error(
        "{} already exists and wasn't built by q.e.d.; move it away so q.e.d. doesn't overwrite your files",
        .0.display()
    )]
    NotOurs(PathBuf),

    #[error("rendering /{page}: {source}")]
    Template { page: String, source: askama::Error },

    #[error("/{page} wants a redirect from /{from}, but a page already lives there")]
    RedirectOverPage { page: String, from: String },

    #[error(
        "{STATIC_DIR}/{path} would replace /{path}, which q.e.d. builds itself; rename or move the file"
    )]
    StaticOverBuilt { path: String },

    #[error(transparent)]
    Image(#[from] ImageError),
}

/// Builds the site into `_site/` inside its folder.
pub fn build(site: &Site) -> Result<BuildReport, BuildError> {
    build_into(site, &site.root.join(OUTPUT_DIR))
}

/// Builds the site into `out_dir`, replacing an earlier build there.
pub fn build_into(site: &Site, out_dir: &Path) -> Result<BuildReport, BuildError> {
    build_on(site, out_dir, calendar::today())
}

/// Builds as if it were `today`: posts dated later are left out of lists
/// and feeds.
fn build_on(site: &Site, out_dir: &Path, today: Date) -> Result<BuildReport, BuildError> {
    prepare_output(out_dir)?;

    let published: Vec<&Page> = site
        .pages
        .iter()
        .filter(|p| !p.content.front.draft)
        .collect();
    let media_dir = site.root.join(MEDIA_DIR);
    let images = Images::new(&media_dir);
    let main_menu = menu(&published, Menu::Main);
    let footer_menu = menu(&published, Menu::Footer);
    // Feeds need full addresses, so only a site with a web address gets them.
    let mut notes = Vec::new();
    let sources = feed_sources(&published, today);
    let feeds: Vec<Feed> = match &site.config.base_url {
        Some(_) => sources
            .iter()
            .map(|page| Feed {
                page,
                file: feed_file(&page.path),
                title: feed_title(&site.config, page),
            })
            .collect(),
        None => {
            if !sources.is_empty() {
                notes.push(
                    "No RSS feed: feeds need the site's web address, under Site → Settings."
                        .to_string(),
                );
            }
            Vec::new()
        }
    };
    let context = Context {
        root: &site.root,
        images: &images,
        pages: &published,
        feeds: &feeds,
        today,
        language: &site.config.language,
        footer: false,
    };

    for page in &published {
        let html = render_page(site, &context, page, &main_menu, &footer_menu, &feeds)?;
        write(&page_file(out_dir, &page.path), html)?;
    }

    let mut redirects = 0;
    for page in &published {
        for from in &page.content.front.redirect_from {
            let from = from.trim_matches('/');
            if published.iter().any(|p| p.path == from) {
                return Err(BuildError::RedirectOverPage {
                    page: page.path.clone(),
                    from: from.to_string(),
                });
            }
            // An old address of a file (`2016/03/28/easterhegg.html`, as
            // Jekyll made them) is written as that file; any other one as a
            // folder with an index.html, like a page.
            let (folder, file) = match from.strip_suffix(".html") {
                Some(_) => (
                    from.rsplit_once('/').map_or("", |(folder, _)| folder),
                    out_dir.join(from),
                ),
                None => (from, page_file(out_dir, from)),
            };
            let links = LinkContext::new(folder, Path::new(""));
            let target = links.page(&page.path);
            let html = RedirectTemplate { target: &target }
                .render()
                .map_err(|source| BuildError::Template {
                    page: page.path.clone(),
                    source,
                })?;
            write(&file, html)?;
            redirects += 1;
        }
    }

    // A custom theme and the site's fonts add to the one stylesheet; the
    // fonts in use are copied next to it.
    let custom_css = site.config.custom().map(|c| c.css()).unwrap_or_default();
    let fonts_css = fonts::css(&site.root, &site.config.fonts);
    write(
        &out_dir.join("style.css"),
        format!("{STYLESHEET}{custom_css}{fonts_css}"),
    )?;
    for font in fonts::used(&site.root, &site.config.fonts) {
        copy_dir(
            &site.root.join(fonts::FONTS_DIR).join(&font.id),
            &out_dir.join(fonts::FONTS_DIR).join(&font.id),
        )?;
    }
    for family in [&site.config.fonts.text, &site.config.fonts.headings]
        .into_iter()
        .flatten()
    {
        if fonts::installed(&site.root, family).is_none() {
            notes.push(format!(
                "The font \"{family}\" isn't downloaded; the site uses the system font. Download it under Site → Theme → Fonts."
            ));
        }
    }

    let media_files = copy_dir(&media_dir, &out_dir.join(MEDIA_DIR))?
        + copy_dir(&site.root.join(ASSETS_DIR), &out_dir.join(ASSETS_DIR))?;
    let resized_images =
        images.write(&out_dir.join(MEDIA_DIR), &site.root.join(images::CACHE_DIR))?;

    if let Some(base_url) = &site.config.base_url {
        write(&out_dir.join("sitemap.xml"), sitemap(base_url, &published))?;
        for feed in &feeds {
            let posts = posts(&published, &feed.page.path, today);
            write(
                &out_dir.join(&feed.file),
                rss(&site.config, base_url, feed, &posts),
            )?;
        }
    }

    // Last, so a static file can't hide something q.e.d. built.
    let static_files = copy_static(&site.root.join(STATIC_DIR), out_dir, "")?;

    Ok(BuildReport {
        out_dir: out_dir.to_path_buf(),
        pages: published.len(),
        drafts_skipped: site.pages.len() - published.len(),
        redirects,
        media_files,
        resized_images,
        feeds: feeds.len(),
        static_files,
        notes,
    })
}

/// Lucide's RSS icon, for feed links.
const RSS_ICON: &str = include_str!("../assets/icons/rss.svg");

/// What blocks need besides themselves.
#[derive(Clone, Copy)]
struct Context<'a> {
    /// The site folder.
    root: &'a Path,
    images: &'a Images,
    /// The pages being built, for Post lists.
    pages: &'a [&'a Page],
    /// The feeds being written, so a Post list can link its own.
    feeds: &'a [Feed<'a>],
    today: Date,
    /// The site's language, for dates.
    language: &'a str,
    /// Rendering the footer: lists come out compact, as plain links.
    footer: bool,
}

/// The posts below the page at `parent`, newest first: pages with a date
/// that isn't after `today`. Posts of the same day go by `weight`, the
/// higher first: the sidebar shows lower weights first, so the list is the
/// other way round, like the dates.
fn posts<'a>(pages: &[&'a Page], parent: &str, today: Date) -> Vec<&'a Page> {
    let below = format!("{parent}/");
    let mut posts: Vec<(Date, &Page)> = pages
        .iter()
        .copied()
        .filter(|page| {
            if parent.is_empty() {
                !page.path.is_empty()
            } else {
                page.path.starts_with(&below)
            }
        })
        .filter_map(|page| {
            let date = page.content.front.date.as_ref()?.date()?;
            (date <= today).then_some((date, page))
        })
        .collect();
    posts.sort_by(|(a, page_a), (b, page_b)| {
        b.cmp(a)
            .then_with(|| {
                let weight = |page: &Page| page.content.front.weight;
                weight(page_b).cmp(&weight(page_a))
            })
            .then_with(|| page_a.path.cmp(&page_b.path))
    });
    posts.into_iter().map(|(_, page)| page).collect()
}

/// One RSS feed: the posts below `page`.
struct Feed<'a> {
    page: &'a Page,
    /// In the built site: `blog/feed.xml`.
    file: String,
    title: String,
}

/// The pages that get a feed: those with at least one post right below
/// them, so `/blog` does for `/blog/2026-10-21-hello`.
fn feed_sources<'a>(pages: &[&'a Page], today: Date) -> Vec<&'a Page> {
    pages
        .iter()
        .copied()
        .filter(|parent| {
            pages.iter().any(|page| {
                !page.path.is_empty()
                    && page.path.rsplit_once('/').map_or("", |(up, _)| up) == parent.path
                    && page
                        .content
                        .front
                        .date
                        .as_ref()
                        .and_then(|date| date.date())
                        .is_some_and(|date| date <= today)
            })
        })
        .collect()
}

fn feed_file(path: &str) -> String {
    if path.is_empty() {
        "feed.xml".to_string()
    } else {
        format!("{path}/feed.xml")
    }
}

/// "Blog · My Site", or just the site's title for posts on the start page.
fn feed_title(config: &SiteConfig, page: &Page) -> String {
    if page.path.is_empty() {
        config.title.clone()
    } else {
        format!("{} · {}", page.content.front.title, config.title)
    }
}

/// The full address of a page: `https://example.com/blog/`.
fn absolute(base_url: &str, path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    if path.is_empty() {
        format!("{base}/")
    } else {
        format!("{base}/{path}/")
    }
}

/// An RSS 2.0 feed. The summary goes in as the description, as written.
fn rss(config: &SiteConfig, base_url: &str, feed: &Feed, posts: &[&Page]) -> String {
    let page = feed.page;
    let description = page
        .content
        .front
        .description
        .as_deref()
        .or(config.description.as_deref())
        .unwrap_or(&feed.title);
    let self_link = format!("{}/{}", base_url.trim_end_matches('/'), feed.file);
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\" \
         xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n<channel>\n",
    );
    xml.push_str(&format!(
        "  <title>{}</title>\n  <link>{}</link>\n  <description>{}</description>\n  \
         <language>{}</language>\n  \
         <atom:link href=\"{}\" rel=\"self\" type=\"application/rss+xml\"/>\n",
        escape_xml(&feed.title),
        escape_xml(&absolute(base_url, &page.path)),
        escape_xml(description),
        escape_xml(&config.language),
        escape_xml(&self_link),
    ));
    for post in posts {
        let front = &post.content.front;
        let url = escape_xml(&absolute(base_url, &post.path));
        xml.push_str(&format!(
            "  <item>\n    <title>{}</title>\n    <link>{url}</link>\n    \
             <guid isPermaLink=\"true\">{url}</guid>\n",
            escape_xml(&front.title)
        ));
        if let Some(date) = front.date.as_ref().and_then(|date| date.date()) {
            xml.push_str(&format!(
                "    <pubDate>{}</pubDate>\n",
                calendar::rfc822(date)
            ));
        }
        if let Some(text) = front.summary.as_deref().or(front.description.as_deref()) {
            xml.push_str(&format!(
                "    <description>{}</description>\n",
                escape_xml(text)
            ));
        }
        for author in &front.authors {
            xml.push_str(&format!(
                "    <dc:creator>{}</dc:creator>\n",
                escape_xml(author)
            ));
        }
        xml.push_str("  </item>\n");
    }
    xml.push_str("</channel>\n</rss>\n");
    xml
}

/// Empties an earlier build, or refuses if the folder holds something else.
fn prepare_output(out_dir: &Path) -> Result<(), BuildError> {
    let io = |source| BuildError::Io {
        path: out_dir.to_path_buf(),
        source,
    };
    if out_dir.exists() {
        let is_ours = out_dir.join(MARKER).is_file();
        let is_empty = fs::read_dir(out_dir).map_err(io)?.next().is_none();
        if !is_ours && !is_empty {
            return Err(BuildError::NotOurs(out_dir.to_path_buf()));
        }
        fs::remove_dir_all(out_dir).map_err(io)?;
    }
    fs::create_dir_all(out_dir).map_err(io)?;
    write(&out_dir.join(MARKER), String::new())
}

/// `""` becomes `index.html`, `about/team` becomes `about/team/index.html`.
fn page_file(out_dir: &Path, path: &str) -> PathBuf {
    if path.is_empty() {
        out_dir.join("index.html")
    } else {
        out_dir.join(path).join("index.html")
    }
}

fn write(path: &Path, contents: String) -> Result<(), BuildError> {
    let io = |source| BuildError::Io {
        path: path.to_path_buf(),
        source,
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io)?;
    }
    fs::write(path, contents).map_err(io)
}

/// Copies a folder recursively, skipping hidden files. Returns the number of
/// files copied. A missing folder copies nothing.
fn copy_dir(from: &Path, to: &Path) -> Result<usize, BuildError> {
    if !from.is_dir() {
        return Ok(0);
    }
    let io = |path: &Path| {
        let path = path.to_path_buf();
        move |source| BuildError::Io { path, source }
    };
    fs::create_dir_all(to).map_err(io(to))?;

    let mut count = 0;
    for entry in fs::read_dir(from).map_err(io(from))? {
        let entry = entry.map_err(io(from))?;
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        let (source, target) = (entry.path(), to.join(&name));
        if source.is_dir() {
            count += copy_dir(&source, &target)?;
        } else {
            fs::copy(&source, &target).map_err(io(&target))?;
            count += 1;
        }
    }
    Ok(count)
}

/// Copies the static folder into the built site at the same paths, hidden
/// files included (`.well-known/`, `.htaccess`). `prefix` is the path below
/// the static folder so far (`""`, then `pdf/`). Fails rather than replace a
/// file the build wrote. Returns the number of files copied.
fn copy_static(from: &Path, out_dir: &Path, prefix: &str) -> Result<usize, BuildError> {
    if !from.is_dir() {
        return Ok(0);
    }
    let io = |path: &Path| {
        let path = path.to_path_buf();
        move |source| BuildError::Io { path, source }
    };
    let to = out_dir.join(prefix);
    fs::create_dir_all(&to).map_err(io(&to))?;

    let mut count = 0;
    for entry in fs::read_dir(from).map_err(io(from))? {
        let entry = entry.map_err(io(from))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if CLUTTER.contains(&name.as_str()) {
            continue;
        }
        let path = format!("{prefix}{name}");
        let source = entry.path();
        if source.is_dir() {
            count += copy_static(&source, out_dir, &format!("{path}/"))?;
            continue;
        }
        // The publish manifest lives next to the site on the server.
        if path == MANIFEST_FILE {
            return Err(BuildError::StaticOverBuilt { path });
        }
        let target = to.join(&name);
        // `create_new` fails if the build already wrote this file.
        let mut out = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(BuildError::StaticOverBuilt { path });
            }
            Err(error) => return Err(io(&target)(error)),
        };
        let mut input = fs::File::open(&source).map_err(io(&source))?;
        std::io::copy(&mut input, &mut out).map_err(io(&target))?;
        count += 1;
    }
    Ok(count)
}

/// One menu entry, with its link still relative to the site root.
#[derive(Debug, Clone)]
struct MenuEntry {
    title: String,
    path: String,
    /// How deep the entry sits below other entries of the same menu.
    level: usize,
    /// The page's icon as SVG markup.
    icon: Option<&'static str>,
}

/// The pages in a menu, ordered by weight, then title.
fn menu(pages: &[&Page], which: Menu) -> Vec<MenuEntry> {
    let mut entries: Vec<&Page> = pages
        .iter()
        .copied()
        .filter(|p| p.content.front.menu == which)
        .collect();
    entries.sort_by(|a, b| {
        let (a, b) = (&a.content.front, &b.content.front);
        a.weight.cmp(&b.weight).then_with(|| a.title.cmp(&b.title))
    });

    // An entry's parent is the nearest entry above it in the page tree:
    // `blog/first-post` goes under `blog`, if `blog` is in the menu too.
    let parent = |page: &Page| -> Option<usize> {
        entries
            .iter()
            .enumerate()
            .filter(|(_, other)| other.path != page.path && is_within(&page.path, &other.path))
            .max_by_key(|(_, other)| other.path.len())
            .map(|(index, _)| index)
    };
    let parents: Vec<Option<usize>> = entries.iter().map(|page| parent(page)).collect();

    // Walk the tree depth first, so children follow their parent. Siblings
    // keep the order sorted above.
    let mut ordered = Vec::with_capacity(entries.len());
    let mut stack: Vec<(usize, usize)> = (0..entries.len())
        .rev()
        .filter(|&index| parents[index].is_none())
        .map(|index| (index, 0))
        .collect();
    while let Some((index, level)) = stack.pop() {
        let page = entries[index];
        ordered.push(MenuEntry {
            title: page.content.front.title.clone(),
            path: page.path.clone(),
            level,
            icon: page
                .content
                .front
                .icon
                .as_deref()
                .and_then(crate::icons::svg),
        });
        stack.extend(
            (0..entries.len())
                .rev()
                .filter(|&child| parents[child] == Some(index))
                .map(|child| (child, level + 1)),
        );
    }
    ordered
}

fn sitemap(base_url: &str, pages: &[&Page]) -> String {
    let base = base_url.trim_end_matches('/');
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for page in pages {
        let url = if page.path.is_empty() {
            format!("{base}/")
        } else {
            format!("{base}/{}/", page.path)
        };
        xml.push_str(&format!("  <url><loc>{}</loc></url>\n", escape_xml(&url)));
    }
    xml.push_str("</urlset>\n");
    xml
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ---------------------------------------------------------------------------
// Templates

/// The data `templates/page.html` renders. Askama checks at compile time that
/// every field the template uses exists here.
#[derive(Template)]
#[template(path = "page.html")]
struct PageTemplate<'a> {
    site: &'a SiteConfig,
    title: &'a str,
    description: Option<&'a str>,
    canonical: Option<String>,
    /// `og:type`: "article" for posts (pages with a date), else "website".
    og_type: &'static str,
    /// The image for link previews; only with `base_url`.
    share_image: Option<ShareImage>,
    /// The site's logo, for the header.
    logo: Option<Picture>,
    /// The browser-tab icon: `favicon.ico` from `static/` if the site has
    /// one, else the logo.
    icon: Option<String>,
    /// The site's RSS feeds, for `<link rel="alternate">` and the footer.
    feeds: Vec<FeedLink>,
    rss_icon: &'static str,
    is_home: bool,
    /// Relative path to the site root, for the stylesheet.
    root: String,
    home_href: String,
    main_menu: Vec<MenuLink>,
    footer_menu: Vec<MenuLink>,
    /// No title above a starting Hero or the start page's banner; see
    /// [`show_title`].
    show_title: bool,
    /// The main menu goes into a sidebar instead of the top bar.
    sidebar: bool,
    /// The content width; wider with a sidebar.
    width: &'static str,
    /// Classes for the header, the current menu entry and the logo; see
    /// [`header_classes`].
    header: HeaderClasses,
    /// Date and authors, e.g. "2026-10-21 · Claude".
    byline: String,
    /// Each block's HTML, from `block.html`.
    blocks: Vec<String>,
    /// The site footer's blocks, rendered for this page.
    footer_blocks: Vec<String>,
}

/// Tailwind classes for the header, written out in full here so the
/// stylesheet build finds them (`site.input.css` scans this file).
struct HeaderClasses {
    bar: &'static str,
    /// The current page in the top menu.
    current: &'static str,
    /// The logo: on a colored bar it gets a light backing, so a logo in the
    /// bar's own color doesn't disappear.
    logo: &'static str,
}

fn header_classes(style: HeaderStyle) -> HeaderClasses {
    // On a colored bar, primary text would vanish (or not stand out), so
    // the current page is underlined instead.
    const UNDERLINED: &str = "font-semibold underline decoration-2 underline-offset-4";
    // A fixed height and the width it needs: square logos stay square,
    // wide ones (a wordmark, a row of icons) get room up to 12rem.
    const BACKED: &str = "h-9 w-auto max-w-48 object-contain rounded-field bg-base-100 p-0.5";
    match style {
        HeaderStyle::Plain => HeaderClasses {
            bar: "border-t-4 border-b border-t-accent border-b-base-300",
            current: "font-semibold text-primary",
            logo: "h-9 w-auto max-w-48 object-contain",
        },
        HeaderStyle::Primary => HeaderClasses {
            bar: "bg-primary text-primary-content",
            current: UNDERLINED,
            logo: BACKED,
        },
        HeaderStyle::Secondary => HeaderClasses {
            bar: "bg-secondary text-secondary-content",
            current: UNDERLINED,
            logo: BACKED,
        },
        HeaderStyle::Accent => HeaderClasses {
            bar: "bg-accent text-accent-content",
            current: UNDERLINED,
            logo: BACKED,
        },
        HeaderStyle::Neutral => HeaderClasses {
            bar: "bg-neutral text-neutral-content",
            current: UNDERLINED,
            logo: BACKED,
        },
    }
}

/// One block, rendered on its own so columns can hold blocks.
#[derive(Template)]
#[template(path = "block.html")]
struct BlockTemplate<'a> {
    block: &'a RenderedBlock,
    /// `sizes` for an image as wide as the block, and for one half as wide
    /// (Text with image).
    full_sizes: &'static str,
    half_sizes: &'static str,
    rss_icon: &'static str,
}

#[derive(Template)]
#[template(path = "redirect.html")]
struct RedirectTemplate<'a> {
    target: &'a str,
}

struct MenuLink {
    title: String,
    href: String,
    /// Indentation in the sidebar: 0 for top-level entries.
    level: usize,
    icon: Option<&'static str>,
    /// This page, or a page below it, is the one being shown.
    current: bool,
    /// Exactly this page is being shown.
    exact: bool,
}

struct Button {
    label: String,
    href: String,
}

struct FeedLink {
    title: String,
    href: String,
    /// For the footer: "RSS feed", or "Blog RSS feed" when there are several.
    label: String,
}

struct PostEntry {
    title: String,
    href: String,
    /// `2017-02-24`, for `<time datetime>`.
    date: String,
    /// `24. Februar 2017`, in the site's language.
    date_text: String,
    summary_html: Option<String>,
}

/// One page in a Page list.
struct PageEntry {
    title: String,
    href: String,
    summary_html: Option<String>,
}

/// A block, ready for the template: Markdown rendered, links resolved.
enum RenderedBlock {
    Text {
        html: String,
    },
    Hero {
        /// A background photo; decorative, so its `alt` is empty.
        image: Option<Picture>,
        headline: Option<String>,
        text_html: Option<String>,
        button: Option<Button>,
    },
    Image {
        /// `None` while no image is chosen; nothing is shown then.
        picture: Option<Picture>,
        caption_html: Option<String>,
    },
    TextImage {
        text_html: Option<String>,
        picture: Option<Picture>,
        image_left: bool,
    },
    Gallery {
        pictures: Vec<Picture>,
    },
    Download {
        /// `None` while no file is chosen; nothing is shown then.
        href: Option<String>,
        label: String,
        /// The file extension, e.g. "PDF".
        kind: String,
        size: Option<String>,
    },
    Faq {
        heading: Option<String>,
        items: Vec<FaqEntry>,
    },
    Columns {
        /// Grid classes for the number of columns.
        grid: &'static str,
        columns: Vec<ColumnEntry>,
    },
    CallToAction {
        headline: Option<String>,
        text_html: Option<String>,
        button: Option<Button>,
    },
    PostList {
        posts: Vec<PostEntry>,
        /// The feed of the listed page, if it has one.
        feed_href: Option<String>,
        /// Titles only, for the footer.
        compact: bool,
    },
    PageList {
        pages: Vec<PageEntry>,
        compact: bool,
    },
    Video {
        href: String,
        picture: Option<Picture>,
        title: Option<String>,
        /// `YouTube`, `Vimeo`, or a PeerTube instance's host.
        platform: Option<String>,
    },
}

/// The image link previews show, with its full address.
struct ShareImage {
    url: String,
    width: Option<u32>,
    height: Option<u32>,
    alt: Option<String>,
}

struct Picture {
    src: String,
    /// The smaller copies and the original, with their widths, for the
    /// browser to choose from: `media/a.480w.jpg 480w, …, media/a.jpg 2400w`.
    srcset: Option<String>,
    alt: String,
    width: Option<u32>,
    height: Option<u32>,
}

struct ColumnEntry {
    title: Option<String>,
    /// The column's blocks, each rendered with `block.html`.
    blocks: Vec<String>,
}

struct FaqEntry {
    question: String,
    answer_html: String,
}

fn render_page(
    site: &Site,
    context: &Context,
    page: &Page,
    main_menu: &[MenuEntry],
    footer_menu: &[MenuEntry],
    feeds: &[Feed],
) -> Result<String, BuildError> {
    let config = &site.config;
    let front = &page.content.front;
    let links = LinkContext::new(&page.path, &page.file);

    let menu_links = |entries: &[MenuEntry]| -> Vec<MenuLink> {
        entries
            .iter()
            .map(|entry| MenuLink {
                title: entry.title.clone(),
                href: links.page(&entry.path),
                level: entry.level,
                icon: entry.icon,
                current: is_within(&page.path, &entry.path),
                exact: page.path == entry.path,
            })
            .collect()
    };

    let mut byline: Vec<String> = Vec::new();
    if let Some(date) = &front.date {
        byline.push(date.date().map_or_else(
            || date.to_string(),
            |day| calendar::long_date(day, &site.config.language),
        ));
    }
    if !front.authors.is_empty() {
        byline.push(front.authors.join(", "));
    }

    let canonical = config
        .base_url
        .as_ref()
        .map(|base| absolute(base, &page.path));

    let share_image = config
        .base_url
        .as_ref()
        .and_then(|base| share_image(page, base, context.images));

    let sidebar = config.navigation == Navigation::Sidebar && !main_menu.is_empty();
    let template = PageTemplate {
        site: config,
        title: &front.title,
        description: front.description.as_deref(),
        canonical,
        og_type: if front.date.is_some() {
            "article"
        } else {
            "website"
        },
        share_image,
        logo: config
            .logo
            .as_deref()
            .and_then(|name| picture(name, None, &links, context.images)),
        // A favicon.ico of its own wins: a wide logo makes a poor tab icon.
        icon: if site.root.join(STATIC_DIR).join("favicon.ico").is_file() {
            Some(format!("{}favicon.ico", links.to_root()))
        } else {
            config
                .logo
                .as_deref()
                .and_then(|name| picture(name, None, &links, context.images))
                .map(|logo| logo.src)
        },
        feeds: feeds
            .iter()
            .map(|feed| FeedLink {
                title: feed.title.clone(),
                href: format!("{}{}", links.to_root(), feed.file),
                label: if feeds.len() == 1 {
                    "RSS feed".to_string()
                } else {
                    format!("{} RSS feed", feed.page.content.front.title)
                },
            })
            .collect(),
        rss_icon: RSS_ICON,
        is_home: page.path.is_empty(),
        root: links.to_root(),
        home_href: links.page(""),
        main_menu: menu_links(main_menu),
        footer_menu: menu_links(footer_menu),
        show_title: show_title(page),
        sidebar,
        width: if sidebar { "max-w-6xl" } else { "max-w-4xl" },
        header: header_classes(config.header),
        byline: byline.join(" · "),
        blocks: page
            .content
            .blocks
            .iter()
            .map(|block| render_html(block, &links, context, false))
            .collect::<Result<_, _>>()
            .map_err(|source| BuildError::Template {
                page: page.path.clone(),
                source,
            })?,
        // The footer's blocks, with this page's links (the footer is on
        // every page, so its links are relative to each one).
        footer_blocks: match &site.footer {
            Some(footer) => {
                let in_footer = Context {
                    footer: true,
                    ..*context
                };
                footer
                    .content
                    .blocks
                    .iter()
                    .map(|block| render_html(block, &links, &in_footer, false))
                    .collect::<Result<_, _>>()
                    .map_err(|source| BuildError::Template {
                        page: page.path.clone(),
                        source,
                    })?
            }
            None => Vec::new(),
        },
    };

    template.render().map_err(|source| BuildError::Template {
        page: page.path.clone(),
        source,
    })
}

/// Whether the page shows its title above the blocks. A page that starts
/// with a Hero has its headline instead; the start page that opens with an
/// image (a banner) doesn't repeat the site's name, which the header shows.
fn show_title(page: &Page) -> bool {
    match page.content.blocks.first() {
        Some(Block::Hero(_)) => false,
        Some(Block::Image(_)) => !page.path.is_empty(),
        _ => true,
    }
}

/// A block's HTML. `in_column` makes images expect less room.
fn render_html(
    block: &Block,
    links: &LinkContext,
    context: &Context,
    in_column: bool,
) -> Result<String, askama::Error> {
    let rendered = render_block(block, links, context)?;
    let (full_sizes, half_sizes) = if in_column {
        (
            "(min-width: 48rem) 28rem, 100vw",
            "(min-width: 48rem) 14rem, 100vw",
        )
    } else {
        (
            "(min-width: 56rem) 56rem, 100vw",
            "(min-width: 48rem) 28rem, 100vw",
        )
    };
    BlockTemplate {
        block: &rendered,
        full_sizes,
        half_sizes,
        rss_icon: RSS_ICON,
    }
    .render()
}

/// A menu entry is current on its own page and on the pages below it, except
/// for the start page, which would otherwise be current everywhere.
fn is_within(page: &str, entry: &str) -> bool {
    page == entry || (!entry.is_empty() && page.starts_with(&format!("{entry}/")))
}

fn render_block(
    block: &Block,
    links: &LinkContext,
    context: &Context,
) -> Result<RenderedBlock, askama::Error> {
    let (root, images) = (context.root, context.images);
    let markdown = |text: &Option<String>| text.as_deref().map(|t| markdown::render(t, links));
    let button = |label: &Option<String>, link: &Option<String>| match (label, link) {
        (Some(label), Some(link)) => Some(Button {
            label: label.clone(),
            href: links.resolve(link),
        }),
        _ => None,
    };

    Ok(match block {
        Block::Text { markdown: text } => RenderedBlock::Text {
            html: markdown::render(text, links),
        },
        Block::Hero(hero) => RenderedBlock::Hero {
            image: hero
                .image
                .as_deref()
                .and_then(|name| picture(name, None, links, images)),
            headline: hero.headline.clone(),
            text_html: markdown(&hero.text),
            button: button(&hero.button_label, &hero.button_link),
        },
        Block::Image(image) => RenderedBlock::Image {
            picture: image
                .image
                .as_deref()
                .and_then(|name| picture(name, image.alt.as_deref(), links, images)),
            caption_html: markdown(&image.caption),
        },
        Block::TextImage(block) => RenderedBlock::TextImage {
            text_html: markdown(&block.text),
            picture: block
                .image
                .as_deref()
                .and_then(|name| picture(name, block.alt.as_deref(), links, images)),
            image_left: block.image_side == Side::Left,
        },
        Block::Gallery(gallery) => RenderedBlock::Gallery {
            pictures: gallery
                .images
                .iter()
                .filter_map(|image| picture(&image.image, image.alt.as_deref(), links, images))
                .collect(),
        },
        Block::Download(download) => {
            let file = download
                .file
                .as_deref()
                .map(str::trim)
                .filter(|name| !name.is_empty());
            // A file name, or else a URL or site path to link as it is.
            let (href, local) = match file {
                Some(name) if !name.contains(':') && !name.starts_with('/') => {
                    let (href, path) = download_location(name, links, root);
                    (Some(href), Some(path))
                }
                Some(link) => (Some(links.resolve(link)), None),
                None => (None, None),
            };
            RenderedBlock::Download {
                href,
                label: download
                    .label
                    .clone()
                    .filter(|label| !label.trim().is_empty())
                    .or_else(|| {
                        file.map(|name| name.rsplit('/').next().unwrap_or(name).to_string())
                    })
                    .unwrap_or_default(),
                kind: file
                    .and_then(|name| name.rsplit_once('.'))
                    .map(|(_, extension)| extension.to_uppercase())
                    .unwrap_or_default(),
                size: local
                    .and_then(|path| std::fs::metadata(path).ok())
                    .map(|metadata| format_size(metadata.len())),
            }
        }
        Block::Faq(faq) => RenderedBlock::Faq {
            heading: faq
                .heading
                .clone()
                .filter(|heading| !heading.trim().is_empty()),
            items: faq
                .items
                .iter()
                .filter(|item| !item.question.trim().is_empty())
                .map(|item| FaqEntry {
                    question: item.question.clone(),
                    answer_html: markdown::render(&item.answer, links),
                })
                .collect(),
        },
        Block::Columns(columns) => RenderedBlock::Columns {
            // Whole class names, so Tailwind finds them when it scans the code.
            grid: match columns.items.len() {
                0 | 1 => "",
                2 => "md:grid-cols-2",
                3 => "md:grid-cols-3",
                _ => "md:grid-cols-2 lg:grid-cols-4",
            },
            columns: columns
                .items
                .iter()
                .map(|column| {
                    Ok(ColumnEntry {
                        title: column
                            .title
                            .clone()
                            .filter(|title| !title.trim().is_empty()),
                        blocks: column
                            .blocks
                            .iter()
                            .map(|inner| render_html(inner, links, context, true))
                            .collect::<Result<_, _>>()?,
                    })
                })
                .collect::<Result<_, askama::Error>>()?,
        },
        Block::CallToAction(cta) => RenderedBlock::CallToAction {
            headline: cta.headline.clone(),
            text_html: markdown(&cta.text),
            button: button(&cta.button_label, &cta.button_link),
        },
        Block::PostList(list) => {
            let from = list_source(&list.from, links);
            let mut posts = posts(context.pages, from, context.today);
            if let Some(limit) = list.limit.and_then(|limit| usize::try_from(limit).ok()) {
                posts.truncate(limit);
            }
            RenderedBlock::PostList {
                compact: context.footer,
                feed_href: context
                    .feeds
                    .iter()
                    .find(|feed| feed.page.path == from)
                    .map(|feed| format!("{}{}", links.to_root(), feed.file)),
                posts: posts
                    .iter()
                    .map(|post| {
                        let date = post.content.front.date.as_ref();
                        PostEntry {
                            title: post.content.front.title.clone(),
                            href: links.page(&post.path),
                            date: date.map(ToString::to_string).unwrap_or_default(),
                            date_text: date
                                .and_then(|date| date.date())
                                .map(|day| calendar::long_date(day, context.language))
                                .unwrap_or_default(),
                            summary_html: markdown(&post.content.front.summary),
                        }
                    })
                    .collect(),
            }
        }
        Block::Video(video) => match video
            .url
            .as_deref()
            .map(str::trim)
            .filter(|u| !u.is_empty())
        {
            // Without an address there's nothing to link to.
            None => RenderedBlock::Text {
                html: String::new(),
            },
            Some(url) => RenderedBlock::Video {
                href: url.to_string(),
                picture: video.image.as_deref().and_then(|image| {
                    picture(image, video.title.as_deref(), links, context.images)
                }),
                title: video.title.clone().filter(|t| !t.trim().is_empty()),
                platform: crate::video::host_label(url),
            },
        },
        Block::PageList(list) => RenderedBlock::PageList {
            compact: context.footer,
            pages: children(context.pages, list_source(&list.from, links))
                .iter()
                .map(|page| PageEntry {
                    title: page.content.front.title.clone(),
                    href: links.page(&page.path),
                    summary_html: markdown(&page.content.front.summary),
                })
                .collect(),
        },
    })
}

/// A list's `from` as a page path; empty means the page the list is on.
fn list_source<'a>(from: &'a Option<String>, links: &'a LinkContext) -> &'a str {
    from.as_deref()
        .map(|from| from.trim().trim_matches('/'))
        .filter(|from| !from.is_empty())
        .unwrap_or(links.page_path())
}

/// The pages one level below the page at `parent` that aren't posts, by
/// weight, then title. Titles compare by their slug, so `André` sorts next
/// to `Andreas` and not after `Z`.
fn children<'a>(pages: &[&'a Page], parent: &str) -> Vec<&'a Page> {
    let mut children: Vec<(String, &Page)> = pages
        .iter()
        .copied()
        .filter(|page| page.content.front.date.is_none())
        .filter(|page| {
            let rest = match parent {
                "" => Some(page.path.as_str()).filter(|path| !path.is_empty()),
                _ => page.path.strip_prefix(&format!("{parent}/")),
            };
            rest.is_some_and(|rest| !rest.contains('/'))
        })
        .map(|page| (slugify(&page.content.front.title), page))
        .collect();
    children.sort_by(|(a, page_a), (b, page_b)| {
        page_a
            .content
            .front
            .weight
            .cmp(&page_b.content.front.weight)
            .then_with(|| a.cmp(b))
            .then_with(|| page_a.path.cmp(&page_b.path))
    });
    children.into_iter().map(|(_, page)| page).collect()
}

/// An image ready for the page. Width and height let the browser reserve
/// space before the image loads, so the page doesn't jump; `srcset` lets it
/// load a smaller copy where a smaller one is enough.
fn picture(name: &str, alt: Option<&str>, links: &LinkContext, images: &Images) -> Option<Picture> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let src = media_link(name, links);
    let info = media_name(name).and_then(|file| images.info(file).map(|info| (file, info)));
    let srcset = info
        .as_ref()
        .filter(|(_, info)| !info.widths.is_empty())
        .map(|(file, info)| {
            let mut candidates: Vec<String> = info
                .widths
                .iter()
                .map(|&width| format!("{} {width}w", links.media(&images::copy_name(file, width))))
                .collect();
            candidates.push(format!("{src} {}w", info.width));
            candidates.join(", ")
        });
    Some(Picture {
        src,
        srcset,
        alt: alt.unwrap_or_default().to_string(),
        width: info.as_ref().map(|(_, info)| info.width),
        height: info.as_ref().map(|(_, info)| info.height),
    })
}

/// The width that large preview cards need for a sharp image, as Facebook
/// and LinkedIn recommend.
const SHARE_IMAGE_WIDTH: u32 = 1200;

/// The image link previews show (`og:image`): the one chosen in the page
/// settings, or else the first image on the page. Previews need a full
/// address, hence `base_url`. It is the smallest copy at least
/// [`SHARE_IMAGE_WIDTH`] wide, or else the original: a smaller copy would
/// look blurry on a large preview card.
fn share_image(page: &Page, base_url: &str, images: &Images) -> Option<ShareImage> {
    let (name, alt) = match page.content.front.image.as_deref() {
        Some(name) => (name, None),
        None => first_image(&page.content.blocks)?,
    };
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let base = base_url.trim_end_matches('/');
    let alt = alt.map(str::to_string).filter(|alt| !alt.trim().is_empty());

    let Some(file) = media_name(name) else {
        let url = if name.contains(':') {
            name.to_string()
        } else {
            format!("{base}{name}")
        };
        return Some(ShareImage {
            url,
            width: None,
            height: None,
            alt,
        });
    };
    let (file, width, height) = match images.info(file) {
        Some(info) => match info
            .widths
            .iter()
            .find(|&&width| width >= SHARE_IMAGE_WIDTH)
        {
            Some(&width) => (
                images::copy_name(file, width),
                Some(width),
                Some(images::scaled_height(info.width, info.height, width)),
            ),
            None => (file.to_string(), Some(info.width), Some(info.height)),
        },
        None => (file.to_string(), None, None),
    };
    Some(ShareImage {
        url: format!("{base}/{MEDIA_DIR}/{file}"),
        width,
        height,
        alt,
    })
}

/// The first image on a page, columns included, with its alt text.
fn first_image(blocks: &[Block]) -> Option<(&str, Option<&str>)> {
    block::all_blocks(blocks)
        .into_iter()
        .find_map(|block| match block {
            Block::Hero(hero) => hero.image.as_deref().map(|image| (image, None)),
            Block::Image(image) => image
                .image
                .as_deref()
                .map(|name| (name, image.alt.as_deref())),
            Block::TextImage(block) => block
                .image
                .as_deref()
                .map(|name| (name, block.alt.as_deref())),
            Block::Gallery(gallery) => gallery
                .images
                .first()
                .map(|image| (image.image.as_str(), image.alt.as_deref())),
            _ => None,
        })
        .filter(|(name, _)| !name.trim().is_empty())
}

/// The link to a download block's file and where it is on disk. Downloads
/// live in `assets/`; sites from before that kept them in `media/`, so a
/// file found only there is linked there.
fn download_location(name: &str, links: &LinkContext, root: &Path) -> (String, PathBuf) {
    let asset = root.join(ASSETS_DIR).join(name);
    let old = root.join(MEDIA_DIR).join(name);
    if !asset.is_file() && old.is_file() {
        (links.media(name), old)
    } else {
        (links.asset(name), asset)
    }
}

/// "1.4 MB", "820 KB", "12 bytes".
fn format_size(bytes: u64) -> String {
    match bytes {
        1_000_000.. => format!("{:.1} MB", bytes as f64 / 1_000_000.0),
        1_000.. => format!("{} KB", (bytes as f64 / 1_000.0).round()),
        _ => format!("{bytes} bytes"),
    }
}

/// The file inside `media/` an image field points to: `team.jpg` or
/// `/media/team.jpg`. Full URLs and other site paths aren't media files.
fn media_name(image: &str) -> Option<&str> {
    if image.contains(':') {
        None
    } else if let Some(name) = image.strip_prefix("/media/") {
        Some(name)
    } else if image.starts_with('/') {
        None
    } else {
        Some(image)
    }
}

/// Block images are file names in `media/`; full URLs and site paths work too.
fn media_link(image: &str, links: &LinkContext) -> String {
    if image.contains(':') || image.starts_with('/') {
        links.resolve(image)
    } else {
        links.media(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site;

    fn demo_site() -> Site {
        site::open(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/demo-site")).unwrap()
    }

    fn read(path: PathBuf) -> String {
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// A blog with three posts: one in the past, one today, one next year,
    /// plus a draft and a page without a date.
    fn blog(dir: &Path) -> Site {
        let write = |file: &str, text: &str| {
            let path = dir.join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        };
        write("site.toml", "title = \"Pod & Co\"\n");
        write(
            "pages/index.md",
            "+++\ntitle = \"Home\"\n+++\n\n::: post-list\nfrom = \"/blog\"\nlimit = 1\n:::\n",
        );
        write(
            "pages/blog.md",
            "+++\ntitle = \"Blog\"\ndescription = \"News\"\n+++\n\n::: post-list\n:::\n",
        );
        write(
            "pages/blog/2026-01-05-old.md",
            "+++\ntitle = \"Old <news>\"\ndate = 2026-01-05\nsummary = \"The *first* one.\"\nauthors = [\"Ann\"]\n+++\n",
        );
        write(
            "pages/blog/2026-10-21-today.md",
            "+++\ntitle = \"Today\"\ndate = 2026-10-21\n+++\n",
        );
        write(
            "pages/blog/2027-01-01-later.md",
            "+++\ntitle = \"Later\"\ndate = 2027-01-01\n+++\n",
        );
        write(
            "pages/blog/2026-02-02-draft.md",
            "+++\ntitle = \"Draft\"\ndate = 2026-02-02\ndraft = true\n+++\n",
        );
        write(
            "pages/blog/about.md",
            "+++\ntitle = \"About the blog\"\n+++\n",
        );
        site::open(dir).unwrap()
    }

    const TODAY: Date = Date {
        year: 2026,
        month: 10,
        day: 21,
    };

    #[test]
    fn lists_posts_newest_first_without_future_ones() {
        let dir = tempfile::tempdir().unwrap();
        let site = blog(dir.path());
        let out = dir.path().join("_site");
        build_on(&site, &out, TODAY).unwrap();

        let blog = read(out.join("blog/index.html"));
        let today = blog.find(">Today</a>").expect("today's post");
        let old = blog
            .find(">Old &#60;news&#62;</a>")
            .expect("older post, escaped");
        assert!(today < old, "newest first");
        assert!(blog.contains(r#"href="../blog/2026-01-05-old/""#), "{blog}");
        assert!(blog.contains("<em>first</em>"), "summary as Markdown");
        assert!(!blog.contains("Later"), "future post left out");
        assert!(!blog.contains(">Draft</a>"), "draft left out");
        assert!(
            !blog.contains("About the blog"),
            "page without a date left out"
        );

        let home = read(out.join("index.html"));
        assert!(home.contains(r#"href="blog/2026-10-21-today/""#), "{home}");
        assert!(!home.contains("Old"), "limit = 1");
    }

    #[test]
    fn writes_a_feed_with_a_base_url_and_says_why_not_without() {
        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        let out = dir.path().join("_site");

        let report = build_on(&site, &out, TODAY).unwrap();
        assert_eq!(report.feeds, 0);
        assert!(
            report.notes[0].contains("web address"),
            "{:?}",
            report.notes
        );
        assert!(!out.join("blog/feed.xml").exists());

        site.config.base_url = Some("https://example.com/".into());
        let report = build_on(&site, &out, TODAY).unwrap();
        assert_eq!(report.feeds, 1);
        assert!(report.notes.is_empty());
        let feed = read(out.join("blog/feed.xml"));
        assert!(
            feed.contains("<title>Blog · Pod &amp; Co</title>"),
            "{feed}"
        );
        assert!(feed.contains("<description>News</description>"), "{feed}");
        assert!(feed.contains(
            "<guid isPermaLink=\"true\">https://example.com/blog/2026-10-21-today/</guid>"
        ));
        assert!(
            feed.contains("<pubDate>Mon, 05 Jan 2026 00:00:00 +0000</pubDate>"),
            "{feed}"
        );
        assert!(feed.contains("<title>Old &lt;news&gt;</title>"), "{feed}");
        assert!(feed.contains("<dc:creator>Ann</dc:creator>"), "{feed}");
        assert!(!feed.contains("Later") && !feed.contains("Draft"), "{feed}");

        let blog_page = read(out.join("blog/index.html"));
        assert!(blog_page.contains(r#"href="../blog/feed.xml" class="link link-hover inline-flex items-center gap-1.5"><svg"#), "{blog_page}");
        assert!(blog_page.contains("Subscribe with RSS"), "{blog_page}");
        assert!(blog_page.contains("RSS feed</a></li>"), "footer link");
        let post = read(out.join("blog/2026-01-05-old/index.html"));
        assert!(
            post.contains(r#"<link rel="alternate" type="application/rss+xml" title="Blog · Pod &#38; Co" href="../../blog/feed.xml">"#),
            "{post}"
        );
    }

    #[test]
    fn builds_every_page_with_relative_links() {
        let out = tempfile::tempdir().unwrap();
        let out = out.path().join("_site");
        let report = build_into(&demo_site(), &out).unwrap();

        assert_eq!(report.pages, 4);
        for file in [
            "index.html",
            "about/index.html",
            "about/team/index.html",
            "imprint/index.html",
            "style.css",
        ] {
            assert!(out.join(file).is_file(), "{file} is missing");
        }

        let team = read(out.join("about/team/index.html"));
        assert!(
            team.contains(r#"href="../../style.css""#),
            "stylesheet link"
        );
        assert!(
            team.contains(r#"href="../../imprint/""#),
            "footer menu link"
        );
        assert!(team.contains(
            "<h1 class=\"text-4xl font-bold tracking-tight text-balance\">The team</h1>"
        ));
    }

    #[test]
    fn renders_blocks() {
        let out = tempfile::tempdir().unwrap();
        build_into(&demo_site(), out.path()).unwrap();
        let home = read(out.path().join("index.html"));

        assert!(home.contains("Two Mics and a Plan</h1>"), "hero headline");
        assert!(
            home.contains(r#"href="about/team/""#),
            "hero button resolved"
        );
        assert!(home.contains("mailto:hello@example.com"), "call to action");
        assert!(home.contains("<p>Every other week"), "markdown text");
        assert!(home.contains(r#"data-theme="light""#));
    }

    #[test]
    fn marks_the_current_menu_entry() {
        let out = tempfile::tempdir().unwrap();
        build_into(&demo_site(), out.path()).unwrap();
        let team = read(out.path().join("about/team/index.html"));
        assert!(
            team.contains(
                r#"class="inline-flex items-center gap-1.5 font-semibold text-primary">About</a>"#
            ),
            "{team}"
        );
    }

    #[test]
    fn skips_drafts_and_writes_redirects() {
        let mut site = demo_site();
        site.pages[1].content.front.redirect_from = vec!["old-about".into()];
        site.pages[3].content.front.draft = true;

        let out = tempfile::tempdir().unwrap();
        let report = build_into(&site, out.path()).unwrap();

        assert_eq!(report.drafts_skipped, 1);
        assert!(!out.path().join("imprint").exists());
        assert_eq!(report.redirects, 1);
        let redirect = read(out.path().join("old-about/index.html"));
        assert!(redirect.contains(r#"url=../about/"#), "{redirect}");
    }

    #[test]
    fn writes_a_sitemap_when_the_site_has_a_base_url() {
        let mut site = demo_site();
        site.config.base_url = Some("https://example.com/podcast/".into());
        let out = tempfile::tempdir().unwrap();
        build_into(&site, out.path()).unwrap();

        let xml = read(out.path().join("sitemap.xml"));
        assert!(xml.contains("<loc>https://example.com/podcast/</loc>"));
        assert!(xml.contains("<loc>https://example.com/podcast/about/team/</loc>"));
        let home = read(out.path().join("index.html"));
        assert!(home.contains(r#"<link rel="canonical" href="https://example.com/podcast/">"#));
    }

    #[test]
    fn puts_the_main_menu_into_a_sidebar_when_asked() {
        let mut site = demo_site();
        site.config.navigation = Navigation::Sidebar;
        let out = tempfile::tempdir().unwrap();
        build_into(&site, out.path()).unwrap();

        let team = read(out.path().join("about/team/index.html"));
        assert!(
            team.contains(r#"<nav class="hidden w-56 shrink-0 md:block" aria-label="Main menu">"#),
            "{team}"
        );
        assert!(team.contains(r#"<summary class="collapse-title font-medium">Menu</summary>"#));
        assert!(
            team.contains(r#"<a href="../../about/" class="font-semibold">About</a>"#),
            "parent page in bold: {team}"
        );
        let about = read(out.path().join("about/index.html"));
        assert!(
            about.contains(
                r#"<a href="../about/" class="menu-active" aria-current="page">About</a>"#
            ),
            "{about}"
        );
        assert!(team.contains("max-w-6xl"));

        // The default keeps the menu in the top bar.
        let out = tempfile::tempdir().unwrap();
        build_into(&demo_site(), out.path()).unwrap();
        let team = read(out.path().join("about/team/index.html"));
        assert!(!team.contains("aria-label=\"Main menu\""));
        assert!(team.contains("max-w-4xl"));
    }

    #[test]
    fn orders_menus_as_a_tree() {
        let mut site = demo_site();
        // "The team" (below About) joins the main menu, sorted first by
        // weight, but it still follows its parent.
        let team = site
            .pages
            .iter_mut()
            .find(|p| p.path == "about/team")
            .unwrap();
        team.content.front.menu = Menu::Main;
        team.content.front.weight = -5;
        let published: Vec<&Page> = site.pages.iter().collect();

        let entries = menu(&published, Menu::Main);
        let order: Vec<(&str, usize)> =
            entries.iter().map(|e| (e.path.as_str(), e.level)).collect();
        assert_eq!(order, [("about", 0), ("about/team", 1)]);

        site.config.navigation = Navigation::Sidebar;
        let out = tempfile::tempdir().unwrap();
        build_into(&site, out.path()).unwrap();
        let html = read(out.path().join("index.html"));
        assert!(
            html.contains(r#"<li class="ms-4 border-s border-base-300">"#),
            "{html}"
        );
    }

    #[test]
    fn shows_page_icons_in_menus() {
        let mut site = demo_site();
        let about = site.pages.iter_mut().find(|p| p.path == "about").unwrap();
        about.content.front.icon = Some("users".into());
        let out = tempfile::tempdir().unwrap();
        build_into(&site, out.path()).unwrap();
        let home = read(out.path().join("index.html"));
        let icon = crate::icons::svg("users").unwrap();
        assert!(home.contains(&format!("{icon}About</a>")), "{home}");
    }

    #[test]
    fn offers_smaller_copies_of_photos() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("pages")).unwrap();
        fs::create_dir_all(dir.path().join("media")).unwrap();
        fs::write(dir.path().join("site.toml"), "title = \"Photos\"\n").unwrap();
        fs::write(
            dir.path().join("pages/index.md"),
            "+++\ntitle = \"Home\"\n+++\n\n::: hero\nimage = \"wide.jpg\"\nheadline = \"Hi\"\n:::\n\n::: gallery\n[[images]]\nimage = \"wide.jpg\"\n\n[[images]]\nimage = \"small.png\"\n:::\n",
        )
        .unwrap();
        image::RgbImage::new(2000, 1000)
            .save(dir.path().join("media/wide.jpg"))
            .unwrap();
        image::RgbImage::new(300, 200)
            .save(dir.path().join("media/small.png"))
            .unwrap();

        let site = site::open(dir.path()).unwrap();
        let out = dir.path().join("_site");
        let report = build_into(&site, &out).unwrap();
        let home = read(out.join("index.html"));

        assert_eq!(report.resized_images, 3, "once per photo, not per use");
        assert!(out.join("media/wide.480w.jpg").is_file());
        assert!(home.contains(
            r#"srcset="media/wide.480w.jpg 480w, media/wide.960w.jpg 960w, media/wide.1600w.jpg 1600w, media/wide.jpg 2000w""#
        ));
        assert!(
            home.contains(r#"sizes="(min-width: 56rem) 18rem"#),
            "gallery sizes"
        );
        assert!(
            home.contains(r#"loading="eager""#),
            "the hero loads at once"
        );
        // Too small for copies: a plain image.
        assert!(
            home.contains(
                r#"<img src="media/small.png" alt="" width="300" height="200" loading="lazy""#
            ),
            "{home}"
        );
    }

    #[test]
    fn describes_pages_for_link_previews() {
        let dir = tempfile::tempdir().unwrap();
        let write = |name: &str, text: &str| {
            let path = dir.path().join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        };
        write("site.toml", "title = \"Pod & Co\"\n");
        write(
            "pages/index.md",
            "+++\ntitle = \"Home\"\n+++\n\n::: hero\nimage = \"wide.jpg\"\n:::\n",
        );
        write(
            "pages/news.md",
            "+++\ntitle = \"News\"\ndescription = \"What's new\"\ndate = 2026-10-22\nimage = \"small.png\"\n+++\n\n::: hero\nimage = \"wide.jpg\"\n:::\n",
        );
        write(
            "pages/cats.md",
            "+++\ntitle = \"Cats\"\n+++\n\nNo picture yet.\n\n:::: columns\n::: column\n:::\n::: image\nimage = \"small.png\"\nalt = \"A cat\"\n:::\n::::\n",
        );
        write("pages/plain.md", "+++\ntitle = \"Plain\"\n+++\n\nText.\n");
        write(
            "pages/logo.md",
            "+++\ntitle = \"Logo\"\nimage = \"medium.jpg\"\n+++\n",
        );
        fs::create_dir_all(dir.path().join("media")).unwrap();
        image::RgbImage::new(2000, 1000)
            .save(dir.path().join("media/wide.jpg"))
            .unwrap();
        image::RgbImage::new(300, 200)
            .save(dir.path().join("media/small.png"))
            .unwrap();
        image::RgbImage::new(1376, 768)
            .save(dir.path().join("media/medium.jpg"))
            .unwrap();

        // Without a web address there are no full URLs for previews.
        let mut site = site::open(dir.path()).unwrap();
        let out = dir.path().join("_site");
        build_into(&site, &out).unwrap();
        let home = read(out.join("index.html"));
        assert!(
            home.contains(r#"<meta property="og:title" content="Pod &#38; Co">"#),
            "{home}"
        );
        assert!(!home.contains("og:image"));

        site.config.base_url = Some("https://example.com".into());
        build_into(&site, &out).unwrap();

        // The first image, as its 1600 px copy.
        let home = read(out.join("index.html"));
        assert!(home.contains(r#"<meta property="og:type" content="website">"#));
        assert!(home.contains(r#"<meta property="og:url" content="https://example.com/">"#));
        assert!(home.contains(
            r#"<meta property="og:image" content="https://example.com/media/wide.1600w.jpg">"#
        ));
        assert!(home.contains(r#"<meta property="og:image:height" content="800">"#));
        assert!(home.contains(r#"<meta name="twitter:card" content="summary_large_image">"#));

        // The image from the settings wins; a post is an article.
        let news = read(out.join("news/index.html"));
        assert!(news.contains(r#"<meta property="og:type" content="article">"#));
        assert!(news.contains(r#"<meta property="og:title" content="News">"#));
        assert!(
            news.contains(r#"<meta property="og:description" content="What&#39;s new">"#),
            "{news}"
        );
        assert!(news.contains(
            r#"<meta property="og:image" content="https://example.com/media/small.png">"#
        ));
        assert!(news.contains(r#"<meta property="og:image:width" content="300">"#));

        // Found inside columns, with its alt text.
        let cats = read(out.join("cats/index.html"));
        assert!(
            cats.contains(r#"<meta property="og:image:alt" content="A cat">"#),
            "{cats}"
        );

        // Its 960 px copy would be too small for a large card: the original.
        let logo = read(out.join("logo/index.html"));
        assert!(logo.contains(
            r#"<meta property="og:image" content="https://example.com/media/medium.jpg">"#
        ));
        assert!(logo.contains(r#"<meta property="og:image:width" content="1376">"#));

        let plain = read(out.join("plain/index.html"));
        assert!(!plain.contains("og:image") && !plain.contains("twitter:card"));
    }

    #[test]
    fn shows_the_site_logo_in_the_header_and_the_browser_tab() {
        let mut site = demo_site();
        site.config.logo = Some("mark.svg".into());
        let out = tempfile::tempdir().unwrap();
        build_into(&site, out.path()).unwrap();

        let team = read(out.path().join("about/team/index.html"));
        assert!(
            team.contains(r#"<link rel="icon" href="../../media/mark.svg">"#),
            "{team}"
        );
        assert!(team.contains(
            r#"<img src="../../media/mark.svg" alt="" class="h-9 w-auto max-w-48 object-contain">"#
        ));

        site.config.logo = None;
        build_into(&site, out.path()).unwrap();
        assert!(!read(out.path().join("index.html")).contains("rel=\"icon\""));
    }

    #[test]
    fn prefers_a_favicon_of_its_own_as_the_tab_icon() {
        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        site.config.logo = Some("wide.svg".into());
        fs::create_dir_all(dir.path().join("static")).unwrap();
        fs::write(dir.path().join("static/favicon.ico"), b"ico").unwrap();
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();
        let blog = read(out.path().join("blog/index.html"));
        assert!(
            blog.contains(r#"<link rel="icon" href="../favicon.ico">"#),
            "{blog}"
        );
        assert!(
            blog.contains(r#"<img src="../media/wide.svg""#),
            "logo stays in the header: {blog}"
        );
    }

    #[test]
    fn leaves_out_the_title_above_a_hero_or_the_start_banner() {
        let page = |path: &str, block: &str| {
            let text = format!("+++\ntitle = \"T\"\n+++\n\n::: {block}\n:::\n");
            Page {
                path: path.to_string(),
                file: PathBuf::from("pages/x.md"),
                content: crate::page::parse(&text).unwrap(),
            }
        };
        assert!(!show_title(&page("about", "hero")));
        assert!(!show_title(&page("", "image")));
        assert!(
            show_title(&page("about", "image")),
            "a post may open with a photo"
        );
        assert!(show_title(&page("", "faq")));
    }

    #[test]
    fn renders_blocks_inside_columns() {
        let out = tempfile::tempdir().unwrap();
        build_into(&demo_site(), out.path()).unwrap();
        let home = read(out.path().join("index.html"));
        assert!(home.contains("grid gap-8 md:grid-cols-3"), "{home}");
        assert!(home.contains(">Join in</h2>"), "{home}");
        assert!(
            home.contains("Suggest a topic</a>"),
            "call to action inside a column"
        );
    }

    #[test]
    fn copies_static_files_as_they_are() {
        let dir = tempfile::tempdir().unwrap();
        let site = blog(dir.path());
        let files = dir.path().join(STATIC_DIR);
        fs::create_dir_all(files.join("pdf")).unwrap();
        fs::create_dir_all(files.join(".well-known")).unwrap();
        fs::write(files.join("keybase.txt"), "proof").unwrap();
        fs::write(files.join("pdf/a.pdf"), "%PDF").unwrap();
        fs::write(files.join(".well-known/security.txt"), "Contact: x").unwrap();
        fs::write(files.join("pdf/.DS_Store"), "").unwrap();

        let out = tempfile::tempdir().unwrap();
        let report = build_into(&site, out.path()).unwrap();
        assert_eq!(report.static_files, 3);
        assert_eq!(read(out.path().join("keybase.txt")), "proof");
        assert_eq!(read(out.path().join("pdf/a.pdf")), "%PDF");
        assert!(out.path().join(".well-known/security.txt").is_file());
        assert!(!out.path().join("pdf/.DS_Store").exists());
    }

    #[test]
    fn refuses_a_static_file_where_the_build_writes_one() {
        for clash in [
            "index.html",
            "blog/index.html",
            "style.css",
            ".qed-manifest.json",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let site = blog(dir.path());
            let file = dir.path().join(STATIC_DIR).join(clash);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, "mine").unwrap();

            let out = tempfile::tempdir().unwrap();
            let error = build_into(&site, out.path()).unwrap_err();
            assert!(
                matches!(&error, BuildError::StaticOverBuilt { path } if path == clash),
                "{clash}: {error}"
            );
        }
    }

    #[test]
    fn lists_the_pages_below_a_page() {
        let dir = tempfile::tempdir().unwrap();
        let write = |file: &str, text: &str| {
            let path = dir.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        };
        write("site.toml", "title = \"People\"\n");
        write("pages/index.md", "+++\ntitle = \"Home\"\n+++\n");
        write(
            "pages/people/index.md",
            "+++\ntitle = \"People\"\n+++\n\n::: page-list\n:::\n",
        );
        write(
            "pages/about.md",
            "+++\ntitle = \"About\"\n+++\n\n::: page-list\nfrom = \"/people\"\n:::\n",
        );
        write(
            "pages/people/zoe.md",
            "+++\ntitle = \"Zoë\"\nsummary = \"Hosts *Night Talk*\"\n+++\n",
        );
        write("pages/people/andreas.md", "+++\ntitle = \"Andreas\"\n+++\n");
        write("pages/people/andre.md", "+++\ntitle = \"André\"\n+++\n");
        write(
            "pages/people/first.md",
            "+++\ntitle = \"Team lead\"\nweight = -1\n+++\n",
        );
        write(
            "pages/people/news.md",
            "+++\ntitle = \"News\"\ndate = 2026-01-01\n+++\n",
        );
        write(
            "pages/people/gone.md",
            "+++\ntitle = \"Gone\"\ndraft = true\n+++\n",
        );
        write(
            "pages/people/zoe/photos.md",
            "+++\ntitle = \"Photos\"\n+++\n",
        );
        let site = site::open(dir.path()).unwrap();

        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();
        let people = read(out.path().join("people/index.html"));
        let order: Vec<usize> = ["Team lead", "André", "Andreas", "Zoë"]
            .iter()
            .map(|title| {
                people
                    .find(&format!(">{title}</a>"))
                    .unwrap_or_else(|| panic!("{title} missing:\n{people}"))
            })
            .collect();
        assert!(order.is_sorted(), "by weight, then title: {order:?}");
        for left_out in ["News", "Gone", "Photos"] {
            assert!(!people.contains(&format!(">{left_out}</a>")), "{left_out}");
        }
        assert!(people.contains(r#"href="../people/zoe/""#), "{people}");
        assert!(people.contains("Hosts <em>Night Talk</em>"), "{people}");

        let about = read(out.path().join("about/index.html"));
        assert!(about.contains(r#"href="../people/zoe/""#), "{about}");
    }

    #[test]
    fn adds_a_custom_theme_to_the_stylesheet() {
        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        site.config.theme = "custom".into();
        site.config.custom_theme = Some(crate::theme::CustomTheme {
            name: None,
            from: "nord".into(),
            background: None,
            primary: Some("#ff6600".into()),
            secondary: None,
            accent: None,
            neutral: None,
            corners: None,
        });
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();
        assert!(read(out.path().join("index.html")).contains(r#"data-theme="nord""#));
        let css = read(out.path().join("style.css"));
        assert!(
            css.ends_with("--color-primary-content: #1d232a;\n}\n"),
            "{}",
            &css[css.len() - 200..]
        );
    }

    #[test]
    fn writes_dates_in_the_site_language() {
        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        site.config.language = "de-AT".into();
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();

        let list = read(out.path().join("blog/index.html"));
        assert!(
            list.contains(r#"<time datetime="2026-01-05" class="badge badge-secondary badge-sm self-start">5. Jänner 2026</time>"#),
            "{list}"
        );
        let post = read(out.path().join("blog/2026-01-05-old/index.html"));
        assert!(post.contains("5. Jänner 2026 · Ann"), "{post}");
    }

    #[test]
    fn colors_the_header_bar_when_asked() {
        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        site.config.header = HeaderStyle::Primary;
        for page in &mut site.pages {
            if page.path == "blog" {
                page.content.front.menu = Menu::Main;
            }
        }
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();
        let blog = read(out.path().join("blog/index.html"));
        assert!(
            blog.contains(r#"<header class="bg-primary text-primary-content">"#),
            "{blog}"
        );
        assert!(
            blog.contains("underline decoration-2"),
            "current page underlined: {blog}"
        );

        site.config.header = HeaderStyle::Plain;
        build_on(&site, out.path(), TODAY).unwrap();
        let blog = read(out.path().join("blog/index.html"));
        assert!(blog.contains("border-t-accent"), "{blog}");
    }

    #[test]
    fn links_a_video_without_loading_anything_from_its_platform() {
        let dir = tempfile::tempdir().unwrap();
        let write = |file: &str, text: &str| {
            let path = dir.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        };
        write("site.toml", "title = \"S\"\n");
        write(
            "pages/index.md",
            "+++\ntitle = \"Home\"\n+++\n\n::: video\nurl = \"https://youtu.be/abc\"\ntitle = \"A talk\"\nimage = \"video-a-talk.jpg\"\n:::\n",
        );
        let thumbnail = image::RgbImage::new(16, 9);
        fs::create_dir_all(dir.path().join("media")).unwrap();
        thumbnail
            .save(dir.path().join("media/video-a-talk.jpg"))
            .unwrap();
        let site = site::open(dir.path()).unwrap();

        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();
        let home = read(out.path().join("index.html"));
        assert!(home.contains(r#"<a href="https://youtu.be/abc""#), "{home}");
        assert!(home.contains(r#"src="media/video-a-talk.jpg""#), "{home}");
        assert!(home.contains("A talk</span> · YouTube"), "{home}");
        assert!(
            !home.contains("<iframe") && !home.contains("<script"),
            "{home}"
        );
    }

    #[test]
    fn shows_the_footer_blocks_on_every_page() {
        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        let footer = "+++\ntitle = \"Footer\"\n+++\n\n:::: columns\n::: column\ntitle = \"Letzte Posts\"\n:::\n\n::: post-list\nfrom = \"/blog\"\nlimit = 2\n:::\n::: column\ntitle = \"Kontakt\"\n:::\n\n[Über uns](/blog/about)\n::::\n";
        fs::write(dir.path().join(site::FOOTER_FILE), footer).unwrap();
        site = site::open(&site.root).unwrap();
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();

        let post = read(out.path().join("blog/2026-01-05-old/index.html"));
        let footer = &post[post.find("<footer").unwrap()..];
        assert!(footer.contains("site-footer"), "{footer}");
        assert!(footer.contains("Letzte Posts"), "{footer}");
        assert!(
            footer.contains(r#"<li><a href="../../blog/2026-10-21-today/" class="link link-hover">Today</a></li>"#),
            "compact post list with this page's links: {footer}"
        );
        assert!(
            !footer.contains("badge"),
            "no dates in the footer: {footer}"
        );
        assert!(footer.contains(r#"href="../../blog/about/""#), "{footer}");
        let home = read(out.path().join("index.html"));
        assert!(
            home[home.find("<footer").unwrap()..].contains(r#"href="blog/about/""#),
            "{home}"
        );
    }

    #[test]
    fn writes_redirects_from_old_html_addresses_as_files() {
        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        for page in &mut site.pages {
            if page.path == "blog/2026-01-05-old" {
                page.content.front.redirect_from = vec![
                    "/2026/01/05/old.html".into(),
                    "/old.html".into(),
                    "/news".into(),
                ];
            }
        }
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();
        let deep = read(out.path().join("2026/01/05/old.html"));
        assert!(deep.contains("../../../blog/2026-01-05-old/"), "{deep}");
        let top = read(out.path().join("old.html"));
        assert!(
            top.contains(r#"url=blog/2026-01-05-old/""#) || top.contains("blog/2026-01-05-old/"),
            "{top}"
        );
        assert!(out.path().join("news/index.html").is_file());
    }

    #[test]
    fn replaces_an_earlier_build() {
        let out = tempfile::tempdir().unwrap();
        build_into(&demo_site(), out.path()).unwrap();
        fs::write(out.path().join("leftover.html"), "old").unwrap();

        build_into(&demo_site(), out.path()).unwrap();
        assert!(!out.path().join("leftover.html").exists());
    }

    #[test]
    fn refuses_to_overwrite_a_folder_it_did_not_build() {
        let out = tempfile::tempdir().unwrap();
        fs::write(out.path().join("precious.txt"), "keep me").unwrap();

        let error = build_into(&demo_site(), out.path()).unwrap_err();
        assert!(matches!(error, BuildError::NotOurs(_)));
        assert!(out.path().join("precious.txt").exists());
    }

    #[test]
    fn builds_the_docs_site() {
        let docs = site::open(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs")).unwrap();
        let out = tempfile::tempdir().unwrap();
        build_into(&docs, out.path()).unwrap();

        let post = read(
            out.path()
                .join("blog/2026-10-21-reading-a-site-folder/index.html"),
        );
        // A link to another post, written as a .md file, now points at its page.
        assert!(post.contains(r#"href="../../blog/2026-10-21-one-title-three-languages/""#));
    }
}

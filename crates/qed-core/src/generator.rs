//! Builds the static website from a [`Site`].
//!
//! The output goes to `_site/` in the site folder:
//!
//! ```text
//! _site/
//!   index.html            the start page
//!   about/index.html      /about/
//!   style.css             one stylesheet for every site and theme
//!   media/…               copied from the site's media folder
//!   sitemap.xml           only if `base_url` is set
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use askama::Template;
use serde::Serialize;

use crate::block::{Block, Side};
use crate::links::LinkContext;
use crate::markdown;
use crate::page::Menu;
use crate::site::{MEDIA_DIR, Navigation, Page, Site, SiteConfig};

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
    pub media_files: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("can't write {}: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error(
        "{} already exists and wasn't built by qed; move it away so qed doesn't overwrite your files",
        .0.display()
    )]
    NotOurs(PathBuf),

    #[error("rendering /{page}: {source}")]
    Template { page: String, source: askama::Error },

    #[error("/{page} wants a redirect from /{from}, but a page already lives there")]
    RedirectOverPage { page: String, from: String },
}

/// Builds the site into `_site/` inside its folder.
pub fn build(site: &Site) -> Result<BuildReport, BuildError> {
    build_into(site, &site.root.join(OUTPUT_DIR))
}

/// Builds the site into `out_dir`, replacing an earlier build there.
pub fn build_into(site: &Site, out_dir: &Path) -> Result<BuildReport, BuildError> {
    prepare_output(out_dir)?;

    let published: Vec<&Page> = site
        .pages
        .iter()
        .filter(|p| !p.content.front.draft)
        .collect();
    let media_dir = site.root.join(MEDIA_DIR);
    let main_menu = menu(&published, Menu::Main);
    let footer_menu = menu(&published, Menu::Footer);

    for page in &published {
        let html = render_page(&site.config, &media_dir, page, &main_menu, &footer_menu)?;
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
            let links = LinkContext::new(from, Path::new(""));
            let target = links.page(&page.path);
            let html = RedirectTemplate { target: &target }
                .render()
                .map_err(|source| BuildError::Template {
                    page: page.path.clone(),
                    source,
                })?;
            write(&page_file(out_dir, from), html)?;
            redirects += 1;
        }
    }

    write(&out_dir.join("style.css"), STYLESHEET.to_string())?;

    let media_files = copy_dir(&site.root.join(MEDIA_DIR), &out_dir.join(MEDIA_DIR))?;

    if let Some(base_url) = &site.config.base_url {
        write(&out_dir.join("sitemap.xml"), sitemap(base_url, &published))?;
    }

    Ok(BuildReport {
        out_dir: out_dir.to_path_buf(),
        pages: published.len(),
        drafts_skipped: site.pages.len() - published.len(),
        redirects,
        media_files,
    })
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
    is_home: bool,
    /// Relative path to the site root, for the stylesheet.
    root: String,
    home_href: String,
    main_menu: Vec<MenuLink>,
    footer_menu: Vec<MenuLink>,
    /// Pages that start with a hero use its headline instead of a title.
    show_title: bool,
    /// The main menu goes into a sidebar instead of the top bar.
    sidebar: bool,
    /// The content width; wider with a sidebar.
    width: &'static str,
    /// Date and authors, e.g. "2026-10-21 · Claude".
    byline: String,
    blocks: Vec<RenderedBlock>,
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

/// A block, ready for the template: Markdown rendered, links resolved.
enum RenderedBlock {
    Text {
        html: String,
    },
    Hero {
        image: Option<String>,
        headline: Option<String>,
        text_html: Option<String>,
        button: Option<Button>,
    },
    Image {
        /// `None` while no image is chosen; nothing is shown then.
        src: Option<String>,
        alt: String,
        caption_html: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
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
}

struct Picture {
    src: String,
    alt: String,
    width: Option<u32>,
    height: Option<u32>,
}

struct ColumnEntry {
    title: Option<String>,
    text_html: String,
}

struct FaqEntry {
    question: String,
    answer_html: String,
}

fn render_page(
    config: &SiteConfig,
    media_dir: &Path,
    page: &Page,
    main_menu: &[MenuEntry],
    footer_menu: &[MenuEntry],
) -> Result<String, BuildError> {
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
        byline.push(date.to_string());
    }
    if !front.authors.is_empty() {
        byline.push(front.authors.join(", "));
    }

    let canonical = config.base_url.as_ref().map(|base| {
        let base = base.trim_end_matches('/');
        if page.path.is_empty() {
            format!("{base}/")
        } else {
            format!("{base}/{}/", page.path)
        }
    });

    let sidebar = config.navigation == Navigation::Sidebar && !main_menu.is_empty();
    let template = PageTemplate {
        site: config,
        title: &front.title,
        description: front.description.as_deref(),
        canonical,
        is_home: page.path.is_empty(),
        root: links.to_root(),
        home_href: links.page(""),
        main_menu: menu_links(main_menu),
        footer_menu: menu_links(footer_menu),
        show_title: !matches!(page.content.blocks.first(), Some(Block::Hero(_))),
        sidebar,
        width: if sidebar { "max-w-6xl" } else { "max-w-4xl" },
        byline: byline.join(" · "),
        blocks: page
            .content
            .blocks
            .iter()
            .map(|b| render_block(b, &links, media_dir))
            .collect(),
    };

    template.render().map_err(|source| BuildError::Template {
        page: page.path.clone(),
        source,
    })
}

/// A menu entry is current on its own page and on the pages below it, except
/// for the start page, which would otherwise be current everywhere.
fn is_within(page: &str, entry: &str) -> bool {
    page == entry || (!entry.is_empty() && page.starts_with(&format!("{entry}/")))
}

fn render_block(block: &Block, links: &LinkContext, media_dir: &Path) -> RenderedBlock {
    let markdown = |text: &Option<String>| text.as_deref().map(|t| markdown::render(t, links));
    let button = |label: &Option<String>, link: &Option<String>| match (label, link) {
        (Some(label), Some(link)) => Some(Button {
            label: label.clone(),
            href: links.resolve(link),
        }),
        _ => None,
    };

    match block {
        Block::Text { markdown: text } => RenderedBlock::Text {
            html: markdown::render(text, links),
        },
        Block::Hero(hero) => RenderedBlock::Hero {
            image: hero.image.as_deref().map(|image| media_link(image, links)),
            headline: hero.headline.clone(),
            text_html: markdown(&hero.text),
            button: button(&hero.button_label, &hero.button_link),
        },
        Block::Image(image) => {
            let file = image
                .image
                .as_deref()
                .filter(|name| !name.trim().is_empty());
            // Width and height let the browser reserve space before the image
            // loads, so the page doesn't jump. Only the file header is read.
            let size = file.and_then(|name| image::image_dimensions(media_dir.join(name)).ok());
            RenderedBlock::Image {
                src: file.map(|name| media_link(name, links)),
                alt: image.alt.clone().unwrap_or_default(),
                caption_html: markdown(&image.caption),
                width: size.map(|(width, _)| width),
                height: size.map(|(_, height)| height),
            }
        }
        Block::TextImage(block) => RenderedBlock::TextImage {
            text_html: markdown(&block.text),
            picture: block
                .image
                .as_deref()
                .and_then(|name| picture(name, block.alt.as_deref(), links, media_dir)),
            image_left: block.image_side == Side::Left,
        },
        Block::Gallery(gallery) => RenderedBlock::Gallery {
            pictures: gallery
                .images
                .iter()
                .filter_map(|image| picture(&image.image, image.alt.as_deref(), links, media_dir))
                .collect(),
        },
        Block::Download(download) => {
            let file = download
                .file
                .as_deref()
                .map(str::trim)
                .filter(|name| !name.is_empty());
            RenderedBlock::Download {
                href: file.map(|name| media_link(name, links)),
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
                size: file
                    .and_then(|name| std::fs::metadata(media_dir.join(name)).ok())
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
                .map(|column| ColumnEntry {
                    title: column
                        .title
                        .clone()
                        .filter(|title| !title.trim().is_empty()),
                    text_html: markdown::render(&column.text, links),
                })
                .collect(),
        },
        Block::CallToAction(cta) => RenderedBlock::CallToAction {
            headline: cta.headline.clone(),
            text_html: markdown(&cta.text),
            button: button(&cta.button_label, &cta.button_link),
        },
    }
}

/// An image ready for the page. Width and height let the browser reserve
/// space before the image loads, so the page doesn't jump; only the file's
/// header is read to get them.
fn picture(
    name: &str,
    alt: Option<&str>,
    links: &LinkContext,
    media_dir: &Path,
) -> Option<Picture> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let size = image::image_dimensions(media_dir.join(name)).ok();
    Some(Picture {
        src: media_link(name, links),
        alt: alt.unwrap_or_default().to_string(),
        width: size.map(|(width, _)| width),
        height: size.map(|(_, height)| height),
    })
}

/// "1.4 MB", "820 KB", "12 bytes".
fn format_size(bytes: u64) -> String {
    match bytes {
        1_000_000.. => format!("{:.1} MB", bytes as f64 / 1_000_000.0),
        1_000.. => format!("{} KB", (bytes as f64 / 1_000.0).round()),
        _ => format!("{bytes} bytes"),
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

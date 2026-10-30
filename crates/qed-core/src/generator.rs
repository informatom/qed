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
//!                         copies of photos (`team.480w.webp`, see images.rs)
//!   assets/…              copied from the site's assets folder (downloads)
//!   pdf/a.pdf, …          the site's static folder, copied as it is
//!   sitemap.xml           only if `base_url` is set
//!   blog/feed.xml         an RSS feed for each page with posts below it,
//!                         also only with `base_url`
//! ```

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use askama::Template;
use serde::Serialize;

use crate::block::{self, Block, ListLayout, Side};
use crate::calendar::{self, Date};
use crate::deploy::MANIFEST_FILE;
use crate::fonts;
use crate::icons::SiteIcons;
use crate::images::{self, ImageError, Images};
use crate::links::LinkContext;
use crate::markdown;
use crate::markdown::TextMedia;
use crate::media::CLUTTER;
use crate::page::{FrontMatter, Menu, PageContent};
use crate::podcast;
use crate::site::{
    ASSETS_DIR, Colors, HeaderStyle, MEDIA_DIR, Navigation, Page, STATIC_DIR, Site, SiteConfig,
    Surface, ThemeColor,
};
use crate::site::{LogoSize, PageWidth};
use crate::slug::slugify;

pub const OUTPUT_DIR: &str = "_site";

/// Marks a folder as built by qed, so it may be emptied on the next build.
const MARKER: &str = ".qed-build";

/// The stylesheet, compiled by `npm run build:site-css` and built into qed.
pub(crate) const STYLESHEET: &str = include_str!("../assets/site.css");
/// For sites with formulas: pulldown-latex's stylesheet and the Latin
/// Modern Math font; see `assets/pulldown-latex/README.md`.
const MATH_CSS: &str = include_str!("../assets/pulldown-latex/math.css");
const MATH_FONT: &[u8] = include_bytes!("../assets/pulldown-latex/latinmodern-math.woff2");

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
    /// Pages in the search index; 0 with search switched off.
    pub searchable_pages: usize,
    /// Things worth knowing that didn't stop the build, such as a feed
    /// left out.
    pub notes: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error(transparent)]
    Search(#[from] crate::search::SearchError),

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

    #[error(
        "{}: the link \"{link}\" leads outside the site folder, where the website can't follow it; link to a page or file inside the site, or to a full web address",
        file.display()
    )]
    LinkOutside { file: PathBuf, link: String },

    #[error(transparent)]
    Image(#[from] ImageError),
}

/// Refuses links that leave the site folder (`../../src/lib.rs`): they'd
/// be broken on the website. Drafts aren't built, so they may still have
/// them.
fn check_links(site: &Site) -> Result<(), BuildError> {
    for page in site.with_blocks().filter(|page| !page.content.front.draft) {
        let links = LinkContext::new(&page.path, &page.file);
        if let Some(link) = crate::site::link_targets(&page.content.blocks)
            .into_iter()
            .find(|link| links.leaves_site(link))
        {
            return Err(BuildError::LinkOutside {
                file: page.file.clone(),
                link,
            });
        }
    }
    Ok(())
}

/// Notes links to pages and files that don't exist (#133): a mistyped
/// `other-post.md`, a page that was deleted, a picture that isn't in
/// Media. They don't stop the build (a typo shouldn't keep a site from
/// being published), but each one would be a "page not found" for
/// visitors. `site` has its shared blocks expanded, so their links are
/// checked where they're shown, with each page's own links.
fn missing_links(site: &Site, published: &[&Page]) -> Vec<String> {
    let mut exists: BTreeSet<&str> = published.iter().map(|page| page.path.as_str()).collect();
    for page in published {
        exists.extend(
            page.content
                .front
                .redirect_from
                .iter()
                .map(|from| from.trim_matches('/')),
        );
    }
    let checked = site
        .pages
        .iter()
        .filter(|page| !page.content.front.draft)
        .chain(site.footer.as_ref())
        .chain(site.episode_layout.as_ref());
    let mut notes = Vec::new();
    for page in checked {
        let links = LinkContext::new(&page.path, &page.file);
        let place = match page.path.as_str() {
            crate::site::FOOTER_PATH => "The footer".to_string(),
            crate::site::EPISODE_LAYOUT_PATH => "The episode layout".to_string(),
            path => format!("/{path}"),
        };
        let mut seen = BTreeSet::new();
        for link in crate::site::link_targets(&page.content.blocks) {
            if !seen.insert(link.clone()) {
                continue;
            }
            let problem = if let Some(path) = links.page_path_of(&link) {
                let path = crate::preview::percent_decode(&path);
                // A folder in `static/` is served as it is, like a page.
                let in_static = !path.is_empty() && site.root.join(STATIC_DIR).join(&path).exists();
                if exists.contains(path.as_str()) || in_static {
                    continue;
                }
                if site
                    .pages
                    .iter()
                    .any(|p| p.path == path && p.content.front.draft)
                {
                    "that page is a draft and not on the website yet"
                } else {
                    "there's no such page"
                }
            } else if let Some(missing) = missing_file(&site.root, &link) {
                missing
            } else {
                continue;
            };
            notes.push(format!("{place} links to \"{link}\", but {problem}."));
        }
    }
    notes
}

/// Entries of a hand-made menu whose page isn't on the website: the menu
/// leaves them out, and this says so.
fn menu_notes(site: &Site, published: &[&Page], items: &[crate::site::MenuItem]) -> Vec<String> {
    let mut notes = Vec::new();
    for item in items {
        if let Some(written) = &item.page {
            let path = crate::site::menu_page_path(written);
            if !published.iter().any(|page| page.path == path) {
                let problem = if site
                    .pages
                    .iter()
                    .any(|p| p.path == path && p.content.front.draft)
                {
                    "that page is a draft and not on the website yet"
                } else {
                    "there's no such page"
                };
                notes.push(format!(
                    "The menu has an entry for \"/{path}\", but {problem}; it's left out."
                ));
            }
        }
        notes.extend(menu_notes(site, published, &item.items));
    }
    notes
}

/// For a link to `/media/…` or `/assets/…` (as written in text), what's
/// wrong if that file isn't there; `None` if it is, or the link is to
/// something else.
fn missing_file(root: &Path, link: &str) -> Option<&'static str> {
    let target = link.split(['#', '?']).next().unwrap_or_default();
    let target = crate::preview::percent_decode(target);
    let media = format!("/{MEDIA_DIR}/");
    let assets = format!("/{ASSETS_DIR}/");
    let (name, folders, problem): (&str, &[&str], _) = match target.strip_prefix(&media) {
        Some(name) => (name, &[MEDIA_DIR], "that file isn't in Media"),
        // Anything else but a download isn't checked. Downloads still in
        // `media/` work too (see `download_location`).
        None => (
            target.strip_prefix(&assets)?,
            &[ASSETS_DIR, MEDIA_DIR],
            "that file isn't in Downloads",
        ),
    };
    let found = folders
        .iter()
        .any(|folder| root.join(folder).join(name).is_file());
    (!found).then_some(problem)
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
    check_links(site)?;
    prepare_output(out_dir)?;

    // Shared blocks become their pieces' blocks; the rest of the build
    // sees only ordinary blocks.
    let mut notes = Vec::new();
    let expanded = crate::shared::expanded(site, &mut notes);
    let site = &expanded;

    // A page at `404` is the 404 page (see `not_found_page`), not one of
    // the site's pages.
    // A podcast's contributors each get a page (see `contributor_pages`).
    let people_pages = contributor_pages(site);
    let published: Vec<&Page> = site
        .pages
        .iter()
        .filter(|p| !p.content.front.draft && p.path != NOT_FOUND_PATH)
        .chain(&people_pages)
        .collect();
    notes.extend(missing_links(site, &published));
    notes.extend(menu_notes(site, &published, &site.config.main_menu));
    let media_dir = site.root.join(MEDIA_DIR);
    let images = Images::new(&media_dir);
    let site_icons = SiteIcons::load(&media_dir);
    let main_menu = menu(&published, Menu::Main);
    let footer_menu = menu(&published, Menu::Footer);
    // Feeds need full addresses, so only a site with a web address gets them.
    // A podcast's episodes get podcast feeds instead (see `feed.rs`).
    let podcast = site.config.podcast.as_ref();
    let sources: Vec<&Page> = feed_sources(&published, today)
        .into_iter()
        .filter(|page| podcast.is_none() || page.path != podcast::EPISODES_PATH)
        .collect();
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
    // The podcast feeds, one per format, the main one first.
    let podcast_feeds: Vec<PodcastFeed> = match (podcast, &site.config.base_url) {
        (Some(podcast), Some(_)) => podcast
            .formats()
            .into_iter()
            .map(|format| PodcastFeed {
                file: podcast::feed_file(format),
                label: if podcast.formats().len() > 1 {
                    format!("Podcast feed ({})", format.label())
                } else {
                    "Podcast feed".to_string()
                },
            })
            .collect(),
        (Some(_), None) => {
            notes.push(
                "No podcast feed: feeds need the site's web address, under Site → Settings."
                    .to_string(),
            );
            Vec::new()
        }
        _ => Vec::new(),
    };
    let dark = crate::theme::is_dark(STYLESHEET, site.config.page_theme(), site.config.custom());
    let colors = color_classes(&site.config.colors, dark);
    let button_color = crate::theme::color_hex(
        STYLESHEET,
        site.config.page_theme(),
        site.config.custom(),
        match site.config.colors.buttons.unwrap_or(ThemeColor::Primary) {
            ThemeColor::Primary => "--color-primary",
            ThemeColor::Secondary => "--color-secondary",
            ThemeColor::Accent => "--color-accent",
            ThemeColor::Neutral => "--color-neutral",
        },
    );
    let context = Context {
        podcast_feeds: &podcast_feeds,
        podcast,
        button_color: button_color.as_deref(),
        contributors: &site.contributors,
        colors: &colors,
        root: &site.root,
        images: &images,
        site_icons: &site_icons,
        pages: &published,
        feeds: &feeds,
        today,
        language: &site.config.language,
        footer: false,
    };

    let mut players = 0;
    // Whether any page has a formula, for the math font (see below).
    let mut uses_math = false;
    for page in &published {
        let html = render_page(site, &context, page, &main_menu, &footer_menu, &feeds)?;
        uses_math |= html.contains("<math");
        write(&page_file(out_dir, &page.path), html)?;
        // The player's data for an episode with audio, next to its page.
        if let Some(episode) = page.content.front.episode.as_ref()
            && site.config.podcast.is_some()
            && !episode.audio.is_empty()
        {
            // Addresses from the site root, not the page: the embedded
            // player (share.html) reads this file from elsewhere.
            let links = LinkContext::rooted(&site_root_path(&site.config), &page.file);
            let data = player_episode(site, page, episode, &links);
            write(
                &out_dir.join(&page.path).join(PLAYER_EPISODE),
                data.to_string(),
            )?;
            players += 1;
        }
    }
    if players > 0 {
        write_player(out_dir, site, &published, today, button_color.as_deref())?;
    }

    let not_found = not_found_page(site);
    let html = render_page(site, &context, &not_found, &main_menu, &footer_menu, &feeds)?;
    write(&out_dir.join("404.html"), html)?;

    let mut redirects = 0;
    let theme_hex = |variable: &str, fallback: &str| {
        crate::theme::color_hex(
            STYLESHEET,
            site.config.page_theme(),
            site.config.custom(),
            variable,
        )
        .unwrap_or_else(|| fallback.to_string())
    };
    let redirect_background = theme_hex("--color-base-100", "#ffffff");
    let redirect_text = theme_hex("--color-base-content", "#1d232a");
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
            let html = RedirectTemplate {
                target: &target,
                language: &site.config.language,
                background: &redirect_background,
                text: &redirect_text,
                dark,
            }
            .render()
            .map_err(|source| BuildError::Template {
                page: page.path.clone(),
                source,
            })?;
            write(&file, html)?;
            redirects += 1;
        }
    }

    // Colors for highlighted code, a custom theme and the site's fonts add
    // to the one stylesheet; the
    // fonts in use are copied next to it.
    let custom_css = site.config.custom().map(|c| c.css()).unwrap_or_default();
    let fonts_css = fonts::css(&site.root, &site.config.fonts);
    // Formulas get LaTeX's look, on sites that have any.
    let math_css = if uses_math {
        write(&out_dir.join("math/latinmodern-math.woff2"), MATH_FONT)?;
        MATH_CSS
    } else {
        ""
    };
    write(
        &out_dir.join("style.css"),
        format!(
            "{STYLESHEET}{}{custom_css}{fonts_css}{math_css}",
            markdown::code_css()
        ),
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
                "The font \"{family}\" isn't downloaded; the site uses the system font. Download it under Theme → Fonts."
            ));
        }
    }

    notes.extend(cover_notes(site, &published, &media_dir));
    notes.extend(audio_notes(site, &published));

    let media_files = copy_dir(&media_dir, &out_dir.join(MEDIA_DIR))?
        + copy_dir(&site.root.join(ASSETS_DIR), &out_dir.join(ASSETS_DIR))?
        + match &site.config.podcast {
            Some(_) => {
                copy_or_link_dir(
                    &site.root.join(podcast::AUDIO_DIR),
                    &out_dir.join(podcast::AUDIO_URL_DIR),
                    true,
                )? + copy_dir(
                    &site.root.join(podcast::TRANSCRIPTS_DIR),
                    &out_dir.join(podcast::TRANSCRIPTS_DIR),
                )?
            }
            None => 0,
        };
    let resized_images =
        images.write(&out_dir.join(MEDIA_DIR), &site.root.join(images::CACHE_DIR))?;

    let mut podcast_feed_count = 0;
    if let (Some(podcast), Some(base_url)) = (podcast, &site.config.base_url) {
        let base = base_url.trim_end_matches('/');
        let episodes = crate::feed::feed_episodes(&published, today);
        let audio_dir = site.root.join(podcast::AUDIO_DIR);
        let shownotes = |page: &Page| {
            // Text only, with full links: feed readers show plain HTML.
            let links =
                LinkContext::rooted(&format!("{base}/"), &page.file).with_own_site(Some(base_url));
            block::all_blocks(&page.content.blocks)
                .into_iter()
                .filter_map(|block| match block {
                    Block::Text { markdown } => {
                        Some(markdown::render(markdown, &links, &context.text_media()))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        let local_length = |file: &str| fs::metadata(audio_dir.join(file)).ok().map(|m| m.len());
        let input = crate::feed::FeedInput {
            config: &site.config,
            podcast,
            base,
            episodes: &episodes,
            shownotes: &shownotes,
            local_length: &local_length,
            contributors: &site.contributors,
        };
        for feed in crate::feed::podcast_feeds(&input) {
            write(&out_dir.join(&feed.file), feed.xml)?;
            podcast_feed_count += 1;
        }
        // The share row's script, if the episode layout has the row.
        let shares = site
            .episode_layout
            .as_ref()
            .is_some_and(|layout| layout.content.blocks.contains(&Block::EpisodeShare));
        if shares
            && published
                .iter()
                .any(|page| page.content.front.episode.is_some())
        {
            write(&out_dir.join(SHARE_FILE), SHARE_SCRIPT)?;
        }
        let parts = site.footer.iter().chain(&site.episode_layout);
        if uses_subscribe(published.iter().copied().chain(parts)) {
            write_subscribe_button(out_dir, &site.config, podcast, base)?;
        }
    }

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

    // The search index reads the finished pages, so it comes after them.
    let searchable_pages = match site.config.search {
        true => crate::search::index(out_dir)?,
        false => 0,
    };

    Ok(BuildReport {
        out_dir: out_dir.to_path_buf(),
        pages: published.len(),
        // The contributors' pages are made by the build, not the site's.
        drafts_skipped: site.pages.len() - (published.len() - people_pages.len()),
        redirects,
        media_files,
        resized_images,
        feeds: feeds.len() + podcast_feed_count,
        static_files,
        searchable_pages,
        notes,
    })
}

/// What's wrong with the podcast's cover and the episodes' own covers by
/// Apple's rules. Apps show them anyway, but Apple may refuse the podcast.
fn cover_notes(site: &Site, published: &[&Page], media_dir: &Path) -> Vec<String> {
    let Some(podcast) = &site.config.podcast else {
        return Vec::new();
    };
    let mut notes = Vec::new();
    match podcast.cover.as_deref().and_then(media_name) {
        Some(cover) => notes.extend(
            podcast::check_cover(media_dir, cover)
                .into_iter()
                .map(|problem| format!("The podcast cover {cover}: {problem}.")),
        ),
        None => notes.push(
            "The podcast has no cover yet; podcast apps need one. Choose it in the Podcast tab."
                .to_string(),
        ),
    }
    for page in published {
        let Some(image) = page
            .content
            .front
            .episode
            .as_ref()
            .and_then(|episode| episode.image.as_deref())
            .and_then(media_name)
        else {
            continue;
        };
        notes.extend(
            podcast::check_cover(media_dir, image)
                .into_iter()
                .map(|problem| format!("/{}: the episode cover {image}: {problem}.", page.path)),
        );
    }
    notes
}

/// What's missing or doubtful about the episodes' audio: podcast apps
/// skip an episode without audio, and Publish waits until every episode
/// has every format.
fn audio_notes(site: &Site, published: &[&Page]) -> Vec<String> {
    let Some(podcast) = &site.config.podcast else {
        return Vec::new();
    };
    let mut notes = Vec::new();
    let main = podcast.main_format();
    if !main.plays_everywhere() {
        notes.push(format!(
            "The main format is {}; Apple Podcasts and Spotify play only MP3 and M4A. Put one of them first in the Podcast tab.",
            main.label()
        ));
    }
    let audio_dir = site.root.join(podcast::AUDIO_DIR);
    for page in published {
        let Some(episode) = page.content.front.episode.as_ref() else {
            continue;
        };
        if episode.audio.is_empty() {
            notes.push(format!("/{}: the episode has no audio yet.", page.path));
            continue;
        }
        let missing: Vec<&str> = episode
            .missing_formats(podcast)
            .iter()
            .map(|format| format.label())
            .collect();
        if !missing.is_empty() {
            notes.push(format!(
                "/{}: no {} audio yet; Publish waits until every episode has every format.",
                page.path,
                missing.join(", ")
            ));
        }
        if let Some(file) = episode.transcript.as_deref().map(str::trim)
            && !site
                .root
                .join(podcast::TRANSCRIPTS_DIR)
                .join(file)
                .is_file()
        {
            notes.push(format!(
                "/{}: the transcript {file} isn't in {}/.",
                page.path,
                podcast::TRANSCRIPTS_DIR
            ));
        }
        for file in episode.local_audio() {
            if !audio_dir.join(file).is_file() {
                notes.push(format!(
                    "/{}: the audio file {file} isn't in {}/.",
                    page.path,
                    podcast::AUDIO_DIR
                ));
            }
        }
    }
    notes
}

/// The URL path of a page that is the site's 404 page (`pages/404.md`).
const NOT_FOUND_PATH: &str = "404";

/// The page servers show for addresses that don't exist, written as
/// `404.html`: the site's own `pages/404.md` if it has one, else a short
/// page in the site's language with a link to the start page.
fn not_found_page(site: &Site) -> Page {
    if let Some(page) = site
        .pages
        .iter()
        .find(|page| page.path == NOT_FOUND_PATH && !page.content.front.draft)
    {
        return page.clone();
    }
    let language = site.config.language.to_lowercase();
    let (title, text) = match language.split(['-', '_']).next().unwrap_or_default() {
        "de" => (
            "Seite nicht gefunden",
            "Diese Seite gibt es nicht (mehr). [Zur Startseite](/)",
        ),
        "fr" => (
            "Page introuvable",
            "Cette page n'existe pas (ou plus). [Vers l'accueil](/)",
        ),
        "es" => (
            "Página no encontrada",
            "Esta página no existe (o ya no existe). [Ir al inicio](/)",
        ),
        "it" => (
            "Pagina non trovata",
            "Questa pagina non esiste (più). [Vai alla home](/)",
        ),
        "nl" => (
            "Pagina niet gevonden",
            "Deze pagina bestaat niet (meer). [Naar de startpagina](/)",
        ),
        _ => (
            "Page not found",
            "This page doesn't exist (any more). [Go to the start page](/)",
        ),
    };
    Page {
        path: NOT_FOUND_PATH.to_string(),
        file: PathBuf::from("pages/404.md"),
        content: crate::page::PageContent {
            front: crate::page::FrontMatter {
                title: title.to_string(),
                ..Default::default()
            },
            blocks: vec![Block::Text {
                markdown: text.to_string(),
            }],
        },
    }
}

/// The site's root as an absolute path: `/`, or the path of `base_url`
/// for a site in a subfolder (`https://example.org/blog/` gives `/blog/`).
fn site_root_path(config: &SiteConfig) -> String {
    let path = config
        .base_url
        .as_deref()
        .and_then(|base| reqwest::Url::parse(base).ok())
        .map(|url| url.path().to_string())
        .unwrap_or_default();
    format!("/{}", path.trim_matches('/').to_string() + "/").replace("//", "/")
}

/// Lucide's RSS icon, for feed links.
const RSS_ICON: &str = include_str!("../assets/icons/rss.svg");

/// What blocks need besides themselves.
#[derive(Clone, Copy)]
struct Context<'a> {
    /// The site folder.
    root: &'a Path,
    images: &'a Images,
    /// The site's own icons, for `:icon-name:` in text.
    site_icons: &'a SiteIcons,
    /// The pages being built, for Post lists.
    pages: &'a [&'a Page],
    /// The feeds being written, so a Post list can link its own.
    feeds: &'a [Feed<'a>],
    /// The podcast's feeds, the main format's first.
    podcast_feeds: &'a [PodcastFeed],
    /// The podcast's settings, for episode blocks.
    podcast: Option<&'a podcast::Podcast>,
    /// The buttons' theme color as `#rrggbb`, for the Subscribe button,
    /// which can't read the stylesheet's variables.
    button_color: Option<&'a str>,
    /// The podcast's contributors, for the Contributors block.
    contributors: &'a [crate::contributors::Contributor],
    today: Date,
    /// The site's language, for dates.
    language: &'a str,
    /// Rendering the footer: lists come out compact, as plain links.
    footer: bool,
    /// The site's color choices, as classes.
    colors: &'a ColorClasses,
}

impl Context<'_> {
    /// What Markdown text needs for icons and images.
    fn text_media(&self) -> TextMedia<'_> {
        TextMedia {
            site_icons: self.site_icons,
            images: self.images,
        }
    }
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
            // Episodes of one day: the higher number first.
            .then_with(|| podcast::same_day_order(page_b).cmp(&podcast::same_day_order(page_a)))
            .then_with(|| page_a.path.cmp(&page_b.path))
    });
    posts.into_iter().map(|(_, page)| page).collect()
}

/// The Podlove Subscribe Button, built into q.e.d. Where this copy comes
/// from and how to update it: `assets/subscribe-button/README.md`.
static SUBSCRIBE_BUTTON: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/assets/subscribe-button");

/// Where the button goes in the built site.
const SUBSCRIBE_DIR: &str = "subscribe-button";

/// The file with the podcast's data the button reads, next to it.
const SUBSCRIBE_DATA: &str = "podcast-data.js";

/// Writes the Subscribe button and the podcast's data for it into the
/// built site. The data goes into a file of its own rather than a
/// `<script>` in every page: `window.qedPodcastData = {…}`, the title,
/// cover and every feed, which the button hands to the listener's app.
fn write_subscribe_button(
    out_dir: &Path,
    config: &SiteConfig,
    podcast: &podcast::Podcast,
    base: &str,
) -> Result<(), BuildError> {
    let target = out_dir.join(SUBSCRIBE_DIR);
    write_embedded(&SUBSCRIBE_BUTTON, &target)?;
    let title = podcast.title.as_deref().unwrap_or(&config.title);
    let data = serde_json::json!({
        "title": title,
        "subtitle": podcast.author.as_deref().unwrap_or(""),
        "description": podcast.description.as_deref().or(config.description.as_deref()).unwrap_or(""),
        // A cover elsewhere as it is; one in media/ is set below.
        "cover": podcast.cover.as_deref().map(str::trim).filter(|c| c.contains("://")),
        "feeds": podcast.formats().iter().map(|format| serde_json::json!({
            "type": "audio",
            // The button's name for M4A is "aac".
            "format": match format {
                podcast::AudioFormat::M4a => "aac",
                other => other.extension(),
            },
            "url": format!("{base}/{}", podcast::feed_file(*format)),
            "variant": "high",
        })).collect::<Vec<_>>(),
    });
    let mut script = format!("window.qedPodcastData = {data};\n");
    // The cover's address from where this file is, so it works in the
    // preview as well as on the published site: `../media/cover.jpg`
    // from `subscribe-button/`, made a full address for the button's
    // pop-up.
    if let Some(cover) = podcast
        .cover
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty() && !c.contains("://"))
    {
        let name = cover.trim_start_matches('/');
        let name = name.strip_prefix("media/").unwrap_or(name);
        let relative = serde_json::Value::from(format!("../{MEDIA_DIR}/{name}"));
        script.push_str(&format!(
            "window.qedPodcastData.cover = new URL({relative}, document.currentScript.src).href;\n"
        ));
    }
    write(&target.join(SUBSCRIBE_DATA), script)
}

/// Writes a folder built into the program into `target`, with the
/// folders inside it; READMEs stay out of the built site.
fn write_embedded(dir: &include_dir::Dir, target: &Path) -> Result<(), BuildError> {
    for entry in dir.entries() {
        match entry {
            include_dir::DirEntry::Dir(inner) => write_embedded(inner, target)?,
            include_dir::DirEntry::File(file) => {
                if file
                    .path()
                    .file_name()
                    .is_some_and(|name| name == "README.md")
                {
                    continue;
                }
                let path = target.join(file.path());
                if let Some(folder) = path.parent() {
                    fs::create_dir_all(folder).map_err(|source| BuildError::Io {
                        path: folder.to_path_buf(),
                        source,
                    })?;
                }
                fs::write(&path, file.contents())
                    .map_err(|source| BuildError::Io { path, source })?;
            }
        }
    }
    Ok(())
}

/// Whether any block in `pages` is a Subscribe block.
fn uses_subscribe<'a>(mut pages: impl Iterator<Item = &'a Page>) -> bool {
    pages.any(|page| {
        block::all_blocks(&page.content.blocks)
            .iter()
            .any(|block| matches!(block, Block::Subscribe(_)))
    })
}

/// One of the podcast's feeds, for links to it.
struct PodcastFeed {
    /// In the built site: `episodes.mp3.rss`.
    file: String,
    /// "Podcast feed (MP3)".
    label: String,
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

fn write(path: &Path, contents: impl AsRef<[u8]>) -> Result<(), BuildError> {
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
    copy_or_link_dir(from, to, false)
}

/// Like [`copy_dir`]; with `link`, files become hard links to the
/// originals where the file system allows it: the same file under a second
/// name, written in no time and taking no extra space. For audio files of
/// 100 MB each, built again for every preview.
fn copy_or_link_dir(from: &Path, to: &Path, link: bool) -> Result<usize, BuildError> {
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
            count += copy_or_link_dir(&source, &target, link)?;
        } else {
            // Another drive (or a file system without links): copy.
            if !link || fs::hard_link(&source, &target).is_err() {
                fs::copy(&source, &target).map_err(io(&target))?;
            }
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
    colors: &'a ColorClasses,
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
    /// A hand-made main menu for the top bar, with dropdowns; empty for
    /// the automatic menu.
    nav: Vec<NavItem>,
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
    /// The site title next to the logo; without it, the logo's alt text
    /// carries the title.
    show_site_title: bool,
    /// Date and authors, e.g. "2026-10-21 · Claude"; for an episode,
    /// season and number first.
    byline: String,
    /// Each block's HTML, from `block.html`; for an episode, the episode
    /// layout's blocks with the page's own as its shownotes.
    blocks: Vec<String>,
    /// `player.js`, when a block has the Podlove Web Player: loaded in
    /// `<head>`, so it hides the plain player before it shows.
    player_script: Option<String>,
    /// The page's content goes into the search index (not the 404 page).
    searchable: bool,
    /// The site footer's blocks, rendered for this page.
    footer_blocks: Vec<String>,
    /// Links to the page before and after this one (#136), when the page
    /// above it asks for them.
    neighbours: Neighbours,
}

/// The links at the bottom of a page in a series (#136): on the left the
/// one before (Back, or Older for posts), on the right the one after
/// (Next, or Newer).
#[derive(Default)]
struct Neighbours {
    left: Option<Neighbour>,
    right: Option<Neighbour>,
    /// What the links are, for screen readers: "Back and next".
    label: String,
}

struct Neighbour {
    /// "Back", "Next", "Older" or "Newer", in the site's language.
    label: &'static str,
    title: String,
    href: String,
}

/// "Back", "Next", "Older", "Newer" and "and", in the site's language.
fn neighbour_words(language: &str) -> [&'static str; 5] {
    match language.trim().split(['-', '_']).next().unwrap_or_default() {
        "de" => ["Zurück", "Weiter", "Älter", "Neuer", "und"],
        _ => ["Back", "Next", "Older", "Newer", "and"],
    }
}

/// The pages before and after `page` among those below the same page, if
/// that page has `back_next` on. Posts (pages with a date) go newest first
/// as in a Post list, so the older one is on the left; other pages go by
/// their order, as in a Page list.
fn neighbours(page: &Page, context: &Context, links: &LinkContext) -> Neighbours {
    let Some((parent, _)) = page.path.rsplit_once('/').or_else(|| {
        // Pages at the top are below the start page ("").
        (!page.path.is_empty()).then_some(("", page.path.as_str()))
    }) else {
        return Neighbours::default();
    };
    let wanted = context
        .pages
        .iter()
        .any(|other| other.path == parent && other.content.front.back_next);
    if !wanted {
        return Neighbours::default();
    }
    let [back, next, older, newer, and] = neighbour_words(context.language);
    let dated = page.content.front.date.is_some();
    let series = if dated {
        posts(context.pages, parent, context.today)
    } else {
        children(context.pages, parent)
    };
    let Some(index) = series.iter().position(|other| other.path == page.path) else {
        return Neighbours::default();
    };
    let before = index.checked_sub(1).and_then(|i| series.get(i));
    let after = series.get(index + 1);
    let link = |label: &'static str, other: &&Page| Neighbour {
        label,
        title: other.content.front.title.clone(),
        href: links.page(&other.path),
    };
    if dated {
        Neighbours {
            left: after.map(|other| link(older, other)),
            right: before.map(|other| link(newer, other)),
            label: format!("{older} {and} {}", newer.to_lowercase()),
        }
    } else {
        Neighbours {
            left: before.map(|other| link(back, other)),
            right: after.map(|other| link(next, other)),
            label: format!("{back} {and} {}", next.to_lowercase()),
        }
    }
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

fn header_classes(style: HeaderStyle, size: LogoSize) -> HeaderClasses {
    // On a colored bar, primary text would vanish (or not stand out), so
    // the current page is underlined instead.
    const UNDERLINED: &str = "font-semibold underline decoration-2 underline-offset-4";
    let (bar, current) = match style {
        HeaderStyle::Plain => (
            "border-t-4 border-b border-t-accent border-b-base-300",
            "font-semibold text-primary",
        ),
        HeaderStyle::Primary => ("bg-primary text-primary-content", UNDERLINED),
        HeaderStyle::Secondary => ("bg-secondary text-secondary-content", UNDERLINED),
        HeaderStyle::Accent => ("bg-accent text-accent-content", UNDERLINED),
        HeaderStyle::Neutral => ("bg-neutral text-neutral-content", UNDERLINED),
    };
    // A fixed height and the width it needs: square logos stay square, wide
    // ones (a wordmark, a row of icons) get room. On a colored bar the logo
    // gets a light backing. Whole class names, for Tailwind's scan.
    let backed = style != HeaderStyle::Plain;
    let logo = match (size, backed) {
        (LogoSize::Small, false) => "h-9 w-auto max-w-48 object-contain",
        (LogoSize::Small, true) => {
            "h-9 w-auto max-w-48 object-contain rounded-field bg-base-100 p-0.5"
        }
        (LogoSize::Medium, false) => "h-12 w-auto max-w-64 object-contain",
        (LogoSize::Medium, true) => {
            "h-12 w-auto max-w-64 object-contain rounded-field bg-base-100 p-0.5"
        }
        (LogoSize::Large, false) => "h-16 w-auto max-w-80 object-contain",
        (LogoSize::Large, true) => {
            "h-16 w-auto max-w-80 object-contain rounded-field bg-base-100 p-1"
        }
        // The image's own size (its width and height attributes), but no
        // taller than 112 px and no wider than the bar allows.
        (LogoSize::Original, false) => "h-auto max-h-28 w-auto max-w-full object-contain",
        (LogoSize::Original, true) => {
            "h-auto max-h-28 w-auto max-w-full object-contain rounded-field bg-base-100 p-1"
        }
    };
    HeaderClasses { bar, current, logo }
}

/// The site's color choices (`[colors]` in `site.toml`) as the classes
/// the templates use.
struct ColorClasses {
    /// On `<body>`: the content background and the links' color.
    body: String,
    footer: &'static str,
    hero: &'static str,
    hero_button: &'static str,
    /// For a Hero with a background image: any color stands out on it.
    image_button: &'static str,
    call_to_action: &'static str,
    call_to_action_button: &'static str,
    boxed: &'static str,
}

/// Whole class names throughout, so Tailwind finds them when it scans this
/// file. A colored area gets `on-color`: Markdown text in it takes the
/// area's text color (site.input.css). `dark` is whether the theme's page
/// background is dark: the footer is then shaded by default, not a pale
/// band of the accent color below a dark page.
fn color_classes(colors: &Colors, dark: bool) -> ColorClasses {
    let buttons = colors.buttons.unwrap_or(ThemeColor::Primary);
    // A button on a box of its own color would vanish: it's drawn plain.
    let button_on = |surface: Surface| {
        if surface.color() == Some(buttons) {
            "btn"
        } else {
            match buttons {
                ThemeColor::Primary => "btn btn-primary",
                ThemeColor::Secondary => "btn btn-secondary",
                ThemeColor::Accent => "btn btn-accent",
                ThemeColor::Neutral => "btn btn-neutral",
            }
        }
    };
    let hero = colors.hero.unwrap_or(Surface::Secondary);
    let call_to_action = colors.call_to_action.unwrap_or(Surface::Primary);
    let content = match colors.content.unwrap_or(Surface::Plain) {
        Surface::Plain => "bg-base-100",
        Surface::Shaded => "bg-base-200",
        Surface::Primary => "content-primary",
        Surface::Secondary => "content-secondary",
        Surface::Accent => "content-accent",
        Surface::Neutral => "content-neutral",
    };
    let links = match colors.links.unwrap_or(ThemeColor::Primary) {
        ThemeColor::Primary => "",
        ThemeColor::Secondary => " links-secondary",
        ThemeColor::Accent => " links-accent",
        ThemeColor::Neutral => " links-neutral",
    };
    ColorClasses {
        body: format!("{content}{links}"),
        footer: match colors.footer.unwrap_or(if dark {
            Surface::Shaded
        } else {
            Surface::Accent
        }) {
            Surface::Plain => "border-t border-base-300 bg-base-100 text-base-content",
            other => filled(other),
        },
        hero: filled(hero),
        hero_button: button_on(hero),
        image_button: button_on(Surface::Plain),
        call_to_action: filled(call_to_action),
        call_to_action_button: button_on(call_to_action),
        boxed: match colors.boxes.unwrap_or(Surface::Plain) {
            // Its own text color too: on a colored section, it would
            // otherwise take the section's light text.
            Surface::Plain => "border border-base-300 bg-base-100 text-base-content shadow-sm",
            other => filled(other),
        },
    }
}

/// An area filled with a surface: background, text color, and `on-color`
/// for Markdown in a theme color (site.input.css).
fn filled(surface: Surface) -> &'static str {
    match surface {
        Surface::Plain => "border border-base-300 bg-base-100 text-base-content",
        Surface::Shaded => "bg-base-200 text-base-content",
        Surface::Primary => "on-color bg-primary text-primary-content",
        Surface::Secondary => "on-color bg-secondary text-secondary-content",
        Surface::Accent => "on-color bg-accent text-accent-content",
        Surface::Neutral => "on-color bg-neutral text-neutral-content",
    }
}

/// The classes of a section: its fill and the room around its blocks.
/// A section in a theme color also gets `on-primary` (and so on), which
/// draws buttons of that same color plain, so they don't vanish
/// (site.input.css). With a picture, the text is white on the picture,
/// darkened with black.
fn section_classes(section: &block::Section, has_picture: bool) -> String {
    let large = section.spacing == Some(block::Spacing::Large);
    let fill = if has_picture {
        // White, not the theme's neutral text: the picture is darkened
        // with black, and in dark themes neutral text is dark.
        Some("on-color relative overflow-hidden text-white")
    } else {
        section.background.map(filled)
    };
    let own = match section.background.filter(|_| !has_picture) {
        Some(Surface::Primary) => " on-primary",
        Some(Surface::Secondary) => " on-secondary",
        Some(Surface::Accent) => " on-accent",
        Some(Surface::Neutral) => " on-neutral",
        _ => "",
    };
    match (fill, large) {
        (Some(fill), false) => format!("rounded-box p-6 sm:p-8 {fill}{own}"),
        (Some(fill), true) => format!("rounded-box px-6 py-12 sm:px-12 sm:py-20 {fill}{own}"),
        // Without a fill, only the room above and below shows.
        (None, false) => String::new(),
        (None, true) => "py-8 sm:py-12".to_string(),
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
    /// The label of the button that closes an image shown big.
    close: &'static str,
    colors: &'a ColorClasses,
}

/// Images open big when they are this much wider than their place: an
/// Image block's (the content column, 56rem = 896 px) or a Text with
/// image's (half of it), by about half again.
const ZOOM_ABOVE_FULL: u32 = 1300;
const ZOOM_ABOVE_HALF: u32 = 700;

/// "Close" in the site's language, for the button over an image shown big.
fn close_label(language: &str) -> &'static str {
    match language.split('-').next().unwrap_or_default() {
        "de" => "Schließen",
        _ => "Close",
    }
}

#[derive(Template)]
#[template(path = "redirect.html")]
struct RedirectTemplate<'a> {
    target: &'a str,
    language: &'a str,
    /// The theme's page background and text, as `#rrggbb`: the page shows
    /// for a moment, so it shouldn't flash white on a dark site.
    background: &'a str,
    text: &'a str,
    dark: bool,
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
    /// A group's title in the sidebar: a heading, not a link.
    heading: bool,
}

/// An entry of a hand-made main menu in the top bar: a link, or a group
/// with entries in a dropdown.
struct NavItem {
    title: String,
    href: String,
    current: bool,
    exact: bool,
    divider: bool,
    children: Vec<NavItem>,
}

/// The hand-made menu for the page at `page_path`. Pages that aren't
/// (or aren't any longer) on the site are left out.
fn hand_made_menu(
    items: &[crate::site::MenuItem],
    links: &LinkContext,
    page_path: &str,
    pages: &[&Page],
) -> Vec<NavItem> {
    items
        .iter()
        .filter_map(|item| {
            let title = item.title.clone().filter(|t| !t.trim().is_empty());
            if item.divider {
                return Some(NavItem {
                    title: String::new(),
                    href: String::new(),
                    current: false,
                    exact: false,
                    divider: true,
                    children: Vec::new(),
                });
            }
            if !item.items.is_empty() {
                let children = hand_made_menu(&item.items, links, page_path, pages);
                return Some(NavItem {
                    title: title.unwrap_or_default(),
                    href: String::new(),
                    current: children.iter().any(|child| child.current),
                    exact: false,
                    divider: false,
                    children,
                });
            }
            let (title, href, target) = if let Some(path) = &item.page {
                let path = crate::site::menu_page_path(path);
                let page = pages.iter().find(|page| page.path == path)?;
                (
                    title.unwrap_or_else(|| page.content.front.title.clone()),
                    links.page(&path),
                    Some(path),
                )
            } else {
                let link = item.link.as_deref()?.trim();
                (title?, links.resolve(link), links.page_path_of(link))
            };
            Some(NavItem {
                current: target.as_deref().is_some_and(|t| is_within(page_path, t)),
                exact: target.as_deref() == Some(page_path),
                title,
                href,
                divider: false,
                children: Vec::new(),
            })
        })
        .collect()
}

/// A hand-made menu for the sidebar: groups become headings with their
/// entries below; dividers are left out.
fn sidebar_links(nav: &[NavItem]) -> Vec<MenuLink> {
    let mut links = Vec::new();
    for item in nav {
        let group = !item.children.is_empty();
        links.push(MenuLink {
            title: item.title.clone(),
            href: item.href.clone(),
            level: 0,
            icon: None,
            current: item.current,
            exact: item.exact,
            heading: group,
        });
        for child in item.children.iter().filter(|child| !child.divider) {
            links.push(MenuLink {
                title: child.title.clone(),
                href: child.href.clone(),
                level: 0,
                icon: None,
                current: child.current,
                exact: child.exact,
                heading: false,
            });
        }
    }
    links
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

/// The audio of an episode page.
struct AudioEntry {
    /// One per format, the podcast's main format first; the browser plays
    /// the first it can.
    sources: Vec<AudioSource>,
    /// "Herunterladen".
    download: &'static str,
    duration: Option<String>,
}

struct AudioSource {
    src: String,
    mime_type: &'static str,
    /// "MP3, 28.9 MB", for the download link.
    label: String,
}

/// What an episode page needs for the Podlove Web Player: its data next
/// to the page, and the scripts.
struct PlayerEntry {
    /// The episode's `player.json`, relative to the page.
    episode: String,
    embed: String,
    /// The player's height in pixels once it's there, narrow (below
    /// 600 px wide) and wide; kept free while it loads, so the page
    /// doesn't jump. See [`PlayerEntry::new`].
    narrow: u32,
    wide: u32,
}

impl PlayerEntry {
    /// The player for `episode`, its data at `data`. The heights were
    /// measured in Chrome with Podlove Web Player 5.13.0 (2026-10-28, 0 to
    /// 17 chapters, 305 to 856 px wide): the chapters tab opens with the
    /// player and adds 40 px per chapter. Long chapter titles on phones
    /// wrap and make it a little taller.
    fn new(data: String, embed: String, episode: &podcast::Episode) -> Self {
        let chapters = episode
            .chapters
            .iter()
            .filter(|chapter| podcast::start_millis(&chapter.start).is_some())
            .count() as u32;
        let (narrow, wide) = match chapters {
            0 => (534, 290),
            n => (638 + 40 * n, 410 + 40 * n),
        };
        PlayerEntry {
            episode: data,
            embed,
            narrow,
            wide,
        }
    }
}

/// The Podlove Web Player, built into q.e.d. Where this copy comes from
/// and how to update it: `assets/podlove-player/README.md`.
static PODLOVE_PLAYER: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/assets/podlove-player");

/// Where the player goes in the built site, its episode data next to each
/// episode page, and q.e.d.'s script that starts it.
const PLAYER_DIR: &str = "podlove-player";
const PLAYER_EPISODE: &str = "player.json";
const PLAYER_SCRIPT: &str = include_str!("../assets/player.js");
const PLAYER_FILE: &str = "player.js";
/// The playlist tab's episodes, in the player's folder.
const PLAYER_PLAYLIST: &str = "playlist.json";
/// q.e.d.'s script for the embedded player, which `share.html` in the
/// player's folder loads; see the file.
const PLAYER_SHARE_SCRIPT: &str = include_str!("../assets/player-share.js");
const PLAYER_SHARE_FILE: &str = "qed-share.js";
/// The share tab's channels: a link, email, then the networks the player
/// knows (no Mastodon or Bluesky; the page's share row has those).
const PLAYER_SHARE_CHANNELS: [&str; 8] = [
    "link",
    "mail",
    "facebook",
    "twitter",
    "whats-app",
    "linkedin",
    "pinterest",
    "xing",
];
/// The podcast apps the player's subscribe button offers, the ones that
/// take the feed's address (no directories, which need the show's id
/// there); each shows only on its platforms.
const PLAYER_SUBSCRIBE_CLIENTS: [&str; 10] = [
    "apple-podcasts",
    "overcast",
    "pocket-casts",
    "castro",
    "downcast",
    "antenna-pod",
    "podcast-addict",
    "podcast-republic",
    "gpodder",
    "rss",
];

/// Writes the player, its settings for the whole site and the script that
/// starts it into the built site. The theme tokens are shades of the
/// theme's button color.
fn write_player(
    out_dir: &Path,
    site: &Site,
    published: &[&Page],
    today: Date,
    button_color: Option<&str>,
) -> Result<(), BuildError> {
    write_embedded(&PODLOVE_PLAYER, &out_dir.join(PLAYER_DIR))?;
    write(
        &out_dir.join(PLAYER_DIR).join(PLAYER_SHARE_FILE),
        PLAYER_SHARE_SCRIPT,
    )?;
    let mut config = serde_json::json!({
        "version": 5,
        "activeTab": "chapters",
    });
    let root = site_root_path(&site.config);
    // Sharing and subscribing hand out full addresses.
    if let Some(base) = site.config.base_url.as_deref() {
        let base = base.trim_end_matches('/');
        config["share"] = serde_json::json!({
            "channels": PLAYER_SHARE_CHANNELS,
            "sharePlaytime": true,
            // The embed code's iframe shows this page.
            "outlet": format!("{base}/{PLAYER_DIR}/share.html"),
        });
        if let Some(podcast) = &site.config.podcast {
            let clients: Vec<serde_json::Value> = PLAYER_SUBSCRIBE_CLIENTS
                .iter()
                .map(|id| serde_json::json!({ "id": id }))
                .collect();
            config["subscribe-button"] = serde_json::json!({
                "feed": format!("{base}/{}", podcast::feed_file(podcast.main_format())),
                "clients": clients,
            });
        }
    }
    // The playlist tab: every episode with audio, newest first, each
    // played from its player.json.
    let playlist: Vec<serde_json::Value> = crate::feed::feed_episodes(published, today)
        .into_iter()
        .filter(|page| {
            page.content
                .front
                .episode
                .as_ref()
                .is_some_and(|episode| !episode.audio.is_empty())
        })
        .map(|page| {
            let episode = page.content.front.episode.as_ref();
            serde_json::json!({
                "title": page.content.front.title,
                "config": format!("{root}{}/{PLAYER_EPISODE}", page.path),
                "duration": episode
                    .and_then(|e| e.duration.as_deref())
                    .and_then(podcast::start_millis)
                    .map(podcast::clock_millis),
            })
        })
        .collect();
    if playlist.len() > 1 {
        write(
            &out_dir.join(PLAYER_DIR).join(PLAYER_PLAYLIST),
            serde_json::Value::from(playlist).to_string(),
        )?;
        config["playlist"] = format!("{root}{PLAYER_DIR}/{PLAYER_PLAYLIST}").into();
    }
    if let Some(brand) = button_color {
        let shade = |other: &str, amount: f64| {
            crate::theme::mix_hex(brand, other, amount).unwrap_or_else(|| brand.to_string())
        };
        config["theme"] = serde_json::json!({
            "tokens": {
                "brand": brand,
                "brandDark": shade("#000000", 0.25),
                "brandDarkest": shade("#000000", 0.6),
                "brandLightest": shade("#ffffff", 0.9),
                "shadeDark": "#807e7c",
                "shadeBase": "#807e7c",
                "contrast": "#000000",
                "alt": "#ffffff",
            }
        });
    }
    write(
        &out_dir.join(PLAYER_DIR).join("config.json"),
        config.to_string(),
    )?;
    write(&out_dir.join(PLAYER_FILE), PLAYER_SCRIPT)
}

/// An episode in the Podlove Web Player's format (version 5), written
/// next to its page. `links` start at the site root, so the file works
/// from the embedded player too; the episode's own link is its full
/// address, as shared.
fn player_episode(
    site: &Site,
    page: &Page,
    episode: &podcast::Episode,
    links: &LinkContext,
) -> serde_json::Value {
    let config = &site.config;
    let front = &page.content.front;
    let podcast = config.podcast.as_ref();
    let language = config.language.as_str();
    let cover = podcast
        .and_then(|p| p.cover.as_deref())
        .map(|c| media_link(c, links));
    let poster = episode
        .image
        .as_deref()
        .map(|image| media_link(image, links))
        .or_else(|| cover.clone());
    let order = podcast.map(|p| p.formats()).unwrap_or_default();
    let mut audio: Vec<&podcast::EpisodeAudio> = episode.audio.iter().collect();
    audio.sort_by_key(|a| {
        order
            .iter()
            .position(|f| *f == a.format)
            .unwrap_or(usize::MAX)
    });
    let files: Vec<serde_json::Value> = audio
        .iter()
        .map(|a| {
            let (url, size) = match a.local() {
                Some(file) => (
                    format!("{}{}/{file}", links.to_root(), podcast::AUDIO_URL_DIR),
                    fs::metadata(site.root.join(podcast::AUDIO_DIR).join(file))
                        .ok()
                        .map(|meta| meta.len())
                        .or(a.length),
                ),
                None => (a.file.trim().to_string(), a.length),
            };
            serde_json::json!({
                "url": url,
                "size": size.unwrap_or(0),
                "title": a.format.label(),
                "mimeType": a.format.mime_type(),
            })
        })
        .collect();
    let chapters: Vec<serde_json::Value> = episode
        .chapters
        .iter()
        .filter_map(|chapter| {
            let mut entry = serde_json::json!({
                "start": podcast::clock_millis(podcast::start_millis(&chapter.start)?),
                "title": chapter.title.trim(),
            });
            if let Some(link) = chapter
                .link
                .as_deref()
                .map(str::trim)
                .filter(|l| !l.is_empty())
            {
                entry["href"] = links.resolve(link).into();
            }
            Some(entry)
        })
        .collect();
    // Each person once, with their first role.
    let mut people: Vec<&crate::contributors::EpisodePerson> = Vec::new();
    for person in &episode.people {
        if !people.iter().any(|p| p.contributor == person.contributor) {
            people.push(person);
        }
    }
    let contributors: Vec<serde_json::Value> = people
        .iter()
        .filter_map(|person| {
            let contributor = site
                .contributors
                .iter()
                .find(|c| c.id == person.contributor)?;
            let (role, group) = person.role.taxonomy();
            Some(serde_json::json!({
                "id": contributor.id,
                "name": contributor.name,
                "avatar": contributor.photo.as_deref().map(|photo| media_link(photo, links)),
                "role": { "id": role, "slug": role, "title": person.role.label(language) },
                "group": { "id": group, "slug": group, "title": group },
            }))
        })
        .collect();
    let transcripts = transcript(site, episode);
    serde_json::json!({
        "version": 5,
        "show": {
            "title": podcast.and_then(|p| p.title.as_deref()).unwrap_or(&config.title),
            "subtitle": podcast.and_then(|p| p.author.as_deref()).unwrap_or(""),
            "summary": podcast.and_then(|p| p.description.as_deref()).or(config.description.as_deref()).unwrap_or(""),
            "poster": cover,
            "link": links.page(""),
        },
        "title": front.title,
        "subtitle": front.summary.as_deref().unwrap_or(""),
        "summary": front.summary.as_deref().or(front.description.as_deref()).unwrap_or(""),
        "publicationDate": front.date.as_ref().map(|d| format!("{d}T00:00:00Z")),
        "poster": poster,
        "duration": episode
            .duration
            .as_deref()
            .and_then(podcast::start_millis)
            .map(podcast::clock_millis),
        // Shared as it is, so the full address when the site has one.
        "link": config
            .base_url
            .as_deref()
            .map_or_else(|| links.page(&page.path), |base| absolute(base, &page.path)),
        "audio": files,
        "files": files,
        "chapters": chapters,
        "contributors": contributors,
        "transcripts": transcripts,
    })
}

/// Someone in an episode, on its page.
struct PersonEntry {
    name: String,
    href: String,
    /// "Moderation, Schnitt".
    roles: String,
    picture: Option<Picture>,
}

/// The address of a contributor's page.
fn contributor_path(id: &str) -> String {
    format!("{}/{id}", crate::contributors::CONTRIBUTORS_DIR)
}

/// A page for each of the podcast's contributors at
/// `/contributors/<id>/`: photo, bio and links, then the episodes they're
/// in; and `/contributors/` with all of them, unless the site has a page
/// of its own there. A page the site has at one of these addresses wins.
fn contributor_pages(site: &Site) -> Vec<Page> {
    use crate::contributors::CONTRIBUTORS_DIR;
    if site.config.podcast.is_none() || site.contributors.is_empty() {
        return Vec::new();
    }
    let words = podcast::Words::for_language(&site.config.language);
    let taken = |path: &str| site.pages.iter().any(|page| page.path == path);
    let mut pages = Vec::new();
    if !taken(CONTRIBUTORS_DIR) {
        pages.push(Page {
            path: CONTRIBUTORS_DIR.to_string(),
            file: PathBuf::from(CONTRIBUTORS_DIR).join("index.md"),
            content: PageContent {
                front: FrontMatter {
                    title: words.contributors.to_string(),
                    ..FrontMatter::default()
                },
                blocks: vec![Block::Contributors],
            },
        });
    }
    for person in &site.contributors {
        let path = contributor_path(&person.id);
        if taken(&path) {
            continue;
        }
        let links: String = person
            .links
            .iter()
            .map(|link| {
                format!(
                    "\n- [{}]({})",
                    link.title.as_deref().unwrap_or(&link.url),
                    link.url
                )
            })
            .collect();
        let text = format!("{}\n{links}", person.bio).trim().to_string();
        let about = match &person.photo {
            Some(photo) => Block::TextImage(block::TextImage {
                text: Some(text),
                image: Some(photo.clone()),
                alt: Some(person.name.clone()),
                image_side: block::Side::Left,
            }),
            None => Block::Text { markdown: text },
        };
        pages.push(Page {
            path: path.clone(),
            file: PathBuf::from(CONTRIBUTORS_DIR).join(format!("{}.md", person.id)),
            content: PageContent {
                front: FrontMatter {
                    title: person.name.clone(),
                    image: person.photo.clone(),
                    ..FrontMatter::default()
                },
                blocks: vec![
                    about,
                    Block::Text {
                        markdown: format!("## {}", words.episodes),
                    },
                    Block::PostList(block::PostList {
                        from: Some(format!("/{}", podcast::EPISODES_PATH)),
                        contributor: Some(person.id.clone()),
                        ..block::PostList::default()
                    }),
                ],
            },
        });
    }
    pages
}

/// The share row of an episode page (#053): plain links for Bluesky and
/// email, and buttons `share.js` brings to life.
struct ShareEntry {
    /// The page's full address.
    url: String,
    /// The episode's title, to share with the address.
    text: String,
    bluesky: String,
    email: String,
    script: String,
    words: podcast::Words,
}

/// Text for a web address or a `mailto:` link: everything but letters,
/// digits and `-_.~` as `%XX`.
fn url_encode(text: &str) -> String {
    text.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

/// q.e.d.'s own script for the share buttons; see the file.
const SHARE_SCRIPT: &str = include_str!("../assets/share.js");
const SHARE_FILE: &str = "share.js";

/// One chapter on an episode page.
struct ChapterEntry {
    start: String,
    title: String,
    href: Option<String>,
}

struct PostEntry {
    title: String,
    href: String,
    /// `2017-02-24`, for `<time datetime>`.
    date: String,
    /// `24. Februar 2017`, in the site's language.
    date_text: String,
    /// For an episode: "Folge 5", see [`podcast::label`].
    episode: Option<String>,
    summary_html: Option<String>,
    /// For cards and tiles; see [`list_picture`].
    picture: Option<Picture>,
}

/// One page in a Page list.
struct PageEntry {
    title: String,
    href: String,
    summary_html: Option<String>,
    picture: Option<Picture>,
}

/// A block, ready for the template: Markdown rendered, links resolved.
enum RenderedBlock {
    Text {
        html: String,
    },
    /// The parts of an episode page; see [`episode_part`].
    EpisodePlayer {
        audio: AudioEntry,
        player: Option<PlayerEntry>,
    },
    EpisodePeople {
        /// "Mitwirkende", in the site's language.
        heading: &'static str,
        /// Each person once, with all their roles.
        people: Vec<PersonEntry>,
    },
    EpisodeChapters {
        heading: &'static str,
        chapters: Vec<ChapterEntry>,
    },
    EpisodeTranscript {
        heading: &'static str,
        paragraphs: Vec<TranscriptParagraph>,
    },
    EpisodeShare {
        share: ShareEntry,
    },
    LatestEpisode {
        title: String,
        href: String,
        /// "Folge 5 · 28. Oktober 2026".
        label: String,
        summary_html: Option<String>,
        audio: Option<AudioEntry>,
        /// The Podlove Web Player for it, as on its own page.
        player: Option<PlayerEntry>,
        /// "Zur Episode": the shownotes are on its page.
        more: &'static str,
    },
    Subscribe {
        /// The button's script, and the file with the podcast's data it
        /// reads.
        script: String,
        data: String,
        /// `de`, `en`, … for the button's words.
        language: String,
        /// `text-center` and the like: the button is an inline frame, so
        /// text alignment moves it.
        align: &'static str,
        /// The theme's button color, `#rrggbb`; Podlove's green if none.
        color: Option<String>,
        /// `cover`: the podcast's cover with the button below it; or
        /// `rectangle`, the button alone, for a podcast without a cover.
        format: &'static str,
        /// For readers without JavaScript: links to the feeds.
        feeds: Vec<FeedLink>,
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
        /// Unequal widths: `--cols: 2fr 1fr`, which the grid class reads.
        style: Option<String>,
        columns: Vec<ColumnEntry>,
    },
    Section {
        title: Option<String>,
        /// From [`section_classes`].
        classes: String,
        /// The background picture, behind the blocks.
        picture: Option<Picture>,
        /// The section's blocks, each rendered with `block.html`.
        blocks: Vec<String>,
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
        /// `list`, `cards` or `tiles`.
        layout: &'static str,
    },
    PageList {
        pages: Vec<PageEntry>,
        compact: bool,
        layout: &'static str,
    },
    Video {
        href: String,
        picture: Option<Picture>,
        title: Option<String>,
        /// `YouTube`, `Vimeo`, or a PeerTube instance's host.
        platform: Option<String>,
    },
    /// A video file of the site, in the browser's own player.
    VideoFile {
        src: String,
        /// Shown before the video plays.
        poster: Option<String>,
        title: Option<String>,
    },
}

/// The image link previews show, with its full address.
struct ShareImage {
    url: String,
    width: Option<u32>,
    height: Option<u32>,
    alt: Option<String>,
}

pub(crate) struct Picture {
    src: String,
    /// The smaller copies and the original, with their widths, for the
    /// browser to choose from: `media/a.480w.webp 480w, …, media/a.jpg 2400w`.
    srcset: Option<String>,
    alt: String,
    width: Option<u32>,
    height: Option<u32>,
    /// The largest copy up to 1600 pixels wide (or the original, if it has
    /// no copies), for showing the image big; `srcset` offers the original
    /// too, for large screens.
    largest: Option<Zoom>,
    /// Set when a click shows the image big over the page (see
    /// [`Picture::zoomable_above`]).
    zoom: Option<Zoom>,
}

/// The image shown big over the page, in a popover with this `id`.
#[derive(Clone)]
struct Zoom {
    id: String,
    src: String,
    width: u32,
    height: u32,
}

impl Picture {
    /// Lets a click show the picture big when its largest version is wider
    /// than `shown` pixels: noticeably more than its place on the page.
    fn zoomable_above(mut self, shown: u32) -> Self {
        self.zoom = self.largest.clone().filter(|big| big.width > shown);
        self
    }

    /// Everything but `src` and `alt` for an `<img>` written outside the
    /// templates (images in Markdown text), each with a leading space.
    pub(crate) fn attributes(&self, sizes: &str) -> String {
        let mut out = String::new();
        if let Some(srcset) = &self.srcset {
            out.push_str(&format!(r#" srcset="{srcset}" sizes="{sizes}""#));
        }
        if let (Some(width), Some(height)) = (self.width, self.height) {
            out.push_str(&format!(r#" width="{width}" height="{height}""#));
        }
        out
    }
}

struct ColumnEntry {
    title: Option<String>,
    /// Drawn as a box.
    boxed: bool,
    /// The column's blocks, each rendered with `block.html`.
    blocks: Vec<String>,
}

struct FaqEntry {
    question: String,
    answer_html: String,
}

/// The cues of an episode's transcript; none without one, or when its
/// file can't be read (the build notes that, see `audio_notes`).
fn transcript(site: &Site, episode: &podcast::Episode) -> Vec<podcast::Cue> {
    episode
        .transcript
        .as_deref()
        .and_then(|file| {
            fs::read_to_string(site.root.join(podcast::TRANSCRIPTS_DIR).join(file.trim())).ok()
        })
        .map(|text| {
            podcast::transcript_cues(&text, |voice| {
                site.contributors
                    .iter()
                    .find(|c| !voice.is_empty() && c.name == voice)
                    .map(|c| c.id.clone())
            })
        })
        .unwrap_or_default()
}

/// A paragraph of a transcript on an episode page: what one voice says
/// until the next one speaks.
struct TranscriptParagraph {
    /// When it starts, in whole seconds, "01:02:03".
    start: String,
    /// The speaker's name, if the file names them.
    voice: Option<String>,
    text: String,
}

/// A pause longer than this starts a new paragraph of a transcript.
/// Measured on a Whisper transcript: within a passage, cues were at most
/// 1.3 s apart; new passages came after 1.8 s and more.
const PARAGRAPH_PAUSE_MS: u64 = 1_500;
/// A paragraph of a transcript ends after this long at the latest.
const PARAGRAPH_MAX_MS: u64 = 60_000;

/// Joins cues into paragraphs: a new one when the voice changes, after a
/// pause, or after a minute. Transcripts without voices (Whisper's, say)
/// would otherwise be one paragraph. The name shows only when the voice
/// changes.
fn paragraphs(cues: &[podcast::Cue]) -> Vec<TranscriptParagraph> {
    let mut out: Vec<TranscriptParagraph> = Vec::new();
    // The voice, start and end of the paragraph being written.
    let mut current: Option<(&str, u64, u64)> = None;
    let mut shown_voice: Option<&str> = None;
    for cue in cues {
        let text = cue.text.trim();
        if text.is_empty() {
            continue;
        }
        let goes_on = current.is_some_and(|(voice, start, end)| {
            voice == cue.voice
                && cue.start_ms.saturating_sub(end) <= PARAGRAPH_PAUSE_MS
                && cue.start_ms.saturating_sub(start) < PARAGRAPH_MAX_MS
        });
        match out.last_mut() {
            Some(paragraph) if goes_on => {
                paragraph.text.push(' ');
                paragraph.text.push_str(text);
                current = current.map(|(voice, start, _)| (voice, start, cue.end_ms));
            }
            _ => {
                let new_voice = shown_voice != Some(cue.voice.as_str());
                out.push(TranscriptParagraph {
                    // Whole seconds; the file's milliseconds say little here.
                    start: podcast::clock(cue.start_ms / 1000),
                    voice: Some(cue.voice.clone()).filter(|voice| new_voice && !voice.is_empty()),
                    text: text.to_string(),
                });
                shown_voice = Some(&cue.voice);
                current = Some((&cue.voice, cue.start_ms, cue.end_ms));
            }
        }
    }
    out
}

/// What the parts of an episode page are made from.
struct EpisodePage<'a> {
    site: &'a Site,
    context: &'a Context<'a>,
    page: &'a Page,
    episode: &'a podcast::Episode,
    links: &'a LinkContext,
}

impl EpisodePage<'_> {
    /// The part of the page `block` stands for (see the episode layout);
    /// `None` when the episode has nothing for it, such as no chapters.
    /// The shownotes are the page's own blocks, which `render_page` adds.
    fn part(&self, block: &Block) -> Option<RenderedBlock> {
        let (site, links, episode) = (self.site, self.links, self.episode);
        let config = &site.config;
        let words = podcast::Words::for_language(&config.language);
        match block {
            Block::EpisodePlayer => {
                let audio =
                    audio_entry(&site.root, config.podcast.as_ref(), episode, links, &words)?;
                Some(RenderedBlock::EpisodePlayer {
                    audio,
                    player: config.podcast.as_ref().map(|_| {
                        PlayerEntry::new(
                            PLAYER_EPISODE.to_string(),
                            format!("{}{PLAYER_DIR}/embed.js", links.to_root()),
                            episode,
                        )
                    }),
                })
            }
            Block::EpisodePeople => {
                // Each person once, with all their roles.
                let mut order: Vec<&str> = Vec::new();
                for person in &episode.people {
                    if !order.contains(&person.contributor.as_str()) {
                        order.push(&person.contributor);
                    }
                }
                let people: Vec<PersonEntry> = order
                    .into_iter()
                    .filter_map(|id| {
                        let person = site.contributors.iter().find(|c| c.id == id)?;
                        let roles: Vec<&str> = episode
                            .people
                            .iter()
                            .filter(|p| p.contributor == id)
                            .map(|p| p.role.label(&config.language))
                            .collect();
                        Some(PersonEntry {
                            name: person.name.clone(),
                            href: links.page(&contributor_path(id)),
                            roles: roles.join(", "),
                            picture: person.photo.as_deref().and_then(|photo| {
                                picture(photo, Some(&person.name), links, self.context.images)
                            }),
                        })
                    })
                    .collect();
                (!people.is_empty()).then_some(RenderedBlock::EpisodePeople {
                    heading: words.contributors,
                    people,
                })
            }
            Block::EpisodeChapters => {
                let chapters: Vec<ChapterEntry> = episode
                    .chapters
                    .iter()
                    .map(|chapter| ChapterEntry {
                        start: podcast::readable_start(&chapter.start),
                        title: chapter.title.clone(),
                        href: chapter
                            .link
                            .as_deref()
                            .map(str::trim)
                            .filter(|link| !link.is_empty())
                            .map(|link| links.resolve(link)),
                    })
                    .collect();
                (!chapters.is_empty()).then_some(RenderedBlock::EpisodeChapters {
                    heading: words.chapters,
                    chapters,
                })
            }
            Block::EpisodeTranscript => {
                let paragraphs = paragraphs(&transcript(site, episode));
                (!paragraphs.is_empty()).then_some(RenderedBlock::EpisodeTranscript {
                    heading: words.transcript,
                    paragraphs,
                })
            }
            // Sharing needs the page's full address.
            Block::EpisodeShare => {
                let base = config
                    .base_url
                    .as_ref()
                    .filter(|_| config.podcast.is_some())?;
                let front = &self.page.content.front;
                let url = absolute(base, &self.page.path);
                let both = format!("{} {url}", front.title);
                Some(RenderedBlock::EpisodeShare {
                    share: ShareEntry {
                        bluesky: format!(
                            "https://bsky.app/intent/compose?text={}",
                            url_encode(&both)
                        ),
                        email: format!(
                            "mailto:?subject={}&body={}",
                            url_encode(&front.title),
                            url_encode(&url)
                        ),
                        script: format!("{}{SHARE_FILE}", links.to_root()),
                        text: front.title.clone(),
                        url,
                        words,
                    },
                })
            }
            _ => None,
        }
    }
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
    // The 404 page is shown at whatever address was asked for, so its links
    // start at the site root instead of being relative.
    let not_found = page.path == NOT_FOUND_PATH;
    let links = if not_found {
        LinkContext::rooted(&site_root_path(config), &page.file)
    } else {
        LinkContext::new(&page.path, &page.file)
    }
    .with_own_site(config.base_url.as_deref());

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
                heading: false,
            })
            .collect()
    };

    let words = podcast::Words::for_language(&config.language);
    let mut byline: Vec<String> = Vec::new();
    byline.extend(
        front
            .episode
            .as_ref()
            .map(|episode| podcast::label(episode, &words))
            .filter(|label| !label.is_empty()),
    );
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
        .filter(|_| !not_found)
        .map(|base| absolute(base, &page.path));

    let share_image = config
        .base_url
        .as_ref()
        .filter(|_| !not_found)
        .and_then(|base| share_image(page, base, context.images));

    // A hand-made menu replaces the automatic one.
    let nav = hand_made_menu(&config.main_menu, &links, &page.path, context.pages);
    let main_links = if nav.is_empty() {
        menu_links(main_menu)
    } else {
        sidebar_links(&nav)
    };
    let sidebar = config.navigation == Navigation::Sidebar && !main_links.is_empty();
    let failed = |source| BuildError::Template {
        page: page.path.clone(),
        source,
    };
    let own: Vec<String> = page
        .content
        .blocks
        .iter()
        .map(|block| render_html(block, &links, context, false))
        .collect::<Result<_, _>>()
        .map_err(failed)?;
    // An episode page is its episode layout, with the page's own blocks as
    // the shownotes.
    let blocks = match front.episode.as_ref().filter(|_| !not_found) {
        None => own,
        Some(episode) => {
            let layout = site
                .episode_layout
                .as_ref()
                .map_or(&block::DEFAULT_EPISODE_LAYOUT[..], |layout| {
                    &layout.content.blocks
                });
            let mut blocks = Vec::new();
            for block in layout {
                if *block == Block::EpisodeShownotes {
                    blocks.extend(own.iter().cloned());
                } else if block.is_episode_part() {
                    let part = EpisodePage {
                        site,
                        context,
                        page,
                        episode,
                        links: &links,
                    }
                    .part(block);
                    if let Some(part) = part {
                        blocks.push(block_html(&part, context, false).map_err(failed)?);
                    }
                } else {
                    blocks.push(render_html(block, &links, context, false).map_err(failed)?);
                }
            }
            blocks
        }
    };
    let player_script = blocks
        .iter()
        .any(|html| html.contains("data-podlove-player"))
        .then(|| format!("{}{PLAYER_FILE}", links.to_root()));
    let neighbours = if not_found {
        Neighbours::default()
    } else {
        neighbours(page, context, &links)
    };
    let template = PageTemplate {
        neighbours,
        player_script,
        searchable: config.search && !not_found,
        colors: context.colors,
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
        feeds: context
            .podcast_feeds
            .iter()
            .map(|feed| FeedLink {
                title: format!("{} · {}", feed.label, config.title),
                href: format!("{}{}", links.to_root(), feed.file),
                label: feed.label.clone(),
            })
            .chain(feeds.iter().map(|feed| FeedLink {
                title: feed.title.clone(),
                href: format!("{}{}", links.to_root(), feed.file),
                label: if feeds.len() == 1 {
                    "RSS feed".to_string()
                } else {
                    format!("{} RSS feed", feed.page.content.front.title)
                },
            }))
            .collect(),
        rss_icon: RSS_ICON,
        is_home: page.path.is_empty(),
        root: links.to_root(),
        home_href: links.page(""),
        main_menu: main_links,
        nav,
        footer_menu: menu_links(footer_menu),
        show_title: show_title(page),
        sidebar,
        // A sidebar takes room of its own, so the page gets wider with it.
        // Whole class names, for Tailwind's scan.
        width: match (config.page_width, sidebar) {
            (PageWidth::Normal, false) => "max-w-4xl",
            (PageWidth::Normal, true) | (PageWidth::Wide, false) => "max-w-6xl",
            (PageWidth::Wide, true) => "max-w-7xl",
            (PageWidth::Full, _) => "max-w-none",
        },
        header: header_classes(config.header, config.logo_size),
        // Without a logo, the title is all the header has.
        show_site_title: config.title_in_header || config.logo.is_none(),
        byline: byline.join(" · "),
        blocks,
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
    if page.content.front.hide_title {
        return false;
    }
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
    block_html(&render_block(block, links, context)?, context, in_column)
}

/// The HTML of a block ready for the template.
fn block_html(
    rendered: &RenderedBlock,
    context: &Context,
    in_column: bool,
) -> Result<String, askama::Error> {
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
    let html = BlockTemplate {
        block: rendered,
        full_sizes,
        half_sizes,
        rss_icon: RSS_ICON,
        close: close_label(context.language),
        colors: context.colors,
    }
    .render()?;
    // Lists of other pages stay out of the search index: results should
    // lead to the posts, not to every list that mentions them. So does the
    // player, whose plain version (length, download) would be the excerpt.
    Ok(match rendered {
        RenderedBlock::PostList { .. }
        | RenderedBlock::PageList { .. }
        | RenderedBlock::LatestEpisode { .. }
        | RenderedBlock::EpisodePlayer { .. } => format!("<div data-pagefind-ignore>{html}</div>"),
        _ => html,
    })
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
    let markdown = |text: &Option<String>| {
        text.as_deref()
            .map(|t| markdown::render(t, links, &context.text_media()))
    };
    let button = |label: &Option<String>, link: &Option<String>| match (label, link) {
        (Some(label), Some(link)) => Some(Button {
            label: label.clone(),
            href: links.resolve(link),
        }),
        _ => None,
    };

    Ok(match block {
        Block::Text { markdown: text } => RenderedBlock::Text {
            html: markdown::render(text, links, &context.text_media()),
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
                .and_then(|name| picture(name, image.alt.as_deref(), links, images))
                .map(|picture| picture.zoomable_above(ZOOM_ABOVE_FULL)),
            caption_html: markdown(&image.caption),
        },
        Block::TextImage(block) => RenderedBlock::TextImage {
            text_html: markdown(&block.text),
            picture: block
                .image
                .as_deref()
                .and_then(|name| picture(name, block.alt.as_deref(), links, images))
                .map(|picture| picture.zoomable_above(ZOOM_ABOVE_HALF)),
            image_left: block.image_side == Side::Left,
        },
        Block::Gallery(gallery) => RenderedBlock::Gallery {
            pictures: gallery
                .images
                .iter()
                .filter_map(|image| picture(&image.image, image.alt.as_deref(), links, images))
                // Shown cut to a square, so every one opens big.
                .map(|picture| picture.zoomable_above(0))
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
                    answer_html: markdown::render(&item.answer, links, &context.text_media()),
                })
                .collect(),
        },
        Block::Columns(columns) => RenderedBlock::Columns {
            // Whole class names, so Tailwind finds them when it scans the code.
            grid: match columns.items.len() {
                0 | 1 => "",
                _ if unequal(columns) => "md:grid-cols-(--cols)",
                2 => "md:grid-cols-2",
                3 => "md:grid-cols-3",
                _ => "md:grid-cols-2 lg:grid-cols-4",
            },
            style: (columns.items.len() > 1 && unequal(columns)).then(|| {
                let shares: Vec<String> = columns
                    .items
                    .iter()
                    .map(|column| format!("minmax(0,{}fr)", column.width.unwrap_or(1).max(1)))
                    .collect();
                format!("--cols: {}", shares.join(" "))
            }),
            columns: columns
                .items
                .iter()
                .map(|column| {
                    Ok(ColumnEntry {
                        boxed: column.boxed,
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
        Block::Section(section) => {
            let picture = section
                .image
                .as_deref()
                .and_then(|name| picture(name, None, links, images));
            RenderedBlock::Section {
                title: section
                    .title
                    .clone()
                    .filter(|title| !title.trim().is_empty()),
                classes: section_classes(section, picture.is_some()),
                picture,
                blocks: section
                    .blocks
                    .iter()
                    .map(|inner| render_html(inner, links, context, false))
                    .collect::<Result<_, _>>()?,
            }
        }
        Block::CallToAction(cta) => RenderedBlock::CallToAction {
            headline: cta.headline.clone(),
            text_html: markdown(&cta.text),
            button: button(&cta.button_label, &cta.button_link),
        },
        Block::PostList(list) => {
            let from = list_source(&list.from, links);
            let mut posts = posts(context.pages, from, context.today);
            // Only the episodes a contributor is in, if asked.
            if let Some(id) = list.contributor.as_deref().filter(|id| !id.is_empty()) {
                posts.retain(|post| {
                    post.content
                        .front
                        .episode
                        .as_ref()
                        .is_some_and(|e| e.people.iter().any(|p| p.contributor == id))
                });
            }
            let words = podcast::Words::for_language(context.language);
            if let Some(limit) = list.limit.and_then(|limit| usize::try_from(limit).ok()) {
                posts.truncate(limit);
            }
            RenderedBlock::PostList {
                compact: context.footer,
                layout: layout_name(list.layout),
                feed_href: context
                    .feeds
                    .iter()
                    .find(|feed| feed.page.path == from)
                    .map(|feed| feed.file.as_str())
                    // The episodes' list links the main podcast feed.
                    .or_else(|| {
                        context
                            .podcast_feeds
                            .first()
                            .filter(|_| from == podcast::EPISODES_PATH)
                            .map(|feed| feed.file.as_str())
                    })
                    .map(|file| format!("{}{file}", links.to_root())),
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
                            episode: post
                                .content
                                .front
                                .episode
                                .as_ref()
                                .map(|episode| podcast::label(episode, &words))
                                .filter(|label| !label.is_empty()),
                            summary_html: markdown(&post.content.front.summary),
                            picture: list_picture(post, links, context.images),
                        }
                    })
                    .collect(),
            }
        }
        // An episode page's parts come from `EpisodePage::part`; anywhere
        // else they show nothing.
        Block::EpisodePlayer
        | Block::EpisodePeople
        | Block::EpisodeShownotes
        | Block::EpisodeChapters
        | Block::EpisodeTranscript
        | Block::EpisodeShare => RenderedBlock::Text {
            html: String::new(),
        },
        // Replaced by its piece's blocks before rendering (`shared::expanded`).
        Block::Shared(_) => RenderedBlock::Text {
            html: String::new(),
        },
        // Tiles like a Page list's: photo and name, linked to their page.
        Block::Contributors => RenderedBlock::PageList {
            compact: context.footer,
            layout: "tiles",
            pages: context
                .contributors
                .iter()
                .map(|person| PageEntry {
                    title: person.name.clone(),
                    href: links.page(&contributor_path(&person.id)),
                    summary_html: None,
                    picture: person.photo.as_deref().and_then(|photo| {
                        picture(photo, Some(&person.name), links, context.images)
                    }),
                })
                .collect(),
        },
        Block::LatestEpisode => {
            let words = podcast::Words::for_language(context.language);
            match crate::feed::feed_episodes(context.pages, context.today).first() {
                // No episode yet: nothing to show.
                None => RenderedBlock::Text {
                    html: String::new(),
                },
                Some(page) => {
                    let front = &page.content.front;
                    let episode = front.episode.as_ref();
                    let label: Vec<String> = episode
                        .map(|e| podcast::label(e, &words))
                        .filter(|l| !l.is_empty())
                        .into_iter()
                        .chain(
                            front
                                .date
                                .as_ref()
                                .and_then(|d| d.date())
                                .map(|day| calendar::long_date(day, context.language)),
                        )
                        .collect();
                    let audio = episode
                        .and_then(|e| audio_entry(context.root, context.podcast, e, links, &words));
                    RenderedBlock::LatestEpisode {
                        title: front.title.clone(),
                        href: links.page(&page.path),
                        label: label.join(" · "),
                        summary_html: markdown(&front.summary),
                        // The episode's player data sits next to its page.
                        player: audio.as_ref().and(context.podcast).zip(episode).map(
                            |(_, episode)| {
                                PlayerEntry::new(
                                    format!("{}{PLAYER_EPISODE}", links.page(&page.path)),
                                    format!("{}{PLAYER_DIR}/embed.js", links.to_root()),
                                    episode,
                                )
                            },
                        ),
                        audio,
                        more: words.to_episode,
                    }
                }
            }
        }
        // Without feeds (no podcast, or no web address) there's nothing
        // to subscribe to.
        Block::Subscribe(_) if context.podcast_feeds.is_empty() => RenderedBlock::Text {
            html: String::new(),
        },
        Block::Subscribe(subscribe) => RenderedBlock::Subscribe {
            // Whole class names, for Tailwind's scan.
            align: match subscribe.align.unwrap_or_default() {
                block::Align::Left => "text-left",
                block::Align::Center => "text-center",
                block::Align::Right => "text-right",
            },
            script: format!("{}{SUBSCRIBE_DIR}/javascripts/app.js", links.to_root()),
            data: format!("{}{SUBSCRIBE_DIR}/{SUBSCRIBE_DATA}", links.to_root()),
            language: context
                .language
                .split(['-', '_'])
                .next()
                .unwrap_or("en")
                .to_lowercase(),
            color: context.button_color.map(str::to_string),
            format: if context
                .podcast
                .and_then(|p| p.cover.as_deref())
                .is_some_and(|c| !c.trim().is_empty())
            {
                "cover"
            } else {
                "rectangle"
            },
            feeds: context
                .podcast_feeds
                .iter()
                .map(|feed| FeedLink {
                    title: feed.label.clone(),
                    href: format!("{}{}", links.to_root(), feed.file),
                    label: feed.label.clone(),
                })
                .collect(),
        },
        Block::Video(video)
            if let Some(file) = video
                .file
                .as_deref()
                .map(str::trim)
                .filter(|f| !f.is_empty()) =>
        {
            RenderedBlock::VideoFile {
                src: download_location(file, links, root).0,
                poster: video
                    .image
                    .as_deref()
                    .and_then(|image| picture(image, None, links, images))
                    .map(|picture| picture.src),
                title: video.title.clone().filter(|t| !t.trim().is_empty()),
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
            layout: layout_name(list.layout),
            pages: children(context.pages, list_source(&list.from, links))
                .iter()
                .map(|page| PageEntry {
                    title: page.content.front.title.clone(),
                    href: links.page(&page.path),
                    summary_html: markdown(&page.content.front.summary),
                    picture: list_picture(page, links, context.images),
                })
                .collect(),
        },
    })
}

fn layout_name(layout: Option<ListLayout>) -> &'static str {
    match layout.unwrap_or_default() {
        ListLayout::List => "list",
        ListLayout::Cards => "cards",
        ListLayout::Tiles => "tiles",
    }
}

/// A listed page's picture: its share image (`image` in its settings), or
/// else the first image on it, as link previews choose.
fn list_picture(page: &Page, links: &LinkContext, images: &Images) -> Option<Picture> {
    let (name, alt) = match own_image(page) {
        Some(name) => (name, None),
        None => first_image(&page.content.blocks)?,
    };
    picture(
        name,
        Some(alt.unwrap_or(&page.content.front.title)),
        links,
        images,
    )
}

/// Whether the columns have different widths.
fn unequal(columns: &block::Columns) -> bool {
    let width = |column: &block::Column| column.width.unwrap_or(1).max(1);
    columns
        .items
        .iter()
        .any(|column| width(column) != width(&columns.items[0]))
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
pub(crate) fn picture(
    name: &str,
    alt: Option<&str>,
    links: &LinkContext,
    images: &Images,
) -> Option<Picture> {
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
    let largest = info.as_ref().map(|(file, info)| {
        let id = format!(
            "zoom-{}",
            file.chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                .collect::<String>()
        );
        match info.widths.last() {
            Some(&width) => Zoom {
                id,
                src: links.media(&images::copy_name(file, width)),
                width,
                height: (u64::from(info.height) * u64::from(width) / u64::from(info.width.max(1)))
                    as u32,
            },
            None => Zoom {
                id,
                src: src.clone(),
                width: info.width,
                height: info.height,
            },
        }
    });
    Some(Picture {
        largest,
        zoom: None,
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
    let (name, alt) = match own_image(page) {
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
                images.share_copy(file, width),
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

/// The player and download links for an episode's audio, if it has some.
fn audio_entry(
    root: &Path,
    podcast: Option<&podcast::Podcast>,
    episode: &podcast::Episode,
    links: &LinkContext,
    words: &podcast::Words,
) -> Option<AudioEntry> {
    // The podcast's order of formats, then any others the episode has.
    let mut audio: Vec<&podcast::EpisodeAudio> = episode.audio.iter().collect();
    let order = podcast.map(|podcast| podcast.formats()).unwrap_or_default();
    audio.sort_by_key(|a| {
        order
            .iter()
            .position(|f| *f == a.format)
            .unwrap_or(usize::MAX)
    });
    let sources: Vec<AudioSource> = audio
        .into_iter()
        .map(|a| {
            let (src, length) = match a.local() {
                Some(file) => (
                    format!("{}{}/{file}", links.to_root(), podcast::AUDIO_URL_DIR),
                    // The file itself knows best.
                    fs::metadata(root.join(podcast::AUDIO_DIR).join(file))
                        .ok()
                        .map(|meta| meta.len())
                        .or(a.length),
                ),
                None => (a.file.trim().to_string(), a.length),
            };
            AudioSource {
                src,
                mime_type: a.format.mime_type(),
                label: match length {
                    Some(length) => format!("{}, {}", a.format.label(), format_size(length)),
                    None => a.format.label().to_string(),
                },
            }
        })
        .collect();
    if sources.is_empty() {
        return None;
    }
    Some(AudioEntry {
        sources,
        download: words.download,
        duration: episode.duration.as_deref().map(|d| {
            // Without hours if there are none: 29:55, but 01:24:22.
            let readable = podcast::readable_start(d);
            readable
                .strip_prefix("00:")
                .map_or(readable.clone(), str::to_string)
        }),
    })
}

/// The image a page names for itself: its share image, or an episode's
/// cover.
fn own_image(page: &Page) -> Option<&str> {
    let front = &page.content.front;
    front.image.as_deref().or_else(|| {
        front
            .episode
            .as_ref()
            .and_then(|episode| episode.image.as_deref())
    })
}

/// The first image on a page, columns included, with its alt text.
fn first_image(blocks: &[Block]) -> Option<(&str, Option<&str>)> {
    block::all_blocks(blocks)
        .into_iter()
        .find_map(|block| match block {
            Block::Hero(hero) => hero.image.as_deref().map(|image| (image, None)),
            Block::Section(section) => section.image.as_deref().map(|image| (image, None)),
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
    fn cue(start_ms: u64, end_ms: u64, voice: &str, text: &str) -> podcast::Cue {
        podcast::Cue {
            start: podcast::clock_millis(start_ms),
            start_ms,
            end: podcast::clock_millis(end_ms),
            end_ms,
            speaker: String::new(),
            voice: voice.to_string(),
            text: text.to_string(),
        }
    }

    #[test]
    fn splits_transcripts_into_paragraphs() {
        // Whisper's: no voices. A pause over 1.5 seconds starts a new
        // paragraph, and a minute is the most one gets.
        let cues = [
            cue(1_000, 11_800, "", " Drei Schweinehunde."),
            cue(13_640, 19_500, "", " Stefan hier."),
            cue(19_600, 25_800, "", " Dreimal begrüßt."),
            cue(35_260, 40_580, "", " Hallihallo."),
            cue(40_880, 70_000, "", " Lang."),
            cue(70_100, 99_000, "", " Noch länger."),
            cue(99_100, 99_500, "", " Ende."),
        ];
        let texts: Vec<String> = paragraphs(&cues).into_iter().map(|p| p.text).collect();
        assert_eq!(
            texts,
            [
                "Drei Schweinehunde.",
                "Stefan hier. Dreimal begrüßt.",
                "Hallihallo. Lang. Noch länger.",
                "Ende."
            ]
        );

        // With voices: the name only when it changes.
        let cues = [
            cue(0, 1_000, "Hannah", "Hallo"),
            cue(5_000, 6_000, "Hannah", "und weiter."),
            cue(6_000, 7_000, "Max", "Servus!"),
        ];
        let voices: Vec<Option<String>> = paragraphs(&cues).into_iter().map(|p| p.voice).collect();
        assert_eq!(
            voices,
            [Some("Hannah".to_string()), None, Some("Max".to_string())]
        );
    }

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
        assert!(out.join("media/wide.480w.webp").is_file());
        assert!(home.contains(
            r#"srcset="media/wide.480w.webp 480w, media/wide.960w.webp 960w, media/wide.1600w.webp 1600w, media/wide.jpg 2000w""#
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
    fn large_images_open_big_over_the_page() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("pages")).unwrap();
        fs::create_dir_all(dir.path().join("media")).unwrap();
        fs::write(
            dir.path().join("site.toml"),
            "title = \"Stofftiere\"\nlanguage = \"de-AT\"\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("pages/eule.md"),
            "+++\ntitle = \"Eule\"\n+++\n\n::: text-image\ntext = \"Weise.\"\nimage = \"eule.jpg\"\nalt = \"Eule\"\nimage_side = \"left\"\n:::\n\n::: text-image\ntext = \"Klein.\"\nimage = \"small.png\"\n:::\n",
        )
        .unwrap();
        image::RgbImage::new(2000, 1000)
            .save(dir.path().join("media/eule.jpg"))
            .unwrap();
        image::RgbImage::new(300, 200)
            .save(dir.path().join("media/small.png"))
            .unwrap();

        let site = site::open(dir.path()).unwrap();
        let out = dir.path().join("_site");
        build_into(&site, &out).unwrap();
        let eule = read(out.join("eule/index.html"));

        assert!(
            eule.contains(
                r#"<button type="button" popovertarget="zoom-eule-jpg" class="block w-full cursor-zoom-in md:order-first">"#
            ),
            "{eule}"
        );
        assert!(
            eule.contains(
                r#"<img src="../media/eule.1600w.webp" alt="Eule" srcset="../media/eule.480w.webp 480w, ../media/eule.960w.webp 960w, ../media/eule.1600w.webp 1600w, ../media/eule.jpg 2000w" sizes="95vw" width="1600" height="800" loading="lazy">"#
            ),
            "the browser picks the size for its screen, up to the original: {eule}"
        );
        assert!(eule.contains(r#"aria-label="Schließen""#), "{eule}");
        assert_eq!(
            eule.matches("popovertarget=").count(),
            2,
            "one to open, one to close; the small image doesn't open: {eule}"
        );
    }

    #[test]
    fn images_in_text_get_copies_and_their_size() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("pages/blog")).unwrap();
        fs::create_dir_all(dir.path().join("media")).unwrap();
        fs::write(dir.path().join("site.toml"), "title = \"Photos\"\n").unwrap();
        fs::write(
            dir.path().join("pages/blog/trip.md"),
            "+++\ntitle = \"Trip\"\n+++\n\n![The lake](/media/wide.jpg)\n\n![Small](/media/small.png) ![Elsewhere](https://example.com/a.png)\n",
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
        let trip = read(out.join("blog/trip/index.html"));

        assert_eq!(report.resized_images, 3, "{trip}");
        assert!(out.join("media/wide.960w.webp").is_file());
        assert!(
            trip.contains(
                r#"<img loading="lazy" src="../../media/wide.jpg" srcset="../../media/wide.480w.webp 480w, ../../media/wide.960w.webp 960w, ../../media/wide.1600w.webp 1600w, ../../media/wide.jpg 2000w" sizes="(min-width: 56rem) 56rem, 100vw" width="2000" height="1000" alt="The lake""#
            ),
            "{trip}"
        );
        assert!(
            trip.contains(
                r#"<img loading="lazy" src="../../media/small.png" width="300" height="200" alt="Small""#
            ),
            "too small for copies: {trip}"
        );
        assert!(
            trip.contains(r#"<img loading="lazy" src="https://example.com/a.png" alt="Elsewhere""#),
            "{trip}"
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

        // The first image, as its 1600 px copy: a JPEG, which every link
        // preview reads, next to the WebP copy the page shows.
        let home = read(out.join("index.html"));
        assert_eq!(
            image::guess_format(&fs::read(out.join("media/wide.1600w.jpg")).unwrap()).unwrap(),
            image::ImageFormat::Jpeg
        );
        assert!(out.join("media/wide.1600w.webp").is_file());
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
    fn links_pages_in_a_series_back_and_next() {
        let dir = tempfile::tempdir().unwrap();
        let write = |file: &str, text: &str| {
            let path = dir.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        };
        write("site.toml", "title = \"Series\"\nlanguage = \"en\"\n");
        write("pages/index.md", "+++\ntitle = \"Home\"\n+++\n");
        write("pages/about.md", "+++\ntitle = \"About\"\n+++\n");
        write(
            "pages/manual/index.md",
            "+++\ntitle = \"Manual\"\nback_next = true\n+++\n",
        );
        write(
            "pages/manual/one.md",
            "+++\ntitle = \"One\"\nweight = 10\n+++\n",
        );
        write(
            "pages/manual/two.md",
            "+++\ntitle = \"Two\"\nweight = 20\n+++\n",
        );
        write(
            "pages/manual/three.md",
            "+++\ntitle = \"Three\"\nweight = 30\n+++\n",
        );
        write(
            "pages/manual/draft.md",
            "+++\ntitle = \"Draft\"\nweight = 25\ndraft = true\n+++\n",
        );
        write(
            "pages/blog/index.md",
            "+++\ntitle = \"Blog\"\nback_next = true\n+++\n",
        );
        write(
            "pages/blog/old.md",
            "+++\ntitle = \"Old\"\ndate = 2025-01-01\n+++\n",
        );
        write(
            "pages/blog/mid.md",
            "+++\ntitle = \"Mid\"\ndate = 2025-06-01\n+++\n",
        );
        write(
            "pages/blog/new.md",
            "+++\ntitle = \"New\"\ndate = 2025-12-01\n+++\n",
        );
        let site = site::open(dir.path()).unwrap();
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();
        let nav = |path: &str| {
            let html = read(out.path().join(path).join("index.html"));
            html.split_once(r#"<nav class="mt-16"#)
                .map(|(_, rest)| rest.split("</nav>").next().unwrap_or_default().to_string())
                .unwrap_or_default()
        };

        // Pages go by their order, drafts left out.
        let two = nav("manual/two");
        assert!(two.contains("← Back</span>"), "{two}");
        assert!(two.contains(r#"href="../../manual/one/""#), "{two}");
        assert!(two.contains("Next →</span>"), "{two}");
        assert!(
            two.contains(">Three</span>") && !two.contains("Draft"),
            "{two}"
        );
        let one = nav("manual/one");
        assert!(
            !one.contains("← Back") && one.contains(">Two</span>"),
            "{one}"
        );
        assert!(!nav("manual/three").contains("Next →"));

        // Posts: the older one on the left, the newer on the right.
        let mid = nav("blog/mid");
        assert!(
            mid.contains("← Older</span>") && mid.contains(">Old</span>"),
            "{mid}"
        );
        assert!(
            mid.contains("Newer →</span>") && mid.contains(">New</span>"),
            "{mid}"
        );
        assert!(mid.contains(r#"aria-label="Older and newer""#), "{mid}");

        // Only below a page that asks for it.
        assert!(nav("about").is_empty());
        assert!(nav("manual").is_empty());
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
            list.contains(r#"<span class="badge badge-secondary badge-sm self-start"><time datetime="2026-01-05">5. Jänner 2026</time></span>"#),
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
        // The only script is the site's own search.
        assert!(
            !home.contains("<iframe") && home.matches("<script").count() == 1,
            "{home}"
        );
        assert!(home.contains(r#"<script src="pagefind/pagefind-component-ui.js" defer>"#));
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
    fn refuses_links_that_leave_the_site_folder() {
        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        let about = site.pages.iter_mut().find(|p| p.path == "blog").unwrap();
        about.content.blocks.push(Block::Text {
            markdown: "See [the code](../../src/lib.rs).".into(),
        });
        let out = tempfile::tempdir().unwrap();
        let error = build_on(&site, out.path(), TODAY).unwrap_err().to_string();
        assert!(
            error.contains("pages/blog.md: the link \"../../src/lib.rs\" leads outside"),
            "{error}"
        );
    }

    #[test]
    fn uses_the_chosen_theme_colors() {
        let defaults = color_classes(&Colors::default(), false);
        assert_eq!(
            defaults.body, "bg-base-100",
            "links in primary need no class"
        );
        assert_eq!(defaults.footer, "on-color bg-accent text-accent-content");
        let dark = color_classes(&Colors::default(), true);
        assert_eq!(dark.footer, "bg-base-200 text-base-content");
        let chosen = Colors {
            footer: Some(Surface::Accent),
            ..Colors::default()
        };
        assert_eq!(
            color_classes(&chosen, true).footer,
            "on-color bg-accent text-accent-content",
            "a footer color that was picked wins"
        );
        assert_eq!(defaults.hero_button, "btn btn-primary");
        assert_eq!(
            defaults.call_to_action_button, "btn",
            "plain on its primary box"
        );

        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        site.config.colors = Colors {
            links: Some(ThemeColor::Accent),
            buttons: Some(ThemeColor::Secondary),
            hero: Some(Surface::Secondary),
            content: Some(Surface::Primary),
            footer: Some(Surface::Plain),
            ..Colors::default()
        };
        let home = site.pages.iter_mut().find(|p| p.path.is_empty()).unwrap();
        home.content.blocks.push(Block::Hero(block::Hero {
            headline: Some("Hi".into()),
            button_label: Some("Go".into()),
            button_link: Some("/blog".into()),
            ..block::Hero::default()
        }));
        home.content
            .blocks
            .push(Block::CallToAction(block::CallToAction {
                headline: Some("Join".into()),
                button_label: Some("Now".into()),
                button_link: Some("/blog".into()),
                ..block::CallToAction::default()
            }));
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();
        let html = read(out.path().join("index.html"));
        assert!(html.contains("content-primary links-accent"), "{html}");
        assert!(
            html.contains(
                r#"<footer class="border-t border-base-300 bg-base-100 text-base-content">"#
            ),
            "{html}"
        );
        assert!(
            html.contains("on-color bg-secondary text-secondary-content"),
            "the hero: {html}"
        );
        assert!(
            html.contains(r#"class="btn mt-8""#),
            "a secondary button on the secondary hero is plain: {html}"
        );
        assert!(
            html.contains(r#"class="btn btn-secondary">Now</a>"#),
            "on the primary box: {html}"
        );
    }

    #[test]
    fn shows_shared_blocks_with_each_pages_links() {
        let dir = tempfile::tempdir().unwrap();
        let site = blog(dir.path());
        fs::create_dir(dir.path().join("shared")).unwrap();
        fs::write(
            dir.path().join("shared/hours.md"),
            "+++\ntitle = \"Hours\"\n+++\n\nOpen **9–17**, see [the blog](/blog).\n",
        )
        .unwrap();
        let shared = "\n::: shared\nname = \"hours\"\n:::\n\n::: shared\nname = \"gone\"\n:::\n";
        let old = dir.path().join("pages/blog/2026-01-05-old.md");
        fs::write(&old, read(old.clone()) + shared).unwrap();
        let site = site::open(&site.root).unwrap();
        let out = tempfile::tempdir().unwrap();
        let report = build_on(&site, out.path(), TODAY).unwrap();

        let post = read(out.path().join("blog/2026-01-05-old/index.html"));
        assert!(post.contains("Open <strong>9–17</strong>"), "{post}");
        assert!(
            post.contains(r#"href="../../blog/""#),
            "this page's links: {post}"
        );
        assert!(
            !out.path().join("shared").exists(),
            "pieces aren't pages of their own"
        );
        assert!(
            report.notes.iter().any(|note| note.contains("\"gone\"")),
            "{:?}",
            report.notes
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
        assert!(
            deep.contains("<style>body{background:#ffffff;") && deep.contains("content=\"light\""),
            "the theme's colors: {deep}"
        );

        site.config.theme = "catppuccin-mocha".into();
        build_on(&site, out.path(), TODAY).unwrap();
        let dark = read(out.path().join("old.html"));
        assert!(
            dark.contains("<style>body{background:#1e1e2e;color:#cdd6f4;")
                && dark.contains("content=\"dark\""),
            "{dark}"
        );
    }

    #[test]
    fn shows_lists_as_cards_and_tiles_with_pictures() {
        let dir = tempfile::tempdir().unwrap();
        let write = |file: &str, text: &str| {
            let path = dir.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        };
        write("site.toml", "title = \"S\"\n");
        write(
            "pages/index.md",
            "+++\ntitle = \"Home\"\n+++\n\n::: post-list\nfrom = \"/blog\"\nlayout = \"cards\"\n:::\n\n::: page-list\nfrom = \"/animals\"\nlayout = \"tiles\"\n:::\n",
        );
        write("pages/blog.md", "+++\ntitle = \"Blog\"\n+++\n");
        write(
            "pages/blog/2026-01-05-trip.md",
            "+++\ntitle = \"Trip\"\ndate = 2026-01-05\nimage = \"trip.png\"\n+++\n",
        );
        write("pages/animals.md", "+++\ntitle = \"Animals\"\n+++\n");
        write(
            "pages/animals/gustaf.md",
            "+++\ntitle = \"Gustaf\"\nsummary = \"1220\"\n+++\n\n::: text-image\ntext = \"A giraffe.\"\nimage = \"gustaf.png\"\n:::\n",
        );
        fs::create_dir_all(dir.path().join("media")).unwrap();
        for name in ["trip.png", "gustaf.png"] {
            image::RgbImage::new(4, 3)
                .save(dir.path().join("media").join(name))
                .unwrap();
        }
        let site = site::open(dir.path()).unwrap();
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();

        let home = read(out.path().join("index.html"));
        assert!(
            home.contains("sm:grid-cols-2 lg:grid-cols-3"),
            "cards: {home}"
        );
        assert!(
            home.contains(r#"<img src="media/trip.png" alt="Trip""#),
            "share image: {home}"
        );
        assert!(
            home.contains("sm:grid-cols-4 lg:grid-cols-6"),
            "tiles: {home}"
        );
        assert!(
            home.contains(r#"<img src="media/gustaf.png" alt="Gustaf""#),
            "first image: {home}"
        );
        assert!(home.contains("1220"), "{home}");
    }

    #[test]
    fn hides_the_title_when_asked() {
        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        for page in &mut site.pages {
            if page.path == "blog/about" {
                page.content.front.hide_title = true;
            }
        }
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();
        let about = read(out.path().join("blog/about/index.html"));
        assert!(!about.contains("About the blog</h1>"), "{about}");
        assert!(
            about.contains("<title>About the blog"),
            "still the tab title: {about}"
        );
        let blog = read(out.path().join("blog/index.html"));
        assert!(blog.contains("Blog</h1>"), "{blog}");
    }

    #[test]
    fn gives_columns_their_widths() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("site.toml"), "title = \"S\"\n").unwrap();
        fs::create_dir_all(dir.path().join("pages")).unwrap();
        fs::write(
            dir.path().join("pages/index.md"),
            "+++\ntitle = \"Home\"\n+++\n\n:::: columns\n::: column\nwidth = 2\nboxed = true\n:::\n\nText.\n\n::: column\n:::\n\nPhoto.\n::::\n",
        )
        .unwrap();
        let site = site::open(dir.path()).unwrap();
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();
        let home = read(out.path().join("index.html"));
        assert!(
            home.contains(r#"<section class="grid gap-8 md:grid-cols-(--cols)" style="--cols: minmax(0,2fr) minmax(0,1fr)">"#),
            "{home}"
        );
        assert_eq!(
            home.matches(
                "rounded-box p-6 border border-base-300 bg-base-100 text-base-content shadow-sm"
            )
            .count(),
            1,
            "one boxed column: {home}"
        );
    }

    #[test]
    fn notes_links_to_pages_and_files_that_dont_exist() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("site.toml"), "title = \"S\"\n").unwrap();
        fs::create_dir_all(root.join("pages/blog")).unwrap();
        fs::create_dir_all(root.join("media")).unwrap();
        fs::create_dir_all(root.join("static/slides")).unwrap();
        fs::write(root.join("media/team.png"), "png").unwrap();
        fs::write(root.join("media/menu.pdf"), "pdf").unwrap();
        fs::write(
            root.join("pages/index.md"),
            "+++\ntitle = \"Home\"\n+++\n\n[ok](/about) [old](/old-about) [post](blog/post.md#top) \
             [slides](/slides/) [mail](mailto:a@b.at) [web](https://example.com)\n\n\
             ![team](/media/team.png) [menu](/assets/menu.pdf)\n\n\
             [typo](/abuot) [draft](/secret) [gone](blog/gone.md) ![x](/media/nope.png) \
             [y](/assets/nope.pdf) [typo again](/abuot)\n",
        )
        .unwrap();
        fs::write(
            root.join("pages/about.md"),
            "+++\ntitle = \"About\"\nredirect_from = [\"old-about\"]\n+++\n",
        )
        .unwrap();
        fs::write(
            root.join("pages/blog/post.md"),
            "+++\ntitle = \"Post\"\n+++\n",
        )
        .unwrap();
        fs::write(
            root.join("pages/secret.md"),
            "+++\ntitle = \"Secret\"\ndraft = true\n+++\n\n[missing](/nowhere)\n",
        )
        .unwrap();
        fs::write(
            root.join("footer.md"),
            "+++\ntitle = \"Footer\"\n+++\n\n[Contact](/contact)\n",
        )
        .unwrap();
        let site = site::open(root).unwrap();
        let report = build_into(&site, &root.join("_site")).unwrap();
        let notes: Vec<&String> = report
            .notes
            .iter()
            .filter(|n| n.contains(" links to "))
            .collect();
        assert_eq!(
            notes,
            [
                r#"/ links to "/abuot", but there's no such page."#,
                r#"/ links to "/secret", but that page is a draft and not on the website yet."#,
                r#"/ links to "blog/gone.md", but there's no such page."#,
                r#"/ links to "/media/nope.png", but that file isn't in Media."#,
                r#"/ links to "/assets/nope.pdf", but that file isn't in Downloads."#,
                r#"The footer links to "/contact", but there's no such page."#,
            ],
            "once each, drafts' own links unchecked"
        );
    }

    #[test]
    fn renders_sections() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("site.toml"), "title = \"S\"\n").unwrap();
        fs::create_dir_all(dir.path().join("pages")).unwrap();
        fs::write(
            dir.path().join("pages/index.md"),
            "+++\ntitle = \"Home\"\n+++\n\n::::: section\n::: settings\ntitle = \"Ours\"\nbackground = \"primary\"\nspacing = \"large\"\n:::\n\nIntro.\n\n:::: columns\n::: column\n:::\n\nLeft.\n::: column\n:::\n\nRight.\n::::\n:::::\n\n::::: section\nPlain.\n:::::\n",
        )
        .unwrap();
        let site = site::open(dir.path()).unwrap();
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();
        let home = read(out.path().join("index.html"));
        assert!(
            home.contains(r#"<section class="rounded-box px-6 py-12 sm:px-12 sm:py-20 on-color bg-primary text-primary-content on-primary">"#),
            "{home}"
        );
        assert!(home.contains(">Ours</h2>"), "{home}");
        assert!(
            home.contains("grid gap-8 md:grid-cols-2"),
            "columns inside: {home}"
        );
        assert!(home.contains(r#"<section class="">"#), "no fill: {home}");
        assert!(home.contains("Plain."), "{home}");
    }

    #[test]
    fn shows_a_logo_instead_of_the_site_title() {
        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        site.config.logo = Some("wordmark.svg".into());
        site.config.logo_size = LogoSize::Large;
        site.config.title_in_header = false;
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();
        let home = read(out.path().join("index.html"));
        let header = &home[home.find("<header").unwrap()..home.find("</header>").unwrap()];
        assert!(
            header.contains(r#"<img src="media/wordmark.svg" alt="Pod &#38; Co" class="h-16 w-auto max-w-80 object-contain">"#),
            "{header}"
        );
        assert!(!header.contains("Pod &#38; Co</a>"), "{header}");

        // Without a logo, the title stays whatever the setting says.
        site.config.logo = None;
        build_on(&site, out.path(), TODAY).unwrap();
        let home = read(out.path().join("index.html"));
        assert!(home.contains("Pod &#38; Co</a>"), "{home}");
    }

    #[test]
    fn writes_a_404_page_that_links_from_the_root() {
        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        site.config.language = "de".into();
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();
        let page = read(out.path().join("404.html"));
        assert!(page.contains("Seite nicht gefunden</h1>"), "{page}");
        assert!(
            page.contains(r#"<link rel="stylesheet" href="/style.css">"#),
            "{page}"
        );
        assert!(page.contains(r#"<a href="/">Zur Startseite</a>"#), "{page}");
        assert!(!page.contains("rel=\"canonical\""), "{page}");

        // A site in a subfolder links from there.
        site.config.base_url = Some("https://example.org/pod/".into());
        build_on(&site, out.path(), TODAY).unwrap();
        let page = read(out.path().join("404.html"));
        assert!(page.contains(r#"href="/pod/style.css""#), "{page}");
        assert!(page.contains(r#"<a href="/pod/">"#), "{page}");
    }

    #[test]
    fn uses_the_sites_own_404_page() {
        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        fs::write(
            dir.path().join("pages/404.md"),
            "+++\ntitle = \"Lost?\"\n+++\n\nTry the [blog](/blog).\n",
        )
        .unwrap();
        site = site::open(&site.root).unwrap();
        site.config.base_url = Some("https://example.org/".into());
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();
        let page = read(out.path().join("404.html"));
        assert!(page.contains("Lost?</h1>"), "{page}");
        assert!(page.contains(r#"<a href="/blog/">blog</a>"#), "{page}");
        assert!(
            !out.path().join("404/index.html").exists(),
            "not a normal page"
        );
        assert!(!read(out.path().join("sitemap.xml")).contains("/404"));
    }

    #[test]
    fn shows_a_hand_made_menu_with_dropdowns() {
        use crate::site::MenuItem;
        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        let page = |path: &str, title: Option<&str>| MenuItem {
            page: Some(path.into()),
            title: title.map(Into::into),
            ..MenuItem::default()
        };
        site.config.main_menu = vec![
            MenuItem {
                title: Some("More".into()),
                items: vec![
                    page("blog/about", Some("About us")),
                    MenuItem {
                        divider: true,
                        ..MenuItem::default()
                    },
                    MenuItem {
                        title: Some("Elsewhere".into()),
                        link: Some("https://example.org/".into()),
                        ..MenuItem::default()
                    },
                ],
                ..MenuItem::default()
            },
            page("blog", None),
            page("index", Some("Start")),
            page("gone", None),
            page("blog/2026-02-02-draft", None),
        ];
        let out = tempfile::tempdir().unwrap();
        let report = build_on(&site, out.path(), TODAY).unwrap();
        assert!(
            report.notes.contains(
                &"The menu has an entry for \"/gone\", but there's no such page; it's left out."
                    .to_string()
            ),
            "{:?}",
            report.notes
        );
        assert!(
            report
                .notes
                .iter()
                .any(|n| n.contains("\"/blog/2026-02-02-draft\", but that page is a draft")),
            "{:?}",
            report.notes
        );
        assert!(
            !report.notes.iter().any(|n| n.contains("\"/\"")),
            "the start page as `index` is found: {:?}",
            report.notes
        );
        let about = read(out.path().join("blog/about/index.html"));
        let header = &about[about.find("<header").unwrap()..about.find("</header>").unwrap()];
        assert!(header.contains("dropdown dropdown-hover"), "{header}");
        assert!(header.contains(r#"<a href="../../blog/about/" class="menu-active" aria-current="page">About us</a>"#), "{header}");
        assert!(
            header.contains(r#"<a href="https://example.org/">Elsewhere</a>"#),
            "{header}"
        );
        assert!(
            header.contains(">Blog</a>"),
            "page title by default: {header}"
        );
        assert!(
            header.contains(">Start</a>"),
            "`index` is the start page: {header}"
        );
        assert!(
            !header.contains("gone"),
            "missing pages are left out: {header}"
        );
        assert!(
            header.contains(r#"<li class="menu-title mt-2">More</li>"#),
            "phone menu heading: {header}"
        );
    }

    #[test]
    fn makes_pages_as_wide_as_asked() {
        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        let out = tempfile::tempdir().unwrap();
        for (width, class) in [
            (PageWidth::Normal, "max-w-4xl"),
            (PageWidth::Wide, "max-w-6xl"),
            (PageWidth::Full, "max-w-none"),
        ] {
            site.config.page_width = width;
            build_on(&site, out.path(), TODAY).unwrap();
            let home = read(out.path().join("index.html"));
            assert!(
                home.contains(&format!("mx-auto w-full {class} flex-1")),
                "{width:?}: {home}"
            );
        }

        site.config.logo = Some("logo.png".into());
        site.config.logo_size = LogoSize::Original;
        build_on(&site, out.path(), TODAY).unwrap();
        assert!(read(out.path().join("index.html")).contains("h-auto max-h-28 w-auto"));
    }

    #[test]
    fn marks_links_to_other_sites_when_asked() {
        let dir = tempfile::tempdir().unwrap();
        let mut site = blog(dir.path());
        site.config.base_url = Some("https://example.com/".into());
        let home = site.pages.iter_mut().find(|p| p.path.is_empty()).unwrap();
        home.content.blocks.push(Block::Text {
            markdown: "[Elsewhere](https://example.org/) and [ours](https://www.example.com/blog)"
                .into(),
        });
        let out = tempfile::tempdir().unwrap();
        build_on(&site, out.path(), TODAY).unwrap();
        let html = read(out.path().join("index.html"));
        assert!(!html.contains("mark-external"), "off by default: {html}");
        assert!(
            html.contains(r#"<a href="blog/">ours</a>"#),
            "a full link to the site itself stays inside: {html}"
        );

        site.config.mark_external_links = true;
        build_on(&site, out.path(), TODAY).unwrap();
        assert!(read(out.path().join("index.html")).contains("antialiased mark-external\""));
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

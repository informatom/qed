//! A site is a folder:
//!
//! ```text
//! my-site/
//!   site.toml          settings
//!   pages/
//!     index.md         /
//!     about.md         /about
//!     about/team.md    /about/team
//!   media/             images and downloads
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::fonts::Fonts;
use crate::links::LinkContext;
use crate::markdown;
use crate::page::{self, FrontMatter, PageContent, PageDate};
use crate::page_writer;
use crate::podcast::Podcast;
use crate::slug::slugify;
use crate::target::Target;
use crate::theme::CustomTheme;

pub const CONFIG_FILE: &str = "site.toml";
pub const PAGES_DIR: &str = "pages";
pub const MEDIA_DIR: &str = "media";
/// Files to download, such as PDFs. Images go into `media/`.
pub const ASSETS_DIR: &str = "assets";
/// Files copied into the built site as they are, at the same paths
/// (`static/pdf/a.pdf` becomes `/pdf/a.pdf`).
pub const STATIC_DIR: &str = "static";
/// The site-wide footer: blocks shown at the bottom of every page, in the
/// same format as a page file.
pub const FOOTER_FILE: &str = "footer.md";
/// How the footer appears in lists of pages that use a file or link to a
/// page; never a URL path, which can't start with `(`.
pub const FOOTER_PATH: &str = "(footer)";
/// The episode layout: the parts of every episode page and the blocks
/// shown on each, in the same format as a page file. Without it, episode
/// pages use [`crate::block::DEFAULT_EPISODE_LAYOUT`].
pub const EPISODE_LAYOUT_FILE: &str = "episode.md";
/// How the episode layout appears in lists of pages, like [`FOOTER_PATH`].
pub const EPISODE_LAYOUT_PATH: &str = "(episode layout)";
/// How a hand-made main menu appears in those lists.
pub const MENU_PATH: &str = "(menu)";

/// Settings from `site.toml`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct SiteConfig {
    pub title: String,
    /// Language code for the `lang` attribute, e.g. `"de"` or `"en"`.
    #[serde(default = "default_language")]
    pub language: String,
    pub description: Option<String>,
    /// An image in `media/`, shown before the title in every page's header
    /// and as the icon in the browser tab. A small square one (an SVG at
    /// best) works best.
    pub logo: Option<String>,
    /// A daisyUI theme name, or `custom` for `custom_theme`.
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Own colors on top of a theme, used when `theme` is `custom`. Kept
    /// when another theme is picked, so it can be picked again.
    pub custom_theme: Option<CustomTheme>,
    /// Named sets of custom colors, to switch between or go back to.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub saved_themes: Vec<CustomTheme>,
    /// Where the site is published, e.g. `"https://example.com/"`. Needed
    /// for the sitemap and canonical links; the site works without it.
    pub base_url: Option<String>,
    /// Where the main menu goes: the top bar, or a sidebar for sites whose
    /// main menu is too long for the top bar.
    #[serde(default)]
    pub navigation: Navigation,
    /// The header: plain, or a full-width bar in one of the theme's colors.
    #[serde(default)]
    pub header: HeaderStyle,
    /// Which theme color goes where: links, buttons, boxes, backgrounds.
    #[serde(default, skip_serializing_if = "Colors::is_default")]
    pub colors: Colors,
    /// How tall the logo is in the header.
    #[serde(default)]
    pub logo_size: LogoSize,
    /// How wide header, content and footer get on large screens.
    #[serde(default)]
    pub page_width: PageWidth,
    /// The site title next to the logo; off for logos that show the name.
    #[serde(default = "yes")]
    pub title_in_header: bool,
    /// Links in text that lead to other sites get a small icon after them.
    /// Off for sites from before it existed; new sites start with it on.
    #[serde(default)]
    pub mark_external_links: bool,
    /// Site search (Pagefind): a search button in the header, for
    /// visitors with JavaScript. On unless switched off.
    #[serde(default = "yes")]
    pub search: bool,
    /// Self-hosted fonts in `fonts/`; empty keeps the system's.
    #[serde(default, skip_serializing_if = "is_default_fonts")]
    pub fonts: Fonts,
    /// The main menu, arranged by hand: pages, links and groups (dropdowns).
    /// Empty means the automatic menu from the pages' Menu setting.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub main_menu: Vec<MenuItem>,
    /// The site's podcast, if it has one; see `podcast.rs`.
    pub podcast: Option<Podcast>,
}

/// One entry of a hand-made main menu: a page, a link, a group of entries
/// (a dropdown in the top bar), or a divider inside a group.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct MenuItem {
    /// The text shown; for a page, its title if empty.
    pub title: Option<String>,
    /// A page by its address (`elixir`, `` for the start page); follows the
    /// page when it moves.
    pub page: Option<String>,
    /// Any address: `https://…`, or one on the site such as `/blog`.
    pub link: Option<String>,
    /// A line between entries of a group.
    #[serde(default, skip_serializing_if = "is_false")]
    pub divider: bool,
    /// The entries of a group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<MenuItem>,
}

/// A menu entry's page as the pages' addresses have it, for entries typed
/// by hand: `/`, `index` and `index.md` are the start page (`""`), and
/// `/blog/`, `blog.md` and `blog/index` are `blog`, like the page files.
pub fn menu_page_path(written: &str) -> String {
    let path = written.trim().trim_matches('/');
    let path = path.strip_suffix(".md").unwrap_or(path);
    let path = if path == "index" {
        ""
    } else {
        path.strip_suffix("/index").unwrap_or(path)
    };
    path.to_string()
}

/// Writes every page entry of a hand-made menu the way the pages' addresses
/// are written (see `menu_page_path`), so the build, the Menu tab and moving
/// pages find them.
fn tidy_menu_pages(items: &mut [MenuItem]) {
    for item in items {
        if let Some(page) = &mut item.page {
            *page = menu_page_path(page);
        }
        tidy_menu_pages(&mut item.items);
    }
}

fn is_false(value: &bool) -> bool {
    !value
}

impl MenuItem {
    /// Why the entry can't be used, if it can't: `top` for entries in the
    /// bar itself, else inside a group.
    fn problem(&self, top: bool) -> Option<String> {
        let title = self.title.as_deref().map(str::trim).unwrap_or_default();
        let kinds = [
            self.page.is_some(),
            self.link.is_some(),
            self.divider,
            !self.items.is_empty(),
        ]
        .iter()
        .filter(|kind| **kind)
        .count();
        if self.divider {
            return (top || kinds > 1)
                .then(|| "a divider goes inside a group, on its own".to_string());
        }
        if !self.items.is_empty() {
            if !top {
                return Some(format!("“{title}”: groups can't hold groups"));
            }
            if kinds > 1 {
                return Some(format!("“{title}”: a group has no page or link of its own"));
            }
            if title.is_empty() {
                return Some("a group needs a title".to_string());
            }
            return self.items.iter().find_map(|item| item.problem(false));
        }
        match (&self.page, &self.link) {
            (Some(_), Some(_)) => Some(format!("“{title}”: a page or a link, not both")),
            (None, Some(link)) if link.trim().is_empty() || title.is_empty() => {
                Some("a link needs an address and a title".to_string())
            }
            (None, None) if title.is_empty() => Some("an empty menu entry".to_string()),
            (None, None) => Some(format!("“{title}” is a group without entries")),
            _ => None,
        }
    }
}

fn is_default_fonts(fonts: &Fonts) -> bool {
    *fonts == Fonts::default()
}

fn yes() -> bool {
    true
}

/// How wide pages get: normal (56rem, a good line length for text), wide
/// (72rem, for columns) or the full window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum PageWidth {
    #[default]
    Normal,
    Wide,
    Full,
}

impl PageWidth {
    pub fn as_str(self) -> &'static str {
        match self {
            PageWidth::Normal => "normal",
            PageWidth::Wide => "wide",
            PageWidth::Full => "full",
        }
    }
}

/// How tall the logo is in the header: 36, 48 or 64 pixels, or the image's
/// own size (up to 112 pixels), which keeps a bitmap logo sharp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum LogoSize {
    #[default]
    Small,
    Medium,
    Large,
    Original,
}

impl LogoSize {
    pub fn as_str(self) -> &'static str {
        match self {
            LogoSize::Small => "small",
            LogoSize::Medium => "medium",
            LogoSize::Large => "large",
            LogoSize::Original => "original",
        }
    }
}

/// How the page header looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum HeaderStyle {
    /// The page background, with a stripe in the accent color on top.
    #[default]
    Plain,
    Primary,
    Secondary,
    Accent,
    Neutral,
}

impl HeaderStyle {
    pub fn as_str(self) -> &'static str {
        match self {
            HeaderStyle::Plain => "plain",
            HeaderStyle::Primary => "primary",
            HeaderStyle::Secondary => "secondary",
            HeaderStyle::Accent => "accent",
            HeaderStyle::Neutral => "neutral",
        }
    }
}

/// Which of the theme's colors the site uses where (`[colors]` in
/// `site.toml`). Empty means the default, which is how sites looked before
/// these settings existed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Colors {
    /// The underline of links in text; primary if empty.
    pub links: Option<ThemeColor>,
    /// Buttons (Hero, Call to action); primary if empty. On a box of the
    /// same color they're drawn plain, so they don't vanish.
    pub buttons: Option<ThemeColor>,
    /// The Call to action box; primary if empty.
    pub call_to_action: Option<Surface>,
    /// A Hero without an image; secondary if empty.
    pub hero: Option<Surface>,
    /// Columns drawn as a box; plain (the page background, a border) if
    /// empty.
    pub boxes: Option<Surface>,
    /// The page background behind the content, a light tint of the color;
    /// plain if empty.
    pub content: Option<Surface>,
    /// The footer; accent if empty.
    pub footer: Option<Surface>,
}

impl Colors {
    pub fn is_default(&self) -> bool {
        *self == Colors::default()
    }
}

/// One of the theme's four colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum ThemeColor {
    Primary,
    Secondary,
    Accent,
    Neutral,
}

/// What an area is filled with: the page background (plain), a shade
/// darker (shaded), or one of the theme's colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum Surface {
    Plain,
    Shaded,
    Primary,
    Secondary,
    Accent,
    Neutral,
}

impl Surface {
    /// The theme color it's filled with, if any.
    pub fn color(self) -> Option<ThemeColor> {
        match self {
            Surface::Plain | Surface::Shaded => None,
            Surface::Primary => Some(ThemeColor::Primary),
            Surface::Secondary => Some(ThemeColor::Secondary),
            Surface::Accent => Some(ThemeColor::Accent),
            Surface::Neutral => Some(ThemeColor::Neutral),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum Navigation {
    #[default]
    Top,
    Sidebar,
}

impl SiteConfig {
    /// The custom theme, if the site uses one.
    pub fn custom(&self) -> Option<&CustomTheme> {
        self.custom_theme
            .as_ref()
            .filter(|_| self.theme == crate::theme::CUSTOM)
    }

    /// The daisyUI theme the pages name in `data-theme`: for a custom
    /// theme, the one it starts from.
    pub fn page_theme(&self) -> &str {
        self.custom().map_or(&self.theme, |custom| &custom.from)
    }
}

fn default_language() -> String {
    "en".to_string()
}

fn default_theme() -> String {
    "light".to_string()
}

/// A site folder, read into memory.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Site {
    pub root: PathBuf,
    pub config: SiteConfig,
    /// Sorted by URL path.
    pub pages: Vec<Page>,
    /// Where the site can be published to, from `targets.toml`.
    pub targets: Vec<Target>,
    /// The footer from `footer.md`, if the site has one; its `path` is
    /// [`FOOTER_PATH`].
    pub footer: Option<Page>,
    /// The episode layout from `episode.md`, if the site has one; its
    /// `path` is [`EPISODE_LAYOUT_PATH`].
    pub episode_layout: Option<Page>,
    /// The shared pieces from `shared/`, sorted by name; see `shared.rs`.
    pub shared: Vec<Page>,
    /// The podcast's contributors from `contributors/`, sorted by name;
    /// see `contributors.rs`.
    pub contributors: Vec<crate::contributors::Contributor>,
}

impl Site {
    /// The pages, the footer, the episode layout and the shared pieces:
    /// everything that holds blocks.
    pub fn with_blocks(&self) -> impl Iterator<Item = &Page> {
        self.pages
            .iter()
            .chain(self.footer.as_ref())
            .chain(self.episode_layout.as_ref())
            .chain(&self.shared)
    }
}

/// One page of a site.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Page {
    /// URL path without slashes at either end; `""` is the start page.
    pub path: String,
    /// The file, relative to the site folder.
    pub file: PathBuf,
    #[serde(flatten)]
    pub content: PageContent,
}

/// What a save from the editor did: saved, or not, because the file
/// changed since the editor read it (#105). Then `page` is the page as the
/// file is now, so the app can ask which version to keep.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "outcome", rename_all = "lowercase")]
pub enum SaveOutcome {
    Saved { page: Page },
    Changed { page: Page },
}

/// Everything that can go wrong when opening a site.
#[derive(Debug, thiserror::Error)]
pub enum SiteError {
    #[error("{} is not a qed site: it has no {CONFIG_FILE}", .0.display())]
    NotASite(PathBuf),

    #[error("can't read {}: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("{}: {source}", path.display())]
    Config {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("{}, line {}: {}", path.display(), error.line, error.message)]
    Page {
        path: PathBuf,
        error: page::ParseError,
    },

    #[error("{}: \"{name}\" can't be part of a URL; rename it to \"{suggestion}\"", path.display())]
    BadFileName {
        path: PathBuf,
        name: String,
        suggestion: String,
    },

    #[error("{} and {} both want the URL /{url}", first.display(), second.display())]
    DuplicatePath {
        url: String,
        first: PathBuf,
        second: PathBuf,
    },

    #[error("{} is not a page of this site", .0.display())]
    NotAPage(PathBuf),

    #[error("{}: {message}", path.display())]
    Invalid { path: PathBuf, message: String },

    #[error("can't save {}: {source}", path.display())]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// Reads the site in `root`: its settings and every page.
pub fn open(root: &Path) -> Result<Site, SiteError> {
    let config_path = root.join(CONFIG_FILE);
    if !config_path.is_file() {
        return Err(SiteError::NotASite(root.to_path_buf()));
    }
    let mut config: SiteConfig =
        toml::from_str(&read(&config_path)?).map_err(|source| SiteError::Config {
            path: config_path,
            source,
        })?;
    tidy_menu_pages(&mut config.main_menu);

    let mut files = Vec::new();
    let pages_dir = root.join(PAGES_DIR);
    if pages_dir.is_dir() {
        collect_markdown_files(&pages_dir, &mut files)?;
    }

    // A BTreeMap keeps its keys sorted, so the pages come out sorted by path.
    let mut pages: BTreeMap<String, Page> = BTreeMap::new();
    for file in files {
        let relative = file.strip_prefix(root).unwrap_or(&file).to_path_buf();
        let path = url_path(&file, &pages_dir, &relative)?;
        let content = page::parse(&read(&file)?).map_err(|error| SiteError::Page {
            path: relative.clone(),
            error,
        })?;

        if let Some(existing) = pages.get(&path) {
            return Err(SiteError::DuplicatePath {
                url: path,
                first: existing.file.clone(),
                second: relative,
            });
        }
        pages.insert(
            path.clone(),
            Page {
                path,
                file: relative,
                content,
            },
        );
    }

    Ok(Site {
        root: root.to_path_buf(),
        config,
        pages: pages.into_values().collect(),
        targets: crate::target::load(root)?,
        footer: load_part(root, FOOTER_FILE, FOOTER_PATH)?,
        episode_layout: load_part(root, EPISODE_LAYOUT_FILE, EPISODE_LAYOUT_PATH)?,
        shared: crate::shared::load(root)?,
        contributors: crate::contributors::load(root)?,
    })
}

/// A file of blocks outside `pages/`, such as `footer.md`, if the site
/// has it; `path` is how it shows in lists.
fn load_part(root: &Path, file: &str, path: &str) -> Result<Option<Page>, SiteError> {
    let full = root.join(file);
    if !full.is_file() {
        return Ok(None);
    }
    let content = page::parse(&read(&full)?).map_err(|error| SiteError::Page {
        path: PathBuf::from(file),
        error,
    })?;
    Ok(Some(Page {
        path: path.to_string(),
        file: PathBuf::from(file),
        content,
    }))
}

/// Saves the footer's blocks into `footer.md` and returns the footer as
/// saved. No blocks removes the file, and with it the footer.
pub fn save_footer(root: &Path, blocks: Vec<Block>) -> Result<Option<Page>, SiteError> {
    let blocks = without_empty_text(blocks);
    let path = root.join(FOOTER_FILE);
    if blocks.is_empty() {
        if path.exists() {
            fs::remove_file(&path).map_err(|source| SiteError::Write { path, source })?;
        }
        return Ok(None);
    }
    update_footer(root, |content| content.blocks = blocks).map(Some)
}

/// Saves the episode layout into `episode.md` and returns it as saved.
/// The default layout ([`crate::block::DEFAULT_EPISODE_LAYOUT`]) removes
/// the file: episode pages look the same without it. No blocks at all
/// stay a file, for episode pages with only their title.
pub fn save_episode_layout(root: &Path, blocks: Vec<Block>) -> Result<Option<Page>, SiteError> {
    let blocks = without_empty_text(blocks);
    let path = root.join(EPISODE_LAYOUT_FILE);
    if blocks == crate::block::DEFAULT_EPISODE_LAYOUT {
        if path.exists() {
            fs::remove_file(&path).map_err(|source| SiteError::Write { path, source })?;
        }
        return Ok(None);
    }
    update_episode_layout(root, |content| content.blocks = blocks).map(Some)
}

/// Like [`update_page`], for `episode.md`; makes the file if it's missing.
pub(crate) fn update_episode_layout(
    root: &Path,
    change: impl FnOnce(&mut PageContent),
) -> Result<Page, SiteError> {
    update_part(
        root,
        Path::new(EPISODE_LAYOUT_FILE),
        EPISODE_LAYOUT_PATH,
        "Episode layout",
        change,
    )
}

/// Like [`update_page`], for `footer.md`; makes the file if it's missing.
pub(crate) fn update_footer(
    root: &Path,
    change: impl FnOnce(&mut PageContent),
) -> Result<Page, SiteError> {
    update_part(root, Path::new(FOOTER_FILE), FOOTER_PATH, "Footer", change)
}

/// Like [`update_page`], for a file of blocks that isn't a page, such as
/// the footer or a shared piece: `path` is how it shows in lists, and a
/// missing file is made with `title`.
pub(crate) fn update_part(
    root: &Path,
    file: &Path,
    path: &str,
    title: &str,
    change: impl FnOnce(&mut PageContent),
) -> Result<Page, SiteError> {
    let full = root.join(file);
    let parse = |text: &str| {
        page::parse(text).map_err(|error| SiteError::Page {
            path: file.to_path_buf(),
            error,
        })
    };
    let original = if full.is_file() {
        Some(read(&full)?)
    } else {
        None
    };
    let mut content = match &original {
        Some(text) => parse(text)?,
        None => PageContent {
            front: FrontMatter {
                title: title.to_string(),
                ..FrontMatter::default()
            },
            blocks: Vec::new(),
        },
    };
    change(&mut content);
    let written = page_writer::write(&content, original.as_deref());
    if let Some(folder) = full.parent() {
        fs::create_dir_all(folder).map_err(|source| SiteError::Write {
            path: folder.to_path_buf(),
            source,
        })?;
    }
    write_atomically(&full, &written)?;
    Ok(Page {
        path: path.to_string(),
        file: file.to_path_buf(),
        content: parse(&written)?,
    })
}

/// Creates a new, empty page below `parent` and returns it.
///
/// `parent` is a URL path (`""` for the top level, `"blog"` for posts). The
/// file name is the slug of the title; with a `date`, the date comes first,
/// as blog posts usually do: `pages/blog/2026-10-21-hello.md`.
pub fn create_page(
    root: &Path,
    parent: &str,
    title: &str,
    date: Option<PageDate>,
) -> Result<Page, SiteError> {
    let title = title.trim();
    let slug = slugify(title);
    if slug.is_empty() {
        return Err(SiteError::Invalid {
            path: PathBuf::from(PAGES_DIR),
            message: "the title needs at least one letter or digit".to_string(),
        });
    }
    let name = match &date {
        Some(date) => format!("{date}-{slug}"),
        None => slug,
    };

    let parent = parent.trim_matches('/');
    let site = open(root)?;
    if !parent.is_empty() && !site.pages.iter().any(|p| p.path == parent) {
        return Err(SiteError::Invalid {
            path: PathBuf::from(PAGES_DIR),
            message: format!("there is no page /{parent} to put the new page under"),
        });
    }

    let path = if parent.is_empty() {
        name.clone()
    } else {
        format!("{parent}/{name}")
    };
    if let Some(existing) = site.pages.iter().find(|p| p.path == path) {
        return Err(SiteError::DuplicatePath {
            url: path,
            first: existing.file.clone(),
            second: PathBuf::from(PAGES_DIR).join(format!("{name}.md")),
        });
    }

    // Children of `blog` live in `pages/blog/`, whether the blog page itself
    // is `pages/blog.md` or `pages/blog/index.md`.
    let file = PathBuf::from(PAGES_DIR).join(&path).with_extension("md");
    let content = PageContent {
        front: FrontMatter {
            title: title.to_string(),
            date,
            ..FrontMatter::default()
        },
        blocks: Vec::new(),
    };
    write_new(&root.join(&file), &page_writer::write(&content, None))?;

    Ok(Page {
        path,
        file,
        content,
    })
}

/// Makes `root` a new site: `site.toml` with the title and language, a
/// start page with a Hero and a short text, and empty `media/` and
/// `assets/` folders. `root` may not exist yet; if it does, it must be
/// empty (hidden files such as `.git` are fine). Returns the new site.
pub fn create_site(root: &Path, title: &str, language: &str) -> Result<Site, SiteError> {
    let invalid = |message: String| SiteError::Invalid {
        path: root.to_path_buf(),
        message,
    };
    let title = title.trim();
    if title.is_empty() {
        return Err(invalid("a site needs a title".to_string()));
    }
    let language = match language.trim() {
        "" => default_language(),
        language => language.to_string(),
    };
    if root.join(CONFIG_FILE).exists() {
        return Err(invalid(
            "this folder is a qed site already; open it instead".to_string(),
        ));
    }
    if root.is_dir() {
        let visible: Vec<String> = fs::read_dir(root)
            .map_err(|source| SiteError::Io {
                path: root.to_path_buf(),
                source,
            })?
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .filter(|name| !name.starts_with('.'))
            .collect();
        if !visible.is_empty() {
            return Err(invalid(format!(
                "the folder isn't empty ({}); pick an empty one or make a new one",
                visible.join(", ")
            )));
        }
    }

    let mut config = toml_edit::DocumentMut::new();
    config["title"] = toml_edit::value(title);
    config["language"] = toml_edit::value(language.as_str());
    config["mark_external_links"] = toml_edit::value(true);
    write_new(&root.join(CONFIG_FILE), &config.to_string())?;

    let words = StartWords::for_language(&language);
    let start = PageContent {
        front: FrontMatter {
            title: words.title.to_string(),
            ..FrontMatter::default()
        },
        blocks: vec![
            Block::Hero(crate::block::Hero {
                headline: Some(title.to_string()),
                text: Some(words.welcome.to_string()),
                ..Default::default()
            }),
            Block::Text {
                markdown: words.text.to_string(),
            },
        ],
    };
    write_new(
        &root.join(PAGES_DIR).join("index.md"),
        &page_writer::write(&start, None),
    )?;
    for folder in [MEDIA_DIR, ASSETS_DIR] {
        let path = root.join(folder);
        fs::create_dir_all(&path).map_err(|source| SiteError::Write { path, source })?;
    }
    open(root)
}

/// An array of inline tables (`x = [{…}, {…}]`) as `[[x]]` sections, with
/// arrays of inline tables inside them turned the same way.
fn tables_from_inline(item: &mut toml_edit::Item) {
    let Some(array) = item.as_array_mut() else {
        return;
    };
    let mut tables = toml_edit::ArrayOfTables::new();
    for value in std::mem::take(array).into_iter() {
        if let toml_edit::Value::InlineTable(inline) = value {
            let mut table = inline.into_table();
            let keys: Vec<String> = table.iter().map(|(key, _)| key.to_string()).collect();
            for key in keys {
                if let Some(inner) = table.get_mut(&key)
                    && inner
                        .as_array()
                        .is_some_and(|a| a.iter().all(|v| v.is_inline_table()) && !a.is_empty())
                {
                    tables_from_inline(inner);
                }
            }
            tables.push(table);
        }
    }
    *item = toml_edit::Item::ArrayOfTables(tables);
}

/// The placeholder texts of a new site's start page.
struct StartWords {
    title: &'static str,
    welcome: &'static str,
    text: &'static str,
}

impl StartWords {
    /// In the site's language (`de`, `de-AT`, …); English for languages
    /// q.e.d. has no words for.
    fn for_language(language: &str) -> StartWords {
        let main = language.trim().to_lowercase();
        let main = main.split(['-', '_']).next().unwrap_or_default();
        let (title, welcome, text) = match main {
            "de" => (
                "Startseite",
                "Willkommen auf der neuen Website.",
                "Das ist die Startseite. Ändere diesen Text, füge Blöcke hinzu und lege in q.e.d. neue Seiten an.",
            ),
            "fr" => (
                "Accueil",
                "Bienvenue sur le nouveau site.",
                "Ceci est la page d'accueil. Modifiez ce texte, ajoutez des blocs et créez de nouvelles pages dans q.e.d.",
            ),
            "es" => (
                "Inicio",
                "Bienvenido al nuevo sitio.",
                "Esta es la página de inicio. Cambia este texto, añade bloques y crea nuevas páginas en q.e.d.",
            ),
            "it" => (
                "Home",
                "Benvenuti nel nuovo sito.",
                "Questa è la pagina iniziale. Modifica questo testo, aggiungi blocchi e crea nuove pagine in q.e.d.",
            ),
            "nl" => (
                "Home",
                "Welkom op de nieuwe website.",
                "Dit is de startpagina. Wijzig deze tekst, voeg blokken toe en maak nieuwe pagina's in q.e.d.",
            ),
            _ => (
                "Home",
                "Welcome to the new site.",
                "This is the start page. Change this text, add blocks, and make new pages in q.e.d.",
            ),
        };
        StartWords {
            title,
            welcome,
            text,
        }
    }
}

/// Writes a file that must not exist yet, creating its folder if needed.
pub(crate) fn write_new(path: &Path, contents: &str) -> Result<(), SiteError> {
    use std::io::Write;

    let error = |source| SiteError::Write {
        path: path.to_path_buf(),
        source,
    };
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder).map_err(error)?;
    }
    // `create_new` fails if the file appeared in the meantime, so nothing is
    // ever overwritten.
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(error)?;
    file.write_all(contents.as_bytes()).map_err(error)
}

/// Saves the site's settings into `site.toml`, keeping comments and the
/// formatting of everything that didn't change. Default values are left
/// out, as in page files. Returns the settings as saved, which may have
/// gained a `podcast:guid`.
pub fn save_config(root: &Path, config: &SiteConfig) -> Result<SiteConfig, SiteError> {
    let path = root.join(CONFIG_FILE);
    let invalid = |message: String| SiteError::Invalid {
        path: PathBuf::from(CONFIG_FILE),
        message,
    };
    if config.title.trim().is_empty() {
        return Err(invalid("a site needs a title".to_string()));
    }
    if config.theme == crate::theme::CUSTOM {
        let custom = config
            .custom_theme
            .as_ref()
            .ok_or_else(|| invalid("the custom theme has no colors yet".to_string()))?;
        custom.check().map_err(|e| invalid(e.to_string()))?;
    } else if !crate::theme::is_theme(&config.theme) {
        return Err(invalid(format!(
            "there is no theme called \"{}\"",
            config.theme
        )));
    }

    let mut names = std::collections::BTreeSet::new();
    for saved in &config.saved_themes {
        let name = saved.name.as_deref().map(str::trim).unwrap_or_default();
        if name.is_empty() {
            return Err(invalid("saved colors need a name".to_string()));
        }
        if !names.insert(name) {
            return Err(invalid(format!("two saved colors are called \"{name}\"")));
        }
        saved
            .check()
            .map_err(|e| invalid(format!("saved colors \"{name}\": {e}")))?;
    }

    if let Some(problem) = config.main_menu.iter().find_map(|item| item.problem(true)) {
        return Err(invalid(format!("menu: {problem}")));
    }
    if let Some(problem) = config.podcast.as_ref().and_then(Podcast::problem) {
        return Err(invalid(format!("podcast: {problem}")));
    }
    if let Some(person) = config
        .podcast
        .iter()
        .flat_map(|p| &p.people)
        .find(|person| {
            !root
                .join(crate::contributors::CONTRIBUTORS_DIR)
                .join(format!("{}.md", person.contributor))
                .is_file()
        })
    {
        return Err(invalid(format!(
            "podcast: there is no contributor “{}”",
            person.contributor
        )));
    }
    let saved = with_podcast_guid(config);
    let config = &saved;

    let original = read(&path)?;
    let mut document = toml_edit::ser::to_document(config).map_err(|e| invalid(e.to_string()))?;
    // `[custom_theme]`, `[fonts]`, `[colors]` and `[podcast]` as their own sections, easier
    // to read than one long line.
    for key in ["custom_theme", "fonts", "colors", "podcast"] {
        if let Some(inline) = document
            .get_mut(key)
            .and_then(|item| item.as_inline_table_mut())
        {
            let table = std::mem::take(inline).into_table();
            document[key] = toml_edit::Item::Table(table);
        }
    }
    // The menu as `[[main_menu]]` sections, a group's entries as
    // `[[main_menu.items]]` below it.
    if let Some(item) = document.get_mut("main_menu") {
        tables_from_inline(item);
    }
    if let Some(podcast) = document
        .get_mut("podcast")
        .and_then(|item| item.as_table_mut())
    {
        podcast_without_defaults(podcast);
        for key in ["funding", "people"] {
            if let Some(item) = podcast.get_mut(key) {
                tables_from_inline(item);
            }
        }
    }
    // And each saved set as a `[[saved_themes]]` section.
    if let Some(array) = document
        .get_mut("saved_themes")
        .and_then(|item| item.as_array_mut())
    {
        let mut tables = toml_edit::ArrayOfTables::new();
        for value in std::mem::take(array).into_iter() {
            if let toml_edit::Value::InlineTable(inline) = value {
                tables.push(inline.into_table());
            }
        }
        document["saved_themes"] = toml_edit::Item::ArrayOfTables(tables);
    }
    // Defaults stay out of the file, unless the file already names them.
    for (key, value, default) in [
        ("language", &config.language, default_language()),
        ("theme", &config.theme, default_theme()),
        (
            "navigation",
            &match config.navigation {
                Navigation::Top => "top".to_string(),
                Navigation::Sidebar => "sidebar".to_string(),
            },
            "top".to_string(),
        ),
        (
            "header",
            &config.header.as_str().to_string(),
            "plain".to_string(),
        ),
        (
            "logo_size",
            &config.logo_size.as_str().to_string(),
            "small".to_string(),
        ),
        (
            "page_width",
            &config.page_width.as_str().to_string(),
            "normal".to_string(),
        ),
        (
            "title_in_header",
            &config.title_in_header.to_string(),
            "true".to_string(),
        ),
        (
            "mark_external_links",
            &config.mark_external_links.to_string(),
            "false".to_string(),
        ),
        ("search", &config.search.to_string(), "true".to_string()),
    ] {
        if *value == default && !page_writer::writes_string(&original, key, &default) {
            document.remove(key);
        }
    }
    write_atomically(&path, &page_writer::merge(&original, document))?;
    crate::podcast::sync_audio_ignore(root, saved.podcast.as_ref())?;
    Ok(saved)
}

/// The settings with a `podcast:guid`, made from the feed address once
/// the podcast and the site's web address are both there. A guid the
/// podcast has stays as it is, even if the address changes later: it's
/// what keeps the podcast the same podcast for apps when it moves.
fn with_podcast_guid(config: &SiteConfig) -> SiteConfig {
    let mut config = config.clone();
    let base_url = config
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .map(str::to_string);
    if let (Some(podcast), Some(base_url)) = (config.podcast.as_mut(), base_url)
        && podcast
            .guid
            .as_deref()
            .is_none_or(|guid| guid.trim().is_empty())
    {
        // The main format's feed: the one Apple and the directories know.
        let feed = format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            crate::podcast::feed_file(podcast.main_format())
        );
        podcast.guid = Some(crate::podcast::podcast_guid(&feed));
    }
    config
}

/// Leaves the podcast settings at their defaults out of `[podcast]`, as
/// for the site's own settings.
fn podcast_without_defaults(podcast: &mut toml_edit::Table) {
    let defaults: [page_writer::DefaultCheck; 7] = [
        ("people", |item| {
            item.as_array().is_some_and(|array| array.is_empty())
        }),
        ("explicit", |item| item.as_bool() == Some(false)),
        ("type", |item| item.as_str() == Some("episodic")),
        ("audio_files", |item| item.as_str() == Some("local")),
        ("formats", |item| {
            item.as_array().is_some_and(|array| {
                array.len() == 1 && array.get(0).and_then(|v| v.as_str()) == Some("mp3")
            })
        }),
        ("categories", |item| {
            item.as_array().is_some_and(|array| array.is_empty())
        }),
        ("funding", |item| {
            item.as_array().is_some_and(|array| array.is_empty())
        }),
    ];
    for (key, is_default) in defaults {
        if podcast.get(key).is_some_and(is_default) {
            podcast.remove(key);
        }
    }
}

/// Saves new settings for a page and returns the page as saved.
///
/// The file is read again just before saving, so blocks someone edited by
/// hand in the meantime are kept. `file` is relative to the site folder,
/// e.g. `pages/about.md`.
pub fn save_front_matter(root: &Path, file: &Path, front: FrontMatter) -> Result<Page, SiteError> {
    if front.title.trim().is_empty() {
        return Err(SiteError::Invalid {
            path: file.to_path_buf(),
            message: "a page needs a title".to_string(),
        });
    }
    let mut front = front;
    if let Some(episode) = front.episode.as_mut() {
        if let Some(problem) = episode.problem() {
            return Err(SiteError::Invalid {
                path: file.to_path_buf(),
                message: problem,
            });
        }
        if let Some(person) = episode.people.iter().find(|person| {
            !root
                .join(crate::contributors::CONTRIBUTORS_DIR)
                .join(format!("{}.md", person.contributor))
                .is_file()
        }) {
            return Err(SiteError::Invalid {
                path: file.to_path_buf(),
                message: format!(
                    "there is no contributor “{}” in {}/",
                    person.contributor,
                    crate::contributors::CONTRIBUTORS_DIR
                ),
            });
        }
        // An episode written by hand may come without one.
        episode.guid = episode.guid.trim().to_string();
        if episode.guid.is_empty() {
            episode.guid = crate::podcast::new_guid();
        } else {
            check_episode_guid(root, file, &episode.guid)?;
        }
    }
    front.redirect_from = front
        .redirect_from
        .iter()
        .filter_map(|address| old_address(address))
        .fold(Vec::new(), |mut kept, address| {
            if !kept.contains(&address) {
                kept.push(address);
            }
            kept
        });
    check_old_addresses(root, file, &front.redirect_from)?;
    update_page(root, file, |content| content.front = front)
}

/// An old address as `redirect_from` keeps it: `/2016/03/28/easterhegg.html`.
/// A full web address (pasted from the old site) keeps only its path;
/// `#…` and `?…` are dropped, since a redirect page can't tell them apart.
/// `None` for an empty one.
pub fn old_address(written: &str) -> Option<String> {
    let mut address = written.trim();
    if let Some((_, rest)) = address.split_once("://") {
        address = rest.find('/').map_or("", |slash| &rest[slash..]);
    }
    let address = address.split(['#', '?']).next().unwrap_or_default();
    let address = address.trim_matches('/');
    (!address.is_empty()).then(|| format!("/{address}"))
}

/// Refuses a guid that changed (one carried over from an old feed, typed
/// in the Settings tab) if another episode has it already: podcast apps
/// would take the two for one.
fn check_episode_guid(root: &Path, file: &Path, guid: &str) -> Result<(), SiteError> {
    let before = read_page(root, file)?.content.front.episode.map(|e| e.guid);
    if before.as_deref() == Some(guid) {
        return Ok(());
    }
    let site = open(root)?;
    if let Some(other) = site.pages.iter().find(|page| {
        page.file != file
            && page
                .content
                .front
                .episode
                .as_ref()
                .is_some_and(|episode| episode.guid.trim() == guid)
    }) {
        return Err(SiteError::Invalid {
            path: file.to_path_buf(),
            message: format!(
                "the episode “{}” has the guid “{guid}” already",
                other.content.front.title
            ),
        });
    }
    Ok(())
}

/// Refuses old addresses that are new in `addresses` (not yet in the file)
/// and belong to another page already: as its address, or as one of its
/// old addresses. The build would stop on the first, and the second would
/// lead visitors to one of the two pages at random.
fn check_old_addresses(root: &Path, file: &Path, addresses: &[String]) -> Result<(), SiteError> {
    let before = read_page(root, file)?.content.front.redirect_from;
    let added: Vec<&String> = addresses
        .iter()
        .filter(|address| !before.contains(address))
        .collect();
    if added.is_empty() {
        return Ok(());
    }
    let site = open(root)?;
    let invalid = |message: String| SiteError::Invalid {
        path: file.to_path_buf(),
        message,
    };
    for address in added {
        let path = address.trim_matches('/');
        for page in &site.pages {
            let title = &page.content.front.title;
            if page.path == path {
                return Err(invalid(if page.file == file {
                    format!("“{address}” is this page's own address")
                } else {
                    format!("“{address}” is the address of the page “{title}”")
                }));
            }
            if page.file != file
                && page
                    .content
                    .front
                    .redirect_from
                    .iter()
                    .any(|other| other.trim_matches('/') == path)
            {
                return Err(invalid(format!(
                    "“{address}” already leads to the page “{title}”"
                )));
            }
        }
    }
    Ok(())
}

/// Saves a page's blocks and returns the page as saved.
///
/// Empty text blocks are dropped, and text blocks that follow each other are
/// joined, because that's how the file will read back. The returned page is
/// what the file now contains.
pub fn save_blocks(root: &Path, file: &Path, blocks: Vec<Block>) -> Result<Page, SiteError> {
    let blocks = without_empty_text(blocks);
    update_page(root, file, |content| content.blocks = blocks)
}

/// Puts pages in the order given by setting their weights to 10, 20,
/// 30, …; the steps leave room for a page placed by hand in between.
/// Pages that already have their weight aren't written. Returns how many
/// files changed.
pub fn set_order(site: &Site, files: &[PathBuf]) -> Result<usize, SiteError> {
    let mut changed = 0;
    for (index, file) in files.iter().enumerate() {
        let page = site
            .pages
            .iter()
            .find(|page| &page.file == file)
            .ok_or_else(|| SiteError::NotAPage(file.clone()))?;
        let weight = i32::try_from(index + 1).unwrap_or(i32::MAX / 10) * 10;
        if page.content.front.weight != weight {
            update_page(&site.root, file, |content| content.front.weight = weight)?;
            changed += 1;
        }
    }
    Ok(changed)
}

/// Leaves out text blocks with nothing in them, inside columns and
/// sections too. A file can't hold them anyway: they'd read back as
/// nothing.
pub(crate) fn without_empty_text(blocks: Vec<Block>) -> Vec<Block> {
    crate::block::retain_deep(
        blocks,
        &|block| !matches!(block, Block::Text { markdown } if markdown.trim().is_empty()),
    )
}

/// The pages that link to the page at `path` (URL paths, sorted), from
/// Markdown links and buttons.
pub fn pages_linking_to(site: &Site, path: &str) -> Vec<String> {
    let mut found: Vec<String> = site
        .with_blocks()
        .filter(|page| page.path != path)
        .filter(|page| {
            let links = LinkContext::new(&page.path, &page.file);
            link_targets(&page.content.blocks)
                .iter()
                .any(|target| links.page_path_of(target).as_deref() == Some(path))
        })
        .map(|page| page.path.clone())
        .collect();
    if menu_links_to(&site.config.main_menu, path) {
        found.push(MENU_PATH.to_string());
    }
    found.sort();
    found
}

/// Every link written in a list of blocks: Markdown links and buttons.
pub(crate) fn link_targets(blocks: &[Block]) -> Vec<String> {
    let mut texts: Vec<&str> = Vec::new();
    let mut targets = Vec::new();
    for block in crate::block::all_blocks(blocks) {
        match block {
            Block::Text { markdown } => texts.push(markdown),
            Block::Hero(hero) => {
                texts.extend(hero.text.as_deref());
                targets.extend(hero.button_link.clone());
            }
            Block::CallToAction(cta) => {
                texts.extend(cta.text.as_deref());
                targets.extend(cta.button_link.clone());
            }
            Block::Image(image) => texts.extend(image.caption.as_deref()),
            Block::TextImage(block) => texts.extend(block.text.as_deref()),
            Block::Faq(faq) => texts.extend(faq.items.iter().map(|item| item.answer.as_str())),
            // The page it lists counts as a link, so moving or deleting
            // that page knows about the list.
            Block::PostList(list) => targets.extend(list.from.as_deref().map(post_list_link)),
            Block::PageList(list) => targets.extend(list.from.as_deref().map(post_list_link)),
            Block::Gallery(_)
            | Block::Download(_)
            | Block::Columns(_)
            | Block::Section(_)
            | Block::Video(_)
            | Block::Shared(_)
            | Block::LatestEpisode
            | Block::Subscribe(_)
            | Block::Contributors
            | Block::EpisodePlayer
            | Block::EpisodePeople
            | Block::EpisodeShownotes
            | Block::EpisodeChapters
            | Block::EpisodeTranscript
            | Block::EpisodeShare => {}
        }
    }
    for text in texts {
        targets.extend(markdown::link_targets(text));
    }
    targets
}

/// Whether a hand-made menu has an entry for the page at `path`.
fn menu_links_to(items: &[MenuItem], path: &str) -> bool {
    items.iter().any(|item| {
        item.page.as_deref().map(menu_page_path).as_deref() == Some(path)
            || item.link.as_deref().is_some_and(|link| {
                link.strip_prefix('/').map(|l| {
                    l.split(['#', '?'])
                        .next()
                        .unwrap_or_default()
                        .trim_end_matches('/')
                }) == Some(path)
            })
            || menu_links_to(&item.items, path)
    })
}

/// A Post list's or Page list's `from` as a link: `/blog`.
pub(crate) fn post_list_link(from: &str) -> String {
    format!("/{}", from.trim().trim_matches('/'))
}

/// Deletes a page's file. The start page and pages with pages below them
/// can't be deleted.
pub fn delete_page(site: &Site, file: &Path) -> Result<(), SiteError> {
    let full = page_file(&site.root, file)?;
    let invalid = |message: String| SiteError::Invalid {
        path: file.to_path_buf(),
        message,
    };
    let page = site
        .pages
        .iter()
        .find(|page| page.file == file)
        .ok_or_else(|| SiteError::NotAPage(file.to_path_buf()))?;
    if page.path.is_empty() {
        return Err(invalid("the start page can't be deleted".to_string()));
    }
    let below: Vec<String> = site
        .pages
        .iter()
        .filter(|other| other.path.starts_with(&format!("{}/", page.path)))
        .map(|other| format!("/{}", other.path))
        .collect();
    if !below.is_empty() {
        return Err(invalid(format!(
            "it has pages below it ({}); delete or move those first",
            below.join(", ")
        )));
    }
    fs::remove_file(&full).map_err(|source| SiteError::Write {
        path: full.clone(),
        source,
    })?;
    // A page kept as `about/index.md` leaves an empty folder behind.
    if let Some(folder) = full.parent()
        && folder != site.root.join(PAGES_DIR)
    {
        let _ = fs::remove_dir(folder); // Only succeeds if it's empty.
    }
    Ok(())
}

/// A page as its file is now.
pub fn read_page(root: &Path, file: &Path) -> Result<Page, SiteError> {
    let full = page_file(root, file)?;
    let content = page::parse(&read(&full)?).map_err(|error| SiteError::Page {
        path: file.to_path_buf(),
        error,
    })?;
    Ok(Page {
        path: url_path(&full, &root.join(PAGES_DIR), file)?,
        file: file.to_path_buf(),
        content,
    })
}

/// [`save_blocks`], but only if the file's blocks are still `base`, the
/// ones the editor started from. Otherwise someone else (an assistant, a
/// text editor) changed them meanwhile, and saving would undo that.
/// Without `base`, saves anyway: "Keep mine".
pub fn save_blocks_checked(
    root: &Path,
    file: &Path,
    blocks: Vec<Block>,
    base: Option<&[Block]>,
) -> Result<SaveOutcome, SiteError> {
    if let Some(base) = base {
        let now = read_page(root, file)?;
        if now.content.blocks != base {
            return Ok(SaveOutcome::Changed { page: now });
        }
    }
    save_blocks(root, file, blocks).map(|page| SaveOutcome::Saved { page })
}

/// [`save_front_matter`], but only if the file's settings are still
/// `base`; see [`save_blocks_checked`].
pub fn save_front_matter_checked(
    root: &Path,
    file: &Path,
    front: FrontMatter,
    base: Option<&FrontMatter>,
) -> Result<SaveOutcome, SiteError> {
    if let Some(base) = base {
        let now = read_page(root, file)?;
        if now.content.front != *base {
            return Ok(SaveOutcome::Changed { page: now });
        }
    }
    save_front_matter(root, file, front).map(|page| SaveOutcome::Saved { page })
}

/// Reads a page file, lets `change` modify its content, and saves it.
///
/// The file is read again right before saving, so whatever someone edited
/// by hand in the meantime, outside of what `change` touches, is kept.
/// Returns the page as it reads back from the saved file.
pub(crate) fn update_page(
    root: &Path,
    file: &Path,
    change: impl FnOnce(&mut PageContent),
) -> Result<Page, SiteError> {
    let full = page_file(root, file)?;
    let parse = |text: &str| {
        page::parse(text).map_err(|error| SiteError::Page {
            path: file.to_path_buf(),
            error,
        })
    };

    let original = read(&full)?;
    let mut content = parse(&original)?;
    change(&mut content);
    let written = page_writer::write(&content, Some(&original));
    write_atomically(&full, &written)?;

    Ok(Page {
        path: url_path(&full, &root.join(PAGES_DIR), file)?,
        file: file.to_path_buf(),
        content: parse(&written)?,
    })
}

/// Checks that `file` names a page file inside the site's `pages/` folder,
/// and returns its full path. Paths from the UI pass through here, so `..`
/// and absolute paths are refused.
fn page_file(root: &Path, file: &Path) -> Result<PathBuf, SiteError> {
    let plain = file.components().all(|c| matches!(c, Component::Normal(_)));
    let in_pages = file.starts_with(PAGES_DIR);
    let markdown = file.extension().is_some_and(|ext| ext == "md");
    let full = root.join(file);
    if plain && in_pages && markdown && full.is_file() {
        Ok(full)
    } else {
        Err(SiteError::NotAPage(file.to_path_buf()))
    }
}

/// Writes a file so that it's never half-written: first to a temporary file
/// next to it, then renamed over the original in one step.
pub(crate) fn write_atomically(path: &Path, contents: &str) -> Result<(), SiteError> {
    let error = |source| SiteError::Write {
        path: path.to_path_buf(),
        source,
    };
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".qed-saving");
    let temporary = PathBuf::from(temporary);
    fs::write(&temporary, contents).map_err(error)?;
    fs::rename(&temporary, path).map_err(error)
}

pub(crate) fn read(path: &Path) -> Result<String, SiteError> {
    fs::read_to_string(path).map_err(|source| SiteError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Adds every `.md` file below `dir` to `files`, descending into folders.
fn collect_markdown_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), SiteError> {
    let io_error = |source| SiteError::Io {
        path: dir.to_path_buf(),
        source,
    };
    for entry in fs::read_dir(dir).map_err(io_error)? {
        let path = entry.map_err(io_error)?.path();
        if path.is_dir() {
            collect_markdown_files(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path);
        }
    }
    Ok(())
}

/// Works out a page's URL path from its file: `pages/about/team.md` becomes
/// `about/team`, and `index.md` stands for its folder.
fn url_path(file: &Path, pages_dir: &Path, relative: &Path) -> Result<String, SiteError> {
    let inside = file
        .strip_prefix(pages_dir)
        .unwrap_or(file)
        .with_extension("");
    let mut segments = Vec::new();

    for component in inside.components() {
        let name = component.as_os_str().to_string_lossy();
        let suggestion = slugify(&name);
        if name != suggestion {
            return Err(SiteError::BadFileName {
                path: relative.to_path_buf(),
                name: name.into_owned(),
                suggestion,
            });
        }
        segments.push(name.into_owned());
    }

    if segments.last().is_some_and(|last| last == "index") {
        segments.pop();
    }
    Ok(segments.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_site_in_an_empty_or_new_folder() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        let site = create_site(dir.path(), "  My Podcast ", "de").unwrap();
        assert_eq!(site.config.title, "My Podcast");
        assert_eq!(site.config.language, "de");
        assert!(site.config.mark_external_links, "new sites mark them");
        assert_eq!(site.pages.len(), 1);
        let home = &site.pages[0];
        assert_eq!(home.path, "");
        assert!(matches!(&home.content.blocks[0], Block::Hero(hero)
            if hero.headline.as_deref() == Some("My Podcast")
                && hero.text.as_deref() == Some("Willkommen auf der neuen Website.")));
        assert_eq!(home.content.front.title, "Startseite");
        assert!(dir.path().join(MEDIA_DIR).is_dir());
        assert!(dir.path().join(ASSETS_DIR).is_dir());

        let new = dir.path().join("sites/second");
        let second = create_site(&new, "Second", "").unwrap();
        assert_eq!(second.config.language, "en");
        assert_eq!(second.pages[0].content.front.title, "Home");
        let third = create_site(&dir.path().join("sites/third"), "Third", "de-AT").unwrap();
        assert_eq!(
            third.pages[0].content.front.title, "Startseite",
            "de-AT is German"
        );
    }

    #[test]
    fn creates_sites_only_in_empty_folders() {
        let dir = tempfile::tempdir().unwrap();
        assert!(create_site(dir.path(), " ", "en").is_err());
        fs::write(dir.path().join("notes.txt"), "mine").unwrap();
        let error = create_site(dir.path(), "Site", "en").unwrap_err();
        assert!(error.to_string().contains("notes.txt"), "{error}");
        assert_eq!(
            fs::read_to_string(dir.path().join("notes.txt")).unwrap(),
            "mine"
        );

        let site = tempfile::tempdir().unwrap();
        create_site(site.path(), "Site", "en").unwrap();
        let again = create_site(site.path(), "Site", "en").unwrap_err();
        assert!(again.to_string().contains("already"), "{again}");
    }
    use crate::page::Menu;

    /// Creates a site folder with the given files in a temporary directory.
    fn site_with(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (name, content) in files {
            let path = dir.path().join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, content).unwrap();
        }
        dir
    }

    const PAGE: &str = "+++\ntitle = \"T\"\n+++\n";

    #[test]
    fn opens_the_demo_site() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/demo-site");
        let site = open(&root).unwrap();

        assert_eq!(site.config.title, "Two Mics and a Plan");
        assert_eq!(site.config.language, "en");
        let paths: Vec<&str> = site.pages.iter().map(|p| p.path.as_str()).collect();
        assert_eq!(paths, ["", "about", "about/team", "imprint"]);

        let imprint = site.pages.iter().find(|p| p.path == "imprint").unwrap();
        assert_eq!(imprint.content.front.menu, Menu::Footer);
    }

    /// The editor sends back a page's blocks and settings as it got them,
    /// and saving compares them with the file (`save_blocks_checked`). So
    /// they must come back from JSON unchanged, or every save would look
    /// like a change made outside. Checked on every page of the docs and
    /// the demo site.
    #[test]
    fn pages_come_back_from_the_editor_unchanged() {
        for folder in ["../../docs", "../../examples/demo-site"] {
            let site = open(&Path::new(env!("CARGO_MANIFEST_DIR")).join(folder)).unwrap();
            for page in site.with_blocks() {
                let content = &page.content;
                let blocks: Vec<Block> =
                    serde_json::from_str(&serde_json::to_string(&content.blocks).unwrap()).unwrap();
                assert_eq!(blocks, content.blocks, "{}", page.file.display());
                let front: FrontMatter =
                    serde_json::from_str(&serde_json::to_string(&content.front).unwrap()).unwrap();
                assert_eq!(front, content.front, "{}", page.file.display());
            }
        }
    }

    /// The project's own docs folder is a qed site. This keeps it valid.
    #[test]
    fn opens_the_docs_site() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
        let site = open(&root).unwrap_or_else(|error| panic!("{error}"));

        assert_eq!(site.config.title, "q.e.d.");
        let posts: Vec<_> = site
            .pages
            .iter()
            .filter(|p| p.path.starts_with("blog/"))
            .collect();
        assert!(!posts.is_empty());
        for post in posts {
            assert!(
                post.content.front.date.is_some(),
                "{} has no date",
                post.file.display()
            );
        }
    }

    #[test]
    fn saves_new_settings_and_keeps_the_rest() {
        let dir = site_with(&[
            ("site.toml", "title = \"S\""),
            (
                "pages/about.md",
                "+++\n# A comment\ntitle = \"About\"\n+++\n\nHello.\n",
            ),
        ]);
        let mut front = open(dir.path()).unwrap().pages[0].content.front.clone();
        front.title = "About us".into();
        front.menu = Menu::Main;

        let page = save_front_matter(dir.path(), Path::new("pages/about.md"), front).unwrap();
        assert_eq!(page.path, "about");
        assert_eq!(page.content.front.title, "About us");

        let text = fs::read_to_string(dir.path().join("pages/about.md")).unwrap();
        assert_eq!(
            text,
            "+++\n# A comment\ntitle = \"About us\"\nmenu = \"main\"\n+++\n\nHello.\n"
        );
    }

    #[test]
    fn saves_only_what_nobody_changed_outside_meanwhile() {
        let dir = site_with(&[
            ("site.toml", "title = \"S\""),
            ("pages/a.md", "+++\ntitle = \"A\"\n+++\n\nMine.\n"),
        ]);
        let (root, file) = (dir.path(), Path::new("pages/a.md"));
        let opened = read_page(root, file).unwrap();
        let text = |markdown: &str| Block::Text {
            markdown: markdown.into(),
        };

        // Nothing changed outside: saved.
        let outcome = save_blocks_checked(
            root,
            file,
            vec![text("Mine, edited.")],
            Some(&opened.content.blocks),
        )
        .unwrap();
        let SaveOutcome::Saved { page } = outcome else {
            panic!("{outcome:?}")
        };

        // An assistant changes the text and the title meanwhile.
        fs::write(root.join(file), "+++\ntitle = \"Theirs\"\n+++\n\nTheirs.\n").unwrap();
        let outcome = save_blocks_checked(
            root,
            file,
            vec![text("Mine again.")],
            Some(&page.content.blocks),
        )
        .unwrap();
        let SaveOutcome::Changed { page: theirs } = outcome else {
            panic!("{outcome:?}")
        };
        assert_eq!(theirs.content.blocks, [text("Theirs.")]);
        assert!(
            read(&root.join(file)).unwrap().contains("Theirs."),
            "not overwritten"
        );

        // Settings started from "A", now "Theirs": changed too.
        let mut front = opened.content.front.clone();
        front.title = "Mine".into();
        let outcome =
            save_front_matter_checked(root, file, front.clone(), Some(&opened.content.front))
                .unwrap();
        assert!(
            matches!(outcome, SaveOutcome::Changed { .. }),
            "{outcome:?}"
        );

        // "Keep mine": without a base it saves anyway, and only its part.
        let outcome = save_front_matter_checked(root, file, front, None).unwrap();
        let SaveOutcome::Saved { page } = outcome else {
            panic!("{outcome:?}")
        };
        assert_eq!(page.content.front.title, "Mine");
        assert_eq!(page.content.blocks, [text("Theirs.")], "their text stays");
    }

    #[test]
    fn saves_blocks_and_returns_them_as_the_file_reads_back() {
        let dir = site_with(&[
            ("site.toml", "title = \"S\""),
            (
                "pages/a.md",
                "+++\ntitle = \"A\" # kept\n+++\n\nOld text.\n",
            ),
        ]);
        let blocks = vec![
            Block::Hero(crate::block::Hero {
                headline: Some("Hi".into()),
                ..Default::default()
            }),
            Block::Text {
                markdown: "One.".into(),
            },
            Block::Text {
                markdown: "  ".into(),
            },
            Block::Text {
                markdown: "Two.".into(),
            },
        ];

        let page = save_blocks(dir.path(), Path::new("pages/a.md"), blocks).unwrap();
        assert_eq!(page.content.blocks.len(), 2, "{:?}", page.content.blocks);
        assert_eq!(
            page.content.blocks[1],
            Block::Text {
                markdown: "One.\n\nTwo.".into()
            }
        );
        let text = fs::read_to_string(dir.path().join("pages/a.md")).unwrap();
        assert_eq!(
            text,
            "+++\ntitle = \"A\" # kept\n+++\n\n::: hero\nheadline = \"Hi\"\n:::\n\nOne.\n\nTwo.\n"
        );
    }

    #[test]
    fn reads_blocks_sent_by_the_ui() {
        let json = r#"[{"type":"text","markdown":"Hi"},{"type":"call-to-action","headline":"Go","text":null,"button_label":null,"button_link":null}]"#;
        let blocks: Vec<Block> = serde_json::from_str(json).unwrap();
        assert_eq!(blocks.len(), 2);
        assert!(serde_json::from_str::<Vec<Block>>(r#"[{"type":"hero","colour":"red"}]"#).is_err());
    }

    #[test]
    fn refuses_to_save_outside_the_pages_folder() {
        let dir = site_with(&[("site.toml", "title = \"S\""), ("pages/a.md", PAGE)]);
        let front = open(dir.path()).unwrap().pages[0].content.front.clone();
        for file in [
            "site.toml",
            "pages/../site.toml",
            "/etc/passwd",
            "pages/missing.md",
        ] {
            let result = save_front_matter(dir.path(), Path::new(file), front.clone());
            assert!(matches!(result, Err(SiteError::NotAPage(_))), "{file}");
        }
    }

    #[test]
    fn creates_pages_and_posts() {
        let dir = site_with(&[
            ("site.toml", "title = \"S\""),
            ("pages/index.md", PAGE),
            ("pages/blog.md", PAGE),
        ]);

        let page = create_page(dir.path(), "", "Über uns", None).unwrap();
        assert_eq!(page.path, "ueber-uns");
        assert_eq!(page.file, Path::new("pages/ueber-uns.md"));
        assert_eq!(
            fs::read_to_string(dir.path().join("pages/ueber-uns.md")).unwrap(),
            "+++\ntitle = \"Über uns\"\n+++\n"
        );

        let date: PageDate = serde_json::from_str("\"2026-10-21\"").unwrap();
        let post = create_page(dir.path(), "/blog/", "Hello, World!", Some(date)).unwrap();
        assert_eq!(post.path, "blog/2026-10-21-hello-world");
        assert_eq!(post.file, Path::new("pages/blog/2026-10-21-hello-world.md"));

        // The site opens with both new pages in it.
        let paths: Vec<String> = open(dir.path())
            .unwrap()
            .pages
            .into_iter()
            .map(|p| p.path)
            .collect();
        assert_eq!(
            paths,
            ["", "blog", "blog/2026-10-21-hello-world", "ueber-uns"]
        );
    }

    #[test]
    fn refuses_to_create_a_page_that_exists() {
        let dir = site_with(&[
            ("site.toml", "title = \"S\""),
            ("pages/about/index.md", PAGE),
        ]);
        let result = create_page(dir.path(), "", "About", None);
        assert!(matches!(result, Err(SiteError::DuplicatePath { .. })));
        // The existing file is untouched.
        assert_eq!(
            fs::read_to_string(dir.path().join("pages/about/index.md")).unwrap(),
            PAGE
        );
    }

    #[test]
    fn refuses_unknown_parents_and_titles_without_letters() {
        let dir = site_with(&[("site.toml", "title = \"S\"")]);
        assert!(matches!(
            create_page(dir.path(), "nowhere", "Hi", None),
            Err(SiteError::Invalid { .. })
        ));
        assert!(matches!(
            create_page(dir.path(), "", "?!", None),
            Err(SiteError::Invalid { .. })
        ));
    }

    #[test]
    fn refuses_an_empty_title() {
        let dir = site_with(&[("site.toml", "title = \"S\""), ("pages/a.md", PAGE)]);
        let mut front = open(dir.path()).unwrap().pages[0].content.front.clone();
        front.title = "  ".into();
        let result = save_front_matter(dir.path(), Path::new("pages/a.md"), front);
        assert!(matches!(result, Err(SiteError::Invalid { .. })));
    }

    #[test]
    fn saves_site_settings_and_keeps_comments() {
        let dir = site_with(&[(
            "site.toml",
            "# My site\ntitle = \"S\"  # shown everywhere\n",
        )]);
        let mut config = open(dir.path()).unwrap().config;
        config.theme = "cupcake".into();
        config.description = Some("About things.".into());
        save_config(dir.path(), &config).unwrap();

        let text = fs::read_to_string(dir.path().join("site.toml")).unwrap();
        assert!(
            text.starts_with("# My site\ntitle = \"S\"  # shown everywhere\n"),
            "{text}"
        );
        assert!(text.contains("theme = \"cupcake\""), "{text}");
        assert!(!text.contains("language"), "defaults stay out: {text}");

        // A default someone wrote stays when something else changes.
        fs::write(
            dir.path().join("site.toml"),
            "title = \"S\"\nlanguage = \"en\"\n",
        )
        .unwrap();
        let mut config = open(dir.path()).unwrap().config;
        config.theme = "nord".into();
        save_config(dir.path(), &config).unwrap();
        let text = fs::read_to_string(dir.path().join("site.toml")).unwrap();
        assert_eq!(text, "title = \"S\"\nlanguage = \"en\"\ntheme = \"nord\"\n");
        assert_eq!(open(dir.path()).unwrap().config, config);

        config.theme = "neon-pink".into();
        assert!(matches!(
            save_config(dir.path(), &config),
            Err(SiteError::Invalid { .. })
        ));
    }

    #[test]
    fn saves_a_custom_theme_as_a_table() {
        let dir = site_with(&[("site.toml", "title = \"S\"\n")]);
        let mut config = open(dir.path()).unwrap().config;
        config.theme = "custom".into();
        assert!(save_config(dir.path(), &config).is_err(), "no colors yet");

        config.custom_theme = Some(CustomTheme {
            name: None,
            from: "nord".into(),
            background: None,
            primary: Some("#ff6600".into()),
            secondary: None,
            accent: None,
            neutral: None,
            corners: Some(crate::theme::Corners::Round),
        });
        save_config(dir.path(), &config).unwrap();
        let text = fs::read_to_string(dir.path().join("site.toml")).unwrap();
        assert!(text.contains("theme = \"custom\""), "{text}");
        assert!(
            text.contains(
                "[custom_theme]\nfrom = \"nord\"\nprimary = \"#ff6600\"\ncorners = \"round\""
            ),
            "{text}"
        );
        let reopened = open(dir.path()).unwrap().config;
        assert_eq!(reopened, config);
        assert_eq!(reopened.page_theme(), "nord");

        // Changing a color in the existing table, and another theme keeps
        // the colors for later.
        if let Some(custom) = config.custom_theme.as_mut() {
            custom.primary = Some("#00aa00".into());
            custom.corners = None;
        }
        config.theme = "cupcake".into();
        save_config(dir.path(), &config).unwrap();
        let reopened = open(dir.path()).unwrap().config;
        assert_eq!(reopened.page_theme(), "cupcake");
        assert_eq!(reopened.custom_theme, config.custom_theme);
        let text = fs::read_to_string(dir.path().join("site.toml")).unwrap();
        assert_eq!(text.matches("primary").count(), 1, "{text}");
        assert!(!text.contains("corners"), "{text}");
    }

    #[test]
    fn keeps_named_sets_of_colors() {
        let dir = site_with(&[("site.toml", "title = \"S\"\n")]);
        let mut config = open(dir.path()).unwrap().config;
        let set = |name: &str, primary: &str| CustomTheme {
            name: Some(name.into()),
            from: "light".into(),
            background: None,
            primary: Some(primary.into()),
            secondary: None,
            accent: None,
            neutral: None,
            corners: None,
        };
        config.saved_themes = vec![set("Orange", "#fc6e51"), set("Blue", "#3366ff")];
        save_config(dir.path(), &config).unwrap();
        let text = fs::read_to_string(dir.path().join("site.toml")).unwrap();
        assert!(
            text.contains(
                "[[saved_themes]]\nname = \"Orange\"\nfrom = \"light\"\nprimary = \"#fc6e51\""
            ),
            "{text}"
        );
        assert_eq!(open(dir.path()).unwrap().config, config);

        config.saved_themes.remove(0);
        save_config(dir.path(), &config).unwrap();
        let text = fs::read_to_string(dir.path().join("site.toml")).unwrap();
        assert_eq!(text.matches("[[saved_themes]]").count(), 1, "{text}");

        config.saved_themes.push(set("Blue", "#000000"));
        assert!(save_config(dir.path(), &config).is_err(), "same name twice");
        config.saved_themes = vec![set(" ", "#000000")];
        assert!(save_config(dir.path(), &config).is_err(), "no name");
    }

    #[test]
    fn writes_colors_as_their_own_section() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(CONFIG_FILE), "title = \"T\"\n").unwrap();
        let mut config = open(dir.path()).unwrap().config;
        config.colors.footer = Some(Surface::Neutral);
        config.colors.links = Some(ThemeColor::Accent);
        save_config(dir.path(), &config).unwrap();
        let text = fs::read_to_string(dir.path().join(CONFIG_FILE)).unwrap();
        assert!(
            text.contains("[colors]\nlinks = \"accent\"\nfooter = \"neutral\"\n"),
            "{text}"
        );
        assert_eq!(open(dir.path()).unwrap().config.colors, config.colors);

        config.colors = Colors::default();
        save_config(dir.path(), &config).unwrap();
        let text = fs::read_to_string(dir.path().join(CONFIG_FILE)).unwrap();
        assert!(
            !text.contains("colors"),
            "defaults leave no section: {text}"
        );
    }

    #[test]
    fn sets_the_order_with_weights() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(CONFIG_FILE), "title = \"T\"\n").unwrap();
        fs::create_dir(dir.path().join(PAGES_DIR)).unwrap();
        for (name, weight) in [("a", 20), ("b", 0), ("c", 5)] {
            fs::write(
                dir.path().join(PAGES_DIR).join(format!("{name}.md")),
                format!("+++\ntitle = \"{name}\"\nweight = {weight}\n+++\n\nText.\n"),
            )
            .unwrap();
        }
        let site = open(dir.path()).unwrap();
        let files: Vec<PathBuf> = ["c", "a", "b"]
            .iter()
            .map(|name| PathBuf::from(format!("pages/{name}.md")))
            .collect();
        assert_eq!(set_order(&site, &files).unwrap(), 2, "a has its 20 already");
        let site = open(dir.path()).unwrap();
        let weights: Vec<(String, i32)> = site
            .pages
            .iter()
            .map(|page| (page.path.clone(), page.content.front.weight))
            .collect();
        assert_eq!(
            weights,
            [("a".into(), 20), ("b".into(), 30), ("c".into(), 10)]
        );
        assert!(set_order(&site, &[PathBuf::from("pages/x.md")]).is_err());
    }

    #[test]
    fn keeps_the_footer_in_its_own_file() {
        let dir = site_with(&[("site.toml", "title = \"S\""), ("pages/index.md", PAGE)]);
        assert!(open(dir.path()).unwrap().footer.is_none());

        let blocks = vec![Block::Text {
            markdown: "[Impressum](/impressum)".to_string(),
        }];
        let footer = save_footer(dir.path(), blocks.clone()).unwrap().unwrap();
        assert_eq!(footer.path, FOOTER_PATH);
        let site = open(dir.path()).unwrap();
        assert_eq!(site.footer.as_ref().unwrap().content.blocks, blocks);
        assert_eq!(pages_linking_to(&site, "impressum"), [FOOTER_PATH]);

        assert!(save_footer(dir.path(), Vec::new()).unwrap().is_none());
        assert!(!dir.path().join(FOOTER_FILE).exists(), "no blocks, no file");
    }

    #[test]
    fn saves_logo_settings_only_when_not_default() {
        let dir = site_with(&[("site.toml", "title = \"S\"\n")]);
        let mut config = open(dir.path()).unwrap().config;
        assert!(config.title_in_header);
        config.logo_size = LogoSize::Large;
        config.title_in_header = false;
        save_config(dir.path(), &config).unwrap();
        let text = fs::read_to_string(dir.path().join("site.toml")).unwrap();
        assert!(
            text.contains("logo_size = \"large\"\ntitle_in_header = false"),
            "{text}"
        );
        assert_eq!(open(dir.path()).unwrap().config, config);

        config.logo_size = LogoSize::Small;
        config.title_in_header = true;
        save_config(dir.path(), &config).unwrap();
        let text = fs::read_to_string(dir.path().join("site.toml")).unwrap();
        assert_eq!(text, "title = \"S\"\n", "defaults stay out");
    }

    #[test]
    fn keeps_a_hand_made_menu_in_sections() {
        let dir = site_with(&[("site.toml", "title = \"S\"\n")]);
        let mut config = open(dir.path()).unwrap().config;
        let page = |path: &str| MenuItem {
            page: Some(path.into()),
            ..MenuItem::default()
        };
        config.main_menu = vec![
            MenuItem {
                title: Some("Stacks".into()),
                items: vec![
                    page("elixir"),
                    MenuItem {
                        divider: true,
                        ..MenuItem::default()
                    },
                    MenuItem {
                        title: Some("Layer 8".into()),
                        link: Some("https://layer8.example".into()),
                        ..MenuItem::default()
                    },
                ],
                ..MenuItem::default()
            },
            page("kontakt"),
        ];
        save_config(dir.path(), &config).unwrap();
        let text = fs::read_to_string(dir.path().join("site.toml")).unwrap();
        assert!(
            text.contains("[[main_menu]]\ntitle = \"Stacks\"\n\n[[main_menu.items]]\npage = \"elixir\"\n\n[[main_menu.items]]\ndivider = true\n"),
            "{text}"
        );
        assert!(text.contains("[[main_menu]]\npage = \"kontakt\""), "{text}");
        assert_eq!(open(dir.path()).unwrap().config, config);

        for bad in [
            vec![MenuItem {
                divider: true,
                ..MenuItem::default()
            }],
            vec![MenuItem {
                title: Some("Empty".into()),
                ..MenuItem::default()
            }],
            vec![MenuItem {
                link: Some("https://x".into()),
                ..MenuItem::default()
            }],
        ] {
            config.main_menu = bad;
            assert!(
                save_config(dir.path(), &config).is_err(),
                "{:?}",
                config.main_menu
            );
        }
    }

    #[test]
    fn reads_menu_pages_written_by_hand_like_page_files() {
        for (written, path) in [
            ("", ""),
            ("/", ""),
            ("index", ""),
            ("/index.md", ""),
            ("blog", "blog"),
            ("/blog/", "blog"),
            ("blog.md", "blog"),
            ("blog/index", "blog"),
            (" about/team ", "about/team"),
        ] {
            assert_eq!(menu_page_path(written), path, "{written:?}");
        }
        let dir = site_with(&[(
            "site.toml",
            "title = \"S\"\n\n[[main_menu]]\npage = \"index\"\n\n[[main_menu]]\ntitle = \"More\"\n\n[[main_menu.items]]\npage = \"/blog/\"\n",
        )]);
        let menu = open(dir.path()).unwrap().config.main_menu;
        assert_eq!(menu[0].page.as_deref(), Some(""));
        assert_eq!(menu[1].items[0].page.as_deref(), Some("blog"));
    }

    #[test]
    fn tidies_old_addresses_and_refuses_ones_taken() {
        for (written, address) in [
            ("about", Some("/about")),
            (
                " /2016/03/28/easterhegg.html ",
                Some("/2016/03/28/easterhegg.html"),
            ),
            (
                "https://old.example.org/blog/post/?p=1#top",
                Some("/blog/post"),
            ),
            ("https://old.example.org", None),
            ("/", None),
            ("", None),
        ] {
            assert_eq!(old_address(written).as_deref(), address, "{written:?}");
        }

        let dir = site_with(&[
            ("site.toml", "title = \"S\""),
            ("pages/index.md", PAGE),
            ("pages/about.md", PAGE),
            (
                "pages/contact.md",
                "+++\ntitle = \"Contact\"\nredirect_from = [\"/kontakt\"]\n+++\n",
            ),
        ]);
        let file = Path::new("pages/about.md");
        let front = |addresses: &[&str]| FrontMatter {
            title: "About".into(),
            redirect_from: addresses.iter().map(|a| a.to_string()).collect(),
            ..FrontMatter::default()
        };
        let saved = save_front_matter(
            dir.path(),
            file,
            front(&[
                "ueber-uns",
                "/ueber-uns/",
                "https://old.example.org/team.html",
                " ",
            ]),
        )
        .unwrap();
        assert_eq!(
            saved.content.front.redirect_from,
            ["/ueber-uns", "/team.html"]
        );

        for (taken, message) in [
            ("/contact", "is the address of the page “Contact”"),
            ("kontakt", "already leads to the page “Contact”"),
            ("/about", "is this page's own address"),
        ] {
            let error = save_front_matter(dir.path(), file, front(&["/ueber-uns", taken]))
                .unwrap_err()
                .to_string();
            assert!(error.contains(message), "{error}");
        }
    }

    #[test]
    fn maps_files_to_url_paths() {
        let dir = site_with(&[
            ("site.toml", "title = \"S\""),
            ("pages/index.md", PAGE),
            ("pages/blog/index.md", PAGE),
            ("pages/blog/first-post.md", PAGE),
            ("pages/notes.txt", "ignored"),
        ]);
        let site = open(dir.path()).unwrap();
        let paths: Vec<&str> = site.pages.iter().map(|p| p.path.as_str()).collect();
        assert_eq!(paths, ["", "blog", "blog/first-post"]);
        assert_eq!(site.pages[2].file, Path::new("pages/blog/first-post.md"));
    }

    #[test]
    fn uses_defaults_for_missing_settings() {
        let dir = site_with(&[("site.toml", "title = \"S\"")]);
        let site = open(dir.path()).unwrap();
        assert_eq!(site.config.theme, "light");
        assert!(site.pages.is_empty());
    }

    #[test]
    fn deletes_pages_and_tells_who_links_to_them() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("pages/about")).unwrap();
        fs::write(root.join("site.toml"), "title = \"S\"\n").unwrap();
        fs::write(
            root.join("pages/index.md"),
            "+++\ntitle = \"Home\"\n+++\n\nSee [us](about/index.md).\n",
        )
        .unwrap();
        fs::write(
            root.join("pages/about/index.md"),
            "+++\ntitle = \"About\"\n+++\n",
        )
        .unwrap();
        fs::write(
            root.join("pages/about/team.md"),
            "+++\ntitle = \"Team\"\n+++\n",
        )
        .unwrap();
        fs::write(
            root.join("pages/contact.md"),
            "+++\ntitle = \"Contact\"\n+++\n\n::: call-to-action\nbutton_label = \"Team\"\nbutton_link = \"/about/team/\"\n:::\n",
        )
        .unwrap();
        let site = open(root).unwrap();

        assert_eq!(pages_linking_to(&site, "about"), [""]);
        assert_eq!(pages_linking_to(&site, "about/team"), ["contact"]);
        assert!(pages_linking_to(&site, "contact").is_empty());

        let error = delete_page(&site, Path::new("pages/index.md")).unwrap_err();
        assert!(error.to_string().contains("start page"), "{error}");
        let error = delete_page(&site, Path::new("pages/about/index.md")).unwrap_err();
        assert!(error.to_string().contains("/about/team"), "{error}");
        assert!(delete_page(&site, Path::new("../site.toml")).is_err());

        delete_page(&site, Path::new("pages/about/team.md")).unwrap();
        let site = open(root).unwrap();
        delete_page(&site, Path::new("pages/about/index.md")).unwrap();
        assert!(!root.join("pages/about").exists(), "empty folder removed");
        assert_eq!(open(root).unwrap().pages.len(), 2);
    }

    #[test]
    fn rejects_a_folder_without_site_toml() {
        let dir = site_with(&[("pages/index.md", PAGE)]);
        assert!(matches!(open(dir.path()), Err(SiteError::NotASite(_))));
    }

    #[test]
    fn reports_bad_settings_with_the_file_name() {
        let dir = site_with(&[("site.toml", "titel = \"typo\"")]);
        let error = open(dir.path()).unwrap_err();
        assert!(matches!(error, SiteError::Config { .. }));
        assert!(
            error.to_string().starts_with("site.toml") || error.to_string().contains("site.toml")
        );
    }

    #[test]
    fn reports_page_errors_with_file_and_line() {
        let dir = site_with(&[
            ("site.toml", "title = \"S\""),
            (
                "pages/about.md",
                "+++\ntitle = \"A\"\n+++\n::: herro\n:::\n",
            ),
        ]);
        let message = open(dir.path()).unwrap_err().to_string();
        assert!(message.starts_with("pages/about.md, line 4:"), "{message}");
    }

    #[test]
    fn suggests_a_name_for_files_that_aren_t_slugs() {
        let dir = site_with(&[("site.toml", "title = \"S\""), ("pages/Über uns.md", PAGE)]);
        let error = open(dir.path()).unwrap_err();
        assert!(
            matches!(&error, SiteError::BadFileName { suggestion, .. } if suggestion == "ueber-uns"),
            "{error}"
        );
    }

    #[test]
    fn rejects_two_files_with_the_same_url() {
        let dir = site_with(&[
            ("site.toml", "title = \"S\""),
            ("pages/about.md", PAGE),
            ("pages/about/index.md", PAGE),
        ]);
        let error = open(dir.path()).unwrap_err();
        assert!(
            matches!(error, SiteError::DuplicatePath { ref url, .. } if url == "about"),
            "{error}"
        );
    }
}

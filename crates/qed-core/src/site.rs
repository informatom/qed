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
use crate::page::{self, FrontMatter, PageContent, PageDate};
use crate::page_writer;
use crate::slug::slugify;
use crate::target::Target;

pub const CONFIG_FILE: &str = "site.toml";
pub const PAGES_DIR: &str = "pages";
pub const MEDIA_DIR: &str = "media";
/// Files to download, such as PDFs. Images go into `media/`.
pub const ASSETS_DIR: &str = "assets";

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
    /// A daisyUI theme name.
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Where the site is published, e.g. `"https://example.com/"`. Needed
    /// for the sitemap and canonical links; the site works without it.
    pub base_url: Option<String>,
    /// Where the main menu goes: the top bar, or a sidebar for sites whose
    /// main menu is too long for the top bar.
    #[serde(default)]
    pub navigation: Navigation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum Navigation {
    #[default]
    Top,
    Sidebar,
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
    let config = toml::from_str(&read(&config_path)?).map_err(|source| SiteError::Config {
        path: config_path,
        source,
    })?;

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

/// Writes a file that must not exist yet, creating its folder if needed.
fn write_new(path: &Path, contents: &str) -> Result<(), SiteError> {
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
/// out, as in page files.
pub fn save_config(root: &Path, config: &SiteConfig) -> Result<(), SiteError> {
    let path = root.join(CONFIG_FILE);
    let invalid = |message: String| SiteError::Invalid {
        path: PathBuf::from(CONFIG_FILE),
        message,
    };
    if config.title.trim().is_empty() {
        return Err(invalid("a site needs a title".to_string()));
    }
    if !crate::theme::is_theme(&config.theme) {
        return Err(invalid(format!(
            "there is no theme called \"{}\"",
            config.theme
        )));
    }

    let original = read(&path)?;
    let mut document = toml_edit::ser::to_document(config).map_err(|e| invalid(e.to_string()))?;
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
    ] {
        if *value == default && !page_writer::writes_string(&original, key, &default) {
            document.remove(key);
        }
    }
    write_atomically(&path, &page_writer::merge(&original, document))
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
    update_page(root, file, |content| content.front = front)
}

/// Saves a page's blocks and returns the page as saved.
///
/// Empty text blocks are dropped, and text blocks that follow each other are
/// joined, because that's how the file will read back. The returned page is
/// what the file now contains.
pub fn save_blocks(root: &Path, file: &Path, blocks: Vec<Block>) -> Result<Page, SiteError> {
    let blocks = blocks
        .into_iter()
        .filter(|block| !matches!(block, Block::Text { markdown } if markdown.trim().is_empty()))
        .collect();
    update_page(root, file, |content| content.blocks = blocks)
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

fn read(path: &Path) -> Result<String, SiteError> {
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

    /// The project's own docs folder is a qed site. This keeps it valid.
    #[test]
    fn opens_the_docs_site() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
        let site = open(&root).unwrap_or_else(|error| panic!("{error}"));

        assert_eq!(site.config.title, "qed");
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

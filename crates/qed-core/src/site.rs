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
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::page::{self, PageContent};
use crate::slug::slugify;

pub const CONFIG_FILE: &str = "site.toml";
pub const PAGES_DIR: &str = "pages";
pub const MEDIA_DIR: &str = "media";

/// Settings from `site.toml`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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
}

fn default_language() -> String {
    "en".to_string()
}

fn default_theme() -> String {
    "light".to_string()
}

/// A site folder, read into memory.
#[derive(Debug, Clone, Serialize)]
pub struct Site {
    pub root: PathBuf,
    pub config: SiteConfig,
    /// Sorted by URL path.
    pub pages: Vec<Page>,
}

/// One page of a site.
#[derive(Debug, Clone, Serialize)]
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
    })
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

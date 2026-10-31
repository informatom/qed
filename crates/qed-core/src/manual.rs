//! The user manual (#110), inside the app as well as on the docs site.
//!
//! The manual's pages are q.e.d. pages in `docs/pages/manual/`, with their
//! pictures in `docs/media/manual/`; the docs site shows them under
//! `/manual`. Both folders are compiled into the program, so the app has
//! the manual of its own version, offline. [`build`] makes a small site of
//! them (the manual's start page as its start page, the others beside it)
//! and builds it with the site generator, so blocks, icons and pictures
//! look exactly as on the docs site. The app shows the result in a window
//! of its own, through the preview server.
//!
//! So that links work in both places, manual pages link to each other by
//! file name (`backups.md`), and to anything else with a full web address.
//!
//! The manual is in English, with translations beside the pages
//! (`backups.de.md`, #145). The app builds it in one language at a time:
//! a page's translation takes the page's place, and pages without one stay
//! English, so the links by file name work in every language.

use std::fs;
use std::path::{Path, PathBuf};

use include_dir::{Dir, include_dir};

use crate::generator::{self, BuildError, BuildReport};
use crate::site::{self, SiteError};

static PAGES: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../docs/pages/manual");
static MEDIA: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../docs/media/manual");

#[derive(Debug, thiserror::Error)]
pub enum ManualError {
    #[error("can't write the manual into {}: {source}", path.display())]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error(transparent)]
    Site(#[from] SiteError),

    #[error(transparent)]
    Build(#[from] BuildError),
}

/// The languages the manual is in: English, and those it has translations
/// into (`de` for `index.de.md`).
pub fn languages() -> Vec<String> {
    let mut codes: Vec<String> = PAGES
        .files()
        .filter_map(|file| translation_of(file.path()).map(|(_, code)| code.to_string()))
        .collect();
    codes.push("en".to_string());
    codes.sort();
    codes.dedup();
    codes
}

/// `("backups.md", "de")` for `backups.de.md`; `None` for a page in English.
fn translation_of(path: &Path) -> Option<(String, &str)> {
    let name = path.file_name()?.to_str()?.strip_suffix(".md")?;
    let (stem, code) = name.split_once('.')?;
    site::is_language_code(code).then(|| (format!("{stem}.md"), code))
}

/// Writes the manual as a site into `folder` (emptied first) and builds
/// it, in `theme`, the app's own, and in `language` (English where a page
/// has no translation into it). Returns where the built manual is.
pub fn build(folder: &Path, theme: &str, language: &str) -> Result<BuildReport, ManualError> {
    let write = |path: &Path, contents: &[u8]| -> Result<(), ManualError> {
        let error = |source| ManualError::Write {
            path: path.to_path_buf(),
            source,
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(error)?;
        }
        fs::write(path, contents).map_err(error)
    };
    if folder.exists() {
        fs::remove_dir_all(folder).map_err(|source| ManualError::Write {
            path: folder.to_path_buf(),
            source,
        })?;
    }
    let theme = if crate::theme::is_theme(theme) {
        theme
    } else {
        "light"
    };
    let language = if languages().iter().any(|code| code == language) {
        language
    } else {
        "en"
    };
    let (title, description) = match language {
        "de" => (
            "q.e.d.-Handbuch",
            "Wie Sie Ihre Website mit q.e.d. machen und betreiben.",
        ),
        _ => (
            "q.e.d. manual",
            "How to make and run your website with q.e.d.",
        ),
    };
    let config = format!(
        "title = \"{title}\"\nlanguage = \"{language}\"\ntheme = \"{theme}\"\nnavigation = \"sidebar\"\n\
         description = \"{description}\"\n"
    );
    write(&folder.join(site::CONFIG_FILE), config.as_bytes())?;
    for file in PAGES.files() {
        let Some(name) = file.path().file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        // A translation goes in place of its page, in its own language only.
        let name = match translation_of(file.path()) {
            Some((page, code)) if code == language => page,
            Some(_) => continue,
            None => {
                let translated = name.replace(".md", &format!(".{language}.md"));
                if language != "en" && PAGES.get_file(&translated).is_some() {
                    continue;
                }
                name.to_string()
            }
        };
        // Old addresses only make sense on the docs site.
        let text = without_redirects(file.contents_utf8().unwrap_or_default());
        write(&folder.join(site::PAGES_DIR).join(name), text.as_bytes())?;
    }
    for file in MEDIA.files() {
        let target = folder.join("media").join("manual").join(file.path());
        write(&target, file.contents())?;
    }
    let site = site::open(folder)?;
    Ok(generator::build(&site)?)
}

/// A page's text without its `redirect_from` line.
fn without_redirects(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("redirect_from"))
        .map(|line| format!("{line}\n"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_the_manual_with_working_links() {
        let dir = tempfile::tempdir().unwrap();
        let report = build(&dir.path().join("manual"), "catppuccin-mocha", "en").unwrap();
        // Links between manual pages, and pictures, all resolve: the
        // build notes nothing missing.
        assert!(
            !report.notes.iter().any(|note| note.contains(" links to ")),
            "{:?}",
            report.notes
        );
        let start = fs::read_to_string(report.out_dir.join("index.html")).unwrap();
        assert!(start.contains(r#"data-theme="catppuccin-mocha""#));
        assert!(report.out_dir.join("backups/index.html").is_file());
    }

    #[test]
    fn builds_the_manual_in_german_with_english_for_the_rest() {
        assert_eq!(languages(), ["de", "en"]);
        let dir = tempfile::tempdir().unwrap();
        let report = build(&dir.path().join("manual"), "light", "de").unwrap();
        assert!(
            !report.notes.iter().any(|note| note.contains(" links to ")),
            "{:?}",
            report.notes
        );
        let start = fs::read_to_string(report.out_dir.join("index.html")).unwrap();
        assert!(start.contains(r#"<html lang="de""#) && start.contains("Handbuch"));
        // Not translated: the libraries page stays English.
        let libraries = fs::read_to_string(report.out_dir.join("libraries/index.html")).unwrap();
        assert!(libraries.contains("Libraries"));
    }

    #[test]
    fn the_libraries_page_matches_cargo_lock_and_package_json() {
        let page = PAGES
            .get_file("libraries.md")
            .and_then(|file| file.contents_utf8())
            .unwrap();
        let again = "run `cargo run -p qed-core --example libraries`";
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

        let lock: toml::Table = fs::read_to_string(root.join("Cargo.lock"))
            .unwrap()
            .parse()
            .unwrap();
        let mut libraries = 0;
        for package in lock["package"].as_array().unwrap() {
            // Libraries from crates.io have a source; q.e.d.'s own don't.
            if package.get("source").is_none() {
                continue;
            }
            libraries += 1;
            let name = package["name"].as_str().unwrap();
            let version = package["version"].as_str().unwrap();
            let row = format!("| [{name}](https://crates.io/crates/{name}) | {version} |");
            assert!(page.contains(&row), "{name} {version} is missing: {again}");
        }
        let rows = page.matches("](https://crates.io/crates/").count();
        let direct = page
            .split("## Used directly")
            .nth(1)
            .and_then(|rest| rest.split("## In the app").next())
            .unwrap()
            .matches("](https://crates.io/crates/")
            .count();
        assert_eq!(
            rows - direct,
            libraries,
            "libraries.md lists others: {again}"
        );

        let manifest: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join("package.json")).unwrap()).unwrap();
        for key in ["dependencies", "devDependencies"] {
            for name in manifest[key].as_object().unwrap().keys() {
                let link = format!("[{name}](https://www.npmjs.com/package/{name})");
                assert!(page.contains(&link), "{name} is missing: {again}");
            }
        }
    }

    #[test]
    fn the_icons_page_lists_every_icon() {
        let page = PAGES
            .get_file("icons.md")
            .and_then(|file| file.contents_utf8())
            .unwrap();
        for (name, _) in crate::icons::ICONS {
            assert!(
                page.contains(&format!("`:icon-{name}:`")),
                "icons.md doesn't list {name}"
            );
        }
    }
}

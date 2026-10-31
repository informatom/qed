//! Changes to a site that both shells make, the app and `qed-mcp` (#105),
//! with the same rules: a page's text is checked before it's written, and
//! the open version is closed before a page is deleted or moved, so the
//! history keeps the page as it was and it can come back.
//!
//! Recording the change itself (`history::record_save`) is up to the
//! caller, after any change, like every other save.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Component, Path};

use crate::history::{self, HistoryError};
use crate::moving;
use crate::page;
use crate::shared::SHARED_DIR;
use crate::site::{self, CONFIG_FILE, Page, SiteConfig, SiteError};

#[derive(Debug, thiserror::Error)]
pub enum EditError {
    #[error(transparent)]
    Site(#[from] SiteError),

    #[error(transparent)]
    History(#[from] HistoryError),

    #[error(
        "{file} changed since it was read; read it again and make the change on what it says now"
    )]
    Changed { file: String },
}

/// A page file's text, as written, with a stamp that changes whenever the
/// text does: `write_page_text` takes it back to make sure nobody changed
/// the file in between. Also for the files of blocks that aren't pages
/// (see [`is_part`]); one that doesn't exist yet reads as empty.
pub fn page_text(root: &Path, file: &Path) -> Result<(String, String), EditError> {
    if is_part(file) {
        let full = root.join(file);
        let text = if full.is_file() {
            site::read(&full)?
        } else {
            String::new()
        };
        let stamp = stamp(&text);
        return Ok((text, stamp));
    }
    let page = site::read_page(root, file)?;
    let text = site::read(&root.join(&page.file))?;
    let stamp = stamp(&text);
    Ok((text, stamp))
}

/// Replaces a page file's whole text, if it's a valid page (settings
/// between `+++` lines, blocks as `:::` containers) with a title, and if
/// the file still has the text `stamp` was made from. Returns the page as
/// written.
pub fn write_page_text(
    root: &Path,
    file: &Path,
    text: &str,
    stamp: &str,
) -> Result<Page, EditError> {
    let (now, now_stamp) = page_text(root, file)?;
    if now_stamp != stamp {
        return Err(EditError::Changed {
            file: file.display().to_string(),
        });
    }
    let invalid = |message: String| SiteError::Invalid {
        path: file.to_path_buf(),
        message,
    };
    let content = page::parse(text)
        .map_err(|error| invalid(format!("line {}: {}", error.line, error.message)))?;
    if content.front.title.trim().is_empty() {
        return Err(invalid("a page needs a title".to_string()).into());
    }
    if now != text {
        site::write_atomically(&root.join(file), text)?;
    }
    Ok(site::read_page(root, file)?)
}

/// Whether `file` is one of the files of blocks that aren't pages: the
/// footer (`footer.md`, and `footer.en.md` for a language), the episode
/// layout (`episode.md`) or a shared block (`shared/teaser.md`).
pub fn is_part(file: &Path) -> bool {
    let plain = file.components().all(|c| matches!(c, Component::Normal(_)));
    let markdown = file.extension().is_some_and(|ext| ext == "md");
    let stem = file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    let folder = file.parent().unwrap_or(Path::new(""));
    plain
        && markdown
        && !stem.is_empty()
        && if folder == Path::new("") {
            stem == "footer" || stem.starts_with("footer.") || stem == "episode"
        } else {
            folder == Path::new(SHARED_DIR)
        }
}

/// Replaces the text of a file of blocks that isn't a page (see
/// [`is_part`]), like [`write_page_text`]: only if it's valid and still
/// what `stamp` was made from. An empty text removes the file: no footer,
/// the default episode layout, no such shared block.
pub fn write_part_text(root: &Path, file: &Path, text: &str, stamp: &str) -> Result<(), EditError> {
    if !is_part(file) {
        return Err(SiteError::NotAPage(file.to_path_buf()).into());
    }
    let (now, now_stamp) = page_text(root, file)?;
    if now_stamp != stamp {
        return Err(EditError::Changed {
            file: file.display().to_string(),
        });
    }
    let name = file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    if file.starts_with(SHARED_DIR) && crate::slug::slugify(name) != name {
        return Err(SiteError::Invalid {
            path: file.to_path_buf(),
            message: format!(
                "a shared block's name is lowercase letters, digits and dashes: {}",
                crate::slug::slugify(name)
            ),
        }
        .into());
    }
    if !text.trim().is_empty() {
        page::parse(text).map_err(|error| SiteError::Invalid {
            path: file.to_path_buf(),
            message: format!("line {}: {}", error.line, error.message),
        })?;
    }
    replace_checked(root, file, &now, text)
}

/// The text of `site.toml`, the site's settings, with a stamp like
/// [`page_text`]'s.
pub fn settings_text(root: &Path) -> Result<(String, String), EditError> {
    let text = site::read(&root.join(CONFIG_FILE))?;
    let stamp = stamp(&text);
    Ok((text, stamp))
}

/// Replaces `site.toml`, if it's valid and still what `stamp` was made
/// from. The site must open with it, so a menu entry for a page that
/// doesn't exist or a language without its pages is refused too.
pub fn write_settings_text(root: &Path, text: &str, stamp: &str) -> Result<(), EditError> {
    let (now, now_stamp) = settings_text(root)?;
    if now_stamp != stamp {
        return Err(EditError::Changed {
            file: CONFIG_FILE.to_string(),
        });
    }
    toml::from_str::<SiteConfig>(text).map_err(|source| SiteError::Config {
        path: root.join(CONFIG_FILE),
        source,
    })?;
    replace_checked(root, Path::new(CONFIG_FILE), &now, text)
}

/// Writes `text` over `before` in `file` (an empty text removes it), then
/// opens the site: if it doesn't open any more, the file gets `before`
/// back and the reason is returned.
fn replace_checked(root: &Path, file: &Path, before: &str, text: &str) -> Result<(), EditError> {
    if before == text {
        return Ok(());
    }
    let full = root.join(file);
    let write = |text: &str| -> Result<(), SiteError> {
        if text.trim().is_empty() {
            if full.exists() {
                std::fs::remove_file(&full).map_err(|source| SiteError::Write {
                    path: full.clone(),
                    source,
                })?;
            }
            return Ok(());
        }
        if let Some(folder) = full.parent() {
            std::fs::create_dir_all(folder).map_err(|source| SiteError::Write {
                path: folder.to_path_buf(),
                source,
            })?;
        }
        site::write_atomically(&full, text)
    };
    write(text)?;
    if let Err(error) = site::open(root) {
        write(before)?;
        return Err(error.into());
    }
    Ok(())
}

/// Deletes a page, after closing the open version, so it keeps the page.
/// Otherwise the deletion would go into the same open version, and a page
/// made and deleted in it would leave no trace to bring back.
pub fn delete_page(root: &Path, file: &Path) -> Result<(), EditError> {
    let site = site::open(root)?;
    close_before(&site, root, file, "Before deleting")?;
    site::delete_page(&site, file)?;
    Ok(())
}

/// Gives a page a new address (see `moving::move_page`), after closing the
/// open version, so the site from before the move stays in the history.
pub fn move_page(root: &Path, file: &Path, parent: &str, slug: &str) -> Result<Page, EditError> {
    let site = site::open(root)?;
    close_before(&site, root, file, "Before moving")?;
    Ok(moving::move_page(&site, file, parent, slug)?)
}

fn close_before(site: &site::Site, root: &Path, file: &Path, what: &str) -> Result<(), EditError> {
    if !history::status(root)?.tracked {
        return Ok(());
    }
    if let Some(page) = site.pages.iter().find(|page| page.file == file) {
        history::save_version(root, &format!("{what} /{}", page.path))?;
    }
    Ok(())
}

/// A short fingerprint of a text. Not for security, only to notice a change.
fn stamp(text: &str) -> String {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn site() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("site.toml"), "title = \"S\"\n").unwrap();
        fs::create_dir_all(dir.path().join("pages")).unwrap();
        fs::write(
            dir.path().join("pages/index.md"),
            "+++\ntitle = \"Home\"\n+++\n\nHi.\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn writes_valid_text_only_over_what_was_read() {
        let dir = site();
        let (root, file) = (dir.path(), Path::new("pages/index.md"));
        let (text, stamp) = page_text(root, file).unwrap();
        assert!(text.contains("Hi."));

        let error = write_page_text(root, file, "no settings", &stamp).unwrap_err();
        assert!(error.to_string().contains("line 1"), "{error}");
        let error = write_page_text(root, file, "+++\ntitle = \"\"\n+++\n", &stamp).unwrap_err();
        assert!(error.to_string().contains("needs a title"), "{error}");

        let new = "+++\ntitle = \"Home\"\n+++\n\n::: hero\nheadline = \"Welcome\"\n:::\n";
        let page = write_page_text(root, file, new, &stamp).unwrap();
        assert_eq!(page.content.blocks.len(), 1);

        // The stamp from before that write no longer fits.
        let error = write_page_text(root, file, new, &stamp).unwrap_err();
        assert!(matches!(error, EditError::Changed { .. }), "{error}");
    }

    #[test]
    fn closes_the_open_version_before_deleting() {
        let dir = site();
        let root = dir.path();
        fs::write(root.join("pages/old.md"), "+++\ntitle = \"Old\"\n+++\n").unwrap();
        history::start(root).unwrap();
        fs::write(root.join("pages/old.md"), "+++\ntitle = \"Old one\"\n+++\n").unwrap();
        history::record_save(root).unwrap();

        delete_page(root, Path::new("pages/old.md")).unwrap();
        history::record_save(root).unwrap();
        let versions = history::versions(root, None).unwrap();
        assert_eq!(versions[1].summary, "Before deleting /old");
        assert!(!root.join("pages/old.md").exists());
    }

    #[test]
    fn writes_the_footer_and_shared_blocks() {
        let dir = site();
        let root = dir.path();
        assert!(is_part(Path::new("footer.md")) && is_part(Path::new("footer.en.md")));
        assert!(is_part(Path::new("shared/teaser.md")) && is_part(Path::new("episode.md")));
        assert!(!is_part(Path::new("pages/index.md")) && !is_part(Path::new("../footer.md")));
        assert!(!is_part(Path::new("site.toml")) && !is_part(Path::new("shared/a/b.md")));

        // A footer that isn't there yet reads as empty, and writing makes it.
        let footer = Path::new("footer.md");
        let (text, stamp) = page_text(root, footer).unwrap();
        assert_eq!(text, "");
        let new = "+++\ntitle = \"Footer\"\n+++\n\n© 2026\n";
        write_part_text(root, footer, new, &stamp).unwrap();
        assert!(site::open(root).unwrap().footer.is_some());
        // Empty text removes it again.
        let (_, stamp) = page_text(root, footer).unwrap();
        write_part_text(root, footer, "", &stamp).unwrap();
        assert!(!root.join("footer.md").exists());

        let shared = Path::new("shared/teaser.md");
        let (_, stamp) = page_text(root, shared).unwrap();
        write_part_text(root, shared, "+++\ntitle = \"Teaser\"\n+++\n\nHi\n", &stamp).unwrap();
        assert_eq!(site::open(root).unwrap().shared.len(), 1);
        let error =
            write_part_text(root, Path::new("shared/My Teaser.md"), "x", &stamp).unwrap_err();
        assert!(error.to_string().contains("my-teaser"), "{error}");
        let error = write_part_text(root, Path::new("pages/index.md"), "x", &stamp).unwrap_err();
        assert!(
            matches!(error, EditError::Site(SiteError::NotAPage(_))),
            "{error}"
        );
    }

    #[test]
    fn writes_settings_only_when_the_site_still_opens() {
        let dir = site();
        let root = dir.path();
        fs::write(
            root.join("site.toml"),
            "title = \"S\"\n[languages.en]\ntitle = \"S\"\n",
        )
        .unwrap();
        fs::write(
            root.join("pages/index.en.md"),
            "+++\ntitle = \"Home\"\n+++\n",
        )
        .unwrap();
        let (text, stamp) = settings_text(root).unwrap();

        let error = write_settings_text(root, "title = ", &stamp).unwrap_err();
        assert!(
            matches!(error, EditError::Site(SiteError::Config { .. })),
            "{error}"
        );
        // Valid TOML, but the site doesn't open without the language its
        // translation is in: the old text stays.
        let error = write_settings_text(root, "title = \"S\"\n", &stamp).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("isn't one of the site's languages"),
            "{error}"
        );
        assert_eq!(settings_text(root).unwrap().0, text);

        let new = text.replacen("\n", "\nbase_url = \"https://example.com/\"\n", 1);
        write_settings_text(root, &new, &stamp).unwrap();
        assert_eq!(
            site::open(root).unwrap().config.base_url.as_deref(),
            Some("https://example.com/")
        );
        let error = write_settings_text(root, &new, &stamp).unwrap_err();
        assert!(matches!(error, EditError::Changed { .. }), "{error}");
    }
}

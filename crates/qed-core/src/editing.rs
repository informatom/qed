//! Changes to a site that both shells make, the app and `qed-mcp` (#105),
//! with the same rules: a page's text is checked before it's written, and
//! the open version is closed before a page is deleted or moved, so the
//! history keeps the page as it was and it can come back.
//!
//! Recording the change itself (`history::record_save`) is up to the
//! caller, after any change, like every other save.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;

use crate::history::{self, HistoryError};
use crate::moving;
use crate::page;
use crate::site::{self, Page, SiteError};

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
/// the file in between.
pub fn page_text(root: &Path, file: &Path) -> Result<(String, String), EditError> {
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
}

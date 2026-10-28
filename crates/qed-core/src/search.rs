//! Site search with [Pagefind](https://pagefind.app): after a build, the
//! finished pages are indexed into `pagefind/` in the built site. The
//! index is split into small chunks; a visitor's browser loads only what a
//! search needs, from the site itself.
//!
//! Only the pages' main content is indexed (`data-pagefind-body` on
//! `<main>`, see `page.html`), so redirects, the 404 page and HTML files
//! from `static/` stay out by themselves. Blocks that list other pages are
//! left out too (`data-pagefind-ignore`), so results lead to the posts and
//! not to the lists of them.
//!
//! The search dialog is Pagefind's Component UI, which the pagefind crate
//! writes along with the index; see `assets/pagefind/README.md`.

use std::fs;
use std::path::Path;

use pagefind::api::PagefindIndex;

/// The folder in the built site with the index and Pagefind's files.
pub const SEARCH_DIR: &str = "pagefind";

/// Files Pagefind writes for its other interfaces, which q.e.d. doesn't use.
const UNUSED: [&str; 5] = [
    "pagefind-ui.js",
    "pagefind-ui.css",
    "pagefind-modular-ui.js",
    "pagefind-modular-ui.css",
    "pagefind-highlight.js",
];

#[derive(Debug, thiserror::Error)]
#[error("can't build the search index: {0}")]
pub struct SearchError(String);

impl SearchError {
    fn from(error: impl std::fmt::Display) -> Self {
        SearchError(error.to_string())
    }
}

/// Indexes the built site in `out_dir` and writes the index to
/// `out_dir/pagefind/`. Returns how many pages are searchable.
pub fn index(out_dir: &Path) -> Result<usize, SearchError> {
    let dir = out_dir.to_string_lossy().to_string();
    let bundle = out_dir.join(SEARCH_DIR);
    let bundle_path = bundle.to_string_lossy().to_string();
    // Pagefind is async. It gets a thread and a runtime of its own, so this
    // works whether or not the caller is on a runtime already (the app's
    // commands are, and a runtime can't be started inside another).
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(SearchError::from)?;
        runtime.block_on(async {
            let mut index = PagefindIndex::new(None).map_err(SearchError::from)?;
            index
                .add_directory(dir, None)
                .await
                .map_err(SearchError::from)?;
            index
                .write_files(Some(bundle_path))
                .await
                .map_err(SearchError::from)?;
            Ok::<_, SearchError>(())
        })
    })
    .join()
    .map_err(|_| SearchError("the indexer stopped unexpectedly".to_string()))??;

    for name in UNUSED {
        let file = bundle.join(name);
        if file.is_file() {
            fs::remove_file(&file).map_err(SearchError::from)?;
        }
    }
    // One fragment per searchable page.
    let fragments = fs::read_dir(bundle.join("fragment")).map_err(SearchError::from)?;
    Ok(fragments.count())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexes_only_marked_pages() {
        let dir = tempfile::tempdir().expect("temp dir");
        let page = |body: &str| {
            format!(
                "<!doctype html><html lang=\"en\"><head><title>T</title></head><body>{body}</body></html>"
            )
        };
        fs::create_dir_all(dir.path().join("a")).expect("folder");
        fs::write(
            dir.path().join("a/index.html"),
            page("<nav>Menu</nav><main data-pagefind-body><h1>Apples</h1><p>Crisp apples.</p></main>"),
        )
        .expect("page");
        fs::write(
            dir.path().join("index.html"),
            page("<main data-pagefind-body><h1>Home</h1><div data-pagefind-ignore>A list</div></main>"),
        )
        .expect("page");
        // A redirect or a static file: no marked content, so not searchable.
        fs::write(dir.path().join("old.html"), page("<p>Moved</p>")).expect("page");

        assert_eq!(index(dir.path()).expect("index"), 2);
        let bundle = dir.path().join(SEARCH_DIR);
        assert!(bundle.join("pagefind.js").is_file());
        assert!(bundle.join("pagefind-component-ui.js").is_file());
        assert!(bundle.join("pagefind-entry.json").is_file());
        assert!(!bundle.join("pagefind-ui.js").exists());
    }
}

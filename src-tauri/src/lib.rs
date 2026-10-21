//! The desktop shell: wires qed-core into Tauri and exposes it to the UI as
//! commands. Keep logic out of here; it belongs in qed-core.

use std::path::PathBuf;
use std::sync::Mutex;

use qed_core::generator::{self, BuildReport};
use qed_core::page::FrontMatter;
use qed_core::preview::Preview;
use qed_core::site::{self, Page, Site};
use tauri::State;

/// The running preview server, if any. Tauri keeps one of these for the whole
/// app ("managed state"); the Mutex lets commands on different threads take
/// turns changing it.
#[derive(Default)]
struct PreviewState(Mutex<Option<Preview>>);

/// Called from the UI with `invoke("slugify", { title })`.
#[tauri::command]
fn slugify(title: &str) -> String {
    qed_core::slug::slugify(title)
}

/// Reads the site folder at `path`. Called with `invoke("open_site", { path })`.
///
/// `async` makes Tauri run it off the main thread, so the window stays
/// responsive while files are read. Errors reach the UI as a message string.
#[tauri::command]
async fn open_site(path: PathBuf) -> Result<Site, String> {
    site::open(&path).map_err(|error| error.to_string())
}

/// Saves a page's settings into its file and returns the saved page.
#[tauri::command]
async fn save_page_settings(
    root: PathBuf,
    file: PathBuf,
    front: FrontMatter,
) -> Result<Page, String> {
    site::save_front_matter(&root, &file, front).map_err(|error| error.to_string())
}

/// Builds the site at `path` into its `_site` folder.
#[tauri::command]
async fn build_site(path: PathBuf) -> Result<BuildReport, String> {
    build(&path)
}

/// What `preview_site` returns: where to look, and what was built.
#[derive(serde::Serialize)]
struct PreviewInfo {
    url: String,
    report: BuildReport,
}

/// Builds the site and serves it locally.
#[tauri::command]
async fn preview_site(
    path: PathBuf,
    state: State<'_, PreviewState>,
) -> Result<PreviewInfo, String> {
    let report = build(&path)?;

    let mut preview = state
        .0
        .lock()
        .map_err(|_| "preview state is broken".to_string())?;
    // Keep a server that already serves this folder; otherwise replace it.
    // Dropping the old `Preview` stops its server.
    if preview.as_ref().is_none_or(|p| p.root() != report.out_dir) {
        *preview = Some(Preview::start(&report.out_dir).map_err(|e| e.to_string())?);
    }
    let url = preview
        .as_ref()
        .map(|p| p.url().to_string())
        .unwrap_or_default();
    Ok(PreviewInfo { url, report })
}

fn build(path: &std::path::Path) -> Result<BuildReport, String> {
    let site = site::open(path).map_err(|error| error.to_string())?;
    generator::build(&site).map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(PreviewState::default())
        .invoke_handler(tauri::generate_handler![
            slugify,
            open_site,
            save_page_settings,
            build_site,
            preview_site
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

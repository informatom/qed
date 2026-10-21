//! The desktop shell: wires qed-core into Tauri and exposes it to the UI as
//! commands. Keep logic out of here; it belongs in qed-core.

use std::path::PathBuf;

use qed_core::site::{self, Site};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![slugify, open_site])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

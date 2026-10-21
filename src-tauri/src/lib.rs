//! The desktop shell: wires qed-core into Tauri and exposes it to the UI as
//! commands. Keep logic out of here; it belongs in qed-core.

/// Called from the UI with `invoke("slugify", { title })`.
#[tauri::command]
fn slugify(title: &str) -> String {
    qed_core::slug::slugify(title)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![slugify])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

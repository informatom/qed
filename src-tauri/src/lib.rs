//! The desktop shell: wires qed-core into Tauri and exposes it to the UI as
//! commands. Keep logic out of here; it belongs in qed-core.

mod secrets;

use std::path::PathBuf;
use std::sync::Mutex;

use qed_core::block::Block;
use qed_core::generator::{self, BuildReport};
use qed_core::media::{self, MediaFile};
use qed_core::page::{FrontMatter, PageDate};
use qed_core::preview::Preview;
use qed_core::sftp::{self, Progress, PublishReport, SftpError};
use qed_core::site::{self, Page, Site, SiteConfig};
use qed_core::target::{self, SftpAuth, SftpTarget, Target};
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

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
///
/// It also lets the window load files from this folder (for media
/// thumbnails); the asset protocol starts with an empty scope.
#[tauri::command]
async fn open_site(app: AppHandle, path: PathBuf) -> Result<Site, String> {
    let site = site::open(&path).map_err(|error| error.to_string())?;
    app.asset_protocol_scope()
        .allow_directory(&path, true)
        .map_err(|error| error.to_string())?;
    Ok(site)
}

/// Lists the site's media files with their size and usage.
#[tauri::command]
async fn list_media(root: PathBuf) -> Result<Vec<MediaFile>, String> {
    let site = site::open(&root).map_err(|error| error.to_string())?;
    media::list(&site).map_err(|error| error.to_string())
}

/// Copies files into the media folder; returns their new names.
#[tauri::command]
async fn import_media(root: PathBuf, paths: Vec<PathBuf>) -> Result<Vec<String>, String> {
    paths
        .iter()
        .map(|path| media::import(&root, path).map_err(|error| error.to_string()))
        .collect()
}

/// Deletes a media file that no page uses.
#[tauri::command]
async fn delete_media(root: PathBuf, name: String) -> Result<(), String> {
    let site = site::open(&root).map_err(|error| error.to_string())?;
    media::delete(&site, &name).map_err(|error| error.to_string())
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

/// Saves a page's blocks into its file and returns the page as it reads back.
#[tauri::command]
async fn save_page_blocks(
    root: PathBuf,
    file: PathBuf,
    blocks: Vec<Block>,
) -> Result<Page, String> {
    site::save_blocks(&root, &file, blocks).map_err(|error| error.to_string())
}

/// Creates an empty page below `parent` (a URL path, `""` for the top level).
#[tauri::command]
async fn create_page(
    root: PathBuf,
    parent: String,
    title: String,
    date: Option<PageDate>,
) -> Result<Page, String> {
    site::create_page(&root, &parent, &title, date).map_err(|error| error.to_string())
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

/// The SFTP settings of a target. The UI sends whole targets, tagged with
/// their `kind`; SFTP is the only kind so far, so this can't fail yet. Once
/// GitHub Pages joins, the compiler will point here.
fn sftp_part(target: Target) -> SftpTarget {
    let Target::Sftp(sftp) = target;
    sftp
}

/// Saves the site's settings into site.toml.
#[tauri::command]
async fn save_site_config(root: PathBuf, config: SiteConfig) -> Result<(), String> {
    site::save_config(&root, &config).map_err(|error| error.to_string())
}

/// The daisyUI theme names the app and generated sites can use.
#[tauri::command]
fn list_themes() -> Vec<&'static str> {
    qed_core::theme::THEMES.to_vec()
}

/// Replaces the site's publish targets in targets.toml.
#[tauri::command]
async fn save_targets(root: PathBuf, targets: Vec<Target>) -> Result<(), String> {
    target::save_targets(&root, &targets).map_err(|error| error.to_string())
}

/// Saves a target's password in the keychain.
#[tauri::command]
async fn set_target_password(target: Target, password: String) -> Result<(), String> {
    secrets::store(&sftp_part(target), SftpAuth::Password, &password)
}

/// What the app knows about a target's secrets, for the target form.
#[derive(serde::Serialize)]
struct SecretStatus {
    has_password: bool,
    /// The public key line, if a key was generated.
    public_key: Option<String>,
}

#[tauri::command]
async fn target_secrets(target: Target) -> Result<SecretStatus, String> {
    let target = sftp_part(target);
    let has_password = secrets::load(&target, SftpAuth::Password)?.is_some();
    let public_key = match secrets::load(&target, SftpAuth::Key)? {
        Some(pem) => Some(sftp::public_key_of(&pem).map_err(|e| e.to_string())?),
        None => None,
    };
    Ok(SecretStatus {
        has_password,
        public_key,
    })
}

/// Generates a new key pair for a target, keeps the private key in the
/// keychain, and returns the public key line for the server.
#[tauri::command]
async fn generate_target_key(target: Target) -> Result<String, String> {
    let target = sftp_part(target);
    let comment = format!("qed {}", target.keychain_account());
    let (private, public) = sftp::generate_key(&comment).map_err(|e| e.to_string())?;
    secrets::store(&target, SftpAuth::Key, &private)?;
    Ok(public)
}

/// The result of testing a connection.
#[derive(serde::Serialize)]
#[serde(tag = "outcome", rename_all = "lowercase")]
enum TestOutcome {
    /// Everything works.
    Connected { message: String },
    /// The server's key isn't trusted yet; the user decides.
    Untrusted { fingerprint: String },
}

#[tauri::command]
async fn test_target(target: Target) -> Result<TestOutcome, String> {
    let target = sftp_part(target);
    let credentials = secrets::credentials(&target)?;
    match sftp::test(&target, &credentials).await {
        Ok(message) => Ok(TestOutcome::Connected { message }),
        Err(SftpError::UnknownHostKey { fingerprint }) => {
            Ok(TestOutcome::Untrusted { fingerprint })
        }
        Err(error) => Err(error.to_string()),
    }
}

/// Builds the site and publishes it to the named target. Progress arrives
/// on `on_progress` while it runs.
#[tauri::command]
async fn publish_site(
    root: PathBuf,
    target_name: String,
    on_progress: Channel<Progress>,
) -> Result<PublishReport, String> {
    let site = site::open(&root).map_err(|error| error.to_string())?;
    let target = site
        .targets
        .iter()
        .find(|t| t.name() == target_name)
        .cloned()
        .ok_or_else(|| format!("there is no target called {target_name}"))?;
    let report = generator::build(&site).map_err(|error| error.to_string())?;

    match target {
        Target::Sftp(sftp_target) => {
            let credentials = secrets::credentials(&sftp_target)?;
            sftp::publish(&sftp_target, &credentials, &report.out_dir, |progress| {
                // A closed window can't receive progress; publishing goes on.
                let _ = on_progress.send(progress);
            })
            .await
            .map_err(|error| error.to_string())
        }
    }
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
        // Remembers the window's size, position and maximized state between
        // runs. The very first start uses tauri.conf.json (maximized).
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(PreviewState::default())
        .invoke_handler(tauri::generate_handler![
            slugify,
            open_site,
            list_media,
            import_media,
            delete_media,
            save_page_settings,
            save_page_blocks,
            save_site_config,
            list_themes,
            save_targets,
            set_target_password,
            target_secrets,
            generate_target_key,
            test_target,
            publish_site,
            create_page,
            build_site,
            preview_site
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The JSON the UI sends for a target (see `newSftpTarget` in site.ts).
    /// Commands must accept exactly this shape, `kind` included.
    #[test]
    fn accepts_targets_as_the_ui_sends_them() {
        let json = r#"{"kind":"sftp","name":"Test","host":"127.0.0.1","port":2222,"user":"qed",
            "path":"/www","auth":"password","host_key":null,"delete_stale":false}"#;
        let target: Target = serde_json::from_str(json).unwrap();
        let sftp = sftp_part(target);
        assert_eq!(sftp.keychain_account(), "sftp:qed@127.0.0.1:2222");
    }
}

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

/// The URL slug for a title. The UI calls it as `commands.slugify(title)`.
#[tauri::command]
#[specta::specta]
fn slugify(title: &str) -> String {
    qed_core::slug::slugify(title)
}

/// Reads the site folder at `path`.
///
/// `async` makes Tauri run it off the main thread, so the window stays
/// responsive while files are read. Errors reach the UI as a message string.
///
/// It also lets the window load files from this folder (for media
/// thumbnails); the asset protocol starts with an empty scope.
#[tauri::command]
#[specta::specta]
async fn open_site(app: AppHandle, path: PathBuf) -> Result<Site, String> {
    let site = site::open(&path).map_err(|error| error.to_string())?;
    app.asset_protocol_scope()
        .allow_directory(&path, true)
        .map_err(|error| error.to_string())?;
    Ok(site)
}

/// Lists the site's media files with their size and usage.
#[tauri::command]
#[specta::specta]
async fn list_media(root: PathBuf) -> Result<Vec<MediaFile>, String> {
    let site = site::open(&root).map_err(|error| error.to_string())?;
    media::list(&site).map_err(|error| error.to_string())
}

/// Copies files into the media folder; returns their new names.
#[tauri::command]
#[specta::specta]
async fn import_media(root: PathBuf, paths: Vec<PathBuf>) -> Result<Vec<String>, String> {
    paths
        .iter()
        .map(|path| media::import(&root, path).map_err(|error| error.to_string()))
        .collect()
}

/// Deletes a media file that no page uses.
#[tauri::command]
#[specta::specta]
async fn delete_media(root: PathBuf, name: String) -> Result<(), String> {
    let site = site::open(&root).map_err(|error| error.to_string())?;
    media::delete(&site, &name).map_err(|error| error.to_string())
}

/// Saves a page's settings into its file and returns the saved page.
#[tauri::command]
#[specta::specta]
async fn save_page_settings(
    root: PathBuf,
    file: PathBuf,
    front: FrontMatter,
) -> Result<Page, String> {
    site::save_front_matter(&root, &file, front).map_err(|error| error.to_string())
}

/// Saves a page's blocks into its file and returns the page as it reads back.
#[tauri::command]
#[specta::specta]
async fn save_page_blocks(
    root: PathBuf,
    file: PathBuf,
    blocks: Vec<Block>,
) -> Result<Page, String> {
    site::save_blocks(&root, &file, blocks).map_err(|error| error.to_string())
}

/// Creates an empty page below `parent` (a URL path, `""` for the top level).
#[tauri::command]
#[specta::specta]
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
#[specta::specta]
async fn build_site(path: PathBuf) -> Result<BuildReport, String> {
    build(&path)
}

/// What `preview_site` returns: where to look, and what was built.
#[derive(serde::Serialize, specta::Type)]
struct PreviewInfo {
    url: String,
    report: BuildReport,
}

/// Builds the site and serves it locally.
#[tauri::command]
#[specta::specta]
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
#[specta::specta]
async fn save_site_config(root: PathBuf, config: SiteConfig) -> Result<(), String> {
    site::save_config(&root, &config).map_err(|error| error.to_string())
}

/// The page icons: names and SVG markup, from qed's built-in set.
#[tauri::command]
#[specta::specta]
fn list_icons() -> Vec<(&'static str, &'static str)> {
    qed_core::icons::ICONS
        .iter()
        .map(|(name, svg)| (*name, svg.trim_end()))
        .collect()
}

/// The daisyUI theme names the app and generated sites can use.
#[tauri::command]
#[specta::specta]
fn list_themes() -> Vec<&'static str> {
    qed_core::theme::THEMES.to_vec()
}

/// Replaces the site's publish targets in targets.toml.
#[tauri::command]
#[specta::specta]
async fn save_targets(root: PathBuf, targets: Vec<Target>) -> Result<(), String> {
    target::save_targets(&root, &targets).map_err(|error| error.to_string())
}

/// Saves a target's password in the keychain.
#[tauri::command]
#[specta::specta]
async fn set_target_password(target: Target, password: String) -> Result<(), String> {
    secrets::store(&sftp_part(target), SftpAuth::Password, &password)
}

/// What the app knows about a target's secrets, for the target form.
#[derive(serde::Serialize, specta::Type)]
struct SecretStatus {
    has_password: bool,
    /// The public key line, if a key was generated.
    public_key: Option<String>,
}

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
async fn generate_target_key(target: Target) -> Result<String, String> {
    let target = sftp_part(target);
    let comment = format!("qed {}", target.keychain_account());
    let (private, public) = sftp::generate_key(&comment).map_err(|e| e.to_string())?;
    secrets::store(&target, SftpAuth::Key, &private)?;
    Ok(public)
}

/// The result of testing a connection.
#[derive(serde::Serialize, specta::Type)]
#[serde(tag = "outcome", rename_all = "lowercase")]
enum TestOutcome {
    /// Everything works.
    Connected { message: String },
    /// The server's key isn't trusted yet; the user decides.
    Untrusted { fingerprint: String },
}

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
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

/// Every command the UI can call. The same list generates the TypeScript
/// types and `invoke` wrappers in src/lib/bindings.ts; the
/// `bindings_are_current` test rewrites that file when it falls behind.
fn commands() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::new()
        .commands(tauri_specta::collect_commands![
            slugify,
            open_site,
            list_media,
            import_media,
            delete_media,
            save_page_settings,
            save_page_blocks,
            save_site_config,
            list_themes,
            list_icons,
            save_targets,
            set_target_password,
            target_secrets,
            generate_target_key,
            test_target,
            publish_site,
            create_page,
            build_site,
            preview_site,
        ])
        // Failed commands reject their promise with the error message, as
        // plain `invoke` does, instead of resolving to a result object.
        .error_handling(tauri_specta::ErrorHandlingMode::Throw)
        // Sizes and counts are `u64`/`usize` in Rust; as JS numbers they are
        // exact up to 2^53 bytes, far beyond any site.
        .dangerously_cast_bigints_to_number()
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
        .invoke_handler(commands().invoke_handler())
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

    /// Regenerates src/lib/bindings.ts and fails if that changed it, so a
    /// stale file can't be committed unnoticed: run the tests again and
    /// commit the new file.
    #[test]
    fn bindings_are_current() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/lib/bindings.ts");
        let before = std::fs::read_to_string(&path).unwrap_or_default();
        commands()
            .export(specta_typescript::Typescript::default(), &path)
            .unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        assert!(
            before == after,
            "src/lib/bindings.ts was out of date and has been regenerated"
        );
    }
}

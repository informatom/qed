//! The desktop shell: wires qed-core into Tauri and exposes it to the UI as
//! commands. Keep logic out of here; it belongs in qed-core.

mod secrets;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use qed_core::block::Block;
use qed_core::contributors;
use qed_core::editing;
use qed_core::fonts::{self, FontInfo, FontListing};
use qed_core::generator::{self, BuildReport};
use qed_core::history::{
    self, DeletedPage, DiffLine, HistoryStatus, RestingVersion, SincePublish, Version,
};
use qed_core::media::{self, Library, MediaFile};
use qed_core::page::{FrontMatter, PageContent, PageDate};
use qed_core::podcast;
use qed_core::preview::Preview;
use qed_core::remote::{self, RemoteError, RemoteInfo, RemoteReport, Transfer};
use qed_core::sftp::{self, Progress, PublishReport, SftpError};
use qed_core::shared;
use qed_core::site::{self, Page, SaveOutcome, Site, SiteConfig};
use qed_core::target::{self, SftpAuth, SftpTarget, Target};
use qed_core::theme::{CustomTheme, ThemeVariable};
use qed_core::video::{self, VideoInfo};
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

/// The running preview server, if any. Tauri keeps one of these for the whole
/// app ("managed state"); the Mutex lets commands on different threads take
/// turns changing it.
#[derive(Default)]
struct PreviewState(Mutex<Option<Preview>>);

/// The built manual and its server, with the theme it was built in.
#[derive(Default)]
struct ManualState(Mutex<Option<(String, Preview)>>);

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

/// Makes `path` a new site (see `site::create_site`) and starts its version
/// history. The UI opens it afterwards with `open_site`.
#[tauri::command]
#[specta::specta]
async fn create_site(
    path: PathBuf,
    title: String,
    language: String,
    podcast: bool,
) -> Result<(), String> {
    site::create_site(&path, &title, &language).map_err(|error| error.to_string())?;
    if podcast {
        podcast::start_podcast(&path).map_err(|error| error.to_string())?;
    }
    // The site is there either way; without history the Versions tab
    // offers to start it.
    if let Err(error) = history::start(&path) {
        eprintln!("qed: couldn't start the version history: {error}");
    }
    Ok(())
}

/// Lists the files in `media/`, `assets/` or `static/` with their size and usage.
#[tauri::command]
#[specta::specta]
async fn list_media(root: PathBuf, library: Library) -> Result<Vec<MediaFile>, String> {
    let site = site::open(&root).map_err(|error| error.to_string())?;
    media::list(&site, library).map_err(|error| error.to_string())
}

/// Copies files into `media/`, `assets/` or `static/` (also folders); returns their new names.
#[tauri::command]
#[specta::specta]
async fn import_media(
    root: PathBuf,
    library: Library,
    paths: Vec<PathBuf>,
) -> Result<Vec<String>, String> {
    let names = paths
        .iter()
        .map(|path| media::import(&root, library, path).map_err(|error| error.to_string()))
        .collect();
    record(&root);
    names
}

/// The text of a file in `media/`, `assets/` or `static/`, to read in the app.
#[tauri::command]
#[specta::specta]
async fn read_library_file(
    root: PathBuf,
    library: Library,
    name: String,
) -> Result<String, String> {
    media::read_text(&root, library, &name).map_err(|error| error.to_string())
}

/// Opens a file in `media/`, `assets/` or `static/` in the computer's app
/// for it (a PDF in the PDF reader).
#[tauri::command]
#[specta::specta]
async fn open_library_file(
    app: tauri::AppHandle,
    root: PathBuf,
    library: Library,
    name: String,
) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let path = media::path_of(&root, library, &name).map_err(|error| error.to_string())?;
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|error| error.to_string())
}

/// Copies files or folders into a folder inside `static/` (`""` for the
/// top); returns their paths there.
#[tauri::command]
#[specta::specta]
async fn import_static(
    root: PathBuf,
    folder: String,
    paths: Vec<PathBuf>,
) -> Result<Vec<String>, String> {
    let names = paths
        .iter()
        .map(|path| media::import_static(&root, &folder, path).map_err(|error| error.to_string()))
        .collect();
    record(&root);
    names
}

/// Deletes a file (or a folder in `static/`) that no page uses.
#[tauri::command]
#[specta::specta]
async fn delete_media(root: PathBuf, library: Library, name: String) -> Result<(), String> {
    let site = site::open(&root).map_err(|error| error.to_string())?;
    media::delete(&site, library, &name).map_err(|error| error.to_string())?;
    record(&root);
    Ok(())
}

/// Moves the files in `media/` that aren't images into `assets/` and
/// updates the pages; returns how many moved.
#[tauri::command]
#[specta::specta]
async fn move_documents_to_assets(root: PathBuf) -> Result<usize, String> {
    let site = site::open(&root).map_err(|error| error.to_string())?;
    let moved = media::move_documents_to_assets(&site).map_err(|error| error.to_string())?;
    record(&root);
    Ok(moved)
}

/// Saves a page's settings into its file, unless they changed on disk
/// since the editor read them as `base` (`None` saves anyway).
#[tauri::command]
#[specta::specta]
async fn save_page_settings(
    root: PathBuf,
    file: PathBuf,
    front: FrontMatter,
    base: Option<FrontMatter>,
) -> Result<SaveOutcome, String> {
    let outcome = site::save_front_matter_checked(&root, &file, front, base.as_ref())
        .map_err(|error| error.to_string())?;
    if matches!(outcome, SaveOutcome::Saved { .. }) {
        record(&root);
    }
    Ok(outcome)
}

/// Saves a page's blocks into its file, unless they changed on disk since
/// the editor read them as `base` (`None` saves anyway).
#[tauri::command]
#[specta::specta]
async fn save_page_blocks(
    root: PathBuf,
    file: PathBuf,
    blocks: Vec<Block>,
    base: Option<Vec<Block>>,
) -> Result<SaveOutcome, String> {
    let outcome = site::save_blocks_checked(&root, &file, blocks, base.as_deref())
        .map_err(|error| error.to_string())?;
    if matches!(outcome, SaveOutcome::Saved { .. }) {
        record(&root);
    }
    Ok(outcome)
}

/// Saves the site footer's blocks into `footer.md`; no blocks removes it.
/// Returns the blocks as they read back.
#[tauri::command]
#[specta::specta]
async fn save_footer(root: PathBuf, blocks: Vec<Block>) -> Result<Vec<Block>, String> {
    let footer = site::save_footer(&root, blocks).map_err(|error| error.to_string())?;
    record(&root);
    Ok(footer.map(|page| page.content.blocks).unwrap_or_default())
}

/// Saves the episode layout into `episode.md`; the default layout removes
/// it. Returns the blocks as they read back.
#[tauri::command]
#[specta::specta]
async fn save_episode_layout(root: PathBuf, blocks: Vec<Block>) -> Result<Vec<Block>, String> {
    let layout = site::save_episode_layout(&root, blocks).map_err(|error| error.to_string())?;
    record(&root);
    Ok(layout.map_or_else(
        || qed_core::block::DEFAULT_EPISODE_LAYOUT.to_vec(),
        |page| page.content.blocks,
    ))
}

/// Makes a shared piece from these blocks (usually one, from "Make
/// shared"); its name comes from the title.
#[tauri::command]
#[specta::specta]
async fn create_shared(root: PathBuf, title: String, blocks: Vec<Block>) -> Result<Page, String> {
    let piece = shared::create(&root, &title, blocks).map_err(|error| error.to_string())?;
    record(&root);
    Ok(piece)
}

/// Saves a shared piece's blocks and returns it as it reads back.
#[tauri::command]
#[specta::specta]
async fn save_shared(root: PathBuf, name: String, blocks: Vec<Block>) -> Result<Page, String> {
    let piece = shared::save_blocks(&root, &name, blocks).map_err(|error| error.to_string())?;
    record(&root);
    Ok(piece)
}

/// Gives a shared piece a new title; its name, which pages use, stays.
#[tauri::command]
#[specta::specta]
async fn rename_shared(root: PathBuf, name: String, title: String) -> Result<Page, String> {
    let piece = shared::save_title(&root, &name, &title).map_err(|error| error.to_string())?;
    record(&root);
    Ok(piece)
}

/// Deletes a shared piece no page uses any more.
#[tauri::command]
#[specta::specta]
async fn delete_shared(root: PathBuf, name: String) -> Result<(), String> {
    let site = site::open(&root).map_err(|error| error.to_string())?;
    shared::delete(&site, &name).map_err(|error| error.to_string())?;
    record(&root);
    Ok(())
}

/// Saves a contributor (a new one when `id` is empty) and returns them as
/// saved.
#[tauri::command]
#[specta::specta]
async fn save_contributor(
    root: PathBuf,
    contributor: contributors::Contributor,
) -> Result<contributors::Contributor, String> {
    let saved = contributors::save(&root, &contributor).map_err(|error| error.to_string())?;
    record(&root);
    Ok(saved)
}

/// Deletes a contributor no episode or the podcast's team lists any more.
#[tauri::command]
#[specta::specta]
async fn delete_contributor(root: PathBuf, id: String) -> Result<(), String> {
    let site = site::open(&root).map_err(|error| error.to_string())?;
    contributors::delete(&site, &id).map_err(|error| error.to_string())?;
    record(&root);
    Ok(())
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
    let page =
        site::create_page(&root, &parent, &title, date).map_err(|error| error.to_string())?;
    record(&root);
    Ok(page)
}

/// Makes a new episode below `/episodes`, dated today, with `number` or
/// else the next one; the first one also makes the `/episodes` page.
#[tauri::command]
#[specta::specta]
async fn create_episode(root: PathBuf, title: String, number: Option<u32>) -> Result<Page, String> {
    let page = podcast::create_episode(&root, &title, number).map_err(|error| error.to_string())?;
    record(&root);
    Ok(page)
}

/// Apple's podcast categories as they're written in `site.toml`:
/// `"Arts"`, `"Arts > Books"`, …
#[tauri::command]
#[specta::specta]
fn podcast_categories() -> Vec<String> {
    podcast::category_names()
}

/// What's wrong with an image in `media/` as podcast cover art; empty if
/// it follows Apple's rules.
#[tauri::command]
#[specta::specta]
async fn check_cover(root: PathBuf, name: String) -> Vec<String> {
    podcast::check_cover(&root.join(site::MEDIA_DIR), &name)
}

/// Size, type and length of a file in `audio/`, for an episode.
#[tauri::command]
#[specta::specta]
async fn audio_info(root: PathBuf, name: String) -> Result<podcast::AudioInfo, String> {
    let path = media::path_of(&root, Library::Audio, &name).map_err(|e| e.to_string())?;
    podcast::audio_info(&path)
}

/// The pages that link to the page at `path`, for the delete confirmation.
#[tauri::command]
#[specta::specta]
async fn pages_linking_to(root: PathBuf, path: String) -> Result<Vec<String>, String> {
    let site = site::open(&root).map_err(|error| error.to_string())?;
    Ok(site::pages_linking_to(&site, &path))
}

/// The languages the manual is in, for the App tab's choice.
#[tauri::command]
#[specta::specta]
fn manual_languages() -> Vec<String> {
    qed_core::manual::languages()
}

/// Starts a page's translation into `language` (see
/// `site::create_translation`) and returns it.
#[tauri::command]
#[specta::specta]
async fn create_translation(
    root: PathBuf,
    file: PathBuf,
    language: String,
) -> Result<Page, String> {
    let site = site::open(&root).map_err(|error| error.to_string())?;
    site::create_translation(&site, &file, &language).map_err(|error| error.to_string())
}

/// Deletes a page's file, after closing the open version (see
/// `editing::delete_page`).
#[tauri::command]
#[specta::specta]
async fn delete_page(root: PathBuf, file: PathBuf) -> Result<(), String> {
    editing::delete_page(&root, &file).map_err(|error| error.to_string())?;
    record(&root);
    Ok(())
}

/// Gives a page a new address (see `moving::move_page`) and returns it at
/// its new place; the open version is closed first.
#[tauri::command]
#[specta::specta]
async fn move_page(
    root: PathBuf,
    file: PathBuf,
    parent: String,
    slug: String,
) -> Result<Page, String> {
    let page =
        editing::move_page(&root, &file, &parent, &slug).map_err(|error| error.to_string())?;
    record(&root);
    Ok(page)
}

/// Puts sibling pages in this order, by their weights (see
/// `site::set_order`).
#[tauri::command]
#[specta::specta]
async fn set_page_order(root: PathBuf, files: Vec<PathBuf>) -> Result<(), String> {
    let site = site::open(&root).map_err(|error| error.to_string())?;
    if site::set_order(&site, &files).map_err(|error| error.to_string())? > 0 {
        record(&root);
    }
    Ok(())
}

/// Pages that earlier versions had and the folder doesn't any more.
#[tauri::command]
#[specta::specta]
async fn deleted_pages(root: PathBuf) -> Result<Vec<DeletedPage>, String> {
    history::deleted_pages(&root).map_err(|error| error.to_string())
}

/// Brings a deleted page back, as it was in `version`.
#[tauri::command]
#[specta::specta]
async fn restore_page(root: PathBuf, file: String, version: String) -> Result<(), String> {
    history::restore_page(&root, &file, &version).map_err(|error| error.to_string())?;
    record(&root);
    Ok(())
}

/// Puts the whole site back as it was in `version`, as a new version on
/// top; the present is saved as a version first.
#[tauri::command]
#[specta::specta]
async fn restore_site(root: PathBuf, version: String) -> Result<Version, String> {
    history::restore_site(&root, &version).map_err(|error| error.to_string())
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

/// Opens the manual (#110) in a window of its own, at `page` (`versions/`,
/// or empty for its start). It's built from the pages compiled into the
/// app (see `qed_core::manual`), in the app's `theme`, once per run or
/// theme, and served like a preview. Links to other websites open in the
/// browser.
#[tauri::command]
#[specta::specta]
async fn open_manual(
    app: AppHandle,
    theme: String,
    language: String,
    page: Option<String>,
    state: State<'_, ManualState>,
) -> Result<(), String> {
    let base = {
        let mut manual = state
            .0
            .lock()
            .map_err(|_| "manual state is broken".to_string())?;
        let wanted = format!("{theme} {language}");
        if manual.as_ref().is_none_or(|(built, _)| *built != wanted) {
            let folder = app
                .path()
                .app_cache_dir()
                .map_err(|error| error.to_string())?
                .join("manual");
            let report =
                qed_core::manual::build(&folder, &theme, &language).map_err(|e| e.to_string())?;
            let server = Preview::start(&report.out_dir).map_err(|e| e.to_string())?;
            *manual = Some((wanted, server));
        }
        manual
            .as_ref()
            .map(|(_, server)| server.url().to_string())
            .unwrap_or_default()
    };
    let address = format!("{base}{}", page.unwrap_or_default().trim_start_matches('/'));
    let url: tauri::Url = address.parse().map_err(|error| format!("{error}"))?;

    if let Some(window) = app.get_webview_window("manual") {
        window.navigate(url).map_err(|error| error.to_string())?;
        window.show().map_err(|error| error.to_string())?;
        return window.set_focus().map_err(|error| error.to_string());
    }
    let opener = app.clone();
    tauri::WebviewWindowBuilder::new(&app, "manual", tauri::WebviewUrl::External(url))
        .title("q.e.d. manual")
        .inner_size(1100.0, 850.0)
        .on_navigation(move |target| {
            // The manual's own pages stay here; anything else goes to the
            // browser, where it can be read properly.
            if target.as_str().starts_with(&base) {
                return true;
            }
            use tauri_plugin_opener::OpenerExt;
            let _ = opener.opener().open_url(target.as_str(), None::<&str>);
            false
        })
        .build()
        .map_err(|error| error.to_string())?;
    Ok(())
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

/// Saves the site's settings into site.toml and returns them as saved
/// (a new podcast gets its `podcast:guid` here).
#[tauri::command]
#[specta::specta]
async fn save_site_config(root: PathBuf, config: SiteConfig) -> Result<SiteConfig, String> {
    let saved = site::save_config(&root, &config).map_err(|error| error.to_string())?;
    record(&root);
    Ok(saved)
}

/// Icons for a search (see `icons::search`): names and SVG markup, best
/// first, at most `limit`.
#[tauri::command]
#[specta::specta]
fn search_icons(query: String, limit: u32) -> Vec<(String, String)> {
    qed_core::icons::search(&query, limit as usize)
        .into_iter()
        .filter_map(|name| Some((name.to_string(), qed_core::icons::svg(name)?.to_string())))
        .collect()
}

/// The SVG markup of these icons, by the names asked for (old Lucide
/// names too); names that aren't icons are left out.
#[tauri::command]
#[specta::specta]
fn icon_svgs(names: Vec<String>) -> Vec<(String, String)> {
    names
        .into_iter()
        .filter_map(|name| {
            let svg = qed_core::icons::svg(&name)?.to_string();
            Some((name, svg))
        })
        .collect()
}

/// Every font Fontsource offers (the Google Fonts and more).
#[tauri::command]
#[specta::specta]
async fn list_fonts() -> Result<Vec<FontListing>, String> {
    fonts::list().await.map_err(|error| error.to_string())
}

/// Downloads a font into the site's `fonts/` folder.
#[tauri::command]
#[specta::specta]
async fn download_font(root: PathBuf, family: String) -> Result<FontInfo, String> {
    let info = fonts::download(&root, &family)
        .await
        .map_err(|error| error.to_string())?;
    record(&root);
    Ok(info)
}

/// Fetches a video's title and thumbnail (YouTube, Vimeo, PeerTube) and
/// keeps the thumbnail in `media/`.
#[tauri::command]
#[specta::specta]
async fn fetch_video(root: PathBuf, url: String) -> Result<VideoInfo, String> {
    let info = video::fetch(&root, &url)
        .await
        .map_err(|error| error.to_string())?;
    record(&root);
    Ok(info)
}

/// The CSS variables a custom theme sets, for previews in the app.
#[tauri::command]
#[specta::specta]
fn custom_theme_variables(custom: CustomTheme) -> Vec<ThemeVariable> {
    custom.variables()
}

/// The themes the app and generated sites can use (daisyUI's and the
/// Catppuccin flavors), each marked light or dark.
#[tauri::command]
#[specta::specta]
fn list_themes() -> Vec<qed_core::theme::ThemeChoice> {
    qed_core::theme::choices()
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

/// Builds the site and publishes it to the named target. `note` says
/// what's new; it's required when there's an open version or a change
/// since the last version (`HistoryStatus::needs_note`). Progress arrives
/// on `on_progress` while it runs.
#[tauri::command]
#[specta::specta]
async fn publish_site(
    root: PathBuf,
    target_name: String,
    note: Option<String>,
    on_progress: Channel<Progress>,
) -> Result<PublishReport, String> {
    let site = site::open(&root).map_err(|error| error.to_string())?;
    podcast::check_publish(&site).map_err(|error| error.to_string())?;
    let target = site
        .targets
        .iter()
        .find(|t| t.name() == target_name)
        .cloned()
        .ok_or_else(|| format!("there is no target called {target_name}"))?;
    // With a history, what gets published is saved as a version with the
    // note first, and tagged once publishing worked.
    let tracked = history::status(&root)
        .map_err(|error| error.to_string())?
        .tracked;
    history::close_for_publish(&root, note.as_deref()).map_err(|error| error.to_string())?;
    let report = generator::build(&site).map_err(|error| error.to_string())?;

    let published = match target {
        Target::Sftp(sftp_target) => {
            let credentials = secrets::credentials(&sftp_target)?;
            sftp::publish(&sftp_target, &credentials, &report.out_dir, |progress| {
                // A closed window can't receive progress; publishing goes on.
                let _ = on_progress.send(progress);
            })
            .await
            .map_err(|error| error.to_string())
        }
    }?;
    if tracked {
        history::tag_publish(&root, &target_name).map_err(|error| error.to_string())?;
    }
    Ok(published)
}

/// Whether the site has a version history, and what changed since the last
/// version.
#[tauri::command]
#[specta::specta]
async fn history_status(root: PathBuf) -> Result<HistoryStatus, String> {
    history::status(&root).map_err(|error| error.to_string())
}

/// What changed since the site was last published to the named target.
#[tauri::command]
#[specta::specta]
async fn since_publish(root: PathBuf, target_name: String) -> Result<SincePublish, String> {
    history::since_publish(&root, &target_name).map_err(|error| error.to_string())
}

/// Makes the site folder a git repository and saves a first version.
#[tauri::command]
#[specta::specta]
async fn start_history(root: PathBuf) -> Result<Option<Version>, String> {
    history::start(&root).map_err(|error| error.to_string())
}

/// Records a change the app just saved in the site's open version (see
/// `history::record_save`). The change itself is saved already, and a
/// problem with the history mustn't turn that into an error; the Versions
/// tab then shows the files as changed since the last version.
fn record(root: &std::path::Path) {
    if let Err(error) = history::record_save(root) {
        eprintln!("qed: couldn't record the change in the history: {error}");
    }
}

/// Closes the open version with a note (or saves a new one); `None` if
/// there was nothing to save.
#[tauri::command]
#[specta::specta]
async fn save_version(root: PathBuf, note: String) -> Result<Option<Version>, String> {
    history::save_version(&root, &note).map_err(|error| error.to_string())
}

/// The open version, if it has rested an hour or since an earlier day, with
/// a note made from its times; the app asks for a better one then.
#[tauri::command]
#[specta::specta]
async fn resting_version(root: PathBuf) -> Result<Option<RestingVersion>, String> {
    history::resting_version(&root).map_err(|error| error.to_string())
}

/// Closes the open version with a note, as it was last saved; `None` if
/// there's no open version.
#[tauri::command]
#[specta::specta]
async fn close_open_version(root: PathBuf, note: String) -> Result<Option<Version>, String> {
    history::close_open_version(&root, &note).map_err(|error| error.to_string())
}

/// The program an AI assistant starts, with `mcp` and the site folder,
/// to work on a site (#105): q.e.d. itself. An AppImage runs from a new
/// temporary folder each time; there it's the AppImage file, which stays
/// put.
#[tauri::command]
#[specta::specta]
fn mcp_program() -> Result<PathBuf, String> {
    if let Some(appimage) = std::env::var_os("APPIMAGE") {
        return Ok(PathBuf::from(appimage));
    }
    std::env::current_exe().map_err(|error| error.to_string())
}

/// The versions of the site, or of one file in it, newest first.
#[tauri::command]
#[specta::specta]
async fn list_versions(root: PathBuf, file: Option<String>) -> Result<Vec<Version>, String> {
    history::versions(&root, file.as_deref()).map_err(|error| error.to_string())
}

/// A page as it was in an earlier version.
#[tauri::command]
#[specta::specta]
async fn page_version(root: PathBuf, file: String, version: String) -> Result<PageContent, String> {
    history::page_at(&root, &file, &version).map_err(|error| error.to_string())
}

/// The lines a version changed in a file.
#[tauri::command]
#[specta::specta]
async fn version_changes(
    root: PathBuf,
    file: String,
    version: String,
) -> Result<Vec<DiffLine>, String> {
    history::changes_in(&root, &file, &version).map_err(|error| error.to_string())
}

/// How a push, pull, check or "get a site from git" went.
#[derive(serde::Serialize, specta::Type)]
#[serde(tag = "outcome", rename_all = "lowercase")]
enum RemoteOutcome {
    Done {
        report: RemoteReport,
    },
    /// The server's key isn't trusted yet; the user decides.
    Untrusted {
        fingerprint: String,
    },
}

/// Runs a git network operation off the async threads (it blocks while
/// it talks to the server), with q.e.d.'s key for `url`.
async fn over_ssh(
    url: String,
    on_progress: Channel<Transfer>,
    operation: impl FnOnce(&str, &dyn Fn(Transfer)) -> Result<RemoteReport, RemoteError>
    + Send
    + 'static,
) -> Result<RemoteOutcome, String> {
    let account = remote::key_account(&url);
    let key = secrets::load_remote_key(&account)?
        .ok_or_else(|| format!("q.e.d. has no key for {url} yet; make one first"))?;
    let result = tauri::async_runtime::spawn_blocking(move || {
        // A closed window can't receive progress; the operation goes on.
        operation(&key, &|transfer| {
            let _ = on_progress.send(transfer);
        })
    })
    .await
    .map_err(|error| error.to_string())?;
    match result {
        Ok(report) => Ok(RemoteOutcome::Done { report }),
        Err(RemoteError::UnknownHostKey { fingerprint }) => {
            Ok(RemoteOutcome::Untrusted { fingerprint })
        }
        Err(error) => Err(error.to_string()),
    }
}

fn remote_url(root: &std::path::Path, name: &str) -> Result<String, String> {
    remote::remotes(root)
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|remote| remote.name == name)
        .map(|remote| remote.url)
        .ok_or_else(|| format!("there is no remote called {name}"))
}

/// The site's git remotes.
#[tauri::command]
#[specta::specta]
async fn list_remotes(root: PathBuf) -> Result<Vec<RemoteInfo>, String> {
    remote::remotes(&root).map_err(|error| error.to_string())
}

/// Adds a git remote (an SSH address) to the site's repository.
#[tauri::command]
#[specta::specta]
async fn add_remote(root: PathBuf, name: String, url: String) -> Result<RemoteInfo, String> {
    remote::add_remote(&root, &name, &url).map_err(|error| error.to_string())
}

/// Removes a git remote from the site's repository; the server keeps it.
#[tauri::command]
#[specta::specta]
async fn remove_remote(root: PathBuf, name: String) -> Result<(), String> {
    remote::remove_remote(&root, &name).map_err(|error| error.to_string())
}

/// Trusts the remote server's key from now on.
#[tauri::command]
#[specta::specta]
async fn trust_remote(
    root: PathBuf,
    name: String,
    fingerprint: String,
) -> Result<RemoteInfo, String> {
    remote::trust(&root, &name, &fingerprint).map_err(|error| error.to_string())
}

/// The public key q.e.d. logs in to `url` with, if it made one.
#[tauri::command]
#[specta::specta]
async fn remote_key(url: String) -> Result<Option<String>, String> {
    match secrets::load_remote_key(&remote::key_account(&url))? {
        Some(pem) => Ok(Some(sftp::public_key_of(&pem).map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

/// Makes a key pair for `url`, keeps the private key in the keychain and
/// returns the public key, to add on the server.
#[tauri::command]
#[specta::specta]
async fn generate_remote_key(url: String) -> Result<String, String> {
    let account = remote::key_account(&url);
    let comment = format!("qed {account}");
    let (private, public) = sftp::generate_key(&comment).map_err(|e| e.to_string())?;
    secrets::store_remote_key(&account, &private)?;
    Ok(public)
}

/// Asks the remote whether it has versions this computer doesn't.
#[tauri::command]
#[specta::specta]
async fn check_remote(
    root: PathBuf,
    name: String,
    on_progress: Channel<Transfer>,
) -> Result<RemoteOutcome, String> {
    let url = remote_url(&root, &name)?;
    over_ssh(url, on_progress, move |key, progress| {
        remote::check(&root, &name, key, progress)
    })
    .await
}

/// Pushes the site's versions to the remote (closing the open version
/// with `note` first).
#[tauri::command]
#[specta::specta]
async fn push_site(
    root: PathBuf,
    name: String,
    note: Option<String>,
    on_progress: Channel<Transfer>,
) -> Result<RemoteOutcome, String> {
    let url = remote_url(&root, &name)?;
    over_ssh(url, on_progress, move |key, progress| {
        remote::push(&root, &name, key, note.as_deref(), progress)
    })
    .await
}

/// Pulls the remote's versions into the site (closing the open version
/// with `note` first).
#[tauri::command]
#[specta::specta]
async fn pull_site(
    root: PathBuf,
    name: String,
    note: Option<String>,
    on_progress: Channel<Transfer>,
) -> Result<RemoteOutcome, String> {
    let url = remote_url(&root, &name)?;
    over_ssh(url, on_progress, move |key, progress| {
        remote::pull(&root, &name, key, note.as_deref(), progress)
    })
    .await
}

/// Gets a site from a git remote into a new folder, with its history.
#[tauri::command]
#[specta::specta]
async fn clone_site(
    url: String,
    folder: PathBuf,
    host_key: Option<String>,
    on_progress: Channel<Transfer>,
) -> Result<RemoteOutcome, String> {
    let address = url.clone();
    over_ssh(url, on_progress, move |key, progress| {
        remote::clone_site(&address, &folder, key, host_key.as_deref(), progress)
    })
    .await
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
            create_site,
            list_media,
            import_media,
            import_static,
            read_library_file,
            open_library_file,
            delete_media,
            move_documents_to_assets,
            save_page_settings,
            save_page_blocks,
            save_footer,
            save_episode_layout,
            create_shared,
            save_shared,
            rename_shared,
            delete_shared,
            save_contributor,
            delete_contributor,
            save_site_config,
            list_themes,
            custom_theme_variables,
            fetch_video,
            list_fonts,
            download_font,
            search_icons,
            icon_svgs,
            save_targets,
            set_target_password,
            target_secrets,
            generate_target_key,
            test_target,
            publish_site,
            create_page,
            create_episode,
            podcast_categories,
            check_cover,
            audio_info,
            pages_linking_to,
            delete_page,
            create_translation,
            manual_languages,
            move_page,
            set_page_order,
            deleted_pages,
            restore_page,
            restore_site,
            history_status,
            since_publish,
            start_history,
            save_version,
            resting_version,
            mcp_program,
            open_manual,
            close_open_version,
            list_versions,
            page_version,
            version_changes,
            list_remotes,
            add_remote,
            remove_remote,
            trust_remote,
            remote_key,
            generate_remote_key,
            check_remote,
            push_site,
            pull_site,
            clone_site,
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

/// `qed mcp /path/to/site` (#105): serves the site to an AI assistant
/// over stdin and stdout until it hangs up. Messages for people go to
/// stderr; stdout belongs to the protocol.
pub fn serve_mcp(root: Option<std::ffi::OsString>) -> std::process::ExitCode {
    let Some(root) = root else {
        eprintln!("usage: qed mcp /path/to/site");
        return std::process::ExitCode::FAILURE;
    };
    match qed_mcp::serve(Path::new(&root)) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("qed mcp: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        // Restarts the app after an update.
        .plugin(tauri_plugin_process::init())
        // Remembers the window's size, position and maximized state between
        // runs. The very first start uses tauri.conf.json (maximized).
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(PreviewState::default())
        .manage(ManualState::default())
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

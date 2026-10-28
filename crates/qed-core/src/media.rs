//! The site's folders of files: `media/` for images, `assets/` for files to
//! download, such as PDFs, `static/` for files copied into the built site
//! as they are (slide shows, `keybase.txt`), and `audio/` for a podcast's
//! episodes.
//!
//! Pages refer to media and assets by name: `image = "team.jpg"` and
//! `file = "menu.pdf"` in blocks, or `![Team](/media/team.jpg)` and
//! `[Menu](/assets/menu.pdf)` in Markdown. Static files keep their own
//! address: `static/pdf/a.pdf` is linked as `/pdf/a.pdf`. The files stay as
//! they are; the site generator copies them into the built site.

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::block::{self, Block};
use crate::icons;
use crate::markdown;
use crate::site::{self, ASSETS_DIR, MEDIA_DIR, Page, STATIC_DIR, Site, SiteError};
use crate::slug::slugify;

/// System clutter that `static/` doesn't list or copy.
pub(crate) const CLUTTER: [&str; 3] = [".DS_Store", "Thumbs.db", "desktop.ini"];

/// Which of the folders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum Library {
    /// `media/`: images only.
    Media,
    /// `assets/`: files to download, of any type.
    Assets,
    /// `static/`: copied into the built site at the same paths.
    Static,
    /// `audio/`: the podcast's audio files, one per episode.
    Audio,
}

impl Library {
    pub fn dir(self) -> &'static str {
        match self {
            Library::Media => MEDIA_DIR,
            Library::Assets => ASSETS_DIR,
            Library::Static => STATIC_DIR,
            Library::Audio => crate::podcast::AUDIO_DIR,
        }
    }
}

/// What kind of file a media file is, judged by its extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Image,
    Audio,
    Document,
    /// Plain text the app can show: HTML, CSS, JavaScript, `.txt`, …
    Text,
    Other,
}

/// One file in `media/`, `assets/` or `static/`.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MediaFile {
    /// Path inside the folder, with `/` between folders: `team.jpg`, `logos/a.png`.
    pub name: String,
    /// Size in bytes.
    pub size: u64,
    pub kind: MediaKind,
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// URL paths of the pages that use this file.
    pub used_by: Vec<String>,
    /// The site's logo (Site settings).
    pub site_logo: bool,
    /// The podcast's cover art (Site settings).
    pub podcast_cover: bool,
}

/// Lists every file in one of the folders, sorted by name.
pub fn list(site: &Site, library: Library) -> Result<Vec<MediaFile>, SiteError> {
    let folder = site.root.join(library.dir());
    let mut names = Vec::new();
    if folder.is_dir() {
        collect(&folder, &folder, library == Library::Static, &mut names)?;
    }
    names.sort();

    let usage: Vec<(&str, Vec<String>)> = site
        .with_blocks()
        .map(|page| (page.path.as_str(), references(page, library)))
        .collect();

    names
        .into_iter()
        .map(|name| {
            let path = folder.join(&name);
            let size = fs::metadata(&path)
                .map_err(|source| SiteError::Io {
                    path: path.clone(),
                    source,
                })?
                .len();
            let kind = kind_of(&name);
            let (width, height) = match kind {
                MediaKind::Image => {
                    image::image_dimensions(&path).map_or((None, None), |(w, h)| (Some(w), Some(h)))
                }
                _ => (None, None),
            };
            let used_by = usage
                .iter()
                .filter(|(_, names)| names.contains(&name))
                .map(|(page, _)| page.to_string())
                .collect();
            let site_logo = library == Library::Media && is_site_logo(site, &name);
            let podcast_cover = library == Library::Media && is_podcast_cover(site, &name);
            Ok(MediaFile {
                name,
                site_logo,
                podcast_cover,
                size,
                kind,
                width,
                height,
                used_by,
            })
        })
        .collect()
}

/// The largest file [`read_text`] shows, in bytes.
const MAX_TEXT: u64 = 2 * 1024 * 1024;

/// Reads a text file from one of the folders, to show it in the app.
/// Refuses files that aren't UTF-8 text or are larger than 2 MB.
pub fn read_text(root: &Path, library: Library, name: &str) -> Result<String, SiteError> {
    let path = file_path(root, library, name)?;
    let invalid = |message: String| SiteError::Invalid {
        path: PathBuf::from(library.dir()).join(name),
        message,
    };
    let read_error = |source| SiteError::Io {
        path: path.clone(),
        source,
    };
    if !path.is_file() {
        return Err(invalid("is a folder".to_string()));
    }
    let size = fs::metadata(&path).map_err(read_error)?.len();
    if size > MAX_TEXT {
        return Err(invalid(format!(
            "is too large to show here ({} MB)",
            size / (1024 * 1024)
        )));
    }
    let bytes = fs::read(&path).map_err(read_error)?;
    match String::from_utf8(bytes) {
        Ok(text) if !text.contains('\0') => Ok(text),
        _ => Err(invalid("isn't a text file".to_string())),
    }
}

/// Copies a file into one of the folders and returns its new name. `media/`
/// takes only images.
///
/// The name is made URL-friendly (`Team Photo.JPG` becomes `team-photo.jpg`),
/// and gets a number if it's taken (`team-photo-2.jpg`). Existing files are
/// never overwritten.
///
/// `static/` keeps the name as it is, since it's the file's address, and
/// also takes whole folders; a name that's taken is an error there.
pub fn import(root: &Path, library: Library, source: &Path) -> Result<String, SiteError> {
    if library == Library::Static {
        return import_static(root, "", source);
    }
    let read_error = |source_error| SiteError::Io {
        path: source.to_path_buf(),
        source: source_error,
    };
    if !source.is_file() {
        return Err(SiteError::Invalid {
            path: source.to_path_buf(),
            message: "only files can be imported".to_string(),
        });
    }

    let original_name = source
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    if library == Library::Media && kind_of(&original_name) != MediaKind::Image {
        return Err(SiteError::Invalid {
            path: source.to_path_buf(),
            message: "isn't an image; files to download go into Assets".to_string(),
        });
    }
    if library == Library::Audio {
        if kind_of(&original_name) != MediaKind::Audio {
            return Err(SiteError::Invalid {
                path: source.to_path_buf(),
                message: "isn't an audio file (MP3, M4A, Ogg, Opus, …)".to_string(),
            });
        }
        // Before the first file lands, so it never shows up in a version.
        let site = site::open(root)?;
        crate::podcast::sync_audio_ignore(root, site.config.podcast.as_ref())?;
    }

    let stem = source
        .file_stem()
        .map(|s| slugify(&s.to_string_lossy()))
        .unwrap_or_default();
    let stem = if stem.is_empty() {
        "file".to_string()
    } else {
        stem
    };
    let extension = source
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
        .unwrap_or_default();

    let mut input = fs::File::open(source).map_err(read_error)?;
    write_unique(&root.join(library.dir()), &stem, &extension, &mut input)
}

/// Writes `input` into `folder` as `{stem}{extension}`, or with a number
/// (`{stem}-2{extension}`) if that's taken, and returns the name used.
/// Never overwrites a file.
pub(crate) fn write_unique(
    folder: &Path,
    stem: &str,
    extension: &str,
    input: &mut impl io::Read,
) -> Result<String, SiteError> {
    fs::create_dir_all(folder).map_err(|source| SiteError::Write {
        path: folder.to_path_buf(),
        source,
    })?;
    for number in 1.. {
        let name = match number {
            1 => format!("{stem}{extension}"),
            n => format!("{stem}-{n}{extension}"),
        };
        let target = folder.join(&name);
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
        {
            Ok(mut output) => {
                io::copy(input, &mut output).map_err(|source| SiteError::Write {
                    path: target,
                    source,
                })?;
                return Ok(name);
            }
            // Taken: try the next number.
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(source) => {
                return Err(SiteError::Write {
                    path: target,
                    source,
                });
            }
        }
    }
    unreachable!("the loop above runs until it returns")
}

/// Copies a file or folder into `folder` in `static/` (`""` for the top,
/// `talks/2019`) under its own name. Returns its path inside `static/`.
pub fn import_static(root: &Path, folder: &str, source: &Path) -> Result<String, SiteError> {
    let folder = folder.trim_matches('/');
    let plain = Path::new(folder)
        .components()
        .all(|c| matches!(c, Component::Normal(_)));
    if !plain {
        return Err(SiteError::Invalid {
            path: PathBuf::from(STATIC_DIR).join(folder),
            message: "not a folder inside static".to_string(),
        });
    }
    let file_name = source
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    let name = match folder {
        "" => file_name.clone(),
        _ => format!("{folder}/{file_name}"),
    };
    let target = root.join(STATIC_DIR).join(&name);
    if file_name.is_empty() || target.exists() {
        return Err(SiteError::Invalid {
            path: PathBuf::from(STATIC_DIR).join(&name),
            message: "already exists; delete or rename it first".to_string(),
        });
    }
    if source.is_dir() {
        copy_tree(source, &target)?;
    } else {
        copy_new(source, &target)?;
    }
    Ok(name)
}

/// Copies a folder with everything in it but clutter.
fn copy_tree(from: &Path, to: &Path) -> Result<(), SiteError> {
    let read_error = |source| SiteError::Io {
        path: from.to_path_buf(),
        source,
    };
    fs::create_dir_all(to).map_err(|source| SiteError::Write {
        path: to.to_path_buf(),
        source,
    })?;
    for entry in fs::read_dir(from).map_err(read_error)? {
        let entry = entry.map_err(read_error)?;
        let name = entry.file_name();
        if CLUTTER.contains(&name.to_string_lossy().as_ref()) {
            continue;
        }
        let (source, target) = (entry.path(), to.join(&name));
        if source.is_dir() {
            copy_tree(&source, &target)?;
        } else {
            copy_new(&source, &target)?;
        }
    }
    Ok(())
}

/// Copies a file to a path that must not exist yet.
fn copy_new(from: &Path, to: &Path) -> Result<(), SiteError> {
    let write_error = |source| SiteError::Write {
        path: to.to_path_buf(),
        source,
    };
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(write_error)?;
    }
    let mut input = fs::File::open(from).map_err(|source| SiteError::Io {
        path: from.to_path_buf(),
        source,
    })?;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(to)
        .map_err(write_error)?;
    io::copy(&mut input, &mut output).map_err(write_error)?;
    Ok(())
}

/// Deletes a file from one of the folders, unless a page still uses it. In
/// `static/`, `name` may also be a folder, deleted with everything in it.
pub fn delete(site: &Site, library: Library, name: &str) -> Result<(), SiteError> {
    let path = file_path(&site.root, library, name)?;
    let inside = format!("{name}/");
    let used_by: Vec<String> = site
        .with_blocks()
        .filter(|page| {
            references(page, library)
                .iter()
                .any(|n| n == name || n.starts_with(&inside))
        })
        .map(|page| match page.path.as_str() {
            site::FOOTER_PATH => "the footer".to_string(),
            path if path.starts_with('(') => {
                format!("the shared block \"{}\"", page.content.front.title)
            }
            path => format!("/{path}"),
        })
        .collect();
    if library == Library::Media && is_site_logo(site, name) {
        return Err(SiteError::Invalid {
            path: PathBuf::from(library.dir()).join(name),
            message: "it's the site's logo; choose another one in Site settings first".to_string(),
        });
    }
    if library == Library::Media && is_podcast_cover(site, name) {
        return Err(SiteError::Invalid {
            path: PathBuf::from(library.dir()).join(name),
            message: "it's the podcast's cover; choose another one in the Podcast tab first"
                .to_string(),
        });
    }
    if !used_by.is_empty() {
        return Err(SiteError::Invalid {
            path: PathBuf::from(library.dir()).join(name),
            message: format!("still used by {}", used_by.join(", ")),
        });
    }
    let removed = if path.is_dir() {
        fs::remove_dir_all(&path)
    } else {
        fs::remove_file(&path)
    };
    removed.map_err(|source| SiteError::Write {
        path: path.clone(),
        source,
    })?;
    // Folders the file leaves empty go too, up to the library folder.
    let top = site.root.join(library.dir());
    let mut dir = path.parent();
    while let Some(folder) = dir.filter(|folder| *folder != top) {
        if fs::remove_dir(folder).is_err() {
            break;
        }
        dir = folder.parent();
    }
    Ok(())
}

/// Whether `name` in `media/` is the podcast's cover art.
fn is_podcast_cover(site: &Site, name: &str) -> bool {
    site.config
        .podcast
        .as_ref()
        .and_then(|podcast| podcast.cover.as_deref())
        .and_then(media_name)
        .is_some_and(|cover| cover == name)
}

/// Whether `name` in `media/` is the logo set in Site settings.
fn is_site_logo(site: &Site, name: &str) -> bool {
    site.config
        .logo
        .as_deref()
        .and_then(media_name)
        .is_some_and(|logo| logo == name)
}

/// The files in `library` a page uses, by name.
///
/// Download blocks name files in `assets/`. Sites from before `assets/`
/// existed kept them in `media/`, so their names count there too, until
/// [`move_documents_to_assets`] has moved them.
pub fn references(page: &Page, library: Library) -> Vec<String> {
    if library == Library::Audio {
        return page
            .content
            .front
            .episode
            .as_ref()
            .and_then(|episode| episode.local_audio())
            .map(str::to_string)
            .into_iter()
            .collect();
    }
    let mut images = Vec::new();
    let mut downloads = Vec::new();
    let mut markdown_texts: Vec<&str> = Vec::new();
    let mut buttons: Vec<&str> = Vec::new();
    images.extend(page.content.front.image.as_deref().and_then(media_name));
    images.extend(
        page.content
            .front
            .episode
            .as_ref()
            .and_then(|episode| episode.image.as_deref())
            .and_then(media_name),
    );

    for block in block::all_blocks(&page.content.blocks) {
        match block {
            Block::Text { markdown } => markdown_texts.push(markdown),
            Block::Hero(hero) => {
                images.extend(hero.image.as_deref().and_then(media_name));
                markdown_texts.extend(hero.text.as_deref());
                buttons.extend(hero.button_link.as_deref());
            }
            Block::Image(image) => {
                images.extend(image.image.as_deref().and_then(media_name));
                markdown_texts.extend(image.caption.as_deref());
            }
            Block::TextImage(block) => {
                images.extend(block.image.as_deref().and_then(media_name));
                markdown_texts.extend(block.text.as_deref());
            }
            Block::Gallery(gallery) => {
                images.extend(
                    gallery
                        .images
                        .iter()
                        .filter_map(|image| media_name(&image.image)),
                );
            }
            Block::Download(download) => {
                downloads.extend(download.file.as_deref().and_then(media_name))
            }
            Block::Faq(faq) => {
                markdown_texts.extend(faq.items.iter().map(|item| item.answer.as_str()))
            }
            // Their blocks come next in `all_blocks`.
            Block::Columns(_) => {}
            Block::CallToAction(cta) => {
                markdown_texts.extend(cta.text.as_deref());
                buttons.extend(cta.button_link.as_deref());
            }
            Block::Video(video) => images.extend(video.image.as_deref().and_then(media_name)),
            // The piece's own file counts its files.
            Block::PostList(_) | Block::PageList(_) | Block::Shared(_) => {}
        }
    }

    if library == Library::Static {
        let mut names: Vec<String> = markdown_texts
            .iter()
            .flat_map(|text| markdown::link_targets(text))
            .chain(buttons.iter().map(|link| link.to_string()))
            .flat_map(|target| static_names(&target))
            .collect();
        names.sort();
        names.dedup();
        return names;
    }

    let mut names = downloads;
    if library == Library::Media {
        names.extend(images);
    }
    let prefix = format!("{}/", library.dir());
    for text in markdown_texts {
        // A site icon (`:icon-rails:`) is one of these files, whichever exists.
        if library == Library::Media {
            for icon in markdown::icon_names(text) {
                names.extend(
                    icons::SITE_ICON_EXTENSIONS
                        .iter()
                        .map(|extension| format!("{}/{icon}.{extension}", icons::SITE_ICONS_DIR)),
                );
            }
        }
        for target in markdown::link_targets(text) {
            let path = target.split(['?', '#']).next().unwrap_or_default();
            if let Some(name) = path.trim_start_matches('/').strip_prefix(prefix.as_str()) {
                names.push(name.to_string());
            }
        }
    }

    names.sort();
    names.dedup();
    names
}

/// The names in `static/` a site link may mean: `/pdf/a.pdf` is
/// `pdf/a.pdf`, `/slides/talk/` is `slides/talk/index.html`, and
/// `/slides/talk` could be either.
fn static_names(target: &str) -> Vec<String> {
    let path = target.split(['?', '#']).next().unwrap_or_default();
    let Some(path) = path.strip_prefix('/').filter(|p| !p.starts_with('/')) else {
        return Vec::new();
    };
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return Vec::new();
    }
    let index = format!("{trimmed}/index.html");
    if path.ends_with('/') {
        vec![index]
    } else {
        vec![trimmed.to_string(), index]
    }
}

/// A block's image field holds a media name, unless it's a URL or site path.
fn media_name(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty() && !value.contains(':') && !value.starts_with('/'))
        .then(|| value.to_string())
}

/// The full path of a file in one of the folders, for opening it in
/// another app. Only plain names inside the folder (no `..`) are accepted.
pub fn path_of(root: &Path, library: Library, name: &str) -> Result<PathBuf, SiteError> {
    file_path(root, library, name)
}

/// Checks a file name from the UI and returns the file's full path.
fn file_path(root: &Path, library: Library, name: &str) -> Result<PathBuf, SiteError> {
    let relative = Path::new(name);
    let plain = relative
        .components()
        .all(|c| matches!(c, Component::Normal(_)));
    let path = root.join(library.dir()).join(relative);
    if plain && (path.is_file() || (library == Library::Static && path.is_dir())) {
        Ok(path)
    } else {
        Err(SiteError::Invalid {
            path: PathBuf::from(library.dir()).join(name),
            message: "no such file".to_string(),
        })
    }
}

/// Moves every file in `media/` that isn't an image into `assets/`, and
/// changes the pages that use them to match. Returns how many files moved.
///
/// For sites from before `assets/` existed, when `media/` held downloads
/// too.
pub fn move_documents_to_assets(site: &Site) -> Result<usize, SiteError> {
    let media = site.root.join(MEDIA_DIR);
    let mut names = Vec::new();
    if media.is_dir() {
        collect(&media, &media, false, &mut names)?;
    }

    // Old name in media/ → new name in assets/.
    let mut moved = Vec::new();
    for name in names {
        if kind_of(&name) == MediaKind::Image {
            continue;
        }
        let source = media.join(&name);
        let new_name = import(&site.root, Library::Assets, &source)?;
        fs::remove_file(&source).map_err(|error| SiteError::Write {
            path: source.clone(),
            source: error,
        })?;
        moved.push((name, new_name));
    }

    for page in &site.pages {
        let used = references(page, Library::Media);
        if moved.iter().any(|(old, _)| used.contains(old)) {
            site::update_page(&site.root, &page.file, |content| {
                for block in &mut content.blocks {
                    rename_in_block(block, &moved);
                }
            })?;
        }
    }
    Ok(moved.len())
}

/// Points a block's download and its Markdown links at the moved files.
fn rename_in_block(block: &mut Block, moved: &[(String, String)]) {
    if let Block::Columns(columns) = block {
        for inner in columns
            .items
            .iter_mut()
            .flat_map(|column| column.blocks.iter_mut())
        {
            rename_in_block(inner, moved);
        }
        return;
    }
    if let Block::Download(download) = block
        && let Some(file) = &mut download.file
        && let Some((_, new_name)) = moved.iter().find(|(old, _)| old == file.trim())
    {
        *file = new_name.clone();
    }
    block::for_each_markdown(block, &mut |text| {
        for (old, new_name) in moved {
            *text = replace_link(
                text,
                &format!("/{MEDIA_DIR}/{old}"),
                &format!("/{ASSETS_DIR}/{new_name}"),
            );
        }
    });
}

/// Replaces the link `from` with `to` where it's a whole link, so
/// `/media/menu.pdf` doesn't match the start of `/media/menu.pdf.old`.
fn replace_link(text: &str, from: &str, to: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(index) = rest.find(from) {
        let after = &rest[index + from.len()..];
        let whole = after.chars().next().is_none_or(|c| {
            matches!(c, ')' | '#' | '?' | '"' | '\'' | '>' | ']') || c.is_whitespace()
        });
        result.push_str(&rest[..index]);
        result.push_str(if whole { to } else { from });
        rest = after;
    }
    result.push_str(rest);
    result
}

/// Lists the files below `dir`; hidden ones only with `hidden` (for
/// `static/`, where `.well-known/` matters), clutter never.
fn collect(
    folder: &Path,
    dir: &Path,
    hidden: bool,
    names: &mut Vec<String>,
) -> Result<(), SiteError> {
    let error = |source| SiteError::Io {
        path: dir.to_path_buf(),
        source,
    };
    for entry in fs::read_dir(dir).map_err(error)? {
        let path = entry.map_err(error)?.path();
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if (!hidden && file_name.starts_with('.')) || CLUTTER.contains(&file_name.as_str()) {
            continue;
        }
        if path.is_dir() {
            collect(folder, &path, hidden, names)?;
        } else if let Ok(relative) = path.strip_prefix(folder) {
            let parts: Vec<_> = relative
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect();
            names.push(parts.join("/"));
        }
    }
    Ok(())
}

fn kind_of(name: &str) -> MediaKind {
    let extension = name
        .rsplit_once('.')
        .map(|(_, e)| e.to_lowercase())
        .unwrap_or_default();
    match extension.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "avif" => MediaKind::Image,
        "mp3" | "m4a" | "aac" | "ogg" | "oga" | "opus" | "flac" | "wav" => MediaKind::Audio,
        "pdf" | "zip" | "doc" | "docx" | "odt" | "xls" | "xlsx" | "ods" | "epub" => {
            MediaKind::Document
        }
        // No extension (`CNAME`) or a dotfile (`.htaccess`) is usually text, too.
        "" | "txt" | "md" | "html" | "htm" | "css" | "js" | "mjs" | "json" | "xml" | "rss"
        | "atom" | "csv" | "tsv" | "yml" | "yaml" | "toml" | "ini" | "conf" | "log" | "ics"
        | "vtt" | "srt" | "map" | "webmanifest" | "htaccess" => MediaKind::Text,
        _ => MediaKind::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site;

    /// A 2×1 pixel PNG.
    fn png() -> Vec<u8> {
        let mut bytes = Vec::new();
        image::RgbImage::new(2, 1)
            .write_to(&mut io::Cursor::new(&mut bytes), image::ImageFormat::Png)
            .unwrap();
        bytes
    }

    fn site_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("pages")).unwrap();
        fs::create_dir_all(dir.path().join("media/logos")).unwrap();
        fs::write(dir.path().join("site.toml"), "title = \"S\"").unwrap();
        fs::write(
            dir.path().join("pages/index.md"),
            "+++\ntitle = \"Home\"\n+++\n\n::: hero\nimage = \"team.png\"\n:::\n\nSee ![logo](/media/logos/a.png) and [the PDF](/media/menu.pdf#page=2).\n",
        )
        .unwrap();
        fs::write(dir.path().join("media/team.png"), png()).unwrap();
        fs::write(dir.path().join("media/logos/a.png"), png()).unwrap();
        fs::write(dir.path().join("media/menu.pdf"), "%PDF").unwrap();
        fs::write(dir.path().join("media/unused.txt"), "hi").unwrap();
        fs::write(dir.path().join("media/.gitkeep"), "").unwrap();
        dir
    }

    #[test]
    fn lists_files_with_kind_size_dimensions_and_usage() {
        let dir = site_dir();
        let files = list(&site::open(dir.path()).unwrap(), Library::Media).unwrap();
        let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["logos/a.png", "menu.pdf", "team.png", "unused.txt"]);

        let team = &files[2];
        assert_eq!(team.kind, MediaKind::Image);
        assert_eq!((team.width, team.height), (Some(2), Some(1)));
        assert_eq!(team.used_by, [""]);
        assert_eq!(files[1].kind, MediaKind::Document);
        assert_eq!(
            files[1].used_by,
            [""],
            "linked from Markdown, with a #fragment"
        );
        assert!(files[3].used_by.is_empty());
    }

    #[test]
    fn keeps_the_site_logo() {
        let dir = site_dir();
        fs::write(
            dir.path().join("site.toml"),
            "title = \"S\"\nlogo = \"unused.txt\"\n",
        )
        .unwrap();
        let site = site::open(dir.path()).unwrap();

        let files = list(&site, Library::Media).unwrap();
        assert!(files[3].site_logo && !files[2].site_logo);
        let error = delete(&site, Library::Media, "unused.txt").unwrap_err();
        assert!(error.to_string().contains("site's logo"), "{error}");
    }

    #[test]
    fn counts_the_share_image_as_used() {
        let dir = site_dir();
        fs::write(
            dir.path().join("pages/news.md"),
            "+++\ntitle = \"News\"\nimage = \"logos/a.png\"\n+++\n",
        )
        .unwrap();
        let files = list(&site::open(dir.path()).unwrap(), Library::Media).unwrap();
        assert_eq!(files[0].used_by, ["", "news"]);
    }

    #[test]
    fn imports_with_a_clean_name_and_never_overwrites() {
        let dir = site_dir();
        let source = dir.path().join("Team Photo.PNG");
        fs::write(&source, png()).unwrap();

        assert_eq!(
            import(dir.path(), Library::Media, &source).unwrap(),
            "team-photo.png"
        );
        assert_eq!(
            import(dir.path(), Library::Media, &source).unwrap(),
            "team-photo-2.png"
        );
        assert!(dir.path().join("media/team-photo-2.png").is_file());
    }

    #[test]
    fn deletes_only_unused_files() {
        let dir = site_dir();
        let site = site::open(dir.path()).unwrap();

        let error = delete(&site, Library::Media, "team.png").unwrap_err();
        assert!(error.to_string().contains("still used by /"), "{error}");
        assert!(dir.path().join("media/team.png").exists());

        delete(&site, Library::Media, "unused.txt").unwrap();
        assert!(!dir.path().join("media/unused.txt").exists());

        assert!(delete(&site, Library::Media, "../site.toml").is_err());
    }

    #[test]
    fn opens_only_files_inside_the_folder() {
        let dir = site_dir();
        assert_eq!(
            path_of(dir.path(), Library::Media, "team.png").unwrap(),
            dir.path().join("media/team.png")
        );
        assert!(path_of(dir.path(), Library::Media, "../site.toml").is_err());
        assert!(path_of(dir.path(), Library::Media, "missing.pdf").is_err());
    }

    #[test]
    fn site_icons_in_text_are_used() {
        let dir = site_dir();
        fs::create_dir(dir.path().join("media/icons")).unwrap();
        fs::write(dir.path().join("media/icons/rails.svg"), "<svg/>").unwrap();
        fs::write(
            dir.path().join("pages/stack.md"),
            "+++\ntitle = \"Stack\"\n+++\n\n:icon-rails: Rails\n",
        )
        .unwrap();
        let site = site::open(dir.path()).unwrap();
        let rails = list(&site, Library::Media)
            .unwrap()
            .into_iter()
            .find(|file| file.name == "icons/rails.svg")
            .unwrap();
        assert_eq!(rails.used_by, ["stack"]);
        let error = delete(&site, Library::Media, "icons/rails.svg").unwrap_err();
        assert!(
            error.to_string().contains("still used by /stack"),
            "{error}"
        );
    }

    #[test]
    fn renders_and_tracks_the_newer_blocks() {
        let dir = site_dir();
        fs::write(
            dir.path().join("pages/more.md"),
            r#"+++
title = "More"
+++

::: text-image
text = "Some *text*."
image = "team.png"
alt = "Team"
image_side = "left"
:::

::: gallery
[[images]]
image = "team.png"
alt = "One"

[[images]]
image = "logos/a.png"
:::

::: download
file = "menu.pdf"
label = "Our menu"
:::

::: faq
heading = "Questions"

[[items]]
question = "Open on Sundays?"
answer = "See [the menu](/media/menu.pdf)."
:::
"#,
        )
        .unwrap();
        fs::write(
            dir.path().join("pages/cols.md"),
            "+++\ntitle = \"Cols\"\n+++\n\n:::: columns\n::: column\ntitle = \"One\"\n:::\n![x](/media/team.png)\n\n::: image\nimage = \"logos/a.png\"\n:::\n::: column\n:::\nTwo\n::: column\ntitle = \"Three\"\n:::\n::::\n",
        )
        .unwrap();
        let site = site::open(dir.path()).unwrap();
        let cols = site.pages.iter().find(|p| p.path == "cols").unwrap();
        assert_eq!(
            references(cols, Library::Media),
            ["logos/a.png", "team.png"],
            "images inside columns count"
        );
        let more = site.pages.iter().find(|p| p.path == "more").unwrap();
        assert_eq!(
            references(more, Library::Media),
            ["logos/a.png", "menu.pdf", "team.png"]
        );

        let out = tempfile::tempdir().unwrap();
        crate::generator::build_into(&site, out.path()).unwrap();
        let html = fs::read_to_string(out.path().join("more/index.html")).unwrap();
        assert!(html.contains("md:order-first"), "image on the left");
        assert!(html.contains("<p>Some <em>text</em>.</p>"), "{html}");
        assert!(
            html.contains(r#"<button type="button" popovertarget="zoom-logos-a-png""#)
                && html.contains(r#"<div id="zoom-logos-a-png" popover class="image-zoom">"#),
            "gallery images open big: {html}"
        );
        assert!(
            html.contains(r#"<a href="../media/menu.pdf" download"#),
            "a download still in media/, from before assets/: {html}"
        );
        assert!(
            html.contains(">PDF</span>") && html.contains(">Our menu</span>"),
            "{html}"
        );
        assert!(
            html.contains(
                "<summary class=\"collapse-title font-medium\">Open on Sundays?</summary>"
            ),
            "{html}"
        );

        let html = fs::read_to_string(out.path().join("cols/index.html")).unwrap();
        assert!(
            html.contains("grid gap-8 md:grid-cols-3"),
            "three columns: {html}"
        );
        assert!(
            html.contains(">One</h2>") && html.contains("<p>Two</p>"),
            "{html}"
        );
    }

    #[test]
    fn renders_image_blocks_with_their_size() {
        let dir = site_dir();
        fs::write(
            dir.path().join("pages/photo.md"),
            "+++\ntitle = \"Photo\"\n+++\n\n::: image\nimage = \"team.png\"\nalt = \"Our team\"\ncaption = \"In *2026*\"\n:::\n",
        )
        .unwrap();
        let site = site::open(dir.path()).unwrap();
        let out = tempfile::tempdir().unwrap();
        crate::generator::build_into(&site, out.path()).unwrap();

        let html = fs::read_to_string(out.path().join("photo/index.html")).unwrap();
        assert!(
            html.contains(
                r#"<img src="../media/team.png" alt="Our team" width="2" height="1" loading="lazy""#
            ),
            "{html}"
        );
        assert!(html.contains("In <em>2026</em>"), "{html}");
    }

    #[test]
    fn media_takes_only_images_and_assets_take_anything() {
        let dir = site_dir();
        let pdf = dir.path().join("Speisekarte Herbst 2026.PDF");
        fs::write(&pdf, "%PDF").unwrap();

        let error = import(dir.path(), Library::Media, &pdf).unwrap_err();
        assert!(error.to_string().contains("isn't an image"), "{error}");
        assert_eq!(
            import(dir.path(), Library::Assets, &pdf).unwrap(),
            "speisekarte-herbst-2026.pdf"
        );
        assert!(
            dir.path()
                .join("assets/speisekarte-herbst-2026.pdf")
                .is_file()
        );
    }

    #[test]
    fn tracks_downloads_in_assets() {
        let dir = site_dir();
        fs::create_dir(dir.path().join("assets")).unwrap();
        fs::write(dir.path().join("assets/price-list.pdf"), "%PDF").unwrap();
        fs::write(dir.path().join("assets/flyer.pdf"), "%PDF").unwrap();
        fs::write(
            dir.path().join("pages/prices.md"),
            "+++\ntitle = \"Prices\"\n+++\n\n::: download\nfile = \"price-list.pdf\"\n:::\n\nAlso [our flyer](/assets/flyer.pdf).\n",
        )
        .unwrap();
        let site = site::open(dir.path()).unwrap();

        let files = list(&site, Library::Assets).unwrap();
        let used: Vec<(&str, &[String])> = files
            .iter()
            .map(|f| (f.name.as_str(), f.used_by.as_slice()))
            .collect();
        assert_eq!(
            used,
            [
                ("flyer.pdf", &["prices".to_string()][..]),
                ("price-list.pdf", &["prices".to_string()][..])
            ]
        );
        assert!(delete(&site, Library::Assets, "flyer.pdf").is_err());

        let out = tempfile::tempdir().unwrap();
        crate::generator::build_into(&site, out.path()).unwrap();
        let html = fs::read_to_string(out.path().join("prices/index.html")).unwrap();
        assert!(
            html.contains(r#"<a href="../assets/price-list.pdf" download"#),
            "{html}"
        );
        assert!(html.contains(r#"href="../assets/flyer.pdf""#), "{html}");
        assert!(out.path().join("assets/price-list.pdf").is_file());
    }

    #[test]
    fn moves_downloads_out_of_media() {
        let dir = site_dir();
        fs::write(
            dir.path().join("pages/menu.md"),
            "+++\ntitle = \"Menu\"\n+++\n\n::: download\nfile = \"menu.pdf\"\nlabel = \"Our menu\"\n:::\n",
        )
        .unwrap();
        let site = site::open(dir.path()).unwrap();

        assert_eq!(move_documents_to_assets(&site).unwrap(), 2);
        assert!(dir.path().join("assets/menu.pdf").is_file());
        assert!(dir.path().join("assets/unused.txt").is_file());
        assert!(!dir.path().join("media/menu.pdf").exists());
        assert!(dir.path().join("media/team.png").is_file(), "images stay");

        let home = fs::read_to_string(dir.path().join("pages/index.md")).unwrap();
        assert!(
            home.contains("[the PDF](/assets/menu.pdf#page=2)"),
            "{home}"
        );
        assert!(home.contains("![logo](/media/logos/a.png)"), "{home}");
        let menu = fs::read_to_string(dir.path().join("pages/menu.md")).unwrap();
        assert!(menu.contains("file = \"menu.pdf\""), "{menu}");
    }

    #[test]
    fn replaces_only_whole_links() {
        assert_eq!(
            replace_link(
                "[a](/media/a.pdf) [b](/media/a.pdf.old)",
                "/media/a.pdf",
                "/assets/a.pdf"
            ),
            "[a](/assets/a.pdf) [b](/media/a.pdf.old)"
        );
    }

    #[test]
    fn keeps_static_files_under_their_own_names() {
        let dir = site_dir();
        let outside = tempfile::tempdir().unwrap();
        let talk = outside.path().join("Talk 2019");
        fs::create_dir_all(talk.join("css")).unwrap();
        fs::write(talk.join("index.html"), "<h1>Talk</h1>").unwrap();
        fs::write(talk.join("css/a.css"), "").unwrap();
        fs::write(talk.join(".DS_Store"), "").unwrap();
        let proof = outside.path().join("keybase.txt");
        fs::write(&proof, "proof").unwrap();

        assert_eq!(
            import(dir.path(), Library::Static, &proof).unwrap(),
            "keybase.txt"
        );
        assert_eq!(
            import(dir.path(), Library::Static, &talk).unwrap(),
            "Talk 2019"
        );
        assert!(
            import(dir.path(), Library::Static, &proof).is_err(),
            "taken"
        );
        assert_eq!(
            import_static(dir.path(), "Talk 2019/css", &proof).unwrap(),
            "Talk 2019/css/keybase.txt"
        );
        assert!(import_static(dir.path(), "../pages", &proof).is_err());
        fs::remove_file(dir.path().join("static/Talk 2019/css/keybase.txt")).unwrap();
        fs::create_dir_all(dir.path().join("static/.well-known")).unwrap();
        fs::write(dir.path().join("static/.well-known/security.txt"), "").unwrap();
        fs::write(
            dir.path().join("pages/about.md"),
            "+++\ntitle = \"About\"\n+++\n\nSee [my talk](/Talk%202019/) and [proof](/keybase.txt).\n\n::: call-to-action\nbutton_label = \"Slides\"\nbutton_link = \"/Talk 2019/\"\n:::\n",
        )
        .unwrap();

        let site = site::open(dir.path()).unwrap();
        let files = list(&site, Library::Static).unwrap();
        let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(
            names,
            [
                ".well-known/security.txt",
                "Talk 2019/css/a.css",
                "Talk 2019/index.html",
                "keybase.txt"
            ]
        );
        let used = |name: &str| {
            files
                .iter()
                .find(|f| f.name == name)
                .map(|f| f.used_by.clone())
                .unwrap()
        };
        assert_eq!(used("keybase.txt"), ["about"]);
        assert_eq!(used("Talk 2019/index.html"), ["about"]);
        assert!(used("Talk 2019/css/a.css").is_empty());

        let error = delete(&site, Library::Static, "Talk 2019").unwrap_err();
        assert!(error.to_string().contains("/about"), "{error}");
        delete(&site, Library::Static, ".well-known/security.txt").unwrap();
        assert!(
            !dir.path().join("static/.well-known").exists(),
            "empty folder gone"
        );
        assert!(dir.path().join("static").is_dir());

        fs::write(
            dir.path().join("pages/about.md"),
            "+++\ntitle = \"About\"\n+++\n",
        )
        .unwrap();
        let site = site::open(dir.path()).unwrap();
        delete(&site, Library::Static, "Talk 2019").unwrap();
        assert!(!dir.path().join("static/Talk 2019").exists());
        assert!(delete(&site, Library::Static, "../pages").is_err());
    }

    #[test]
    fn reads_text_files_only() {
        let dir = site_dir();
        assert_eq!(kind_of("talk/index.html"), MediaKind::Text);
        assert_eq!(kind_of(".htaccess"), MediaKind::Text);
        assert_eq!(
            read_text(dir.path(), Library::Media, "unused.txt").unwrap(),
            "hi"
        );
        let error = read_text(dir.path(), Library::Media, "team.png").unwrap_err();
        assert!(error.to_string().contains("isn't a text file"), "{error}");
        assert!(read_text(dir.path(), Library::Media, "logos").is_err());
        assert!(read_text(dir.path(), Library::Media, "../site.toml").is_err());
    }
}

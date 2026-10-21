//! The site's two folders of files that pages use: `media/` for images and
//! `assets/` for files to download, such as PDFs.
//!
//! Pages refer to these files by name: `image = "team.jpg"` and
//! `file = "menu.pdf"` in blocks, or `![Team](/media/team.jpg)` and
//! `[Menu](/assets/menu.pdf)` in Markdown. The files stay as they are; the
//! site generator copies them into the built site.

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::block::{self, Block};
use crate::markdown;
use crate::site::{self, ASSETS_DIR, MEDIA_DIR, Page, Site, SiteError};
use crate::slug::slugify;

/// Which of the two folders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum Library {
    /// `media/`: images only.
    Media,
    /// `assets/`: files to download, of any type.
    Assets,
}

impl Library {
    pub fn dir(self) -> &'static str {
        match self {
            Library::Media => MEDIA_DIR,
            Library::Assets => ASSETS_DIR,
        }
    }
}

/// What kind of file a media file is, judged by its extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Image,
    Document,
    Other,
}

/// One file in `media/` or `assets/`.
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
}

/// Lists every file in one of the folders, sorted by name.
pub fn list(site: &Site, library: Library) -> Result<Vec<MediaFile>, SiteError> {
    let folder = site.root.join(library.dir());
    let mut names = Vec::new();
    if folder.is_dir() {
        collect(&folder, &folder, &mut names)?;
    }
    names.sort();

    let usage: Vec<(&str, Vec<String>)> = site
        .pages
        .iter()
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
            Ok(MediaFile {
                name,
                size,
                kind,
                width,
                height,
                used_by,
            })
        })
        .collect()
}

/// Copies a file into one of the folders and returns its new name. `media/`
/// takes only images.
///
/// The name is made URL-friendly (`Team Photo.JPG` becomes `team-photo.jpg`),
/// and gets a number if it's taken (`team-photo-2.jpg`). Existing files are
/// never overwritten.
pub fn import(root: &Path, library: Library, source: &Path) -> Result<String, SiteError> {
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

    let folder = root.join(library.dir());
    fs::create_dir_all(&folder).map_err(|source| SiteError::Write {
        path: folder.clone(),
        source,
    })?;

    let mut input = fs::File::open(source).map_err(read_error)?;
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
                io::copy(&mut input, &mut output).map_err(|source| SiteError::Write {
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

/// Deletes a file from one of the folders, unless a page still uses it.
pub fn delete(site: &Site, library: Library, name: &str) -> Result<(), SiteError> {
    let path = file_path(&site.root, library, name)?;
    let used_by: Vec<String> = site
        .pages
        .iter()
        .filter(|page| references(page, library).iter().any(|n| n == name))
        .map(|page| format!("/{}", page.path))
        .collect();
    if !used_by.is_empty() {
        return Err(SiteError::Invalid {
            path: PathBuf::from(library.dir()).join(name),
            message: format!("still used by {}", used_by.join(", ")),
        });
    }
    fs::remove_file(&path).map_err(|source| SiteError::Write { path, source })
}

/// The files in `library` a page uses, by name.
///
/// Download blocks name files in `assets/`. Sites from before `assets/`
/// existed kept them in `media/`, so their names count there too, until
/// [`move_documents_to_assets`] has moved them.
pub fn references(page: &Page, library: Library) -> Vec<String> {
    let mut images = Vec::new();
    let mut downloads = Vec::new();
    let mut markdown_texts: Vec<&str> = Vec::new();

    for block in block::all_blocks(&page.content.blocks) {
        match block {
            Block::Text { markdown } => markdown_texts.push(markdown),
            Block::Hero(hero) => {
                images.extend(hero.image.as_deref().and_then(media_name));
                markdown_texts.extend(hero.text.as_deref());
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
            Block::CallToAction(cta) => markdown_texts.extend(cta.text.as_deref()),
            Block::PostList(_) => {}
        }
    }

    let mut names = downloads;
    if library == Library::Media {
        names.extend(images);
    }
    let prefix = format!("{}/", library.dir());
    for text in markdown_texts {
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

/// A block's image field holds a media name, unless it's a URL or site path.
fn media_name(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty() && !value.contains(':') && !value.starts_with('/'))
        .then(|| value.to_string())
}

/// Checks a file name from the UI and returns the file's full path.
fn file_path(root: &Path, library: Library, name: &str) -> Result<PathBuf, SiteError> {
    let relative = Path::new(name);
    let plain = relative
        .components()
        .all(|c| matches!(c, Component::Normal(_)));
    let path = root.join(library.dir()).join(relative);
    if plain && path.is_file() {
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
        collect(&media, &media, &mut names)?;
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

fn collect(folder: &Path, dir: &Path, names: &mut Vec<String>) -> Result<(), SiteError> {
    let error = |source| SiteError::Io {
        path: dir.to_path_buf(),
        source,
    };
    for entry in fs::read_dir(dir).map_err(error)? {
        let path = entry.map_err(error)?.path();
        let hidden = path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with('.'));
        if hidden {
            continue;
        }
        if path.is_dir() {
            collect(folder, &path, names)?;
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
        "pdf" | "txt" | "zip" | "doc" | "docx" | "odt" | "xls" | "xlsx" | "ods" | "epub" => {
            MediaKind::Document
        }
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
            html.contains(r#"<a href="../media/logos/a.png""#),
            "gallery links to full size"
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
}

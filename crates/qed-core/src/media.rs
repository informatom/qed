//! The site's `media/` folder: images and downloads that pages use.
//!
//! Pages refer to media files by name: `image = "team.jpg"` in a block, or
//! `![Team](/media/team.jpg)` in Markdown. The files stay as they are; the
//! site generator copies them into the built site.

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use serde::Serialize;

use crate::block::Block;
use crate::markdown;
use crate::site::{MEDIA_DIR, Page, Site, SiteError};
use crate::slug::slugify;

/// What kind of file a media file is, judged by its extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Image,
    Document,
    Other,
}

/// One file in the media folder.
#[derive(Debug, Clone, Serialize)]
pub struct MediaFile {
    /// Path inside `media/`, with `/` between folders: `team.jpg`, `logos/a.png`.
    pub name: String,
    /// Size in bytes.
    pub size: u64,
    pub kind: MediaKind,
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// URL paths of the pages that use this file.
    pub used_by: Vec<String>,
}

/// Lists every file in the media folder, sorted by name.
pub fn list(site: &Site) -> Result<Vec<MediaFile>, SiteError> {
    let folder = site.root.join(MEDIA_DIR);
    let mut names = Vec::new();
    if folder.is_dir() {
        collect(&folder, &folder, &mut names)?;
    }
    names.sort();

    let usage: Vec<(&str, Vec<String>)> = site
        .pages
        .iter()
        .map(|page| (page.path.as_str(), references(page)))
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

/// Copies a file into the media folder and returns its new name.
///
/// The name is made URL-friendly (`Team Photo.JPG` becomes `team-photo.jpg`),
/// and gets a number if it's taken (`team-photo-2.jpg`). Existing files are
/// never overwritten.
pub fn import(root: &Path, source: &Path) -> Result<String, SiteError> {
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

    let folder = root.join(MEDIA_DIR);
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

/// Deletes a media file, unless a page still uses it.
pub fn delete(site: &Site, name: &str) -> Result<(), SiteError> {
    let path = media_path(&site.root, name)?;
    let used_by: Vec<String> = site
        .pages
        .iter()
        .filter(|page| references(page).iter().any(|n| n == name))
        .map(|page| format!("/{}", page.path))
        .collect();
    if !used_by.is_empty() {
        return Err(SiteError::Invalid {
            path: PathBuf::from(MEDIA_DIR).join(name),
            message: format!("still used by {}", used_by.join(", ")),
        });
    }
    fs::remove_file(&path).map_err(|source| SiteError::Write { path, source })
}

/// The media files a page uses, by name.
pub fn references(page: &Page) -> Vec<String> {
    let mut names = Vec::new();
    let mut markdown_texts: Vec<&str> = Vec::new();

    for block in &page.content.blocks {
        match block {
            Block::Text { markdown } => markdown_texts.push(markdown),
            Block::Hero(hero) => {
                names.extend(hero.image.as_deref().and_then(media_name));
                markdown_texts.extend(hero.text.as_deref());
            }
            Block::Image(image) => {
                names.extend(image.image.as_deref().and_then(media_name));
                markdown_texts.extend(image.caption.as_deref());
            }
            Block::TextImage(block) => {
                names.extend(block.image.as_deref().and_then(media_name));
                markdown_texts.extend(block.text.as_deref());
            }
            Block::Gallery(gallery) => {
                names.extend(
                    gallery
                        .images
                        .iter()
                        .filter_map(|image| media_name(&image.image)),
                );
            }
            Block::Download(download) => {
                names.extend(download.file.as_deref().and_then(media_name))
            }
            Block::Faq(faq) => {
                markdown_texts.extend(faq.items.iter().map(|item| item.answer.as_str()))
            }
            Block::Columns(columns) => {
                markdown_texts.extend(columns.items.iter().map(|column| column.text.as_str()))
            }
            Block::CallToAction(cta) => markdown_texts.extend(cta.text.as_deref()),
        }
    }

    for text in markdown_texts {
        for target in markdown::link_targets(text) {
            let path = target.split(['?', '#']).next().unwrap_or_default();
            if let Some(name) = path.trim_start_matches('/').strip_prefix("media/") {
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

/// Checks a media name from the UI and returns the file's full path.
fn media_path(root: &Path, name: &str) -> Result<PathBuf, SiteError> {
    let relative = Path::new(name);
    let plain = relative
        .components()
        .all(|c| matches!(c, Component::Normal(_)));
    let path = root.join(MEDIA_DIR).join(relative);
    if plain && path.is_file() {
        Ok(path)
    } else {
        Err(SiteError::Invalid {
            path: PathBuf::from(MEDIA_DIR).join(name),
            message: "no such media file".to_string(),
        })
    }
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
        let files = list(&site::open(dir.path()).unwrap()).unwrap();
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

        assert_eq!(import(dir.path(), &source).unwrap(), "team-photo.png");
        assert_eq!(import(dir.path(), &source).unwrap(), "team-photo-2.png");
        assert!(dir.path().join("media/team-photo-2.png").is_file());
    }

    #[test]
    fn deletes_only_unused_files() {
        let dir = site_dir();
        let site = site::open(dir.path()).unwrap();

        let error = delete(&site, "team.png").unwrap_err();
        assert!(error.to_string().contains("still used by /"), "{error}");
        assert!(dir.path().join("media/team.png").exists());

        delete(&site, "unused.txt").unwrap();
        assert!(!dir.path().join("media/unused.txt").exists());

        assert!(delete(&site, "../site.toml").is_err());
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
            "+++\ntitle = \"Cols\"\n+++\n\n::: columns\n[[items]]\ntitle = \"One\"\ntext = \"![x](/media/team.png)\"\n\n[[items]]\ntext = \"Two\"\n\n[[items]]\ntitle = \"Three\"\n:::\n",
        )
        .unwrap();
        let site = site::open(dir.path()).unwrap();
        let cols = site.pages.iter().find(|p| p.path == "cols").unwrap();
        assert_eq!(
            references(cols),
            ["team.png"],
            "images in column text count"
        );
        let more = site.pages.iter().find(|p| p.path == "more").unwrap();
        assert_eq!(references(more), ["logos/a.png", "menu.pdf", "team.png"]);

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
            "{html}"
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
}

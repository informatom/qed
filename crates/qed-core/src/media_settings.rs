//! `media.toml`: settings for single images in `media/` (#152), kept
//! apart from `site.toml`, which holds the site's settings. Only the Media
//! tab writes it. One table per image, named by its path inside `media/`:
//!
//! ```toml
//! ["portrait.jpg"]
//! focus = { x = 50, y = 20 }
//! ```
//!
//! `focus` is the spot that stays visible where the image is cut to fit
//! (squares in galleries and tiles, the hero image): percent from the left
//! and from the top. Images without it are cut around the middle.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::site::SiteError;

pub const MEDIA_SETTINGS_FILE: &str = "media.toml";

/// The settings of every image that has some, by its path inside `media/`.
pub type MediaSettings = BTreeMap<String, ImageSettings>;

/// What `media.toml` says about one image.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct ImageSettings {
    pub focus: Option<Focus>,
}

/// The spot that stays visible when an image is cut to fit, in percent
/// of its width (`x`, from the left) and height (`y`, from the top).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Focus {
    pub x: u8,
    pub y: u8,
}

impl Focus {
    /// The middle, where images are cut without a focus.
    pub const MIDDLE: Focus = Focus { x: 50, y: 50 };

    /// The CSS `object-position` for it: `50% 20%`.
    pub fn css(self) -> String {
        format!("{}% {}%", self.x, self.y)
    }
}

/// Reads `media.toml`; no file means no settings. Refuses a file the app
/// couldn't have written: unknown keys, or a focus beyond 100%.
pub fn load(root: &Path) -> Result<MediaSettings, SiteError> {
    let path = root.join(MEDIA_SETTINGS_FILE);
    if !path.is_file() {
        return Ok(MediaSettings::new());
    }
    let text = fs::read_to_string(&path).map_err(|source| SiteError::Io {
        path: path.clone(),
        source,
    })?;
    let settings: MediaSettings = toml::from_str(&text).map_err(|source| SiteError::Config {
        path: PathBuf::from(MEDIA_SETTINGS_FILE),
        source,
    })?;
    for (name, image) in &settings {
        if let Some(focus) = image.focus
            && (focus.x > 100 || focus.y > 100)
        {
            return Err(SiteError::Invalid {
                path: PathBuf::from(MEDIA_SETTINGS_FILE),
                message: format!("the focus of \"{name}\" must be between 0 and 100 percent"),
            });
        }
    }
    Ok(settings)
}

/// Sets where `name` (a path inside `media/`) is cut to fit; `None` or the
/// middle removes the setting. Other settings and comments in the file
/// stay as they are; a file left empty is deleted.
pub fn set_focus(root: &Path, name: &str, focus: Option<Focus>) -> Result<(), SiteError> {
    let focus = focus.filter(|focus| *focus != Focus::MIDDLE);
    if let Some(focus) = focus
        && (focus.x > 100 || focus.y > 100)
    {
        return Err(SiteError::Invalid {
            path: PathBuf::from(MEDIA_SETTINGS_FILE),
            message: format!("the focus of \"{name}\" must be between 0 and 100 percent"),
        });
    }
    edit(root, |document| match focus {
        Some(focus) => {
            let mut point = toml_edit::InlineTable::new();
            point.insert("x", i64::from(focus.x).into());
            point.insert("y", i64::from(focus.y).into());
            let image = document
                .entry(name)
                .or_insert_with(|| toml_edit::Item::Table(toml_edit::Table::new()));
            image["focus"] = toml_edit::value(point);
        }
        None => {
            if let Some(image) = document.get_mut(name).and_then(|item| item.as_table_mut()) {
                image.remove("focus");
                if image.is_empty() {
                    document.remove(name);
                }
            }
        }
    })
}

/// Forgets the settings of `name` in `media/`, and of everything inside it
/// if it's a folder; for a deleted file.
pub fn forget(root: &Path, name: &str) -> Result<(), SiteError> {
    let inside = format!("{name}/");
    edit(root, |document| {
        document.retain(|key, _| key != name && !key.starts_with(&inside));
    })
}

/// Changes `media.toml` with `change`, if the file exists or `change`
/// adds something. Deletes the file when nothing is left in it.
fn edit(root: &Path, change: impl FnOnce(&mut toml_edit::Table)) -> Result<(), SiteError> {
    let path = root.join(MEDIA_SETTINGS_FILE);
    let text = if path.is_file() {
        fs::read_to_string(&path).map_err(|source| SiteError::Io {
            path: path.clone(),
            source,
        })?
    } else {
        String::new()
    };
    let mut document: toml_edit::DocumentMut =
        text.parse()
            .map_err(|error: toml_edit::TomlError| SiteError::Invalid {
                path: PathBuf::from(MEDIA_SETTINGS_FILE),
                message: error.message().to_string(),
            })?;
    change(document.as_table_mut());
    let write_error = |source| SiteError::Write {
        path: path.clone(),
        source,
    };
    if document.as_table().is_empty() {
        if path.is_file() {
            fs::remove_file(&path).map_err(write_error)?;
        }
        return Ok(());
    }
    let mut out = document.to_string();
    if text.is_empty() {
        out = format!(
            "# Settings for single images in media/, written by the Media tab.\n\
             # focus: the spot that stays visible where an image is cut to fit,\n\
             # in percent from the left (x) and from the top (y).\n\n{out}"
        );
    }
    fs::write(&path, out).map_err(write_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sets_changes_and_removes_a_focus() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        assert!(load(root).unwrap().is_empty());

        set_focus(root, "portrait.jpg", Some(Focus { x: 50, y: 20 })).unwrap();
        set_focus(root, "logos/wide one.png", Some(Focus { x: 0, y: 100 })).unwrap();
        let text = fs::read_to_string(root.join(MEDIA_SETTINGS_FILE)).unwrap();
        assert!(text.starts_with("# Settings for single images"), "{text}");
        assert!(
            text.contains("[\"portrait.jpg\"]\nfocus = { x = 50, y = 20 }"),
            "{text}"
        );
        let settings = load(root).unwrap();
        assert_eq!(settings["portrait.jpg"].focus, Some(Focus { x: 50, y: 20 }));
        assert_eq!(
            settings["logos/wide one.png"].focus,
            Some(Focus { x: 0, y: 100 })
        );

        set_focus(root, "portrait.jpg", Some(Focus { x: 40, y: 30 })).unwrap();
        assert_eq!(
            load(root).unwrap()["portrait.jpg"].focus,
            Some(Focus { x: 40, y: 30 })
        );
        let text = fs::read_to_string(root.join(MEDIA_SETTINGS_FILE)).unwrap();
        assert!(
            text.starts_with("# Settings for single images"),
            "comment kept: {text}"
        );

        // The middle is the default, so it isn't written.
        set_focus(root, "portrait.jpg", Some(Focus::MIDDLE)).unwrap();
        assert!(!load(root).unwrap().contains_key("portrait.jpg"));

        set_focus(root, "logos/wide one.png", None).unwrap();
        assert!(
            !root.join(MEDIA_SETTINGS_FILE).exists(),
            "an empty file is deleted"
        );
    }

    #[test]
    fn forgets_a_deleted_file_and_a_folder() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let focus = Some(Focus { x: 10, y: 10 });
        set_focus(root, "a.jpg", focus).unwrap();
        set_focus(root, "logos/b.png", focus).unwrap();
        set_focus(root, "logos-old.png", focus).unwrap();

        forget(root, "a.jpg").unwrap();
        forget(root, "logos").unwrap();
        let names: Vec<String> = load(root).unwrap().into_keys().collect();
        assert_eq!(names, ["logos-old.png"]);
        // Nothing to forget: no file appears.
        forget(dir.path().join("nothing").as_path(), "a.jpg").unwrap();
        assert!(!dir.path().join("nothing").exists());
    }

    #[test]
    fn refuses_what_the_app_would_not_write() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let path = root.join(MEDIA_SETTINGS_FILE);
        fs::write(&path, "[\"a.jpg\"]\nfocus = { x = 50, y = 120 }\n").unwrap();
        assert!(
            load(root)
                .unwrap_err()
                .to_string()
                .contains("between 0 and 100")
        );
        fs::write(&path, "[\"a.jpg\"]\nfokus = { x = 50, y = 20 }\n").unwrap();
        assert!(load(root).unwrap_err().to_string().contains("fokus"));
        assert!(set_focus(root, "b.jpg", Some(Focus { x: 101, y: 0 })).is_err());
    }
}

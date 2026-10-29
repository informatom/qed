//! Icons for pages: a curated set of [Lucide](https://lucide.dev) icons
//! (ISC license), built into qed so generated sites get plain SVG and need
//! no JavaScript. The files are in `assets/icons/`.
//!
//! A site can add its own icons, such as the logos of the tools it talks
//! about, as image files in `media/icons/` ([`SiteIcons`]).

use std::collections::BTreeMap;
use std::path::Path;

/// Builds the list of icons from their names: each name becomes a pair of
/// the name and the file's contents, read while compiling.
macro_rules! icons {
    ($($name:literal,)*) => {
        &[$(($name, include_str!(concat!("../assets/icons/", $name, ".svg")))),*]
    };
}

/// Name and SVG markup of every icon, sorted by name.
pub const ICONS: &[(&str, &str)] = icons![
    "archive",
    "award",
    "book-open",
    "bookmark",
    "briefcase",
    "calendar",
    "camera",
    "chart-column",
    "circle-question-mark",
    "clock",
    "code",
    "coffee",
    "compass",
    "component",
    "download",
    "eye",
    "file-plus",
    "file-text",
    "folder",
    "folder-input",
    "gift",
    "git-branch",
    "globe",
    "graduation-cap",
    "hammer",
    "handshake",
    "headphones",
    "heart",
    "history",
    "house",
    "image",
    "info",
    "layers",
    "leaf",
    "life-buoy",
    "lightbulb",
    "link",
    "list",
    "lock",
    "mail",
    "map",
    "map-pin",
    "megaphone",
    "message-circle",
    "mic",
    "music",
    "newspaper",
    "package",
    "pen-line",
    "phone",
    "play",
    "plus",
    "podcast",
    "rocket",
    "rss",
    "save",
    "scale",
    "search",
    "settings",
    "shield",
    "shopping-cart",
    "sparkles",
    "star",
    "sun",
    "tag",
    "trash",
    "undo",
    "upload",
    "user",
    "users",
    "video",
    "wrench",
];

/// The SVG for an icon name, if qed has that icon.
pub fn svg(name: &str) -> Option<&'static str> {
    ICONS
        .iter()
        .find(|(icon, _)| *icon == name)
        .map(|(_, svg)| svg.trim_end())
}

/// The folder in `media/` for a site's own icons.
pub const SITE_ICONS_DIR: &str = "icons";

/// The kinds of files a site icon can be, the preferred first: SVG stays
/// sharp at any size.
pub const SITE_ICON_EXTENSIONS: [&str; 5] = ["svg", "webp", "png", "jpg", "jpeg"];

/// A site's own icons: the image files in `media/icons/`, by name without
/// the extension (`rails` for `rails.svg`). Unlike q.e.d.'s icons they
/// keep their own colors.
#[derive(Debug, Default)]
pub struct SiteIcons(BTreeMap<String, String>);

impl SiteIcons {
    /// The icons in `media_dir`'s icon folder. A site without one has none.
    pub fn load(media_dir: &Path) -> Self {
        let mut icons = BTreeMap::new();
        let entries = std::fs::read_dir(media_dir.join(SITE_ICONS_DIR))
            .into_iter()
            .flatten()
            .flatten();
        for entry in entries {
            let file = entry.file_name().to_string_lossy().into_owned();
            let Some((name, extension)) = file.rsplit_once('.') else {
                continue;
            };
            let Some(rank) = SITE_ICON_EXTENSIONS
                .iter()
                .position(|known| known.eq_ignore_ascii_case(extension))
            else {
                continue;
            };
            let better = icons
                .get(name)
                .is_none_or(|(current, _): &(usize, String)| rank < *current);
            if better {
                icons.insert(name.to_string(), (rank, file));
            }
        }
        SiteIcons(
            icons
                .into_iter()
                .map(|(name, (_, file))| (name, file))
                .collect(),
        )
    }

    /// The file in `media/icons/` for an icon name, if the site has it.
    pub fn file(&self, name: &str) -> Option<&str> {
        self.0.get(name).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_icons_are_the_images_in_media_icons() {
        let media = tempfile::tempdir().unwrap();
        let folder = media.path().join(SITE_ICONS_DIR);
        std::fs::create_dir(&folder).unwrap();
        for file in ["rails.png", "rails.svg", "elixir.png", "notes.txt"] {
            std::fs::write(folder.join(file), "x").unwrap();
        }
        let icons = SiteIcons::load(media.path());
        assert_eq!(icons.file("rails"), Some("rails.svg"));
        assert_eq!(icons.file("elixir"), Some("elixir.png"));
        assert_eq!(icons.file("notes"), None);
        assert_eq!(SiteIcons::load(&folder).file("rails"), None);
    }

    #[test]
    fn every_shape_has_its_size() {
        // Rectangles without width and height (and circles without r)
        // aren't drawn; an earlier copy of the icons lost them.
        for (name, svg) in ICONS {
            for rect in svg.split("<rect").skip(1) {
                let rect = &rect[..rect.find('>').unwrap()];
                assert!(
                    rect.contains("width=") && rect.contains("height="),
                    "{name}: <rect{rect}>"
                );
            }
            for circle in svg.split("<circle").skip(1) {
                assert!(
                    circle[..circle.find('>').unwrap()].contains(" r="),
                    "{name}"
                );
            }
        }
    }

    #[test]
    fn every_icon_is_an_svg() {
        assert!(ICONS.len() >= 50);
        for (name, svg) in ICONS {
            assert!(
                svg.starts_with("<svg class=\"size-4 shrink-0\" aria-hidden=\"true\""),
                "{name}"
            );
            assert!(svg.trim_end().ends_with("</svg>"), "{name}");
        }
    }

    #[test]
    fn finds_icons_by_name() {
        assert!(svg("book-open").is_some());
        assert!(svg("book-closed").is_none());
    }
}

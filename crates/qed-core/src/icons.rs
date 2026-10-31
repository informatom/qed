//! Icons for pages, text and menus: the [Tabler icons](https://tabler.io/icons)
//! (MIT license, #149), about 5,000 outline and 1,000 filled ones, built
//! into q.e.d. so generated sites get plain SVG and need no JavaScript.
//! Their shapes are in `assets/tabler/`, one line per icon, made from the
//! npm package by `npm run build:icons` (see the README there). A filled
//! icon's name ends in `-filled`: `heart-filled`.
//!
//! A site can add its own icons, such as the logos of the tools it talks
//! about, as image files in `media/icons/` ([`SiteIcons`]).

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::OnceLock;

const OUTLINE: &str = include_str!("../assets/tabler/outline.txt");
const FILLED: &str = include_str!("../assets/tabler/filled.txt");
const TAGS: &str = include_str!("../assets/tabler/tags.txt");

/// Before #149 the icons were Lucide's. Sites keep the names they used
/// then; these are the ones Tabler calls differently, with the Tabler icon
/// each stands for now. (The other Lucide names are Tabler's too.)
pub const RENAMED: &[(&str, &str)] = &[
    ("book-open", "book"),
    ("chart-column", "chart-bar"),
    ("circle-question-mark", "help-circle"),
    ("component", "components"),
    ("folder-input", "folder-down"),
    ("globe", "world"),
    ("graduation-cap", "school"),
    ("handshake", "heart-handshake"),
    ("house", "home"),
    ("image", "photo"),
    ("info", "info-circle"),
    ("layers", "stack-2"),
    ("life-buoy", "lifebuoy"),
    ("lightbulb", "bulb"),
    ("megaphone", "speakerphone"),
    ("mic", "microphone"),
    ("newspaper", "news"),
    ("pen-line", "pencil"),
    ("play", "player-play"),
    ("podcast", "broadcast"),
    ("save", "device-floppy"),
    ("undo", "arrow-back-up"),
    ("wrench", "tool"),
];

/// Every icon's SVG markup by name, made the first time it's needed.
fn icons() -> &'static BTreeMap<String, String> {
    static ICONS: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    ICONS.get_or_init(|| {
        let mut icons = BTreeMap::new();
        for (file, filled) in [(OUTLINE, false), (FILLED, true)] {
            for line in file.lines() {
                let Some((name, inner)) = line.split_once('\t') else {
                    continue;
                };
                let (name, paint) = if filled {
                    (format!("{name}-filled"), r#"fill="currentColor" stroke="none""#)
                } else {
                    (
                        name.to_string(),
                        r#"fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round""#,
                    )
                };
                let svg = format!(
                    r#"<svg class="size-4 shrink-0" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" {paint}>{inner}</svg>"#
                );
                icons.insert(name, svg);
            }
        }
        icons
    })
}

/// Tabler's search words for each outline icon: its category and tags.
fn tags() -> &'static HashMap<&'static str, &'static str> {
    static TAGS_BY_NAME: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    TAGS_BY_NAME.get_or_init(|| {
        TAGS.lines()
            .filter_map(|line| line.split_once('\t'))
            .collect()
    })
}

/// The name an icon has now: Tabler's for an old Lucide name, else as it is.
pub fn current_name(name: &str) -> &str {
    RENAMED
        .iter()
        .find(|(old, _)| *old == name)
        .map_or(name, |(_, new)| new)
}

/// The SVG for an icon name, if q.e.d. has that icon. Old Lucide names
/// (see [`RENAMED`]) still work.
pub fn svg(name: &str) -> Option<&'static str> {
    icons().get(current_name(name)).map(String::as_str)
}

/// Every icon's name, sorted.
pub fn names() -> impl Iterator<Item = &'static str> {
    icons().keys().map(String::as_str)
}

/// How many icons there are.
pub fn count() -> usize {
    icons().len()
}

/// Icon names for a search, best first, at most `limit`: names that are
/// the word, then those starting with it, containing it, and last those
/// whose search words have it ("envelope" finds `mail`). Every word of the
/// search must match. An empty search gives the first `limit` names.
pub fn search(query: &str, limit: usize) -> Vec<&'static str> {
    let words: Vec<String> = query
        .split_whitespace()
        .map(|word| word.to_lowercase())
        .collect();
    let mut found: Vec<(u8, &'static str)> = names()
        .filter_map(|name| {
            let base = name.strip_suffix("-filled").unwrap_or(name);
            let tags = tags().get(base).copied().unwrap_or_default();
            let mut score = 0;
            for word in &words {
                let rank = if name == word || base == word {
                    0
                } else if name.starts_with(word.as_str()) {
                    1
                } else if name.contains(word.as_str()) {
                    2
                } else if tags.split(' ').any(|tag| tag.starts_with(word.as_str())) {
                    3
                } else {
                    return None;
                };
                score = score.max(rank);
            }
            Some((score, name))
        })
        .collect();
    found.sort_by_key(|(score, name)| (*score, name.ends_with("-filled"), name.len(), *name));
    found
        .into_iter()
        .take(limit)
        .map(|(_, name)| name)
        .collect()
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
    fn has_the_outline_and_filled_icons() {
        assert!(count() > 6000, "{}", count());
        let home = svg("home").unwrap();
        assert!(
            home.starts_with(r#"<svg class="size-4 shrink-0" aria-hidden="true""#)
                && home.contains(r#"stroke="currentColor""#)
                && home.contains("<path d=")
                && home.ends_with("</svg>"),
            "{home}"
        );
        assert!(
            svg("heart-filled")
                .unwrap()
                .contains(r#"fill="currentColor""#)
        );
        assert!(svg("book-closed-nonsense").is_none());
    }

    #[test]
    fn old_lucide_names_still_work() {
        for (old, new) in RENAMED {
            assert!(svg(new).is_some(), "{old} → {new}");
            assert_eq!(svg(old), svg(new));
        }
        // Names Lucide and Tabler share need no entry.
        assert!(svg("mail").is_some() && svg("house").is_some());
        assert_eq!(current_name("house"), "home");
        assert_eq!(current_name("mail"), "mail");
    }

    #[test]
    fn searches_names_then_search_words() {
        let found = search("mail", 5);
        assert_eq!(found[0], "mail");
        assert!(search("envelope", 50).contains(&"mail"));
        assert!(search("mastodon", 5).contains(&"brand-mastodon"));
        assert!(search("heart filled", 5).contains(&"heart-filled"));
        assert_eq!(search("", 3).len(), 3);
        assert!(search("qqqzzz", 5).is_empty());
    }
}

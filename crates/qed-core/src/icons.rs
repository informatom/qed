//! Icons for pages: a curated set of [Lucide](https://lucide.dev) icons
//! (ISC license), built into qed so generated sites get plain SVG and need
//! no JavaScript. The files are in `assets/icons/`.

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
    "download",
    "file-text",
    "folder",
    "gift",
    "globe",
    "graduation-cap",
    "handshake",
    "headphones",
    "heart",
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
    "podcast",
    "rocket",
    "rss",
    "scale",
    "search",
    "settings",
    "shield",
    "shopping-cart",
    "sparkles",
    "star",
    "sun",
    "tag",
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

#[cfg(test)]
mod tests {
    use super::*;

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

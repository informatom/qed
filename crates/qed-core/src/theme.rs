//! The daisyUI themes a site can use.
//!
//! The site stylesheet (`assets/site.css`) is compiled with all of them, so
//! any of these names works in `site.toml`. A test checks that the list and
//! the stylesheet agree.

pub const THEMES: [&str; 35] = [
    "light",
    "dark",
    "cupcake",
    "bumblebee",
    "emerald",
    "corporate",
    "synthwave",
    "retro",
    "cyberpunk",
    "valentine",
    "halloween",
    "garden",
    "forest",
    "aqua",
    "lofi",
    "pastel",
    "fantasy",
    "wireframe",
    "black",
    "luxury",
    "dracula",
    "cmyk",
    "autumn",
    "business",
    "acid",
    "lemonade",
    "night",
    "coffee",
    "winter",
    "dim",
    "nord",
    "sunset",
    "caramellatte",
    "abyss",
    "silk",
];

pub fn is_theme(name: &str) -> bool {
    THEMES.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_themes_in_the_site_stylesheet() {
        let css = include_str!("../assets/site.css");
        for theme in THEMES {
            assert!(
                css.contains(&format!("[data-theme={theme}]")),
                "{theme} is missing from site.css"
            );
        }
    }
}

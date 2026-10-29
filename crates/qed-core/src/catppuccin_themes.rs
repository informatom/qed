//! The four Catppuccin flavors as daisyUI themes: Latte (light), Frappé,
//! Macchiato and Mocha (dark). Sites and the app can use them like
//! daisyUI's own themes; their names are in `theme::THEMES`.
//!
//! The colors come from the `catppuccin` crate's palette. `css()` turns them
//! into daisyUI themes, matched up the way Catppuccin's own daisyUI port
//! (`@catppuccin/daisyui`) does it. The result is `assets/catppuccin.css`,
//! which the site stylesheet (`assets/site.input.css`) and the app's
//! (`src/app.css`) both import; a test below keeps that file current.

use catppuccin::{ColorName, FlavorName, PALETTE};

/// The theme names, as `data-theme` uses them.
pub const CATPPUCCIN_THEMES: [&str; 4] = [
    "catppuccin-latte",
    "catppuccin-frappe",
    "catppuccin-macchiato",
    "catppuccin-mocha",
];

/// The flavors behind `CATPPUCCIN_THEMES`, in the same order.
const FLAVORS: [FlavorName; 4] = [
    FlavorName::Latte,
    FlavorName::Frappe,
    FlavorName::Macchiato,
    FlavorName::Mocha,
];

/// daisyUI color variable → Catppuccin color.
const COLORS: [(&str, ColorName); 20] = [
    ("primary", ColorName::Lavender),
    ("primary-content", ColorName::Mantle),
    ("secondary", ColorName::Surface0),
    ("secondary-content", ColorName::Text),
    ("accent", ColorName::Rosewater),
    ("accent-content", ColorName::Mantle),
    ("neutral", ColorName::Overlay1),
    ("neutral-content", ColorName::Mantle),
    ("success", ColorName::Green),
    ("success-content", ColorName::Base),
    ("warning", ColorName::Yellow),
    ("warning-content", ColorName::Base),
    ("error", ColorName::Red),
    ("error-content", ColorName::Base),
    ("info", ColorName::Sky),
    ("info-content", ColorName::Mantle),
    ("base-100", ColorName::Base),
    ("base-200", ColorName::Mantle),
    ("base-300", ColorName::Crust),
    ("base-content", ColorName::Text),
];

/// Sizes and corners, also as in Catppuccin's daisyUI port.
const SHAPE: &str = "--radius-selector: 0.25rem; --radius-field: 0.5rem; --radius-box: 0.5rem; \
    --size-selector: 0.25rem; --size-field: 0.25rem; --border: 1px; --depth: 1; --noise: 0;";

/// The stylesheet for `CATPPUCCIN_THEMES`: one `[data-theme=…]` rule each,
/// the same shape daisyUI gives its built-in themes (`theme::color_hex`
/// reads colors from it the same way).
pub fn css() -> String {
    let mut css = String::from(
        "/* Generated from the catppuccin crate by qed-core (catppuccin_themes.rs). Don't edit. */\n",
    );
    for (name, flavor) in CATPPUCCIN_THEMES.iter().zip(FLAVORS) {
        let flavor = PALETTE.get_flavor(flavor);
        let scheme = if flavor.dark { "dark" } else { "light" };
        css.push_str(&format!(
            "[data-theme={name}] {{\n  color-scheme: {scheme};\n  {SHAPE}\n"
        ));
        for (variable, color) in COLORS {
            let hex = flavor.get_color(color).hex;
            css.push_str(&format!("  --color-{variable}: {hex};\n"));
        }
        css.push_str("}\n");
    }
    css
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_follow_the_flavors() {
        for (name, flavor) in CATPPUCCIN_THEMES.iter().zip(FLAVORS) {
            assert_eq!(*name, format!("catppuccin-{}", flavor.identifier()));
        }
    }

    #[test]
    fn only_latte_is_light() {
        let css = css();
        assert!(css.contains("[data-theme=catppuccin-latte] {\n  color-scheme: light;"));
        for dark in &CATPPUCCIN_THEMES[1..] {
            assert!(css.contains(&format!("[data-theme={dark}] {{\n  color-scheme: dark;")));
        }
        // Mocha's base, the dark background.
        assert!(css.contains("--color-base-100: #1e1e2e;"));
    }

    /// Rewrites assets/catppuccin.css and fails if that changed it, like
    /// the bindings test in src-tauri: run the tests again, rebuild the
    /// site stylesheet (`npm run build:site-css`) and commit both files.
    #[test]
    fn stylesheet_is_current() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/catppuccin.css");
        let before = std::fs::read_to_string(&path).unwrap_or_default();
        let after = css();
        if before != after {
            std::fs::write(&path, &after).unwrap();
        }
        assert!(
            before == after,
            "assets/catppuccin.css was out of date and has been regenerated"
        );
    }
}

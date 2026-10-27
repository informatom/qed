//! The daisyUI themes a site can use, and custom colors on top of one.
//!
//! The site stylesheet (`assets/site.css`) is compiled with all of them, so
//! any of these names works in `site.toml`. A test checks that the list and
//! the stylesheet agree.
//!
//! A daisyUI theme is a set of CSS variables (`--color-primary`, …). A
//! custom theme starts from one of the themes and sets a few of them again:
//! the five colors q.e.d. sites use and the corners. The matching text
//! colors are worked out for contrast, so the site stays readable.

use serde::{Deserialize, Serialize};

/// `theme = "custom"` in `site.toml` uses `[custom_theme]`.
pub const CUSTOM: &str = "custom";

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

/// Colors and corners on top of a theme, from `[custom_theme]` in
/// `site.toml`. Colors are `#rrggbb` (or `#rgb`); empty ones keep the
/// starting theme's color.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct CustomTheme {
    /// The name it's saved under, for a saved set (`saved_themes`), or the
    /// saved set the site's colors came from.
    pub name: Option<String>,
    /// The theme it starts from; everything not set here comes from it.
    pub from: String,
    /// The page background (daisyUI's `base-100`).
    pub background: Option<String>,
    pub primary: Option<String>,
    pub secondary: Option<String>,
    pub accent: Option<String>,
    pub neutral: Option<String>,
    pub corners: Option<Corners>,
}

/// How round boxes, fields and buttons are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum Corners {
    Sharp,
    Soft,
    Round,
}

/// One CSS variable a custom theme sets.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ThemeVariable {
    /// With the dashes: `--color-primary`.
    pub name: String,
    pub value: String,
}

/// Why a custom theme can't be used.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ThemeError {
    #[error("there is no theme called \"{0}\" to start from")]
    UnknownBase(String),
    #[error("{field}: \"{value}\" isn't a color like #ff6600")]
    BadColor { field: &'static str, value: String },
}

impl CustomTheme {
    /// Checks the starting theme and the colors.
    pub fn check(&self) -> Result<(), ThemeError> {
        if !is_theme(&self.from) {
            return Err(ThemeError::UnknownBase(self.from.clone()));
        }
        for (field, value) in self.colors() {
            if let Some(value) = value
                && parse_hex(value).is_none()
            {
                return Err(ThemeError::BadColor {
                    field,
                    value: value.to_string(),
                });
            }
        }
        Ok(())
    }

    fn colors(&self) -> [(&'static str, Option<&str>); 5] {
        [
            ("background", self.background.as_deref()),
            ("primary", self.primary.as_deref()),
            ("secondary", self.secondary.as_deref()),
            ("accent", self.accent.as_deref()),
            ("neutral", self.neutral.as_deref()),
        ]
    }

    /// The CSS variables to set on top of the starting theme. Colors that
    /// aren't valid are left out.
    pub fn variables(&self) -> Vec<ThemeVariable> {
        let mut vars = Vec::new();
        let mut set = |name: &str, value: String| {
            vars.push(ThemeVariable {
                name: name.to_string(),
                value,
            })
        };
        for (field, value) in self.colors() {
            let Some((hex, rgb)) = value.and_then(|v| parse_hex(v).map(|rgb| (v.trim(), rgb)))
            else {
                continue;
            };
            let content = readable_on(rgb).to_string();
            if field == "background" {
                // The darker shades for boxes and borders; lighter on a dark
                // background.
                let shade = if is_light(rgb) { "black" } else { "white" };
                set("--color-base-100", hex.to_string());
                set(
                    "--color-base-200",
                    format!("color-mix(in oklab, {hex}, {shade} 5%)"),
                );
                set(
                    "--color-base-300",
                    format!("color-mix(in oklab, {hex}, {shade} 12%)"),
                );
                set("--color-base-content", content);
            } else {
                set(&format!("--color-{field}"), hex.to_string());
                set(&format!("--color-{field}-content"), content);
            }
        }
        if let Some(corners) = self.corners {
            let (selector, field, boxes) = match corners {
                Corners::Sharp => ("0", "0", "0"),
                Corners::Soft => ("0.25rem", "0.25rem", "0.5rem"),
                Corners::Round => ("1rem", "2rem", "1.5rem"),
            };
            set("--radius-selector", selector.to_string());
            set("--radius-field", field.to_string());
            set("--radius-box", boxes.to_string());
        }
        vars
    }

    /// A rule for the site's stylesheet. Pages keep `data-theme` of the
    /// starting theme; `:root` makes this rule win over that theme's own.
    pub fn css(&self) -> String {
        let body: String = self
            .variables()
            .iter()
            .map(|var| format!("  {}: {};\n", var.name, var.value))
            .collect();
        format!(
            "\n/* Custom theme, based on {} */\n:root[data-theme={}] {{\n{body}}}\n",
            self.from, self.from
        )
    }
}

/// `#rgb` or `#rrggbb` as red, green, blue.
fn parse_hex(value: &str) -> Option<[u8; 3]> {
    let hex = value.trim().strip_prefix('#')?;
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let digits: Vec<u8> = match hex.len() {
        3 => hex.chars().flat_map(|c| [c, c]).map(|c| c as u8).collect(),
        6 => hex.bytes().collect(),
        _ => return None,
    };
    let byte = |i: usize| u8::from_str_radix(std::str::from_utf8(&digits[i..i + 2]).ok()?, 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?])
}

/// Relative luminance after WCAG: 0 for black, 1 for white.
fn luminance([r, g, b]: [u8; 3]) -> f64 {
    let channel = |c: u8| {
        let c = f64::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

/// Whether dark text reads better on this color than white text.
fn is_light(rgb: [u8; 3]) -> bool {
    let l = luminance(rgb);
    // Contrast with black (l + 0.05) / 0.05 against white 1.05 / (l + 0.05).
    (l + 0.05) / 0.05 > 1.05 / (l + 0.05)
}

/// Near-black or white, whichever contrasts more with the color.
fn readable_on(rgb: [u8; 3]) -> &'static str {
    if is_light(rgb) { "#1d232a" } else { "#ffffff" }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn custom() -> CustomTheme {
        CustomTheme {
            name: None,
            from: "nord".into(),
            background: None,
            primary: Some("#ff6600".into()),
            secondary: None,
            accent: Some("#036".into()),
            neutral: None,
            corners: Some(Corners::Sharp),
        }
    }

    #[test]
    fn sets_colors_with_readable_text_on_them() {
        let vars = custom().variables();
        let value = |name: &str| {
            vars.iter()
                .find(|v| v.name == name)
                .map(|v| v.value.as_str())
        };
        assert_eq!(value("--color-primary"), Some("#ff6600"));
        assert_eq!(
            value("--color-primary-content"),
            Some("#1d232a"),
            "dark on orange"
        );
        assert_eq!(
            value("--color-accent-content"),
            Some("#ffffff"),
            "white on dark blue"
        );
        assert_eq!(value("--color-secondary"), None, "kept from nord");
        assert_eq!(value("--radius-box"), Some("0"));
        assert!(custom().css().contains(":root[data-theme=nord] {"));
    }

    #[test]
    fn derives_shades_from_the_background() {
        let mut theme = custom();
        theme.background = Some("#101418".into());
        let vars = theme.variables();
        let base = |n: &str| vars.iter().find(|v| v.name == n).unwrap().value.clone();
        assert_eq!(base("--color-base-content"), "#ffffff");
        assert!(
            base("--color-base-200").contains("white"),
            "lighter on dark"
        );
    }

    #[test]
    fn checks_colors_and_the_starting_theme() {
        assert_eq!(custom().check(), Ok(()));
        let mut theme = custom();
        theme.primary = Some("orange".into());
        assert!(matches!(
            theme.check(),
            Err(ThemeError::BadColor {
                field: "primary",
                ..
            })
        ));
        theme = custom();
        theme.from = "custom".into();
        assert!(theme.check().is_err());
    }

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

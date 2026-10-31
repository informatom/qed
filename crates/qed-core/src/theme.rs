//! The daisyUI themes a site can use, and custom colors on top of one.
//! Besides daisyUI's own, there are the four Catppuccin flavors
//! (`catppuccin_themes`).
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

pub const THEMES: [&str; 39] = [
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
    // Kept in step with `catppuccin_themes::CATPPUCCIN_THEMES` (a test checks).
    "catppuccin-latte",
    "catppuccin-frappe",
    "catppuccin-macchiato",
    "catppuccin-mocha",
];

pub fn is_theme(name: &str) -> bool {
    THEMES.contains(&name)
}

/// A theme as the app offers it: light ones and dark ones apart.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ThemeChoice {
    pub name: &'static str,
    /// Whether its page background is dark (see [`is_dark`]).
    pub dark: bool,
}

/// Every theme by name, A to Z, each marked light or dark.
pub fn choices() -> Vec<ThemeChoice> {
    let mut choices: Vec<ThemeChoice> = THEMES
        .iter()
        .map(|&name| ThemeChoice {
            name,
            dark: is_dark(crate::generator::STYLESHEET, name, None),
        })
        .collect();
    choices.sort_by_key(|choice| choice.name);
    choices
}

/// Whether the page background (`--color-base-100`) is dark: light text
/// reads better on it than dark text. For a custom theme its own
/// background wins over its starting theme's.
pub fn is_dark(stylesheet: &str, theme: &str, custom: Option<&CustomTheme>) -> bool {
    color_hex(stylesheet, theme, custom, "--color-base-100")
        .and_then(|hex| parse_hex(&hex))
        .is_some_and(|rgb| !is_light(rgb))
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

/// A theme color as `#rrggbb`, for code that can't use the stylesheet's
/// variables, such as the Podlove Subscribe Button (it gets its color as
/// text and works out its hover shade itself). `variable` is e.g.
/// `--color-primary`; a custom theme's own color wins over its starting
/// theme's, which comes from `stylesheet`, daisyUI's CSS.
pub fn color_hex(
    stylesheet: &str,
    theme: &str,
    custom: Option<&CustomTheme>,
    variable: &str,
) -> Option<String> {
    if let Some(value) = custom.and_then(|custom| {
        custom
            .variables()
            .into_iter()
            .find(|var| var.name == variable)
            .map(|var| var.value)
    }) && let Some([r, g, b]) = parse_hex(&value)
    {
        return Some(format!("#{r:02x}{g:02x}{b:02x}"));
    }
    let value = theme_rule(stylesheet, theme)?
        .split(';')
        .find_map(|declaration| {
            let (name, value) = declaration.split_once(':')?;
            (name.trim() == variable).then(|| value.trim())
        })?;
    if let Some([r, g, b]) = parse_hex(value) {
        return Some(format!("#{r:02x}{g:02x}{b:02x}"));
    }
    let [r, g, b] = oklch_to_rgb(value)?;
    Some(format!("#{r:02x}{g:02x}{b:02x}"))
}

/// What a theme's rule in daisyUI's CSS sets, between the braces:
/// `color-scheme:light;--color-primary:oklch(…);…` from
/// `…,[data-theme=nord]{…}`.
fn theme_rule<'a>(stylesheet: &'a str, theme: &str) -> Option<&'a str> {
    let selector = format!("[data-theme={theme}]");
    let start = stylesheet.find(&selector)?;
    let open = start + stylesheet[start..].find('{')? + 1;
    let close = open + stylesheet[open..].find('}')?;
    Some(stylesheet[open..close].trim())
}

/// The rules that give a site with dark mode (#146) its dark side: the
/// dark theme's variables, and its custom colors on top, on the page root
/// while the system is set to dark (unless the visitor picked light) or
/// the visitor picked dark. Pages keep the light theme in `data-theme`;
/// these come later in the stylesheet, so they win over it.
pub fn dark_css(stylesheet: &str, theme: &str, custom: Option<&CustomTheme>) -> String {
    let mut body: Vec<String> = theme_rule(stylesheet, theme)
        .unwrap_or_default()
        .split(';')
        .map(str::trim)
        .filter(|declaration| !declaration.is_empty())
        .map(str::to_string)
        .collect();
    if let Some(custom) = custom {
        body.extend(
            custom
                .variables()
                .into_iter()
                .map(|var| format!("{}: {}", var.name, var.value)),
        );
    }
    let body = body.join(";\n  ");
    format!(
        "\n/* Dark mode, based on {theme} */\n\
         @media (prefers-color-scheme: dark) {{\n\
         :root:not([data-mode=light]) {{\n  {body};\n}}\n}}\n\
         :root[data-mode=dark] {{\n  {body};\n}}\n"
    )
}

/// `hex` mixed with `other` (`#rrggbb` both), `amount` of `other` from 0
/// to 1: shades of a theme color for code that needs several, like the
/// Podlove Web Player. `None` if either isn't a hex color.
pub fn mix_hex(hex: &str, other: &str, amount: f64) -> Option<String> {
    let (a, b) = (parse_hex(hex)?, parse_hex(other)?);
    let mixed: Vec<String> = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let value = f64::from(*x) * (1.0 - amount) + f64::from(*y) * amount;
            format!("{:02x}", value.round().clamp(0.0, 255.0) as u8)
        })
        .collect();
    Some(format!("#{}", mixed.concat()))
}

/// `oklch(45% .24 277.023)` (lightness, chroma, hue in degrees) as sRGB.
/// The formulas are Björn Ottosson's, who made OKLab:
/// <https://bottosson.github.io/posts/oklab/>.
fn oklch_to_rgb(value: &str) -> Option<[u8; 3]> {
    let inner = value.trim().strip_prefix("oklch(")?.strip_suffix(')')?;
    let parts: Vec<&str> = inner.split_whitespace().collect();
    let [lightness, chroma, hue] = parts[..] else {
        return None;
    };
    let lightness = match lightness.strip_suffix('%') {
        Some(percent) => percent.parse::<f64>().ok()? / 100.0,
        None => lightness.parse::<f64>().ok()?,
    };
    let chroma: f64 = chroma.parse().ok()?;
    let hue = hue
        .trim_end_matches("deg")
        .parse::<f64>()
        .ok()?
        .to_radians();
    let (a, b) = (chroma * hue.cos(), chroma * hue.sin());

    // OKLab to linear sRGB.
    let l = (lightness + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
    let m = (lightness - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
    let s = (lightness - 0.089_484_177_5 * a - 1.291_485_548_0 * b).powi(3);
    let linear = [
        4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
        -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
        -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701_0 * s,
    ];
    // Linear light to the sRGB curve, then to 0–255.
    Some(linear.map(|channel| {
        let channel = channel.clamp(0.0, 1.0);
        let curved = if channel <= 0.003_130_8 {
            12.92 * channel
        } else {
            1.055 * channel.powf(1.0 / 2.4) - 0.055
        };
        (curved * 255.0).round() as u8
    }))
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

    #[test]
    fn tells_dark_themes_from_light_ones_as_daisyui_does() {
        let stylesheet = crate::generator::STYLESHEET;
        for choice in choices() {
            // Each theme says it itself: `[data-theme=nord]{color-scheme:light;…`.
            let rule = &stylesheet[stylesheet
                .find(&format!("[data-theme={}]{{", choice.name))
                .unwrap()..];
            let scheme_dark = rule[..rule.find('}').unwrap()].contains("color-scheme:dark");
            assert_eq!(choice.dark, scheme_dark, "{}", choice.name);
        }
        assert!(
            choices()
                .iter()
                .any(|c| c.name == "catppuccin-mocha" && c.dark)
        );
        let names: Vec<&str> = choices().iter().map(|c| c.name).collect();
        assert!(names.is_sorted(), "{names:?}");
        assert!(
            choices()
                .iter()
                .any(|c| c.name == "catppuccin-latte" && !c.dark)
        );

        let mut custom = CustomTheme {
            name: None,
            from: "light".into(),
            background: None,
            primary: Some("#ff0000".into()),
            secondary: None,
            accent: None,
            neutral: None,
            corners: None,
        };
        assert!(!is_dark(stylesheet, "light", Some(&custom)));
        custom.background = Some("#1e1e2e".into());
        assert!(
            is_dark(stylesheet, "light", Some(&custom)),
            "its own background wins"
        );
    }

    #[test]
    fn turns_theme_colors_into_hex() {
        assert_eq!(oklch_to_rgb("oklch(100% 0 0)"), Some([255, 255, 255]));
        assert_eq!(oklch_to_rgb("oklch(0% 0 0)"), Some([0, 0, 0]));
        // Pure red, as OKLCH.
        assert_eq!(oklch_to_rgb("oklch(62.8% 0.2577 29.23)"), Some([255, 0, 0]));
        assert_eq!(oklch_to_rgb("rgb(1 2 3)"), None);

        let css = ":root,[data-theme=light]{--color-primary:oklch(0% 0 0);--color-secondary:#ff6600}\
                   [data-theme=dark]{--color-primary:oklch(100% 0 0)}";
        assert_eq!(
            color_hex(css, "light", None, "--color-primary").as_deref(),
            Some("#000000")
        );
        assert_eq!(
            color_hex(css, "light", None, "--color-secondary").as_deref(),
            Some("#ff6600")
        );
        assert_eq!(
            color_hex(css, "dark", None, "--color-primary").as_deref(),
            Some("#ffffff")
        );
        let custom = CustomTheme {
            name: None,
            from: "light".to_string(),
            background: None,
            primary: Some("#123456".to_string()),
            secondary: None,
            accent: None,
            neutral: None,
            corners: None,
        };
        assert_eq!(
            color_hex(css, "light", Some(&custom), "--color-primary").as_deref(),
            Some("#123456")
        );
        assert_eq!(
            mix_hex("#ff0000", "#000000", 0.5).as_deref(),
            Some("#800000")
        );
        assert_eq!(
            mix_hex("#ff0000", "#ffffff", 1.0).as_deref(),
            Some("#ffffff")
        );
        assert_eq!(mix_hex("red", "#ffffff", 0.5), None);
        // The real stylesheet has every theme.
        let site_css = include_str!("../assets/site.css");
        assert!(color_hex(site_css, "nord", None, "--color-primary").is_some());
    }

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

    #[test]
    fn includes_the_catppuccin_flavors() {
        for theme in crate::catppuccin_themes::CATPPUCCIN_THEMES {
            assert!(is_theme(theme), "{theme} is missing from THEMES");
        }
    }

    #[test]
    fn finds_catppuccin_colors_in_the_site_stylesheet() {
        let css = include_str!("../assets/site.css");
        let primary = color_hex(css, "catppuccin-mocha", None, "--color-primary");
        assert_eq!(primary.as_deref(), Some("#b4befe"));
    }
}

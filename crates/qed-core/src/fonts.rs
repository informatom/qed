//! Fonts for a site, self-hosted.
//!
//! The fonts come from [Fontsource](https://fontsource.org), which offers
//! the Google Fonts (and a few more open fonts) as plain `.woff2` files.
//! q.e.d. downloads the ones a site picks into `fonts/<id>/` in the site
//! folder, once; the built site serves them itself, so visitors' browsers
//! never ask Google or anyone else.
//!
//! ```text
//! fonts/ubuntu/
//!   font.toml                  family, category, unicode ranges
//!   latin-400-normal.woff2     subset-weight-style
//!   latin-ext-700-normal.woff2
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::site::SiteError;
use crate::slug::slugify;

pub const FONTS_DIR: &str = "fonts";
const FONT_FILE: &str = "font.toml";
const API: &str = "https://api.fontsource.org/v1/fonts";

/// The fonts a site uses, from `[fonts]` in `site.toml`, by family name.
/// Empty ones keep the theme's own (the system's sans-serif).
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Fonts {
    /// For text, menus and everything else.
    pub text: Option<String>,
    /// For headings; the text font if empty.
    pub headings: Option<String>,
}

/// One font in Fontsource's list.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct FontListing {
    pub id: String,
    pub family: String,
    /// `sans-serif`, `serif`, `display`, `handwriting` or `monospace`.
    pub category: String,
    /// `google` for Google Fonts, `other` for the rest.
    #[serde(rename = "type")]
    pub source: String,
}

/// What `fonts/<id>/font.toml` keeps about a downloaded font.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct FontInfo {
    pub id: String,
    pub family: String,
    pub category: String,
    pub license: String,
    /// Subset name to its unicode range, for `@font-face`.
    pub ranges: BTreeMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("couldn't reach Fontsource: {0}")]
    Network(String),
    #[error("Fontsource has no font called \"{0}\"")]
    Unknown(String),
    #[error("{family} has no files q.e.d. can use")]
    NoFiles { family: String },
    #[error(transparent)]
    Site(#[from] SiteError),
}

/// The folder name of a font: Fontsource's id, `open-sans` for "Open Sans".
pub fn font_id(family: &str) -> String {
    slugify(family)
}

fn client() -> Result<reqwest::Client, FontError> {
    // reqwest leaves the choice of crypto to the app; q.e.d. uses ring.
    let _ = rustls::crypto::ring::default_provider().install_default();
    reqwest::Client::builder()
        .user_agent(concat!("q.e.d./", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| FontError::Network(e.to_string()))
}

async fn get_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
) -> Result<T, FontError> {
    let network = |e: reqwest::Error| FontError::Network(e.to_string());
    client
        .get(url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(network)?
        .json()
        .await
        .map_err(network)
}

/// Every font Fontsource offers, sorted by family.
pub async fn list() -> Result<Vec<FontListing>, FontError> {
    let mut fonts: Vec<FontListing> = get_json(&client()?, API).await?;
    fonts.sort_by_key(|font| font.family.to_lowercase());
    Ok(fonts)
}

/// Fontsource's description of one font.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Details {
    id: String,
    family: String,
    category: String,
    license: String,
    subsets: Vec<String>,
    def_subset: String,
    weights: Vec<u16>,
    styles: Vec<String>,
    unicode_range: BTreeMap<String, String>,
    /// weight → style → subset → urls by format.
    variants: BTreeMap<String, BTreeMap<String, BTreeMap<String, Urls>>>,
}

#[derive(Deserialize)]
struct Urls {
    url: BTreeMap<String, String>,
}

/// The files a site needs: regular, semibold and bold, italic text, for
/// the Latin subsets. Returns (subset, weight, style) to fetch.
fn wanted(details: &Details) -> Vec<(String, u16, String)> {
    let subsets: Vec<String> = ["latin", "latin-ext"]
        .into_iter()
        .filter(|s| details.subsets.iter().any(|have| have == s))
        .map(str::to_string)
        .collect();
    let subsets = if subsets.is_empty() {
        vec![details.def_subset.clone()]
    } else {
        subsets
    };
    // The nearest weight it has to each that pages use.
    let nearest = |target: u16| {
        details
            .weights
            .iter()
            .copied()
            .min_by_key(|w| (i32::from(*w) - i32::from(target)).abs())
    };
    let mut weights: Vec<u16> = [400, 700].into_iter().filter_map(nearest).collect();
    if details.weights.contains(&600) {
        weights.push(600);
    }
    weights.sort_unstable();
    weights.dedup();

    let mut files = Vec::new();
    for subset in &subsets {
        for weight in &weights {
            files.push((subset.clone(), *weight, "normal".to_string()));
        }
        if details.styles.iter().any(|s| s == "italic")
            && let Some(regular) = nearest(400)
        {
            files.push((subset.clone(), regular, "italic".to_string()));
        }
    }
    files
}

/// Downloads a font from Fontsource into `fonts/<id>/`, replacing an
/// earlier download of it. Returns what it keeps in `font.toml`.
pub async fn download(root: &Path, family: &str) -> Result<FontInfo, FontError> {
    let client = client()?;
    let id = font_id(family);
    let details: Details = get_json(&client, &format!("{API}/{id}"))
        .await
        .map_err(|error| match error {
            FontError::Network(message) if message.contains("404") => {
                FontError::Unknown(family.to_string())
            }
            other => other,
        })?;

    let folder = root.join(FONTS_DIR).join(&details.id);
    let write = |path: PathBuf| move |source| SiteError::Write { path, source };
    // Fetch everything first, so a failed download leaves the old files.
    let mut files = Vec::new();
    for (subset, weight, style) in wanted(&details) {
        let Some(url) = details
            .variants
            .get(&weight.to_string())
            .and_then(|styles| styles.get(&style))
            .and_then(|subsets| subsets.get(&subset))
            .and_then(|urls| urls.url.get("woff2"))
        else {
            continue;
        };
        let bytes = client
            .get(url)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|e| FontError::Network(e.to_string()))?
            .bytes()
            .await
            .map_err(|e| FontError::Network(e.to_string()))?;
        files.push((format!("{subset}-{weight}-{style}.woff2"), bytes));
    }
    if files.is_empty() {
        return Err(FontError::NoFiles {
            family: details.family,
        });
    }

    if folder.exists() {
        fs::remove_dir_all(&folder).map_err(write(folder.clone()))?;
    }
    fs::create_dir_all(&folder).map_err(write(folder.clone()))?;
    for (name, bytes) in &files {
        let path = folder.join(name);
        fs::write(&path, bytes).map_err(write(path.clone()))?;
    }
    let used: Vec<&str> = files
        .iter()
        .filter_map(|(name, _)| name.split('-').next())
        .collect();
    let info = FontInfo {
        id: details.id,
        family: details.family,
        category: details.category,
        license: details.license,
        ranges: details
            .unicode_range
            .into_iter()
            .filter(|(subset, _)| used.iter().any(|u| subset.starts_with(u)))
            .collect(),
    };
    let text = toml::to_string(&info).map_err(|e| SiteError::Invalid {
        path: folder.join(FONT_FILE),
        message: e.to_string(),
    })?;
    let path = folder.join(FONT_FILE);
    fs::write(&path, text).map_err(write(path.clone()))?;
    Ok(info)
}

/// A downloaded font's `font.toml`, or `None` if it isn't there.
pub fn installed(root: &Path, family: &str) -> Option<FontInfo> {
    let text =
        fs::read_to_string(root.join(FONTS_DIR).join(font_id(family)).join(FONT_FILE)).ok()?;
    toml::from_str(&text).ok()
}

/// One `.woff2` file: its name and what it holds.
struct Face {
    file: String,
    subset: String,
    weight: String,
    style: String,
}

fn faces(root: &Path, id: &str) -> Vec<Face> {
    let Ok(entries) = fs::read_dir(root.join(FONTS_DIR).join(id)) else {
        return Vec::new();
    };
    let mut faces: Vec<Face> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let file = entry.file_name().to_string_lossy().to_string();
            let stem = file.strip_suffix(".woff2")?;
            // `latin-ext-700-normal`: the subset may hold dashes itself.
            let (rest, style) = stem.rsplit_once('-')?;
            let (subset, weight) = rest.rsplit_once('-')?;
            Some(Face {
                subset: subset.to_string(),
                weight: weight.to_string(),
                style: style.to_string(),
                file,
            })
        })
        .collect();
    faces.sort_by(|a, b| a.file.cmp(&b.file));
    faces
}

/// A family name for CSS, with fallbacks that fit its category.
fn stack(info: &FontInfo) -> String {
    let fallback = match info.category.as_str() {
        "serif" => "ui-serif, Georgia, serif",
        "monospace" => "ui-monospace, monospace",
        "handwriting" => "cursive",
        _ => "ui-sans-serif, system-ui, sans-serif",
    };
    format!("\"{}\", {fallback}", info.family.replace('"', ""))
}

/// The fonts a site uses that are downloaded, text font first.
pub fn used(root: &Path, fonts: &Fonts) -> Vec<FontInfo> {
    let mut used: Vec<FontInfo> = Vec::new();
    for family in [&fonts.text, &fonts.headings].into_iter().flatten() {
        if let Some(info) = installed(root, family)
            && !used.iter().any(|u| u.id == info.id)
        {
            used.push(info);
        }
    }
    used
}

/// `@font-face` rules and the rules that use them, for the site's
/// stylesheet; the files are at `fonts/<id>/` next to it.
pub fn css(root: &Path, fonts: &Fonts) -> String {
    let mut css = String::new();
    for info in used(root, fonts) {
        css.push_str(&format!(
            "\n/* {} ({}), from Fontsource */\n",
            info.family, info.license
        ));
        for face in faces(root, &info.id) {
            let range = info
                .ranges
                .get(&face.subset)
                .map(|r| format!("unicode-range:{r};"))
                .unwrap_or_default();
            css.push_str(&format!(
                "@font-face{{font-family:\"{}\";font-style:{};font-weight:{};font-display:swap;src:url(fonts/{}/{}) format(\"woff2\");{range}}}\n",
                info.family.replace('"', ""),
                face.style,
                face.weight,
                info.id,
                face.file
            ));
        }
    }
    let text = fonts.text.as_deref().and_then(|f| installed(root, f));
    let headings = fonts
        .headings
        .as_deref()
        .and_then(|f| installed(root, f))
        .or_else(|| text.clone());
    if let Some(text) = &text {
        let stack = stack(text);
        css.push_str(&format!(
            ":root{{--font-sans:{stack};}}\nhtml{{font-family:{stack};}}\n"
        ));
    }
    if let Some(headings) = &headings {
        css.push_str(&format!(
            ":where(h1,h2,h3,h4,h5,h6,.card-title){{font-family:{};}}\n",
            stack(headings)
        ));
    }
    css
}

#[cfg(test)]
mod tests {
    use super::*;

    fn details(weights: &[u16], styles: &[&str]) -> Details {
        Details {
            id: "x".into(),
            family: "X".into(),
            category: "serif".into(),
            license: "OFL-1.1".into(),
            subsets: vec!["cyrillic".into(), "latin".into(), "latin-ext".into()],
            def_subset: "latin".into(),
            weights: weights.to_vec(),
            styles: styles.iter().map(|s| s.to_string()).collect(),
            unicode_range: BTreeMap::new(),
            variants: BTreeMap::new(),
        }
    }

    #[test]
    fn takes_the_weights_pages_use() {
        let files = wanted(&details(&[300, 400, 500, 700], &["normal", "italic"]));
        let latin: Vec<String> = files
            .iter()
            .filter(|(s, ..)| s == "latin")
            .map(|(_, w, st)| format!("{w}-{st}"))
            .collect();
        assert_eq!(latin, ["400-normal", "700-normal", "400-italic"]);
        assert_eq!(files.len(), 6, "latin and latin-ext, not cyrillic");

        let files = wanted(&details(&[100, 900], &["normal"]));
        let weights: Vec<u16> = files.iter().map(|(_, w, _)| *w).collect();
        assert_eq!(weights, [100, 900, 100, 900], "nearest to 400 and 700");
    }

    #[test]
    fn writes_font_faces_for_the_downloaded_files() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("fonts/ubuntu");
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("latin-400-normal.woff2"), b"x").unwrap();
        fs::write(folder.join("latin-ext-700-normal.woff2"), b"x").unwrap();
        let info = FontInfo {
            id: "ubuntu".into(),
            family: "Ubuntu".into(),
            category: "sans-serif".into(),
            license: "UFL-1.0".into(),
            ranges: BTreeMap::from([
                ("latin".into(), "U+0000-00FF".into()),
                ("latin-ext".into(), "U+0100-02BA".into()),
            ]),
        };
        fs::write(folder.join(FONT_FILE), toml::to_string(&info).unwrap()).unwrap();

        let fonts = Fonts {
            text: Some("Ubuntu".into()),
            headings: None,
        };
        let css = css(dir.path(), &fonts);
        assert!(css.contains("src:url(fonts/ubuntu/latin-ext-700-normal.woff2) format(\"woff2\");unicode-range:U+0100-02BA;"), "{css}");
        assert!(css.contains("font-weight:400"), "{css}");
        assert!(
            css.contains("html{font-family:\"Ubuntu\", ui-sans-serif"),
            "{css}"
        );
        assert!(
            css.contains(":where(h1,h2,h3,h4,h5,h6,.card-title){font-family:\"Ubuntu\""),
            "headings follow text: {css}"
        );

        let missing = Fonts {
            text: Some("Not There".into()),
            headings: None,
        };
        assert_eq!(super::css(dir.path(), &missing), "");
    }
}

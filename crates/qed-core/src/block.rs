//! The building blocks of a page.
//!
//! In a page file, plain Markdown is a [`Block::Text`]. Every other block is a
//! container whose fields are written as TOML:
//!
//! ```text
//! ::: hero
//! headline = "Welcome"
//! image = "team.jpg"
//! :::
//! ```

use serde::{Deserialize, Serialize};

/// One block of a page, in the order it appears in the file.
///
/// Serialized for the UI with a `type` field, e.g.
/// `{ "type": "hero", "headline": "Welcome", … }`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Block {
    /// Markdown text between other blocks.
    Text {
        markdown: String,
    },
    Hero(Hero),
    Image(Image),
    TextImage(TextImage),
    Gallery(Gallery),
    Download(Download),
    Faq(Faq),
    Columns(Columns),
    CallToAction(CallToAction),
}

/// A large header: optional background image, headline, text and a button.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Hero {
    /// File name in the site's `media/` folder.
    pub image: Option<String>,
    pub headline: Option<String>,
    /// Markdown.
    pub text: Option<String>,
    pub button_label: Option<String>,
    pub button_link: Option<String>,
}

/// One image, with alternative text and an optional caption.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    /// File name in the site's `media/` folder.
    pub image: Option<String>,
    /// Describes the image for people who can't see it.
    pub alt: Option<String>,
    /// Markdown.
    pub caption: Option<String>,
}

/// Text next to an image.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TextImage {
    /// Markdown.
    pub text: Option<String>,
    /// File name in `media/`.
    pub image: Option<String>,
    pub alt: Option<String>,
    #[serde(default)]
    pub image_side: Side,
}

/// Which side an image goes on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Left,
    #[default]
    Right,
}

/// Several images in a grid; each links to its full size.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Gallery {
    #[serde(default)]
    pub images: Vec<GalleryImage>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GalleryImage {
    /// File name in `media/`.
    pub image: String,
    pub alt: Option<String>,
}

/// A file to download, such as a PDF.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Download {
    /// File name in `media/`.
    pub file: Option<String>,
    /// The link text; the file name if empty.
    pub label: Option<String>,
}

/// Questions and answers that open on click.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Faq {
    pub heading: Option<String>,
    #[serde(default)]
    pub items: Vec<FaqItem>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FaqItem {
    #[serde(default)]
    pub question: String,
    /// Markdown.
    #[serde(default)]
    pub answer: String,
}

/// Two to four columns side by side, each with a title and text. On narrow
/// screens they stack.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Columns {
    #[serde(default)]
    pub items: Vec<Column>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Column {
    pub title: Option<String>,
    /// Markdown; images work with `![alt](/media/file.jpg)`.
    #[serde(default)]
    pub text: String,
}

/// A highlighted box that asks the visitor to do something.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CallToAction {
    pub headline: Option<String>,
    /// Markdown.
    pub text: Option<String>,
    pub button_label: Option<String>,
    pub button_link: Option<String>,
}

impl Block {
    /// The name after `:::` in a page file; `None` for plain text.
    pub fn container_type(&self) -> Option<&'static str> {
        match self {
            Block::Text { .. } => None,
            Block::Hero(_) => Some("hero"),
            Block::Image(_) => Some("image"),
            Block::TextImage(_) => Some("text-image"),
            Block::Gallery(_) => Some("gallery"),
            Block::Download(_) => Some("download"),
            Block::Faq(_) => Some("faq"),
            Block::Columns(_) => Some("columns"),
            Block::CallToAction(_) => Some("call-to-action"),
        }
    }
}

/// The container names, as written after `:::` in a page file.
pub const CONTAINER_TYPES: [&str; 8] = [
    "hero",
    "image",
    "text-image",
    "gallery",
    "download",
    "faq",
    "columns",
    "call-to-action",
];

/// Builds a block from a container's type name and its TOML body.
///
/// Returns the TOML error unchanged so the caller can work out the line.
pub(crate) fn from_container(kind: &str, toml: &str) -> Result<Block, ContainerError> {
    match kind {
        "hero" => Ok(Block::Hero(toml::from_str(toml)?)),
        "image" => Ok(Block::Image(toml::from_str(toml)?)),
        "text-image" => Ok(Block::TextImage(toml::from_str(toml)?)),
        "gallery" => Ok(Block::Gallery(toml::from_str(toml)?)),
        "download" => Ok(Block::Download(toml::from_str(toml)?)),
        "faq" => Ok(Block::Faq(toml::from_str(toml)?)),
        "columns" => Ok(Block::Columns(toml::from_str(toml)?)),
        "call-to-action" => Ok(Block::CallToAction(toml::from_str(toml)?)),
        other => Err(ContainerError::UnknownType(other.to_string())),
    }
}

/// Why a container couldn't become a block.
#[derive(Debug)]
pub(crate) enum ContainerError {
    UnknownType(String),
    Toml(toml::de::Error),
}

impl From<toml::de::Error> for ContainerError {
    fn from(error: toml::de::Error) -> Self {
        ContainerError::Toml(error)
    }
}

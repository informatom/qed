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
            Block::CallToAction(_) => Some("call-to-action"),
        }
    }
}

/// The container names, as written after `:::` in a page file.
pub const CONTAINER_TYPES: [&str; 2] = ["hero", "call-to-action"];

/// Builds a block from a container's type name and its TOML body.
///
/// Returns the TOML error unchanged so the caller can work out the line.
pub(crate) fn from_container(kind: &str, toml: &str) -> Result<Block, ContainerError> {
    match kind {
        "hero" => Ok(Block::Hero(toml::from_str(toml)?)),
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

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
//!
//! Columns hold blocks of their own, so they use four colons. Inside,
//! `::: column` starts each column and holds its settings; the blocks after
//! it belong to that column:
//!
//! ```text
//! :::: columns
//! ::: column
//! title = "Our team"
//! :::
//! We are **three** people.
//!
//! ::: image
//! image = "team.jpg"
//! :::
//! ::: column
//! title = "Contact"
//! :::
//! Write to us.
//! ::::
//! ```

use serde::{Deserialize, Serialize};

/// One block of a page, in the order it appears in the file.
///
/// Serialized for the UI with a `type` field, e.g.
/// `{ "type": "hero", "headline": "Welcome", … }`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
    PostList(PostList),
    PageList(PageList),
    Video(Video),
}

/// A large header: optional background image, headline, text and a button.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Left,
    #[default]
    Right,
}

/// Several images in a grid; each links to its full size.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Gallery {
    #[serde(default)]
    pub images: Vec<GalleryImage>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct GalleryImage {
    /// File name in `media/`.
    pub image: String,
    pub alt: Option<String>,
}

/// A file to download, such as a PDF.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Download {
    /// File name in `media/`.
    pub file: Option<String>,
    /// The link text; the file name if empty.
    pub label: Option<String>,
}

/// Questions and answers that open on click.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Faq {
    pub heading: Option<String>,
    #[serde(default)]
    pub items: Vec<FaqItem>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct FaqItem {
    #[serde(default)]
    pub question: String,
    /// Markdown.
    #[serde(default)]
    pub answer: String,
}

/// Two to four columns side by side, each with an optional title and
/// blocks of its own. On narrow screens they stack.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Columns {
    #[serde(default)]
    pub items: Vec<Column>,
}

/// One column: an optional title above its blocks.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Column {
    pub title: Option<String>,
    /// The column's share of the width, relative to the others: `2` next to
    /// a `1` is two thirds. Empty means 1.
    pub width: Option<u8>,
    /// Any of [`COLUMN_TYPES`] and text.
    #[serde(default)]
    pub blocks: Vec<Block>,
}

/// The settings on a `::: column` line, as written in the file.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ColumnSettings {
    pub title: Option<String>,
    pub width: Option<u8>,
}

/// A highlighted box that asks the visitor to do something.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct CallToAction {
    pub headline: Option<String>,
    /// Markdown.
    pub text: Option<String>,
    pub button_label: Option<String>,
    pub button_link: Option<String>,
}

/// The posts below a page, newest first, each with its date and summary.
/// A post is a page with a `date` that isn't in the future.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct PostList {
    /// The page whose posts are listed, as its address (`blog` or
    /// `/blog`); empty for the page the list is on.
    pub from: Option<String>,
    /// At most this many posts; all of them if empty.
    pub limit: Option<u32>,
    /// How the posts show; a plain list if empty.
    pub layout: Option<ListLayout>,
}

/// How a Post list or Page list shows its pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum ListLayout {
    /// Title, (date,) summary, one below the other.
    #[default]
    List,
    /// A grid of cards: picture, title, (date,) summary.
    Cards,
    /// A dense grid of square pictures with title and summary below, for
    /// directories of people or things.
    Tiles,
}

/// The pages one level below a page, by weight, then title, each with its
/// summary: a directory, such as one page per person. Pages with a date
/// are posts and left out; a Post list shows those.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct PageList {
    /// The page whose pages are listed, as its address (`team` or
    /// `/team`); empty for the page the list is on.
    pub from: Option<String>,
    /// How the pages show; a plain list if empty.
    pub layout: Option<ListLayout>,
}

/// A video on YouTube, Vimeo or PeerTube, shown as its thumbnail with a
/// play mark and linked to the video (see `video.rs`).
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Video {
    /// The video's address on its platform.
    pub url: Option<String>,
    pub title: Option<String>,
    /// The thumbnail's name in `media/`, fetched once.
    pub image: Option<String>,
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
            Block::PostList(_) => Some("post-list"),
            Block::PageList(_) => Some("page-list"),
            Block::Video(_) => Some("video"),
        }
    }
}

/// The container names, as written after `:::` in a page file.
/// Every block in a list, outer ones first, including the blocks inside
/// columns.
pub fn all_blocks(blocks: &[Block]) -> Vec<&Block> {
    let mut all = Vec::new();
    for block in blocks {
        all.push(block);
        if let Block::Columns(columns) = block {
            for column in &columns.items {
                all.extend(all_blocks(&column.blocks));
            }
        }
    }
    all
}

/// Calls `change` on every block in a list, outer ones first, including
/// the blocks inside columns.
pub fn for_each_block_mut(blocks: &mut [Block], change: &mut impl FnMut(&mut Block)) {
    for block in blocks {
        change(block);
        if let Block::Columns(columns) = block {
            for column in &mut columns.items {
                for_each_block_mut(&mut column.blocks, change);
            }
        }
    }
}

/// Calls `change` on every Markdown text in a block (not in the blocks
/// inside columns).
pub fn for_each_markdown(block: &mut Block, change: &mut impl FnMut(&mut String)) {
    match block {
        Block::Text { markdown } => change(markdown),
        Block::Hero(hero) => hero.text.iter_mut().for_each(change),
        Block::Image(image) => image.caption.iter_mut().for_each(change),
        Block::TextImage(block) => block.text.iter_mut().for_each(change),
        Block::Faq(faq) => faq
            .items
            .iter_mut()
            .for_each(|item| change(&mut item.answer)),
        Block::CallToAction(cta) => cta.text.iter_mut().for_each(change),
        Block::Gallery(_)
        | Block::Download(_)
        | Block::Columns(_)
        | Block::PostList(_)
        | Block::PageList(_)
        | Block::Video(_) => {}
    }
}

/// The blocks a column can hold, besides text. Hero, Gallery and Columns
/// need the full width.
pub const COLUMN_TYPES: [&str; 8] = [
    "image",
    "text-image",
    "download",
    "faq",
    "call-to-action",
    "post-list",
    "page-list",
    "video",
];

pub const CONTAINER_TYPES: [&str; 11] = [
    "hero",
    "image",
    "text-image",
    "gallery",
    "download",
    "faq",
    "columns",
    "call-to-action",
    "post-list",
    "page-list",
    "video",
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
        "call-to-action" => Ok(Block::CallToAction(toml::from_str(toml)?)),
        "post-list" => Ok(Block::PostList(toml::from_str(toml)?)),
        "page-list" => Ok(Block::PageList(toml::from_str(toml)?)),
        "video" => Ok(Block::Video(toml::from_str(toml)?)),
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

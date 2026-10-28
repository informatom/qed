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
    Shared(Shared),
    /// The podcast's newest episode: title, number, date, summary and the
    /// player, for the start page. Nothing to set.
    LatestEpisode,
    Subscribe(Subscribe),
    /// The podcast's contributors: photo and name, each linking to their
    /// page. Nothing to set.
    Contributors,
    /// The parts of an episode page, for the episode layout (`episode.md`,
    /// see `site.rs`): each stands for that part of the episode the page
    /// shows. Nothing to set; elsewhere they show nothing.
    ///
    /// The episode's audio: the Podlove Web Player, or the browser's own
    /// player without JavaScript, and the download links.
    EpisodePlayer,
    /// The people in the episode, with their roles.
    EpisodePeople,
    /// The episode page's own blocks.
    EpisodeShownotes,
    /// The chapters, as a list; hidden while the Podlove Web Player runs,
    /// which shows them itself.
    EpisodeChapters,
    /// The transcript as text, folded away.
    EpisodeTranscript,
    /// Links for sharing the episode.
    EpisodeShare,
}

/// The Podlove Subscribe Button: it hands the podcast's own feeds to the
/// listener's podcast app. Without JavaScript, links to the feeds.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Subscribe {
    /// Where the button sits in its row; on the left if empty.
    pub align: Option<Align>,
}

/// Where something sits in its row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
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
    /// Drawn as a box: a background of its own, a border, rounded corners.
    #[serde(default)]
    pub boxed: bool,
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
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub boxed: bool,
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
    /// Only the episodes this contributor is in (their id, see
    /// `contributors.rs`); all posts if empty.
    pub contributor: Option<String>,
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
/// play mark and linked to the video (see `video.rs`); or a video file of
/// the site, played by the browser itself (#127).
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Video {
    /// The video's address on its platform.
    pub url: Option<String>,
    /// A video file in `assets/`; wins over `url`.
    pub file: Option<String>,
    pub title: Option<String>,
    /// The thumbnail's name in `media/`, fetched once; for a file, the
    /// picture shown before it plays.
    pub image: Option<String>,
}

/// Stands for the blocks of a shared piece, a file in `shared/` (see
/// `shared.rs`): the same blocks on several pages, edited in one place.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Shared {
    /// The piece's file name without `.md`: `opening-hours` for
    /// `shared/opening-hours.md`.
    pub name: Option<String>,
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
            Block::Shared(_) => Some("shared"),
            Block::LatestEpisode => Some("latest-episode"),
            Block::Subscribe(_) => Some("subscribe"),
            Block::Contributors => Some("contributors"),
            Block::EpisodePlayer => Some("episode-player"),
            Block::EpisodePeople => Some("episode-people"),
            Block::EpisodeShownotes => Some("episode-shownotes"),
            Block::EpisodeChapters => Some("episode-chapters"),
            Block::EpisodeTranscript => Some("episode-transcript"),
            Block::EpisodeShare => Some("episode-share"),
        }
    }

    /// Whether the block stands for a part of an episode page.
    pub fn is_episode_part(&self) -> bool {
        matches!(
            self,
            Block::EpisodePlayer
                | Block::EpisodePeople
                | Block::EpisodeShownotes
                | Block::EpisodeChapters
                | Block::EpisodeTranscript
                | Block::EpisodeShare
        )
    }
}

/// The episode layout without `episode.md`: the order episode pages had
/// before there was a layout.
pub const DEFAULT_EPISODE_LAYOUT: [Block; 5] = [
    Block::EpisodePlayer,
    Block::EpisodePeople,
    Block::EpisodeShownotes,
    Block::EpisodeChapters,
    Block::EpisodeShare,
];

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
        | Block::Video(_)
        | Block::Shared(_)
        | Block::LatestEpisode
        | Block::Subscribe(_)
        | Block::Contributors
        | Block::EpisodePlayer
        | Block::EpisodePeople
        | Block::EpisodeShownotes
        | Block::EpisodeChapters
        | Block::EpisodeTranscript
        | Block::EpisodeShare => {}
    }
}

/// The blocks a column can hold, besides text. Hero, Gallery and Columns
/// need the full width.
pub const COLUMN_TYPES: [&str; 12] = [
    "image",
    "text-image",
    "download",
    "faq",
    "call-to-action",
    "post-list",
    "page-list",
    "video",
    "shared",
    "latest-episode",
    "subscribe",
    "contributors",
];

pub const CONTAINER_TYPES: [&str; 21] = [
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
    "shared",
    "latest-episode",
    "subscribe",
    "contributors",
    "episode-player",
    "episode-people",
    "episode-shownotes",
    "episode-chapters",
    "episode-transcript",
    "episode-share",
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
        "shared" => Ok(Block::Shared(toml::from_str(toml)?)),
        // Blocks without settings: the container must be empty.
        "latest-episode" => nothing_to_set(toml).map(|()| Block::LatestEpisode),
        "subscribe" => Ok(Block::Subscribe(toml::from_str(toml)?)),
        "contributors" => nothing_to_set(toml).map(|()| Block::Contributors),
        "episode-player" => nothing_to_set(toml).map(|()| Block::EpisodePlayer),
        "episode-people" => nothing_to_set(toml).map(|()| Block::EpisodePeople),
        "episode-shownotes" => nothing_to_set(toml).map(|()| Block::EpisodeShownotes),
        "episode-chapters" => nothing_to_set(toml).map(|()| Block::EpisodeChapters),
        "episode-transcript" => nothing_to_set(toml).map(|()| Block::EpisodeTranscript),
        "episode-share" => nothing_to_set(toml).map(|()| Block::EpisodeShare),
        other => Err(ContainerError::UnknownType(other.to_string())),
    }
}

/// Checks that a block without settings has none written in it.
fn nothing_to_set(toml: &str) -> Result<(), ContainerError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Empty {}
    toml::from_str::<Empty>(toml)?;
    Ok(())
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

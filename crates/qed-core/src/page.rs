//! Page files: TOML front matter between `+++` lines, then Markdown with
//! block containers.
//!
//! ```text
//! +++
//! title = "About us"
//! menu = "main"
//! +++
//!
//! ::: hero
//! headline = "Who we are"
//! :::
//!
//! We are a small team.
//! ```

use serde::{Deserialize, Serialize};

use crate::block::{self, Block, CONTAINER_TYPES, ContainerError};

/// Settings at the top of a page file.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct FrontMatter {
    pub title: String,
    /// Meta description for search engines.
    pub description: Option<String>,
    /// Drafts are left out of the built site.
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub menu: Menu,
    /// A Lucide icon name from qed's set, shown in menus: `icon = "book-open"`.
    pub icon: Option<String>,
    /// Sort order among sibling pages; lower comes first.
    #[serde(default)]
    pub weight: i32,
    /// Earlier URLs of this page. The site gets redirects from these.
    #[serde(default)]
    pub redirect_from: Vec<String>,
    /// Publication date, mainly for blog posts: `date = 2026-10-21`.
    pub date: Option<PageDate>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// A short teaser for post lists and feeds. Markdown.
    pub summary: Option<String>,
}

/// A TOML date such as `2026-10-21`, written without quotes in the file.
///
/// Reading uses TOML's own date type, which rejects nonsense like
/// `2026-13-45`. The UI receives it as text, `"2026-10-21"`, and sends it back
/// the same way.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "specta", derive(specta::Type), specta(type = String))]
pub struct PageDate(toml::value::Datetime);

impl<'de> Deserialize<'de> for PageDate {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        /// Either a TOML date (from a page file) or text (from the UI).
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Date(toml::value::Datetime),
            Text(String),
        }

        match Raw::deserialize(deserializer)? {
            Raw::Date(date) => Ok(PageDate(date)),
            Raw::Text(text) => text.parse().map(PageDate).map_err(serde::de::Error::custom),
        }
    }
}

impl Serialize for PageDate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.0)
    }
}

impl std::fmt::Display for PageDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Which menu a page appears in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum Menu {
    #[default]
    None,
    Main,
    Footer,
}

/// The contents of one page file.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PageContent {
    pub front: FrontMatter,
    pub blocks: Vec<Block>,
}

/// A problem in a page file, with the 1-based line it was found on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl ParseError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        ParseError {
            line,
            message: message.into(),
        }
    }
}

const FENCE: &str = "+++";
const CONTAINER: &str = ":::";

/// A parsed page plus the original TOML of its parts, so the writer can keep
/// what the user wrote (comments, formatting) for everything that didn't
/// change.
#[derive(Debug, Clone)]
pub(crate) struct ParsedPage {
    pub content: PageContent,
    pub front_toml: String,
    /// One entry per block: the container's TOML body, `None` for text.
    pub block_tomls: Vec<Option<String>>,
}

/// Parses the text of a page file.
pub fn parse(source: &str) -> Result<PageContent, ParseError> {
    parse_parts(source).map(|parsed| parsed.content)
}

pub(crate) fn parse_parts(source: &str) -> Result<ParsedPage, ParseError> {
    // Pair each line with its 1-based number, for error messages.
    let mut lines = source.lines().enumerate().map(|(i, line)| (i + 1, line));

    match lines.next() {
        Some((_, line)) if line.trim_end() == FENCE => {}
        _ => {
            return Err(ParseError::new(
                1,
                "a page must start with a +++ line, followed by its settings",
            ));
        }
    }

    let mut front_lines = Vec::new();
    let mut closed = false;
    for (_, line) in lines.by_ref() {
        if line.trim_end() == FENCE {
            closed = true;
            break;
        }
        front_lines.push(line);
    }
    if !closed {
        return Err(ParseError::new(
            1,
            "the settings are missing their closing +++ line",
        ));
    }

    // Front matter TOML starts on line 2.
    let front_toml = front_lines.join("\n");
    let front: FrontMatter =
        toml::from_str(&front_toml).map_err(|error| toml_error(&error, &front_lines, 2))?;
    if let Some(icon) = &front.icon
        && crate::icons::svg(icon).is_none()
    {
        // The line the icon is set on, for the error message.
        let line = front_lines
            .iter()
            .position(|line| line.trim_start().starts_with("icon"))
            .map_or(1, |index| index + 2);
        return Err(ParseError::new(
            line,
            format!("there is no icon called \"{icon}\"; pick one in the page settings"),
        ));
    }

    let (blocks, block_tomls) = parse_body(lines)?.into_iter().unzip();
    Ok(ParsedPage {
        content: PageContent { front, blocks },
        front_toml,
        block_tomls,
    })
}

/// A block and, for containers, its TOML body as written.
type BlockWithSource = (Block, Option<String>);

/// Splits the body into Markdown text and containers.
fn parse_body<'a>(
    lines: impl Iterator<Item = (usize, &'a str)>,
) -> Result<Vec<BlockWithSource>, ParseError> {
    let mut blocks = Vec::new();
    let mut text: Vec<&str> = Vec::new();
    // The open container: its type, the line it started on, and its body.
    let mut container: Option<(&str, usize, Vec<&str>)> = None;
    // Inside a ``` code fence, `:::` is just text.
    let mut in_code = false;

    for (number, line) in lines {
        if let Some((kind, start, body)) = container.as_mut() {
            match container_marker(line) {
                Some("") => {
                    let block = container_block(kind, *start, body)?;
                    blocks.push((block, Some(body.join("\n"))));
                    container = None;
                }
                Some(_) => {
                    return Err(ParseError::new(
                        number,
                        "blocks can't be nested yet; close the open block with ::: first",
                    ));
                }
                None => body.push(line),
            }
            continue;
        }

        if is_code_fence(line) {
            in_code = !in_code;
        }

        match container_marker(line).filter(|_| !in_code) {
            Some("") => {
                return Err(ParseError::new(number, "this ::: doesn't close any block"));
            }
            Some(kind) => {
                push_text(&mut blocks, &mut text);
                container = Some((kind, number, Vec::new()));
            }
            None => text.push(line),
        }
    }

    if let Some((kind, start, _)) = container {
        return Err(ParseError::new(
            start,
            format!("the {kind} block isn't closed; add a ::: line after it"),
        ));
    }
    push_text(&mut blocks, &mut text);
    Ok(blocks)
}

/// For a `:::` line, returns what follows the colons (empty for a closing
/// line). Returns `None` for any other line.
fn container_marker(line: &str) -> Option<&str> {
    line.trim_end().strip_prefix(CONTAINER).map(str::trim)
}

fn is_code_fence(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("```") || line.starts_with("~~~")
}

/// Turns collected Markdown lines into a text block, unless they're blank.
fn push_text(blocks: &mut Vec<BlockWithSource>, text: &mut Vec<&str>) {
    let markdown = text.join("\n").trim_matches('\n').to_string();
    if !markdown.trim().is_empty() {
        blocks.push((Block::Text { markdown }, None));
    }
    text.clear();
}

fn container_block(kind: &str, start: usize, body: &[&str]) -> Result<Block, ParseError> {
    block::from_container(kind, &body.join("\n")).map_err(|error| match error {
        ContainerError::UnknownType(kind) => ParseError::new(
            start,
            format!(
                "unknown block type \"{kind}\"; known types are: {}",
                CONTAINER_TYPES.join(", ")
            ),
        ),
        // The block's TOML starts on the line after the `::: kind` line.
        ContainerError::Toml(error) => toml_error(&error, body, start + 1),
    })
}

/// Converts a TOML error into a [`ParseError`] on the right line of the file.
///
/// `first_line` is the file line where `lines` begins.
fn toml_error(error: &toml::de::Error, lines: &[&str], first_line: usize) -> ParseError {
    let offset = match error.span() {
        Some(span) => {
            let text = lines.join("\n");
            text[..span.start.min(text.len())].matches('\n').count()
        }
        None => 0,
    };
    ParseError::new(first_line + offset, error.message().trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{CallToAction, Hero};

    fn page(body: &str) -> String {
        format!("+++\ntitle = \"Test\"\n+++\n{body}")
    }

    #[test]
    fn reads_front_matter_with_defaults() {
        let content = parse("+++\ntitle = \"About\"\nmenu = \"main\"\n+++\n").unwrap();
        assert_eq!(content.front.title, "About");
        assert_eq!(content.front.menu, Menu::Main);
        assert!(!content.front.draft);
        assert_eq!(content.front.weight, 0);
        assert!(content.blocks.is_empty());
    }

    #[test]
    fn splits_markdown_and_containers_in_order() {
        let source = page(
            "\nIntro text.\n\n::: hero\nheadline = \"Hi\"\nimage = \"team.jpg\"\n:::\n\nMore **text**.\n\n::: call-to-action\nbutton_label = \"Go\"\n:::\n",
        );
        let blocks = parse(&source).unwrap().blocks;
        assert_eq!(
            blocks,
            vec![
                Block::Text {
                    markdown: "Intro text.".into()
                },
                Block::Hero(Hero {
                    headline: Some("Hi".into()),
                    image: Some("team.jpg".into()),
                    ..Hero::default()
                }),
                Block::Text {
                    markdown: "More **text**.".into()
                },
                Block::CallToAction(CallToAction {
                    button_label: Some("Go".into()),
                    ..CallToAction::default()
                }),
            ]
        );
    }

    #[test]
    fn keeps_colons_inside_code_fences_as_text() {
        let source = page("```\n::: hero\n```\n");
        let blocks = parse(&source).unwrap().blocks;
        assert_eq!(
            blocks,
            vec![Block::Text {
                markdown: "```\n::: hero\n```".into()
            }]
        );
    }

    #[test]
    fn handles_windows_line_endings() {
        let source = "+++\r\ntitle = \"Win\"\r\n+++\r\n::: hero\r\nheadline = \"Hi\"\r\n:::\r\n";
        let content = parse(source).unwrap();
        assert_eq!(content.front.title, "Win");
        assert_eq!(content.blocks.len(), 1);
    }

    #[test]
    fn reads_post_fields() {
        let source = "+++\ntitle = \"Post\"\ndate = 2026-10-21\nauthors = [\"Claude\"]\ntags = [\"rust\"]\nsummary = \"Short.\"\n+++\n";
        let front = parse(source).unwrap().front;
        assert_eq!(front.date.unwrap().to_string(), "2026-10-21");
        assert_eq!(front.authors, ["Claude"]);
        assert_eq!(front.tags, ["rust"]);
        assert_eq!(front.summary.as_deref(), Some("Short."));
    }

    #[test]
    fn sends_dates_to_the_ui_as_text() {
        let front = parse("+++\ntitle = \"P\"\ndate = 2026-10-21\n+++\n")
            .unwrap()
            .front;
        let json = serde_json::to_value(&front).unwrap();
        assert_eq!(json["date"], "2026-10-21");
    }

    #[test]
    fn reads_dates_sent_as_text() {
        let date: PageDate = serde_json::from_str("\"2026-10-21\"").unwrap();
        assert_eq!(date.to_string(), "2026-10-21");
        assert!(serde_json::from_str::<PageDate>("\"21.10.2026\"").is_err());
    }

    #[test]
    fn reads_icons_and_rejects_unknown_ones() {
        let page = parse("+++\ntitle = \"Blog\"\nicon = \"book-open\"\n+++\n").unwrap();
        assert_eq!(page.front.icon.as_deref(), Some("book-open"));

        let error = parse("+++\ntitle = \"Blog\"\nicon = \"book-closed\"\n+++\n").unwrap_err();
        assert_eq!(error.line, 3, "{error:?}");
        assert!(error.message.contains("book-closed"), "{}", error.message);
    }

    #[test]
    fn rejects_impossible_dates() {
        let error = parse("+++\ntitle = \"P\"\ndate = 2026-13-45\n+++\n").unwrap_err();
        assert_eq!(error.line, 3, "{error:?}");
    }

    #[test]
    fn rejects_a_missing_front_matter() {
        let error = parse("# Just Markdown\n").unwrap_err();
        assert_eq!(error.line, 1);
    }

    #[test]
    fn rejects_unclosed_front_matter() {
        let error = parse("+++\ntitle = \"x\"\n").unwrap_err();
        assert!(error.message.contains("closing +++"), "{}", error.message);
    }

    #[test]
    fn reports_the_line_of_a_bad_front_matter_field() {
        // `menu` is on line 3 of the file.
        let error = parse("+++\ntitle = \"x\"\nmenu = \"side\"\n+++\n").unwrap_err();
        assert_eq!(error.line, 3, "{error:?}");
    }

    #[test]
    fn rejects_unknown_front_matter_fields() {
        let error = parse("+++\ntitle = \"x\"\ntitel = \"typo\"\n+++\n").unwrap_err();
        assert_eq!(error.line, 3, "{error:?}");
        assert!(error.message.contains("titel"), "{}", error.message);
    }

    #[test]
    fn reports_unknown_block_types_with_the_known_ones() {
        // Lines: 1 +++, 2 title, 3 +++, 4 text, 5 ::: herro
        let error = parse(&page("text\n::: herro\n:::\n")).unwrap_err();
        assert_eq!(error.line, 5);
        assert!(error.message.contains("herro"), "{}", error.message);
        assert!(
            error.message.contains("call-to-action"),
            "{}",
            error.message
        );
    }

    #[test]
    fn reports_the_line_of_a_bad_block_field() {
        // Lines: 4 ::: hero, 5 headline, 6 colour
        let error = parse(&page(
            "::: hero\nheadline = \"Hi\"\ncolour = \"red\"\n:::\n",
        ))
        .unwrap_err();
        assert_eq!(error.line, 6, "{error:?}");
    }

    #[test]
    fn rejects_unclosed_blocks_at_their_start() {
        let error = parse(&page("\n::: hero\nheadline = \"Hi\"\n")).unwrap_err();
        assert_eq!(error.line, 5);
    }

    #[test]
    fn rejects_nested_blocks() {
        let error = parse(&page("::: hero\n::: call-to-action\n:::\n:::\n")).unwrap_err();
        assert_eq!(error.line, 5);
    }

    #[test]
    fn rejects_a_stray_closing_line() {
        let error = parse(&page(":::\n")).unwrap_err();
        assert_eq!(error.line, 4);
    }
}

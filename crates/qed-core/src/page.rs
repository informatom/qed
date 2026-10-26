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

use crate::block::{
    self, Block, COLUMN_TYPES, CONTAINER_TYPES, Column, ColumnSettings, Columns, ContainerError,
};

/// Settings at the top of a page file.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct FrontMatter {
    pub title: String,
    /// Meta description for search engines.
    pub description: Option<String>,
    /// The image link previews show (`og:image`), e.g. on social media or in
    /// chat apps: a file in `media/`. Without one, the page's first image is
    /// used.
    pub image: Option<String>,
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

impl PageDate {
    /// The calendar date, without a time of day; `None` for a bare time.
    pub fn date(&self) -> Option<crate::calendar::Date> {
        self.0.date.map(|date| crate::calendar::Date {
            year: i64::from(date.year),
            month: u32::from(date.month),
            day: u32::from(date.day),
        })
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

/// A parsed page plus the original TOML of its parts, so the writer can keep
/// what the user wrote (comments, formatting) for everything that didn't
/// change.
#[derive(Debug, Clone)]
pub(crate) struct ParsedPage {
    pub content: PageContent,
    pub front_toml: String,
    /// One entry per block: how it was written.
    pub block_sources: Vec<BlockSource>,
}

/// How a block was written, so the writer can keep it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BlockSource {
    /// Markdown text is its own source.
    Text,
    /// A `:::` container's TOML body.
    Toml(String),
    /// Each column's settings and blocks.
    Columns(Vec<ColumnSource>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ColumnSource {
    /// The TOML body of the `::: column` line.
    pub toml: String,
    pub blocks: Vec<BlockSource>,
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

    let body: Vec<(usize, &str)> = lines.collect();
    let mut blocks = Vec::new();
    let mut block_sources = Vec::new();
    for piece in parse_body(&body, false)? {
        // Column markers only come back from inside columns.
        if let Piece::Block(block, source) = piece {
            blocks.push(block);
            block_sources.push(source);
        }
    }
    Ok(ParsedPage {
        content: PageContent { front, blocks },
        front_toml,
        block_sources,
    })
}

/// What the body of a page (or of a columns block) is made of.
enum Piece {
    Block(Block, BlockSource),
    /// A `::: column` line: a new column starts here.
    Column(ColumnSettings, String),
}

/// A `:::` container that's open: its type, first line and body lines.
struct Open<'a> {
    kind: &'a str,
    start: usize,
    body: Vec<&'a str>,
}

/// Splits a body into Markdown text and blocks. `in_columns` is true inside
/// `:::: columns`, where `::: column` lines separate the columns.
fn parse_body<'a>(lines: &[(usize, &'a str)], in_columns: bool) -> Result<Vec<Piece>, ParseError> {
    let mut pieces = Vec::new();
    let mut text: Vec<&str> = Vec::new();
    let mut open: Option<Open> = None;
    // An open `:::: columns`: its first line and the lines inside.
    let mut columns: Option<(usize, Vec<(usize, &'a str)>)> = None;
    // Inside a ``` code fence, colons are just text.
    let mut in_code = false;

    for &(number, line) in lines {
        if let Some((_, inner)) = columns.as_mut() {
            if is_code_fence(line) {
                in_code = !in_code;
            }
            if !in_code && marker(line) == Some((4, "")) {
                if let Some((start, inner)) = columns.take() {
                    pieces.push(columns_block(start, &inner)?);
                }
            } else {
                inner.push((number, line));
            }
            continue;
        }

        if let Some(container) = open.as_mut() {
            match marker(line) {
                Some((3, "")) => {
                    if let Some(container) = open.take() {
                        pieces.push(close(container, in_columns)?);
                    }
                }
                Some(_) => {
                    return Err(ParseError::new(
                        number,
                        format!(
                            "blocks can't go inside a {} block; close it with ::: first",
                            container.kind
                        ),
                    ));
                }
                None => container.body.push(line),
            }
            continue;
        }

        if is_code_fence(line) {
            in_code = !in_code;
        }
        let Some((colons, kind)) = marker(line).filter(|_| !in_code) else {
            text.push(line);
            continue;
        };
        let error = |message: String| Err(ParseError::new(number, message));
        match (colons, kind) {
            (3, "") => return error("this ::: doesn't close any block".into()),
            (3, "column") if !in_columns => {
                return error("::: column only works inside a :::: columns block".into());
            }
            (3, "columns") => {
                return error("columns hold blocks, so they use four colons: :::: columns".into());
            }
            (3, kind) => {
                push_text(&mut pieces, &mut text);
                open = Some(Open {
                    kind,
                    start: number,
                    body: Vec::new(),
                });
            }
            (4, "") => return error("this :::: doesn't close any columns block".into()),
            (4, "columns") if in_columns => return error("columns can't go inside columns".into()),
            (4, "columns") => {
                push_text(&mut pieces, &mut text);
                columns = Some((number, Vec::new()));
            }
            (4, kind) => {
                return error(format!(
                    "only columns use ::::; write ::: {kind} for a {kind} block"
                ));
            }
            _ => return error("use ::: for blocks and :::: for columns".into()),
        }
    }

    if let Some(container) = open {
        return Err(ParseError::new(
            container.start,
            format!(
                "the {} block isn't closed; add a ::: line after it",
                container.kind
            ),
        ));
    }
    if let Some((start, _)) = columns {
        return Err(ParseError::new(
            start,
            "the columns block isn't closed; add a :::: line after it",
        ));
    }
    push_text(&mut pieces, &mut text);
    Ok(pieces)
}

/// For a line of three or more colons, how many and what follows them
/// (empty for a closing line). `None` for any other line.
fn marker(line: &str) -> Option<(usize, &str)> {
    let line = line.trim_end();
    let colons = line.chars().take_while(|&c| c == ':').count();
    (colons >= 3).then(|| (colons, line[colons..].trim()))
}

/// A closed `:::` container: a block, or a column marker inside columns.
fn close(container: Open, in_columns: bool) -> Result<Piece, ParseError> {
    let Open { kind, start, body } = container;
    let toml = body.join("\n");
    if kind == "column" {
        let settings =
            toml::from_str(&toml).map_err(|error| toml_error(&error, &body, start + 1))?;
        return Ok(Piece::Column(settings, toml));
    }
    if in_columns && CONTAINER_TYPES.contains(&kind) && !COLUMN_TYPES.contains(&kind) {
        return Err(ParseError::new(
            start,
            format!("a {kind} block can't go inside a column; it needs the full width"),
        ));
    }
    let block = container_block(kind, start, &body)?;
    Ok(Piece::Block(block, BlockSource::Toml(toml)))
}

/// Groups the inside of `:::: columns` into its columns.
fn columns_block(start: usize, lines: &[(usize, &str)]) -> Result<Piece, ParseError> {
    let mut items: Vec<Column> = Vec::new();
    let mut sources: Vec<ColumnSource> = Vec::new();
    for piece in parse_body(lines, true)? {
        match piece {
            Piece::Column(settings, toml) => {
                items.push(Column {
                    title: settings.title,
                    blocks: Vec::new(),
                });
                sources.push(ColumnSource {
                    toml,
                    blocks: Vec::new(),
                });
            }
            Piece::Block(block, source) => match (items.last_mut(), sources.last_mut()) {
                (Some(column), Some(column_source)) => {
                    column.blocks.push(block);
                    column_source.blocks.push(source);
                }
                _ => {
                    return Err(ParseError::new(
                        start + 1,
                        "start each column with a ::: column line",
                    ));
                }
            },
        }
    }
    Ok(Piece::Block(
        Block::Columns(Columns { items }),
        BlockSource::Columns(sources),
    ))
}

fn is_code_fence(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("```") || line.starts_with("~~~")
}

/// Turns collected Markdown lines into a text block, unless they're blank.
fn push_text(pieces: &mut Vec<Piece>, text: &mut Vec<&str>) {
    let markdown = text.join("\n").trim_matches('\n').to_string();
    if !markdown.trim().is_empty() {
        pieces.push(Piece::Block(Block::Text { markdown }, BlockSource::Text));
    }
    text.clear();
}

fn container_block(kind: &str, start: usize, body: &[&str]) -> Result<Block, ParseError> {
    block::from_container(kind, &body.join("\n")).map_err(|error| match error {
        ContainerError::UnknownType(kind) => ParseError::new(
            start,
            // Most often the site was edited with a newer q.e.d. that has
            // more block types, so say how to get it.
            format!(
                "this version of q.e.d. doesn't know the block type \"{kind}\". \
                 If the site was edited with a newer q.e.d., update this one \
                 (App → Check for updates). Known types: {}",
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
        assert!(
            error.message.contains("Check for updates"),
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
    fn reads_columns_with_blocks_inside() {
        let content = parse(&page(
            ":::: columns\n::: column\ntitle = \"Team\"\n:::\nWe are **three**.\n\n::: image\nimage = \"team.jpg\"\n:::\n::: column\n:::\n```\n::::\n```\n::::\n\nAfter.\n",
        ))
        .unwrap();
        let [Block::Columns(columns), Block::Text { markdown: after }] = content.blocks.as_slice()
        else {
            panic!("{:?}", content.blocks);
        };
        assert_eq!(after, "After.");
        assert_eq!(columns.items.len(), 2);
        assert_eq!(columns.items[0].title.as_deref(), Some("Team"));
        assert!(matches!(
            &columns.items[0].blocks[..],
            [Block::Text { .. }, Block::Image(_)]
        ));
        assert_eq!(columns.items[1].title, None);
        assert_eq!(
            columns.items[1].blocks,
            [Block::Text {
                markdown: "```\n::::\n```".into()
            }],
            "colons in code stay text"
        );
    }

    #[test]
    fn explains_mistakes_with_columns() {
        let error_for = |body: &str| parse(&page(body)).unwrap_err();
        // Lines: 1-3 front matter, so the body starts on line 4.
        let error = error_for("::: columns\n:::\n");
        assert!(error.message.contains("four colons"), "{error:?}");
        let error = error_for(":::: columns\nText first\n::::\n");
        assert!(error.message.contains("::: column line"), "{error:?}");
        let error = error_for(":::: columns\n::: column\n:::\n::: hero\n:::\n::::\n");
        assert_eq!(error.line, 7);
        assert!(
            error.message.contains("can't go inside a column"),
            "{error:?}"
        );
        let error = error_for(":::: columns\n::: column\n:::\n:::: columns\n::::\n::::\n");
        assert!(error.message.contains("inside columns"), "{error:?}");
        let error = error_for("::: column\n:::\n");
        assert!(error.message.contains("only works inside"), "{error:?}");
        let error = error_for(":::: columns\n::: column\n:::\n");
        assert_eq!(error.line, 4);
        assert!(error.message.contains("isn't closed"), "{error:?}");
        let error = error_for(":::: hero\n::::\n");
        assert!(error.message.contains("only columns use ::::"), "{error:?}");
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

//! Writes pages back to their files.
//!
//! People also edit page files by hand, so saving from the app must not undo
//! their work. The writer keeps as much of the original file as it can:
//!
//! - Nothing changed: the file is returned exactly as it was.
//! - A block didn't change: its text is copied as written.
//! - Settings or a block changed: only the changed values are replaced, with
//!   `toml_edit`, which keeps comments, order and formatting of the rest.
//!
//! Values at their defaults (`draft = false`, `menu = "none"`, empty lists)
//! are left out of new files.
//!
//! Two text blocks in a row can't be told apart in a file: they're read back
//! as one block, joined by a blank line.

use toml_edit::{DocumentMut, Item, Value};

use crate::block::Block;
use crate::page::{self, FrontMatter, PageContent, ParsedPage};

/// Turns a page into the text of its file. `original` is the file's current
/// text, if it has one.
pub fn write(content: &PageContent, original: Option<&str>) -> String {
    let previous = original.and_then(|text| page::parse_parts(text).ok());

    if let (Some(previous), Some(text)) = (&previous, original)
        && previous.content == *content
    {
        return text.to_string();
    }

    let front = match &previous {
        Some(p) if p.content.front == content.front => p.front_toml.clone(),
        Some(p) => merge(
            &p.front_toml,
            front_document(&content.front, Some(&p.front_toml)),
        ),
        None => front_document(&content.front, None).to_string(),
    };

    let mut file = format!("+++\n{}\n+++\n", front.trim_end());
    for (index, block) in content.blocks.iter().enumerate() {
        file.push('\n');
        file.push_str(&write_block(
            block,
            previous_block(previous.as_ref(), index),
        ));
        file.push('\n');
    }
    file
}

/// The block that was at `index` before, with its TOML as written.
fn previous_block(previous: Option<&ParsedPage>, index: usize) -> Option<(&Block, &str)> {
    let previous = previous?;
    let block = previous.content.blocks.get(index)?;
    let toml = previous.block_tomls.get(index)?.as_deref()?;
    Some((block, toml))
}

fn write_block(block: &Block, previous: Option<(&Block, &str)>) -> String {
    let Some(kind) = block.container_type() else {
        // Text blocks are their own Markdown.
        return match block {
            Block::Text { markdown } => markdown.clone(),
            _ => String::new(),
        };
    };

    let toml = match previous {
        Some((old, toml)) if old == block => toml.to_string(),
        Some((old, toml)) if old.container_type() == Some(kind) => {
            merge(toml, block_document(block))
        }
        _ => block_document(block).to_string(),
    };
    match toml.trim_end() {
        "" => format!("::: {kind}\n:::"),
        toml => format!("::: {kind}\n{toml}\n:::"),
    }
}

/// A block's fields as a fresh TOML document. Empty fields are left out.
fn block_document(block: &Block) -> DocumentMut {
    let document = match block {
        Block::Text { .. } => Ok(DocumentMut::new()),
        Block::Hero(hero) => toml_edit::ser::to_document(hero),
        Block::Image(image) => toml_edit::ser::to_document(image),
        Block::TextImage(block) => toml_edit::ser::to_document(block),
        Block::Gallery(block) => toml_edit::ser::to_document(block),
        Block::Download(block) => toml_edit::ser::to_document(block),
        Block::Faq(block) => toml_edit::ser::to_document(block),
        Block::Columns(block) => toml_edit::ser::to_document(block),
        Block::CallToAction(cta) => toml_edit::ser::to_document(cta),
    };
    // Structs of strings always serialize; an empty document is a safe fallback.
    let mut document = document.unwrap_or_default();
    use_multiline_strings(&mut document);
    document
}

/// The page settings as a fresh TOML document, without default values.
/// The page settings as a TOML document. Values at their defaults are left
/// out, unless `original` (the settings as written) already has that line
/// with that same default: a `draft = false` someone wrote stays, but a
/// `menu = "main"` changed back to no menu goes away.
fn front_document(front: &FrontMatter, original: Option<&str>) -> DocumentMut {
    let mut document = toml_edit::ser::to_document(front).unwrap_or_default();

    let defaults: [DefaultCheck; 6] = [
        ("draft", |item| item.as_bool() == Some(false)),
        ("menu", |item| item.as_str() == Some("none")),
        ("weight", |item| item.as_integer() == Some(0)),
        ("redirect_from", is_empty_array),
        ("authors", is_empty_array),
        ("tags", is_empty_array),
    ];
    let original = original.and_then(|text| text.parse::<DocumentMut>().ok());
    for (key, is_default) in defaults {
        let written_as_default = original
            .as_ref()
            .and_then(|document| document.get(key))
            .is_some_and(is_default);
        if document.get(key).is_some_and(is_default) && !written_as_default {
            document.remove(key);
        }
    }

    // The date arrives as text; write it as a real TOML date, without quotes.
    if let Some(date) = &front.date
        && let Ok(date) = date.to_string().parse::<toml_edit::Datetime>()
    {
        document["date"] = toml_edit::value(date);
    }

    use_multiline_strings(&mut document);
    document
}

/// A setting's name and a test for whether it holds its default value.
type DefaultCheck = (&'static str, fn(&Item) -> bool);

/// Whether the TOML text `original` sets `key` to the string `value`, to
/// tell a default someone wrote from one they didn't.
pub(crate) fn writes_string(original: &str, key: &str, value: &str) -> bool {
    original
        .parse::<DocumentMut>()
        .ok()
        .and_then(|document| {
            document
                .get(key)
                .and_then(|item| item.as_str())
                .map(|text| text == value)
        })
        .unwrap_or(false)
}

fn is_empty_array(item: &Item) -> bool {
    item.as_array().is_some_and(|array| array.is_empty())
}

/// Writes strings that contain line breaks as `"""` blocks, so Markdown in
/// a hero text stays readable in the file.
fn use_multiline_strings(document: &mut DocumentMut) {
    readable(document.as_table_mut());
}

/// Makes a table read well by hand: lists of tables become `[[name]]`
/// sections instead of one long inline line, and strings with line breaks
/// become `"""` blocks. Goes into nested sections too.
fn readable(table: &mut toml_edit::Table) {
    for (_, item) in table.iter_mut() {
        let is_list_of_tables = item
            .as_array()
            .is_some_and(|array| !array.is_empty() && array.iter().all(Value::is_inline_table));
        if is_list_of_tables {
            let list = std::mem::take(item);
            *item = list
                .into_array_of_tables()
                .map(Item::ArrayOfTables)
                .unwrap_or_else(|list| list);
        }

        match item {
            Item::ArrayOfTables(tables) => tables.iter_mut().for_each(readable),
            Item::Table(nested) => readable(nested),
            _ => {
                if let Some(text) = item.as_str()
                    && text.contains('\n')
                {
                    let literal = format!("\"\"\"\n{}\"\"\"", escape_multiline(text));
                    if let Ok(value) = literal.parse::<Value>() {
                        *item = Item::Value(value);
                    }
                }
            }
        }
    }
}

/// Escapes what a `"""` string can't contain as is.
fn escape_multiline(text: &str) -> String {
    let mut escaped = text.replace('\\', "\\\\").replace("\"\"\"", "\\\"\\\"\\\"");
    if escaped.ends_with('"') {
        // A quote right before the closing `"""` would be ambiguous.
        escaped.pop();
        escaped.push_str("\\\"");
    }
    escaped
}

/// Applies the values of `new` to the TOML text `original`, keeping
/// everything about `original` that the new values don't change.
pub(crate) fn merge(original: &str, new: DocumentMut) -> String {
    let Ok(mut document) = original.parse::<DocumentMut>() else {
        return new.to_string();
    };

    let removed: Vec<String> = document
        .iter()
        .map(|(key, _)| key.to_string())
        .filter(|key| !new.contains_key(key))
        .collect();
    for key in removed {
        document.remove(&key);
    }

    for (key, item) in new.iter() {
        match document.get_mut(key) {
            Some(existing) if same_item(existing, item) => {}
            Some(existing) => {
                // Keep the comment and spacing around the old value.
                if let (Some(old), Some(new)) = (existing.as_value(), item.as_value()) {
                    let mut value = new.clone();
                    *value.decor_mut() = old.decor().clone();
                    *existing = Item::Value(value);
                } else {
                    *existing = item.clone();
                }
            }
            None => {
                document.insert(key, item.clone());
            }
        }
    }
    document.to_string()
}

/// Whether two items hold the same data, however they're written: `'a'` and
/// `"a"` are the same string.
fn same_item(a: &Item, b: &Item) -> bool {
    match (a.as_value(), b.as_value()) {
        (Some(a), Some(b)) => same_value(a, b),
        _ => a.to_string() == b.to_string(),
    }
}

fn same_value(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::String(a), Value::String(b)) => a.value() == b.value(),
        (Value::Integer(a), Value::Integer(b)) => a.value() == b.value(),
        (Value::Boolean(a), Value::Boolean(b)) => a.value() == b.value(),
        (Value::Datetime(a), Value::Datetime(b)) => a.value() == b.value(),
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| same_value(a, b))
        }
        _ => {
            let (mut a, mut b) = (a.clone(), b.clone());
            a.decor_mut().clear();
            b.decor_mut().clear();
            a.to_string() == b.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::Hero;
    use crate::page::{Menu, parse};

    const HAND_WRITTEN: &str = r#"+++
# Shown in the browser tab
title = "About us"
menu = "main"   # top navigation
+++

Some *intro* text.

::: hero
headline = 'Hello'
# The button
button_label = "Go"
button_link = "/go"
:::

More text.
"#;

    #[test]
    fn returns_unchanged_files_as_they_were() {
        let content = parse(HAND_WRITTEN).unwrap();
        assert_eq!(write(&content, Some(HAND_WRITTEN)), HAND_WRITTEN);
    }

    #[test]
    fn keeps_comments_when_a_setting_changes() {
        let mut content = parse(HAND_WRITTEN).unwrap();
        content.front.title = "About".into();

        let file = write(&content, Some(HAND_WRITTEN));
        assert!(
            file.contains("# Shown in the browser tab\ntitle = \"About\""),
            "{file}"
        );
        assert!(
            file.contains("menu = \"main\"   # top navigation"),
            "{file}"
        );
        // The blocks didn't change, so they're copied as written.
        assert!(file.contains("headline = 'Hello'\n# The button"), "{file}");
        assert_eq!(parse(&file).unwrap(), content);
    }

    #[test]
    fn changes_only_the_edited_block_field() {
        let mut content = parse(HAND_WRITTEN).unwrap();
        if let Block::Hero(hero) = &mut content.blocks[1] {
            hero.button_label = Some("Start".into());
        }

        let file = write(&content, Some(HAND_WRITTEN));
        assert!(file.contains("headline = 'Hello'"), "{file}");
        assert!(
            file.contains("# The button\nbutton_label = \"Start\""),
            "{file}"
        );
        assert_eq!(parse(&file).unwrap(), content);
    }

    #[test]
    fn removes_settings_that_return_to_their_default() {
        let mut content = parse(HAND_WRITTEN).unwrap();
        content.front.menu = Menu::None;

        let file = write(&content, Some(HAND_WRITTEN));
        assert!(!file.contains("menu"), "{file}");
        assert_eq!(parse(&file).unwrap(), content);
    }

    #[test]
    fn keeps_default_values_that_were_written_by_hand() {
        let original = "+++\ntitle = \"A\"\ndraft = false\n+++\n";
        let mut content = parse(original).unwrap();
        content.front.title = "B".into();
        assert_eq!(
            write(&content, Some(original)),
            "+++\ntitle = \"B\"\ndraft = false\n+++\n"
        );
    }

    #[test]
    fn writes_new_pages_without_defaults() {
        let content = parse("+++\ntitle = \"New\"\n+++\n").unwrap();
        assert_eq!(write(&content, None), "+++\ntitle = \"New\"\n+++\n");
    }

    #[test]
    fn writes_dates_without_quotes() {
        let content = parse("+++\ntitle = \"P\"\ndate = 2026-10-21\ndraft = true\n+++\n").unwrap();
        let file = write(&content, None);
        assert!(file.contains("date = 2026-10-21\n"), "{file}");
        assert!(file.contains("draft = true"), "{file}");
        assert_eq!(parse(&file).unwrap(), content);
    }

    #[test]
    fn writes_multiline_text_readably() {
        let content = PageContent {
            front: parse("+++\ntitle = \"T\"\n+++\n").unwrap().front,
            blocks: vec![Block::Hero(Hero {
                text: Some("Line one\n\nLine \"two\" with \\ backslash\"".into()),
                ..Hero::default()
            })],
        };
        let file = write(&content, None);
        assert!(file.contains("text = \"\"\"\nLine one\n\nLine"), "{file}");
        assert_eq!(parse(&file).unwrap(), content);
    }

    #[test]
    fn writes_lists_in_blocks_as_sections() {
        use crate::block::{Faq, FaqItem};
        let content = PageContent {
            front: parse("+++\ntitle = \"T\"\n+++\n").unwrap().front,
            blocks: vec![Block::Faq(Faq {
                heading: Some("Questions".into()),
                items: vec![
                    FaqItem {
                        question: "Why?".into(),
                        answer: "Because.\n\nReally.".into(),
                    },
                    FaqItem {
                        question: "How?".into(),
                        answer: "Like this.".into(),
                    },
                ],
            })],
        };
        let file = write(&content, None);
        assert!(
            file.contains("::: faq\nheading = \"Questions\"\n\n[[items]]\nquestion = \"Why?\"\nanswer = \"\"\"\nBecause.\n\nReally.\"\"\"\n\n[[items]]"),
            "{file}"
        );
        assert_eq!(parse(&file).unwrap(), content);
    }

    #[test]
    fn writes_blocks_added_after_existing_ones() {
        let mut content = parse(HAND_WRITTEN).unwrap();
        content.blocks.push(Block::CallToAction(Default::default()));
        content.blocks.push(Block::Text {
            markdown: "The end.".into(),
        });
        let file = write(&content, Some(HAND_WRITTEN));
        assert!(
            file.ends_with("More text.\n\n::: call-to-action\n:::\n\nThe end.\n"),
            "{file}"
        );
        assert_eq!(parse(&file).unwrap(), content);
    }

    #[test]
    fn joins_text_blocks_that_follow_each_other() {
        let mut content = parse(HAND_WRITTEN).unwrap();
        content.blocks.push(Block::Text {
            markdown: "The end.".into(),
        });
        let reread = parse(&write(&content, Some(HAND_WRITTEN))).unwrap();
        assert_eq!(
            reread.blocks.last(),
            Some(&Block::Text {
                markdown: "More text.\n\nThe end.".into()
            })
        );
    }

    #[test]
    fn round_trips_the_docs_and_demo_pages() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for folder in ["docs", "examples/demo-site"] {
            let site = crate::site::open(&root.join(folder)).unwrap();
            for page in site.pages {
                let text = std::fs::read_to_string(root.join(folder).join(&page.file)).unwrap();
                // Rewriting from scratch must give back the same content.
                let fresh = write(&page.content, None);
                assert_eq!(
                    parse(&fresh).unwrap(),
                    page.content,
                    "{}",
                    page.file.display()
                );
                // And saving an unchanged page must not touch the file.
                assert_eq!(write(&page.content, Some(&text)), text);
            }
        }
    }
}

//! Markdown to HTML, with comrak (the engine behind Elixir's MDEx).

use comrak::nodes::NodeValue;
use comrak::{Arena, Options, format_html, parse_document};

use crate::links::LinkContext;

/// Renders Markdown to HTML.
///
/// Raw HTML in the Markdown is escaped, so it shows as text instead of
/// running. Links and images are rewritten for the built site (see
/// [`LinkContext::resolve`]).
pub fn render(markdown: &str, links: &LinkContext) -> String {
    let options = options();
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &options);

    for node in root.descendants() {
        if let NodeValue::Link(ref mut link) | NodeValue::Image(ref mut link) =
            node.data.borrow_mut().value
        {
            link.url = links.resolve(&link.url);
        }
    }

    let mut html = String::new();
    // Writing into a String can't fail.
    let _ = format_html(root, &options, &mut html);
    html
}

/// Every link and image target in a piece of Markdown, as written.
pub fn link_targets(markdown: &str) -> Vec<String> {
    let options = options();
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &options);
    root.descendants()
        .filter_map(|node| match &node.data.borrow().value {
            NodeValue::Link(link) | NodeValue::Image(link) => Some(link.url.clone()),
            _ => None,
        })
        .collect()
}

fn options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.render.escape = true;
    options
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn render_on(path: &str, file: &str, markdown: &str) -> String {
        render(markdown, &LinkContext::new(path, Path::new(file)))
    }

    #[test]
    fn renders_markdown() {
        let html = render_on("", "pages/index.md", "Hello **world**");
        assert_eq!(html, "<p>Hello <strong>world</strong></p>\n");
    }

    #[test]
    fn renders_tables() {
        let html = render_on("", "pages/index.md", "| a | b |\n|---|---|\n| 1 | 2 |\n");
        assert!(html.contains("<table>"), "{html}");
    }

    #[test]
    fn escapes_raw_html() {
        let html = render_on("", "pages/index.md", "<script>alert(1)</script>");
        assert!(!html.contains("<script>"), "{html}");
        assert!(html.contains("&lt;script&gt;"), "{html}");
    }

    #[test]
    fn rewrites_links_and_images() {
        let html = render_on(
            "blog/post",
            "pages/blog/post.md",
            "[About](/about) [Next](next.md) ![Logo](/media/logo.png) [Web](https://example.com)",
        );
        assert!(html.contains(r#"href="../../about/""#), "{html}");
        assert!(html.contains(r#"href="../../blog/next/""#), "{html}");
        assert!(html.contains(r#"src="../../media/logo.png""#), "{html}");
        assert!(html.contains(r#"href="https://example.com""#), "{html}");
    }
}

//! Markdown to HTML, with comrak (the engine behind Elixir's MDEx).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use comrak::nodes::NodeValue;
use comrak::options::Plugins;
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use comrak::{Arena, Options, format_html_with_plugins, parse_document};

use crate::icons::{SITE_ICONS_DIR, SiteIcons};
use crate::images::Images;
use crate::links::LinkContext;

/// Renders Markdown to HTML.
///
/// Raw HTML in the Markdown is escaped, so it shows as text instead of
/// running. Links and images are rewritten for the built site (see
/// [`LinkContext::resolve`]). Images from `media/` get their smaller
/// copies and their size, like image blocks. `:icon-name:` becomes an
/// icon, the site's own if it has one by that name.
pub fn render(markdown: &str, links: &LinkContext, media: &TextMedia) -> String {
    let options = options();
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &options);

    // What each image from `media/` gets added to its `<img src="…"`.
    let mut image_attributes: BTreeMap<String, String> = BTreeMap::new();
    for node in root.descendants() {
        match node.data.borrow_mut().value {
            NodeValue::Link(ref mut link) => link.url = links.resolve(&link.url),
            NodeValue::Image(ref mut image) => {
                // Only `/media/…`: a relative `photo.png` isn't in media/.
                let picture = image
                    .url
                    .starts_with("/media/")
                    .then(|| crate::generator::picture(&image.url, None, links, media.images));
                image.url = links.resolve(&image.url);
                if let Some(picture) = picture.flatten() {
                    image_attributes.insert(image.url.clone(), picture.attributes(TEXT_SIZES));
                }
            }
            _ => {}
        }
    }

    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(highlighter());
    let mut html = String::new();
    // Writing into a String can't fail.
    let _ = format_html_with_plugins(root, &options, &mut html, &plugins);
    for (src, attributes) in &image_attributes {
        let tag = format!("<img src=\"{src}\"");
        html = html.replace(&tag, &format!("{tag}{attributes}"));
    }
    // Images further down load when they're about to be seen.
    let html = html.replace("<img src=", "<img loading=\"lazy\" src=");
    with_icons(&html, links, media.site_icons)
}

/// What text needs from the site besides links: its own icons and its
/// images, whose sizes and smaller copies the build keeps track of.
pub struct TextMedia<'a> {
    pub site_icons: &'a SiteIcons,
    pub images: &'a Images,
}

/// How wide an image in text is shown at most: the width of the content,
/// like a full-width image block (`full_sizes` in the generator). In a
/// column it's less, so the browser may take a copy larger than needed,
/// never a blurry one.
const TEXT_SIZES: &str = "(min-width: 56rem) 56rem, 100vw";

/// Replaces `:icon-phone:` in text with that icon, sized to the text: the
/// site's own from `media/icons/` (an image, in its own colors) or else
/// q.e.d.'s (see `icons.rs`). Code is left alone, and so are names that
/// are in neither.
fn with_icons(html: &str, links: &LinkContext, site_icons: &SiteIcons) -> String {
    if !html.contains(":icon-") {
        return html.to_string();
    }
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find(":icon-") {
        // Inside <code> or <pre>, write everything up to its end as it is.
        let before = &rest[..start];
        let open_code = ["<code", "<pre"]
            .iter()
            .filter_map(|tag| before.rfind(tag).map(|at| (at, *tag)))
            .max_by_key(|(at, _)| *at);
        if let Some((at, tag)) = open_code {
            let close = if tag == "<pre" { "</pre>" } else { "</code>" };
            if !before[at..].contains(close)
                && let Some(end) = rest[at..].find(close)
            {
                let end = at + end + close.len();
                out.push_str(&rest[..end]);
                rest = &rest[end..];
                continue;
            }
        }
        out.push_str(before);
        let after = &rest[start + ":icon-".len()..];
        let name_len = after
            .find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
            .unwrap_or(after.len());
        let name = &after[..name_len];
        if !after[name_len..].starts_with(':') {
            out.push_str(":icon-");
            rest = after;
            continue;
        }
        // Names are lowercase letters, digits and dashes, so they need no
        // escaping in the attribute.
        if let Some(file) = site_icons.file(name) {
            let src = links.media(&format!("{SITE_ICONS_DIR}/{file}"));
            out.push_str(&format!(
                r#"<img class="inline-icon site-icon" src="{src}" alt="" aria-hidden="true">"#
            ));
        } else if let Some(svg) = crate::icons::svg(name) {
            out.push_str(&svg.replacen("class=\"size-4 shrink-0\"", "class=\"inline-icon\"", 1));
        } else {
            out.push_str(":icon-");
            rest = after;
            continue;
        }
        rest = &after[name_len + 1..];
    }
    out.push_str(rest);
    out
}

/// Code blocks with a language (```rust) get their parts marked with CSS
/// classes, `hl-keyword` and so on; [`code_css`] colors them. The `hl-`
/// prefix keeps syntect's names apart from daisyUI's (both have `link`).
/// Loading the grammars takes a moment, so it happens once.
fn highlighter() -> &'static SyntectAdapter {
    static ADAPTER: OnceLock<SyntectAdapter> = OnceLock::new();
    ADAPTER.get_or_init(|| {
        SyntectAdapterBuilder::new()
            .css_with_class_prefix(CLASS_PREFIX)
            .build()
    })
}

const CLASS_PREFIX: &str = "hl-";

/// The colors for highlighted code, for the site's stylesheet. Code blocks
/// are dark on every theme, so the colors come from a dark scheme.
pub fn code_css() -> &'static str {
    static CSS: OnceLock<String> = OnceLock::new();
    CSS.get_or_init(|| {
        let themes = syntect::highlighting::ThemeSet::load_defaults();
        themes
            .themes
            .get("base16-ocean.dark")
            .and_then(|theme| {
                syntect::html::css_for_theme_with_class_style(
                    theme,
                    syntect::html::ClassStyle::SpacedPrefixed {
                        prefix: CLASS_PREFIX,
                    },
                )
                .ok()
            })
            .map(|css| {
                // The scheme's background is on `.hl-code`, which comrak
                // doesn't write; give every code block those colors.
                format!(
                    "{css}\n.prose pre.syntax-highlighting, pre.syntax-highlighting {{ background-color: #2b303b; color: #c0c5ce; }}\n"
                )
            })
            .unwrap_or_default()
    })
}

/// The icon names written as `:icon-name:` in a piece of Markdown, code
/// included (it errs on the side of a file counting as used).
pub fn icon_names(markdown: &str) -> Vec<&str> {
    markdown
        .split(":icon-")
        .skip(1)
        .filter_map(|after| {
            let len = after
                .find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
                .unwrap_or(after.len());
            (len > 0 && after[len..].starts_with(':')).then_some(&after[..len])
        })
        .collect()
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

/// Replaces link and image targets in Markdown, as written, by the ones
/// in `targets` (old target to new). Everything else stays byte for byte.
///
/// Targets are found where Markdown puts them: after `](` in inline links,
/// in reference definitions (`[team]: about.md`) and in autolinks
/// (`<https://…>`). One pass, so a new target is never replaced again.
pub fn replace_link_targets(markdown: &str, targets: &BTreeMap<String, String>) -> String {
    if targets.is_empty() {
        return markdown.to_string();
    }
    let bytes = markdown.as_bytes();
    let mut out = String::with_capacity(markdown.len());
    let mut copied = 0;
    let mut i = 0;
    while i < bytes.len() {
        // Only ASCII bytes are compared, so every index lands on a
        // character boundary.
        let found = if bytes[i] == b']' && bytes.get(i + 1) == Some(&b'(') {
            destination(markdown, i + 2)
        } else if bytes[i] == b'<' {
            autolink(markdown, i + 1)
        } else if i == 0 || bytes[i - 1] == b'\n' {
            definition(&markdown[i..]).and_then(|offset| destination(markdown, i + offset))
        } else {
            None
        };
        match found {
            Some((from, to)) => {
                if let Some(new) = targets.get(&markdown[from..to]) {
                    out.push_str(&markdown[copied..from]);
                    out.push_str(new);
                    copied = to;
                }
                i = to.max(i + 1);
            }
            None => i += 1,
        }
    }
    out.push_str(&markdown[copied..]);
    out
}

/// The link destination starting at `start`: `about.md` in `(about.md "Title")`,
/// or what's inside `<…>`. As a byte range.
fn destination(text: &str, start: usize) -> Option<(usize, usize)> {
    let rest = text.get(start..)?;
    if let Some(inside) = rest.strip_prefix('<') {
        let end = inside.find(['>', '\n'])?;
        return inside[end..]
            .starts_with('>')
            .then_some((start + 1, start + 1 + end));
    }
    let end = rest.find([' ', '\t', '\n', ')']).unwrap_or(rest.len());
    (end > 0).then_some((start, start + end))
}

/// An autolink's target, starting after its `<`: up to `>`, without spaces.
fn autolink(text: &str, start: usize) -> Option<(usize, usize)> {
    let rest = text.get(start..)?;
    let end = rest.find(['>', ' ', '\t', '\n', '<'])?;
    (end > 0 && rest[end..].starts_with('>')).then_some((start, start + end))
}

/// Where the target of a reference definition (`[label]: target`) starts,
/// if `line` begins with one.
fn definition(line: &str) -> Option<usize> {
    let indent = line.bytes().take_while(|b| *b == b' ').count();
    if indent > 3 || !line[indent..].starts_with('[') {
        return None;
    }
    let close = indent + line[indent..].find("]:")?;
    if line[indent..close].contains('\n') {
        return None;
    }
    let after = close + 2;
    let spaces = line[after..]
        .bytes()
        .take_while(|b| *b == b' ' || *b == b'\t')
        .count();
    Some(after + spaces)
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
        let images = Images::new(Path::new("media"));
        render(
            markdown,
            &LinkContext::new(path, Path::new(file)),
            &TextMedia {
                site_icons: &SiteIcons::default(),
                images: &images,
            },
        )
    }

    #[test]
    fn replaces_only_link_targets() {
        let targets = BTreeMap::from([
            ("/about".to_string(), "/company".to_string()),
            ("about.md#team".to_string(), "company.md#team".to_string()),
            ("/company".to_string(), "/never".to_string()),
        ]);
        let text = "See [us](/about) and [the team](about.md#team \"Team\"),\n\
                    ![logo](</about>) but not /about or `about`.\n\n\
                    [ref]: /about\n";
        assert_eq!(
            replace_link_targets(text, &targets),
            "See [us](/company) and [the team](company.md#team \"Team\"),\n\
             ![logo](</company>) but not /about or `about`.\n\n\
             [ref]: /company\n"
        );
        let unchanged = "Ünïcode [x](/elsewhere) <https://example.com> a<b";
        assert_eq!(replace_link_targets(unchanged, &targets), unchanged);
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
    fn highlights_code_with_a_language() {
        let html = render_on("", "pages/index.md", "```rust\nlet x = \"<b>\";\n```\n");
        assert!(html.contains(r#"<code class="language-rust">"#), "{html}");
        assert!(
            html.contains(r#"<span class="hl-storage hl-type hl-rust">let</span>"#),
            "{html}"
        );
        assert!(html.contains("&lt;b&gt;"), "code is escaped: {html}");
        assert!(code_css().contains(".hl-keyword {"), "{}", code_css());
    }

    #[test]
    fn puts_icons_into_text_but_not_into_code() {
        let html = render_on(
            "",
            "pages/index.md",
            ":icon-phone: +43 1 234\n\n`:icon-phone:` and :icon-nope: stay\n\n```\n:icon-mail:\n```\n",
        );
        assert!(
            html.contains(r#"<p><svg class="inline-icon" aria-hidden="true""#),
            "{html}"
        );
        assert!(html.contains("</svg> +43 1 234</p>"), "{html}");
        assert!(html.contains("<code>:icon-phone:</code>"), "{html}");
        assert!(html.contains(":icon-nope: stay"), "{html}");
        assert!(
            html.contains(":icon-mail:\n</span></code></pre>") || html.contains(":icon-mail:"),
            "{html}"
        );
        assert_eq!(html.matches("<svg").count(), 1, "{html}");
    }

    #[test]
    fn prefers_the_sites_own_icons() {
        let media = tempfile::tempdir().unwrap();
        let folder = media.path().join(SITE_ICONS_DIR);
        std::fs::create_dir(&folder).unwrap();
        std::fs::write(folder.join("rails.svg"), "<svg/>").unwrap();
        std::fs::write(folder.join("phone.png"), "png").unwrap();
        let images = Images::new(media.path());
        let html = render(
            ":icon-rails: Rails, :icon-phone: and :icon-mail:",
            &LinkContext::new("stack", Path::new("pages/stack.md")),
            &TextMedia {
                site_icons: &SiteIcons::load(media.path()),
                images: &images,
            },
        );
        assert!(
            html.contains(
                r#"<img class="inline-icon site-icon" src="../media/icons/rails.svg" alt="" aria-hidden="true"> Rails"#
            ),
            "{html}"
        );
        assert!(html.contains(r#"src="../media/icons/phone.png""#), "{html}");
        assert_eq!(html.matches("<svg").count(), 1, "mail is q.e.d.'s: {html}");
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

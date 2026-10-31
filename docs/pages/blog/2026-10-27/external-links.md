+++
title = "Links to elsewhere"
description = "Marking links to other sites: an icon from CSS alone, and a builder method that takes self"
menu = "main"
weight = 36
date = 2026-10-27
authors = ["Claude"]
tags = ["css", "rust", "links"]
summary = "Links in text that lead to other sites can now get a small external-link icon (#116). It needs no change to the HTML of a link: CSS finds them by their address. And full links to the site itself now count as internal, with a builder method on LinkContext."
redirect_from = ["/blog/2026-10-27-external-links"]
+++

A reader can't tell from an underline whether a link stays on the site or
leaves it. Many sites add a small "arrow out of a box" after the ones that
leave. q.e.d. now does that too, as a switch in Site settings,
**Mark links to other sites** (#116). New sites start with it on;
existing ones keep their look until it's switched on.

## Finding the links with CSS

q.e.d. writes every link inside a site as a relative one: `../about/`,
`blog/post/`. So in the built HTML, a link that starts with `http://` or
`https://` can only lead elsewhere. That means the HTML of the links
doesn't need to change at all; one rule in
`crates/qed-core/assets/site.input.css` finds them:

```css
.mark-external .prose a:is([href^="http://"], [href^="https://"]):not(:has(img, svg))::after {
  content: "";
  display: inline-block;
  width: 0.75em;
  height: 0.75em;
  background-color: currentColor;
  mask: url("data:image/svg+xml,…") center / contain no-repeat;
}
```

Read from left to right:

- **`.mark-external`** is a class on `<body>`, written by the page
  template only when the switch is on
  (`{% if site.mark_external_links %}`). Without it, the rule matches
  nothing.
- **`.prose a`**: links inside text. Menus and buttons aren't in
  `.prose`, so they stay as they are.
- **`[href^="https://"]`** is an
  [attribute selector](https://developer.mozilla.org/en-US/docs/Web/CSS/Attribute_selectors):
  `^=` means "starts with". [`:is(…)`](https://developer.mozilla.org/en-US/docs/Web/CSS/:is)
  accepts either of the two.
- **`:not(:has(img, svg))`** leaves out links that contain an image or
  an icon, such as a linked photo.
  [`:has()`](https://developer.mozilla.org/en-US/docs/Web/CSS/:has) looks
  *inside* an element, something CSS couldn't do for a long time.
- **`::after`** adds a box after the link's text that isn't in the HTML.

## An icon from a mask

The box gets the link's colour as its background (`currentColor`) and
Lucide's external-link drawing as its
[`mask`](https://developer.mozilla.org/en-US/docs/Web/CSS/mask): only the
parts of the box under the drawing's lines stay visible. So the icon
always has the colour of the link it belongs to, also on hover or in a
dark footer, without one SVG per colour.

The drawing itself is written into the stylesheet as a
[`data:` URL](https://developer.mozilla.org/en-US/docs/Web/URI/Reference/Schemes/data):
the SVG's text, with `<`, `>` and `#` encoded (`%3C`, `%3E`, `%23`), so
no extra file has to be loaded.

## Full links to the site itself

podcasterei.at has links written as `https://www.podcasterei.at/…`, to
its own pages. They'd get the icon although they don't leave the site,
and in the local preview they lead to the live site instead of the
preview. Now the generator turns them into relative links, like any
other internal link. `LinkContext` (`crates/qed-core/src/links.rs`)
learned the site's address with a new method:

```rust
pub fn with_own_site(mut self, base_url: Option<&str>) -> Self {
    self.own_site = base_url.map(str::to_string);
    self
}
```

and the generator uses it where a page is rendered:

```rust
let links = LinkContext::new(&page.path, &page.file)
    .with_own_site(config.base_url.as_deref());
```

This is how a [builder](../2026-10-21/new-pages.md) method is written. It
takes `self` *by value*, not `&self` or `&mut self`: it consumes the
context, changes it and hands it back. The `mut` in `mut self` only says
the function may change its own copy, just like `mut` on any parameter.
Because it returns `Self`, the call chains straight after `new`, and the
`let` gets the finished context. The rest of the code, which never cares
about the site's address, keeps calling `LinkContext::new` as before.

The Elixir counterpart is a pipeline over a struct:

```elixir
links = LinkContext.new(page.path, page.file) |> LinkContext.with_own_site(config.base_url)
```

In Elixir every function returns a new value anyway. Rust offers both
ways, and `self` by value is how you say "this returns the changed
value". The compiler then makes sure nobody uses the old one by
mistake: it has been moved into the call.

Whether a full link is "own" is decided by `own_path`: same host
(ignoring `www.` and upper case), over http or https, and below the
site's folder if it lives in one, such as `https://example.com/podcast/`.
`https://example.com/podcaster` is *not* inside `/podcast`, which is why
it checks that the rest begins with `/`, `?` or `#`.

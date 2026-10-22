+++
title = "Link previews"
description = "Link previews: Open Graph tags, the first image with find_map, and pub(crate)"
menu = "main"
weight = 25
date = 2026-10-22
authors = ["Claude"]
tags = ["rust", "generator", "html"]
summary = "Shared links to a q.e.d. site now show a title, a description and an image. What Open Graph tags are, why the image needs a full address, how find_map picks the first image on a page, and what pub(crate) means."
+++

Paste a link into a chat app or a social network, and it usually turns
into a small card: a picture, a title, a line of text. Until now links to
q.e.d. sites stayed bare. Every page now tells those apps what to show.

Each page has a new setting, **Share image**, in its Settings tab. Leave
it empty and the page's first image is used: a hero picture, an image
block, a text-with-image block or the first photo of a gallery, columns
included. A page without any image gets a card without a picture.

## Open Graph

The card's contents come from `<meta>` tags in the page's `<head>`, a
format Facebook started in 2010 called
[Open Graph](https://ogp.me/). Almost every app that shows previews
reads it. A post on this site now starts like this:

```html
<meta property="og:title" content="Link previews">
<meta property="og:site_name" content="q.e.d.">
<meta property="og:type" content="article">
<meta property="og:description" content="Link previews: Open Graph tags, …">
<meta property="og:url" content="https://…/blog/2026-10-22-link-previews/">
<meta property="og:image" content="https://…/media/cover.1600w.jpg">
<meta property="og:image:width" content="1600">
<meta property="og:image:height" content="900">
<meta name="twitter:card" content="summary_large_image">
```

The tags are written by
[`templates/page.html`](../../../crates/qed-core/templates/page.html),
from fields that `render_page` in
[`generator.rs`](../../../crates/qed-core/src/generator.rs) fills in.
Some details:

- `og:type` is `article` for posts (pages with a date) and `website` for
  everything else.
- `twitter:card` asks for the large picture instead of a small square.
  X reads the `og:` tags for everything else.
- Width and height let an app lay out the card before it has loaded the
  picture.

## Why the image needs the web address

Inside the site, q.e.d. writes relative links (`../media/cover.jpg`), so
a built site works from any folder and on any server. A preview is
different: the chat app fetches the page, then fetches the picture *on
its own*, without knowing where the page was. It needs the full address,
`https://example.com/media/cover.jpg`.

That full address is only known when the site has a **web address** in
Site settings (`base_url` in `site.toml`). Without it, pages still get
the title, type and description tags, but no image and no `og:url`. The
sitemap and the RSS feeds have the same rule, for the same reason.

Large preview cards want a picture at least 1200 px wide; a smaller one
looks blurry. The responsive images already come in 480, 960 and 1600 px
copies (see [Responsive images](2026-10-21-responsive-images.md)), so the
preview takes the smallest copy that is wide enough: the 1600 px one for
a large photo from a camera. An image between 1200 and about 1800 px gets
no 1600 px copy, and its 960 px copy is too small, so the original is
used. In code:

```rust
const SHARE_IMAGE_WIDTH: u32 = 1200;

match info.widths.iter().find(|&&width| width >= SHARE_IMAGE_WIDTH) {
    Some(&width) => /* that copy */,
    None => /* the original */,
}
```

`find` returns a reference to the first element that passes the test, so
`|&&width|` unpacks two layers: the iterator gives `&u32`, and `find`
hands the test a reference to that, `&&u32`. The copy's height comes from
the same function the resizer uses, `scaled_height`, so the number in the
tag matches the file.

## The first image, with `find_map`

Finding the first image means: go through the blocks (the ones inside
columns too), and for each one ask "do you have an image?"; stop at the
first yes. That is exactly what
[`Iterator::find_map`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.find_map)
does:

```rust
fn first_image(blocks: &[Block]) -> Option<(&str, Option<&str>)> {
    block::all_blocks(blocks)
        .into_iter()
        .find_map(|block| match block {
            Block::Hero(hero) => hero.image.as_deref().map(|image| (image, None)),
            Block::Image(image) => image
                .image
                .as_deref()
                .map(|name| (name, image.alt.as_deref())),
            // … Text with image and Gallery the same way
            _ => None,
        })
        .filter(|(name, _)| !name.trim().is_empty())
}
```

The closure returns an `Option` for each block: `Some((file, alt))` when
the block has an image, `None` when it doesn't. `find_map` calls it block
after block and returns the first `Some`. It stops there, so the blocks
after it aren't looked at.

In Elixir this is
[`Enum.find_value/2`](https://hexdocs.pm/elixir/Enum.html#find_value/2):
the function returns `nil` to go on and anything else to stop. Rust's
version uses `Option` instead of `nil`, so the type says that there may
be no result, and the compiler makes the caller handle that.

Two small helpers do a lot of the work:

- `as_deref()` turns an `&Option<String>` (a reference to an optional
  owned string) into an `Option<&str>` (an optional borrowed one), so no
  text is copied.
- `.map(…)` on an `Option` changes the value inside a `Some` and leaves a
  `None` alone, like `Enum.map` on a list with zero or one element.

The setting in the page's front matter wins over the search. One `match`
says so, and `?` ends the function with `None` if the search finds
nothing:

```rust
let (name, alt) = match page.content.front.image.as_deref() {
    Some(name) => (name, None),
    None => first_image(&page.content.blocks)?,
};
```

## `pub(crate)`

The preview needs `scaled_height` from
[`images.rs`](../../../crates/qed-core/src/images.rs), which was private
to that file (Rust calls a file a *module*). Making it `pub` would add it
to qed-core's public API, the part the app and other crates may use.
`pub(crate)` is the step in between: visible everywhere inside qed-core,
invisible outside it.

```rust
pub(crate) fn scaled_height(width: u32, height: u32, new_width: u32) -> u32 {
```

Elixir has no direct match: a module's functions are `def` (public to
everyone) or `defp` (private to the module); `@doc false` only hides a
function from the docs. Rust has more steps; the
[Rust reference on visibility](https://doc.rust-lang.org/reference/visibility-and-privacy.html)
lists them all.

## The image counts as used

The media library won't delete a file that a page still uses. The share
image is a new place where a page can use a file, so `media::references`
now looks at it too. Without that, a share image could be deleted while
its page still points to it. A test, `counts_the_share_image_as_used`,
checks this.

## Links

- [The Open Graph protocol](https://ogp.me/)
- [`Iterator::find_map`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.find_map)
- [`Option::as_deref`](https://doc.rust-lang.org/std/option/enum.Option.html#method.as_deref)
- [Elixir `Enum.find_value/2`](https://hexdocs.pm/elixir/Enum.html#find_value/2)
- [Rust reference: visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html)

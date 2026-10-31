+++
title = "A Result as a list"
description = "Site icons: reading a folder that may not exist, with a Result that iterates like a list of zero or one items"
menu = "main"
weight = 35
date = 2026-10-27T17:56:00
authors = ["Claude"]
tags = ["rust", "iterators", "error-handling"]
summary = "A site can now bring its own icons: image files in media/icons/ that work like :icon-phone:. Reading that folder shows a small Rust trick: Result and Option are iterators of zero or one items, so flatten() skips what failed."
+++

Since icons in text (#114), `:icon-phone:`
in any Markdown becomes one of q.e.d.'s built-in Lucide icons. Those are
plain line drawings in the text colour. The old informatom.com had
something else: the coloured logos of Rails, Elixir, Python and friends
next to the text. They're no icons q.e.d. could ship, so now a site can
bring its own (#115): put `rails.svg` into `media/icons/` and write
`:icon-rails:`.

## Reading a folder that may not be there

Most sites have no `media/icons/` at all, and that's fine: they simply
have no site icons. `SiteIcons::load` in
`crates/qed-core/src/icons.rs` starts like this:

```rust
let entries = std::fs::read_dir(media_dir.join(SITE_ICONS_DIR))
    .into_iter()
    .flatten()
    .flatten();
for entry in entries {
    let file = entry.file_name().to_string_lossy().into_owned();
    // …
}
```

Two `flatten()`s in a row look odd. Here is what each layer is:

1. [`read_dir`](https://doc.rust-lang.org/std/fs/fn.read_dir.html) returns
   `Result<ReadDir, io::Error>`: the folder's listing, or an error (for
   example when the folder doesn't exist).
2. `ReadDir` is an iterator, but of `Result<DirEntry, io::Error>`: each
   entry can fail on its own.

The trick is that a
[`Result` can be turned into an iterator](https://doc.rust-lang.org/std/result/enum.Result.html#impl-IntoIterator-for-Result%3CT,+E%3E):
it yields the `Ok` value once, or nothing at all for an `Err`. `Option`
works the same way, with `Some` and `None`. So:

- `.into_iter()` turns the outer `Result` into an iterator of zero or one
  `ReadDir`s.
- The first `.flatten()` opens that `ReadDir`, so we get its entries, each
  still a `Result`. If the folder was missing, there is nothing to open
  and the iterator is empty.
- The second `.flatten()` does the same to each entry: the good ones come
  through as `DirEntry`, the broken ones disappear.

No `match`, no early return, and a missing folder needs no special case.
This only fits where an error really means "nothing here". q.e.d.'s rule
is to return a typed error from `qed-core`, and most file reading does
(`SiteError::Io` with the path). For optional icons, a folder that can't
be read just means no site icons, so swallowing the error is the honest
behaviour.

In Elixir you'd write roughly:

```elixir
case File.ls(Path.join(media_dir, "icons")) do
  {:ok, files} -> files
  {:error, _} -> []
end
```

Rust's version reads like `{:ok, x}` being a list `[x]` and
`{:error, _}` being `[]`, and then
[`Enum.concat`](https://hexdocs.pm/elixir/Enum.html#concat/1) on top.

## The best file per name

An icon might exist as `rails.png` and `rails.svg`. SVG stays sharp at
any size, so it wins. The extensions are listed best first:

```rust
pub const SITE_ICON_EXTENSIONS: [&str; 5] = ["svg", "webp", "png", "jpg", "jpeg"];
```

For each file, the position of its extension in that list is its rank,
found with
[`Iterator::position`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.position).
A file only replaces the one already kept for that name when its rank is
lower, checked with `is_none_or`, which you may remember from the
[preview server](2026-10-21-from-folder-to-website.md): "no icon yet, or
a worse one".

## An image, not inlined SVG

q.e.d.'s own icons are written straight into the HTML as `<svg>`, so they
take the text colour. Site icons become an `<img>` instead:

```html
<img class="inline-icon site-icon" src="../media/icons/rails.svg" alt="" aria-hidden="true">
```

That keeps their own colours, and it's safe with any SVG: two inlined
logos could both use the same `id` inside them and break each other,
which can't happen across images. The `alt` is empty because the text
next to the logo already says "Ruby on Rails"; a screen reader would only
say it twice.

## How big is an icon?

The logos come in all sizes: Rails is drawn on a 24 × 24 grid, Nerves
says `width="53" height="43"`, Fiori `198 × 207`. Yet on the page they
all stand the same height next to the text. The stylesheet
(`crates/qed-core/assets/site.input.css`) sets only one of the two
sizes:

```css
.inline-icon.site-icon {
  width: auto;
  height: 1.5em;
  vertical-align: middle;
}
```

- **`em` is the font size of the surrounding text.** In a paragraph
  of 16 px, `1.5em` is 24 px; in a heading of 24 px it's 36 px. So an
  icon grows and shrinks with the text it stands in, without anyone
  telling it how big that text is.
  ([MDN on `em`](https://developer.mozilla.org/en-US/docs/Learn_web_development/Core/Styling_basics/Values_and_units#relative_length_units))
- **CSS beats the file's own size.** A `width` and `height` written in
  the SVG are only the image's natural size, what it gets when nothing
  else is said. The stylesheet's `height` overrides it.
- **`width: auto` keeps the proportions.** Every SVG has a
  [`viewBox`](https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Attribute/viewBox),
  the rectangle its drawing lives in, and from it the browser knows the
  logo's aspect ratio. Given only the height, it works out the width: the
  square Rails logo becomes 24 × 24, the wider Nerves logo
  24 × 30. A PNG works the same way, from its pixel dimensions.
- **`vertical-align: middle`** puts the middle of the icon at the middle
  of the small letters, so a logo taller than the letters doesn't push
  the line's text down to its bottom edge.

q.e.d.'s own Lucide icons are `1em` square instead: they are all drawn
on the same 24 × 24 grid, as thin lines meant to sit among the letters,
while a coloured logo needs a little more room to be recognised.

One more place needed to know about them: the media library decides
whether a file may be deleted by looking for pages that use it. A page
writing `:icon-rails:` now counts as using `media/icons/rails.svg`, so
the logo can't vanish from under the start page.

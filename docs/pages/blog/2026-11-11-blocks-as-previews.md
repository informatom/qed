+++
title = "Blocks as previews"
description = "Blocks as previews in the editor: the generator's HTML in the app's colors, collecting Results into a String, a slice of one, a Svelte class with private fields, and HTML that can't be clicked"
menu = "main"
weight = 79
date = 2026-11-11T12:00:00
authors = ["Claude"]
tags = ["rust", "svelte", "typescript", "css"]
summary = "A page with many blocks used to be a wall of forms. Now each block shows as it looks on the site, and a click opens its form (#155). The preview is the generator's own HTML, drawn in the app's colors. On the way: collecting Results into a String, std::slice::from_ref, a Svelte class with private fields, and the inert attribute."
+++

Open a page with a dozen blocks, and the Content tab used to show a
dozen forms: text editors, image pickers, input fields, one below the
other. Finding the paragraph you wanted to change meant reading field
labels. Now every block shows as a preview, the way it looks on the
site. A click on it, or on **Edit** at its top left, swaps in the form,
and **Done** (or Esc) brings the preview back (#155).

## The generator draws, the app colors

There were two ways to draw a preview. The app could have a small
Svelte view for each block type: a thumbnail for an image, a list of
questions for an FAQ. Or it could ask the generator, which already knows
how to turn every block into HTML. The second way shows the real thing,
Post lists and Latest episode included, and it can't drift from the
site, because it *is* the site's code.

The catch is the stylesheet. The generated HTML uses Tailwind and
daisyUI classes (`hero`, `card`, `prose`, `md:grid-cols-2`), and a
site's stylesheet would bring the site's theme along. The choice was to
use the app's colors instead, so the app needs no second stylesheet. It
uses the same two libraries, so all it has to do is generate the classes
the templates use. Tailwind only writes rules for classes it finds in
the files it scans, so `src/app.css` now names the templates as a source
as well:

```css
@source "../crates/qed-core/templates";
@source "../crates/qed-core/src/generator.rs";
@plugin "@tailwindcss/typography";
@import "../crates/qed-core/assets/blocks.css";
```

The few rules written by hand that blocks need (underlined links in
text, keycaps, icons in text, charts) moved from the site's
stylesheet into `crates/qed-core/assets/blocks.css`, which both
stylesheets import. The rebuilt `site.css` has exactly the same rules
as before. See Tailwind's
[detecting classes in source files](https://tailwindcss.com/docs/detecting-classes-in-source-files).

## Rust: one HTML string per block

The new function `preview_blocks` in
`crates/qed-core/src/generator.rs` builds the same `Context` the site
build uses (the pages for Post lists, the feeds, the images) and renders
each block. Building the feeds used to sit inside `build_on`. It moved
into a function of its own, `site_feeds`, so both can use it.

A Shared block is a small puzzle. On the site it becomes its piece's
blocks, possibly several of them, but the app wants exactly one preview
per block. So each block is expanded on its own, and the HTML of the
results is joined:

```rust
blocks
    .iter()
    .map(|block| {
        crate::shared::expand_blocks(site, std::slice::from_ref(block), language)
            .iter()
            .map(|block| render_html(block, &links, &context, false))
            .collect::<Result<String, _>>()
    })
    .collect()
```

Two things are worth a closer look.

[`std::slice::from_ref`](https://doc.rust-lang.org/std/slice/fn.from_ref.html)
turns a `&Block` into a `&[Block]` of length one, without copying.
`expand_blocks` takes a slice, because pages hand it all their blocks.
Writing `&[block.clone()]` would also work, but it copies the whole
block, columns and all, just to look at it. `from_ref` works because a
single value in memory already is a valid slice of length one.

The inner `collect::<Result<String, _>>()` does two jobs at once.
`render_html` returns a `Result<String, askama::Error>` for each block.
Collecting an iterator of Results into a `Result` stops at the first
error and returns it. If there's none, it collects the `Ok` values into
whatever comes inside, and a `String` can be collected from strings: it
concatenates them. The outer `collect()` does the same into
`Result<Vec<String>, _>`, a type the function's signature gives, so it
needs no turbofish. In Elixir you'd write a `reduce_while` that stops on
`{:error, _}`. In Rust it's a type annotation. The trait behind it is
[`FromIterator`](https://doc.rust-lang.org/std/iter/trait.FromIterator.html#impl-FromIterator%3CResult%3CA,+E%3E%3E-for-Result%3CV,+E%3E).

The links start at the site root (`LinkContext::rooted("/")`), so a
picture comes out as `/media/photo.jpg`. The Tauri command
`preview_blocks` in `src-tauri/src/lib.rs` just opens the site and calls
it.

## Svelte: a class that remembers

On the app side, `src/lib/blockPreviews.svelte.ts` holds a class:

```ts
export class BlockPreviews {
  forms = $state(readForms());
  #cache = new SvelteMap<string, Preview>();
  #loading = new Set<string>();
  …
}
```

A `$state` field in a class is reactive like a `$state` variable in a
component: the switch **All blocks as forms** reads and writes
`previews.forms`, and every block list follows. That only works in files
Svelte compiles, so the file name ends in `.svelte.ts`. See
[`$state` in classes](https://svelte.dev/docs/svelte/$state#Classes).

The `#` fields are JavaScript's own
[private fields](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Private_properties).
Nothing outside the class can read them, not even with a type cast.
TypeScript's `private` keyword is only checked by the compiler and
disappears from the JavaScript it outputs. The cache is keyed by the
block's content as JSON, so equal blocks share one preview, and a block
that changed gets a fresh one. It's a `SvelteMap`, so a preview that
arrives makes the waiting block redraw.

The BlockEditor makes one `BlockPreviews` and puts it into Svelte's
context. The block lists inside Columns and Sections find it there, the
same way they find the shared blocks (see
[Shared blocks](2026-10-27-shared-blocks.md)).

Which blocks are open is a `SvelteSet<Block>`, keyed by the block
objects themselves. There's one trap here. When a new block is added and
should start open, this doesn't work:

```ts
const block = newBlock(type);
blocks.push(block);
open.add(block); // never matches
```

`blocks` is `$state`, and Svelte wraps every object put into it in a
[proxy](https://svelte.dev/docs/svelte/$state#Deep-state) that watches
its changes. The list then holds the proxy, not `block`, so the set
would hold an object the list never shows. The code takes the block back
out of the list, `open.add(blocks[blocks.length - 1])`, and gets the
proxy.

## HTML that can't be clicked

The generator's HTML goes through a
[`<template>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/template)
element before it's shown. A template parses HTML without loading
anything, so the code can fix it first:

- `/media/photo.jpg` becomes an address in the site folder through
  Tauri's `convertFileSrc`, like everywhere else in the app.
- `srcset` goes, because the smaller copies of a picture only exist in a
  built site.
- `<script>` goes. `{@html}` wouldn't run scripts anyway, so the Podlove
  player doesn't start and the plain audio player shows in its place.

A preview is full of links and buttons, and none of them should do
anything: a click anywhere should open the form. The HTML attribute
[`inert`](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/inert)
does exactly that for everything inside an element. Nothing in it can be
clicked, focused or reached with Tab. The click passes through to the
surrounding element, which opens the form. That element is a `div` with
`role="button"` and `tabindex="0"`, so the keyboard can reach it too:
Enter or Space opens the block.

## Further reading

- Tailwind: [Detecting classes in source files](https://tailwindcss.com/docs/detecting-classes-in-source-files)
- Rust: [`std::slice::from_ref`](https://doc.rust-lang.org/std/slice/fn.from_ref.html),
  [collecting into a `Result`](https://doc.rust-lang.org/std/result/index.html#collecting-into-result),
  [`FromIterator`](https://doc.rust-lang.org/std/iter/trait.FromIterator.html)
- Svelte: [`$state` in classes and deep state](https://svelte.dev/docs/svelte/$state),
  [`{@html}`](https://svelte.dev/docs/svelte/@html),
  [context](https://svelte.dev/docs/svelte/context)
- MDN: [private class fields](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Private_properties),
  [`<template>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/template),
  [`inert`](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/inert)

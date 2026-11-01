+++
title = "Where to cut a picture"
description = "Choosing where a picture is cut to a square: CSS object-position, Svelte's style: directive, a click turned into percent, and media.toml"
menu = "main"
weight = 77
date = 2026-11-01T18:22:00
authors = ["Claude"]
tags = ["css", "svelte", "rust", "toml"]
summary = "Galleries cut pictures to squares around the middle. For a portrait that loses its head, the Media tab now lets you click the spot that must stay visible (#152). How CSS decides what to cut, how a click becomes a percentage, and why the setting lives in its own file."
+++

A Gallery shows its pictures as squares, and post lists show tiles. A
photo that isn't square has to lose something, and until now that was
always the edges: the square was cut from the middle. That's right for
most pictures, but not for a portrait whose head is near the top. Now
you can choose the spot for each picture in the Media tab (#152).

## The browser does the cutting

The picture files aren't cut at all. The template gives the `<img>` a
square box and tells the browser to fill it:

```html
<img src="media/portrait.jpg" class="aspect-square object-cover"
     style="object-position: 50% 15%">
```

`object-fit: cover` (Tailwind's `object-cover`) scales the picture until
it covers the whole box and lets the rest stick out, hidden.
[`object-position`](https://developer.mozilla.org/en-US/docs/Web/CSS/object-position)
says which part stays in view. Its default is `50% 50%`, the middle.
`50% 15%` doesn't mean "move it by 15%". It lines up the point 15% down
the picture with the point 15% down the box. So `0%` shows the top edge,
`100%` the bottom edge, and anything in between slides smoothly. That's
why a focus in percent works for any picture size and any box.

Because only the browser cuts, one setting is enough for every place
the picture shows up: the Gallery square, the tile in a post list, the
wide Hero. `picture()` in `crates/qed-core/src/generator.rs` looks up
the focus and passes it to the template as text (`"50% 15%"`). The `img`
macro in `crates/qed-core/templates/block.html` writes the `style` only
when there is one, so pictures cut around the middle stay as they were.

## From a click to a percentage

In the app, the big image view (`src/lib/ImageViewer.svelte`) has a
**Crop** button. In crop mode, a click on the picture sets the spot:

```ts
const box = img.getBoundingClientRect();
const x = Math.round(((event.clientX - box.left) / box.width) * 100);
```

[`getBoundingClientRect()`](https://developer.mozilla.org/en-US/docs/Web/API/Element/getBoundingClientRect)
says where the picture is on the screen right now, at whatever size it's
shown. `clientX` is where the mouse was, measured the same way. The
difference divided by the width is the share of the picture to the left
of the click. It's the same number whether the picture is shown at 400
or 1400 pixels, which is what CSS wants.

The square beside it uses Svelte's
[`style:` directive](https://svelte.dev/docs/svelte/style):

```svelte
<img class="aspect-square object-cover" style:object-position={position} />
```

`style:object-position={position}` sets one CSS property from a value
and updates it when `position` changes. Without it you would build a
whole `style="…"` string. In Elixir's HEEx you would write
`style={"object-position: #{@position}"}`, which is the same string
building. Here Svelte keeps the other styles alone and changes only this
one. The cards in the Media tab use it too, so their thumbnails are cut
like the site.

## A file of its own

The setting belongs to the picture, not to one block that uses it, so
it can't go into a page. The site's settings are in `site.toml`, but that
file is for the Settings tab and the assistant's `write_settings`, and
a list of pictures would bury the settings. So there's a new file,
`media.toml`, written only by the Media tab
(`crates/qed-core/src/media_settings.rs`):

```toml
["portrait.jpg"]
focus = { x = 50, y = 15 }
```

The table names are the picture names in quotes, because a name with a
dot like `portrait.jpg` would otherwise mean a table `jpg` inside a
table `portrait`. Reading the file uses `toml` and serde, as for
`site.toml`. Writing uses `toml_edit` (see
[Saving without breaking](2026-10-21-saving-without-breaking.md)), so a
comment or a future setting next to the focus stays put when one
picture changes. When a picture is deleted, its entry goes too, and when
the last entry goes, so does the file.

Choosing the middle again doesn't write `{ x = 50, y = 50 }`. It removes
the entry. That's what a focus of `None` means in Rust, and
[`Option::filter`](https://doc.rust-lang.org/std/option/enum.Option.html#method.filter)
turns "the middle" into `None` in one line:

```rust
let focus = focus.filter(|focus| *focus != Focus::MIDDLE);
```

It keeps the value only if the test holds, much like Elixir's
`Enum.filter/2` on a list of at most one element.

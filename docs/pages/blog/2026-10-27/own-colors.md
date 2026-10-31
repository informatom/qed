+++
title = "Own colors"
description = "Own colors: a daisyUI theme is a set of CSS variables, so a custom theme is one more rule, with text colors picked for contrast"
menu = "main"
weight = 28
date = 2026-10-27
authors = ["Claude"]
tags = ["css", "rust", "svelte", "themes"]
summary = "Sites can have their own colors now: start from any of the 35 themes, change five colors and the corners. Why that's a single CSS rule, how q.e.d. picks readable text on each color, and how the app reads a theme's colors out of the browser."
redirect_from = ["/blog/2026-10-27-own-colors"]
+++

Until now a site had one of daisyUI's 35 themes, nothing in between. Now
Site → Theme has a *Custom colors* card. It starts from a theme you pick,
lets you change the background, primary, secondary, accent and neutral
colors and the corners, and saves on every change. In `site.toml` it looks
like this:

```toml
theme = "custom"

[custom_theme]
from = "light"
primary = "#f26522"
accent = "#1a1a1a"
corners = "sharp"
```

## A theme is a set of CSS variables

In daisyUI 5 a theme isn't a stylesheet of its own. It's a rule that sets
[CSS custom properties](https://developer.mozilla.org/en-US/docs/Web/CSS/Using_CSS_custom_properties),
and every component reads them:

```css
[data-theme=light] {
  --color-primary: oklch(45% 0.24 277);
  --color-primary-content: oklch(93% 0.03 272);
  --radius-box: 0.5rem;
  /* … about 28 in all */
}
.btn-primary { background-color: var(--color-primary); }
```

`site.css` already holds all 35 themes. So a custom theme needs no new
build of the stylesheet, only one more rule that sets a few variables
again. `CustomTheme::css` in `crates/qed-core/src/theme.rs` writes it, and
the generator appends it to `style.css`:

```css
:root[data-theme=light] {
  --color-primary: #f26522;
  --color-primary-content: #1d232a;
  /* … */
}
```

The page keeps `data-theme="light"`, so every variable the custom theme
doesn't set still comes from *light*. Why does this rule win over light's
own? [Specificity](https://developer.mozilla.org/en-US/docs/Web/CSS/Specificity):
`[data-theme=light]` is one attribute selector, `:root[data-theme=light]`
is that plus a pseudo-class, so it counts as more specific, whatever
order the rules come in.

## Readable text on any color

Each color in a daisyUI theme has a partner for text on it
(`--color-primary-content`). Asking for both would double the settings,
and it's easy to get wrong. So q.e.d. picks it: near-black or white,
whichever contrasts more. Contrast is defined by the
[WCAG](https://www.w3.org/TR/WCAG21/#dfn-contrast-ratio) on the relative
luminance of the color:

```rust
fn is_light(rgb: [u8; 3]) -> bool {
    let l = luminance(rgb);
    // Contrast with black against contrast with white.
    (l + 0.05) / 0.05 > 1.05 / (l + 0.05)
}
```

`luminance` undoes the sRGB gamma per channel and weighs green most,
because the eye is most sensitive to it. The function takes the array by
pattern, `fn luminance([r, g, b]: [u8; 3])`. Rust can take a pattern
apart right in the parameter list, much like an Elixir function head
matching `{r, g, b}`.

The background gets two darker shades for boxes and borders
(`base-200`, `base-300`). Here the browser does the mixing, with
[`color-mix`](https://developer.mozilla.org/en-US/docs/Web/CSS/color_value/color-mix):
`color-mix(in oklab, #fdf6e3, black 5%)`. On a dark background it mixes
with white instead.

## The same code for the build and the preview

The legend beside the theme list previews the colors while you drag the
color picker. Instead of a second copy of the contrast logic in
TypeScript, the app asks Rust: the command `custom_theme_variables` calls
the same `CustomTheme::variables` that the build uses. The legend puts the
result into an inline `style`, which beats the theme's rule just like
`:root[…]` does on the site.

The color picker (`<input type="color">`) has two events: `input` while
dragging, which updates the preview, and `change` when it closes, which
saves. Saving on every `input` would write `site.toml` dozens of times a
second.

## Reading a theme's colors out of the browser

*Start from nord* fills in nord's five colors so you can change them. But
the themes define their colors in `oklch(…)`, and the color picker only
understands `#rrggbb`. The app lets the browser convert: it reads the
variable from an element with `data-theme="nord"`, paints one pixel with
it on a canvas, and reads the pixel back
([`getImageData`](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/getImageData)):

```ts
context.fillStyle = getComputedStyle(probe).getPropertyValue("--color-primary");
context.fillRect(0, 0, 1, 1);
const [r, g, b] = context.getImageData(0, 0, 1, 1).data;
```

No color library, and it handles any color the browser can draw.

## A table, not a line

[`toml_edit`](https://docs.rs/toml_edit/) turned the nested struct into
an inline table: `custom_theme = { from = "light", primary = "#f26522", … }`.
Valid TOML, but one long line is hard to edit by hand. `save_config`
converts it into a real `[custom_theme]` section before merging it into
the file:

```rust
if let Some(inline) = document
    .get_mut("custom_theme")
    .and_then(|item| item.as_inline_table_mut())
{
    let table = std::mem::take(inline).into_table();
    document["custom_theme"] = toml_edit::Item::Table(table);
}
```

[`std::mem::take`](https://doc.rust-lang.org/std/mem/fn.take.html) moves
the inline table out from behind the `&mut` and leaves an empty one in its
place. A mutable reference can't be moved out of directly, because the
document still owns the value.

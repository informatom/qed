+++
title = "Self-hosted fonts"
description = "Self-hosted fonts: Google Fonts without Google, @font-face with unicode-range, and a font preview in the app"
menu = "main"
weight = 30
date = 2026-10-27
authors = ["Claude"]
tags = ["css", "fonts", "rust", "svelte", "privacy"]
summary = "A site can pick fonts now, from the Google Fonts and other open fonts. q.e.d. downloads them once into the site folder, and the website serves them itself, so no visitor's browser asks Google. How the files are chosen, how @font-face and unicode-range work, and how the app previews a font before the site is built."
+++

Site → Theme has a *Fonts* card now: one font for text and, if you
like, another for headings. The list holds about 2,100 fonts, most of
them the Google Fonts. In `site.toml`:

```toml
[fonts]
text = "Ubuntu"
headings = "Playfair Display"
```

## Google Fonts without Google

Linking a font from `fonts.googleapis.com` is the easy way, but every
page view then tells Google who's visiting. In 2022 a court in Munich
ordered a website owner to pay a visitor damages for exactly that
([LG München I, 3 O 17493/20](https://www.theregister.com/2022/01/31/website_fine_google_fonts_gdpr/)). The fonts themselves are under open licenses
(mostly the SIL Open Font License), which allow copying them onto your
own server. That's what q.e.d. does.

The files come from [Fontsource](https://fontsource.org), which packages
the Google Fonts (and some more) as plain `.woff2` files, with a JSON API
that describes each font: its weights, styles, character subsets and
their Unicode ranges. Google's own CSS service is harder to use for
this: it answers each browser with different files.

`fonts::download` in `crates/qed-core/src/fonts.rs` takes only what the
pages use: regular (400), semibold (600) if the font has it, bold (700),
and italic, for the Latin and Latin-Extended subsets. Ubuntu comes to six
files and 250 KB. They go into `fonts/ubuntu/` in the site folder, with a
`font.toml` that remembers the family name and the ranges.

Picking the weights is a small search. Some fonts don't have 400 or 700
at all, so `wanted` takes the nearest weight they do have:

```rust
let nearest = |target: u16| {
    details.weights.iter().copied()
        .min_by_key(|w| (i32::from(*w) - i32::from(target)).abs())
};
```

The weights are `u16`, unsigned, so `400 - 700` would underflow. Turning
them into `i32` first makes the difference safe to take and then
`.abs()`.

## @font-face and unicode-range

The build copies the fonts in use next to `style.css` and appends one
[`@font-face`](https://developer.mozilla.org/en-US/docs/Web/CSS/@font-face)
rule per file:

```css
@font-face {
  font-family: "Ubuntu"; font-style: normal; font-weight: 700;
  font-display: swap;
  src: url(fonts/ubuntu/latin-ext-700-normal.woff2) format("woff2");
  unicode-range: U+0100-02BA, …;
}
:root { --font-sans: "Ubuntu", ui-sans-serif, system-ui, sans-serif; }
:where(h1,h2,h3,h4,h5,h6,.card-title) { font-family: "Playfair Display", ui-serif, Georgia, serif; }
```

[`unicode-range`](https://developer.mozilla.org/en-US/docs/Web/CSS/@font-face/unicode-range)
tells the browser which characters a file covers, so it only downloads
`latin-ext` when a page actually has a character from it (`ő`, `ł`, …).
German umlauts are in plain `latin`. `font-display: swap` shows the text
in the fallback font right away and switches once the font has arrived,
so nobody stares at an empty page.

Tailwind sets the page font from the variable `--font-sans`, so
overriding that variable is enough for the text. The headings rule uses
[`:where()`](https://developer.mozilla.org/en-US/docs/Web/CSS/:where),
which counts as zero specificity, so a class on a heading still wins.

One font for both is perfectly normal: headings then differ by weight
and size. daisyUI themes don't define fonts at all, only colors and
shapes, so this lives beside the theme rather than in it.

## A preview before building

The Fonts card shows a sample in the chosen fonts right after the
download, before anything is built. The browser's
[FontFace API](https://developer.mozilla.org/en-US/docs/Web/API/FontFace)
loads a font file from JavaScript, the same thing `@font-face` does in
CSS:

```ts
const face = new FontFace(name, `url(${convertFileSrc(`${site.root}/fonts/${id}/${file}.woff2`)})`, { weight });
document.fonts.add(await face.load());
```

`convertFileSrc` turns a path on disk into an address the Tauri window
may load (see [Media and the compiler as checklist](2026-10-21-media-and-the-compiler-as-checklist.md)).
Each font is added under a name of its own, `qed-preview-ubuntu`, so it
can't change the look of the app itself.

## Shared by every picker: `<script module>`

The list of 2,100 fonts is the same for both pickers and every site, so
it should be fetched once per run, not each time a picker opens. A
Svelte component can have a second script block for exactly that:

```svelte
<script lang="ts" module>
  let listing: Promise<FontListing[]> | null = null;
  function allFonts() {
    listing ??= commands.listFonts();
    return listing;
  }
</script>
```

The normal `<script>` runs once per component *instance*. The
[`module`](https://svelte.dev/docs/svelte/svelte-files#script-module)
one runs once, when the file is first imported, so its variables are
shared by all instances. That's like a module attribute or an Agent in
Elixir. It keeps the promise, not the list, so two pickers opening at
the same moment wait on the same request.

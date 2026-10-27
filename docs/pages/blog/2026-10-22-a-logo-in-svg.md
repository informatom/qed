+++
title = "A logo in SVG"
description = "A logo in SVG: why a traced picture makes a poor logo, drawing one by hand, and app icons from one file"
menu = "main"
weight = 26
date = 2026-10-22
authors = ["Claude"]
tags = ["svg", "tauri", "design"]
summary = "q.e.d. has a logo and its own app icon now. Why an automatic trace of the picture wasn't usable, how the mark is drawn by hand from circles and paths, and how Tauri makes every icon size from one PNG."
+++

q.e.d. has a logo: a Q with a few houses and towers in it, a site being
built. Until now the app shipped with Tauri's placeholder icon. The
artwork lives in `brand/` at the top of the repository:

![The q.e.d. mark](/media/logo-mark.svg)

- `logo-mark.svg`: the mark on its own, square, transparent.
- `logo.svg`: the mark with "q.e.d." below it.
- `app-icon.svg` and `app-icon.png`: the mark on a light tile with
  rounded corners, the source for the app icons.
- Some JPEG pictures with the mascot, for the README and the docs.

## Why the traced logo wasn't usable

The first SVG was made by a tracer, a program that turns a picture into
shapes by finding areas of one colour. It looked right, but:

- **It was 1 MB.** 131 shapes, each a long list of curves with
  coordinates like `0.6610272216796602`. A logo should be a few KB.
- **It traced everything**, including the white background, the grey disc
  and its shadow. A logo needs a transparent background to sit on any
  colour.
- **It was the wrong shape.** 1024 × 572 instead of square, and the small
  details (windows, the letters) turned to blobs at icon size.

A tracer copies the *pixels*. What the picture *means* (a ring, three
rectangles, a house) is lost. Drawing the same thing by hand keeps it, in
about 50 lines that a person can read.

## How the mark is drawn

SVG is a text format for vector graphics: shapes with coordinates, which
stay sharp at any size. The
[MDN SVG tutorial](https://developer.mozilla.org/en-US/docs/Web/SVG/Tutorial)
covers it well. A few ideas are enough for this logo.

**The coordinate system.** `viewBox="0 0 512 512"` says: inside, the
drawing is 512 units wide and high, whatever size it is shown at. All
numbers below are in those units.

**Later shapes paint over earlier ones.** There is no z-index; the order
in the file is the order of painting. So the ring comes first, the towers
over it, the house in front, the ground line last.

**One ring, three strokes.** The ring in the picture has a dark rim, a
blue body and a light stripe. Instead of drawing three rings, the same
circle is stroked three times, each narrower than the one before:

```xml
<circle cx="256" cy="214" r="146" stroke="#14366B" stroke-width="72"/>
<circle cx="256" cy="214" r="146" stroke="#1F6FD8" stroke-width="52"/>
<circle cx="256" cy="214" r="146" stroke="#6FD0F7" stroke-width="10"/>
```

A stroke is centred on the line, so a 72 wide navy stroke under a 52 wide
blue one leaves 10 units of navy on each side: the rim.

**Paths for everything else.** A tower is
`<path d="M232 152 L300 124 L300 378 L232 378 Z"/>`: *move* to a point,
draw *lines* to three more, *close* the shape. The slanted roof is just a
first line that goes up to the right.

**Clipping.** The ring is a full circle, but it should stand on the
ground, not poke out below it. A `clipPath` is a stencil: only what lies
inside it is painted.

```xml
<clipPath id="above-ground">
  <rect x="0" y="0" width="512" height="378"/>
</clipPath>
<g clip-path="url(#above-ground)"> … the ring and its tail … </g>
```

**Text as shapes.** `logo.svg` has the word "q.e.d." below the mark. A
`<text>` element would only look right where the same font is installed.
So the text was set in Ubuntu Sans and then turned into paths with
Inkscape (`inkscape --export-text-to-path`): the letters are now shapes
like the rest and look the same everywhere.

## App icons from one file

Every platform wants its icon in other sizes and formats: PNGs from
32 to 512 pixels for Linux, an `.ico` file holding several sizes for
Windows, an `.icns` file for macOS. Tauri makes all of them from one
square PNG:

```sh
npx tauri icon brand/app-icon.png
```

It writes into `src-tauri/icons/`, and
`src-tauri/tauri.conf.json` lists which of
those files the installers use. It also makes icons for Android and iOS,
which q.e.d. doesn't need, so those folders are deleted again. See
[Tauri's icon guide](https://v2.tauri.app/develop/icons/).

The icon puts the mark on a light tile with rounded corners. Without the
tile, the dark rim of the Q would disappear on a dark taskbar or dock.

## The logo in the app

The app shows the mark next to the site's title and, larger, on the
screen that asks to open a site. The Svelte code imports the file like
code:

```svelte
<script lang="ts">
  import logoMark from "$lib/assets/logo-mark.svg";
</script>

<img id="app-logo" src={logoMark} alt="" class="size-14" />
```

An SVG isn't JavaScript, so what does the import give? Vite, the tool
that bundles the UI, treats files it doesn't run as *assets*: the import
is a URL for the file. A larger file is copied into the build under a
name with a hash of its contents (`logo-mark.3f9c2a.svg`), so a changed
file gets a new name and no window keeps an old cached copy. A file under
4 KB, like this 2 KB logo, is put right into the code as a `data:` URL:
one request less. Either way, a typo in the path is an error at build
time instead of a broken image later. See
[Vite: static asset handling](https://vite.dev/guide/assets.html).

The `alt` text is empty on purpose: the name "q.e.d." is written right
next to the image, so a screen reader would only read it twice. The name
stays text, not part of the image, so it takes the theme's colour; the
navy letters of `logo.svg` would almost disappear in the dark theme.

## On this site

Sites made with q.e.d. can have a logo now: **Site settings → Logo**
picks an image from the media library and writes `logo = "logo-mark.svg"`
into `site.toml`. The generator puts it before the site's name in every
page's header, and into the `<head>` as the icon of the browser tab:

```html
<link rel="icon" href="../media/logo-mark.svg">
```

Every current browser takes an SVG there, so one small file serves both
places. The media library labels the file "Site logo" and won't delete
it while it is one.

This site uses the mark as its logo, the mascot banner at the top of the
start page and the sitting mascot on the blog page. The start page
doesn't show its title above the banner, as pages that start with a Hero
don't: the header already says "q.e.d.", and so does the banner.

## Links

- [MDN: SVG tutorial](https://developer.mozilla.org/en-US/docs/Web/SVG/Tutorial)
- [MDN: `clipPath`](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/clipPath)
- [MDN: paths](https://developer.mozilla.org/en-US/docs/Web/SVG/Tutorial/Paths)
- [Tauri: app icons](https://v2.tauri.app/develop/icons/)
- [Vite: static asset handling](https://vite.dev/guide/assets.html)

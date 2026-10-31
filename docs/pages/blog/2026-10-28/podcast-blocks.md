+++
title = "Podcast blocks"
description = "Latest episode and Subscribe: blocks without settings, a folder built into the program, and a button that hands over our own feeds"
menu = "main"
weight = 47
date = 2026-10-28
authors = ["Claude"]
tags = ["rust", "serde", "podcasting"]
summary = "Two new blocks for podcast sites (#069): the latest episode with its player, and the Podlove Subscribe Button with the podcast's own feeds. They're enum variants without data, and the button's files travel inside q.e.d. itself."
redirect_from = ["/blog/2026-10-28-podcast-blocks"]
+++

A podcast's start page wants two things: the newest episode, ready to
play, and a way to subscribe. Those are the two new blocks (#069). New
sites with "This site is a podcast" switched on get both on their start
page.

**Latest episode** shows the newest published episode: title, "Folge 5 ·
28. Oktober 2026", summary and the player. It changes by itself with
every new episode.

**Subscribe** is the [Podlove Subscribe Button](https://github.com/podlove/podlove-subscribe-button).
It asks which podcast app the listener uses and hands it **the podcast's
own feed**, every format of it. No directory sits in between: the feeds
on the site are the source of truth. Without JavaScript, the block shows
plain links to the feeds instead (`<noscript>`).

## Blocks with nothing to set

Most blocks carry settings: a Hero has a headline, a Post list has a
page to list. These two have none, so in `crates/qed-core/src/block.rs`
they're variants without data:

```rust
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Block {
    // …
    Shared(Shared),
    LatestEpisode,
    Subscribe,
}
```

With `tag = "type"`, serde writes a variant as an object with a `type`
field and the variant's data beside it. A variant without data becomes
just `{ "type": "latest-episode" }`. In a page file it's an empty
container:

```text
::: latest-episode
:::
```

The first attempt was an empty struct, `LatestEpisode(LatestEpisode {})`.
Rust was fine with it, but the TypeScript generated from it
(`Record<string, never>`, "an object with no fields at all") clashed
with the `type` field every block has. A variant without data says what
is meant: there's nothing here. See
[enum representations](https://serde.rs/enum-representations.html).

## A folder inside the program

The button is about a hundred files: script, stylesheet, fonts, icons
for each app. q.e.d. has to put them into every built site that uses the
block, also when it runs as a single program file on someone's laptop,
far away from this repository. The
[include_dir](https://docs.rs/include_dir) crate builds a whole folder
into the program:

```rust
static SUBSCRIBE_BUTTON: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/assets/subscribe-button");
```

`include_dir!` is a macro that runs while compiling: it reads the folder
and turns every file into bytes inside the program, like `include_str!`
does for one file (the icons use that). `$CARGO_MANIFEST_DIR` is the
crate's own folder, so the path works wherever the code is built.
Writing it out again is a small recursive function over `dir.entries()`.

Where the copy comes from and how to update it is written down next to
it, in `assets/subscribe-button/README.md`: taken from
jekyll-octopod-bulma, which version, what was left out. The web player
(#049) will get the same, and AGENTS.md now asks for it.

## The data the button reads

The button wants the podcast's title, cover and feeds as a JavaScript
object. jekyll-octopod wrote it into every page as an inline `<script>`.
q.e.d. writes it once, into `subscribe-button/podcast-data.js`:

```js
window.qedPodcastData = {"title":"…","cover":"…","feeds":[{"format":"mp3","url":"https://…/episodes.mp3.rss", …}]};
```

and the block loads that file before the button. `serde_json::json!`
builds the object; its `Display` gives valid JSON, which is valid
JavaScript too.

## The theme's color, as the button wants it

The button should look like the site's other buttons. It takes its color
as text in `data-colors` and works out its hover shade with an old color
library that knows `#rrggbb`, but not the `oklch(45% .24 277.023)` that
daisyUI themes are written in. So the build reads the theme's button
color out of the stylesheet and converts it (`theme::color_hex` in
`crates/qed-core/src/theme.rs`). OKLCH describes a color by lightness,
chroma and hue, made so that equal steps look equally different;
turning it into the screen's red, green and blue is a few lines of
arithmetic from Björn Ottosson, who designed it
([OKLab](https://bottosson.github.io/posts/oklab/)). The test checks
pure red: `oklch(62.8% 0.2577 29.23)` must come out as `#ff0000`.

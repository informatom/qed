+++
title = "Episode layout"
description = "The episode layout: blocks that stand for parts of a page, a const array of enum values, and impl Trait as an argument"
menu = "main"
weight = 50
date = 2026-10-28
authors = ["Claude"]
tags = ["rust", "enums", "templates", "podcasting"]
summary = "Episode pages are built from a block list in episode.md now (#126): six blocks without settings stand for the player, people, shownotes, chapters, transcript and share links. How that works, and three bits of Rust it uses."
+++

Until now every episode page had the same fixed order: player, people,
shownotes, chapters, share links. `page.html` wrote those sections one
after another. With #126 the order lives in the site folder, in
`episode.md` next to `footer.md`:

```text
+++
title = "Episode layout"
+++

::: episode-player
:::

::: episode-shownotes
:::

::: subscribe
align = "center"
:::

::: episode-transcript
:::
```

The episode's parts are blocks, so the block editor can already move
them around. Any other block can go between them, and it then shows on
every episode.

## Placeholders, like `@inner_content`

Five of the six new blocks stand for something the generator builds from
the episode's front matter. The sixth, `episode-shownotes`, stands for
the page's own blocks. If you know Phoenix layouts, it plays the part of
`<%= @inner_content %>`: the layout wraps the page, and this one spot
says where the page goes.

In `crates/qed-core/src/generator.rs`, `render_page` first renders the
page's own blocks into `own`. Then it walks the layout:

```rust
for block in layout {
    if *block == Block::EpisodeShownotes {
        blocks.extend(own.iter().cloned());
    } else if block.is_episode_part() {
        // Player, people, …: built from the episode
    } else {
        // An ordinary block, rendered with this page's links
    }
}
```

The parts used to be fields of the page template (`audio`, `people`,
`chapters`, …). Now they are variants of `RenderedBlock` and render in
`block.html`, like every other block. `page.html` just loops over the
finished HTML.

## Unit variants and a `const` array

The new blocks have nothing to set, so they are *unit variants*: enum
variants without data, like `Block::LatestEpisode` before them:

```rust
pub enum Block {
    Text { markdown: String },
    Hero(Hero),
    // …
    EpisodePlayer,
    EpisodePeople,
    EpisodeShownotes,
    // …
}
```

Without `episode.md`, pages keep the old order. That order is written
once in `crates/qed-core/src/block.rs`:

```rust
pub const DEFAULT_EPISODE_LAYOUT: [Block; 5] = [
    Block::EpisodePlayer,
    Block::EpisodePeople,
    Block::EpisodeShownotes,
    Block::EpisodeChapters,
    Block::EpisodeShare,
];
```

A `const` must be computable while the program compiles. `Block` holds
`String`s elsewhere, which need memory at run time, so you might expect
this to fail. It doesn't: a unit variant has no string inside, and Rust
only cares about the values you actually write. `[Block; 5]` is an
array of exactly five, with its length in the type, unlike a `Vec`.
Where a slice is wanted, `&DEFAULT_EPISODE_LAYOUT[..]` lends the whole
array as one. Saving compares against it, too: when the layout saved is
the default, `save_episode_layout` deletes the file, because pages look
the same without it.

In Elixir you'd write the default as a module attribute,
`@default_layout [:player, :people, …]`. Rust's
[`const`](https://doc.rust-lang.org/std/keyword.const.html) is the same
idea, checked by the type system.

## `Option` and `?` for parts with nothing to show

An episode without chapters shouldn't get an empty "Kapitel" heading.
Each part comes from `EpisodePage::part`, which returns
`Option<RenderedBlock>`: `None` means "nothing to show". Two short forms
keep it compact:

```rust
Block::EpisodePlayer => {
    let audio = audio_entry(/* … */)?;   // no audio: return None
    Some(RenderedBlock::EpisodePlayer { audio, player })
}
Block::EpisodeChapters => {
    let chapters: Vec<ChapterEntry> = /* … */;
    (!chapters.is_empty()).then_some(RenderedBlock::EpisodeChapters { /* … */ })
}
```

`?` on an `Option` returns `None` from the whole function right away,
just as it returns the error for a `Result`. That's Elixir's `with`
dropping to its `else` on the first non-match.
[`bool::then_some`](https://doc.rust-lang.org/std/primitive.bool.html#method.then_some)
turns `true` into `Some(value)` and `false` into `None`.

## `impl Iterator` as an argument

The Subscribe button's script is written only if some block uses it. It
used to check the pages and the footer. Now the episode layout counts
too, so instead of one more parameter, the function takes any sequence
of pages:

```rust
fn uses_subscribe<'a>(mut pages: impl Iterator<Item = &'a Page>) -> bool {
    pages.any(|page| /* … */)
}

let parts = site.footer.iter().chain(&site.episode_layout);
uses_subscribe(published.iter().copied().chain(parts))
```

[`impl Trait` in argument position](https://doc.rust-lang.org/book/ch10-02-traits.html#traits-as-parameters)
means "some type that is an iterator over `&Page`; the caller picks
which". A chain of a `Vec`'s iterator and two `Option`s is a long,
nested type nobody wants to spell out, and the function doesn't need to
know it. The compiler makes a copy of `uses_subscribe` for the exact
type at each call site, so nothing gets slower. `Option` can join the
chain because an `Option` is itself something you can iterate: zero or
one items, like `List.wrap/1` in Elixir.

## The transcript, as text

One part is new: `episode-transcript`. The Podlove Web Player already
shows the transcript, but only with JavaScript, and search engines don't
see it. The new block writes it into the page as text. Cues follow each
other every few seconds, so `paragraphs` in the generator joins a
speaker's cues in a row into one paragraph, with the time and name in
front. It sits in a `<details>` element, which stays closed until a
visitor opens it, and that works without JavaScript too.

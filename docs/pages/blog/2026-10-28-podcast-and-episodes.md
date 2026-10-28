+++
title = "Podcast and episodes"
description = "Phase 2 begins: the podcast switch and episode pages, with a field called type, two kinds of UUID and a merge that goes into sections"
menu = "main"
weight = 43
date = 2026-10-28
authors = ["Claude"]
tags = ["rust", "serde", "toml", "podcasting"]
summary = "A site can now be a podcast (#066), and episodes are pages below /episodes with an [episode] section (#067). Along the way: a Rust keyword as a field name, random and name-based UUIDs, and a recursive merge that keeps comments inside TOML sections."
+++

Phase 2 is about podcasting. The first step gives a site a podcast and
gives the podcast episodes. There's no feed yet (#068) and no audio file
(#023); this is the groundwork they build on.

Two decisions shaped it. A site has **one** podcast at most; a second
podcast is a second site. And an episode is **a page**: a dated page below
`/episodes`, like a blog post below `/blog`. So episodes get everything
pages already have: the block editor for the shownotes, links that follow
a move, Post lists, drafts and versions. What makes a page an episode is
one more section in its front matter:

```toml
+++
title = "Getting started"
date = 2026-10-28

[episode]
season = 1
number = 3
guid = "6f0c1d9e-…"

[[episode.chapters]]
start = "00:00:00"
title = "Intro"
+++
```

The podcast's own settings go into `site.toml` as `[podcast]`. Having that
section *is* the switch: `podcast: Option<Podcast>` in `SiteConfig`, and
`None` means "not a podcast". Both structs live in
`crates/qed-core/src/podcast.rs`.

## A field called `type`

Apple's feed format speaks of an episode's *type* (full, trailer, bonus)
and a podcast's *type* (episodic or serial), so the file should say
`type = "bonus"`. But `type` is a Rust keyword; a field can't be called
that. There are two ways out. Rust lets you write a keyword as a name with
`r#`, a *raw identifier*: `pub r#type: EpisodeType`. Or the field gets
another name in Rust and serde renames it for the file:

```rust
/// A full episode, a trailer or a bonus.
#[serde(default, rename = "type")]
pub kind: EpisodeType,
```

q.e.d. uses the second: `episode.kind` reads better in Rust than
`episode.r#type`, and the file still says `type`. The rename also reaches
TypeScript: tauri-specta reads serde's attributes, so `src/lib/bindings.ts`
has `type?: EpisodeType` and the Svelte code writes `episode.type`. See
serde's [field attributes](https://serde.rs/field-attrs.html) and
[raw identifiers](https://doc.rust-lang.org/rust-by-example/compatibility/raw_identifiers.html).

In Elixir the problem doesn't come up: `type` is an ordinary atom, and a
map can have a `:type` key like any other.

## Two kinds of UUID

Podcast apps recognise an episode by its `guid`, not by its title or its
address. If the guid changes, every subscriber gets the episode again as a
new one. So q.e.d. makes it once, when the episode is created, and never
touches it again, not even when the page moves:

```rust
pub fn new_guid() -> String {
    uuid::Uuid::new_v4().to_string()
}
```

A **version 4** UUID is 122 random bits. Nobody has to keep a counter or
ask a server; two random ones being the same is too unlikely to worry
about.

The podcast as a whole gets a guid too, `podcast:guid` from the
[Podcasting 2.0 namespace](https://podcastindex.org/namespace/1.0#guid),
so apps know it's the same podcast when its feed moves to another address.
That one is a **version 5** UUID: not random, but computed from a name, the
feed address, inside a fixed namespace:

```rust
pub fn podcast_guid(feed_url: &str) -> String {
    const NAMESPACE: uuid::Uuid = uuid::uuid!("ead4c236-bf58-58c6-a2c6-a6b28d128cb6");
    let address = feed_url
        .split_once("://")
        .map_or(feed_url, |(_, rest)| rest)
        .trim_end_matches('/');
    uuid::Uuid::new_v5(&NAMESPACE, address.as_bytes()).to_string()
}
```

The same address always gives the same UUID, so a podcast directory can
work out the guid of any feed by itself. The test checks it against the
example in the namespace's documentation. `uuid!` checks the namespace text
while compiling, so a typo in it is a compile error, not a crash later.

q.e.d. makes this guid when the podcast and the site's web address are both
there, and writes it into `site.toml`. After that the address may change,
the guid doesn't: that's its whole point. Because Rust now fills in a
value while saving, `save_config` returns the settings as saved, and the
UI takes that copy instead of its own (otherwise the next save would send
the guid back as empty). Docs: [uuid](https://docs.rs/uuid).

The feed will live at `/episodes.mp3.rss`, the address jekyll-octopod gave
it, so a podcast that moves over keeps its subscribers without a redirect.

## A merge that goes into sections

Saving a page doesn't rewrite its front matter from scratch. The writer
in `crates/qed-core/src/page_writer.rs` merges the new values into the
file as written, so comments and formatting survive. Until now it only
looked at the top level: a changed `[episode]` section would have been
replaced whole, and a comment inside it lost. Now the merge calls itself
for sections inside sections:

```rust
fn merge_table(table: &mut toml_edit::Table, new: &toml_edit::Table) {
    // … keys that are gone are removed …
    for (key, item) in new.iter() {
        match table.get_mut(key) {
            Some(Item::Table(existing)) if item.is_table() => {
                if let Some(inner) = item.as_table() {
                    merge_table(existing, inner);
                }
            }
            Some(existing) if same_item(existing, item) => {}
            // … replace a changed value, keeping its comment …
        }
    }
}
```

The order of the arms matters. Rust tries them top to bottom, like the
clauses of an Elixir function, and the first one whose pattern and guard
(`if …`) match wins. With the "same item" arm first, two sections were
compared as text, and `toml_edit` writes a section's own lines without
the `[[podcast.funding]]` sections below it. So a podcast whose only change
was a new support link counted as unchanged, and the link never reached
the file. A test caught it; moving the table arm up fixed it. See
[match guards](https://doc.rust-lang.org/reference/expressions/match-expr.html#match-guards).

## Cover art by Apple's rules

Apple Podcasts wants cover art that is square, 1400 to 3000 pixels and a
JPEG or PNG. `check_cover` reads only the image's header (no decoding of
the pixels) and returns what's wrong, in words: "it must be square, not
3000 × 1500 pixels". The Site settings show it under the cover, the
episode settings under an episode's own cover, and the build lists it in
its notes. The podcast still works with a wrong cover; Apple may just
refuse it, so it's a note, not an error.

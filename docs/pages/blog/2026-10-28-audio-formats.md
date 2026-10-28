+++
title = "Several audio formats"
description = "One episode, several audio formats and a feed for each: returning impl Iterator, defaults from a function, and refusing to publish half an episode"
menu = "main"
weight = 45
date = 2026-10-28
authors = ["Claude"]
tags = ["rust", "serde", "podcasting"]
summary = "A podcast can now offer every episode in several formats, such as MP3 for everyone and Ogg or FLAC for those who prefer them, each with a feed of its own. Along the way: functions that return impl Iterator, serde defaults from a function, and a publish that waits for complete episodes."
+++

Apple Podcasts and Spotify play MP3 and M4A, nothing else. Some
listeners prefer Ogg, an open format, or FLAC, which loses nothing of the
sound. jekyll-octopod offered each episode in several formats, each with
its own feed (`episodes.mp3.rss`, `episodes.ogg.rss`), and q.e.d. now
does the same.

The Podcast tab has a list of formats; the first is the **main format**,
whose feed goes to Apple and the directories. Each episode has one file
per format:

```toml
[episode]
number = 1
duration = "00:29:55"

[[episode.audio]]
format = "mp3"
file = "episode1.mp3"
length = 28856805

[[episode.audio]]
format = "ogg"
file = "episode1.ogg"
length = 20114032
```

The length of the episode is the same in every format, so it's stored once.
Choosing `episode1.mp3` in the episode's settings also picks
`episode1.ogg` if it's in `audio/`: tools like Auphonic make all formats
at once, with the same name.

## A default from a function

The formats are a `Vec<AudioFormat>`, and a podcast without the line
should have MP3. `#[serde(default)]` would give an empty list, the
`Default` of a `Vec`. serde can call a function instead:

```rust
#[serde(default = "default_formats")]
pub formats: Vec<AudioFormat>,

fn default_formats() -> Vec<AudioFormat> {
    vec![AudioFormat::Mp3]
}
```

The function's name is given as a string, because attributes take
literals; serde turns it back into a call when it generates the code. See
[default = "path"](https://serde.rs/field-attrs.html#default--path).

## Returning "some iterator"

Several places need the names of an episode's files in `audio/`: the
media library (which files are used), the build (which files are
missing). The method in `crates/qed-core/src/podcast.rs`:

```rust
pub fn local_audio(&self) -> impl Iterator<Item = &str> {
    self.audio.iter().filter_map(EpisodeAudio::local)
}
```

The real type of that chain is long, something like
`FilterMap<slice::Iter<'_, EpisodeAudio>, fn(&EpisodeAudio) -> Option<&str>>`,
and nobody wants to write it down. `impl Iterator<Item = &str>` says
"some iterator of `&str`; which one is my business". The caller can loop
over it, `collect` it or chain more steps, and nothing is copied into a
`Vec` that a caller might not need. Returning `impl Trait` is checked at
compile time: the compiler still knows the exact type and makes it as
fast as writing the loop by hand.

`filter_map(EpisodeAudio::local)` passes a method as a function: `local`
takes `&EpisodeAudio` and returns `Option<&str>`, which is exactly what
`filter_map` wants. Elixir's `Enum.flat_map/2` or a comprehension with a
filter would be the closest thing, though Elixir's `Enum` builds lists
eagerly, where Rust's iterators (like Elixir's `Stream`) do nothing until
someone asks for the next item. See
[returning types that implement traits](https://doc.rust-lang.org/book/ch10-02-traits.html#returning-types-that-implement-traits).

## Every feed gets every episode

If an episode had an MP3 but no Ogg yet, the Ogg feed would skip it, and
its subscribers would never know. So publishing waits:

```rust
pub fn check_publish(site: &Site) -> Result<(), SiteError> {
    let incomplete = incomplete_episodes(site);
    if incomplete.is_empty() {
        return Ok(());
    }
    Err(SiteError::Invalid { /* … */ })
}
```

Saving and previewing still work, so the Ogg can follow later; the page
tree marks the episode ("no Ogg"), its settings say what's missing, and
the build lists it in its notes. Drafts don't count, so an episode can
wait as a draft until all its files are there. The check sits in
qed-core and the Tauri command only calls it, as the shell should stay
thin.

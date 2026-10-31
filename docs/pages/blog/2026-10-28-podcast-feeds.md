+++
title = "Podcast feeds"
description = "The feeds podcast apps subscribe to: four XML namespaces, one feed per format, and functions handed over as &dyn Fn"
menu = "main"
weight = 46
date = 2026-10-28T08:50:00
authors = ["Claude"]
tags = ["rust", "podcasting", "rss"]
summary = "q.e.d. now writes the podcast feeds (#068): one per audio format, at jekyll-octopod's addresses, with the tags of Apple, Podcasting 2.0 and Podlove chapters. The new feed module gets two functions from the build as trait objects, &dyn Fn."
+++

A podcast *is* its feed. Podcast apps never look at the website; they
fetch an RSS file again and again and download whatever is new in it. So
the feed has to be right, and it has to stay where it is: q.e.d. writes
`episodes.mp3.rss` (and `episodes.ogg.rss` and so on for every format),
the addresses jekyll-octopod used, so a podcast that moves over keeps
its subscribers.

## Four vocabularies in one file

The code lives in `crates/qed-core/src/feed.rs`. A feed is RSS 2.0 plus
three extensions, each an XML *namespace* declared at the top:

```xml
<rss version="2.0"
     xmlns:atom="http://www.w3.org/2005/Atom"
     xmlns:content="http://purl.org/rss/1.0/modules/content/"
     xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd"
     xmlns:podcast="https://podcastindex.org/namespace/1.0"
     xmlns:psc="http://podlove.org/simple-chapters">
```

- **RSS** has the basics: the episode's title, link, date, and the
  `enclosure`, the audio file with its size and type.
- **`itunes:`** is Apple's: cover art, categories, owner, explicit,
  season and episode numbers, duration. Nearly every app reads it.
- **`podcast:`** is the open [Podcasting 2.0](https://podcastindex.org/namespace/1.0)
  namespace: the podcast's lasting guid, support links (`podcast:funding`),
  and episode numbers that may be 0, which Apple's may not.
- **`psc:`** is [Podlove Simple Chapters](https://podlove.org/simple-chapters/),
  the chapter marks.

Each episode's `guid` is the one made when the episode was created, or,
for episodes that came from octopod, octopod's old one. Apps recognize
episodes by it, so they don't offer the old episodes again as new.

Feeds can be split into pages that link each other (RFC 5005), and
octopod could do that. q.e.d. doesn't: Apple Podcasts and most apps read
only the first page, so older episodes would quietly disappear there.

## Handing functions over: `&dyn Fn`

The feed needs two things only the build knows: an episode's shownotes
as HTML with full links (that takes the Markdown renderer, the images
and the site's icons), and the size of an audio file. Instead of handing
`feed.rs` all of that, the build hands it two functions:

```rust
pub struct FeedInput<'a> {
    // …
    pub shownotes: &'a dyn Fn(&Page) -> String,
    pub local_length: &'a dyn Fn(&str) -> Option<u64>,
}
```

and fills them with closures that capture what they need:

```rust
let local_length = |file: &str| fs::metadata(audio_dir.join(file)).ok().map(|m| m.len());
let input = FeedInput { local_length: &local_length, /* … */ };
```

Every closure in Rust has its own type, one nobody can write down. A
struct field needs a type, though. `dyn Fn(&str) -> Option<u64>` is a
*trait object*: "any value that can be called like this". It sits behind
a reference (`&`), because different closures have different sizes, and
a reference is always the same size. The call `(input.local_length)(file)`
then looks up the actual function at run time.

The other way would be a generic struct, `FeedInput<S, L>` with
`S: Fn(&Page) -> String`. That's checked at compile time and can be a
little faster, but it spreads the type parameters into every function
that touches the struct. For two calls per episode, `&dyn` is the
simpler choice. In Elixir, passing a function is everyday business
(`fn page -> … end` or `&String.length/1`); Rust just asks you to say
whether you want it decided at compile time or at run time. See
[trait objects](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
and [closures](https://doc.rust-lang.org/book/ch13-01-closures.html).

## Checking it

`xmllint --noout episodes.mp3.rss` says whether the XML is well-formed.
For the test podcast, the guids and audio addresses come out exactly as
octopod wrote them. Before submitting to Apple, a podcast feed validator
such as [Podbase](https://podba.se/validate/) checks the podcast tags as
well.

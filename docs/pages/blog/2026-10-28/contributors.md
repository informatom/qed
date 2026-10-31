+++
title = "Contributors"
description = "Contributors entered once and reused: a file per person, pages the build makes up, roles from the Podcasting 2.0 taxonomy, and an Option that iterates"
menu = "main"
weight = 49
date = 2026-10-28
authors = ["Claude"]
tags = ["rust", "podcasting"]
summary = "The people of a podcast now live in contributors/, one file each, and episodes and the podcast's team pick them with a role (#047). Each gets a page with their episodes, and the feeds name them with podcast:person. Along the way: pages that exist only in the build, and Option as a tiny iterator."
redirect_from = ["/blog/2026-10-28-contributors"]
+++

In jekyll-octopod, every episode listed its contributors in full, name,
website and email, copied from the episode before. q.e.d. keeps each
person once, in `contributors/`, and episodes only point to them (#047):

```text
contributors/stefan-haslinger.md

+++
name = "Stefan Haslinger"

[[links]]
title = "informatom"
url = "https://informatom.com/"
+++

A short bio, in Markdown.
```

```toml
[[episode.people]]
contributor = "stefan-haslinger"
role = "guest"
```

The file name is the person's id. It's made from the name when they're
added and then stays, so renaming "Stefan H." to "Stefan Haslinger"
breaks no episode. The Contributors tab edits these files; the Podcast
tab has a **Team** (the regular crew, which new episodes start with);
episodes have **People**. Someone in several roles, say host and audio
editor, simply has a row for each.

## Roles apps understand

The roles come from the [Podcasting 2.0 taxonomy](https://github.com/Podcastindex-org/podcast-namespace/blob/main/taxonomy.json):
each has a name and a group, `guest` in `cast`, `audio editor` in
`audio post-production`. The feeds write them as

```xml
<podcast:person role="guest" group="cast" href="https://…/contributors/stefan-haslinger/">Stefan Haslinger</podcast:person>
```

and apps that know the tag can show who's speaking. On the site, the
same roles appear in the site's language: "Gast", "Moderation".

The enum `Role` in `crates/qed-core/src/contributors.rs` derives
`PartialEq`, and so does `EpisodePerson`. That makes catching a slip
(the same person twice in the same role) one line:
`people[..index].contains(person)` compares whole values, contributor
and role together.

## Pages that only the build knows

Every contributor gets a page at `/contributors/<id>/` with photo, bio,
links and the episodes they're in, and `/contributors/` lists them all.
None of these pages is a file in `pages/`. The build makes them up as
ordinary `Page` values (`contributor_pages` in
`crates/qed-core/src/generator.rs`), out of blocks the generator knows
anyway:

```rust
blocks: vec![
    about,                                        // Text with image: photo and bio
    Block::Text { markdown: format!("## {}", words.episodes) },
    Block::PostList(block::PostList {
        from: Some("/episodes".to_string()),
        contributor: Some(person.id.clone()),     // only their episodes
        ..block::PostList::default()
    }),
],
```

Then they go into the list of pages to build, next to the real ones:

```rust
let people_pages = contributor_pages(site);
let published: Vec<&Page> = site.pages.iter()
    .filter(|p| !p.content.front.draft && p.path != NOT_FOUND_PATH)
    .chain(&people_pages)
    .collect();
```

`published` holds references (`&Page`) into `people_pages`, so
`people_pages` must live at least as long as `published`. A local
variable in the same function does, and the borrow checker confirms it;
had `contributor_pages` been called inside the `.chain(…)`, the made-up
pages would be a temporary that's gone at the end of the statement, and
the compiler would refuse to keep references to them. A site that writes its own
`pages/contributors.md` keeps it: made-up pages never replace real ones.

The Post list's new `contributor` filter works on any page, too: "all
episodes with Stefan" is a Post list with him picked.

## An Option is a list of at most one

Deleting a contributor is refused while someone still lists them, and
the message names where: the podcast's team and each episode. The team
is one place or none, the episodes any number:

```rust
let team = in_team.then(|| "the podcast's team".to_string());
team.into_iter().chain(site.pages.iter().filter(…).map(…)).collect()
```

`bool::then` gives `Some(…)` or `None`, and `Option::into_iter` turns
that into an iterator of one or zero items, which chains in front of the
episodes like any other list. In Elixir you'd write
`if in_team, do: ["the podcast's team"], else: []` and `++`; Rust gets
there without building the list first. See
[`Option::into_iter`](https://doc.rust-lang.org/std/option/enum.Option.html#method.into_iter)
and [`bool::then`](https://doc.rust-lang.org/std/primitive.bool.html#method.then).

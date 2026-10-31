+++
title = "Two languages"
description = "Sites in several languages: translations beside their pages, a map keyed by Option, Arc for a set many links share, and hreflang"
menu = "main"
weight = 72
date = 2026-10-31
authors = ["Claude"]
tags = ["rust", "i18n", "generator"]
summary = "A site can now be in several languages (#145): about.en.md beside about.md, at /en/about. How the generator keeps each language to itself with a map keyed by Option<&str>, why the link resolver holds its set of addresses in an Arc, and what hreflang tells search engines."
+++

The manual is now in German too, and the docs site has a German start
page. Underneath is a feature every site gets: further languages, set
under `[languages.en]` in `site.toml`.

## A translation is a file beside its page

`about.md` is the page in the site's main language; `about.en.md` beside
it is its English translation, at `/en/about`. `url_path` in
`crates/qed-core/src/site.rs`, which turns a file name into an address,
now returns two things, the address and the language:

```rust
fn url_path(file: &Path, pages_dir: &Path, relative: &Path)
    -> Result<(String, Option<String>), SiteError>
```

A tuple is Rust's lightest way to return more than one value, like
`{:ok, path, language}` in Elixir, but checked by the compiler: every
caller has to take both apart (`let (path, language) = …`). The
`Option<String>` is `None` for the main language. Before this change a
dot in a file name was simply an error, so no existing site can have a
file that suddenly means something else.

The page keeps both: `path` is the address (`en/about`), `language` is
`Some("en")`, and `Page::main_path()` gives back `about`, the page it
translates. Most of the generator only ever looks at `path`, so feeds
end up at `/en/blog/feed.xml` and posts at `/en/blog/…` without anything
knowing about languages.

## A map keyed by Option

A page should see only its own language: in its menus, post lists and
back and next links. Instead of teaching each of those to filter, the
build sorts the pages into one list per language, in
`crates/qed-core/src/generator.rs`:

```rust
let mut by_language: BTreeMap<Option<&str>, Vec<&Page>> = BTreeMap::new();
for page in &published {
    by_language.entry(page.language.as_deref()).or_default().push(page);
}
```

`Option<&str>` works as a key because it can be compared and ordered
(`None` comes first, so the main language does). `entry(…).or_default()`
is "get or insert empty", the same idea as `Map.update/4` with a default.
Each page is then rendered with a `Context` whose `pages` is its own
language's list, so the Post list, the Page list and back and next
links stay in one language without changing a line of their own.

## Arc: one set, many owners

Links written by file name (`[About](about.md)`) should lead to the
translation in the page's own language, if there is one. The link
resolver, `LinkContext` in `crates/qed-core/src/links.rs`, needs to know
which addresses exist for that. There is one `LinkContext` per page,
and they're cloned freely, so copying a set of a hundred addresses into
each would be wasteful. Instead they share it:

```rust
language: Option<(String, Arc<BTreeSet<String>>)>,
```

[`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html) is a pointer
with a count of its owners. Cloning it copies the pointer and adds one
to the count; the set is freed when the last owner is gone. It's the
closest Rust gets to Elixir's immutable data, where passing a big
structure around never copies it: an `Arc` hands out read-only access,
and anyone who wants to change the contents would have to make their own
copy. (`Rc` is the same thing for one thread only; `Arc` costs a little
more but may cross threads.)

## hreflang

Each page names its translations in its head:

```html
<link rel="alternate" hreflang="de" href="https://qed.informatom.com/de/">
<link rel="alternate" hreflang="en" href="https://qed.informatom.com/">
```

Search engines use that to show people the version in their language,
and not count the two as duplicate content. Google's
[guide to localized versions](https://developers.google.com/search/docs/specialty/international/localized-versions)
wants full addresses, so the links are only written when the site has
its web address. The switch in the header is plain links, no
JavaScript: to the same page in the other language, or to that
language's start page when the page has no translation there.

## The manual in the app

The app builds its manual in one language at a time
(`crates/qed-core/src/manual.rs`): a German page (`versions.de.md`) is
written as `versions.md`, in place of the English one, and pages without
a translation stay English. So the manual's links by file name work
unchanged in both languages, and a page nobody has translated yet is
still there, just in English.

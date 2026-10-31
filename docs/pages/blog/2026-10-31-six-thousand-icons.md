+++
title = "Six thousand icons"
description = "From 72 Lucide icons to over 6,000 Tabler icons: data files made by a script, a table for old names, a search sorted by tuples, and a search field in Svelte that ignores stale answers"
menu = "main"
weight = 75
date = 2026-10-31
authors = ["Claude"]
tags = ["rust", "svelte", "icons", "search"]
summary = "q.e.d. switched from 72 hand-picked Lucide icons to all of Tabler's, for sites and for the app (#149). How 6,000 icons get into the program as three text files, how old names keep working, how the search ranks its results with a tuple, and how a Svelte search field drops answers that arrive too late."
+++

Until now, a site could use 72 icons: a hand-picked set from
[Lucide](https://lucide.dev), one SVG file each in
`crates/qed-core/assets/icons/`, listed by name in `src/icons.rs`. That
was enough to start with, but whoever wanted a Mastodon logo in the
footer was out of luck. Now q.e.d. has all of
[Tabler's icons](https://tabler.io/icons): 5,184 drawn as outlines and
1,054 filled ones, brand logos among them. The app's own buttons use
them too.

## Three text files instead of 6,000 SVG files

6,000 files in the repository, each listed by name in Rust, would be
unwieldy. Tabler's npm package has the shapes as data
(`tabler-nodes-outline.json`): for each icon a list of elements and
their attributes. A small script, `scripts/tabler-icons.mjs`, writes
them as text, one line per icon:

```text
mail	<path d="M3 7a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v10a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-10"/><path d="M3 7l9 6l9 -6"/>
```

The name, a tab, the inside of the `<svg>`. `npm run build:icons` runs
it after an update; the three files (outline, filled, search words) end
up in `crates/qed-core/assets/tabler/` with a README that says where
they come from, as q.e.d. does for every piece of others' code it
carries.

`src/icons.rs` puts them into the program with
[`include_str!`](https://doc.rust-lang.org/std/macro.include_str.html),
which reads a file while compiling and makes it a `&'static str`, text
that lives as long as the program. Turning the lines into SVGs happens
once, the first time an icon is needed, in a `OnceLock` (see
[Git over SSH](2026-10-28-git-over-ssh.md)):

```rust
fn icons() -> &'static BTreeMap<String, String> {
    static ICONS: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    ICONS.get_or_init(|| { /* each line: name → <svg …>shapes</svg> */ })
}
```

That's 2 MB of text in the program. The installer, which is compressed,
grows by well under one.

## Old names keep working

Sites made before this use Lucide's names: `icon = "house"` in a page's
settings, `:icon-book-open:` in text. Most names are the same in
Tabler; 23 aren't. Rather than rewriting people's pages, `icons.rs` has
a table:

```rust
pub const RENAMED: &[(&str, &str)] = &[
    ("book-open", "book"),
    ("house", "home"),
    // …
];
```

and `svg("house")` looks there first. A page that opened yesterday opens
today. The manual, though, should show the new names, so a test goes
through every manual page and fails on an old name outside code.

## Ranking with a tuple

With 6,000 icons the app can't show them all; it needs a search. A
search for `heart` should show `heart` first, then `heart-broken`, then
icons that merely contain the word, and last those whose *search words*
have it, Tabler's tags (that's how `envelope` finds `mail`).

`search` in `src/icons.rs` gives every match a rank from 0 (the name is
the word) to 3 (a search word starts with it) and sorts by a tuple:

```rust
found.sort_by_key(|(score, name)| (*score, name.ends_with("-filled"), name.len(), *name));
```

Rust compares tuples element by element, like words in a dictionary:
first the rank; for the same rank, outline icons before filled ones
(`false` sorts before `true`); then shorter names, which are usually the
plain version; and finally the name, so the order never depends on
chance. Elixir's `Enum.sort_by/2` with a tuple does exactly the same.
See [`sort_by_key`](https://doc.rust-lang.org/std/primitive.slice.html#method.sort_by_key).

## A search field that ignores stale answers

`src/lib/IconSearch.svelte` searches while you type. Each keystroke
could ask Rust, and the answers could come back out of order: the one
for `he` after the one for `heart`. Two small things prevent that:

```ts
let latest = 0;
$effect(() => {
  const words = query;
  const run = ++latest;
  const timer = setTimeout(async () => {
    const found = await searchIcons(words, 96);
    if (run === latest) results = found;
  }, 150);
  return () => clearTimeout(timer);
});
```

- The `$effect` runs whenever `query` changes. The function it
  returns runs before the next time: it cancels the timer, so only a
  pause of 150 ms in typing sends a search.
- Each search gets a number. An answer is only shown if no later search
  was started meanwhile (`run === latest`).

The same component is in two places: at **Icon** in the page settings,
and under every text field. A page can have several text fields, and
every element in q.e.d. gets a stable `id`, so each needs ids of its
own. Svelte 5's
[`$props.id()`](https://svelte.dev/docs/svelte/$props#$props.id) gives
every instance of a component a unique one:

```ts
const uid = $props.id();
```

and the search field becomes `markdown-icon-{uid}-search`.

## The app's own icons

The app imported each icon as a component, `import Save from
"@lucide/svelte/icons/save"`. Tabler's Svelte 5 package,
`@tabler/icons-svelte-runes`, works the same way, so switching was one
line per icon: `import Save from
"@tabler/icons-svelte-runes/icons/device-floppy"`. The name in the code
stays `Save`; only where it comes from changed. One difference showed up
in the type check: Lucide's components took `class` as a list
(`["size-4", open && "rotate-90"]`), Tabler's only as text, so two
places now build the text themselves.

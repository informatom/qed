+++
title = "Shared blocks, context"
description = "Shared blocks: one file in shared/, shown on many pages, with Svelte's context for the editor and shadowing for the build"
menu = "main"
weight = 40
date = 2026-10-27
authors = ["Claude"]
tags = ["svelte", "rust", "blocks"]
summary = "Opening hours, a contact box, a newsletter call: some blocks belong on several pages. Shared blocks (#043) keep them in one file in shared/. The editor reaches them through Svelte's context, and the generator swaps them in before anything renders."
redirect_from = ["/blog/2026-10-27-shared-blocks"]
+++

Some blocks belong on several pages: the opening hours, a contact box, a
call to subscribe. Copying them means changing every copy later. Shared
blocks (#043) keep them in one place.

## One file, many pages

A shared block is a file in the new `shared/` folder, written like a page:
a title in the front matter, then blocks.

```text
shared/opening-hours.md
+++
title = "Opening hours"
+++

Monday to Friday, **9 to 17**.
```

A page shows it with a block of the new type `shared`, which only names
the file:

```text
::: shared
name = "opening-hours"
:::
```

`Block::Shared` in `crates/qed-core/src/block.rs` is the smallest block
there is: one optional `name`. Everything else lives in
`crates/qed-core/src/shared.rs`: reading the folder, making a new piece
(its name is the slug of its title, with `-2` added if that's taken),
saving, and refusing to delete a piece a page still shows.

## Swapping them in before the build

The generator has a lot of code that looks at blocks: rendering, the first
image for link previews, "a page starting with a Hero hides its title".
Teaching each of them about shared blocks would be easy to get wrong. So
`build_on` in `generator.rs` replaces every Shared block with its piece's
blocks first, and the rest of the build never sees one:

```rust
let mut notes = Vec::new();
let expanded = crate::shared::expanded(site, &mut notes);
let site = &expanded;
```

The last line is *shadowing*: a new variable with the same name as the
old one. From here on `site` means the expanded copy, and the original
can't be used by mistake. Elixir does the same with rebinding
(`site = expand(site)`); in Rust it's a new binding, so it may even have
a different type ([the Rust book on
shadowing](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html#shadowing)).

`expanded` clones the site, a deep copy of every page. That costs a few
microseconds per page, far less than writing the HTML. It also lets a
missing piece become a note in the build report instead of an error: the
page still builds, and the note says which page asks for which name.

A piece can't hold another Shared block: `save_blocks` drops them, and
`expand` drops any written into the file by hand. So two pieces can never
include each other in a circle.

## Svelte's context

In the app, a Shared block needs a list of all pieces for its dropdown and
a way to open one in the editor. "Make shared", on every block, needs a
way to create one. The blocks are drawn by `BlockList.svelte`, which sits
inside `BlockEditor`, inside `PageEditor`, and inside itself for every
column. Passing the pieces down as a prop through every one of those
layers is called *prop drilling*, and it gets tiresome.

Svelte's [context](https://svelte.dev/docs/svelte/context) skips the
layers. A component calls `setContext(key, value)`, and every component
below it, however deep, can call `getContext(key)` to get that value.
`src/lib/sharedBlocks.ts` wraps the two calls with a type:

```ts
const KEY = Symbol("shared-blocks");

export function provideSharedBlocks(value: SharedBlocks) {
  setContext(KEY, value);
}

export function sharedBlocks(): SharedBlocks | undefined {
  return getContext<SharedBlocks | undefined>(KEY);
}
```

A `Symbol` as the key can't clash with any other key, since no other code
has that symbol.

The main page provides it once (`src/routes/+page.svelte`):

```ts
provideSharedBlocks({
  get pieces() {
    return site?.shared ?? [];
  },
  insidePiece: false,
  open: (name) => select(`shared/${name}.md`),
  async make(title, block) { /* createShared, then add it to the list */ },
});
```

Note the `get pieces()`. Context is set once, when the component starts,
and isn't reactive by itself. A plain `pieces: site.shared` would hand
over the list as it was at that moment. A
[getter](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/get)
runs each time someone reads `pieces`, and since `site` is `$state`, a
`BlockList` reading it inside its markup updates when a new piece arrives.

Context also follows the component tree, so a lower component can
provide a different value for its own part. `SharedEditor.svelte` does
that: inside a shared block, `insidePiece` is `true`, and `BlockList`
leaves out "Make shared" and the Shared block type there.

In Elixir terms, context is a bit like the assigns a Phoenix LiveView
gives to its components, except that a component deep down can reach
them without every component in between passing them on. React has the
same idea under the same name.

# Tabler icons

The icons for generated sites (#149): [Tabler Icons](https://tabler.io/icons)
by Paweł Kuna, version 3.49.0 of the npm package
[`@tabler/icons`](https://www.npmjs.com/package/@tabler/icons), taken
2026-10-31. MIT license, see `LICENSE`.

## What's here

Made from the npm package by `scripts/tabler-icons.mjs`, not by hand:

- `outline.txt`: the 5,184 outline icons, one per line: the name, a tab,
  and the SVG's inner markup (Tabler's `tabler-nodes-outline.json`
  written out as elements).
- `filled.txt`: the 1,054 filled icons, the same way. q.e.d. names them
  with `-filled` at the end (`heart-filled`).
- `tags.txt`: each icon's category and Tabler's search words, so a
  search for "envelope" finds `mail`.
- `LICENSE`: Tabler's license, copied from the package.

Nothing is changed in the shapes. The `<svg>` around them is added by
`src/icons.rs`, with the classes generated sites use.

## How q.e.d. uses them

`src/icons.rs` includes the three files in the program and turns a name
into an SVG (`icons::svg`), for `:icon-name:` in text, page icons in
menus and the RSS icon. Old Lucide names from before #149 are mapped in
`icons::RENAMED`. The app's own interface uses the same version through
`@tabler/icons-svelte-runes`, and its search through `icons::search`.

## Updating

1. Raise both packages in `package.json` to the same new version:
   `@tabler/icons` (dev dependency) and `@tabler/icons-svelte-runes`.
2. `npm install`, then `npm run build:icons`.
3. Update the version here and on the manual's *License and credits*
   page, run `cargo run -p qed-core --example libraries`, and
   `npm run check:all`.

Tabler sometimes renames icons; a page using an old name then fails to
open with a message naming the icon. Add the old name to
`icons::RENAMED` if that happens.

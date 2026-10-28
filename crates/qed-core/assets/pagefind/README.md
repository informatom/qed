# Pagefind

Site search for generated sites (#108): [Pagefind](https://pagefind.app),
<https://github.com/Pagefind/pagefind>, MIT license.

## Where it comes from

Unlike the other folders here, nothing is copied into this folder. The
`pagefind` crate (version 1.5.2, in `crates/qed-core/Cargo.toml`, without
its default `serve` feature) does the indexing and carries Pagefind's
browser files in its `vendor/` folder. It writes them into the built site
together with the index. So the crate's version decides the files'
version; Cargo.lock pins it.

## What q.e.d. uses

`qed_core::search::index` runs after each build (unless `search = false`
in `site.toml`) and writes `_site/pagefind/`:

- the index: `pagefind-entry.json`, `pagefind.*.pf_meta`, `index/`,
  `fragment/` (one file per page), `filter/`
- `pagefind.js`, `pagefind-worker.js` and `wasm.*.pagefind`: the search
  itself, with word stemming for the site's language
- `pagefind-component-ui.js` and `.css`: the search button
  (`<pagefind-modal-trigger>`) and dialog (`<pagefind-modal>`) in
  `templates/page.html`

Pagefind's other interfaces (`pagefind-ui.*`, `pagefind-modular-ui.*`,
`pagefind-highlight.js`) are deleted after writing.

## Changes

None to Pagefind's files. Their colors come from the theme: `site.input.css`
sets Pagefind's `--pf-*` variables from daisyUI's colors, and `page.html`
loads `style.css` after Pagefind's stylesheet so these win.

What gets indexed is decided in the templates: `data-pagefind-body` and
the page title (`data-pagefind-meta`) on `<main>`; `data-pagefind-ignore`
around Post list, Page list, Latest episode and the episode player
(`block_html` in `generator.rs`).

## Updating

Raise the `pagefind` version in `crates/qed-core/Cargo.toml`, run
`cargo update -p pagefind`, build a site and try the search (button,
Ctrl+K, results, a German site). Check the crate's `vendor/` folder for
renamed files and the component UI's `--pf-*` variables for new ones.

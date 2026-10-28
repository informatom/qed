+++
title = "Backlog"
description = "Open work and decisions for q.e.d., numbered #nnn."
menu = "footer"
weight = 10
+++

Open work and open decisions. Items keep their number for good; when one is
done or decided, it moves to [Done](done.md) with a short note, and items
that no longer apply go to its **Dropped** section.

q.e.d. is a single-user desktop app (Rust + Tauri) that turns a podcaster's
content into a static site and publishes it. Decided 2026-10-21; it replaces
the earlier Phoenix web app.

## To decide

Nothing open.

## To build

### Phase 2: Podcasting

- **#048 Transcripts.** WebVTT per episode. Deployed as a file, linked with
  `podcast:transcript`, shown on the episode page and in the player.
- **#049 Web player.** Self-hosted Podlove Web Player, copied into each site,
  with chapters and transcripts. Like the Subscribe button: in
  `crates/qed-core/assets/`, with a README on where it comes from and how
  to update it (rule in AGENTS.md).
- **#126 Episode layout.** The parts of every episode page as a block list
  in `episode.md`, like the footer (user, 2026-10-28): placeholder blocks
  for the episode's player (#049), people, shownotes (the episode's own
  blocks), chapters, transcript (#048) and share row, plus any normal
  block (a Subscribe button, a Call to action) shown on every episode.
  Edited in the page tree next to Footer. Title and byline stay fixed at
  the top. Without the file, today's order: player, people, shownotes,
  chapters, share. The episode parts move from `page.html` into
  `block.html`. After #048 and #049, so their blocks are there.
### Phase 3: Nice to have

- **#070 Code signing.** Apple developer account and notarization, Windows
  signing (e.g. Azure Trusted Signing). Decided 2026-10-28: moved from
  phase 2; not needed for 1.0.

- **#029 Publisher: GitHub Pages.** Push the built site to the configured
  branch with a deploy key. Needs a second `Target` kind, its own key
  handling and connection test (#031). For podcasts, GitHub Pages warns
  when an episode lacks an external audio URL (#068).
- **#011 GitHub Pages.** Decided: deploy key using q.e.d.'s generated key
  pair. Audio must be external for GitHub targets.
- **#085 Images in Markdown text, WebP.** *Copies in text done
  2026-10-27*: images written into text as `![…](/media/…)` get the
  smaller copies in `srcset`, `sizes`, their width and height and
  `loading="lazy"`, like image blocks (#075). Still open: WebP copies for
  all images (#033). The image crate writes only lossless WebP, larger
  than the JPEGs, so this needs another encoder.
- **#083 Closing versions by themselves.** Close an open version with an
  automatic note after a while without saves (from #073).
- **#084 Signed commits.** libgit2 doesn't sign, so q.e.d.'s versions are
  unsigned (from #073).
- **#109 Back up button.** A *Back up* button in the Versions tab that
  pushes the site's git repository to a configured remote (see
  [Backups](backups.md)), so editors don't need the command line: libgit2
  (already used for versions) can push over SSH with a key from the
  keychain, like publishing. Close the open version first; show what was
  pushed and when.
- **#110 Manual.** A user manual inside the app (a *Manual* or *Help*
  view, also reachable from where a topic comes up, e.g. the Versions tab
  for backups), and the same pages on the docs site. Written for site
  owners and editors, not developers. Starts with [Backups](backups.md)
  (#037); further topics: the first site, pages and blocks, media and
  files, themes and fonts, versions, publishing, moving a site over,
  icons in text (`:icon-phone:`, #114, with the list of icons), the
  hand-made menu (#111), boxed columns (#113).
  To decide: where the pages live so app and docs site share them (e.g.
  `docs/pages/manual/`, built into the app), and offline use.
- **#087 Section block.** A full-width block that holds other blocks, in
  the same nested format as Columns (#074).
- **#106 More languages for code highlighting.** syntect's built-in
  grammars (#094) lack TypeScript, Svelte, TOML, Elixir and Jinja, so about
  60 code blocks in the docs show without colors. The `two-face` crate
  (the grammars of `bat`, some 200 languages) plugs into the same
  highlighter; it costs a few MB in the app and a slower first build per
  run.
- **#107 Math formulas.** LaTeX math in Markdown (`$x^2$`, `$$ … $$`, a
  ```` ```math ```` block), which comrak already recognizes, turned into
  MathML when the site is built (e.g. with the pure-Rust `pulldown-latex`).
  Browsers draw MathML themselves (Chrome, Edge, Firefox, Safari since
  2023): no JavaScript and no extra files, unlike KaTeX in the browser.
  Optionally a math font such as STIX Two Math, self-hosted via #102.
- **#108 Site search with Pagefind.** [Pagefind](https://pagefind.app)
  (written in Rust, usable as a library) builds a search index from the
  finished pages during the normal build, split into small chunks; the
  browser loads only what a search needs, from the site itself, nothing
  from third parties. A search box that works without JavaScript as a
  plain form to a search engine limited to the site (`site:…`), and with
  JavaScript Pagefind takes over and shows results on the site. Decided
  2026-10-27: Pagefind rather than lunr.js, MiniSearch or Fuse.js, which
  load the whole index at once.

### Phase 4: Moving in, open to AI assistants

- **#123 Import from Podlove Publisher.** Bring a podcast over from
  WordPress with the [Podlove Publisher](https://podlove.org/podlove-podcast-publisher/)
  plugin, mostly to archive it without having to keep WordPress running
  (user, 2026-10-28): podcast settings, episodes with shownotes,
  chapters, contributors (#047), transcripts (#048), cover art and the
  audio in every format, the old episode guids (so apps don't show
  everything again) and the old addresses as redirects. To decide:
  the source (Podlove's own export file, the feeds, the WordPress REST
  API, or a mix), and how much of the shownotes' HTML becomes Markdown.
  Related: #034 (import from a feed or jekyll-octopod).
- **#034 Feed import.** Import an existing podcast from its RSS feed or from a
  jekyll-octopod site (compare `octopod import`). Keep each episode's old
  guid (octopod used the episode's address) and its old address as a
  redirect. Moved to phase 4 on 2026-10-28, with #123.
- **#124 Feed redirects.** Podlove Publisher's feeds live at other
  addresses (`/feed/mp3/`, or `/?feed=…` without pretty links) than
  q.e.d.'s (`/episodes.mp3.rss`), and subscribers must keep getting new
  episodes after the move (from #123). A static site can't answer with
  an HTTP 301 by itself. To decide, per target: a `<itunes:new-feed-url>`
  feed at the old address (Apple and most apps then switch for good; works
  on any host, also GitHub Pages, but not for `?feed=` addresses), an
  `.htaccess` with `Redirect 301` rules for Apache hosts (handles query
  strings too), or both. Old feed addresses per format would be a
  podcast setting, filled in by the import.

- **#105 MCP server.** A second thin shell around qed-core next to the
  Tauri app, so AI assistants (Claude Code, Claude Desktop) can work on a
  site through the [Model Context Protocol](https://modelcontextprotocol.io):
  a small crate with the official Rust SDK (`rmcp`), started locally over
  stdio. Tools such as `list_pages`, `read_page`, `save_blocks`,
  `create_page`, `move_page`, `list_media`, `import_media`, `build_site`,
  `list_versions` and `save_version`, with the app's rules: usage checks
  before deleting, links rewritten on moves, every change recorded in the
  site's history, so it shows in Versions and can be undone. To decide:
  publishing left out or only prepared (it reaches the outside world; keys
  stay in the keychain); access only to sites opened in q.e.d.; the app
  noticing changes made outside it and reloading instead of overwriting
  them (useful for edits by hand, too); a separate program or built into
  the running app.

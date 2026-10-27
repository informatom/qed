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

### Phase 1: CMS and site generator

A complete site builder without podcast features. Decided 2026-10-26: finish
phase 1 before podcasting, testing it by moving the owner's Jekyll sites
over by hand.

- **#043 Block CMS.** Mercator's block CMS design rebuilt in Rust
  (`docs/notes/cms-handover-qed.md`; its Elixir file map no longer
  applies). *Done 2026-10-21*: nine block types (Text, Hero, Image, Text
  with image, Gallery, Download, FAQ, Columns with nested blocks (#074),
  Call to action); the block editor with Content/Versions/Settings tabs
  per page, OverType for Markdown, the image picker (#033), add, remove,
  move up/down, save with `site::save_blocks`, a warning before losing
  unsaved changes. Still open: a Shared block (one block used on several
  pages) and drag and drop.
- **#033 Images and media library.** Import, tag, search and resize images to
  WebP; keep PDFs for downloads. Phase 2 adds Apple's cover art rules
  (1400–3000 px square). *Media library done 2026-10-21*
  (`qed_core::media`): list with size, dimensions and usage, import with
  clean names, delete only unused files, image picker, Image block with
  width/height and lazy loading. Still open: resizing to several sizes
  with a build cache (#075, done), tags.
- **#026 Site generator.** Render HTML for published pages, posts, menus,
  redirects (meta-refresh stubs) and the sitemap with Rust templates. Copy
  the images used. Rewrite relative links to other `.md` pages into page
  URLs; links that leave the site folder (e.g. to source code) need a base
  URL such as the GitHub repo. *First version done 2026-10-21*
  (`qed_core::generator`): pages, menus, relative links, Markdown links,
  redirects, sitemap and canonical links with `base_url`, media copied,
  preview server and "Build" / "Preview in browser" in the app. *Link
  previews done 2026-10-22*: Open Graph tags on every page; `og:image`
  from the new `image` page setting ("Share image") or the page's first
  image (smallest copy at least 1200 px wide, else the original), only with `base_url`. Still open: copying only the
  media files that are used, and links that leave the site folder.
- **#054 Theme options.** Per site: daisyUI theme, logo, a few options.
  *Themes done 2026-10-21*: Site settings dialog edits `site.toml` (title,
  description, language, web address, theme); the app has its own theme
  (App settings, per computer, can follow the system). *Logo done
  2026-10-22*: `logo` in `site.toml` (Site settings, image picker) shows
  before the title in every page's header and is the browser-tab icon
  (`<link rel="icon">`); the media library marks it "Site logo" and won't
  delete it. *Theme colors done 2026-10-27*: sites use all four theme
  colors (primary: buttons, Call to action, current menu page, link
  underlines; secondary: Hero without image, post dates; accent: stripe at
  the top, quote bars; neutral: footer, sidebar's current page, file
  types), and Site settings explains them in a legend beside the theme
  switcher; a picked theme saves at once. *Header bar done 2026-10-27*:
  `header` in `site.toml` (plain, primary, secondary, accent, neutral); a colored bar
  underlines the current page and gives the logo a light backing. Fonts:
  #102 (done). Other options are still open.
- **#032 Publish history.** Record each publish run per target as a git tag
  (#073) plus a log; show progress live. *Tags done 2026-10-21* (with
  #073); the log and a list of publishes per target are still open.
- **#085 Images in Markdown text.** Resized copies and `srcset` for images
  written into Text blocks as Markdown, like the image blocks get (#075),
  plus WebP copies for all images (#033).
- **#086 Testing on macOS and Windows.** The installers build in CI but
  were never run on macOS or Windows; friends try them and report back.

### Phase 2: Podcasting

- **#066 Podcast switch.** Per site; turns on episodes, feed, contributors and
  the podcast fields: type, owner name + email, cover, categories, explicit,
  `podcast:guid`, funding.
- **#067 Episodes.** Season, number, type, image, explicit, guid generated
  once, Markdown shownotes, chapters. Episode pages and the archive at
  `/episodes`.
- **#022 Episode editor.** Markdown shownotes (#060) and a chapter editor.
- **#023 Audio file.** One per episode: pick a local file or enter an external
  URL. Read size, MIME type and duration in Rust (`symphonia` or `lofty`).
- **#068 Podcast feed.** One feed per podcast with iTunes, Podcasting 2.0 and
  Podlove Simple Chapters tags, plus paged feeds. Publishers copy the audio
  (GitHub Pages: see #029).
- **#069 Podcast blocks.** Latest episodes and Subscribe blocks; seed them on
  the `home` page of new podcast sites.
- **#047 Contributors.** Contributors per site (name, photo, Markdown bio,
  links), linked to episodes with a role. `podcast:person` in the feed,
  people on episode pages, a Contributors block.
- **#048 Transcripts.** WebVTT per episode. Deployed as a file, linked with
  `podcast:transcript`, shown on the episode page and in the player.
- **#049 Web player.** Self-hosted Podlove Web Player, copied into each site,
  with chapters and transcripts.
- **#053 Share links.** Plain share links on episode pages (Mastodon, Bluesky,
  email, copy link), no third-party scripts.
- **#070 Code signing.** Before a public 1.0: Apple developer account and
  notarization, Windows signing (e.g. Azure Trusted Signing).
- **#034 Feed import.** Import an existing podcast from its RSS feed or from a
  jekyll-octopod site (compare `octopod import`).

### Phase 3: Nice to have

- **#029 Publisher: GitHub Pages.** Push the built site to the configured
  branch with a deploy key. Needs a second `Target` kind, its own key
  handling and connection test (#031). For podcasts, GitHub Pages warns
  when an episode lacks an external audio URL (#068).
- **#011 GitHub Pages.** Decided: deploy key using q.e.d.'s generated key
  pair. Audio must be external for GitHub targets.
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

### Phase 4: Open to AI assistants

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

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

### In progress

- **#110 Manual.** A user manual inside the app (a *Manual* or *Help*
  view, also reachable from where a topic comes up, e.g. the Versions tab
  for backups), and the same pages on the docs site. Written for site
  owners and editors, not developers. Starts with [Backups](backups.md)
  (#037); further topics: the first site, pages and blocks, media and
  files, themes and fonts, versions, publishing, moving a site over,
  icons in text (`:icon-phone:`, #114, with the list of icons), the
  hand-made menu (#111), boxed columns (#113). To decide: where the
  pages live so app and docs site share them (e.g. `docs/pages/manual/`,
  built into the app), and offline use.
- **#133 Links to missing pages.** The build stops on links that leave
  the site folder, but a link to a page, media file or download that
  doesn't exist (a mistyped `2026-10-21-link-previews.md`, found
  2026-10-29) builds without a word and gives visitors a 404. The build
  should note each one, with the page it's on, like a missing shared
  block; the app could also mark it while editing.

### Future

- **#070 Code signing.** For downloads from the releases page, not app
  stores. macOS: notarization needs the Apple Developer Program (99 USD
  a year); without it, macOS 15 and later make people allow the app in
  System Settings (right-click → Open no longer works). Windows: the
  SignPath Foundation signs open-source projects for free (builds from
  CI; check their terms); SmartScreen still warns until the app has a
  reputation. Updates are already safe through the updater's own
  signature. Moved here on 2026-10-29: worth it once Mac users come in.
- **#123 Import from Podlove Publisher.** Bring a podcast over from
  WordPress with the
  [Podlove Publisher](https://podlove.org/podlove-podcast-publisher/)
  plugin, mostly to archive it without having to keep WordPress running
  (user, 2026-10-28): podcast settings, episodes with shownotes,
  chapters, contributors (#047), transcripts (#048), cover art and the
  audio in every format, the old episode guids (so apps don't show
  everything again) and the old addresses as redirects. To decide:
  the source (Podlove's own export file, the feeds, the WordPress REST
  API, or a mix), and how much of the shownotes' HTML becomes Markdown.
  Related: #034 (import from a feed or jekyll-octopod).
- **#034 Feed import.** Import an existing podcast from its RSS feed or
  from a jekyll-octopod site (compare `octopod import`). Keep each
  episode's old guid (octopod used the episode's address) and its old
  address as a redirect. Moved here on 2026-10-28, with #123.
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
- **#105 MCP server.** A second thin shell around qed-core next to the Tauri
  app, so AI assistants (Claude Code, Claude Desktop) can work on a site
  through the [Model Context Protocol](https://modelcontextprotocol.io): a
  small crate with the official Rust SDK (`rmcp`), started locally over
  stdio. Tools such as `list_pages`, `read_page`, `save_blocks`,
  `create_page`, `move_page`, `list_media`, `import_media`, `build_site`,
  `list_versions` and `save_version`, with the app's rules: usage checks
  before deleting, links rewritten on moves, every change recorded in the
  site's history, so it shows in Versions and can be undone. To decide:
  publishing left out or only prepared (it reaches the outside world; keys
  stay in the keychain); access only to sites opened in q.e.d.; the app
  noticing changes made outside it and reloading instead of overwriting them
  (useful for edits by hand, too); a separate program or built into the
  running app.

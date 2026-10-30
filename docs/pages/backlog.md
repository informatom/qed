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

### Phase 3:  In progress

In the order to build them (sorted 2026-10-30). Sites in several
languages (#145) come before the MCP server (#105), since its tools and
format guide describe the site folder that #145 changes. #138–#144,
found while moving blog.panoptikum.social over by hand, are done.

- **#146 Light and dark mode.** A site with two themes, one light and
  one dark (e.g. latte and mocha), and a switch on the page for light,
  dark and the system's setting. Without JavaScript the page follows
  the system (`prefers-color-scheme`; daisyUI 5 can mark a theme
  `--prefersdark`); the switch is an optional feature that remembers
  the choice in the browser and must set it before the page shows, so
  it doesn't flash in the wrong colors. To decide: the setting in the
  Site tab (a second theme, off by default), what a custom theme
  (#101) needs for its dark side, where the switch goes (header,
  footer), and whether `[colors]` and the footer color are set per
  mode (see #141). Added by the user on 2026-10-30.
- **#145 Sites in several languages.** A site in German and English
  (more later), starting with the manual (#110), for which a German
  version was asked for. To decide: how a translation is stored
  (`about.md` next to `about.en.md`, or a folder per language), the
  addresses (the main language at `/`, others under `/en/`), what
  belongs to each language (title, description, menus, footer, shared
  blocks, posts, feeds), a language switch in the header (a plain link
  to the same page in the other language, no JavaScript) and
  `<link rel="alternate" hreflang>`, how pages without a translation
  show, and how the app's editor and New page handle it. For the
  manual compiled into the app: which language the Manual button opens
  (the app's own texts are English only for now), and how manual pages
  keep linking to each other by file name in both languages. Added
  by the user on 2026-10-30.
- **#105 MCP server, picked up again.** Dropped on 2026-10-29, back
  on the list on 2026-10-30: AI assistants (Claude Desktop, Claude
  Code) working on a site over the Model Context Protocol, with the
  app's rules. Continue on the branch `mcp-server` (one commit,
  `qed-mcp` with 13 tools, a format guide as server instructions, a
  stdio test, settings shown in App settings): first bring it up to
  date with main, then make it one program instead of a second one
  bundled as a sidecar, which more than doubled the installers (.deb
  12 → 28 MB): the app itself answers over stdio when started as
  `qed mcp`. To check: stdio of a Windows GUI program started by an
  assistant, and what the settings for Claude Desktop and Claude Code
  then point to (the app's own path, per system).
- **#147 blog.panoptikum.social again, through the MCP server.** After
  #105: delete the q.e.d. copy of blog.panoptikum.social made by hand on
  2026-10-30, move the blog over once more with an AI assistant working
  through the MCP server, publish it, and test the Mastodon
  verification (#140) against the profile. A test of the MCP server on
  a real site (62 posts, 12 pages, sidebar menu) and of `rel="me"`.
  Added by the user on 2026-10-30.

### Phase 4: Podcast Publisher Import

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

### Phase 5: Roll out

- **#070 Code signing.** For downloads from the releases page, not app
  stores. macOS: notarization needs the Apple Developer Program (99 USD
  a year); without it, macOS 15 and later make people allow the app in
  System Settings → Privacy & Security → Open Anyway (right-click → Open
  no longer works; the instructions say so since 2026-10-29). Windows: the
  SignPath Foundation signs open-source projects for free (builds from
  CI; check their terms); SmartScreen still warns until the app has a
  reputation. Updates are already safe through the updater's own
  signature. Moved here on 2026-10-29: worth it once Mac users come in.

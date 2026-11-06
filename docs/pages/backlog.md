+++
title = "Backlog"
description = "Open work and decisions for q.e.d., numbered #nnn."
menu = "main"
icon = "list"
weight = 40
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

- **#154 The whole site as a PDF.** A button in the app makes one PDF
  of the site, only when asked (user, 2026-11-06). It lands in
  `assets/` like any other PDF, so it can be linked from a page, offered
  for download and published with the site. It starts with a table of
  contents of the top two levels of pages (top-level pages and the
  pages in their folders), with page numbers and PDF bookmarks. Made
  with [Typst](https://typst.app), as in the user's Phoenix app
  mercator (offers and order confirmations, "fantastic" results); there
  it ran the `typst` program, here the `typst` and `typst-pdf` crates
  can be compiled into qed-core, so nobody has to install anything.
  Decided 2026-11-06: all posts and episodes come along, in a chapter
  of their own after the pages; a translated site gets one PDF per
  language; blocks keep their meaning in print (columns, FAQ, call to
  action, text with image, galleries), while the player and the
  subscribe button become a link; plain black text on white, without
  the site's theme, fonts or logo; links between pages jump inside the
  PDF where the page is in it, else they keep the web address.

- **#151 New manual screenshots.** The screenshots in
  `docs/media/manual/` still show the app's Lucide icons from before
  #149, and the theme tab is slightly older than that. Retake them all
  (theme *catppuccin-mocha*, as AGENTS.md says) at the end of phase 4,
  when its changes to the app are in (user, 2026-10-31).

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

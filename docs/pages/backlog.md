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

- **#134 Old addresses in the page settings.** A page's earlier
  addresses (`redirect_from`) are only set by moving or renaming it in
  the app; old addresses a q.e.d. page can't have (such as ones ending in
  `.html`, from a site moved over by hand) have to be typed into the
  page's file. The manual's [Moving a site over](manual/moving-a-site.md)
  says so. Wanted: a list in the page's Settings tab to see, add and
  remove them. Found 2026-10-29 while writing the manual (#110).
- **#135 Old episode guids.** A new episode gets a guid that never
  changes, and the app offers no way to give it the one it had before.
  A podcast moved over by hand from another system then shows every
  episode as new in podcast apps; the manual says to move one with help
  for now. Wanted: entering an episode's old guid in its Settings tab
  (with a warning that it must match the old feed exactly), until the
  imports (#034, #123) carry guids over by themselves. Found 2026-10-29
  while writing the manual (#110).

### Phase 4: Future

- **#070 Code signing.** For downloads from the releases page, not app
  stores. macOS: notarization needs the Apple Developer Program (99 USD
  a year); without it, macOS 15 and later make people allow the app in
  System Settings → Privacy & Security → Open Anyway (right-click → Open
  no longer works; the instructions say so since 2026-10-29). Windows: the
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

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

### Phase 3:  In progress

In the order to build them (sorted 2026-10-30). Sites in several
languages (#145) came before the MCP server (#105), since its tools and
format guide describe the site folder; #145 is done. #138–#144,
found while moving blog.panoptikum.social over by hand, are done, and so
are #146 (light and dark mode) and #105 (MCP server).

- **#147 blog.panoptikum.social again, through the MCP server.** After
  #105: delete the q.e.d. copy of blog.panoptikum.social made by hand on
  2026-10-30, move the blog over once more with an AI assistant working
  through the MCP server, publish it, and test the Mastodon
  verification (#140) against the profile. A test of the MCP server on
  a real site (62 posts, 12 pages, sidebar menu) and of `rel="me"`.
  Added by the user on 2026-10-30. *2026-10-31:* the hand-made copy is
  gone; a new empty site was made like the app does, and Claude Code
  (headless, only Read/Glob/Grep on the Jekyll source plus the MCP
  tools) moved the blog over in 14 minutes: 62 posts, 13 pages, 75
  images, 2 PDFs, 69 redirects, links rewritten, one version saved.
  Left for the site owner: site settings (web address, logo, theme,
  footer, sidebar extras, Mastodon `profiles`), the slide shows in
  `static/`. Gaps went to #148. Still to do: the user's look in the app,
  publishing, and the `rel="me"` check.

- **#148 MCP server follow-ups.** Found while testing #105 and #147 on
  2026-10-31. *Done the same day (uncommitted):* `site_info` and
  `list_pages` count posts; the guide lists all page settings, the
  site settings, theme and icon names (both generated from q.e.d.'s
  lists); `create_page` takes a `slug`; `import_file` copies files and
  folders into `static/` at a given folder, which also keeps old file
  addresses (`/pdf/33C3.pdf`, redirects only work for pages);
  `read_page`/`write_page` take the footer, episode layout and shared
  blocks; `read_settings`/`write_settings` for `site.toml`, refused if
  the site wouldn't open with it (`editing::write_settings_text`, with
  the old text put back); the build notes unknown `:icon-…:` in text,
  for the app too. The guide is now a `guide` tool, and the instructions
  sent at the start only point to it: Claude Code cut the long
  instructions off after about 2 KB, so the first assistant never saw
  most of the guide. A tool for deleting files was first left out, then
  added the same day for replacing screenshots from another project
  (user): `delete_file`, refused while anything still uses the file,
  like the Media tab. Still open: stdio of the Windows build when an
  assistant starts `qed mcp` (a friend of the user's tries it).

- **#150 Posts with a time.** A post's `date` is a day only, so the
  feeds give every post 00:00, and feed readers order posts of the same
  day at random (user, 2026-10-31). Allow a time as well
  (`date = 2026-10-31T14:30:00`, TOML's local date-time), use it in
  the feeds, post lists and back/next order, and offer it in the page
  settings (new posts get the current time). A time is in the
  computer's time zone at build time (decided by the user, 2026-10-31). Docs site: give the blog posts times in the order they
  were written, from git (the commit that added each post) or made up
  along their `weight`, so the feed has the right order.

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

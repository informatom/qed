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

In the order to build them (sorted 2026-10-30). #138–#144 were found
on 2026-10-30 while moving blog.panoptikum.social over by hand (62
posts, 12 pages, sidebar menu, theme catppuccin-mocha). Sites in
several languages (#145) come before the MCP server (#105), since
its tools and format guide describe the site folder that #145
changes.

#### Small fixes

- **#137 Misleading count before publishing.** The Publish dialog says
  "3 files changed since the last publish" and lists the changed files
  of the site (`pages/about.md`), but publishing then uploads many more
  files: it builds the website and uploads every built file that differs
  on the server, and one changed title or new page changes every page
  (menu, sidebar, Post lists, back and next links). Reported by the user
  on 2026-10-29. Wanted: wording that keeps the two apart, e.g. "3 of
  your files changed since the last publish" with a line saying that
  more files of the website can change (a title shows in every page's
  menu), and "website files" in the report after publishing ("57
  website files uploaded"). Possibly, for later: build first and show
  how many website files will go up, before asking. The manual's
  [Publishing](manual/publishing.md) page should say the same.
- **#141 Footer color in dark themes.** The footer takes the theme's
  accent color unless set otherwise; in catppuccin-mocha (and likely
  the other dark Catppuccin themes) that's a pale pink band below a
  dark page. blog.panoptikum.social now has `footer = "shaded"`.
  Decided 2026-10-30: when the theme's background is dark, the footer
  is shaded by default (no list per theme); a footer color set in the
  Theme tab still wins. Fits #146, where a site has a light and a dark
  theme.

#### For sites moved over by hand

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
- **#138 Line breaks in table cells.** Markdown tables have no line
  break of their own inside a cell; the usual way is `<br>`, but q.e.d.
  escapes all raw HTML, so it shows as text. Panoptikum's API page used
  it in many cells (it now has "; " instead). Decided 2026-10-30: a
  short list of HTML tags goes through in text, everywhere (not only
  in tables): `<br>`, `<kbd>` (#139), `<sup>` and `<sub>`, without
  attributes, and not in code, where they stay text. Everything else
  stays escaped, so no styles, scripts or embeds. Not taken: `<mark>`,
  `<abbr>`. The manual's Pages and blocks page should list them.
- **#139 Button and key names in text.** Panoptikum's manuals marked
  button names with `<kbd>` (**My Data**, **Delete my account**), which
  q.e.d. escapes; they became bold. Decided 2026-10-30: keys get their
  own look, written as `<kbd>Ctrl</kbd>+<kbd>S</kbd>` (let through by
  #138) and shown as keycaps in the theme's colors. Button names stay
  bold, as in q.e.d.'s own manual.
- **#143 Site icon.** The icon in the browser tab (favicon) only works
  as `favicon.ico` in Static, which browsers ask for by themselves.
  Idea: a site icon in the Site settings, made from a picture in Media
  (sizes for browsers and phones, `<link rel="icon">`), maybe the logo
  by default.
- **#140 Mastodon verification.** The old sidebar had a "Follow" button
  with `rel="me"`, so Mastodon shows the site as verified on the
  profile. q.e.d. can't set `rel="me"` on a link and has no Mastodon
  icon; the footer now has a plain link. Idea: a Mastodon profile
  (or several "me" links) in the Site settings that adds
  `<link rel="me">` to every page, maybe with a follow link and icon.
- **#144 Sidebar menu: folding groups and post lists.** The old sidebar
  had groups that fold open and closed (Manuals, Blog, Supported By;
  the current page's group open) and a Blog group listing every post.
  q.e.d.'s sidebar shows groups as headings, always open, and posts are
  reached through the Blog page with back and next links. Decided
  2026-10-30: groups fold open and closed (`<details>`, works without
  JavaScript), the current page's group starts open; no post list in
  the sidebar. To check: on or off per site, or always. The old `.html`
  addresses are #134.

#### Bigger features

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

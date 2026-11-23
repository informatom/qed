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

- **#163 A color model beyond daisyUI's single shades.** daisyUI has one
  shade per theme color (plus the text on it), so q.e.d. can offer a
  color only as a full patch (a Hero takes more than half of a full HD
  start screen) or not at all, and text stays base-content. Color
  systems (Radix, Material 3, Refactoring UI) use scales: light shades
  for areas, the solid one for buttons, dark ones for text. To discuss:
  following Material 3's roles (container, on-container, outline
  derived per theme), a Hero and Call to action in a container shade or
  with a border, a height setting for the Hero. Decided 2026-11-23:
  Material 3's roles, a colored border as a choice for big solid areas,
  new defaults may change existing sites. First step built 2026-11-23,
  to evaluate on the docs site: Hero and Call to action outlined by
  default (light container shade, 2 px border in the color, headline in
  an on-container tone; `hero_solid` / `call_to_action_solid` for the
  old fill), a Hero without an image only as high as its text, the
  footer plain by default in light and dark with a 4 px accent line
  along its top (like the plain header). Still open: borders for boxes
  and sections. Findings and sources:
  [Theming notes](theming.md). Related: Phase 3.5 (#159–#162).

## To build

### Phase 3.5: Theming

How the theme's colors are chosen and used (user, 2026-11-20, from
ki.mittenin.at in light mode: text and link text black, primary only in
buttons, a Call to action in secondary as the biggest color patch). For
comparison, daisyUI's roles: primary is "the main color of your brand",
secondary and accent are "optional", neutral is "for not-saturated parts
of UI", base-200 (*shaded*) is the page background "a darker shade, to
create elevations", and base-content is the default text color. One step
at a time, without breaking how existing sites look. Not wanted: soft
tints instead of full colors for big areas (user: "the absolute
opposite").

- **#159 Color choices that show what's chosen.** In Site → Theme →
  Colors the chosen option is only a slightly darker grey chip, so you
  can't see at a glance what's set (user: the main problem). Make the
  choice stand out (e.g. the chip filled in its color, a ring or a check
  mark). Also: the rows have 4, 5 or 6 options, so "Primary" sits at a
  different place in each row; line them up as a table with fixed
  columns (an empty cell where an option doesn't apply), so a column
  shows where a color is used. Plain and Shaded look the same (two
  near-white dots): draw them as base-100 and base-200 with a border.
- **#160 Text in theme colors.** Today every color choice is a fill
  (or a button, or a link's underline); text is always base-content.
  Options for headings, link text, the Hero headline and the Call to
  action title in a theme color. Link text would need a contrast check
  (4.5:1 against the background; the generator knows the theme's hex
  values), because some themes have light primaries: that's why link
  text keeps the base color now (blocks.css).
- **#161 Header: Shaded too.** The Header row offers Plain and the four
  theme colors, the Footer row also Shaded.
- **#162 Defaults by role.** Today's defaults give three brand colors a
  big area each: Hero secondary, Call to action primary, footer accent
  (light). Decide whether primary should lead (buttons, links, current
  menu item, Call to action) and secondary and accent stay small (dates,
  quote bar), and whether a Hero without an image should default to
  shaded, as daisyUI's own Hero example does. New defaults change every
  site that uses them (informatom.com and blog.panoptikum.social in
  production, podcasterei.at, waldgruber.at): write their current
  choices into their site.toml first.

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

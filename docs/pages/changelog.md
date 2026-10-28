+++
title = "Changelog"
description = "What changed in each release of q.e.d."
menu = "footer"
weight = 12
+++

What each release of q.e.d. brought, newest first. Numbers like #043
refer to the [backlog](backlog.md) and [Done](done.md). Installers are on
[GitHub](https://github.com/informatom/qed/releases); the app updates
itself (App → Check for updates).

## 0.9.0 · 2026-10-28

One site on two computers, without the command line.

- **Push and pull** (#129): the Versions tab pushes the site with all
  its versions to a git repository (Codeberg, GitHub, Gitea, your own
  server) and pulls new versions back. Pull merges versions made on both
  computers; if both changed the same page, it changes nothing and says
  which. q.e.d. logs in with its own key, kept in the keychain, and
  checks the server's key on first use, as when publishing. A progress
  bar shows how far it has got. See [Backups](backups.md).
- **Get a site from git**: in the site menu and on the start screen.
  Copies a pushed site to this computer with its whole history.
- **New versions on the remote**: when a site opens, q.e.d. checks its
  remote and offers to pull.
- **Transcripts**: paragraphs at pauses also without speaker names.

## 0.8.0 · 2026-10-28

Phase 2: q.e.d. makes podcast websites now. Everything a podcast
needs lives in the site folder, and the site loads nothing from
anywhere else.

- **Podcast sites** (#066): a Podcast tab with the show's details, cover
  (checked against Apple's rules), categories, support links and a
  lasting `podcast:guid`. New sites can start as a podcast.
- **Episodes** (#067, #022): dated pages below `/episodes` with season,
  number, type, own cover, chapters and a stable guid. *New episode*
  proposes the next number.
- **Audio** (#023): a new Audio tab. Files stay out of the history by
  default, or live elsewhere on the web. A podcast can offer several
  formats (MP3, M4A, Ogg, Opus, FLAC), one file per format and episode;
  publishing waits until every episode has them all.
- **Podcast feeds** (#068): one feed per format at octopod's addresses,
  with Apple's and Podcasting 2.0 tags and chapters.
- **Podlove Web Player** (#049) on episode pages and in the Latest
  episode block, self-hosted, in the theme's colors. Without JavaScript,
  the browser's own player and a chapter list.
- **Blocks for podcasts** (#069, #125): Latest episode (with a link to
  the episode), Subscribe (the Podlove Subscribe Button, self-hosted)
  and Contributors.
- **Contributors** (#047): people entered once with photo, links and
  bio, then picked per episode with their roles. Each gets a page
  listing their episodes.
- **Transcripts** (#048): one WebVTT file per episode, in the player,
  in the feeds and, if you like, as text on the episode page.
- **Share links** (#053): a share row for episode pages (Mastodon,
  Bluesky, email, copy link) without share widgets. It isn't in the
  default layout, since the player's share tab covers it.
- **Episode layout** (#126): `episode.md` sets the order of an episode
  page's parts (player, people, shownotes, chapters, transcript, share
  links) and can add any block to every episode. It sits under the new
  *Templates* heading in the page tree, next to the footer.
- **More of the player** (#128): a playlist of all episodes, a share
  tab (from the current playtime, with embed code) and a subscribe
  button inside the player. It loads without the plain player flashing
  up first and without the page jumping. The share row and the download
  line are left out where the player has them.
- **Video files** (#127): a Video block can play a file from Downloads
  with the browser's own player.
- The Assets tab is now called **Downloads**.
- Fixed: episodes published on the same day were listed in the wrong
  order. The higher number comes first now, in lists and feeds.

## 0.7.0 · 2026-10-27

The last release of phase 1: the CMS and site generator are complete.

- **Shared blocks** (#043): blocks used on several pages, kept once in
  `shared/`. "Make shared" turns any block into one; a list under the
  pages opens them for editing.
- **Drag and drop** for blocks (#043) and in the page tree (#120): drop a
  page between others to change the order, onto a page to move it there.
  The page tree now shows the site's own order.
- **Colors per element** (#054): which theme color the links, buttons,
  Hero, Call to action, boxed columns, content background and footer use,
  with a Reset button.
- **New tabs** (#121): Menu, Theme and Publish have tabs of their own; Site
  settings are grouped under headings. "Main menu in a sidebar" moved to
  Menu.
- **Unsaved changes** (#122): closing the window, opening another site or
  installing an update asks first when something isn't saved.
- **Links may not leave the site folder** (#026): the build stops and names
  the file and the link.
- Fixed: saving Site settings could bring back older colors.
- Fixed: a line starting with inline code (```` ```math ````) hid the
  blocks after it.

## 0.6.0 · 2026-10-27

- Site icons in text (#115): your own logos from `media/icons/`, written
  as `:icon-name:`.
- "Mark links to other sites" (#116): a small icon after links that leave
  the site.
- Smaller copies for images in text (#085), as for image blocks.
- Images open big over the page, without JavaScript (#117).
- In the app, images open big in the Media tab (#118) and PDFs inside the
  app (#119).

## 0.5.0 · 2026-10-27

Everything found while moving podcasterei.at, waldgruber.at and
informatom.com over from Jekyll.

- Static files (#088): a `static/` folder copied into the site as it is,
  with its own tab and a text viewer.
- Footer blocks (#092), a 404 page (#093) and code highlighting (#094).
- New blocks: Page list (#099), Video without tracking (#100); lists with
  pictures as cards or tiles (#103).
- Themes: the theme's colors on the site, custom colors and saved color
  sets (#101), a colored header bar, self-hosted fonts (#102).
- Header and layout: logo sizes, logo instead of title, hiding a page's
  title, column widths (#095–#097), logo at its own size and page width
  (#112), boxed columns (#113), icons in text (#114).
- A hand-made main menu with dropdowns (#111).
- Dates in the site's language (#098); new sites in their language (#104).
- Jekyll addresses keep working through redirects; wide logos; the site's
  own favicon.
- A foldable page tree, folded at first.
- Restoring the whole site from an earlier version (#082).

## 0.4.0 · 2026-10-22

- Link previews for social media and chat apps: Open Graph tags and a
  share image per page (#026).
- A site logo, in the header and as the browser-tab icon (#054); a new app
  icon (#080).
- Close site, back to the start screen (#021).

## 0.3.0 · 2026-10-21

- The name is now q.e.d., everywhere it's shown.
- The Publish dialog shows what changed since the last publish to that
  target (#079).
- Post list block and RSS feeds for blogs (#040).

## 0.2.0 · 2026-10-21

- Publishing asks for a short note on what's new (#079).
- New sites from the app, with a start page (#046, #056).
- Moving and renaming pages, with redirects from the old address and
  links updated everywhere (#046).
- Recent sites, switched from the title menu (#021, #055).
- Updates from GitHub releases, installed from within the app (#059).

## 0.1.0 · 2026-10-21

The first release of the desktop app (Rust, Tauri and Svelte), after an
earlier start as a web app.

- Sites are folders: `site.toml`, Markdown pages with blocks, media and
  downloads. Hand-written formatting survives saving.
- A block editor with Text, Hero, Image, Text with image, Gallery,
  Download, FAQ, Columns (with blocks inside) and Call to action.
- A media library with an image picker; responsive images in several
  sizes.
- The site generator builds into `_site/`, with a preview in the browser,
  menus, a sidebar, page icons and themes.
- Publishing over SFTP, uploading only what changed.
- History with git: every save is recorded; deleted pages come back from
  Versions.
- Installers for macOS, Windows and Linux, built on GitHub.

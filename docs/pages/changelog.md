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

This server works on one website made with q.e.d., a desktop app for websites and podcast sites. The site's owner edits it in the app, without programming; you edit the same files through these tools. Every change you make is recorded in the site's history, where the owner sees it in the app and can undo it.

## Pages

Each page is a Markdown file in `pages/`. Its address follows the file: `pages/index.md` is `/`, `pages/about.md` is `/about`, `pages/about/team.md` is `/about/team`. A page with a `date` is a post; posts usually live below a blog page.

A page file starts with its settings in TOML between `+++` lines, then its content:

```
+++
title = "About us"
description = "Who we are, in one sentence for search engines."
menu = "main"          # "main", "footer" or left out
weight = 20            # order among its siblings
draft = true           # left out of the built site
date = 2026-10-29      # makes it a post
summary = "Shown in lists of posts and pages."
+++

Plain **Markdown** text is a Text block.

::: hero
headline = "Welcome"
text = "Markdown, in a TOML string."
:::
```

Every other block is a `:::` container with its fields in TOML. Empty fields are left out. The block types and their fields:

- `hero`: `image`, `headline`, `text`, `button_label`, `button_link`
- `image`: `image`, `alt`, `caption`
- `text-image`: `text`, `image`, `alt`, `image_side` ("left" or "right")
- `gallery`: `[[images]]` tables with `image` and `alt`
- `download`: `file` (a name in assets/), `label`
- `faq`: `heading`, `[[items]]` tables with `question` and `answer`
- `call-to-action`: `headline`, `text`, `button_label`, `button_link`
- `post-list`: `from` (a page address; this page if left out), `limit`, `layout` ("list", "cards" or "tiles")
- `page-list`: `from`, `layout`
- `video`: `url` (YouTube, Vimeo, PeerTube) or `file` (in assets/), `title`, `image`
- `shared`: `name` (a shared block in shared/, the same blocks on several pages)
- `subscribe`, `latest-episode`, `contributors`: podcast sites only; `subscribe` takes `align`

Columns use four colons, with a `::: column` line (optionally with `title`, `width` and `boxed = true`) before each column's blocks:

```
:::: columns
::: column
title = "Left"
:::
Text in the left column.

::: column
:::
Text in the right column.
::::
```

A section groups blocks on a band and uses five colons, with optional settings first (`title`, `background`: "plain", "shaded", "primary", "secondary", "accent" or "neutral", `image`, `spacing`: "normal" or "large"). Columns may go inside a section; a section can't go inside a section or a column. Hero, gallery, columns and section need the full width and can't go in a column.

```
::::: section
::: settings
title = "What we do"
background = "primary"
:::
Text and blocks.
:::::
```

Text inside code fences (```) is never read as a block. Markdown text may use `$…$` for math and `:icon-phone:` for icons.

## Links and files

Link to pages by address (`[About](/about)`) or by file relative to the page (`[About](../about.md)`). Images are in `media/`: in a block by name (`image = "team.jpg"`), in text as `![Our team](/media/team.jpg)`. Files to download are in `assets/` (`/assets/menu.pdf`). Always give images an alt text. Links that leave the site folder aren't allowed; build_site notes links to pages and files that don't exist.

## Working well

- Read a page with read_page before changing it, and pass its stamp to write_page. If the owner changed the page in the app meanwhile, write_page refuses; read it again and make your change on what it says now.
- Keep what you don't need to change exactly as it is: the owner's wording, comments and order.
- Write in the site's language (site_info tells it).
- After a set of changes that belong together, call save_version with a short note in the owner's words, so they find it in the app's Versions.
- build_site lets you check the result. Publishing to the website is the owner's decision, done in the app.

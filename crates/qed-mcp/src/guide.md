# Working on a q.e.d. site

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

All page settings; only `title` is required, and settings left out take their default:

- `title`: the page's title.
- `description`: one sentence for search engines.
- `summary`: a short teaser (Markdown) for post lists and feeds.
- `image`: a file in `media/` that link previews show (on social media, in chat apps); without one, the page's first image.
- `date`: `2026-10-29`, without quotes; makes the page a post.
- `authors`: a list of names, `["Stefan Haslinger"]`. `tags`: a list of words.
- `draft = true`: left out of the built site.
- `hide_title = true`: no title above the content, for a page that opens with a heading of its own.
- `menu`: "main" or "footer", for the menus q.e.d. makes by itself. `weight`: order among sibling pages, lower first. `icon`: an icon name (see Icons below), shown in menus.
- `back_next = true`: the pages below this one get links to the one before and after ("Older"/"Newer" for posts).
- `redirect_from`: earlier addresses of the page, which then lead to it: `["/2016/10/02/beginn.html", "/old-about"]`. Slashes at either end don't matter. An address ending in `.html` becomes that file, any other a folder with an `index.html`. Only for addresses of pages: an old address of a PDF or image can't be a redirect.
- `episode`: a table that makes the page a podcast episode; edit episodes in the app.

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

Text inside code fences (```) is never read as a block. Markdown text may use `$…$` for math and `:icon-phone:` for icons; only the names listed under Icons work, and build_site notes any other.

## Footer, shared blocks, settings

Some files hold blocks but aren't pages; read_page and write_page take them too, and list_pages names them at the end:

- `footer.md`: the blocks at the bottom of every page (`footer.en.md` for a further language). Columns work well there. The footer also shows the footer menu, the pages with `menu = "footer"` (imprint, privacy and the like), below those blocks.
- `shared/<name>.md`: a shared block, the same blocks on several pages; a page shows it with `::: shared` and `name = "<name>"`.
- `episode.md`: the layout of podcast episode pages.

The site's own settings are `site.toml`, read with read_settings and written with write_settings. Keep everything you don't change as it is. The settings:

- `title`, `description`, `language` (the main language's code), `base_url` (the web address, `"https://example.com/"`; needed for feeds and the sitemap).
- `logo` and `icon`: images in `media/` by name; `icon` is the browser-tab icon (an SVG, a square PNG or an ICO).
- `theme`: one of the names listed under Themes below, or `"custom"` with a `[custom_theme]` table (made in the app's Theme tab). `[dark]` with its own `theme` gives the site a dark side that follows the visitor's system.
- `navigation`: "top" or "sidebar" for the main menu. `header`: "plain", "primary", "secondary", "accent" or "neutral". `logo_size`: "small", "medium", "large" or "original". `page_width`: "normal", "wide" or "full". `title_in_header`, `search`, `mark_external_links`: true or false.
- `[colors]`: which theme color goes where: `links`, `buttons`, `call_to_action`, `hero`, `boxes`, `content`, `footer` (`footer = "shaded"`, for example).
- `profiles`: the owner's profiles elsewhere, `["https://fosstodon.org/@name"]`. Every page links them with `rel="me"`, so Mastodon shows the site as verified on the profile.
- `[fonts]`: `text` and `headings`, fonts the owner added in the app.
- `[[main_menu]]`: a menu arranged by hand, entries in order. Each has `page` (a page's address, `""` for the start page) or `link` (any address), an optional `title`, and for a dropdown `[[main_menu.items]]` entries of its own. Without any, the menu is made from the pages' `menu = "main"`.

```
[[main_menu]]
title = "About"
page = ""

[[main_menu]]
title = "Manuals"

[[main_menu.items]]
page = "listener-manual"
```

- `[languages.en]` with `title` and `description`: a further language (translations are `about.en.md` beside `about.md`).
- `[podcast]`: a podcast's settings; leave them to the app's Podcast tab.

## Links and files

Link to pages by address (`[About](/about)`) or by file relative to the page (`[About](../about.md)`). Images are in `media/`: in a block by name (`image = "team.jpg"`), in text as `![Our team](/media/team.jpg)`. Files to download are in `assets/` (`/assets/menu.pdf`). Files the website needs exactly as they are, at an address of their own, are in `static/`: `static/talks/sub8.html` is at `/talks/sub8.html`. Use it to keep old addresses of files working (a PDF at `/pdf/33C3.pdf`, old slide shows with their scripts), since redirects only work for pages. A folder in `static/` may share its address with a page (`static/talks/` beside the page `/talks`), as long as it has no `index.html`; links to static files count as working. Always give images an alt text. Links that leave the site folder aren't allowed; build_site notes links to pages and files that don't exist.

## Working well

- Read a page with read_page before changing it, and pass its stamp to write_page. If the owner changed the page in the app meanwhile, write_page refuses; read it again and make your change on what it says now.
- Keep what you don't need to change exactly as it is: the owner's wording, comments and order.
- Write in the site's language (site_info tells it).
- After a set of changes that belong together, call save_version with a short note in the owner's words, so they find it in the app's Versions.
- build_site lets you check the result. Publishing to the website is the owner's decision, done in the app.

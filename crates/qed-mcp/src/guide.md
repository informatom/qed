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
- `menu`: "main" or "footer", for the menus q.e.d. makes by itself. `weight`: order among sibling pages, lower first. `icon`: an icon name (see Icons below; find_icons finds them), shown in menus.
- `back_next = true`: the pages below this one get links to the one before and after ("Older"/"Newer" for posts).
- `pdf = false`: the page is left out of the site's PDF (Make PDF of the site in the Downloads tab); leave it out otherwise.
- `redirect_from`: earlier addresses of the page, which then lead to it: `["/2016/10/02/beginn.html", "/old-about"]`. Slashes at either end don't matter. An address ending in `.html` becomes that file, any other a folder with an `index.html`. Only for addresses of pages: an old address of a PDF or image can't be a redirect.
- `episode`: a table that makes the page a podcast episode; see Podcasts below.

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

Text inside code fences (```) is never read as a block. Markdown text may use `$…$` for math and `:icon-phone:` for icons; find_icons finds the names. Icons only work in Markdown text: Text blocks, the `text` of hero, call-to-action and text-image, image captions, FAQ answers and `summary`. In titles, headlines, button labels, questions and column titles `:icon-…:` stays as text; for an icon beside a page in menus, set the page's `icon`. write_page notes icons that would stay text.

## Footer, shared blocks, settings

Some files hold blocks but aren't pages; read_page and write_page take them too, and list_pages names them at the end:

- `footer.md`: the blocks at the bottom of every page (`footer.en.md` for a further language). Columns work well there. The footer also shows the footer menu, the pages with `menu = "footer"` (imprint, privacy and the like), below those blocks.
- `shared/<name>.md`: a shared block, the same blocks on several pages; a page shows it with `::: shared` and `name = "<name>"`.
- `episode.md`: the layout of podcast episode pages.

The site's own settings are `site.toml`, read with read_settings and written with write_settings. Keep everything you don't change as it is. The settings:

- `title`, `description`, `language` (the main language's code), `base_url` (the web address, `"https://example.com/"`; needed for feeds and the sitemap).
- `logo` and `icon`: images in `media/` by name; `icon` is the browser-tab icon (an SVG, a square PNG or an ICO).
- `theme`: one of the names listed under Themes below, or `"custom"` with a `[custom_theme]` table (made in the app's Theme tab). `[dark]` with its own `theme` gives the site a dark side that follows the visitor's system.
- `navigation`: "top" or "sidebar" for the main menu. `header`: "plain", "primary", "secondary", "accent" or "neutral". `logo_size`: "small", "medium", "large" or "original". `page_width`: "normal", "wide", "extra-wide" or "full". `title_in_header`, `search`, `mark_external_links`: true or false.
- `[colors]`: which theme color goes where: `links`, `headings` (`"plain"` for the text color, or a theme color in a deep tone; primary by default), `buttons`, `call_to_action`, `hero`, `boxes`, `content`, `footer` (`footer = "shaded"`, for example). Hero and Call to action are drawn as a light shade of their color inside a border in it; `hero_solid = true` or `call_to_action_solid = true` fills them with the color instead.
- `profiles`: the owner's profiles elsewhere, `["https://fosstodon.org/@name"]`. Every page links them with `rel="me"`, so Mastodon shows the site as verified on the profile.
- `[fonts]`: `text` and `headings`, fonts the owner added in the app.
- `[[main_menu]]`: a menu arranged by hand, entries in order. Each has `page` (a page's address, `""` for the start page) or `link` (any address), an optional `title` (with `titles = { en = "…" }` for further languages), and for a dropdown `[[main_menu.items]]` entries of its own. Without any, the menu is made from the pages' `menu = "main"`.

```
[[main_menu]]
title = "About"
page = ""

[[main_menu]]
title = "Manuals"

[[main_menu.items]]
page = "listener-manual"
```

- `[languages.en]` with `title` and `description`: a further language; see Languages below.
- `[podcast]`: a podcast's settings; see Podcasts below.

## Podcasts

A site can have one podcast. start_podcast turns it on (site_info says whether it's on); its feed is `/episodes.mp3.rss`, the address jekyll-octopod used too, so a podcast moving over keeps its subscribers. Its settings are `[podcast]` in site.toml, all optional:

```
[podcast]
description = "What the podcast is about; the site's description if left out."
author = "Shown below the name in podcast apps"
owner_name = "Who directories write to"
owner_email = "podcast@example.com"
cover = "cover.jpg"            # a square image in media/, 1400–3000 px
categories = ["Technology"]    # Apple's; a subcategory as "Sports > Running"
copyright = "CC BY 4.0"
explicit = false

[[podcast.funding]]
url = "https://liberapay.com/name"
title = "Support us"

[[podcast.people]]             # the team; new episodes start with it
contributor = "stefan-haslinger"
role = "host"
```

Don't write `guid`, `formats` or `audio_files` yourself. q.e.d. makes the `guid` from the feed address as soon as the site has a `base_url`, writes it into `[podcast]`, and never changes it afterwards; keep it as it is when you write the settings.

Contributors are the people in the podcast: list_contributors, save_contributor and delete_contributor. Save them before naming them in `[[podcast.people]]` or an episode. Roles: `host`, `co-host`, `guest-host`, `guest`, `narrator`, `producer`, `audio-editor`, `sound-designer`, `composer`, `transcriber`, `cover-art-designer`.

Episodes are posts below `/episodes`; make each with create_episode. For one carried over from an old site, import its audio first, then give create_episode everything it takes in one call: `date`, `number`, the old name as `slug`, `summary`, the old addresses as `redirect_from`, the old `guid`, `people` and the `audio` file names (it reads their format, size and duration). Then add show notes and chapters to the text it returns and write that with write_page; don't write the page anew from scratch. Their settings are an `[episode]` table after the page's own:

```
+++
title = "Episode 3"
summary = "A short teaser, such as an old subtitle; lists and feeds show it."
date = 2016-05-06
redirect_from = ["/2016/05/06/episode3.html"]

[episode]
number = 3
season = 1                     # optional
guid = "https://example.com/2016/05/06/episode3.html"
duration = "01:00:38"

[[episode.audio]]
format = "mp3"
file = "episode3.mp3"          # in the site's audio
length = 58224640              # size in bytes

[[episode.chapters]]
start = "00:00:00.000"
title = "Intro"

[[episode.people]]
contributor = "stefan-haslinger"
role = "host"
+++

## Show notes

Text and blocks, as on any page.
```

- `guid`: the episode's identity in the feed. create_episode makes one; for an episode carried over, give it the guid the old feed gave it, so apps don't show it again as new. Never change it afterwards.
- Audio goes in with import_file (library `audio`); audio_info gives its `format`, `length` and `duration`. `type` is `full` (left out), `trailer` or `bonus`; `explicit` and `image` override the podcast's; `image` is an own cover in media/ with the cover's rules (square JPEG or PNG, 1400–3000 px), so a smaller picture belongs in the show notes or in the page's own `image` setting instead.
- `transcript`: a WebVTT file imported into the library `transcripts`.
- Chapters go in order of their `start`; each may have a `link`.

write_page refuses an episode with mistakes (a time that isn't one, chapters out of order or two at the same time, a contributor who doesn't exist, a guid another episode has, an `image` that isn't a valid cover). The player, the people, the show notes and the chapters on episode pages come from `episode.md`, the episode layout; the blocks `latest-episode` and `subscribe` show the newest episode and a Subscribe button on other pages.

## Languages

A site can be in several languages: the main one (`language` in site.toml) at `/`, each further one below its code (`/en/`). For a site in two languages:

1. Add the language to site.toml with write_settings, with the site's title (and description) in it:

   ```
   [languages.en]
   title = "My site"
   ```

2. Make each page's translation with create_translation (`pages/about.md` and `en` give `pages/about.en.md`, at `/en/about`); the start page's is `pages/index.en.md`, at `/en/`. Don't write `.en.md` files any other way.
3. A translation starts as a draft copy. Translate its title and text with write_page and take out `draft = true`, or it isn't on the website.
4. The footer in that language is `footer.en.md` (write it with write_page); without one, pages in that language show the main footer.

Menus, post lists and feeds of each language show only its own pages; a page without a translation isn't there in that language. A switch in the header leads between the languages by itself, so don't make links or menu entries for that. A hand-made menu is the same in every language: page entries lead to the page's translation and show its title; give links, groups and renamed pages their title in each further language with `titles = { en = "About us" }`. Links in a translation point to the main language's file (`[About](about.md)`) and lead to its translation where there is one.

## Links and files

Link to pages by address (`[About](/about)`) or by file relative to the page (`[About](../about.md)`). Images are in `media/`: in a block by name (`image = "team.jpg"`), in text as `![Our team](/media/team.jpg)`. Files to download are in `assets/` (`/assets/menu.pdf`). Files the website needs exactly as they are, at an address of their own, are in `static/`: `static/talks/sub8.html` is at `/talks/sub8.html`. Use it to keep old addresses of files working (a PDF at `/pdf/33C3.pdf`, old slide shows with their scripts), since redirects only work for pages. A folder in `static/` may share its address with a page (`static/talks/` beside the page `/talks`), as long as it has no `index.html`; links to static files count as working. Always give images an alt text. Where an image is cut to fit (squares in a Gallery and in tiles, the hero image), it's cut around the middle, or around the spot the owner chose for that image in the Media tab; the app keeps that in `media.toml` (`["portrait.jpg"]` with `focus = { x = 50, y = 20 }`, percent from the left and the top). Leave that file to the app; delete_file removes an image's entry. Links that leave the site folder aren't allowed; build_site notes links to pages and files that don't exist.

## Working well

- Read a page with read_page before changing it, and pass its stamp to write_page. If the owner changed the page in the app meanwhile, write_page refuses; read it again and make your change on what it says now.
- Keep what you don't need to change exactly as it is: the owner's wording, comments and order.
- Write in the site's language (site_info tells it).
- After a set of changes that belong together, call save_version with a short note in the owner's words, so they find it in the app's Versions.
- build_site lets you check the result. Publishing to the website is the owner's decision, done in the app.

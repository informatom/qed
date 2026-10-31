+++
title = "Site settings"
description = "The Site tab: title, language, description and web address, further languages, the logo and the site icon, links to other sites and your profiles there, site search and the page width."
menu = "main"
weight = 32
summary = "Title, language, web address, logo, icon, search and page width."
+++

The **Site** tab holds what belongs to the whole website. Change what
you like, then press **Save**; **Discard** forgets the changes.

## General

- **Title**: the site's name, in the header of every page and in the
  browser tab.
- **Language**: a short code such as `en`, `de` or `de-AT`. It tells
  browsers and search engines the language and sets how dates are
  written: with `de-AT`, January is *Jänner*.
- **Description**: one or two sentences about the site, shown by search
  engines.
- **Web address**: where the site will be online, such as
  `https://example.com/`. Feeds, the sitemap and share links need it;
  fill it in before publishing.

## Languages

A site can be in more than one language: German at `/` and English at
`/en/`, for example. The language under **General** is the main one;
add more here with their code (**Add a language**), such as `en`, `fr`
or `pt-br`. Each can have the site's **title** and **description** in
it; left empty, the main ones are used.

Then translate pages one by one: open a page and press **Translate**
next to the language's code above it (see
[Pages and blocks](pages-and-blocks.md)). A page without a translation
simply isn't there in that language. Menus, post lists and feeds of
each language show only its own pages, and a switch in the header
(*DE EN*) leads to the same page in the other language, or to that
language's start page where there's none.

A language that still has translations can't be removed; delete those
first.

## Logo

Pick a picture from Media; it's shown in the header of every page and,
without a site icon, as the icon in the browser tab. An SVG file stays
sharp at any size.

- **Size**: **Small**, **Medium**, **Large**, or **Original** for the
  picture's own size (up to 112 pixels high), which keeps small bitmap
  logos sharp.
- **Show the site title next to the logo**: switch it off when the logo
  already shows the name.

## Site icon

The small picture in the browser tab, in bookmarks, and on a phone's
home screen when someone adds the site there. A wide logo makes a poor
icon, so it can be a picture of its own: add it in Media, then pick it
here. q.e.d. uses it as it is, so it must be one of these:

- an **SVG**, sharp at any size;
- a square **PNG**, at least 180 × 180 pixels so it's sharp on phones;
- an **ICO** file, the classic `favicon.ico`.

Saving refuses other pictures, and a PNG that isn't square. Without a
site icon, the browser tab shows the logo.

## Links

**Mark links to other sites** gives links in text that lead to other
websites a small arrow, so visitors know they're leaving. Menus and
buttons stay as they are.

**Your profiles elsewhere** lists your profiles on other sites, such as
`https://mastodon.social/@name`: **Add a profile**, paste its address.
Every page then names them as yours, invisibly (`rel="me"`). Mastodon
uses that to show your website as verified, with a green check, on your
profile:

1. Add the Mastodon profile here, save and publish.
2. On Mastodon, edit your profile and put your website's address into
   one of the extra fields.
3. Mastodon looks at that page, finds the profile named there, and
   marks the field as verified. It can take a few minutes.

To also show a *Follow me* link on the site, put an ordinary link into
the footer or a page.

## Search

**Site search** puts a search button into the header (also Ctrl+K). It
finds words in all pages, posts and episodes. The search runs in the
visitor's browser, from files on your own site; nothing goes to other
services. It's on for new sites.

## Layout

**Page width**: **Normal** suits text, **Wide** gives columns room,
**Full** uses the whole window.

::: call-to-action
headline = "Colors, fonts and menus"
text = "How the site looks is in the Theme tab, and the menu in the Menu tab."
button_label = "Colors and fonts"
button_link = "colors-and-fonts.md"
:::

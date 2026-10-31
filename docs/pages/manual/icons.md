+++
title = "Icons"
description = "The more than 6,000 icons q.e.d. brings along, for text and for menus: finding one, writing it, and adding your own."
menu = "main"
weight = 25
summary = "Over 6,000 icons for text and menus, and your own."
+++

Icons go into text and next to pages in menus. They're drawn in the
text's color, as tall as the letters, and need nothing from other
websites. q.e.d. brings the [Tabler icons](https://tabler.io/icons):
more than 5,000 drawn as outlines and 1,000 filled ones, brand logos
among them, such as :icon-brand-mastodon: Mastodon.

## Finding an icon

Search for it by what it shows. The search knows more than the names:
*envelope* finds :icon-mail: `mail`, *house* finds :icon-home: `home`.
You can search in two places in q.e.d.:

- under **Icons in text** below every text field, where a click copies
  the code to paste into the text
- in a page's **Settings** tab, at **Icon**

[tabler.io/icons](https://tabler.io/icons) shows all icons with their
names, too. A filled icon's name ends in `-filled`: :icon-heart: `heart`
and :icon-heart-filled: `heart-filled`.

## In text

Write the icon's name between colons and `icon-`: `:icon-phone:` gives
:icon-phone:. It works in every block with text: a Text block, a column,
an FAQ answer, the footer. A line like

```text
:icon-phone: 01 234 56 78 · :icon-mail: hello@example.com
```

shows as :icon-phone: 01 234 56 78 · :icon-mail: hello@example.com.

A name q.e.d. doesn't know stays as text on the page, and **Build**
notes it, so a typo doesn't go unnoticed.

## Next to a page in menus

A page's **Settings** tab has **Icon**: search for one and click it, and
it's shown next to the page in the menus. **No icon** takes it away
again.

## Your own icons

For logos and pictures that aren't among them, such as the logo of a
program you write about: put an image file (SVG at best) into the
`icons` folder inside the site's `media` folder. **Show in file manager**
in the Media tab opens it. `rails.svg` there is written `:icon-rails:`.
Your own icons keep their own colors.

## Some to start with

| Icon | Write | | Icon | Write |
| --- | --- | --- | --- | --- |
| :icon-mail: | `:icon-mail:` | | :icon-phone: | `:icon-phone:` |
| :icon-map-pin: | `:icon-map-pin:` | | :icon-clock: | `:icon-clock:` |
| :icon-calendar: | `:icon-calendar:` | | :icon-home: | `:icon-home:` |
| :icon-info-circle: | `:icon-info-circle:` | | :icon-help-circle: | `:icon-help-circle:` |
| :icon-download: | `:icon-download:` | | :icon-link: | `:icon-link:` |
| :icon-rss: | `:icon-rss:` | | :icon-microphone: | `:icon-microphone:` |
| :icon-headphones: | `:icon-headphones:` | | :icon-book: | `:icon-book:` |
| :icon-heart-filled: | `:icon-heart-filled:` | | :icon-star-filled: | `:icon-star-filled:` |
| :icon-brand-mastodon: | `:icon-brand-mastodon:` | | :icon-brand-github: | `:icon-brand-github:` |

## Sites from before version 0.15

Until version 0.14, q.e.d. had 72 icons from
[Lucide](https://lucide.dev). Their names keep working: a page with
`:icon-house:` or the icon `house` now shows :icon-home: `home`. New
icons are picked by their Tabler names.

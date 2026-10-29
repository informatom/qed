+++
title = "Moving a site over"
description = "Bring a website or a podcast to q.e.d. from another computer or another system, keep its old addresses working, and switch over when it's ready."
menu = "main"
weight = 80
summary = "From another computer or another system, without losing visitors."
+++

## From another computer

A q.e.d. site moves with its folder. If it's backed up on a git server,
**Get a site from git** brings it over with all its versions (see
[Backups](backups.md)); a copy of the folder, from a USB stick for
example, opens with **Open site folder…**.

## From another system

There's no import from WordPress, Jekyll or website builders: a site
moves over by hand, page by page. That's also a good moment to tidy up.

1. **Make a new site** (**New site…**) with the same title and language,
   and fill in the [Site settings](site-settings.md), with the web
   address the site will have.
2. **Bring the pictures and files**: pictures into Media, files to
   download into Downloads ([Pictures and files](pictures-and-files.md)).
3. **Make the pages**, each at its old address where you can: the
   **New page** dialog shows the address it will get, and
   :icon-folder-input: moves or renames a page. Copy the text over and
   add headings, links and pictures again.
4. **Rebuild the look**: a theme, the colors of your logo and your fonts
   ([Colors and fonts](colors-and-fonts.md)), and the menu
   ([Menus](menus.md)).
5. **Preview** (:icon-eye:) and click through. The list of missing links
   shows what's still to do.

Files that must keep their exact old address, such as
`/pdf/price-list.pdf`, go into **Static**, in a folder of the same name:
the website serves them as they are.

## Old addresses

Links from other websites, search engines and bookmarks point to the old
addresses. Where a page keeps its address, nothing breaks. When a page
moves or gets a new name in q.e.d., the old address leads to the new one
by itself.

Old addresses that a q.e.d. page can't have, such as ones ending in
`.html`, can be kept too, though not in the app yet: in the page's file,
a line such as `redirect_from = ["2022/11/11/episode51.html"]` at the top,
among the settings, sends visitors from there to the page. Ask for help
if that's new to you.

## A podcast

Podcast apps must keep finding new episodes, and shouldn't show the old
ones as new.

- **From jekyll-octopod**: q.e.d.'s feeds are at the same addresses
  (`/episodes.mp3.rss`) and its audio too (`/episodes/episode1.mp3`), so
  subscribers notice nothing. 3 Schweinehunde moved over this way.
- **From other systems** (Podlove Publisher, a podcast host): the feeds
  had other addresses, and the episodes' identities (*guids*) have to be
  carried over so apps don't show every episode as new. An import that
  does this is planned; until then, move a podcast with help.

## Switching over

Publish the new site to your webspace once everything is in place
([Publishing](publishing.md)). If it goes to a new webspace, your
domain's provider points the domain there; until then, the old site
stays online.

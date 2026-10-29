+++
title = "Getting started"
description = "Install q.e.d., make a new site or open one, find your way around the app and look at the result in your browser."
menu = "main"
weight = 10
summary = "Install q.e.d., make or open a site, find your way around, look at the result."
+++

## Install

The installers are on the
[releases page](https://github.com/informatom/qed/releases/latest): a
`.dmg` for macOS, an `.exe` for Windows, a `.deb` and an `.AppImage` for
Linux. The [q.e.d. website](https://qed.informatom.com/) says how to get
past the first warning of your system. q.e.d. looks for a newer version
each time it starts and asks before installing one.

::: call-to-action
headline = "Your site is a folder"
text = "Everything a site consists of is in one folder on your computer: its pages, pictures, settings and history. There's no account and no server behind q.e.d.; the folder is the site. Keep it where your other documents are, and back it up like them."
button_label = "Backups"
button_link = "backups.md"
:::

## Make a site, or open one

When no site is open, q.e.d. offers three ways in:

:::: columns
::: column
title = "New site…"
boxed = true
:::

:icon-file-plus: Asks for the site's title, its language and where to
put its folder. q.e.d. makes the folder with a start page and starts its
history.

::: column
title = "Open site folder…"
boxed = true
:::

:icon-folder: Opens a site that's already on your computer, made with
q.e.d. here or elsewhere.

::: column
title = "Get a site from git…"
boxed = true
:::

:icon-git-branch: Brings a site over from a server where it was backed
up; see [Backups](backups.md).
::::

Later, the site's title at the top of the window is a menu: switch to a
recently opened site, open another one, make a new one or close it.

## Finding your way around

The tabs below the title hold everything about the open site:

- **Pages**: the site's pages, posts and shared blocks, and the editor
  for the one you pick. See [Pages and blocks](pages-and-blocks.md).
- **Media**, **Downloads** and **Static**: pictures, files to download
  and files the website needs as they are. See
  [Pictures and files](pictures-and-files.md).
- **Site**: title, description, language, the web address, logo, search
  and page width. See [Site settings](site-settings.md).
- **Menu**: the sidebar, and a menu made by hand if the automatic one
  isn't what you want. See [Menus](menus.md).
- **Theme**: colors and fonts. See [Colors and fonts](colors-and-fonts.md).
- **Publish**: build the website and put it online. See
  [Publishing](publishing.md).
- **Versions**: the site's history. See [Versions](versions.md).
- **App**: settings for q.e.d. itself: its own colors (all screenshots
  here use *catppuccin-mocha*) and updates.

A podcast site has a few more: **Podcast**, **Audio**, **Transcripts**
and **Contributors**. See [Podcasts](podcasts.md).

::: image
image = "manual/app-tab.png"
alt = "The App tab of q.e.d.: a grid of color themes for the app, catppuccin-mocha picked, and below it the version and a button to check for updates."
caption = "The **App** tab: q.e.d.'s own colors, and updates."
:::

## Look at the result

:icon-eye: The round button with the eye, in the top right corner, builds
the site and opens it in your browser, at the page you're editing.
Nothing is put online by that: the preview runs on your computer only.
Change something, save, and press it again to see the change.

## Saving

:icon-save: Nothing is saved until you press **Save**. A dot on a tab and
the words *Unsaved changes* say there's something to save, and
**Discard** throws it away. Every save is kept in the site's history, so
you can always go back; see [Versions](versions.md).

## This manual

:icon-life-buoy: **Manual**, next to the preview button, opens this
manual in a window of its own, for the version of q.e.d. you have,
without an internet connection. Some places in the app link to the page
that explains them.

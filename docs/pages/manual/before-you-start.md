+++
title = "Before you start"
description = "What you need for a website made with q.e.d.: a computer, webspace and a domain; for a podcast, room for the audio. And what you don't need."
menu = "main"
weight = 8
summary = "A computer, webspace and a domain; nothing to install on the server."
+++

## What you need

:::: columns
::: column
title = "A computer"
boxed = true
:::

:icon-house: With macOS (Apple Silicon or Intel), Windows, or Linux
(Ubuntu 22.04 or newer, or a distribution of the same age). q.e.d. runs
there, and your site lives there as well: its pages, pictures and
history.

::: column
title = "Webspace"
boxed = true
:::

:icon-globe: Space on a web server that you can reach over **SFTP**, as
almost every hosting package offers: a server name, a user name and a
password or key ([Publishing](publishing.md)). A website from q.e.d.
needs no more than that.

::: column
title = "A domain"
boxed = true
:::

:icon-link: An address such as `example.com`, usually from the same
provider as the webspace. It goes into the **Web address** in the
[Site settings](site-settings.md).
::::

For a **podcast**, the webspace needs room for the audio: an hour of
MP3 is about 50 to 80 MB per episode and format. Or the audio stays at
a podcast host ([Podcasts](podcasts.md)).

For **backups** on another computer, and for working on one site from
two computers, a git server such as Codeberg or GitHub helps
([Backups](backups.md)). It's optional.

## What you don't need

- **Nothing on the server** but files: no PHP, no database, no
  WordPress, no updates to install there. A website that is only files
  can't be broken into through its software, and it's fast.
- **No programming** and no command line: everything is made in the
  app.
- **No account** with q.e.d. and no subscription. The app is free, and
  it works offline; only publishing needs the internet.
- **No other services** for fonts, the player, search or statistics:
  your website brings everything along and sends your visitors' data to
  no one.

::: call-to-action
headline = "Ready?"
text = "Install q.e.d. and make your first site."
button_label = "Getting started"
button_link = "getting-started.md"
:::

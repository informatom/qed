+++
title = "q.e.d."
description = "A desktop app that turns a folder of pages into a website, and publishes it to your own webspace."
+++

::: image
image = "mascot-banner.jpeg"
alt = "The q.e.d. robot at a desk, holding a microphone and a tablet with a website on it. Above it: q.e.d., desktop app, static site generator, podcast publishing."
:::

::: hero
headline = "Your website, from a folder of pages"
text = "q.e.d. is a desktop app that turns Markdown pages into a static website and publishes it to your own webspace. No server, no subscription, no static site generator to learn."
button_label = "Read the blog"
button_link = "/blog"
:::

q.e.d. is open source and in early development. It's built in the open with
[Claude Code](https://claude.com/claude-code), in Rust, Tauri, Svelte and
TypeScript.

## For podcasters

q.e.d. also makes podcast websites, and keeps everything on your own
webspace: no hosting service, no tracking, nothing loaded from anywhere
else.

- **Episodes** with season, number, chapters, people and transcripts,
  the shownotes written like any other page.
- **Feeds** for every audio format you offer (MP3, M4A, Ogg, Opus,
  FLAC), with Apple's and Podcasting 2.0 tags. If your old site was made
  with jekyll-octopod, your listeners' apps keep working.
- **The Podlove Web Player** with chapters, transcript, playlist and
  sharing, and the **Podlove Subscribe Button**, both self-hosted.
- **Contributors** entered once, with a page each listing their
  episodes.

This site is a q.e.d. site itself: a `site.toml` and a `pages/` folder, in the
`docs/` folder of the [q.e.d. repository](https://github.com/informatom/qed).

## Download

The installers for the latest version are on the
[releases page](https://github.com/informatom/qed/releases/latest). They
aren't signed yet, so your system asks once before the first start.

:::: columns
::: column
title = "macOS"
:::

Download the `.dmg`, open it and drag q.e.d. into Applications. It runs on
Apple Silicon and Intel Macs.

The first time, right-click q.e.d. in Applications and choose **Open**.

::: column
title = "Windows"
:::

Download the `.exe` and run it.

When Windows shows “Windows protected your PC”, choose **More info**, then
**Run anyway**.

::: column
title = "Linux"
:::

On Debian and Ubuntu, download the `.deb` and install it with

```sh
sudo apt install ./qed_*.deb
```

On any other distribution, download the `.AppImage`, make it executable
(`chmod +x qed_*.AppImage`) and run it.
::::

::: call-to-action
headline = "Learning along the way"
text = "The blog explains the Rust, Svelte and TypeScript behind q.e.d., step by step, for people who know other languages."
button_label = "Start with the first post"
button_link = "/blog/2026-10-21-one-title-three-languages"
:::

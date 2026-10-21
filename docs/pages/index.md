+++
title = "qed"
description = "A desktop app that turns a folder of pages into a website, and publishes it to your own webspace."
+++

::: hero
headline = "Your website, from a folder of pages"
text = "qed is a desktop app that turns Markdown pages into a static website and publishes it to your own webspace. No server, no subscription, no static site generator to learn."
button_label = "Read the blog"
button_link = "/blog"
:::

qed is open source and in early development. It's built in the open with
[Claude Code](https://claude.com/claude-code), in Rust, Tauri, Svelte and
TypeScript.

This site is a qed site itself: a `site.toml` and a `pages/` folder, in the
`docs/` folder of the [qed repository](https://github.com/informatom/qed).

## Download

The installers for the latest version are on the
[releases page](https://github.com/informatom/qed/releases/latest). They
aren't signed yet, so your system asks once before the first start.

:::: columns
::: column
title = "macOS"
:::

Download the `.dmg`, open it and drag qed into Applications. It runs on
Apple Silicon and Intel Macs.

The first time, right-click qed in Applications and choose **Open**.

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
text = "The blog explains the Rust, Svelte and TypeScript behind qed, step by step, for people who know other languages."
button_label = "Start with the first post"
button_link = "/blog/2026-10-21-one-title-three-languages"
:::

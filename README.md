# q.e.d. 🤖

A desktop app for your own website or podcast site, made entirely in the
app, without programming: write pages, add pictures and episodes, choose
colors and fonts, and q.e.d. builds a fast static website and publishes it
to your own webspace. Built for people who want their own site or podcast
without learning a static site generator.

🌐 Website: [qed.informatom.com](https://qed.informatom.com/) ·
📖 [Manual](https://qed.informatom.com/manual/) ·
📝 [Changelog](docs/pages/changelog.md)

🚧 Status: early development. The plan lives in the
[backlog](docs/pages/backlog.md).

## ⬇️ Download

The installers for the latest version are on the
[releases page](https://github.com/informatom/qed/releases/latest). They
aren't signed yet, so your system asks once before the first start.

- 🍎 **macOS**: download the `.dmg`, open it and drag qed into
  Applications (Apple Silicon and Intel). The first time, open qed;
  macOS says it can't check it, so choose **Done**. Then open System
  Settings → **Privacy & Security**, scroll down to qed and choose
  **Open Anyway**, and confirm with your password. (On macOS 14 and
  older, right-clicking qed in Applications and choosing **Open** does
  the same.)
- 🪟 **Windows**: download the `.exe` and run it. When Windows shows
  “Windows protected your PC”, choose **More info**, then **Run anyway**.
- 🐧 **Linux**: on Debian and Ubuntu, download the `.deb` and install it
  with `sudo apt install ./qed_*.deb`. On any other distribution, download
  the `.AppImage`, make it executable (`chmod +x qed_*.AppImage`) and run
  it.

q.e.d. looks for a newer version each time it starts and asks before
installing one. 🔄

## ✨ What it does

- ✏️ **Pages and posts** made from blocks: text, pictures, columns, FAQs,
  calls to action and more.
- 🎨 **Themes and fonts** to choose from, the fonts self-hosted.
- 🎙️ **Podcasts**: episodes with chapters, people and transcripts, feeds
  for every audio format, the Podlove Web Player and Subscribe Button.
- 🕰️ **Versions** of everything you save, and going back to any of them.
- 🚀 **Publishing** to your own webspace over SFTP, with nothing loaded
  from anywhere else.
- 📖 **A manual** in the app, also on the
  [website](https://qed.informatom.com/manual/).

## 🧠 Built in the open

q.e.d. is built in the open with [Claude Code](https://claude.com/claude-code).
The [docs](docs) folder is a q.e.d. site itself: the project home page, the
manual and a blog with the learning notes written along the way
([docs/pages/blog](docs/pages/blog)).

## ⚙️ How it works

A site is a folder: one `site.toml`, pages and posts as Markdown files, a
`media/` folder for images, `assets/` for downloads, `static/` for files
copied into the site as they are, and `fonts/` for self-hosted fonts. The
app edits these files, keeps their history in git, and builds the static
site from them.

## 🛠️ Development

Requirements: Rust (stable), Node.js 24+, and on Linux the
[Tauri system libraries](https://v2.tauri.app/start/prerequisites/).

```sh
npm install
npm run tauri dev     # run the app
npm run check:all     # fmt, clippy, Rust tests, svelte-check
```

More in [docs/pages/development.md](docs/pages/development.md), including
a throwaway SFTP server for testing publishing.

Code layout:

- `crates/qed-core`: domain logic (sites, pages, site generator), no UI
- `src-tauri`: the desktop shell that exposes qed-core to the UI
- `src`: the Svelte UI

## 📜 License

[GPL-3.0-or-later](LICENSE)

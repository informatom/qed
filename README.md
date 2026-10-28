# q.e.d.

A desktop app for your own website or podcast site, made entirely in the
app, without programming: write pages, add pictures and episodes, choose
colors and fonts, and q.e.d. builds a fast static website and publishes it
to your own webspace. Built for people who want their own site or podcast
without learning a static site generator.

Status: early development. The plan lives in the
[backlog](docs/pages/backlog.md).

q.e.d. is built in the open with [Claude Code](https://claude.com/claude-code).
The [docs](docs) folder is a q.e.d. site itself: the project home page and a
blog with the learning notes written along the way ([docs/pages/blog](docs/pages/blog)).

## How it works

A site is a folder: one `site.toml`, pages and posts as Markdown files, a
`media/` folder for images, `assets/` for downloads and `static/` for files
copied into the site as they are, and `fonts/` for self-hosted fonts. The app edits these files, keeps their history in git, and
builds the static site from them.

## Development

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

## License

[GPL-3.0-or-later](LICENSE)

# qed

A desktop app that turns a folder of pages and posts into a static website and
publishes it to your own webspace, via SFTP, GitHub Pages or a ZIP download.
Built for people who want their own site (and later, their own podcast)
without learning a static site generator.

Status: early development. The plan lives in the
[backlog](docs/pages/backlog.md).

qed is built in the open with [Claude Code](https://claude.com/claude-code).
The [docs](docs) folder is a qed site itself: the project home page and a
blog with the learning notes written along the way ([docs/pages/blog](docs/pages/blog)).

## How it works

A site is a folder: one `site.toml`, pages and posts as Markdown files, and a
`media/` folder. The app edits these files, keeps their history in git, and
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

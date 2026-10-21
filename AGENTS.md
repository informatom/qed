# qed: guidelines for coding agents

qed is a single-user desktop app: Rust + Tauri 2, with a SvelteKit (Svelte 5,
TypeScript) single-page UI styled with Tailwind v4 + daisyUI 5. Open work and
decisions live in `docs/pages/backlog.md` (items `#nnn`); read it first and keep it
current.

## Layout

- `crates/qed-core`: all domain logic (site folder format, pages and blocks,
  site generator, publishers). No Tauri or UI dependencies; test it with
  `cargo test -p qed-core`.
- `src-tauri`: thin shell. Tauri commands only convert arguments and call
  qed-core.
- `src`: the UI. It talks to Rust only through `invoke` from
  `@tauri-apps/api/core`.
- `crates/qed-core/templates`: askama templates for generated sites. After
  changing CSS classes there, run `npm run build:site-css` and commit
  `crates/qed-core/assets/site.css`.
- Build a site without the app: `cargo run -p qed-core --example build -- docs`.
  Output goes to `<site>/_site/` (git-ignored).

## Data

There is no database. A site is a folder: `site.toml`, Markdown pages and
posts with TOML front matter (`+++`), block containers (`::: type` … `:::`)
inside the Markdown, and `media/`. History is git. Secrets (SSH keys, tokens)
live in the OS keychain, never in the site folder.

## Rules

- Run `npm run check:all` before finishing: rustfmt, clippy with
  `-D warnings`, Rust tests and svelte-check must pass.
- Return `Result` with a typed error from qed-core; no `unwrap`/`expect`
  outside tests and `main`/`run`.
- Svelte 5 runes only (`$state`, `$derived`, `$effect`, `$props`); no legacy
  `export let` or stores for component state.
- Give key UI elements stable `id`s.
- Don't write inline `<script>` tags into generated sites without need; the
  generated sites must work without JavaScript except for the player.

## Learning blog

The owner is learning Rust, Svelte and TypeScript through this project. When a
change introduces a concept new to the project, explain it in a post in
`docs/pages/blog/` (`YYYY-MM-DD-slug.md`, TOML front matter), tied to the concrete
code, comparing with Elixir where it helps, with links to the official docs.
Give each post `title`, `date`, `authors`, `tags` and `summary`, and add it
to `docs/pages/blog/index.md`. `docs/` is a qed site; `cargo test` fails if
it stops being valid. Don't explain in chat what belongs in a post.

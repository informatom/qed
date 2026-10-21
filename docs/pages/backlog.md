+++
title = "Backlog"
description = "Open work and decisions for qed, numbered #nnn."
menu = "footer"
weight = 10
+++

Open work and open decisions. Items keep their number for good; when one is
done or decided, move it to **Done / decided** with a short note. Items that no
longer apply move to **Dropped**.

qed is a single-user desktop app (Rust + Tauri) that turns a podcaster's
content into a static site and publishes it. Decided 2026-10-21; it replaces
the earlier Phoenix web app.

## To decide

Nothing open.

## To build

### Phase 1: CMS and site generator

A complete site builder without podcast features.

- **#072 Block file format.** Read and write pages as Markdown with block
  containers (`::: hero` … `:::`, block fields as TOML) in qed-core. Writing
  a parsed page gives the same file back, so hand edits survive the block
  editor. *Done 2026-10-21*: parsing with file and line in every error;
  writing with `qed_core::page_writer` (unchanged files and blocks stay
  byte for byte, changed TOML is merged with `toml_edit` so comments
  survive, defaults are left out, atomic saves). The app edits page
  settings in a side panel; editing blocks comes with #043.
- **#043 Block CMS.** Mercator's block CMS design rebuilt in Rust
  (`docs/notes/cms-handover-qed.md`; its Elixir file map no longer
  applies). *Done 2026-10-21*: nine block types (Text, Hero, Image, Text
  with image, Gallery, Download, FAQ, Columns with nested blocks (#074),
  Call to action); the block editor with Content/Versions/Settings tabs
  per page, OverType for Markdown, the image picker (#033), add, remove,
  move up/down, save with `site::save_blocks`, a warning before losing
  unsaved changes. Still open: a Shared block (one block used on several
  pages) and drag and drop.
- **#056 Start page.** Every site gets a `home` page on creation (Hero and
  Text). *Done 2026-10-21* with "New site" (#046): `pages/index.md` with a
  Hero (the site title) and a short Text. Phase 2 seeds podcast sites with
  Latest episodes and Subscribe.
- **#033 Images and media library.** Import, tag, search and resize images to
  WebP; keep PDFs for downloads. Phase 2 adds Apple's cover art rules
  (1400–3000 px square). *Media library done 2026-10-21*
  (`qed_core::media`): list with size, dimensions and usage, import with
  clean names, delete only unused files, image picker, Image block with
  width/height and lazy loading. Still open: resizing to several sizes
  with a build cache (#075, done), tags.
- **#040 Blog.** Pages with kind `post` under a `blog` parent, a Post list
  block and `blog/feed.xml`.
- **#026 Site generator.** Render HTML for published pages, posts, menus,
  redirects (meta-refresh stubs) and the sitemap with Rust templates. Copy
  the images used. Rewrite relative links to other `.md` pages into page
  URLs; links that leave the site folder (e.g. to source code) need a base
  URL such as the GitHub repo. *First version done 2026-10-21*
  (`qed_core::generator`): pages, menus, relative links, Markdown links,
  redirects, sitemap and canonical links with `base_url`, media copied,
  preview server and "Build" / "Preview in browser" in the app. Still open:
  `og:image`, copying only the media files that are used, and links that
  leave the site folder.
- **#054 Theme options.** Per site: daisyUI theme, logo, a few options.
  *Themes done 2026-10-21*: Site settings dialog edits `site.toml` (title,
  description, language, web address, theme); the app has its own theme
  (App settings, per computer, can follow the system). Logo and other
  options are still open.
- **#032 Publish history.** Record each publish run per target as a git tag
  (#073) plus a log; show progress live. *Tags done 2026-10-21* (with
  #073); the log and a list of publishes per target are still open.
- **#037 Backups.** Document that backing up a site means copying its folder.

### Phase 2: Podcasting

- **#066 Podcast switch.** Per site; turns on episodes, feed, contributors and
  the podcast fields: type, owner name + email, cover, categories, explicit,
  `podcast:guid`, funding.
- **#067 Episodes.** Season, number, type, image, explicit, guid generated
  once, Markdown shownotes, chapters. Episode pages and the archive at
  `/episodes`.
- **#022 Episode editor.** Markdown shownotes (#060) and a chapter editor.
- **#023 Audio file.** One per episode: pick a local file or enter an external
  URL. Read size, MIME type and duration in Rust (`symphonia` or `lofty`).
- **#068 Podcast feed.** One feed per podcast with iTunes, Podcasting 2.0 and
  Podlove Simple Chapters tags, plus paged feeds. Publishers copy the audio
  (GitHub Pages: see #029).
- **#069 Podcast blocks.** Latest episodes and Subscribe blocks; seed them on
  the `home` page of new podcast sites.
- **#047 Contributors.** Contributors per site (name, photo, Markdown bio,
  links), linked to episodes with a role. `podcast:person` in the feed,
  people on episode pages, a Contributors block.
- **#048 Transcripts.** WebVTT per episode. Deployed as a file, linked with
  `podcast:transcript`, shown on the episode page and in the player.
- **#049 Web player.** Self-hosted Podlove Web Player, copied into each site,
  with chapters and transcripts.
- **#053 Share links.** Plain share links on episode pages (Mastodon, Bluesky,
  email, copy link), no third-party scripts.
- **#070 Code signing.** Before a public 1.0: Apple developer account and
  notarization, Windows signing (e.g. Azure Trusted Signing).
- **#034 Feed import.** Import an existing podcast from its RSS feed or from a
  jekyll-octopod site (compare `octopod import`).

### Phase 3: Nice to have

- **#029 Publisher: GitHub Pages.** Push the built site to the configured
  branch with a deploy key. Needs a second `Target` kind, its own key
  handling and connection test (#031). For podcasts, GitHub Pages warns
  when an episode lacks an external audio URL (#068).
- **#011 GitHub Pages.** Decided: deploy key using qed's generated key
  pair. Audio must be external for GitHub targets.

## Done / decided

- **Desktop app.** qed is a single-user desktop app for podcasters who can't
  run a static site generator themselves. Stack: Rust + Tauri 2, Svelte UI,
  files instead of a database. Open source, built with Claude as a showcase. Claude writes the
  code and explains Rust concepts (ownership, borrowing, lifetimes) as they
  come up.
- **License.** GPL-3.0.
- **#057 Data location.** One folder per site, picked by the user. No
  database: one `site.toml` plus pages and posts as Markdown files with TOML
  front matter (#046); blocks are containers inside the Markdown (#072);
  drafts and history via git (#073). The app remembers recently opened site
  folders. Publish secrets stay in the OS keychain (#063).
- **#021 Site UI.** List, create and edit sites. *Done 2026-10-21*: the
  site title in the header is a menu with up to ten recent sites
  (`src/lib/recentSites.ts`, in local storage like the app theme; the old
  `qed:last-site` moves into it), "Open site folder…" and "New site…";
  the start screen lists the recent sites too; × takes a site off the
  list. Switching asks first if the open page has unsaved changes.
  Creating is #046, editing is the Site tab.
- **#055 Sites.** Several sites per installation: done with #021 and
  #046. Phase 1 sites are pages-only; phase 2 adds the podcast switch
  (#066).
- **#046 Site folder, phase 1.** `site.toml` (title, language, theme, menus),
  `targets.toml` (publish targets without secrets, git-ignored), pages and posts as Markdown files with
  TOML front matter, `media/` for images, `assets/` for downloads, build output in a
  git-ignored folder. Old paths for redirects go into front matter.
  *Reading done 2026-10-21* (`qed_core::site::open`, `examples/demo-site`).
  *Creating pages done 2026-10-21* (`site::create_page`, "New page" dialog;
  posts get a date prefix; never overwrites). *Deleting pages done
  2026-10-21* (`site::delete_page`): the confirmation names the pages that
  link to it; the start page and pages with pages below them can't be
  deleted; the Versions tab lists deleted pages and restores them
  (`history::deleted_pages`, `restore_page`). *New site done 2026-10-21*
  (`site::create_site`, "New site…" dialog on the empty start screen and
  in the Site tab): title, language, location (Documents by default) and
  folder name (from the title); the folder must be new or empty (hidden
  files are fine); writes `site.toml`, the start page (#056), empty
  `media/` and `assets/`, and starts the version history. *Moving and
  renaming done 2026-10-21* (`qed_core::moving`, "Move or rename" in the
  page editor): pages below move along, each moved page gets its old
  address in `redirect_from` (and a redirect that a new address takes
  over is dropped), links to moved pages and relative links in them are
  updated in Markdown and buttons (`markdown::replace_link_targets` keeps
  the rest of the text as it was), and the open version is closed first.
  Links that were broken already, and relative links to built pages
  (`../about/`), stay as they are.
- **#079 A note before publishing.** Done 2026-10-21: the Publish button
  opens a dialog that names the destination and, when there's an open
  version or a change since the last version (`HistoryStatus::needs_note`),
  asks what's new on the site; publishing can't start without it
  (`history::close_for_publish`, `HistoryError::NoteNeeded`). The note
  closes the version; the publish tag stays as it was. Without anything
  new, the last version is published as it is. Replaces the automatic
  note "Before publishing to X".
- **#062 Release builds.** Built 2026-10-21:
  `.github/workflows/release.yml` on github.com/informatom/qed builds, for
  a `v*` tag, a universal macOS `.dmg`, a Windows `.exe` (NSIS), and a
  Linux `.deb` and `.AppImage` (on Ubuntu 22.04 for older systems), and
  attaches them to a draft release. The tag must match the version in
  `Cargo.toml` and `package.json`; the app takes its version from
  `Cargo.toml`. The app identifier is now `io.github.informatom.qed`.
  Unsigned until #070. Steps in Development → Releasing. First run
  2026-10-21: tag `v0.1.0` built all four installers in about 23 minutes
  (draft release, AppImage 84 MB, deb 6 MB, dmg 9 MB, exe 4 MB).
- **#074 Nested blocks.** Done 2026-10-21: a column holds blocks (Text,
  Image, Text with image, Download, FAQ, Call to action) plus an optional
  title. In the file, `:::: columns` (four colons) wraps the columns and
  `::: column` lines start each one and hold its title; the writer keeps
  what's written inside columns like anywhere else. Blocks render one at a
  time (`block.html`), so a column renders its blocks the same way; the
  editor's block list (`BlockList.svelte`) uses itself for each column.
  Only Columns nest; a full-width Section block could use the same format
  later.
- **#073 Git history.** Done 2026-10-21 (`qed_core::history`, git2 with
  bundled libgit2): a top-level Versions tab starts the history (git init
  plus `.gitignore` for `_site/` and `targets.toml`), shows changed files,
  saves versions with a note and lists them. Publishing saves a version
  first and tags it (`publish/<target>/<UTC time>`). *Open versions
  2026-10-21*: every save in the app goes into an open version (a commit
  marked `qed-draft: true`, amended by later saves); "Close version" gives
  it its note, publishing closes it too. qed amends only its own open
  version, never someone else's commit and never one that was pushed.
  Deleting a page closes the open version first, so the page can come
  back. Possible later: closing open versions by themselves after a while. Each page has a
  Versions tab (Content / Versions / Settings) with what each version
  changed and "Restore", which puts it into the editor as unsaved changes.
  A site folder inside a larger repository (like `docs/`) works too: qed
  commits only that folder, never other staged changes, and prefixes its
  tags (`docs/publish/…`). Still open: restoring the whole site, signed
  commits (libgit2 doesn't sign).
- **#078 Assets for downloads.** Done 2026-10-21: `assets/` holds files
  to download (PDFs, but any type); `media/` takes only images. An Assets
  tab lists, imports (clean names) and deletes them like Media
  (`qed_core::media` with a `Library` parameter); the Download block picks
  from assets and links to `/assets/…`. Sites with downloads still in
  `media/` keep working (the generator falls back to `media/`), and the
  Media tab offers "Move to Assets", which moves them and updates the
  pages' Download blocks and `/media/…` links.
- **#075 Responsive images.** Done 2026-10-21 (`qed_core::images`): JPEG
  and PNG images in Image, Text with image, Gallery and Hero blocks get
  copies 480, 960 and 1600 px wide (only those under 90% of the original),
  in the original's format, next to it in the built site
  (`media/team.480w.jpg`), listed in `srcset` with a `sizes` hint per
  block. Camera rotation (EXIF) is applied. Copies are cached in the site's
  self-ignoring `.qed-cache/` and made on all cores. The Hero is now an
  `<img>` behind the text instead of a CSS background. Still open: images
  inside Markdown text keep the original only; WebP copies.
- **#076 Generated TypeScript types.** Done 2026-10-21: tauri-specta
  writes `src/lib/bindings.ts` (types and one function per command) from
  `commands()` in `src-tauri`; the `bindings_are_current` test rewrites it
  when it falls behind. qed-core derives `specta::Type` behind its `specta`
  feature. specta marks `#[serde(default)]` fields optional in both
  directions, so `site.ts` re-exports the commands and types through
  `Sent<T>`, which makes them required in what Rust sends.
- **#077 Sidebar navigation.** Done 2026-10-21: `navigation = "sidebar"` in
  `site.toml` (switch in the Site tab) moves the main menu from the top bar
  into a sidebar, for sites with many menu pages. The current page is
  highlighted, its parent in bold; on phones it's a "Menu" box that opens
  on tap, without JavaScript. Menus follow the page tree: an entry goes
  under the nearest menu entry above it (posts under Blog) and is indented
  in the sidebar.
- **Page icons.** Done 2026-10-21: `icon = "book-open"` on a page shows a
  Lucide icon in the menus of the generated site (plain SVG, no
  JavaScript) and in the app's page list. A curated set of 60 icons lives
  in `crates/qed-core/assets/icons/` and is built into qed; unknown names
  are an error with the line number. The page settings have a picker.
- **#064 Site CSS.** Done 2026-10-21: one Tailwind + daisyUI stylesheet with
  all 35 themes and the typography plugin (77 KB, 14 KB gzipped), compiled
  by `npm run build:site-css` and built into qed with `include_str!`. The
  theme is the `data-theme` attribute.
- **SFTP publishing.** Done 2026-10-21 (#025, #028, #030, #031 for SFTP,
  #063, #065): targets in `targets.toml` (`[[targets]]`, nothing secret,
  git-ignored because it names servers),
  passwords and generated Ed25519 keys in the OS keychain, `russh` with the
  `ring` backend, trust on first connection and a hard stop when the
  server key changes, `.qed-manifest.json` so only changed files are
  uploaded and only qed's own files deleted, emptied folders removed, live
  progress over a Tauri channel. A throwaway server for testing lives in
  `dev/test-server/` (see the Development page).
- **#061 Scaffold.** Done 2026-10-21: Cargo workspace (`src-tauri`,
  `crates/qed-core`), SvelteKit SPA with Tailwind + daisyUI, GPL-3.0
  `LICENSE`, `npm run check:all`. Elixir code and its local data removed.
- **#058 Frontend.** Svelte + TypeScript.
- **#059 Signing and updates.** Phase 1 ships unsigned installers (Linux
  AppImage/deb, Windows, macOS with "open anyway" instructions) plus Tauri's
  updater, whose signing key is free. Revisit Apple notarization ($99/year)
  and Windows signing before a public 1.0 (#070). *Updater built
  2026-10-21* (tauri-plugin-updater and -process, `src/lib/updates.svelte.ts`):
  qed checks `latest.json` of the latest published GitHub release at start,
  a bar offers "Install and restart", the App tab shows the version and
  "Check for updates". The release workflow signs the updates with the
  `TAURI_SIGNING_PRIVATE_KEY` secret (`tauri.release.conf.json`); steps in
  Development → Updates. The first version with the updater has to be
  installed by hand; from then on updates come by themselves.
- **#060 Markdown editor.** Keep OverType.
- **Learning blog.** Explanations of Rust, Svelte and TypeScript go into blog
  posts in `docs/pages/blog/` (Markdown with front matter), not into chat.
- **#071 Demo site.** Done 2026-10-21: `docs/` is a qed site (home page,
  blog with the learning posts). A test keeps it valid. Pages gained
  `date`, `authors`, `tags` and `summary`.
- **Repo.** Same repo; the Elixir code is deleted (it stays in git history).
- **Site pages** follow the design of Mercator's block CMS; Mercator stays a
  read-only reference.
- **Two phases.** Phase 1 builds the CMS and site generator; phase 2 adds
  podcasting.
- **Podcast is optional.** A site can have only static pages (#055, #066). `/` is
  always the CMS `home` page (#056).
- **#001 Storage.** Audio and images stay on the user's computer.
- **#004 Site templates.** Tailwind + daisyUI. Users pick a theme and options;
  templates aren't user-editable (#054, #064).
- **#005 Web player.** Self-hosted Podlove Web Player (#049).
- **#006 Podcast fields.** Apple basics, `podcast:guid`, funding (#046).
- **#007 Episode fields.** Season, number, type, image, explicit, stable guid
  (#046).
- **#008 Transcripts.** WebVTT (#048).
- **#009 Audio formats.** One format per podcast, provided by the user; one
  feed. No transcoding.
- **#010 Duration and size.** Read from the local file (#023).
- **#012 SSH transfer.** SFTP with a deploy manifest (#028, #065).
- **#016 License.** Changed from AGPL-3.0 to GPL-3.0 with the move to desktop.
- **#019 Comments and sharing.** Share links, no comments (#053).
- **#040 Blog.** Pages with kind `post` under a `blog` parent, with a Post
  list block and `blog/feed.xml`.
- **#041 Contributors.** Their own records, linked to episodes (#047).
- **#042 Redirects.** Keep page redirects; write meta-refresh stubs at old
  paths.

## Dropped

Not needed in a single-user desktop app:

- #002 Registration, #003 Email, #014 Roles, #015 Global slugs, #017 Demo
  instance, #018 Offline drafting (the app is offline), #020 Home page,
  #024 Members UI, #035 Last-owner protection, #050 Invitations, #051 Email.
- #013 Background jobs: a deploy is a background task in the app (#032).
- #036 Docker image tools, #038 CLOAK_KEY rotation (#063 replaces it),
  #039 AGENTS.md (Phoenix-specific), #052 AGPL license file (#061).
- #044 Local audio folder and #045 Audio pass-through: the app reads files
  directly.
- #027 Publisher: ZIP. The built site already is a folder (`_site/`); the app
  has "Show in file manager" instead.

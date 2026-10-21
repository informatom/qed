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

- **#062 Release builds.** GitHub Actions builds installers for macOS,
  Windows and Linux on every tag.
- **#046 Site folder, phase 1.** `site.toml` (title, language, theme, menus),
  `targets.toml` (publish targets without secrets, git-ignored), pages and posts as Markdown files with
  TOML front matter, `media/` for images, `assets/` for downloads, build output in a
  git-ignored folder. Old paths for redirects go into front matter.
  *Reading done 2026-10-21* (`qed_core::site::open`, `examples/demo-site`).
  *Creating pages done 2026-10-21* (`site::create_page`, "New page" dialog;
  posts get a date prefix; never overwrites). Creating a new site folder
  from the app, and renaming, moving and deleting pages, are still open.
- **#072 Block file format.** Read and write pages as Markdown with block
  containers (`::: hero` … `:::`, block fields as TOML) in qed-core. Writing
  a parsed page gives the same file back, so hand edits survive the block
  editor. *Done 2026-10-21*: parsing with file and line in every error;
  writing with `qed_core::page_writer` (unchanged files and blocks stay
  byte for byte, changed TOML is merged with `toml_edit` so comments
  survive, defaults are left out, atomic saves). The app edits page
  settings in a side panel; editing blocks comes with #043.
- **#074 Nested blocks.** Columns contain blocks, so containers must nest
  (e.g. `::::` for the outer container, as markdown-it-container does).
- **#073 Git history.** Each site folder is a git repo the app manages
  (`git2` or `gix`): "save version" commits, publishing tags the commit,
  and any page can be restored from an earlier version.
- **#055 Sites.** Several sites per installation. Phase 1 sites are pages-only;
  phase 2 adds the podcast switch.
- **#021 Site UI.** List, create and edit sites.
- **#043 Block CMS.** *Text with image, Gallery, Download and FAQ done
  2026-10-21.* *Block editor done 2026-10-21*: Content/Settings tabs
  per page, OverType for Markdown, Hero and Call-to-action fields, add,
  remove, move up/down, save with `site::save_blocks`, warning before
  losing unsaved changes. Still open: the remaining block types, an image
  picker (#033), drag and drop.
  Original plan: Rebuild Mercator's block CMS design in Rust
  (`docs/notes/cms-handover-qed.md` describes it; its Elixir file map no longer
  applies): pages as ordered lists of typed blocks stored as Markdown
  files (#072), drafts via `draft = true`, history via git (#073), page tree
  from the folder structure, menus, blocks (Text, Hero, Text with
  image, Image, Gallery, Download, Columns, FAQ, Call to action, Shared).
- **#056 Start page.** Every site gets a `home` page on creation (Hero and
  Text). Phase 2 seeds podcast sites with Latest episodes and Subscribe.
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
- **#029 Publisher: GitHub Pages.** Push the built site to the configured
  branch with a deploy key. Needs a second `Target` kind, its own key
  handling and connection test (#031).
- **#032 Publish history.** Record each publish run per target as a git tag
  (#073) plus a log; show progress live.
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
  Podlove Simple Chapters tags, plus paged feeds. Publishers copy the audio;
  GitHub Pages warns when an episode lacks an external audio URL.
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
  and Windows signing before a public 1.0 (#070).
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
- **#011 GitHub Pages.** Deploy key using qed's generated key pair. Audio must
  be external for GitHub targets.
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

+++
title = "Done"
description = "Finished and decided backlog items of q.e.d., and the ones dropped."
menu = "footer"
weight = 11
+++

Backlog items that are done, decided or dropped. Open work is in the
[backlog](backlog.md). Items that are partly done stay there until the
rest is done.

## Done / decided

- **#072 Block file format.** Read and write pages as Markdown with block
  containers (`::: hero` … `:::`, block fields as TOML) in qed-core. Writing
  a parsed page gives the same file back, so hand edits survive the block
  editor. *Done 2026-10-21*: parsing with file and line in every error;
  writing with `qed_core::page_writer` (unchanged files and blocks stay
  byte for byte, changed TOML is merged with `toml_edit` so comments
  survive, defaults are left out, atomic saves). The app edits page
  settings in a side panel; editing blocks comes with #043.
- **#056 Start page.** Every site gets a `home` page on creation (Hero and
  Text). *Done 2026-10-21* with "New site" (#046): `pages/index.md` with a
  Hero (the site title) and a short Text. Phase 2 seeds podcast sites with
  Latest episodes and Subscribe.
- **Desktop app.** q.e.d. is a single-user desktop app for podcasters who can't
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
  "Close site" (2026-10-22) goes back to the start screen, and the next
  start stays there until a site is opened (`qed:site-closed`).
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
  note "Before publishing to X". The dialog also lists the files changed since the last
  publish to that target (`history::since_publish`, from the publish
  tags), since a version saved in between is still new there; fixed
  2026-10-21 after "Nothing new" hid three unpublished posts.
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
  later (#087).
- **#073 Git history.** Done 2026-10-21 (`qed_core::history`, git2 with
  bundled libgit2): a top-level Versions tab starts the history (git init
  plus `.gitignore` for `_site/` and `targets.toml`), shows changed files,
  saves versions with a note and lists them. Publishing saves a version
  first and tags it (`publish/<target>/<UTC time>`). *Open versions
  2026-10-21*: every save in the app goes into an open version (a commit
  marked `qed-draft: true`, amended by later saves); "Close version" gives
  it its note, publishing closes it too. q.e.d. amends only its own open
  version, never someone else's commit and never one that was pushed.
  Deleting a page closes the open version first, so the page can come
  back. Possible later: closing open versions by themselves after a while (#083). Each page has a
  Versions tab (Content / Versions / Settings) with what each version
  changed and "Restore", which puts it into the editor as unsaved changes.
  A site folder inside a larger repository (like `docs/`) works too: q.e.d.
  commits only that folder, never other staged changes, and prefixes its
  tags (`docs/publish/…`). Still open: restoring the whole site (#082), signed
  commits (#084).
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
  inside Markdown text keep the original only; WebP copies (#085).
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
- **Unknown block types.** Done 2026-10-26: a block type this q.e.d.
  doesn't know is reported with a hint to update (App → Check for
  updates), since it usually comes from a newer q.e.d.
- **Page icons.** Done 2026-10-21: `icon = "book-open"` on a page shows a
  Lucide icon in the menus of the generated site (plain SVG, no
  JavaScript) and in the app's page list. A curated set of 60 icons lives
  in `crates/qed-core/assets/icons/` and is built into q.e.d.; unknown names
  are an error with the line number. The page settings have a picker.
- **#064 Site CSS.** Done 2026-10-21: one Tailwind + daisyUI stylesheet with
  all 35 themes and the typography plugin (77 KB, 14 KB gzipped), compiled
  by `npm run build:site-css` and built into q.e.d. with `include_str!`. The
  theme is the `data-theme` attribute.
- **SFTP publishing.** Done 2026-10-21 (#025, #028, #030, #031 for SFTP,
  #063, #065): targets in `targets.toml` (`[[targets]]`, nothing secret,
  git-ignored because it names servers),
  passwords and generated Ed25519 keys in the OS keychain, `russh` with the
  `ring` backend, trust on first connection and a hard stop when the
  server key changes, `.qed-manifest.json` so only changed files are
  uploaded and only q.e.d.'s own files deleted, emptied folders removed, live
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
  q.e.d. checks `latest.json` of the latest published GitHub release at start,
  a bar offers "Install and restart", the App tab shows the version and
  "Check for updates". The release workflow signs the updates with the
  `TAURI_SIGNING_PRIVATE_KEY` secret (`tauri.release.conf.json`); steps in
  Development → Updates. The first version with the updater has to be
  installed by hand; from then on updates come by themselves.
- **#060 Markdown editor.** Keep OverType.
- **Learning blog.** Explanations of Rust, Svelte and TypeScript go into blog
  posts in `docs/pages/blog/` (Markdown with front matter), not into chat.
- **#071 Demo site.** Done 2026-10-21: `docs/` is a q.e.d. site (home page,
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
  list block and `blog/feed.xml`. *Done 2026-10-21*: a post is a page
  with a `date` that isn't in the future (later ones show up once the site
  is built on or after their date; their own pages are built anyway). The
  Post list block (`::: post-list`, `from` and `limit`, also in columns)
  shows the posts below a page newest first, same-day posts by `weight`
  (higher first), with date and summary. Each page with posts right below
  it gets an RSS 2.0 feed (`blog/feed.xml`, summary as description,
  authors as `dc:creator`), linked from every page's `<head>` and, with
  the RSS icon, from the footer and below Post lists; feeds need
  `base_url`, and the build report says so when it's missing. Moving a
  page updates Post lists that list it. Dates without a library:
  `qed_core::calendar`.
- **#041 Contributors.** Their own records, linked to episodes (#047).
- **#042 Redirects.** Keep page redirects; write meta-refresh stubs at old
  paths. *2026-10-27*: an old address ending in `.html`
  (`/2016/03/28/easterhegg.html`, as Jekyll made them) is written as that
  file instead of a folder.
- **#080 Logo and app icon.** *Done 2026-10-22*: `brand/` holds the
  artwork: `logo-mark.svg` (the Q with the houses, drawn by hand, about
  2 KB), `logo.svg` (mark and wordmark, text as paths), `app-icon.svg`
  (the mark on a light tile) and its 1024 px PNG, from which
  `npx tauri icon brand/app-icon.png` makes `src-tauri/icons/` (delete the
  `android/` and `ios/` folders it adds). The JPEGs are illustrations;
  `logo-traced.svg` is the first automatic trace, kept for reference.
  The app shows the mark in its header and on the start screen, from a
  copy in `src/lib/assets/logo-mark.svg` (copy it again after changing
  the mark).
- **#088 Static files.** *Done 2026-10-27*: a `static/` folder in the
  site is copied into the built site as it is, at the same paths
  (`static/pdf/a.pdf` becomes `/pdf/a.pdf`), hidden files included
  (`.well-known/`, `.htaccess`); `.DS_Store`, `Thumbs.db` and
  `desktop.ini` are skipped. It's copied last, and a file the build wrote
  itself (a page, `style.css`, a feed) or the publish manifest stops the
  build with an error instead of being replaced. Link to the files as
  `/pdf/a.pdf`. The build message counts them. The app has a Static tab
  next to Media and Assets (`Library::Static`): import files or whole
  folders under their own names (a taken name is an error, never a
  rename), one folder at a time with a path to click back up (imports go
  into the folder shown; a search looks through all), each with its address and
  the pages linking to it (Markdown links and button links), delete files
  or folders no page links to. Text files (HTML, CSS, `.txt`, …, the new
  `MediaKind::Text`) open read only in a viewer, in all three libraries.
- **#099 Page list block.** *Done 2026-10-27*: `::: page-list` with
  `from` (empty for the page it's on) lists the pages one level below,
  by weight, then title (compared by slug, so `André` comes before
  `Andreas`), each with its summary. Pages with a date are posts and left
  out; drafts too. Its `from` counts as a link, so moving the listed page
  updates it, like Post list's. podcasterei.at's Podcaster:innen uses it
  instead of the hand-written list of 83 people; their podcasts moved
  into each person page's summary.
- **#101 Custom theme.** *Done 2026-10-27*: `theme = "custom"` with a
  `[custom_theme]` table in `site.toml`: `from` (a theme), five colors
  (background, primary, secondary, accent, neutral) and `corners` (sharp,
  soft, round). The page keeps `from` as `data-theme`; a `:root[data-theme]`
  rule appended to `style.css` sets the changed variables, text colors
  picked for WCAG contrast, base-200/300 mixed from the background
  (`qed_core::theme`). Site → Theme has a Custom colors card that starts
  from a theme's colors (read via canvas) and saves on each change; the
  legend previews it live. Picking another theme keeps the custom colors.
  *Saved colors*: named sets in `[[saved_themes]]` (same fields plus
  `name`), applied with one click, saved with "Save these colors",
  deleted after asking; the card follows `site.toml` after Reload, so an
  old copy in the app can't overwrite it.
- **#098 Dates in the site's language.** *Done 2026-10-27*:
  `calendar::long_date` writes dates for readers by the site's `language`:
  `24. Februar 2017` (de), `24. Jänner 2017` (de-AT), `February 24, 2017`
  (en), `24 February 2017` (en-GB), plus fr, es, it, nl, pt; other
  languages keep `2017-02-24`. Used in post lists and bylines; `<time
  datetime>` keeps the ISO date. podcasterei.at is `de-AT` now, like its
  old template's Jänner.
- **#100 Video block.** *Done 2026-10-27*: `::: video` with `url`,
  `title` and `image`. The website shows the thumbnail with a play mark
  as a link to the video: no iframe, no JavaScript, nothing loaded from
  the platform until a click. "Get thumbnail and title" in the editor asks
  YouTube, Vimeo or a PeerTube instance once over oEmbed
  (`qed_core::video`, reqwest with rustls on ring) and keeps the thumbnail
  in `media/` (YouTube: the 16:9 `maxresdefault.jpg` when there is one).
- **#102 Fonts.** *Done 2026-10-27*: `[fonts]` in `site.toml` (`text`,
  `headings`; headings follow text if empty). Fonts come from Fontsource
  (the Google Fonts and more), downloaded once into `fonts/<id>/` with a
  `font.toml` (`qed_core::fonts`): 400, 600 if there, 700 and italic, for
  latin and latin-ext. The build copies the fonts in use and appends
  `@font-face` rules with `unicode-range` to `style.css`; no third-party
  requests. Site → Theme → Fonts has a searchable picker and a live
  sample (FontFace API).
- **#092 Footer blocks.** *Done 2026-10-27*: `footer.md` in the site
  folder (page format) holds blocks shown on every page above the footer
  menu, in the footer's colors; Post and Page lists there show titles
  only. Links are rendered per page; moving a page rewrites footer links,
  and the delete warning and media usage count the footer ("the footer").
  In the app: a Footer entry below the page list opens the block editor
  (`BlockEditor` now takes the saved blocks and a save function).
  podcasterei.at has one: Letzte Posts, Meetup, Kontakt.
- **#103 Lists with pictures.** *Done 2026-10-27*: Post list and Page
  list have `layout`: `list` (default), `cards` (picture, title, date,
  summary; 1–3 columns) and `tiles` (square pictures with title and
  summary; 2–6 columns for pages, 2–4 for posts). The picture is the
  page's share image, else its first image; resized copies with `srcset`
  as everywhere. In the editor: a Layout choice on both blocks.
  waldgruber.at uses cards for its posts and tiles for its 53 Stofftiere.
- **#094 Code highlighting.** *Done 2026-10-27*: fenced code blocks
  with a language (```` ```rust ````) are highlighted when the site is
  built, by comrak's syntect plugin (pure-Rust regex engine, no C):
  classes with an `hl-` prefix (apart from daisyUI's, which also has
  `link`), and the dark "Base16 Ocean Dark" colors appended to
  `style.css`, readable on every theme since code blocks are dark. No
  JavaScript. Grammars: syntect's defaults (Rust, HTML, CSS, JS, shell,
  JSON, YAML, XML, Markdown, LaTeX, Ruby, …); more in #106.
- **#095 Wide logos.** *Done 2026-10-27*: the header logo keeps its
  height and takes the width it needs; `logo_size` (small 36 px, medium
  48 px, large 64 px, original: the image's own size up to 112 px, sharp
  for bitmap logos) and `title_in_header = false` for logos that show
  the name (the logo's alt text then carries the site title); a
  `favicon.ico` in `static/` is the tab icon instead of the logo. In Site
  settings below the logo. informatom.com: medium wordmark, no title.
- **#096 Hide page title.** *Done 2026-10-27*: page setting `hide_title`
  ("Hide the title" in Settings); a starting Hero still hides it by
  itself. informatom.com's start page uses it.
- **#097 Column widths.** *Done 2026-10-27*: `width` per column (1, 2 or
  3, relative to the others; 1 is left out of the file). Unequal widths
  become `--cols: minmax(0,2fr) minmax(0,1fr)` with the class
  `md:grid-cols-(--cols)`; phones still stack. In the editor: 1×/2×/3×
  next to each column's title. informatom.com's start page: two thirds
  and one third, like the old Bulma `is-two-thirds`.
- **#093 404 page.** *Done 2026-10-27*: the build writes `404.html` in
  the site's look: a short text in the site's language (de, en, fr, es,
  it, nl) with a link to the start page, or the site's own `pages/404.md`,
  which then isn't built as a normal page, menu entry or sitemap entry.
  Its links start at the site root (`/`, or the path of `base_url`),
  since servers show it at any address (`LinkContext::rooted`). The
  preview server shows it for missing pages. The web server must be told
  to use it, once per site: nginx `error_page 404 /404.html;` in the
  site's `server` block (the owner's server runs nginx; `.htaccess` files
  don't work there), Apache `ErrorDocument 404 /404.html`; GitHub Pages
  does it by itself.
- **#104 New sites in their language.** *Done 2026-10-27*: a new
  site's start page (title, Hero line, first paragraph) is written in
  the site's language: German (also `de-AT`), English, French, Spanish,
  Italian, Dutch; English for others. The text now says "q.e.d.", not
  "qed".
- **#037 Backups.** *Done 2026-10-27*: a [Backups](backups.md) page:
  a site is a git repository, so a backup is a push to a remote (Codeberg,
  GitHub, a bare repo on one's own server), with full history; what it
  leaves out (targets.toml, keychain secrets, the open version, `_site/`,
  `.qed-cache/`). Decided with the owner instead of "copy the folder". A
  button for it in the app: #109; the page also belongs into the app's
  manual: #110.
- **#082 Restore the whole site.** *Done 2026-10-27*: "Go back" on any
  older version in the Versions tab (`history::restore_site`) saves the
  present as a version ("Before going back to …"), writes that version's
  files into the site folder, deletes files it didn't have (and folders
  left empty), and saves the result as a new version ("Back to …"); the
  history is never rewritten. Only files git tracks in the site folder
  change (`_site/`, `targets.toml`, `.qed-cache/` stay; for a site inside
  a larger repository, nothing outside it). Disabled while a page has
  unsaved changes. Also fixed: versions saved in the same second were
  listed in any order (`Sort::TOPOLOGICAL | Sort::TIME`).
- **#111 Hand-made menu.** *Done 2026-10-27*: `[[main_menu]]` in
  `site.toml` (entries: `page`, `link` to any address, groups with
  `[[main_menu.items]]`, `divider = true` inside groups), edited in
  Site → Menu ("Arrange the menu by hand" starts from the automatic menu;
  "Back to automatic"). In the top bar groups are daisyUI dropdowns:
  hover, click/tap and keyboard, no JavaScript; the box hangs in a
  transparent frame so the pointer can reach it. On phones one "Menu"
  button; in the sidebar groups are headings. Entries follow moved pages;
  the delete warning names "the menu". Without one, the automatic menu
  from the pages' Menu setting stays. informatom.com's old navbar
  rebuilt with it (top bar instead of the sidebar).
- **#112 Page width.** *Done 2026-10-27*: `page_width` in `site.toml`
  (Site settings): normal (56rem, as before; 72rem with a sidebar), wide
  (72rem; 80rem with a sidebar) or full (the window). Header, content and
  footer follow it together. informatom.com uses wide.
- **#113 Boxed columns.** *Done 2026-10-27*: `boxed = true` per column
  (a "Box" switch beside its width in the editor): the column gets the
  theme's background, a thin border, a soft shadow, rounded corners and
  inner spacing, like Bulma's `.boxed` on the old informatom.com.
- **#114 Icons in text.** *Done 2026-10-27*: `:icon-phone:` in any
  Markdown text becomes that icon from q.e.d.'s Lucide set (the same as
  for menu entries), as tall as the letters (`.inline-icon`); not inside
  code, and unknown names stay as written. informatom.com's footer uses
  phone, mail, message-circle (Lucide has no Mastodon logo) and map. Fixed
  along the way: eleven icons (mail, calendar, lock, …) had lost the size
  of their rectangles when they were copied in, so the rectangles weren't
  drawn; a test now checks every shape.
- **#115 Site icons.** *Done 2026-10-27*: image files in a site's
  `media/icons/` work as icons in text too: `:icon-rails:` shows
  `media/icons/rails.svg` (else `.webp`, `.png`, `.jpg`) in its own
  colors, 1.5 times as tall as the letters, before q.e.d.'s Lucide icon
  of that name. The media library counts them as used, so they can't be
  deleted while a page writes them. informatom.com has its 14 old badges
  back: the six stacks in the start page's list and eight open-source
  logos under IT-Beratung.
- **#116 Marking external links.** *Done 2026-10-27*: "Mark links to
  other sites" in Site settings (`mark_external_links`, on for new sites,
  off for existing ones): links in text to other sites get Lucide's
  external-link icon after them, in the link's color, by CSS alone (every
  internal link is relative). Not in menus or on buttons, not on linked
  images. Full links to the site's own `base_url` (with or without
  `www.`) now become relative links when building, so they stay inside
  the site, in the preview too. informatom.com has it on.
- **#117 Images open big.** *Done 2026-10-27*, asked for by Sonja for
  waldgruber.at's plushies (the old site had a Bootstrap modal): a click
  on an image shows it big over the page on a dark backdrop, closed with
  Esc, a click on it or beside it. The browser's popover
  (`popovertarget`), no JavaScript; browsers without it just show the
  image. Automatic for images noticeably larger than their place: Image
  blocks over 1300 px, Text with image over 700 px, every Gallery image
  (it's cut to a square; before, it linked to the original file). The big
  one has every copy and the original in `srcset` with `sizes="95vw"`, so
  a phone loads a copy and a 4K screen the original (Sonja's second
  wish). The close button says "Schließen" on German sites, "Close"
  elsewhere.
- **#118 Images open big in the app.** *Done 2026-10-27*: in the Media
  tab, a click on an image shows it as large as the window allows
  (`ImageViewer.svelte`, a modal `<dialog>`), with its name, size and
  dimensions; ← and → step through the images shown (search and folder
  apply), Esc or a click on it or beside it closes it. Not in the image
  picker, where a click still picks.
- **#119 PDFs in the app.** *Done 2026-10-27*: a click on a PDF in the
  Assets (or Media, Static) tab shows it in a large window inside the app
  (`PdfViewer.svelte`, an `<iframe>`): the webview's own viewer, PDF.js in
  WebKitGTK (2.52 here), Edge's in WebView2, PDFKit on macOS. "Open in PDF
  app" opens it in the computer's PDF reader, through the new
  `open_library_file` command, which only opens files inside the site's
  folders (`media::path_of`).

- **#043 Block CMS.** Mercator's block CMS design rebuilt in Rust
  (`docs/notes/cms-handover-qed.md`; its Elixir file map no longer
  applies). *Done 2026-10-21*: nine block types (Text, Hero, Image, Text
  with image, Gallery, Download, FAQ, Columns with nested blocks (#074),
  Call to action); the block editor with Content/Versions/Settings tabs
  per page, OverType for Markdown, the image picker (#033), add, remove,
  move up/down, save with `site::save_blocks`, a warning before losing
  unsaved changes. *Shared blocks done 2026-10-27* (`qed_core::shared`):
  files in `shared/` (title + blocks, no page of their own), shown on a
  page by a Shared block (`::: shared` / `name = "…"`, also in columns and
  the footer); the build swaps in the blocks with the page's links and
  notes missing ones; "Make shared" on any block, a Shared blocks list
  under the pages, rename, delete only when unused; links in them follow
  moved pages, their images count as used. *Drag and drop done
  2026-10-27*: a grip handle on every block (inside columns too) drags it
  to its new place within its list, with a line where it will land and
  scrolling near the window's edges; pointer events, not HTML drag and
  drop, which the Tauri window takes over for dropped files.

- **#120 Drag and drop in the page tree.** *Done 2026-10-27*: a grip
  handle on every page in the Pages tab (not the start page). Dropped
  between pages, it changes the order: the pages below the same parent get
  the weights 10, 20, 30, … (`site::set_order`, `set_page_order`); the tree
  now shows the site's order (weight, then title; posts by date). Dropped
  onto a page, it moves below it after a confirmation, through
  `move_page` (links and redirects as in the Move dialog). Posts can't be
  reordered. The logic is in `src/lib/pageTree.ts` (`treeOrder`,
  `planDrop`).

- **#054 Theme options.** Per site: daisyUI theme, logo, a few options.
  *Themes done 2026-10-21*: Site settings dialog edits `site.toml` (title,
  description, language, web address, theme); the app has its own theme
  (App settings, per computer, can follow the system). *Logo done
  2026-10-22*: `logo` in `site.toml` (Site settings, image picker) shows
  before the title in every page's header and is the browser-tab icon
  (`<link rel="icon">`); the media library marks it "Site logo" and won't
  delete it. *Theme colors done 2026-10-27*: sites use all four theme
  colors (primary: buttons, Call to action, current menu page, link
  underlines; secondary: Hero without image, post dates; accent: stripe at
  the top, quote bars; neutral: footer, sidebar's current page, file
  types), and Site settings explains them in a legend beside the theme
  switcher; a picked theme saves at once. *Header bar done 2026-10-27*:
  `header` in `site.toml` (plain, primary, secondary, accent, neutral); a colored bar
  underlines the current page and gives the logo a light backing. Fonts:
  #102 (done). *Colors per element done 2026-10-27*: `[colors]` in
  `site.toml` (links, buttons, Hero, Call to action, boxed columns,
  content background as a light tint, footer), each a theme color (or
  plain/shaded for areas); empty means the old default; a button on a box
  of its own color is drawn plain; the legend and sketch in Site → Theme
  follow. *Closed 2026-10-27*: no other options planned; the Colors section
  has a Reset button.

- **#121 Menu, Theme and Publish tabs.** *Done 2026-10-27*: the Site tab
  was long; Menu, Theme and Publish (Build and the publish targets) now
  have tabs of their own next to it (kept mounted, like Site, so unsaved
  edits survive switching). Site keeps the folder and the settings. "Show the main menu in a sidebar" moved
  to the Menu tab and saves at once. Fixed on the way: saving Site
  settings no longer brings back older colors or menu layout from its own
  copy of `site.toml`.

- **#122 Warning before losing unsaved changes.** *Done 2026-10-27*:
  closing the window, opening another site, closing the site and
  "Install and restart" ask first when something is unsaved: the open
  page, footer or shared block (content or settings), Site settings or
  the hand-made menu. Each part reports itself (`trackUnsaved` in
  `src/lib/unsaved.svelte.ts`); the window's close request waits for the
  answer (`onCloseRequested`, permission `core:window:allow-destroy`).

- **#032 Publish history.** Record each publish run per target as a git tag
  (#073) plus a log; show progress live. *Tags done 2026-10-21* (with
  #073); the log and a list of publishes per target are still open.
  *Closed 2026-10-27*: written for the multi-user web app (who published
  what); in the desktop app the Versions tab shows each publish as a tag
  on its version and progress shows live, which is enough.

- **#033 Images and media library.** Import, tag, search and resize images to
  WebP; keep PDFs for downloads. Phase 2 adds Apple's cover art rules
  (1400–3000 px square). *Media library done 2026-10-21*
  (`qed_core::media`): list with size, dimensions and usage, import with
  clean names, delete only unused files, image picker, Image block with
  width/height and lazy loading. Still open: resizing to several sizes
  with a build cache (#075, done), tags.
  *Closed 2026-10-27*: tags dropped (a podcaster's site has dozens of
  images; the Media tab's search by name is enough); Apple's cover art
  rules moved to #066.

- **#086 Testing on macOS and Windows.** The installers build in CI but
  were never run on macOS or Windows; friends try them and report back.
  *Closed 2026-10-27*: no item of its own; testing on macOS and Windows
  goes on with every release, and problems come back as new items.

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
- #081 Jekyll import, #089 raw HTML in Markdown, #090 redirects from
  `.html` addresses, #091 link pages and dropdown menus (2026-10-26): they
  would pull q.e.d. towards Jekyll; the owner's Jekyll sites move over by
  hand instead.
- #027 Publisher: ZIP. The built site already is a folder (`_site/`); the app
  has "Show in file manager" instead.

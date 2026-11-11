# Working with Stefan on qed

Project rules are in AGENTS.md; this file holds what Claude learned while
working on qed (preferences, corrections, project state). Where it differs
from AGENTS.md, this file wins. It replaces Claude's memory files: put new
preferences, corrections and status updates here. It is public, so no third
parties' names.

## qed project vision

qed is an open-source static site generator for podcasters who can't run a static site generator themselves. Repo: /daten/DATA/rust/qed (moved from /daten/DATA/phoenix/qed on 2026-10-21; Claude project folder copied to match).

History: started 2026-10-20 as a Phoenix + Ash + SQLite web app (commits 627130e, 7e3d6e2). On 2026-10-21 the user pivoted to a **single-user desktop app in Rust + Tauri 2** with a SvelteKit + TypeScript UI. No database: a site is a folder with `site.toml` and Markdown pages/posts (TOML front matter, `::: block` containers), history via git managed by the app is planned (#073), not built yet. Same repo; the Elixir code was deleted 2026-10-21 (scaffold #061 done: Cargo workspace src-tauri + crates/qed-core). License GPL-3.0.

The user's goals, in priority order: (1) a showcase open-source project built with Claude, (2) learning Rust (ownership/borrowing), (3) something podcasters benefit from.

Built in two phases: (1) CMS + site generator, (2) podcasting. Product decisions carried over (see docs/pages/backlog.md): podcast optional per site (pages-only sites possible), `/` is always the CMS `home` page, block CMS designed after sibling /daten/DATA/phoenix/mercator (read-only reference, see docs/notes/cms-handover-qed.md), one audio format + one feed, daisyUI themes compiled once, Podlove Web Player, publish via SFTP (password or generated key) / GitHub Pages (deploy key); ZIP dropped. Feed fields modelled after /daten/DATA/jekyll/jekyll-octopod.

**Why:** the user's product vision and learning goals shape every choice.
**How to apply:** no auth/multi-user features; favour idiomatic Rust and explain Rust concepts as they come up (rust-learning-mode); keep docs/pages/backlog.md current (backlog-convention).

## Ask questions freely

Ask the user questions whenever a design decision is open, rather than picking silently.

**Why:** user said so explicitly (2026-10-20); they also have sibling repos with existing implementations of many features and will point to them "in time".
**How to apply:** before modelling a feature, ask about open choices; also ask whether a sibling repo already implements it before building from scratch. See qed-project-vision.

## Dev server is user-run

The user runs the dev app themselves (now `npm run tauri dev`; in the Elixir era `mix phx.server`). If Claude needs it and it's not running, ask the user to start it — don't start it yourself.

**Why:** user's explicit instruction (2026-10-20): "always prompt in sessions, when I forget to start it".
**How to apply:** before checks against the running app (Vite on localhost:1420), check whether it responds; if not, ask the user to start it.

To see the running app: find the viewable window with `xwininfo -root -tree | grep '"qed"'` (pick the large, mapped one, not the 10x10 ones) and capture only it with `import -window <id> -resize 50% <scratchpad>/qed-window.png`. Never capture the whole screen.

## Backlog convention

qed tracks future work and open decisions in `docs/pages/backlog.md` (moved from the repo root on 2026-10-21; it is a page of the docs site with TOML front matter), each item numbered `#nnn` (started 2026-10-20). Numbers are permanent; finished/decided items move to `docs/pages/done.md` ("Done / decided", dropped ones under "Dropped"; split out 2026-10-26). Partly done items stay in the backlog.

**Why:** user asked for it to carry work across sessions.
**How to apply:** read docs/pages/backlog.md at session start; add new open items/decisions there with the next free number; refer to items by number.

## Rust learning mode

The user chose "I write, I explain" for the Rust rewrite of qed (2026-10-21): Claude writes the code. Explanations of Rust (ownership, borrowing, lifetimes, traits, errors), Svelte and TypeScript go into **blog posts** in `docs/pages/blog/` (Markdown with TOML front matter: short sidebar `title` ~25 chars, full headline as `description`, `menu = "main"`, next `weight`, date, authors, tags, summary; also listed in docs/pages/blog/index.md — rule since 2026-10-21, see AGENTS.md), not into the chat. The user knows Elixir well but has only read about Rust, Svelte and TypeScript.

**Why:** learning Rust is the user's second goal; they want to reread the explanations later, and the posts will become a demo site built with qed (backlog #071) — part of the showcase goal.
**How to apply:** when a change introduces a concept new to this project, write or extend a post tied to the concrete code (link files), compare with Elixir where it helps, link official docs (Rust book, svelte.dev, TS handbook, Tauri) inline plus a "Further reading" list — the user asked for links, and mention the post in one line in chat. Don't repeat concepts already covered. Related: qed-project-vision.

## App UI density

For qed's desktop UI, the user wants layouts that use the whole window and avoid wasted space: no centered max-width column, side panels widen on large screens with fields in two columns (container queries), and related short lines sit side by side (e.g. header: app label + folder/stats on one line, site title + description on the next). Confirmed "perfect" on 2026-10-21 after these changes.

**Why:** the user runs the app full HD and noticed scrolling while most of the window was empty.
**How to apply:** design new app screens desktop-first and dense; check at 1920px width that nothing needs scrolling that could fit side by side. Generated websites are different — they keep readable max widths. Related: qed-project-vision.

## Shell pitfalls

`pkill -f "<pattern>"` matches the running Bash command line too. If the same command also started the process (e.g. `python3 -m http.server 8767 …; …; pkill -f "http[.]server 8767"`), the literal text in the command line matches and the shell kills itself (exit code 144), skipping the rest. Happened twice on 2026-10-21.

**Why:** lost output and half-finished steps.
**How to apply:** stop background helpers in a separate Bash call, or kill by PID captured at start (`server_pid=$!` with `&`), never with a pattern that occurs in the same command line.

**Second pitfall (2026-10-27):** a Bash command whose text contains an unbalanced backtick outside the quoted heredoc (e.g. a `grep "```"` after a `python3 - <<'EOF'` block) fails to parse ("Dateiende beim Suchen nach »`«") — but the heredoc part may already have run. For edits whose text holds ``` or many backticks, write the Python script to the scratchpad with Write and run `python3 -I script.py`; afterwards check the file's state before retrying.

## qed status

State on 2026-11-11: phases 1 (CMS + site generator), 2 (podcasting) and 3 (beyond the basics) are done. **v0.18.0 is published**; **v0.19.0** (#155 blocks as previews, #154 follow-ups) is committed and tagged, not pushed yet. The 0.17.0 release is still a draft on GitHub. Last backlog item number: #155. Learning posts: weights 1–79 (next post is 80).

**Next:** Phase 4 "Podcast Publisher Import" (#123, #034, #124, #151 new manual screenshots last), then Phase 5 "Roll out" (#070).

**App tabs:** Pages · Media · Downloads (folder assets/) · Static · Audio* · Transcripts* · Site · Podcast · Contributors* · Menu · Theme · Publish · Versions · App (* podcast sites only).

**Wiring:** bindings.ts generated by tauri-specta (`cargo test -p qed` rewrites it — run twice). Unit enum variants need `as Block` casts in site.ts newBlock. Throwaway site actions: crates/qed-core/tests/tmp_*.rs. After template class changes: `npm run build:site-css`. Rust/template changes need the user to restart `npm run tauri dev`. The app also runs as an MCP server (`qed mcp <site>`, one binary). PDF export (Typst) is slow on long tables: split them into 30-row tables, no code highlighting, no footnotes in tables. When removing a backlog item by slicing up to the next item, check that no section heading lies in between.

**Releases:** github.com/informatom/qed (remote `github`). Changelog in docs/pages/changelog.md; Claude bumps the version (Cargo.toml + `npm version X --no-git-tag-version`, `cargo update -w`), commits "Version X", tags vX (moving an unpushed tag is fine); the user pushes main + tag (`git push origin main`, `git push github main`, `git push github vX`; the tag push to GitHub starts the release workflow); CI (~20 min) drafts; the user publishes. No draft check (see No draft check).

**How to apply:** read this plus docs/pages/backlog.md at session start; update both when a step lands. The test sites are under qed test sites.

## Git workflow

Each step ends with Claude summarizing what to try, the user testing it in the running app, and then saying "commit" (sometimes "commit and continue"). Claude then commits with a descriptive message plus the Co-Authored-By line. The user said "I prefer to push myself" (2026-10-21) — never push, never create remotes or PRs.

**Why:** the user wants to see each step working before it lands, and keeps control of what goes public.
**How to apply:** don't commit unasked; before committing check that `targets.toml` (server names) isn't staged and that test files the user created in docs/ or examples/ are intended. docs/ lives inside this repository, so the app itself commits there when the user tests or publishes the docs site ("Before publishing to Alpha", "Before moving /x", "Work in progress"); such commits can sweep up uncommitted docs changes of Claude's (backlog, posts) — check `git log` before reporting what's committed. The user sometimes commits a docs change himself. Rewriting history is off the table once pushed; ask before squashing local commits. Theme changes in a `site.toml` (e.g. docs/site.toml, from trying themes in the app) are always fine: include them in the commit without asking or pointing them out (user, 2026-10-21). Related: qed-status.

## Not Jekyll

Don't shape q.e.d. after Jekyll: no Jekyll importer, no features whose only reason is Jekyll compatibility (raw HTML pass-through, `.html` redirects, Jekyll-style permalinks or menus). The user's Jekyll sites (/daten/DATA/jekyll) are moved over by hand and serve as usability tests.

**Why:** 2026-10-26 the user stopped a half-built importer: "we push q.e.d. in direction of Jekyll, what I don't want." q.e.d. is a block CMS for non-technical podcasters.
**How to apply:** when a migrated site needs something q.e.d. lacks, discuss it as a feature in q.e.d.'s own terms (blocks, page tree) before building; ask whether it's worth having without the Jekyll context. Related: qed-project-vision, ask-questions-freely.

## qed test sites

Real Jekyll sites are moved over by one-off scratchpad scripts (no importer, see not-jekyll); gaps found become backlog items. Sites live in /daten/DATA/rust/sites/; each has its own git history started via q.e.d. (the app records saves there).

**podcasterei.at** — the main working example (user, 2026-10-27; not informatom.com). 145 pages. Custom theme + saved set "Podcasterei" (#fc6e51 orange, #434a54 slate), header bar chosen by the user, language de-AT, font chosen by the user, footer.md (Letzte Posts / Meetup / Kontakt), start page Hero (picnic photo) + Call to action for the meetup on 25 Nov 2026 (update after that), Podcaster:innen via Page list (person summaries = podcast names), meetup slide shows in static/ (meetup7–14) linked from the notes and the Meetups page, Video block on the Unsubscribe post. Not published yet: the user trains its editors first.

**waldgruber.at** — a private homepage; Jekyll source in /daten/DATA/jekyll/waldgruber.at/homepage (that repo also holds old Rails apps mcr/aocqa/blog — not part of the site). Converted 2026-10-27 into /daten/DATA/rust/sites/waldgruber.at: 26 posts in blog/ (cards on the start page), 53 Stofftiere (Text with image, tiles overview), inhalte pages (raw HTML converted to Markdown, Skills as tables), 3 remark presentations + remark.js + font-awesome.css in static/, 90 redirects from old .html addresses. Hand-drawn SVG tree logo media/logo.svg, custom theme "Waldgruber" (#337ab7 / #d9edf7 / #3c763d), font Arimo (stand-in for the old Helvetica/Arial), footer.md (LinkedIn · 2016 – 2026). 53 unused small Stofftier images deleted; 4 old unused images left for the site owner to decide. Media usage doesn't see references from static/ (rudolf-steiner-schule-logo.png is used by a presentation). Since 2026-10-28 also pushed to git@code.informatom.com:Ponysee/waldgruber.at.git (remote origin, Gitea, deploy keys) and cloned onto the user's mini PC with "Get a site from git"; edited on both and pulled back (#129 test). The site owner asked for plushie images to open big (#117 done, also for her 4K monitor); static/boja-2017 etc. are remark slide shows identical to the old Jekyll ones.

**informatom.com** — converted 2026-10-26 into /daten/DATA/rust/sites/informatom.com (21 pages, 98 posts, sidebar navigation); parked for a day because too many gaps frustrated the user, back as test site 2026-10-27 afternoon. Finished then: 17 presentations from the old _site in static/presentations/ (+ /wikidata/, remark.js, Lato, CC badge), keybase.txt, id_rsa.pub, robots.txt, favicon.ico, PDFs/zips back at /pdf/ and /materials/, 98 old post addresses as .html redirects; medium wordmark logo without title, start page hide_title and columns 2:1. History started (e24ea15 First version, 659 files, 86 MB). Later on 2026-10-27: hand-made top menu (Stacks & Services and Podcasts dropdowns, old labels) instead of the sidebar, logo size original, page width wide, footer.md with four columns (Meta, Neueste Beiträge, Adresse, Kontakt; the six Meta pages taken out of the footer menu), custom theme + saved set "Informatom" (#8cc152 logo green, #e6f2d9, #507329 accent, #3f4654, background #f9fafb). Since then also boxed columns, footer icons, and the 14 old badges as site icons in media/icons/ (#115: stack list and IT-Beratung on the start page). mark_external_links = true in site.toml (#116). Next ideas against the wall of text: Post list cards in the "Mein Blog" box, podcast covers in "Podcasting".

**stefan-wills-wissen** — the podcast test site (stands in for the archived layer8 podcast, same complexity), converted 2026-10-28 from /daten/DATA/jekyll/stefan-wills-wissen (jekyll-octopod) into /daten/DATA/rust/sites/stefan-wills-wissen by scratchpad script convert_sww.py: `[podcast]` (cover = octopod's default logo-itunes.jpg, 1400 px), 2 episodes with chapters, old guids (`https://podcast.stefan-haslinger.at/2017/08/22/episode1.html`) and redirects from the old .html and /players/ addresses, Impressum, Datenschutz, footer (CC BY 4.0, Mastodon); About (theme boilerplate) dropped. Not carried over yet: subtitle (in `summary`), duration, audio (mp3 only, outside the repo in /daten/DATA/PODCAST-EPISODES/stefan-wills-wissen/, old URL /episodes/episodeN.mp3), contributors (name, uri, email) — all still in the Jekyll source. History started by the user on 2026-10-28 right after the conversion (a clean point to go back to). Later on 2026-10-28: audio imported (episode1/2.mp3 in audio/, local, not in history), Latest episode + Subscribe blocks on the start page (by the user), and the old jekyll-octopod-bulma look as custom theme + saved set "Stefan will's wissen" (#48cfad mint primary/accent, #7bd1bd secondary, #4a4a4a neutral, soft corners), header bar primary, footer shaded, font Ubuntu (fonts/ubuntu). Also: contributors/ (Stefan Haslinger = team host, two guests in ep. 1/2), a made-up demo transcript transcripts/episode1.vtt linked to episode 1 (its NOTE says so). The site is the podcast test bed: Podlove player, subscribe button, feeds verified there.

**3-schweinehun.de** — second podcast test site (running podcast, 4 hosts), converted 2026-10-28 from /daten/DATA/jekyll/3schweinehunde (jekyll-octopod-bulma) into /daten/DATA/rust/sites/3-schweinehun.de by scratchpad script convert_3sh.py (session bc0e2e63): 52 episodes (0–51, subtitle = title, old guids/redirects like sww), audio hard-linked from /daten/DATA/PODCAST-EPISODES/3-schweinehun.de into audio/ (local), 34 contributors from the posts' contributors (latest uri as link, emails dropped, no photos; team = Stefan Haslinger and three co-hosts as hosts, a fifth host where present, others guests), category Sports > Running, sww's mint theme + Ubuntu, start page Latest episode + Subscribe + Post list, Letzte Läufe = link to the Strava club (the iframes dropped), /img → media/, /pdf + /video → static/, the two videos are Video blocks with files in assets/ (#127). Datenschutz text still mentions Google search and Strava embeds (outdated, user's call). History not started yet.

**blog.panoptikum.social** — Panoptikum's English blog (no podcast), Jekyll source /daten/DATA/jekyll/blog.panoptikum.social. The hand-made copy of 2026-10-30 was removed on 2026-10-31 (#147); the site in /daten/DATA/rust/sites/blog.panoptikum.social was made anew (like the app's New site) and filled by two headless Claude Code runs through `qed mcp` only: 62 posts, 13 pages, 75 images, 69 redirects, slide shows + scripts in static/ at their old addresses, PDFs at /pdf/…, logo, footer (Supported by, Follow), hand-made top menu. The user set web address, themes catppuccin-latte + [dark] catppuccin-mocha, profiles = fosstodon @panoptikum. Since 2026-11-01 in production, published from this q.e.d. site (#147 closed; user: the MCP server works nicely). Mastodon rel="me" verification done with the q.e.d. site (user). The site folder is the source; never regenerate it.

**How to apply:** when the user mentions one of these, work in its folder under /daten/DATA/rust/sites/; don't edit the Jekyll sources; site changes show as open changes until the next app save.

## Backlog pruning

When the user asks about old backlog items ("who brought this up and why?"), trace the origin (git log -S on backlog files; the first backlog is from the Phoenix web app, commit 7e3d6e2, 2026-10-20) and say plainly whether it still fits the single-user desktop app. The user then closed #032, #033 and #086 on 2026-10-27; ongoing activities (e.g. testing on other OSes) don't get items of their own.

**Why:** many items were written for the multi-user web app (users, roles, per-user logs) or for a shop (Mercator: image tags); they inflate the backlog.

**How to apply:** before building a leftover item, check what's already covered and recommend closing or shrinking it; closed items go to done.md with a "Closed <date>" note saying why. Related: backlog-convention, qed-status.

## No third parties

Generated sites self-host everything (Podlove Subscribe Button, Podlove Web Player, fonts) and use no third-party services in between: no directory links (Apple, Spotify) for subscribing, no share services (Mastodon via the reader's own server), no external CDNs.

**Why:** the user, 2026-10-28: "we have the feed here, and that's where people should subscribe and not from any 3rd party services. Our feed(s) are the source of truth." Earlier choices point the same way (self-hosted fonts, Podlove player copied into each site).

**How to apply:** when a feature would link to or load from an outside service, propose a self-hosted or own-feed variant first; vendored front-end code goes into crates/qed-core/assets/<name>/ with a provenance README (AGENTS.md rule). JavaScript only as optional enhancement with a no-JS fallback. Related: qed-project-vision, qed-status.

## User-facing wording

When describing q.e.d. to users (docs front page, README intro, app descriptions, changelog lead-ins), lead with what they love: the whole site or podcast site is made in the app's GUI, without programming, and published to their own webspace. The "a site is a folder" fact is true but secondary; keep it for developer-facing parts (README "How it works", AGENTS.md, learning posts).

**Why:** the user (2026-10-28): users won't care much about folders; they'll love creating the site without any programming, just within the GUI.

**How to apply:** check new user-facing text for folder-first framing; see qed-project-vision.

## Example people

When docs, manual pages or blog posts need an example person (a contributor, a guest, an author), use Stefan Haslinger himself (slug `stefan-haslinger`, link informatom.com), not real third parties such as podcast guests.

**Why:** on 2026-10-30 the user had a real guest's name replaced in the contributors post to respect their privacy. The waldgruber.at owner's name in the manual's Podcasts screenshot is fine: they agreed. Don't write third parties' names into this file either. Git history doesn't need rewriting for this.

**How to apply:** pick Stefan (or made-up names) for new examples; made-up first names in Rust tests are fine. Related: user-facing-wording, rust-learning-mode.

## No source churn while dev runs

Twice on 2026-10-31 a write to watched files restarted `tauri dev` and cut off the user's running publish: in the morning a `git stash`/`pop` for a check, in the evening the version bump to 0.15.0 (Cargo.toml + Cargo.lock), done right after the user had said a feature worked, while he was publishing.

**Why:** the dev server watches src-tauri, crates and Cargo.toml/Cargo.lock; any write there recompiles and restarts the app, killing whatever it was doing (publishing over SFTP, saving). The user publishes often between steps, also while Claude works.

**How to apply:** before a change to Rust sources, Cargo.toml/Cargo.lock (version bumps, `cargo update`, new dependencies) or a stash/checkout, ask in one line whether the app is busy (publishing) and wait — unless the user has just asked for that very change in this message. For checks, use a scratchpad copy or git worktree instead. Svelte-only edits hot-reload without a restart. Related: dev-server-user-runs, git-workflow.

## No draft check

Don't check the CI-built draft release any more, and don't offer to.

**Why:** the user said on 2026-10-31 that the drafts worked the last 12 times, so the check is no longer needed.

**How to apply:** after "Version X" + tag, just tell the user it's ready to push; they push and publish themselves. Related: git-workflow, qed-status.

## Test MCP headless

The user runs Claude Code only inside VS Code; `claude` is not on the PATH. The extension's binary is at `~/.vscode/extensions/anthropic.claude-code-<version>-linux-x64/resources/native-binary/claude` (pick the newest version). Linux has no Claude Desktop.

To test `qed mcp` with a real agent: write an MCP config JSON in the scratchpad (`{"mcpServers":{"name":{"command":".../target/debug/qed","args":["mcp","<site>"]}}}`) and run `claude -p "<task>" --mcp-config <file> --strict-mcp-config --allowedTools "mcp__<name>__site_info,..."` from the scratchpad. Allow only read tools on the user's real sites (e.g. blog.panoptikum.social, see qed-test-sites) unless the user asked for changes. `claude mcp list` in a folder with `.mcp.json` shows project servers as "Pending approval". Found two bugs on 2026-10-31 this way. Without an agent: `python3 dev/mcp-smoke.py <program> <site>` (read-only smoke test); on Windows the "MCP on Windows" GitHub workflow runs it against an installed release (`gh run list --workflow mcp-windows.yml`, `gh run view <id> --log`). Related: qed-status.

## Scratchpad: ask first

Don't use the session scratchpad without asking the user first: no git worktrees there, no copies of sites, no test projects or screenshots. Clean up whatever was put there.

**Why:** on 2026-11-06 (#154) Claude built the PDF export in a scratchpad worktree with its own target/ (Typst's ~220 crates compiled several times over) plus copies of the docs and test sites; the user found the repeated long compiles and the hidden work tiresome and asked for the scratchpad to be cleaned up.

**How to apply:** when work would go into the scratchpad (e.g. to avoid restarting `tauri dev`, see no-source-churn-while-dev-runs), ask first and say what and why; otherwise work in the checkout. Keep compile runs few; don't start optional builds (e.g. size measurements) without saying so.

## Ask before compiling

Before every command that compiles Rust, ask the user and wait for a yes: `cargo build`, `cargo run` (examples too, e.g. `--example build`), `cargo test`, `cargo check`, `cargo clippy`, `npm run check:all`, release builds. Say what it compiles and why, in one line. Plain `cargo fmt` and non-Rust commands don't count.

**Why:** on 2026-11-06 (#154, Typst) Claude compiled the ~220 new crates many times (a scratchpad worktree, profile changes, a release build for a size measurement, then `cargo run --example build` just to check a page edit, while rust-analyzer was checking too). The user: "every time ask for permission to compile".

**How to apply:** batch what needs compiling into one asked-for run; check page edits through the user's running app instead of the command-line generator. No `npm run check:all` (or any compile) before the user's commits: the user said his commits need no compilation; his test in the app is enough. This overrides AGENTS.md's check:all rule. Related: scratchpad-ask-first, no-source-churn-while-dev-runs, dev-server-user-runs.

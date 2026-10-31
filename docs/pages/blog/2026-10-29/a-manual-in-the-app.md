+++
title = "A manual in the app"
description = "The user manual in the app and on the website: pages compiled into the program, built as a small site, shown in a second window that sends outside links to the browser"
menu = "main"
weight = 65
date = 2026-10-29
authors = ["Claude"]
tags = ["rust", "tauri", "docs", "build"]
summary = "q.e.d. has a user manual now (#110): pages for site owners, on this website under Manual and in the app, offline, through the life-buoy button. The same files serve both: the app compiles them in, builds them as a small site with its own generator, and shows it in a second window."
redirect_from = ["/blog/2026-10-29-a-manual-in-the-app"]
+++

q.e.d. was explained so far for developers, in this blog. Now there's a
[manual](../../manual/index.md) for the people who use it: from getting
started to menus, colors and fonts, podcasts and moving a site over
(#110). It's on this website, and in the app: the life-buoy button next
to the preview button opens it, and small **Manual** links in most tabs
open the page about that tab.

## One set of files, two places

The manual's pages are ordinary q.e.d. pages in `docs/pages/manual/`,
with columns, FAQs, calls to action and icons next to button names
(`:icon-eye:`). On this website they're simply pages of the docs site.

For the app, `crates/qed-core/src/manual.rs` compiles them in with the
same `include_dir!` macro that already brings along the Podlove player:

```rust
static PAGES: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../docs/pages/manual");
static MEDIA: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../docs/media/manual");
```

So the app always has the manual of its own version, without an
internet connection. When it's opened, `manual::build` writes the pages
into a small site folder (the manual's start page as its start page), in
the app's own theme, and builds it with the site generator, the same one
that builds every q.e.d. site. Blocks, icons and screenshots look exactly
as here, with no second way of showing them to keep in step.

For that, links have to work in both places. Manual pages link to each
other by file name (`[Versions](versions.md)`), which both sites resolve,
and to anything else with a full web address. A test builds the manual
site and fails on a link the build notes as missing (#133), so a broken
one can't slip in.

## A build script that watches files

`include_dir!` reads the files while compiling. But Cargo only knows to
compile again when Rust code changes; an edited manual page would go
unnoticed. A [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
tells it what else to watch. `crates/qed-core/build.rs` is three lines
of output:

```rust
fn main() {
    println!("cargo:rerun-if-changed=../../docs/pages/manual");
    println!("cargo:rerun-if-changed=../../docs/media/manual");
}
```

Cargo reads what a build script prints: a line starting with `cargo:` is
an instruction, here to run it again, and so to compile the crate again,
when anything in those folders changes. (The catppuccin crate's build
script, in [Catppuccin themes](catppuccin.md), used the same
mechanism to turn JSON into Rust.)

## Built once, in the app's theme

Building the manual takes a moment, so the app does it the first time
it's opened, and again only when the app's theme has changed since. The
UI sends the theme along (`src/lib/manual.ts`: the one chosen in the App
tab, or light or dark as the system has it). What was built is kept in
Tauri's managed state, like the preview server in
[From folder to website](../2026-10-21/from-folder-to-website.md):

```rust
struct ManualState(Mutex<Option<(String, Preview)>>);
```

Nothing yet, or a pair: the theme it was built in and the server that
serves it. The pair is a [tuple](https://doc.rust-lang.org/book/ch03-02-data-types.html#the-tuple-type),
two values without a struct around them, as `{theme, server}` would be
in Elixir. `open_manual` decides with one call:

```rust
if manual.as_ref().is_none_or(|(built, _)| *built != theme) {
    // build, start a server, store Some((theme, server))
}
```

[`is_none_or`](https://doc.rust-lang.org/std/option/enum.Option.html#method.is_none_or)
is true for `None`, and for `Some` it asks the closure. The closure takes
the tuple apart right in its parameter, `(built, _)`, the way a function
head pattern-matches in Elixir; `_` skips the server. Storing the new
pair drops the old one, and with it the old server, which stops (its
`Drop`, as with the preview).

The whole part is wrapped in a block, `let base = { … };`, that hands
back only the server's address as a `String`. The lock on the Mutex is
a value too, and it's released when the block ends, so the window below
is opened without holding it.

## A second window

The built manual is served by the same small web server as the preview,
and shown in a window of its own, so it can stay open next to the main
one. `open_manual` in `src-tauri/src/lib.rs` makes it with Tauri's
[`WebviewWindowBuilder`](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html):

```rust
tauri::WebviewWindowBuilder::new(&app, "manual", tauri::WebviewUrl::External(url))
    .title("q.e.d. manual")
    .on_navigation(move |target| {
        if target.as_str().starts_with(&base) {
            return true;
        }
        let _ = opener.opener().open_url(target.as_str(), None::<&str>);
        false
    })
    .build()
```

`on_navigation` is asked before the window follows any link. The
manual's own pages (addresses starting with the local server's) may
load; anything else, the releases page on GitHub for example, goes to
the browser instead, and the window stays where it was. The closure
returns `bool`, so the decision is one line, and `move` hands it its own
copies of `base` and the app handle, since it's called long after
`open_manual` has returned.

The label `"manual"` makes it one window: opening the manual again
finds it with `app.get_webview_window("manual")`, sends it to the
requested page and brings it to the front. It also keeps the window out
of the app's permissions: those name the main window only
(`src-tauri/capabilities/default.json`), so pages shown in the manual
window can't call q.e.d.'s commands.

## Manual links in the tabs

The small **Manual** links are one component, `src/lib/ManualLink.svelte`,
with two props: the page (`versions/`) and the topic for its tooltip.
The Versions tab, for example, has

```svelte
<ManualLink page="versions/" topic="Versions" />
```

A click calls `openManual(page)`, which calls the `open_manual` command.
If that fails, the error goes into a `$state` variable and shows next to
the link, instead of disappearing in the console.

## Screenshots

The manual has a few screenshots, all taken with the app's theme set to
*catppuccin-mocha*, so they look alike. They were taken from the running
app with ImageMagick's `import -window`, which captures one window by its
id on X11. The one of the Publish tab has the server's address blurred.
The rule about the theme is in `AGENTS.md`, for whoever takes the next
one.

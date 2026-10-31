+++
title = "Site search"
description = "Site search with Pagefind: an async library on a thread of its own, and custom HTML elements that only exist with JavaScript"
menu = "main"
weight = 56
date = 2026-10-28
authors = ["Claude"]
tags = ["rust", "search", "threads", "html"]
summary = "Every q.e.d. site can now be searched (#108): Pagefind indexes the built pages, and a button in the header opens a search dialog. In Rust that means calling an async library from plain code; on the website, HTML elements the browser only learns about when JavaScript runs."
+++

Sites made with q.e.d. now have a search button in the header (#108).
It opens a dialog, and results show up while you type. Ctrl+K (⌘K on a
Mac) opens it as well. The search runs in the visitor's browser, from
files on the site itself; nothing is sent anywhere. The Site tab can
switch it off.

## Pagefind

[Pagefind](https://pagefind.app) is written in Rust, and it's a library
as well as a program. After q.e.d. has written all pages,
`crates/qed-core/src/search.rs` hands the output folder to it:

```rust
let mut index = PagefindIndex::new(None)?;
index.add_directory(dir, None).await?;
index.write_files(Some(bundle_path)).await?;
```

Pagefind reads every HTML file, splits the words, reduces them to their
stem ("Läufe" and "laufen" both become "lauf") and writes the index into
`_site/pagefind/`, in small chunks. A browser searching for one word
loads only the chunk with that word, not the whole index. The docs site's
62 pages make 936 KB of files, most of it Pagefind's own scripts. Indexing
them takes about a tenth of a second in a release build.

## Calling async code from plain code

`add_directory` and `write_files` are `async fn`s. The generator isn't
async: `generator::build` is a plain function, called from tests, from
the `build` example and from the app. To wait for a future, you need a
runtime. But the app calls `build` from a Tauri command, which already
runs on Tauri's tokio runtime, and starting a runtime inside another
panics ("Cannot start a runtime from within a runtime"). Pagefind gets a
thread of its own instead:

```rust
std::thread::spawn(move || {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(SearchError::from)?;
    runtime.block_on(async {
        // … add_directory, write_files …
    })
})
.join()
.map_err(|_| SearchError("the indexer stopped unexpectedly".to_string()))??;
```

A new thread belongs to no runtime, so `block_on` is allowed there.
[`join`](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html#method.join)
waits for the thread and returns what its closure returned. The two
question marks at the end each unwrap one layer. `join` gives
`Err` if the thread panicked (the first `?`), and the closure's own
`Result` has Pagefind's error (the second `?`).

The `move` in front of the closure hands the variables it uses (the folder
paths) over to the thread. A thread may outlive the function that started
it, so it can't borrow them. That's the same rule as in
[Publishing over SFTP](2026-10-21-publishing-over-sftp.md) with
`spawn_blocking`.

In Elixir you'd write `Task.async(fn -> … end) |> Task.await()`: start
the work elsewhere, wait for the answer here. The difference is that Rust
has to be told which runtime runs the async part, while the BEAM always
has one.

## What gets indexed

Without instructions, Pagefind would index every page completely: the
menu, the footer, the redirects from old addresses. `templates/page.html`
marks what matters:

```html
<main … data-pagefind-body data-pagefind-meta="title:{{ title }}">
```

As soon as one page has `data-pagefind-body`, Pagefind indexes only pages
that have it, and only inside that element. Redirect pages, the 404 page
and HTML files in `static/` (old slide shows, for example) stay out by
themselves. `data-pagefind-meta` gives the result its title. A page that
starts with a Hero has no title heading, so Pagefind couldn't find one on
its own.

Some blocks only repeat other pages: Post list, Page list, Latest
episode. Results should lead to the post, not to every list that mentions
it. `block_html` in `generator.rs` wraps them in
`<div data-pagefind-ignore>`. The episode player gets the same, because
its plain version for browsers without JavaScript ("29:55 · Download:
MP3, 28.9 MB") would otherwise become the excerpt.

## Elements that only exist with JavaScript

The search button and the dialog are two tags in the template:

```html
<pagefind-modal-trigger compact></pagefind-modal-trigger>
…
<pagefind-modal></pagefind-modal>
```

These aren't HTML elements a browser knows. They're
[custom elements](https://developer.mozilla.org/en-US/docs/Web/API/Web_components/Using_custom_elements):
a script (`pagefind-component-ui.js`) registers them with
`customElements.define("pagefind-modal", …)`, and from then on the
browser draws them as whatever the script says. Without JavaScript, the
browser treats an unknown element like an empty `<span>`, and there's
nothing to see. So there's no search button that does nothing, and no
search box sending visitors to a search engine. Visitors without
JavaScript don't miss anything that would have been there for them.

In Phoenix terms, this is a bit like a LiveView hook, except that the
browser itself takes care of it: any `<pagefind-modal>`, wherever it
appears, comes to life once the definition is loaded.

The script is loaded with `defer` (after the page is read), not as a
module. That matters for one detail: as a classic script it can ask
`document.currentScript` where it was loaded from, and so find the index
next to itself. That works when a site lives in a subfolder, like
`example.com/docs/`, without any setting.

## Colors from the theme

Pagefind's dialog takes its colors from CSS variables (`--pf-text`,
`--pf-background`, …). `assets/site.input.css` sets them from daisyUI's
theme colors:

```css
:root {
  --pf-text: var(--color-base-content);
  --pf-background: var(--color-base-100);
  --pf-border-focus: var(--color-primary);
  /* … */
}
```

Pagefind sets the same variables on `:root` in its own stylesheet. With
equal selectors, the later one wins, so `page.html` loads Pagefind's
stylesheet first and `style.css` after it. The dialog then matches every
theme, the custom ones too. Pagefind's files aren't changed at all;
`assets/pagefind/README.md` says where they come from and how to update
them.

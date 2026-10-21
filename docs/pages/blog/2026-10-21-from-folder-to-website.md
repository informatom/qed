+++
title = "From folder to website: templates, iterators and a server in a thread"
date = 2026-10-21
authors = ["Claude"]
tags = ["rust", "askama", "iterators", "closures", "threads", "tauri"]
summary = "qed now builds real websites, including this one. The generator brings templates the compiler checks, iterator chains, closures, a web server in a background thread, and shared state with Arc and Mutex."
+++

Until now, qed could *read* a site. Now it builds one. You're probably
looking at the result: this blog is a qed site, and the page you're reading
was rendered by the code this post describes.

The generator lives in
[`crates/qed-core/src/generator.rs`](../../../crates/qed-core/src/generator.rs).
For every page that isn't a draft, it renders HTML into `_site/`, then adds
the stylesheet, the media files, redirect pages for old URLs, and a sitemap.

## Templates the compiler checks

The HTML comes from templates in
[`crates/qed-core/templates`](../../../crates/qed-core/templates), rendered
with [askama](https://askama.readthedocs.io/). The syntax looks like Jinja or
Liquid:

```html
<title>{% if is_home %}{{ site.title }}{% else %}{{ title }} · {{ site.title }}{% endif %}</title>
```

The difference from EEx or Liquid: the template is turned into Rust code when
qed compiles. Each template belongs to a struct that holds its data:

```rust
#[derive(Template)]
#[template(path = "page.html")]
struct PageTemplate<'a> {
    site: &'a SiteConfig,
    title: &'a str,
    is_home: bool,
    blocks: Vec<RenderedBlock>,
    // …
}
```

If the template uses `{{ subtitle }}` and the struct has no `subtitle`, qed
doesn't compile. A typo in a template can't reach a user. HEEx gives you some
of this in Phoenix; askama goes all the way, because the template *is* Rust
code after compiling.

Templates can even `match` on Rust enums. Each kind of block is one arm:

```html
{%- match block %}
{%- when RenderedBlock::Text with { html } %}
<div class="prose max-w-none">{{ html|safe }}</div>
{%- when RenderedBlock::Hero with { image, headline, text_html, button } %}
<section class="hero …">…</section>
{%- endmatch %}
```

Everything inserted with `{{ … }}` is HTML-escaped automatically. `|safe`
turns that off, which is only right for HTML we produced ourselves, here the
rendered Markdown.

### A struct that borrows: `PageTemplate<'a>`

The `<'a>` is a lifetime, like the one in the
[second post](2026-10-21-reading-a-site-folder.md). `title: &'a str` means
the template doesn't own the page title; it borrows it from the page. The
compiler checks that the page lives at least as long as the template. Since a
template only exists for the moment it's rendered, that's easy, and nothing
gets copied just to be printed.

## Iterators: Rust's Enum module

Which pages go into the site?

```rust
let published: Vec<&Page> = site
    .pages
    .iter()
    .filter(|p| !p.content.front.draft)
    .collect();
```

In Elixir, that's `Enum.reject(site.pages, & &1.draft)`. Rust's
[iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html) are its
`Enum` and `Stream` in one:

- `.iter()` walks the list *by reference*. The pages stay where they are.
  That's why the result is `Vec<&Page>`, a list of borrowed pages, not
  copies.
- `.filter(…)` and `.map(…)` are lazy, like `Stream`. Nothing happens until
  something consumes the chain.
- `.collect()` consumes it and builds a collection. Which kind? The one the
  variable's type asks for, here `Vec<&Page>`.

The compiler turns these chains into plain loops, so they cost nothing
compared to writing the loop by hand.

## Closures that capture

Menus are built with a [closure](https://doc.rust-lang.org/book/ch13-01-closures.html),
an anonymous function stored in a variable:

```rust
let menu_links = |entries: &[MenuEntry]| -> Vec<MenuLink> {
    entries
        .iter()
        .map(|entry| MenuLink {
            title: entry.title.clone(),
            href: links.page(&entry.path),
            current: is_within(&page.path, &entry.path),
        })
        .collect()
};
```

It uses `links` and `page` without receiving them as arguments. It
*captures* them from the surrounding function, as `fn` does in Elixir. Rust
just needs to know how: here the closure only reads them, so it borrows them.
It's then called twice, once for the main menu and once for the footer.

## Links that work anywhere

Every link on a built site is relative: from `/blog/some-post/`, the
stylesheet is `../../style.css`, not `/style.css`. That way a site works at
the root of a domain, in a subfolder (as GitHub Pages serves project sites),
and in the preview.

[`links.rs`](../../../crates/qed-core/src/links.rs) rewrites what people
write:

| Written in a page    | On the built page `/blog/post/` |
|----------------------|---------------------------------|
| `/about`             | `../../about/`                  |
| `other-post.md`      | `../../blog/other-post/`        |
| `/media/logo.png`    | `../../media/logo.png`          |
| `https://…`, `#top`  | unchanged                       |

The Markdown renderer, [comrak](https://docs.rs/comrak), first parses a page
into a tree. qed walks the tree and rewrites every link before turning it
into HTML:

```rust
for node in root.descendants() {
    if let NodeValue::Link(ref mut link) | NodeValue::Image(ref mut link) =
        node.data.borrow_mut().value
    {
        link.url = links.resolve(&link.url);
    }
}
```

`borrow_mut()` is new. Normally Rust checks borrowing at compile time. The
nodes of comrak's tree point at each other, though, and the compiler can't
follow that. So each node keeps its data in a
[`RefCell`](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html),
which does the same check *while the program runs*: one writer or many
readers, otherwise it stops. Rust calls this *interior mutability*. You rarely
need it, but it's how linked structures like trees are built.

## A web server in a background thread

"Preview in browser" needs a web server. qed brings a tiny one,
[`preview.rs`](../../../crates/qed-core/src/preview.rs), built on
[tiny_http](https://docs.rs/tiny_http). It listens only on `127.0.0.1`, on a
free port the system picks, and runs in its own
[thread](https://doc.rust-lang.org/book/ch16-01-threads.html):

```rust
let thread = {
    let server = Arc::clone(&server);
    let root = root.to_path_buf();
    thread::spawn(move || {
        while let Ok(request) = server.recv() {
            let _ = respond(&root, request);
        }
    })
};
```

Three things are going on:

- **`move`** hands the captured variables over to the thread. A thread can
  outlive the function that started it, so it can't borrow from that
  function; it must own what it uses. Elixir processes have the same rule,
  enforced by copying: a spawned process gets its own copy of everything.
  Rust lets you choose, and checks that you did.
- **[`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html)** ("atomically
  reference counted") lets two owners share one value: the `Preview` struct
  keeps the server, and the thread uses it too. `Arc::clone` doesn't copy the
  server; it adds an owner. The server is freed when the last owner is gone.
- **`while let Ok(request) = server.recv()`** loops as long as requests
  arrive. That's a `receive` loop in a process, minus the mailbox.

How does the server stop? Through `Drop`, from the
[second post](2026-10-21-reading-a-site-folder.md):

```rust
impl Drop for Preview {
    fn drop(&mut self) {
        self.server.unblock();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
```

When a `Preview` value goes away, it wakes the server (`recv` then returns an
error, which ends the loop) and waits for the thread to finish. Nobody has to
remember to stop it. That's the closest Rust gets to a linked process.

## Shared state in the app: Mutex

The app keeps one preview server running across button clicks. Tauri holds it
as *managed state*, in
[`src-tauri/src/lib.rs`](../../../src-tauri/src/lib.rs):

```rust
#[derive(Default)]
struct PreviewState(Mutex<Option<Preview>>);
```

Commands can run on several threads at once, so the state sits behind a
[`Mutex`](https://doc.rust-lang.org/book/ch16-03-shared-state.html): to touch
the value, you `lock()` it, and only one thread at a time gets in. In Elixir
you'd put the state in a GenServer, which handles one message at a time. A
Mutex gives the same guarantee without a separate process.

```rust
let mut preview = state.0.lock().map_err(|_| "preview state is broken".to_string())?;
if preview.as_ref().is_none_or(|p| p.root() != report.out_dir) {
    *preview = Some(Preview::start(&report.out_dir).map_err(|e| e.to_string())?);
}
```

Assigning a new `Some(Preview)` drops the old one, and `Drop` stops its
server. The lock is released when `preview` goes out of scope at the end of
the function. Again, cleanup happens because an owner went away.

## One stylesheet for all sites

Generated sites are styled with [Tailwind CSS](https://tailwindcss.com/) and
[daisyUI](https://daisyui.com/), plus the
[typography plugin](https://github.com/tailwindlabs/tailwindcss-typography)
for Markdown text (`prose`). Since users pick a theme but don't edit
templates, the set of CSS classes is fixed. So the stylesheet is compiled
once, with all 35 daisyUI themes (77 KB, 14 KB compressed), by
`npm run build:site-css`, and built into qed:

```rust
const STYLESHEET: &str = include_str!("../assets/site.css");
```

[`include_str!`](https://doc.rust-lang.org/std/macro.include_str.html) reads
the file while compiling and puts its contents into the program. The theme
is one attribute on each page: `<html data-theme="light">`.

## Try it

From the repository, without the app:

```sh
cargo run -p qed-core --example build -- docs
```

That builds this blog into `docs/_site/`. In the app, open a site folder and
click "Preview in browser".

## Further reading

- [askama book](https://askama.readthedocs.io/): template syntax.
- [The Rust book, chapter 13](https://doc.rust-lang.org/book/ch13-00-functional-features.html):
  closures and iterators.
- [The Rust book, chapter 16](https://doc.rust-lang.org/book/ch16-00-concurrency.html):
  threads, `Arc` and `Mutex`.
- [The Rust book, chapter 15.5](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html):
  `RefCell` and interior mutability.

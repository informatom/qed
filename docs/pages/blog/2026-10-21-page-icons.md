+++
title = "Icons and a macro"
description = "Page icons: SVG files built into the program, and a first macro_rules! of our own"
menu = "main"
weight = 12
date = 2026-10-21
authors = ["Claude"]
tags = ["rust", "macros", "svg", "svelte"]
summary = "Pages can have an icon now, shown in the menus of the generated site. The icons are SVG files compiled into q.e.d., listed by a small macro_rules! macro, the first macro q.e.d. defines itself."
+++

A page can now have an icon: `icon = "book-open"` in its settings, or a click
in the icon picker of the page settings. The icon shows in the sidebar and the
top menu of the generated site, and in the app's page list. The open book next
to "Blog" in this site's sidebar is the first one.

## Icons a website can use without JavaScript

The app's buttons use [Lucide](https://lucide.dev) through its Svelte
package, which draws icons with JavaScript. Generated sites have no
JavaScript, so their icons must be plain SVG in the HTML. And the HTML is
written by Rust, which can't use a Svelte package.

So q.e.d. carries its own copies: 60 Lucide icons chosen for typical pages
(house, book, newspaper, people, mail, calendar, microphone, scales for an
imprint, a shield for privacy, …), as SVG files in
`crates/qed-core/assets/icons/`,
with Lucide's license. Each file was simplified to one line with the
classes the generated site needs.

## A macro of our own

To use the files, Rust has to read them while compiling, with
`include_str!` from the [fourth post](2026-10-21-from-folder-to-website.md).
Sixty icons would mean sixty nearly identical lines like this one:

```rust
("book-open", include_str!("../assets/icons/book-open.svg")),
```

Instead, `crates/qed-core/src/icons.rs` defines a
small macro with
[`macro_rules!`](https://doc.rust-lang.org/book/ch20-05-macros.html):

```rust
macro_rules! icons {
    ($($name:literal,)*) => {
        &[$(($name, include_str!(concat!("../assets/icons/", $name, ".svg")))),*]
    };
}

pub const ICONS: &[(&str, &str)] = icons![
    "archive",
    "award",
    // … 58 more names
];
```

Reading the pattern from the outside in:

- `$( … ),*` and `$( … )*` mean "repeat this": the macro takes any number of
  names, and produces one pair for each.
- `$name:literal` says each name must be a literal value, such as a string.
- [`concat!`](https://doc.rust-lang.org/std/macro.concat.html) glues literals
  together while compiling, so `include_str!` gets a complete file path.

The result is a list of `(name, svg)` pairs that's part of the program, and
a missing file is a compile error, not a broken page. Elixir developers know
the idea from `defmacro`, but Rust's `macro_rules!` works by pattern
matching on code instead of transforming a syntax tree: "if the input looks
like this, write that".

## A name that doesn't exist is a typo

A page with `icon = "book-closed"` gets the same treatment as other typos in
its settings: opening the site says which file and line, and what's wrong:

```text
pages/blog/index.md, line 4: there is no icon called "book-closed"; pick one in the page settings
```

## One list, two languages

The app's icon picker needs the same 60 icons. It doesn't copy them; it asks
Rust once, through a command that returns the list, and keeps the answer:

```ts
let iconCache: Promise<Map<string, string>> | null = null;

export function pageIcons(): Promise<Map<string, string>> {
  iconCache ??= invoke<[string, string][]>("list_icons").then((list) => new Map(list));
  return iconCache;
}
```

`??=` assigns only if the variable is still `null`: the first caller starts
the request, every later one gets the same promise. The SVG text is then
inserted with Svelte's [`{@html …}`](https://svelte.dev/docs/svelte/@html),
which is only safe because the markup is q.e.d.'s own; text from users never
goes through `{@html}`.

## Further reading

- [The Rust book, chapter 20.5: macros](https://doc.rust-lang.org/book/ch20-05-macros.html)
- [The Little Book of Rust Macros](https://veykril.github.io/tlborm/)
- [Lucide](https://lucide.dev)

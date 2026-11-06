+++
title = "The site as a book"
description = "The whole site as a PDF: Typst compiled into q.e.d., a World trait that decides what Typst may read, writing markup that can't be misread, and finding out why it took 90 seconds"
menu = "main"
weight = 78
date = 2026-11-06T14:23:00
authors = ["Claude"]
tags = ["rust", "typst", "pdf", "generator"]
summary = "Make PDF of the site turns the whole site into one PDF with a table of contents (#154). How Typst runs inside q.e.d. as a library, why q.e.d. hands it a whole world of its own, why every dot in the text gets a backslash, and how 90 seconds became 8."
+++

The Downloads tab has a new button, :icon-file-type-pdf: **Make PDF of
the site** (#154). It makes the whole site into one PDF: a title page, a
table of contents of the top two levels of pages, the pages in the
order of the site, then the posts and episodes in chapters of their
own. Plain black on white, with links between pages that jump inside
the PDF. The docs site you're reading becomes 271 pages.

The PDF is set by [Typst](https://typst.app), a typesetting system like
LaTeX, but young, fast and written in Rust. The owner had already used it
in a Phoenix app for offers and order confirmations. There it ran as a
separate program: write a `.typ` file, call `typst compile`, read the
PDF. Here Typst is a library compiled into q.e.d., so nobody has to
install anything.

## Two steps: markup, then Typst

`crates/qed-core/src/pdf.rs` works like the site generator, with Typst
markup instead of HTML as the output. A `Book` walks the pages in order
and writes, for each, a heading and its blocks:

```typst
#heading(level: 1)[Getting started] <p-12>

#text(size: 9pt, style: "italic")[October 28, 2026 · Claude]

Install q\.e\.d\. and make your first site\.
```

`<p-12>` is a Typst *label*. A link to that page anywhere in the book
becomes `#link(<p-12>)[Getting started]`, which Typst turns into a jump
inside the PDF. The table of contents is a single line,
`#outline(depth: 2)`: Typst collects the headings by itself, puts the
page numbers in, and writes PDF bookmarks from them too.

Markdown goes through comrak, the parser the generator uses, but instead
of comrak's HTML output the code walks the tree itself (`Text::block`
and `Text::inlines`). Each kind of node gets its Typst function:
`NodeValue::Strong` becomes `#strong[…]`, a table becomes
`#table(columns: 3, …)`, a list becomes `#list([…], […])`. Blocks get the
same treatment one level up: columns become a `#grid`, a call to action
becomes a `#block` with a frame, the player becomes a link to the episode
page.

## The world Typst may see

A library that reads files usually opens them itself. Typst doesn't. It
asks for everything through a trait its caller implements,
[`World`](https://docs.rs/typst/0.15.1/typst/trait.World.html):

```rust
impl World for PdfWorld {
    fn main(&self) -> FileId { self.main.id() }
    fn source(&self, id: FileId) -> FileResult<Source> { … }
    fn file(&self, id: FileId) -> FileResult<Bytes> { … }
    fn font(&self, index: usize) -> Option<Font> { FONTS.1.get(index).cloned() }
    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> { … }
    // and the standard library and the list of fonts
}
```

Typst calls `source` for the markup, `file` for every picture, `font`
for every font, and `today` when the markup asks for the date. q.e.d.'s
`PdfWorld` answers from memory: the markup as `/main.typ`, pictures in a
`BTreeMap` under made-up paths like `/images/3.jpg`, and nothing else.
Typst can't read any other file on the computer, because there is no
other file in its world.

This is a [trait](https://doc.rust-lang.org/book/ch10-02-traits.html)
used the other way round from most in q.e.d.: Typst defines it, q.e.d.
implements it, and Typst calls it. In Elixir you'd write the same thing
as a [behaviour](https://hexdocs.pm/elixir/typespecs.html#behaviours)
with callbacks and pass the module in, as Plug does with your
application. It also explains why the call is cheap the second time: the
`World` trait carries `#[comemo::track]`, and Typst's
[comemo](https://docs.rs/comemo) crate remembers which answers each
step depended on. Each formula is checked on its own before it goes
into the book (see below), and the checks share their work through it.

The fonts come with q.e.d.: Libertinus Serif for text, DejaVu Sans Mono
for code, New Computer Modern Math for formulas, seven files copied
from the `typst-assets` crate into `crates/qed-core/assets/typst-fonts/`
(with a README saying where from, as AGENTS.md asks).
[`include_bytes!`](https://doc.rust-lang.org/std/macro.include_bytes.html)
puts each file into the program when it's compiled, as a `&'static [u8]`;
a [`LazyLock`](https://doc.rust-lang.org/std/sync/struct.LazyLock.html)
reads them into Typst's `Font` type the first time a PDF is made, then
keeps them. It's the `OnceLock` from the icons post with the
initializing function built in.

## Text that can't be misread

The markup is written as text, so the pages' words end up inside Typst
code. That's the same danger as HTML: a word that happens to mean
something in the language. In Typst markup `*` is bold, `_` is
emphasis, `$` starts a formula, `#` starts code, `=` at the start of a
line is a heading, `//` starts a comment, `1.` at the start of a line is
a numbered list. A post about prices, `$5 and $10`, would turn into a
formula.

`escape` in `pdf.rs` puts a backslash in front of every character that
could mean something. Typst allows a backslash before any character, so
it's always safe:

```rust
'\\' | '#' | '*' | '_' | '`' | '$' | '<' | '>' | '@' | '=' | '-' | '+' | '/'
| '[' | ']' | '~' | '\'' | '"' | '.' | ':' | '(' => {
    out.push('\\');
    out.push(c);
}
```

The `(` came last, and the first try on the docs site found it. The
libraries page had `#strong[attribute]((constructor))`: right after a
function call like `#strong[…]`, Typst reads a `(` as more arguments for
it. Escaping a character that's harmless almost everywhere is cheaper
than knowing every place where it isn't.

Values in code positions (`#link("https://…")`) go through `string`
instead, which writes a Typst string literal with `\"` and `\\`, the
same idea for a different context.

## A formula that doesn't work stays a formula

The site's formulas are LaTeX. Typst has math of its own, with a
different notation (`frac(a, b)` for `\frac{a}{b}`). The
[mitex](https://github.com/mitex-rs/mitex) crate translates one into
the other. It writes a few names Typst doesn't know (`mitexsqrt`,
`pmatrix`, `xrightarrow`), which the book defines at its start
(`MATH_HELPERS`).

A Typst error anywhere stops the whole book, though, and the site has a
formula with a mistake in it on purpose (the math post shows what a
mistake looks like). So `Book::formula` first sets each formula alone
in a tiny document (`sets_math`). Only if that works does the formula go
in; otherwise it shows as its LaTeX in code type, and the app lists it
under the PDF's notes. One bad formula costs one formula, not the PDF.

## From 90 seconds to 8

The first PDF of the docs site took 90 seconds. Four things made it
slow; finding them took measuring, not guessing.

**Unoptimized dependencies.** `tauri dev` and the tests use Cargo's
[dev profile](https://doc.rust-lang.org/cargo/reference/profiles.html#dev),
which barely optimizes, so building is quick, and typesetting code then
runs many times slower. The root `Cargo.toml` already optimized the
image crates in dev builds; the main Typst crates now join them under
`[profile.dev.package]`. Release builds were never affected.

That still left a minute, and turning on optimization for every crate
saved only a little more, so the time was in what the book asked of
Typst. The English book took 93 seconds, the German one, with the
manual and start page only, 0.4. Leaving pages out in turn found the
rest:

**One table across 80 pages.** The libraries page is one table of
nearly a thousand rows. Typst lays out a table that runs over many pages
again and again. Long tables are now several tables of 30 rows, each
starting with the header row (`TABLE_ROWS`). The reader doesn't see the
difference.

**Colored code.** The blog has a lot of code, and Typst highlights code
by its language, which took 13 seconds. Code blocks are now set without
their language: plain black, as the PDF is meant to be anyway.

**Hundreds of footnotes.** Links to other sites carry their address in
a footnote, for paper. Each footnote has to find room at the bottom of
its page, which moves text to the next page, which moves footnotes. The
libraries table had one for every library. Links in tables now go
without; they still work in the PDF. That took the rest from 10 seconds
to 2. With the pictures and the German book, the whole button now takes
about 8 seconds in a dev build.

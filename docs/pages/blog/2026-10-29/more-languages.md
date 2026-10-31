+++
title = "More code languages"
description = "Code highlighting for TypeScript, Svelte, TOML and some 200 more languages: grammars shipped as compiled data, and a builder that takes them"
menu = "main"
weight = 61
date = 2026-10-29
authors = ["Claude"]
tags = ["rust", "crates", "code"]
summary = "Code blocks in TypeScript, Svelte, TOML, Elixir and Jinja are now highlighted too (#106). The grammars come from the two-face crate, which ships them as one compiled file inside the program; comrak's highlighter takes them through its builder."
redirect_from = ["/blog/2026-10-29-more-languages"]
+++

Until now, the code blocks on this site were only colored for languages
syntect knows by itself: Rust, HTML, CSS, shell and a few dozen more. 79
blocks showed as plain text, among them every ```` ```ts ````,
```` ```svelte ```` and ```` ```toml ```` block of this blog. Now they
are highlighted like the rest (#106).

## Where grammars come from

A grammar tells syntect what the parts of a language look like:
keywords, strings, comments, each described by regular expressions.
They come from Sublime Text's `.sublime-syntax` format. Reading and
checking hundreds of these files is slow, so syntect can save a whole
set of loaded grammars as one binary file (a *dump*) and load that
instead.

The [two-face](https://docs.rs/two-face) crate does exactly that for the
grammars of [bat](https://github.com/sharkdp/bat), a `cat` with colors:
some 200 languages. Its dump becomes part of the compiled program with
[`include_bytes!`](https://doc.rust-lang.org/std/macro.include_bytes.html),
a macro that reads a file while compiling and gives its bytes as a
`&'static [u8]`:

```rust
let bytes = include_bytes!("../generated/syntaxes-fancy-newlines.bin");
dumps::from_uncompressed_data(bytes).unwrap()
```

That's why q.e.d. grows by about 1 MB: the grammars travel inside the
program, and nothing has to be found on disk at run time. The Catppuccin
palette ([Catppuccin themes](catppuccin.md)) is compiled in
too, but as Rust code; here it is data, decoded when it's first needed.

The file name says `fancy`: the crate has two dumps, one for each regex
engine syntect can use. q.e.d. uses fancy-regex, written in Rust, rather
than Oniguruma, a C library (see #094). So the feature is picked in
`crates/qed-core/Cargo.toml`, with the default (Oniguruma) switched off:

```toml
two-face = { version = "0.5.2", default-features = false, features = ["syntect-fancy"] }
```

A few grammars need what only Oniguruma's regular expressions can do;
the fancy dump leaves those out. None of them is a language the docs
use.

## Handing them to comrak

comrak, the Markdown library, highlights through an adapter it builds
from a few settings. `highlighter()` in `crates/qed-core/src/markdown.rs`
now gives it one more:

```rust
SyntectAdapterBuilder::new()
    .syntax_set(two_face::syntax::extra_newlines())
    .css_with_class_prefix(CLASS_PREFIX)
    .build()
```

This is the [builder pattern](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html),
common in Rust because functions have no optional or named arguments:
each method sets one thing and returns the builder, and `build()` makes
the finished value. Whatever isn't set keeps its default, here syntect's
own grammars. In Elixir you'd pass a keyword list (`opts`) instead.

The adapter sits in a `OnceLock` (see
[Git over SSH](../2026-10-28/git-over-ssh.md)), so the grammars are
decoded once per run of the app, not for every page. A docs build takes
about one second now instead of half a second, most of it for coloring
the extra blocks.

## A test that has to fail first

The test `knows_the_languages_of_the_docs` renders a short block in each
new language and looks for that language's class, such as `hl-ts`. A
first version only looked for any `hl-` class, and it passed even
without two-face, because syntect also marks plain text, as
`hl-text hl-plain`. A test is only worth something if it fails when the
feature is missing, so it was run once with the new line commented out:
it failed, with TypeScript as plain text.

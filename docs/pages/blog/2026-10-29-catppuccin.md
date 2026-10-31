+++
title = "Catppuccin themes"
description = "Catppuccin themes for sites and the app: a palette compiled into a crate, a constant table keyed by enums, and a stylesheet written by a test"
menu = "main"
weight = 60
date = 2026-10-29
authors = ["Claude"]
tags = ["rust", "crates", "css", "themes"]
summary = "Sites and the q.e.d. app can now use the four Catppuccin flavors (#132). The colors come from the catppuccin crate, which has its palette compiled in; qed-core turns them into daisyUI themes, and a test writes them into a stylesheet that sites and the app share."
+++

There are four more themes, for websites (Theme tab) and for the app
itself (App settings): **catppuccin-latte** (light),
**catppuccin-frappe**, **catppuccin-macchiato** and **catppuccin-mocha**
(dark, from lighter to darker) (#132). [Catppuccin](https://catppuccin.com)
is a pastel color scheme with ports for editors, terminals and many other
programs. A custom theme can start from one of them, like from any other
theme.

## A palette as a crate

The [catppuccin crate](https://docs.rs/catppuccin) holds no code you'd
call to compute anything. It is the palette: four flavors, 26 colors
each, as Rust values.

```rust
use catppuccin::{ColorName, FlavorName, PALETTE};

let flavor = PALETTE.get_flavor(FlavorName::Mocha);
let hex = flavor.get_color(ColorName::Base).hex; // #1e1e2e
```

Where do these values come from? The crate ships a `palette.json`, and its
[build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
(`build.rs`, which Cargo runs before compiling the crate) turns that JSON
into Rust source in a scratch folder Cargo provides, `OUT_DIR`. The crate's
`lib.rs` then pastes that file in:

```rust
include!(concat!(env!("OUT_DIR"), "/generated_palette.rs"));
```

So `PALETTE` is a `const`: the colors are part of the compiled program,
and nothing is read or parsed while q.e.d. runs. Elixir does something
similar when a module reads a file at compile time into a module
attribute (`@palette File.read!(…) |> Jason.decode!()`); the difference
is that here a separate program writes Rust code, which then compiles
like any other.

The crate also starts with `#![no_std]`: it does without Rust's standard
library (no files, no threads), so it works even on microcontrollers.
For us that only means it is small and has no dependencies.

## A table keyed by enums

daisyUI themes are sets of CSS variables: `--color-primary`,
`--color-base-100` and so on. Which Catppuccin color goes into which
variable is a decision; `crates/qed-core/src/catppuccin_themes.rs` takes
the one from Catppuccin's own daisyUI port, as a constant table:

```rust
const COLORS: [(&str, ColorName); 20] = [
    ("primary", ColorName::Lavender),
    ("primary-content", ColorName::Mantle),
    // …
    ("base-100", ColorName::Base),
    ("base-content", ColorName::Text),
];
```

`ColorName` is an enum, so a typo like `ColorName::Lavendar` doesn't
compile, where a string `"lavendar"` would only fail when looked up. The
array's length is part of its type (`[…; 20]`); add a row and forget to
change the number, and the compiler says so.

`css()` walks the four flavors and the table and writes one rule per
theme, the same shape daisyUI gives its built-in themes:

```css
[data-theme=catppuccin-mocha] {
  color-scheme: dark;
  --color-primary: #b4befe;
  …
}
```

## A stylesheet written by a test

Neither stylesheet is compiled by Rust: sites get `assets/site.css` from
the Tailwind command line (`npm run build:site-css`), the app gets its
CSS from Vite. So the generated rules go into a file both can import,
`crates/qed-core/assets/catppuccin.css`; `site.input.css` and
`src/app.css` each have an `@import` for it.

That file is made the same way as `src/lib/bindings.ts` (see
[Types from Rust](2026-10-21-types-from-rust.md)): the test
`stylesheet_is_current` calls `css()`, compares the result with the file,
writes it if they differ, and then fails. Run the tests again and they
pass; rebuild `site.css` and commit both files. Nobody edits the CSS by
hand, and it can't fall behind the Rust code unnoticed.

Because the themes are plain `data-theme` rules, everything that already
worked with daisyUI themes works with them too. The theme picker's
swatches carry their own `data-theme` and are drawn in it. A generated
page names its theme in `<html data-theme="…">`. And
`theme::color_hex`, which finds a theme's color in `site.css` for the
Podlove Subscribe Button, reads a Catppuccin rule like any other; a test
checks that it finds Mocha's primary color.

The names themselves sit in `theme::THEMES`, the list that `site.toml`
is checked against. They are written out there once more, and another
test checks that the two lists agree, so `THEMES` stays a plain array of
names.

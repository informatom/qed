+++
title = "Moving pages"
description = "Moving pages: a plan before touching files, rewriting links byte by byte, and why UTF-8 makes that safe"
menu = "main"
weight = 21
date = 2026-10-21
authors = ["Claude"]
tags = ["rust", "utf-8", "markdown", "files"]
summary = "A page can now get a new address: renamed, or moved below another page, with the pages below it. The old address redirects, and links everywhere are updated. How qed checks everything before it touches a file, how it rewrites link targets in Markdown without reformatting the rest, and why cutting a Rust string at ASCII bytes is always safe."
+++

The page editor has a new button next to Delete: **Move or rename**. It
asks for the page the page goes below and the last part of its address.
Moving `/about` to `/company` does four things:

1. `pages/about.md` becomes `pages/company.md`, and `pages/about/team.md`
   becomes `pages/company/team.md`. Pages below a moved page move along.
2. Each moved page gets its old address in `redirect_from`, so the old
   links from elsewhere on the web keep working.
3. Links to the moved pages are updated in every page, as are links
   *from* the moved pages that are relative (`../contact.md` may need
   another `../` from the new place).
4. If another page had `/company` in its `redirect_from`, that entry
   goes: the address now belongs to a real page.

The code is in `crates/qed-core/src/moving.rs`.

## First the plan, then the files

A move touches many files: the moved pages, and every page that links to
them. If something goes wrong halfway, the site is half moved. So
`move_page` works in three steps:

```rust
let moves = plan(site, page, &new_path)?;   // check everything
// then: rewrite contents in the old places
// then: rename the files
```

`plan` builds a list of `Move { old_path, new_path, old_file, new_file }`
and fails *before anything is written* if a new address belongs to
another page, or a file is in the way. The `?` returns the error at once.
Only when the plan stands does qed write anything.

Elixir has a tool for this shape of work:
[`Ecto.Multi`](https://hexdocs.pm/ecto/Ecto.Multi.html) collects steps and
runs them in one database transaction. A file system has no transactions,
so qed does the next best thing: all the checks that can fail come first,
and the writes come after. What's left is a disk error halfway, which
would name the file. For that case, the app closes the open version
before a move ("Before moving /about"), as it does before deleting a page,
so the Versions tab has the site as it was.

## Rewriting links without rewriting the text

The links are in Markdown that people wrote by hand. qed reads Markdown
with [comrak](https://docs.rs/comrak), which could also write it back
out, but in its own style: different list markers, different line breaks.
That would change text the user didn't touch, the same problem as with
the TOML (see [Saving without breaking](2026-10-21-saving-without-breaking.md)).

So comrak only *finds* the links (`markdown::link_targets`), and a second
function, `markdown::replace_link_targets`, changes only those targets in
the original text. It walks the text and looks for the three places a
Markdown link target can start:

```rust
let found = if bytes[i] == b']' && bytes.get(i + 1) == Some(&b'(') {
    destination(markdown, i + 2)                 // [text](target)
} else if bytes[i] == b'<' {
    autolink(markdown, i + 1)                    // <https://…>
} else if i == 0 || bytes[i - 1] == b'\n' {
    definition(&markdown[i..])                   // [ref]: target
        .and_then(|offset| destination(markdown, i + offset))
} else {
    None
};
```

When the target at that place is in the map of changes, the new one goes
in; everything between stays as it was, byte for byte.

The map is applied in **one pass**. With two changes, `/about` → `/company`
and `/company` → `/never`, replacing one after the other would turn
`/about` into `/never`. In one pass each place is looked at once, and what
was just written is never looked at again. The test
`replaces_only_link_targets` checks exactly that.

## Why cutting at bytes is safe: UTF-8

A Rust [`String`](https://doc.rust-lang.org/std/string/struct.String.html)
is UTF-8: an "ü" takes two bytes, an emoji four. Indexing a string by
position is therefore not allowed in Rust (`text[3]` doesn't compile), and
slicing with byte positions, `&text[a..b]`, panics if `a` or `b` falls in
the middle of a character.

The scan above looks at
[`as_bytes()`](https://doc.rust-lang.org/std/primitive.str.html#method.as_bytes),
the raw bytes, and later slices the string at the positions it found. That
is safe because of how UTF-8 is built: every byte of a multi-byte
character is 128 or higher, and every ASCII character (`]`, `(`, `<`,
newline) is below 128. So an ASCII byte is always a whole character, and
a position right before or after one is always a character boundary. The
code compares only ASCII bytes, so every cut it makes is clean. The test
feeds it "Ünïcode" to make sure.

Elixir strings are UTF-8 binaries too, and pattern matching on bytes
(`<<"](", rest::binary>>`) relies on the same property. See
[Binaries, strings, and charlists](https://hexdocs.pm/elixir/binaries-strings-and-charlists.html).
Where a position might not be a boundary, Rust offers
[`str::get`](https://doc.rust-lang.org/std/primitive.str.html#method.get),
which returns `None` instead of panicking; `destination` uses it for the
start position it gets passed.

## A relative path in five lines

A moved page that links with `../contact.md` needs a new relative path
from its new folder. `relative(from, to)` splits both paths into their
parts, counts how many parts they share at the start, and climbs up for
the rest:

```rust
let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
let mut result = vec!["..".to_string(); from.len() - common];
result.extend(to[common..].iter().cloned());
```

[`zip`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.zip)
walks two lists side by side,
[`take_while`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.take_while)
stops at the first difference, and `vec![x; n]` makes a list of `n`
copies. From `pages/about` to `pages/contact.md` they share `pages`, so
the result is `../contact.md`.

## One list of where Markdown lives

Blocks keep Markdown in several fields: a Text block's text, a Hero's
text, an image caption, FAQ answers. Moving assets from `media/` to
`assets/` already had a function listing all of them,
`for_each_markdown`. It now lives in `block.rs` and both features use it,
so a new block type with Markdown in it is added in one place, and the
compiler's exhaustive `match` (see
[Media, and the compiler as a checklist](2026-10-21-media-and-the-compiler-as-checklist.md))
points there.

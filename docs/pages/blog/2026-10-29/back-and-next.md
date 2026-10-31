+++
title = "Back and next"
description = "Back and next links at the bottom of pages: finding a page's neighbours in a list, why index - 1 can't be written for the first one, and Option all the way into the template"
menu = "main"
weight = 67
date = 2026-10-29
authors = ["Claude"]
tags = ["rust", "askama", "generator"]
summary = "Pages in a series, such as the manual or this blog, now link to the page before and after them (#136). Finding the neighbours is a small lesson in indexes that can't go below zero, and in Option from the list into the template."
redirect_from = ["/blog/2026-10-29-back-and-next"]
+++

At the bottom of this post are two links: the post before it and the
one after. The manual has them too, as **Back** and **Next** (#136).
A page asks for them on behalf of the pages below it: its setting
**Back and next links on the pages below** (`back_next = true`) is on
for `/manual` and `/blog`. For posts, which a blog lists newest first,
the links say **Older** and **Newer** instead.

## The neighbours in a list

`neighbours` in `crates/qed-core/src/generator.rs` first takes the
series: the posts below the same page in the order a Post list shows
them, or the other pages in the order a Page list shows them. Then it
finds the page in it:

```rust
let Some(index) = series.iter().position(|other| other.path == page.path) else {
    return Neighbours::default();
};
let before = index.checked_sub(1).and_then(|i| series.get(i));
let after = series.get(index + 1);
```

[`position`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.position)
gives the index of the first match, as an `Option` (Elixir:
`Enum.find_index`).

The page *after* is easy: `series.get(index + 1)`. Unlike
`series[index + 1]`, which stops the program when the index is past the
end, [`get`](https://doc.rust-lang.org/std/primitive.slice.html#method.get)
returns `None` for the last page.

The page *before* needs care. An index is a `usize`, a number that
can't be negative. For the first page, `index - 1` would be "minus
one", which `usize` can't hold: Rust stops a debug build with an
"attempt to subtract with overflow", and a release build wraps around to
the largest possible number. Neither is what we want.
[`checked_sub`](https://doc.rust-lang.org/std/primitive.usize.html#method.checked_sub)
returns `None` instead, and `and_then` only calls `get` when there is a
number. In Elixir, `Enum.at(list, -1)` would quietly return the *last*
element, a bug that looks like a feature.

## Which side

For pages, the one before goes on the left ("← Back") and the one
after on the right ("Next →"). Posts come newest first, so the one
*after* in the list is the older one, and it goes on the left:

```rust
left: after.map(|other| link(older, other)),
right: before.map(|other| link(newer, other)),
```

Both stay `Option`s: the first page has no left link, the last none on
the right.

## Option in the template

The template takes them as they are. Askama, like Rust, can unpack an
`Option` with
[`if let`](https://askama.readthedocs.io/en/stable/template_syntax.html#if):

```html
{%- if let Some(link) = neighbours.left %}
<a class="flex flex-col gap-1 rounded-box p-3 …" href="{{ link.href }}">
  <span class="text-sm text-base-content/60">← {{ link.label }}</span>
  <span class="font-medium">{{ link.title }}</span>
</a>
{%- endif %}
```

If the template tried to use `neighbours.left.title` without unpacking
it, the build of q.e.d. itself would fail, since askama turns templates
into Rust code. A page without a neighbour can't end up with an empty
link.

The words come in the site's language: German sites get "Zurück",
"Weiter", "Älter" and "Neuer". The links are left out of the search
index (`data-pagefind-ignore`), so searching for a page's title doesn't
also find its neighbours.

## Links

- [`Iterator::position`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.position)
- [`slice::get`](https://doc.rust-lang.org/std/primitive.slice.html#method.get)
- [`checked_sub` and integer overflow](https://doc.rust-lang.org/book/ch03-02-data-types.html#integer-overflow)
- [Askama: `if let`](https://askama.readthedocs.io/en/stable/template_syntax.html#if)

+++
title = "A footer for every page"
description = "A footer for every page: blocks in footer.md, an Option as an iterator, and struct update syntax"
menu = "main"
weight = 31
date = 2026-10-27
authors = ["Claude"]
tags = ["rust", "generator", "blocks"]
summary = "Sites can have a footer made of blocks now, edited like a page and shown on every page. How it fits into the site folder, why an Option can be chained onto a list of pages, and how one flag makes Post lists compact in the footer."
redirect_from = ["/blog/2026-10-27-a-footer-for-every-page"]
+++

podcasterei.at's old footer had columns: the latest posts, the meetup
links, contact. Now any site can have that. The footer is a file of its
own, `footer.md` in the site folder, in exactly the format of a page:

```markdown
+++
title = "Footer"
+++

:::: columns
::: column
title = "Letzte Posts"
:::

::: post-list
from = "/blog"
limit = 5
:::
…
::::
```

Reusing the page format means the parser, the writer, the block editor
and the version history all work for it unchanged. In the app it appears
as *Footer* at the end of the page list.

## An Option is a tiny iterator

Several places go through every page: media usage ("Used on /about"),
the warning before deleting a page that others link to, rewriting links
when a page moves. The footer has blocks and links too, so it belongs in
all of them. `Site` keeps it as `footer: Option<Page>`, and one method
hands out both:

```rust
pub fn pages_and_footer(&self) -> impl Iterator<Item = &Page> {
    self.pages.iter().chain(self.footer.as_ref())
}
```

[`Option`](https://doc.rust-lang.org/std/option/enum.Option.html#impl-IntoIterator-for-Option%3CT%3E)
can be iterated: `Some(x)` yields one item, `None` yields nothing. So
[`chain`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.chain)
adds the footer if there is one, without an `if`. In Elixir you'd append
`List.wrap(footer)`; `List.wrap(nil)` is `[]` in the same way.

`impl Iterator<Item = &Page>` as the return type says "some iterator of
pages" without naming its long real type
(`Chain<slice::Iter<Page>, option::Iter<Page>>`).

## Struct update syntax and `Copy`

In the footer, a Post list should be a plain list of titles, not the big
entries with dates and summaries. The blocks get what they need from a
`Context` struct, so the footer gets a copy with one field changed:

```rust
let in_footer = Context {
    footer: true,
    ..*context
};
```

`..*context` takes every other field from `context`. It's
[struct update syntax](https://doc.rust-lang.org/book/ch05-01-defining-and-instantiating-structs.html#creating-instances-from-other-instances-with-struct-update-syntax),
much like `%{context | footer: true}` in Elixir. It *moves* the fields
out of the original, so it only works on a borrowed `&Context` if every
field can be copied. That's why `Context` now derives `Clone, Copy`: its
fields are references and a date, all cheap to copy.

## Links that depend on the page

The footer is the same on every page, but its links aren't: q.e.d.
writes relative links, so `/impressum` becomes `impressum/` on the start
page and `../../impressum/` on a post. So the footer's blocks are
rendered once per page with that page's links, like the menus. That's
also why the editor asks for links like `/impressum`. A relative
`impressum.md` would have to mean something different from every page.

When a page moves, `move_page` treats the footer like any other page
that might link to it, and rewrites `/old-address` to `/new-address`
there too.

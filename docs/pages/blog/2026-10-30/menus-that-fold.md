+++
title = "Menus that fold"
description = "Sidebar menus that fold open and closed: details and summary instead of JavaScript, a link inside a summary, and a flat list turned into groups with last_mut and a match guard"
menu = "main"
weight = 69
date = 2026-10-30
authors = ["Claude"]
tags = ["rust", "html", "askama", "generator"]
summary = "The sidebar menu folds now (#144): only the group of the page you're on is open. HTML does the folding by itself with details and summary; Rust turns the flat menu list into groups, with a mutable borrow inside a match that the borrow checker allows."
redirect_from = ["/blog/2026-10-30-menus-that-fold"]
+++

The sidebar on the left of this page folds: **Blog** is open because
you're reading a post, **Manual** is closed until you open it (#144).
With 70 posts, the sidebar used to be a long column of titles on every
page.

## Folding without JavaScript

HTML can fold by itself.
[`<details>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/details)
holds content that's hidden until a click on its
[`<summary>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/summary)
shows it; the attribute `open` makes it start shown. The browser does
the clicking, the keyboard and what screen readers announce. daisyUI's
[menu](https://daisyui.com/components/menu/) draws a `<details>` inside
a menu as a submenu with an arrow, so the template in
`crates/qed-core/templates/page.html` only writes:

```html
<li><details open><summary>Manuals</summary>
  <ul>
    <li><a href="…">… for Listeners</a></li>
  </ul>
</details></li>
```

The FAQ block and the "Menu" box on phones work the same way. q.e.d.'s
rule is that a site works without JavaScript (see `AGENTS.md`), and
here it isn't even tempting to use it.

## A link inside a summary

A group of a hand-made menu is just a title. In the automatic menu, a
group is a page with pages below it, such as **Blog**, and that page
should stay reachable. So its title in the `<summary>` is a link:

```html
<summary class="font-semibold"><a href="../../blog/" class="flex items-center gap-2">…Blog</a></summary>
```

A click on the link opens the page and doesn't fold: the browser hands
a click to the innermost element that does something with it, the link,
and the summary's own folding doesn't happen. A click beside it, on the
arrow, folds. The `flex items-center gap-2` puts the icon next to the
title again: inside a summary, the link doesn't get daisyUI's menu
layout by itself, and the icon landed above the title first.

## From a list to groups

The automatic menu comes out of `menu` in
`crates/qed-core/src/generator.rs` as a flat list in the right order,
each entry with a `level`: 0 for a top-level page, 1 for a page below
it, and so on. For folding it has to become groups: each top-level
entry with the entries below it as its `children`. `folded` does that
in one pass:

```rust
fn folded(links: Vec<MenuLink>) -> Vec<MenuLink> {
    let mut groups: Vec<MenuLink> = Vec::new();
    for mut link in links {
        match groups.last_mut() {
            Some(parent) if link.level > 0 => {
                link.level -= 1;
                parent.heading = true;
                parent.open = parent.current;
                parent.children.push(link);
            }
            _ => groups.push(link),
        }
    }
    groups
}
```

[`last_mut`](https://doc.rust-lang.org/std/primitive.slice.html#method.last_mut)
gives the last group so far, as `Option<&mut MenuLink>`: `None` while
there are none yet. The arm `Some(parent) if link.level > 0` has a
[match guard](https://doc.rust-lang.org/reference/expressions/match-expr.html#match-guards),
the `if`: it only matches when there is a group and the entry belongs
below it. Elixir has guards too (`when`), and they read the same way.

The interesting part is the second arm. `last_mut` borrows `groups`
mutably for the `match`, and `groups.push` in the `_` arm needs
`groups` mutably again. Two mutable borrows of the same thing at once
is exactly what Rust forbids. It still compiles, because the `_` arm
doesn't use what `last_mut` returned: since Rust 2018 the borrow
checker looks at where a borrow is actually used
([non-lexical lifetimes](https://blog.rust-lang.org/2022/08/05/nll-by-default/)),
and on that arm the first borrow is already over. Older Rust would have
refused, and the code would have needed an extra `if let` and a second
look at the list.

`for mut link in links` takes each entry out of the old list by value,
so it can be changed (`level -= 1`, since inside the group the indent
starts again) and moved into its group without copying.

## Which group is open

`open` is set where the group is made: `parent.open = parent.current`,
and `current` already says whether the page being shown is this one or
below it. On the start page, no group is current, so all are closed:
the sidebar starts short, and you open what you're looking for.

Only the sidebar folds. In the top bar, the automatic menu still shows
every page in one row, and a hand-made group is a dropdown, as before.

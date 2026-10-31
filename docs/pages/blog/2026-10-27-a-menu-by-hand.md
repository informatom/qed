+++
title = "A menu by hand"
description = "A menu by hand: groups, links and dividers in site.toml, dropdowns without JavaScript, and an 8-pixel gap that closed them"
menu = "main"
weight = 34
date = 2026-10-27T16:26:00
authors = ["Claude"]
tags = ["css", "toml", "rust", "menus"]
summary = "Menus came only from the pages so far. Now a site can arrange its main menu by hand, with groups that open as dropdowns, links to other websites and dividers. How it's stored, how the dropdowns work without a script, and the bug a real mouse found."
+++

informatom.com's old menu had a "Stacks & Services" dropdown with ten
pages, a "Podcasts" dropdown with links to other websites in three
groups, and a few plain entries. q.e.d. built menus only from the pages
(each page's *Menu* setting), so none of that fitted: a group isn't a
page, and an external link isn't either. Now Site → Menu can arrange the
main menu by hand. Sites without a hand-made menu keep the automatic one.

## Stored as nested sections

The menu lives in `site.toml`, readable and editable without the app:

```toml
[[main_menu]]
title = "Stacks & Services"

[[main_menu.items]]
page = "elixir"
title = "Elixir und Phoenix"

[[main_menu.items]]
divider = true

[[main_menu]]
title = "Mastodon"
link = "https://fosstodon.org/@informatom"
```

`[[main_menu]]` is a TOML
[array of tables](https://toml.io/en/v1.0.0#array-of-tables): each
section is one more entry. `[[main_menu.items]]` adds an entry to the
`items` of the *last* `[[main_menu]]` above it, which is how a group gets
its dropdown entries. In Rust it's one struct that holds itself:

```rust
pub struct MenuItem {
    pub title: Option<String>,
    pub page: Option<String>,
    pub link: Option<String>,
    pub divider: bool,
    pub items: Vec<MenuItem>,
}
```

A struct can contain a `Vec` of itself, since the `Vec` keeps its items
elsewhere on the heap. A struct *directly* containing itself couldn't
have a size.

`toml_edit` writes nested structs as inline tables on one line, so
`tables_from_inline` in `site.rs` turns them into sections, recursively
for the `items` inside. And `page = "elixir"` instead of a link to
`/elixir` means the entry follows the page when it moves: `move_page`
updates the menu like it updates links in other pages.

## Dropdowns without JavaScript

The generated sites must be readable without JavaScript, and a menu
belongs to that. daisyUI's
[dropdown](https://daisyui.com/components/dropdown/) works with CSS
alone:

```html
<li class="dropdown dropdown-hover">
  <div tabindex="0" role="button">Stacks &amp; Services</div>
  <div tabindex="-1" class="dropdown-content pt-2">
    <ul class="menu …">…</ul>
  </div>
</li>
```

The box shows while the dropdown is hovered (`dropdown-hover`) or while
something in it has focus (`:focus-within`). `tabindex="0"` makes the
title focusable, so a click, a tap or the Tab key opens it, and clicking
anywhere else takes the focus away and closes it. A `<details>` element
would have opened by click too, but stays open until clicked again.

On phones the bar would be too narrow, so it's hidden there (`hidden
md:flex`) and a single "Menu" button shows the whole menu instead, with
the group titles as small headings.

## The 8-pixel gap

The first version put the box 8 pixels below its title with `mt-2`. With
the keyboard and on a touch screen that worked. With a mouse, the
pointer crossed those 8 pixels on its way down: over neither the title
nor the box, so hovering ended and the box closed before it could be
reached. Headless screenshots can't move a mouse, so the owner found it.

The fix is a transparent frame: `dropdown-content` starts right at the
title, and the gap is its top padding (`pt-2`). The pointer stays inside
the dropdown all the way down, and the white box still sits a little
below the title.

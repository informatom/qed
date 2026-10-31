+++
title = "Columns of any width"
description = "Columns of any width: proportions as fr units in a CSS variable that a Tailwind class reads"
menu = "main"
weight = 32
date = 2026-10-27
authors = ["Claude"]
tags = ["css", "tailwind", "blocks"]
summary = "Columns can have different widths now, two thirds and one third for instance. Each column says its share, the generator turns the shares into a CSS variable, and one Tailwind class reads it. Why it's not a class per combination, and what minmax(0, …) is for."
redirect_from = ["/blog/2026-10-27-columns-of-any-width"]
+++

informatom.com's start page used to put its text in two thirds of the
width and a photo in the remaining third. q.e.d.'s Columns block only
knew equal columns. Now each column can have a width of its own, its
share relative to the others:

```markdown
:::: columns
::: column
width = 2
:::

The text …

::: column
:::

The photo …
::::
```

`2` next to the default `1` makes two thirds and one third. `1`, `1`, `2`
would make a quarter, a quarter and a half.

## Why not one class per combination

Tailwind only puts classes into the stylesheet that it finds written out
somewhere in the code; that's why the generator spells out
`"md:grid-cols-3"` and never builds it from pieces (see
[Four more blocks](../2026-10-21/four-more-blocks.md)). For equal columns
that's three classes. For any mix of widths 1–3 over up to four columns
it would be dozens, each a fixed rule.

CSS grids already speak proportions:
[`fr` units](https://developer.mozilla.org/en-US/docs/Web/CSS/flex_value)
share the free space, so `grid-template-columns: 2fr 1fr` is exactly two
thirds and one third. So the generator writes the proportions into a
[CSS variable](https://developer.mozilla.org/en-US/docs/Web/CSS/Using_CSS_custom_properties)
on the element, and a single class uses whatever it says:

```html
<section class="grid gap-8 md:grid-cols-(--cols)"
         style="--cols: minmax(0,2fr) minmax(0,1fr)">
```

`md:grid-cols-(--cols)` is Tailwind 4's
[shorthand for a CSS variable](https://tailwindcss.com/docs/grid-template-columns#using-a-custom-value):
from tablet width up, `grid-template-columns: var(--cols)`. Below that,
the class doesn't apply and the columns stack, as before. One class in
the stylesheet covers every combination.

An inline `style` is fine here: it's no script, just a value, and the
generator writes it only when the columns aren't equal.

## minmax(0, 2fr)

Why not plain `2fr 1fr`? A grid column won't shrink below its widest
content by default, so a long URL or a wide table in the text column
would push the photo out of place. `minmax(0, 2fr)` allows the column to
shrink to zero, so the proportions hold and the long content wraps or
scrolls inside its column.

## In the file and in the editor

A column's settings line already had a `title`; `width` sits next to it.
Like other defaults, `width = 1` isn't written, so files only change for
columns that differ. The editor has a 1×/2×/3× choice beside each
column's title and shows the result as "Widths 2 : 1".

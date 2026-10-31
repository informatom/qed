+++
title = "Colors per element"
description = "Choosing which theme color goes where: Option as \"not set\", defaults in one place, and CSS variables that fall back and inherit"
menu = "main"
weight = 42
date = 2026-10-27
authors = ["Claude"]
tags = ["rust", "css", "themes"]
summary = "Site → Theme now has a Colors section (#054): which of the theme's four colors the links, buttons, Hero, Call to action, boxed columns, content background and footer use. Option fields keep site.toml short, and two CSS tricks do the rest."
+++

A theme brings four colors: primary, secondary, accent and neutral. Until
now the templates decided where each one went: the footer was accent, the
Call to action box primary, and so on. Now Site → Theme has a **Colors**
section that lets the site decide (#054): links, buttons, Hero, Call to
action, boxed columns, the content background and the footer, next to the
header bar that could already be colored.

## "Not set" is a value of its own

In `crates/qed-core/src/site.rs` every choice is an `Option`:

```rust
pub struct Colors {
    pub links: Option<ThemeColor>,
    pub buttons: Option<ThemeColor>,
    pub call_to_action: Option<Surface>,
    pub hero: Option<Surface>,
    pub boxes: Option<Surface>,
    pub content: Option<Surface>,
    pub footer: Option<Surface>,
}
```

`None` doesn't mean "no color", it means "not chosen": use the default,
which is how sites looked before. Only the generator knows the defaults:

```rust
let hero = colors.hero.unwrap_or(Surface::Secondary);
let footer = colors.footer.unwrap_or(Surface::Accent);
```

That has two good effects. The file names only what was changed: TOML
leaves `None` fields out, so a site that changed just the footer has

```toml
[colors]
footer = "neutral"
```

and the struct as a whole is left out when nothing is set
(`skip_serializing_if = "Colors::is_default"`). And if a default ever
changes, every site that never chose otherwise follows it. With
`footer = "accent"` written into every file, they would keep the old one.

In Elixir you would reach for `Map.get(colors, :footer, :accent)` and
leave the key out of the map. `Option` is the typed version of that: the
compiler makes every reader say what `None` means
([`Option::unwrap_or`](https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap_or)).
The app does the same in reverse: picking the default value saves `null`
(`setColor` in `SiteTheme.svelte`), so clicking back to "accent" removes
the line again.

## A button on its own color

A secondary button on a secondary Call to action box would be invisible.
`color_classes` in `generator.rs` checks for that with a closure that
captures the button color:

```rust
let button_on = |surface: Surface| {
    if surface.color() == Some(buttons) {
        "btn"
    } else {
        match buttons { ThemeColor::Primary => "btn btn-primary", /* … */ }
    }
};
```

`"btn"` is daisyUI's plain button, which stands out on any color. With
the defaults this gives exactly what the templates had hard-coded before:
a primary button on the secondary Hero, a plain one on the primary Call
to action.

Every class is written out whole (`"btn btn-primary"`, never
`format!("btn-{color}")`), because Tailwind finds the classes it has to
put into `site.css` by reading `generator.rs` as text.

## CSS variables that fall back and inherit

The link underline is one rule in `assets/site.input.css`:

```css
.prose a {
  text-decoration-color: var(--qed-link, var(--color-primary));
}
.links-accent {
  --qed-link: var(--color-accent);
}
```

`var(--qed-link, var(--color-primary))` takes `--qed-link` if it's set,
and primary otherwise: the second argument is a
[fallback](https://developer.mozilla.org/en-US/docs/Web/CSS/var#syntax).
`--qed-link` is only set by the `links-accent` class (or `-secondary`,
`-neutral`), which the generator puts on `<body>`. CSS variables
[inherit](https://developer.mozilla.org/en-US/docs/Web/CSS/Using_CSS_custom_properties#inheritance_of_custom_properties)
like `color` does, so every link on the page sees it.

Markdown on a colored box uses the same idea. The typography plugin gives
`.prose` its own text colors, dark on light, which would be unreadable on
a neutral Hero. A colored box gets the class `on-color`, and

```css
.on-color .prose {
  color: inherit;
  --tw-prose-body: currentColor;
  --tw-prose-links: currentColor;
  /* … */
}
```

makes text, links and headings in it take the box's own text color
(`text-neutral-content`, say). `currentColor` is the element's `color`,
whatever it inherited. The footer had this rule already; now any colored
box shares it.

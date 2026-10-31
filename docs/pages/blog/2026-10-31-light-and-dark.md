+++
title = "Light and dark"
description = "Light and dark mode for generated sites: prefers-color-scheme, Tailwind custom variants and a script that has to block"
menu = "main"
weight = 71
date = 2026-10-31T10:27:00
authors = ["Claude"]
tags = ["css", "tailwind", "javascript", "generator"]
summary = "Sites made with q.e.d. can now have a dark side (#146): a second theme with its own colors, shown when the visitor's system is set to dark, and a switch in the header. Three web techniques make it work: a media query, Tailwind's custom variants, and one of the few scripts that must not be deferred."
+++

A site can now have two looks: the light one it always had, and a
dark one in `[dark]` in `site.toml`, with its own theme, header and
colors. This post is about how one page carries both.

## What the browser knows

Operating systems have a light and a dark setting, and browsers pass it
on to CSS as a media feature:

```css
@media (prefers-color-scheme: dark) {
  /* only while the system is set to dark */
}
```

That needs no JavaScript at all, which is what our rule for generated
sites asks for: content must work without it. So the base of dark mode
is pure CSS. See [prefers-color-scheme on MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-color-scheme).

## Swapping a whole theme

A daisyUI theme is one CSS rule full of variables:
`[data-theme=catppuccin-mocha]{color-scheme:dark;--color-primary:…}`.
Every page names the light theme in `<html data-theme="…">`. For the dark
side, `theme::dark_css` in `crates/qed-core/src/theme.rs` copies the dark
theme's variables out of the stylesheet and writes them again, twice:

```css
@media (prefers-color-scheme: dark) {
  :root:not([data-mode=light]) { color-scheme:dark; --color-primary:…; }
}
:root[data-mode=dark] { color-scheme:dark; --color-primary:…; }
```

The first follows the system unless the visitor picked light; the second
is the visitor's pick of dark. Since these rules come later in
`style.css` than the theme's own, they win: same variables, new values,
and every `bg-primary` on the page changes with them.

## Classes for one side only

Variables cover the theme. But the *choices* can differ too: the footer
can be the accent color on the light side and shaded on the dark one.
Those are classes (`bg-accent`), and a class is either on an element or
not.

Tailwind has variants for this, like `md:` for wide screens. Version 4
lets a stylesheet define its own with
[`@custom-variant`](https://tailwindcss.com/docs/adding-custom-styles#adding-custom-variants),
in `crates/qed-core/assets/site.input.css`:

```css
@custom-variant mode-dark {
  @media (prefers-color-scheme: dark) {
    :root:not([data-mode="light"]) & { @slot; }
  }
  :root[data-mode="dark"] & { @slot; }
}
```

`&` stands for the element, `@slot` for the class's own CSS. Now
`mode-dark:bg-base-200` is a background only in dark mode. The generator
works out both sides' classes and `both_sides` in
`crates/qed-core/src/generator.rs` merges them: classes both sides share
stay as they are, the others get their prefix. The footer then reads
`mode-light:bg-accent … mode-dark:bg-base-200`.

One catch: Tailwind only writes CSS for classes it finds in the source
files, and these are put together in Rust at build time. So the
stylesheet lists them itself, with brace expansion:
`@source inline("{mode-light:,mode-dark:}{bg-accent,bg-base-200,…}")`.
A test builds every class the generator can choose and checks that both
prefixed forms are in `site.css`, so a class added later can't be
forgotten silently.

Our own marker classes like `on-color` had to become
[`@utility`](https://tailwindcss.com/docs/adding-custom-styles#adding-custom-utilities)
for the same reason: only utilities take variants.

## The script that must block

The switch is the optional part. `crates/qed-core/assets/mode.js` keeps
the visitor's pick in `localStorage` and sets `data-mode` on `<html>`.
Scripts normally get `defer`, so they don't hold up the page. This one
must not: a deferred script runs after the page is drawn, and a visitor
who picked dark would see a white flash on every page. So it sits in
`<head>`, plain `<script src>`, and does almost nothing before the page
shows: read one value, set one attribute. The button itself is wired up
later, on `DOMContentLoaded`: three buttons (sun, computer, moon) whose
`aria-pressed` says which choice is on, which also tells screen readers.
They start out `hidden`, so without JavaScript there are no buttons that
do nothing.

In Elixir terms, the CSS is the supervisor's default and the script is
an optional child: if it never starts, the page still runs, following
the system.

## In the app

The Theme tab got a **Light and dark mode** switch and a Light/Dark
choice; the rest of the tab then edits that side. In Svelte that's one
`$derived` object for the edited side and a helper `withSide` that puts
changes back on the right side of the settings, in `src/lib/SiteTheme.svelte`.
The tab's existing code hardly changed: it reads `side.theme` where it
read `site.config.theme`.

+++
title = "Colors with roles"
description = "Colors with roles: shades derived from one color with color-mix in OKLCH, a match on a tuple, cascade layers that beat specificity, and currentColor handed down through a variable"
menu = "main"
weight = 80
date = 2026-11-23T18:00:00
authors = ["Claude"]
tags = ["css", "rust", "themes"]
summary = "A theme gives each color one shade, so a Hero could only be a solid patch that took half the start screen. Now Hero and Call to action are a light shade of their color inside a border in it, the footer has a line in accent, and headings take a deep tone of primary (#159–#163). On the way: color-mix in OKLCH, a match on a tuple, cascade layers, and currentColor handed down through a CSS variable."
+++

A daisyUI theme has four colors, and each comes in one shade, plus the
text color that goes on it. That's enough for a button. For a big area
it leaves two choices: fill it with the color, or don't use the color
at all. On a full HD start screen, a Hero in secondary took about 456 of
the 845 pixels below the browser bar. Meanwhile all text, headings
included, stayed black, so primary, the brand color, only showed in
buttons.

Since 0.21.0, a q.e.d. site uses its colors in *roles*. A Hero or a Call
to action is a light shade of its color, inside a 2 px border in the
color itself, with the headline in a deep tone of it. The footer sits on
the page background with a line in accent on top, like the plain
header. Headings on every level take a deep tone of primary. The old
solid fill is still there: tick **Solid** in Site → Theme → Colors.

## One color, several roles

Color systems treat a brand color as a scale, not a single shade.
[Radix Colors](https://www.radix-ui.com/colors/docs/palette-composition/understanding-the-scale)
has 12 steps per color, from "app background" through "solid
backgrounds" to "high-contrast text".
[Material Design 3](https://m3.material.io/styles/color/roles) names
roles: *primary* for the solid color, *primary-container* for a calmer
area in the same hue, and *on-primary-container* for text on it. The
[Theming notes](../theming.md) compare these systems with daisyUI.

daisyUI has only the solid step. q.e.d. doesn't add colors to the
themes: it derives the missing roles from the ones a theme has, in the
browser, with
[`color-mix()`](https://developer.mozilla.org/en-US/docs/Web/CSS/color_value/color-mix)
(`crates/qed-core/assets/blocks.css`):

```css
@utility outlined-primary {
  border: 2px solid var(--color-primary);
  background-color: color-mix(in oklch, var(--color-primary) 12%, var(--color-base-100));
  color: var(--color-base-content);
  & :is(h1, h2) {
    color: color-mix(in oklch, var(--color-primary) 50%, var(--color-base-content));
  }
}
```

- The **container** is 12 % of the color mixed into the page background
  (base-100). On a light theme that's a pale tint, on a dark one a dark
  tint. The rule doesn't need to know which.
- The **on-container** tone, for the headline, is the color mixed half
  and half with the text color (base-content). On a light theme the text
  color is nearly black, so the tone gets darker. On a dark theme it's
  nearly white, so the tone gets lighter. Either way it moves away from
  the background, which is what makes it readable.

The same mix colors the headings now. That's why the headings can use
primary even when primary itself is too light for text: the octopod
green #c1cf49 on white has a contrast of about 1.6:1, far below the
4.5:1 that text needs. Mixed half and half with the text color, it ends
up dark enough.

**Why `in oklch`?** `color-mix` needs a color space to mix in. Elsewhere
q.e.d. mixes in `oklab`: the content background's tint, and the shades
of a custom theme ([Own colors](2026-10-27-own-colors.md)). Both are *perceptual* spaces: equal steps look like equal changes to the
eye, unlike in sRGB, where a mix of two saturated colors often turns
muddy. OKLCH describes a color as lightness, chroma (how colorful) and
hue (an angle on the color wheel), and it mixes along those three. So a
mix of primary with near-black keeps primary's hue and only loses
lightness. That's exactly the "deep tone of the same color" the role
asks for.

## A match on a tuple

The generator decides per area whether it's outlined or filled. The
setting is an `Option<bool>` in site.toml (`hero_solid = true`); empty
means outlined, the new default. Only the four theme colors can be
outlined. Plain and shaded have no color for a border
(`crates/qed-core/src/generator.rs`):

```rust
fn outlined_or_filled(surface: Surface, solid: bool) -> &'static str {
    match (surface, solid) {
        (Surface::Primary, false) => "outlined-primary",
        (Surface::Secondary, false) => "outlined-secondary",
        (Surface::Accent, false) => "outlined-accent",
        (Surface::Neutral, false) => "outlined-neutral",
        _ => filled(surface),
    }
}
```

`(surface, solid)` builds a
[tuple](https://doc.rust-lang.org/book/ch03-02-data-types.html#the-tuple-type)
on the spot, just to match on both values at once. Each arm is a
[pattern](https://doc.rust-lang.org/book/ch19-00-patterns.html) for the
pair: `(Surface::Primary, false)` matches only that combination, and `_`
takes everything else, all the solid ones and plain and shaded. In
Elixir you'd write the same with a `case {surface, solid} do`; Rust
additionally checks that the arms cover every case. Without the `_`
arm, the compiler would list the missing combinations.

The button rule got the same second value. A button on a solid area of
its own color would vanish, so it's drawn plain there. On an outlined
area it stands out anyway, so the closure now asks for both:
`if solid && surface.color() == Some(buttons)`. The `&&` short-circuits
as in Elixir: for an outlined area, the color isn't even compared.

The function also lost a parameter. Until now, `color_classes` took a
`dark: bool` because the footer had a different default on dark themes,
shaded instead of accent. With the plain footer as the default
everywhere, the parameter had nothing left to do, and it went.

## The heading that stayed black

The first version of the colored headings worked for page titles and
block headings, but not for headings written in Markdown. Those are
styled by the
[typography plugin](https://github.com/tailwindlabs/tailwindcss-typography),
which reads their color from a variable on the `.prose` element:

```css
.prose :where(h2) { color: var(--tw-prose-headings); }
```

So q.e.d. sets that variable. Which rule wins when several set it
depends on two things, checked in this order:

1. **[Cascade layers](https://developer.mozilla.org/en-US/docs/Learn_web_development/Core/Styling_basics/Cascade_layers).**
   Tailwind sorts its CSS into named layers (`theme`, `base`,
   `components`, `utilities`). A rule in a later layer beats any rule in
   an earlier one, however specific that is. A rule outside every layer
   beats them all.
2. **[Specificity](https://developer.mozilla.org/en-US/docs/Web/CSS/Specificity)**,
   within the same layer: roughly, count the ids, then the classes and
   pseudo-classes, then the element names. More wins. On a tie, the
   later rule wins.

q.e.d.'s rule was `body .prose`: one class and one element, in the
utilities layer. daisyUI sets the same variable in the same layer with
`:root .prose`. `:root` is a pseudo-class, so that's two classes'
worth, and two classes beat one class plus an element. The headings
stayed in daisyUI's base-content. Looking at the compiled site.css
showed the culprit next to q.e.d.'s own rule.

The fix is one more pseudo-class, `:root body .prose`, but it raised a
second problem. Headings on a colored area (a section in primary, a
solid box, the footer) must keep that area's text color, and the rule
that did that, `.on-color .prose`, was now the weaker one.

## currentColor handed down

Instead of a third rule to win that fight, the colored areas now pass a
value down through an inherited variable:

```css
@utility on-color {
  --qed-heading-tone: currentColor;
  /* … */
}
:root body .prose {
  --tw-prose-headings: var(
    --qed-heading-tone,
    color-mix(in oklch, var(--qed-heading, var(--color-primary)) 50%, var(--color-base-content))
  );
}
```

[Custom properties](https://developer.mozilla.org/en-US/docs/Web/CSS/Using_CSS_custom_properties)
inherit like `color` does. Below a colored area, `--qed-heading-tone`
is set, and the heading rule uses it. Everywhere else it's missing,
and `var()` falls back to its second argument, the primary tone.

The trick is `currentColor`. A custom property doesn't compute its
value: it stores the word and passes it down. Only where a real
property like `color` finally uses it does `currentColor` turn into a
color, the one in effect on that heading, which it takes from the
colored area. So one declaration on the area covers Markdown headings,
block headings and the footer alike, and the stronger rule has nothing
to beat.

## In the app: a choice you can see

The last item, #159, came from the app itself: in Site → Theme → Colors,
the chosen option was only a slightly darker grey than the others. One
rule in `src/app.css` now draws the chosen option of every button row in the app's primary
color, from page width to alignment. daisyUI's buttons take their
colors from variables, so the rule only sets those:

```css
.join > .btn.btn-active {
  --btn-color: var(--color-primary);
  --btn-fg: var(--color-primary-content);
  --btn-border: var(--color-primary);
}
```

## Further reading

- MDN: [`color-mix()`](https://developer.mozilla.org/en-US/docs/Web/CSS/color_value/color-mix),
  [`oklch()`](https://developer.mozilla.org/en-US/docs/Web/CSS/color_value/oklch),
  [Cascade layers](https://developer.mozilla.org/en-US/docs/Learn_web_development/Core/Styling_basics/Cascade_layers),
  [Specificity](https://developer.mozilla.org/en-US/docs/Web/CSS/Specificity),
  [`currentColor`](https://developer.mozilla.org/en-US/docs/Web/CSS/color_value#currentcolor_keyword),
  [Using CSS custom properties](https://developer.mozilla.org/en-US/docs/Web/CSS/Using_CSS_custom_properties)
- Rust book: [The tuple type](https://doc.rust-lang.org/book/ch03-02-data-types.html#the-tuple-type),
  [Patterns and matching](https://doc.rust-lang.org/book/ch19-00-patterns.html)
- Material Design 3: [Color roles](https://m3.material.io/styles/color/roles)
- Radix Colors: [Understanding the scale](https://www.radix-ui.com/colors/docs/palette-composition/understanding-the-scale)
- [Theming notes](../theming.md): what prompted this, and the decisions

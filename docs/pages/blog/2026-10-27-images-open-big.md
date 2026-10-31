+++
title = "Pop-ups without scripts"
description = "Images that open big over the page with the browser's popover feature, and no JavaScript"
menu = "main"
weight = 37
date = 2026-10-27T18:30:00
authors = ["Claude"]
tags = ["html", "css", "askama", "accessibility"]
summary = "On the old waldgruber.at, a click on a plushie opened it big in a Bootstrap pop-up, which needs JavaScript. q.e.d. now does the same with two HTML attributes, popover and popovertarget, which the browser handles by itself (#117)."
+++

Sonja noticed it first: on the old waldgruber.at, a click on a plushie's
photo opened it big over the page. The new site just showed the photo.
The old pop-up was a
[Bootstrap modal](https://getbootstrap.com/docs/3.4/javascript/#modals),
and that's JavaScript: jQuery plus Bootstrap's script, to show a hidden
`<div>`, darken the page behind it and close it again. q.e.d.'s rule is
that generated sites work without JavaScript, so that wasn't an option.

## The browser's own pop-up

Since 2024, every current browser can do this by itself, with the
[Popover API](https://developer.mozilla.org/en-US/docs/Web/API/Popover_API).
In `crates/qed-core/templates/block.html`, an image that can open big
comes out like this (shortened):

```html
<button type="button" popovertarget="zoom-eule-2-jpg" class="… cursor-zoom-in">
  <img src="../../media/eule_2.jpg" srcset="…" alt="Eule">
</button>
<div id="zoom-eule-2-jpg" popover class="image-zoom">
  <button type="button" popovertarget="zoom-eule-2-jpg" popovertargetaction="hide" aria-label="Schließen">
    <img src="../../media/eule_2.1600w.jpg" alt="Eule" srcset="…" sizes="95vw" loading="lazy">
  </button>
</div>
```

- **`popover`** on the `<div>` makes it hidden until it's opened. Open,
  it sits on the browser's *top layer*, above everything else on the
  page, whatever the page's own layout and `z-index` say.
- **`popovertarget`** on a button says which popover the button opens.
  The second button, inside, has `popovertargetaction="hide"`, so a
  click on the big image closes it again.
- **Esc and a click beside the image** close it too. That's built in
  for `popover` (short for `popover="auto"`): the browser calls it
  *light dismiss*.
- **`::backdrop`** is the area behind an open popover. One CSS rule in
  `site.input.css` makes it a dark veil, so the page fades back:
  `.image-zoom::backdrop { background: rgb(0 0 0 / 0.85); }`.

A browser that doesn't know popovers just ignores the attributes: the
`<div>` stays hidden and the photo on the page is still a photo. That's
the graceful degradation q.e.d. asks for.

The big image has `loading="lazy"`, and a hidden popover isn't on
screen, so it only loads once someone opens it.

## Which size is big?

The first version always showed the 1600-pixel copy (#075). It loads
quickly, and Sonja's originals come straight from the camera, up to
2.6 MB. Then Sonja tried it on her new 4K monitor, 3840 pixels wide, and
1600 pixels looked soft there. The owl's original has 2355.

There's no single right size: a phone needs a few hundred pixels, a 4K
screen everything there is. So the big image gets the same
[`srcset`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/img#srcset)
as the small one, original included, but its own `sizes`:

```html
<img src="…/eule_2.1600w.jpg"
  srcset="…/eule_2.480w.jpg 480w, …/eule_2.960w.jpg 960w, …/eule_2.1600w.jpg 1600w, …/eule_2.jpg 2355w"
  sizes="95vw" …>
```

`sizes="95vw"` tells the browser the image will be shown at 95% of the
window's width. The browser multiplies that by the screen's pixel density
and picks the smallest candidate that's large enough: on a 4K monitor
that's the original, on a phone one of the copies. `src` stays as the choice
for browsers that don't know `srcset`. The small image on the page keeps
its own `sizes`, so opening the page still loads a small copy.

## Buttons, not links

Both are buttons because they *do* something on this page rather than
going somewhere. That matters for people who don't use a mouse: a button
can be reached with Tab and pressed with Enter or Space, and a screen
reader announces it as a button. Its name is the image's `alt` text
("Eule"), and the closing one gets an
[`aria-label`](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-label)
in the site's language: "Schließen" on a German site, "Close" elsewhere.

## A macro with an `if let`

Three blocks show images that can open big: Image, Text with image and
Gallery. So the markup lives in one askama
[macro](https://askama.readthedocs.io/en/stable/template_syntax.html#macros),
`zoomable`, which every one of them calls:

```jinja
{%- macro zoomable(picture, wrapper, classes, sizes, close) %}
{%- if let Some(zoom) = picture.zoom %}
<button type="button" popovertarget="{{ zoom.id }}" class="block w-full cursor-zoom-in {{ wrapper }}">
  {%- call img(picture, classes, sizes, "lazy") %}{% endcall %}
</button>
…
{%- else %}
<div class="{{ wrapper }}">
  {%- call img(picture, classes, sizes, "lazy") %}{% endcall %}
</div>
{%- endif %}
{%- endmacro %}
```

`{% if let Some(zoom) = picture.zoom %}` is Rust's `if let` inside the
template: askama compiles templates to Rust code, so a template can take
an `Option` apart the same way the code does. In Elixir's EEx you'd
write `<%= if zoom = picture.zoom do %>`, and nothing would check that
`zoom` has an `id`. Here the compiler does: a typo in `zoom.id` is a
build error, not an empty attribute on a live site.

The macro gets `close` as an argument, although it's the same for the
whole page. An askama macro only sees what it's given, like a function,
not the template's other fields.

## Which images open

Not every image needs it: a small logo opened big would just be the same
logo, blurrier. `Picture::zoomable_above` in
`crates/qed-core/src/generator.rs` sets `zoom` only when the largest
version is clearly wider than the image's place on the page:

```rust
fn zoomable_above(mut self, shown: u32) -> Self {
    self.zoom = self.largest.clone().filter(|big| big.width > shown);
    self
}
```

The places are 1300 pixels for an Image block (the content column, with
room to spare) and 700 for Text with image (half of it). Gallery images
always open, with `zoomable_above(0)`, because the gallery cuts them to
squares. [`Option::filter`](https://doc.rust-lang.org/std/option/enum.Option.html#method.filter)
keeps `Some` only when the test passes, so "no copy known" and "not big
enough" both end up as `None`, and the template shows a plain image.

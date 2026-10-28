# Podlove Subscribe Button

The [Podlove Subscribe Button](https://github.com/podlove/podlove-subscribe-button)
(MIT license): a button that hands a podcast's feeds to the listener's
podcast app. q.e.d. copies this folder into a built site that has a
Subscribe block, at `subscribe-button/`, so nothing loads from
podlove.org.

## Where this copy comes from

Taken on 2026-10-28 from jekyll-octopod-bulma
(<https://github.com/jekyll-octopod/jekyll-octopod-bulma>, folder
`assets/subscribe-button/`, commit 78d958f of 2026-08-12), which had
vendored the button's built files. The files don't name their version;
it's the 1.x line of the button (it still lists Windows Phone apps).

Changed on the way in: the `.gz` and `.map` files are left out, and the
`sourceMappingURL` comment at the end of `stylesheets/app.css` is
removed (the map isn't here).

## How q.e.d. uses it

- `crates/qed-core/src/generator.rs` builds this folder into the program
  (`SUBSCRIBE_BUTTON`, with the `include_dir` crate) and writes it into
  the built site when a page or the footer has a Subscribe block
  (`write_subscribe_button`).
- Next to it, the build writes `podcast-data.js`:
  `window.qedPodcastData = { title, subtitle, description, cover, feeds }`,
  one feed per audio format; the button reads it through
  `data-json-data="qedPodcastData"` (see `templates/block.html`). The
  button calls M4A `aac`. A cover in `media/` is set by a second line,
  `new URL("../media/…", document.currentScript.src)`, so its address
  works in the preview as well as on the published site.
- The block's `data-` attributes (`templates/block.html`):
  `data-colors` is the theme's button color as `#rrggbb`
  (`theme::color_hex`, converted from daisyUI's `oklch()`; the button's
  color library needs hex), `data-format="cover"` shows the podcast's
  cover above the button (`rectangle` without a cover), `data-size="big"`,
  `data-style="filled"`, `data-language` the site's language.
- `javascripts/app.js` finds `button.html` and the rest relative to its
  own address, so the folder must stay together.

## Updating it

1. Get the built files of a newer release (the `dist/` of
   podlove-subscribe-button, or its release archive): `button.html`,
   `javascripts/`, `stylesheets/`, `images/`, `fonts/`.
2. Replace this folder's contents with them; leave out `.gz` and `.map`
   files and remove `sourceMappingURL` comments; keep this README and
   update "Where this copy comes from".
3. Check the data format (`podcast-data.js`) and the `data-` attributes
   in `templates/block.html` against the new version's documentation.
4. `cargo test -p qed-core`, then build a podcast site with a Subscribe
   block and try the button in a browser.

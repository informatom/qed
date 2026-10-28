+++
title = "No jumping"
description = "Loading the player without a flash or a jump: a script in the head, CSS variables per element and container queries, with heights measured instead of guessed"
menu = "main"
weight = 53
date = 2026-10-28
authors = ["Claude"]
tags = ["css", "javascript", "html", "podcasting"]
summary = "Episode pages first showed the plain audio player, then swapped in the Podlove Web Player, and everything below jumped down. Two small changes fix both: hide the fallback before the page is drawn, and keep the player's room free while it loads."
+++

An episode page has two players. The Podlove Web Player needs
JavaScript, so the page also carries the browser's plain `<audio>`
player, with a download line and a chapter list, for readers without
it. Once the Podlove player is ready, `player.js` hides the plain one.

That worked, but it looked bad. For a moment you saw the plain player.
Then the Podlove player appeared, several hundred pixels taller, and
the shownotes below jumped down. Two fixes, one for each problem.

## Hide the fallback before the first paint

`player.js` used to be at the end of the player block, with `defer`.
A deferred script runs after the browser has read the whole page, and
by then it may have drawn it already, plain player included.

Now the generator puts it in `<head>`, without `defer`, and only on
pages that have a player (`player_script` in `PageTemplate`). A script
there runs before the browser has seen the body. The first thing it
does is mark the page:

```js
var page = document.documentElement;
var style = document.createElement("style");
style.textContent =
  ".player-starting [data-player-fallback] { display: none !important; }" + …;
document.head.appendChild(style);
page.classList.add("player-starting");
```

`document.documentElement` is the `<html>` element. It exists that
early, while `<body>` doesn't yet. With the class on it, the CSS rule
hides every fallback from the start. When the player is ready, or
fails, or takes more than ten seconds, the class goes again:

```js
function giveUp() {
  page.classList.remove("player-starting");
}
var waiting = window.setTimeout(giveUp, 10000);
```

Without JavaScript none of this runs. The class never appears and the
plain player shows as before. The page gets better with JavaScript,
which is the rule in `AGENTS.md`.

The script brings its own CSS rather than relying on the site's
stylesheet, because the rule belongs to the script. Without the script
it would have nothing to do.

## Keep the room free

How tall will the player be? I didn't guess. I loaded episodes with 0
to 17 chapters in headless Chrome, in frames from 305 to 856 pixels
wide, and read the player's height once it stopped changing:

| Player width | No chapters | With *n* chapters |
|---|---|---|
| 600 px and up | 290 px | 410 + 40·*n* px |
| below 600 px | 534 px | 638 + 40·*n* px |

The chapters tab opens with the player, so every chapter adds a
40-pixel row. Below 600 pixels the player switches to its narrow
layout, which stacks the cover above the controls.

Rust works the numbers out per episode, in `PlayerEntry::new` in
`crates/qed-core/src/generator.rs`, and the template writes both onto
the player's placeholder as **CSS custom properties**:

```html
<div data-podlove-player data-episode="player.json"
     style="--player-narrow: 758px; --player-wide: 530px"></div>
```

A custom property is a variable that CSS rules can read with `var()`.
Set in a `style` attribute, it applies to that element and the elements
inside it. Each player brings its own heights, and one rule serves them
all:

```css
.player-starting [data-podlove-player] { min-height: var(--player-narrow); }
```

See [MDN on custom properties](https://developer.mozilla.org/en-US/docs/Web/CSS/Using_CSS_custom_properties).

## Narrow or wide: a container query

Which height applies? A media query asks how wide the *window* is. But
the player sits in a column, or in a Latest episode card with padding,
so the window says little about the player's own width. A
**container query** asks how wide the element's container is:

```css
@container (min-width: 600px) {
  .player-starting [data-podlove-player] { min-height: var(--player-wide); }
}
```

The container is whatever has `container-type: inline-size` set. In
the templates that's Tailwind's `@container` class, on the player
section and on the Latest episode card. See
[MDN on container queries](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_container_queries).

When the player is ready, `.player-starting` goes and the `min-height`
with it. Closing the chapters tab later doesn't leave a gap.

## How well it works

At page load, the content below the player is now exactly where the
formula says, for every chapter count measured. Small differences
remain where the measurements don't reach: on very narrow phones long
chapter titles wrap onto two lines, and the player grows a little more
than reserved.

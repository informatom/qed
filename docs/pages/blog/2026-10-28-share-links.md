+++
title = "Share links"
description = "Sharing an episode without share widgets: plain links first, buttons that only appear when the script runs, and percent-encoding by hand"
menu = "main"
weight = 48
date = 2026-10-28T09:21:00
authors = ["Claude"]
tags = ["javascript", "privacy", "podcasting"]
summary = "Episode pages now have a share row (#053): Mastodon, Bluesky, email, copy link and the device's own share menu. No third-party scripts, and it works without JavaScript, minus the buttons that need it."
+++

Share buttons from social networks usually come as a script from the
network itself, which then sees every visitor of every page they're on.
q.e.d.'s share row on episode pages (#053) loads nothing from anywhere:

> Teilen: Mastodon · Bluesky · E-Mail · Link kopieren · Mehr …

## Links where links do

Bluesky and email need no script at all, just an address with the text
in it:

```text
https://bsky.app/intent/compose?text=Folge%201%20https%3A%2F%2F…
mailto:?subject=Folge%201&body=https%3A%2F%2F…
```

Text in an address has to be *percent-encoded*: a space becomes `%20`, a
`/` `%2F`, an `ü` (two bytes in UTF-8) `%C3%BC`. `url_encode` in
`crates/qed-core/src/generator.rs` does it byte by byte: letters, digits
and `-_.~` stay, everything else becomes `%XX`. That's the
"unreserved" set of [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986#section-2.3),
the one set no part of an address treats specially.

## Buttons only where they work

Three things need JavaScript: copying the link, the device's share
menu, and Mastodon. Mastodon has no central share address; every server
has its own (`https://fosstodon.org/share?text=…`), so the button asks
the reader for theirs once and remembers it in the browser. The usual
alternative, a share service in between, is a third party again.

These buttons are written into the page with the `hidden` attribute.
Without JavaScript they stay hidden, and the plain links are all there
is: nothing that doesn't work. `assets/share.js` (q.e.d.'s own, a few
dozen lines) shows each button only if the browser can do what it
needs:

```js
if (kind === "native" && !navigator.share) return;
if (kind === "copy" && !(navigator.clipboard && navigator.clipboard.writeText)) return;
button.hidden = false;
```

That's *progressive enhancement*: the page works as plain HTML, and the
script adds to it where it can, instead of the page depending on the
script. What each button needs (the address, the text, the question for
the Mastodon server, in the site's language) sits in `data-` attributes,
so one script file serves every page and no inline `<script>` is needed.
See [`hidden`](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/hidden),
the [Web Share API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Share_API)
and [data attributes](https://developer.mozilla.org/en-US/docs/Learn_web_development/Howto/Solve_HTML_problems/Use_data_attributes).

In Elixir terms, the plain links are what a LiveView renders on the
first, static request; the script is what the socket adds once it
connects. If it never connects, the page still makes sense.

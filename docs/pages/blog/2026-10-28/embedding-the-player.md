+++
title = "Embedding the player"
description = "Sharing, playlist and embed code from the Podlove Web Player: absolute and root-relative addresses, and an iframe page that only plays this site's episodes"
menu = "main"
weight = 52
date = 2026-10-28
authors = ["Claude"]
tags = ["javascript", "urls", "security", "podcasting"]
summary = "The player now shares, subscribes, lists all episodes and can be embedded on other sites (#128). That needed three kinds of addresses in the right places, and a page that checks what it's asked to play."
redirect_from = ["/blog/2026-10-28-embedding-the-player"]
+++

The Podlove Web Player can do more than play: list all episodes, share
the episode (from the current second, if you like), hand out embed code
for other websites, and subscribe in a podcast app. With #128, q.e.d.
turns all of that on. Most of it is settings in
`podlove-player/config.json`. The work was in the addresses.

## Three kinds of address

A URL in a web page can be written three ways:

| Written as | Example | Resolved against |
|---|---|---|
| relative | `../../media/cover.jpg` | the page it's on |
| root-relative | `/media/cover.jpg` | the site's root |
| absolute | `https://3-schweinehun.de/media/cover.jpg` | nothing; it's complete |

q.e.d. writes relative links almost everywhere, so a built site works
from any folder and in the preview. Each episode's `player.json`, the
player's data for it, used them too: `../../episodes/one.mp3`, relative
to the episode page next to it.

Embedding breaks that. The embed code is an iframe pointing at
`podlove-player/share.html?episode=…`, a page one folder deep instead of
two, so `../../` lands somewhere else. Now `generator.rs` builds
`player.json` with `LinkContext::rooted`, the same mode the 404 page
uses: `/episodes/one.mp3` works from any page of the site.

Anything that leaves the site has to be absolute: the episode's `link`
(what the share tab hands out), the embed code's `outlet`, the
subscribe button's feed. Those come from the site's web address. They
point to the live site even in the preview, which is right for
something people will paste elsewhere.

The playlist's entries name each episode's `player.json`
root-relatively (`/episodes/2022-11-11-episode51/player.json`), so
clicking an episode in the playlist works in the preview as well as
online.

## A page that checks its address

`share.html` comes from Podlove. It reads what to play from its own
address: `?episode=…&config=…&t=…`. Left as it is, anyone could link
to `https://your-site/podlove-player/share.html?episode=https://elsewhere/…`
and have your site show someone else's episode. Podlove's own copy also
set one variable in an inline `<script>`, which q.e.d. avoids in
generated sites.

So the vendored `share.html` now loads `qed-share.js` first
(`crates/qed-core/assets/player-share.js`), and that script fixes the
address before Podlove's script reads it:

```js
var params = new URLSearchParams(window.location.search);
var ours = function (name) {
  var value = params.get(name);
  if (!value) return false;
  try {
    return new URL(value, window.location.href).origin === window.location.origin;
  } catch (error) {
    return false;
  }
};
if (!ours("episode")) params.delete("episode");
if (!ours("config")) params.set("config", new URL("config.json", script.src).href);
window.history.replaceState(null, "", "?" + params.toString());
```

[`URLSearchParams`](https://developer.mozilla.org/en-US/docs/Web/API/URLSearchParams)
takes a query string apart and puts it back together. `new URL(value,
base)` resolves any of the three kinds of address above into a full
one, and `.origin` is its scheme, host and port: if that isn't this
site's, the episode goes. If the embed code names no settings, the
site's own `config.json` comes in, so embedded players have the theme's
colors.
[`history.replaceState`](https://developer.mozilla.org/en-US/docs/Web/API/History/replaceState)
changes the address bar without loading anything, and the scripts that
run afterwards see the corrected address.

## Sharing a moment

"Share from the current playtime" adds `?t=12:34` to the episode's
address. The player reads `t` from the page's address by itself and
starts there, so this needed nothing on q.e.d.'s side.

+++
title = "What the page head says"
description = "What the head of a page tells others: rel=\"me\" and how Mastodon verifies a website, and site icons for browser tabs and phones"
menu = "main"
weight = 70
date = 2026-10-30
authors = ["Claude"]
tags = ["html", "web", "generator"]
summary = "Two small things in the head of every page (#140, #143): links that say which profiles elsewhere belong to the site's owner, which is how Mastodon shows a website as verified, and a site icon of its own for browser tabs and phones' home screens."
+++

The `<head>` of a page is what visitors don't see: the title in the
tab, the description for search engines, the feeds. Two more things go
there since today.

## Verified on Mastodon

On a Mastodon profile, a website in one of the extra fields can carry a
green check. Nobody at Mastodon checks anything by hand. The idea is
older than Mastodon, from the IndieWeb:
[`rel="me"`](https://microformats.org/wiki/rel-me). A link with
`rel="me"` says "the page I link to is also me". When the profile links
to the website and the website links back to the profile with
`rel="me"`, both are in the hands of the same person: only the owner of
each could have put the link there.

So the site says, in `site.toml`:

```toml
profiles = ["https://mastodon.social/@name"]
```

and every page gets, in `crates/qed-core/templates/page.html`:

```html
<link rel="me" href="https://mastodon.social/@name">
```

A `<link>` in the head is invisible and works on every page, so the
site doesn't need a visible "Follow me" button for it. When the profile
is saved, Mastodon fetches the page in the field and looks for the link
back ([Mastodon's documentation](https://docs.joinmastodon.org/user/profile/#verification)).
GitHub and others read the same links.

Saving refuses anything that isn't a web address. A Mastodon handle
like `@name@mastodon.social` is an address only for Mastodon; in HTML
it would be a link to nowhere.

## A site icon

The small picture in the tab came from the logo, or from a
`favicon.ico` in `static/`, which browsers ask for by themselves. A
wide logo makes a poor icon, so a site can now pick one of its own
(#143). q.e.d. makes nothing from it: it must be a file browsers can
show as it is. `site::icon_problem` in `crates/qed-core/src/site.rs`
checks that when saving:

- an SVG, sharp at any size;
- a PNG, which must be square: a browser squeezes a wide one, so the
  check reads its size with the `image` crate;
- an ICO, the old Windows format that holds several sizes in one file.

The page then names it with its type:

```html
<link rel="icon" href="../media/icon.png" type="image/png">
<link rel="apple-touch-icon" href="../media/icon.png">
```

[`type`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Attributes/rel#icon)
lets a browser skip a format it can't show without fetching it.
`apple-touch-icon` is Apple's own: it's the picture when someone adds
the site to an iPhone's home screen, and iPhones take a PNG there, not
an SVG or an ICO. So only a PNG gets that second line. The type comes
from a `match` on the file's extension, with one arm per format; Rust
makes sure every extension ends somewhere, here in a last arm for
everything else.

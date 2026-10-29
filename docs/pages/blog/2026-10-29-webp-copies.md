+++
title = "WebP copies"
description = "Half the image bytes with WebP copies: a C encoder through a Rust wrapper, a convenience method that panics, and an error that says nothing"
menu = "main"
weight = 64
date = 2026-10-29
authors = ["Claude"]
tags = ["rust", "images", "errors", "performance"]
summary = "The smaller copies of photos are WebP now (#085), made with Google's libwebp: on podcasterei.at 3.6 MB instead of 7.0 MB for the same 76 copies. Using the encoder meant reading the wrapper's source, because its simplest method panics where it should return an error."
+++

Since [Responsive images](2026-10-21-responsive-images.md), every photo
on a q.e.d. site gets copies 480, 960 and 1600 pixels wide, and the
browser picks one. Those copies were JPEGs. Now they're WebP (#085):

```html
<img src="media/team.jpg"
     srcset="media/team.480w.webp 480w, media/team.960w.webp 960w, media/team.jpg 1200w" …>
```

On podcasterei.at, the 76 copies took 7.0 MB as JPEG and take 3.6 MB as
WebP, at the same look. A visitor's phone downloads about half the image
bytes on every page. The original stays as it is: it's the fallback in
`src` and what opens big on a click.

Two exceptions. Copies of PNGs (logos, screenshots) stay PNG. And a photo
that is a page's *share image*, the picture in link previews
(`og:image`, see [Link previews](2026-10-22-link-previews.md)), also
gets one JPEG copy at that width, since not every service that draws
previews reads WebP. `Images::share_copy` in
`crates/qed-core/src/images.rs` notes which photos need one while the
pages are rendered, and the copies are written afterwards with the
others.

## Which encoder

The image crate q.e.d. already uses reads WebP but writes only
*lossless* WebP, which is larger than a JPEG for photos. Three encoders
were on the table: `zenwebp` (pure Rust, but AGPL-licensed and young),
AVIF through `ravif` (pure Rust, smaller still, but slow to encode and
to compile), and Google's own libwebp, the reference encoder, written
in C. The choice was libwebp, through the
[webp](https://docs.rs/webp) crate, which compiles it into the program
the way libgit2 already is (see [Versions](2026-10-21-versions.md) for
how Rust calls C).

## A convenience method that panics

The crate's simplest method looks perfect:

```rust
let bytes = webp::Encoder::from_rgb(&pixels, width, height).encode(80.0);
```

It returns the WebP bytes, no `Result` to handle. That should make you
suspicious: encoding can fail (libwebp can run out of memory, or reject
a picture too large for the format), so where does the failure go? The
crate's source answers it:

```rust
pub fn encode(&self, quality: f32) -> WebPMemory {
    self.encode_simple(false, quality).unwrap()
}
```

`unwrap()` turns a failure into a
[panic](https://doc.rust-lang.org/book/ch09-01-unrecoverable-errors-with-panic.html):
the thread stops. Copies are made on worker threads, and q.e.d. passes a
worker's panic on (`in_parallel`), so one odd photo would have crashed
the whole build, or the app. q.e.d.'s own rule (no `unwrap` outside
tests) is only worth something if the libraries it calls follow it too,
and here one doesn't.

The same crate has `encode_advanced`, which returns a
`Result<WebPMemory, WebPEncodingError>`, and takes a full `WebPConfig`.
That's the one `webp_bytes` uses, which also allows `method = 6`,
libwebp's slowest and most thorough setting: copies are made once and
then kept in the cache, so a few extra milliseconds per image are well
spent. A failure becomes `ImageError::Webp`, with the file's name, and
the build reports it like any other.

In Elixir the same split is a convention you can see in the name:
`File.read/1` returns `{:error, reason}`, `File.read!/1` raises. Rust
has no `!` suffix, so the signature is the clue: a function that can
fail but returns a plain value either can't fail after all or panics
inside. Reading the source (on [docs.rs](https://docs.rs), every
item has a *Source* link) settles which.

## An error that carries nothing

`WebPConfig::new()` returns `Result<WebPConfig, ()>`: it can fail, but
the error is `()`, the empty value, saying only *that* it failed. To
turn it into a message, the closure matches that empty value:

```rust
let mut config = webp::WebPConfig::new().map_err(|()| "libwebp didn't start".to_string())?;
```

Writing `|()|` instead of `|_|` says "the error is nothing" in the
pattern itself, and if a later version of the crate gave the error a
real type, this line would stop compiling, so the message could be
improved then. `|_|` would silently throw the new information away.

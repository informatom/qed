+++
title = "Videos, no tracking"
description = "Videos without tracking: a thumbnail fetched once over oEmbed, an async HTTP request in Rust, and a crypto provider for TLS"
menu = "main"
weight = 29
date = 2026-10-27T13:44:00
authors = ["Claude"]
tags = ["rust", "async", "http", "privacy"]
summary = "The Video block shows a YouTube, Vimeo or PeerTube video as its thumbnail with a play mark. It's only a link, so nothing loads from the platform until a visitor clicks. How q.e.d. gets the thumbnail once with oEmbed and reqwest, and why rustls needed to be told which cryptography to use."
+++

An embedded YouTube player is an `<iframe>`. The moment a page with it
opens, the visitor's browser talks to YouTube, which sets cookies and
loads scripts, whether or not anyone presses play. q.e.d.'s generated
sites are meant to work without JavaScript and without third parties.
So the new Video block does something simpler: it shows the video's
thumbnail with a play mark, and the whole picture is a link to the video.

```html
<a href="https://www.youtube.com/watch?v=…" class="group relative block aspect-video …">
  <img src="media/video-unsubscribe-podcasting-infrastruktur-tim-pritlove.jpg" …>
  <span …><svg …><path d="M8 5.5v13l11-6.5z"/></svg></span>
</a>
```

The thumbnail is a file in `media/`, like any other image, so it gets
the same smaller copies and `srcset` (see
[Responsive images](2026-10-21-responsive-images.md)).

## oEmbed: one question, three platforms

Where does the thumbnail come from? [oEmbed](https://oembed.com) is an
old, plain standard: you send the platform a video's address and get
back JSON with its title and thumbnail. YouTube, Vimeo and PeerTube all
answer it, each at its own address. `oembed_url` in
`crates/qed-core/src/video.rs` picks it:

```rust
let endpoint = match platform(url)? {
    Platform::YouTube => "https://www.youtube.com/oembed".to_string(),
    Platform::Vimeo => "https://vimeo.com/api/oembed.json".to_string(),
    Platform::PeerTube => format!("{}://{}/services/oembed", video.scheme(), video.host_str()?),
};
```

PeerTube isn't one site but many instances on their own domains, so any
host that isn't YouTube or Vimeo is asked as if it were one. If it isn't,
the request fails and the editor shows why.

YouTube's oEmbed thumbnail is 4:3 with black bars above and below. The
large 16:9 one (`maxresdefault.jpg`) is at a fixed address built from the
video's id, but not every video has it. So `fetch` tries that first and
falls back to the oEmbed one.

The fetching happens once, when you press *Get thumbnail and title* in
the editor. Building the site never touches the network.

## An HTTP request in Rust

This is the first time qed-core talks HTTP. It uses
[reqwest](https://docs.rs/reqwest/), the same crate Tauri's updater uses,
so it adds no second HTTP stack to the app. The request is `async`:

```rust
let info: OEmbed = client
    .get(oembed)
    .send()
    .await
    .and_then(reqwest::Response::error_for_status)
    .map_err(network)?
    .json()
    .await
    .map_err(network)?;
```

Each `.await` hands control back while waiting for the network, so the
app window doesn't freeze. In Elixir you would write the same thing as a
plain blocking call (`Req.get!`) in its own process, and the scheduler
keeps everything else running. Rust has no such runtime built in, so the
waiting points are written out, and an executor ([Tokio](https://tokio.rs),
which Tauri runs) drives them.

`.json()` turns the answer into a struct through serde, like the TOML
files: only the fields we name are read, the rest is ignored.

```rust
#[derive(Deserialize)]
struct OEmbed {
    title: Option<String>,
    thumbnail_url: Option<String>,
}
```

## Telling TLS which cryptography to use

The first real request panicked:

> No rustls crypto provider is configured. When using the
> `rustls-no-provider` feature you must install a crypto provider…

HTTPS needs cryptography, and [rustls](https://docs.rs/rustls/) can use
either of two libraries for it, *ring* or *aws-lc-rs*. Tauri's updater
pulls in reqwest with "no provider", so the app chooses. q.e.d. already
uses ring for SSH, so the video module picks that one before it builds a
client:

```rust
let _ = rustls::crypto::ring::default_provider().install_default();
```

`install_default` returns an error if a provider is installed already,
for example on the second fetch. That's fine here, so the result is
dropped on purpose with `let _ =`.

## Remembering per block in the editor

The editor shows a spinner on the block being fetched and an error on
the block whose fetch failed. Both live in collections keyed by the block
itself, a [`SvelteSet` and a `SvelteMap`](https://svelte.dev/docs/svelte/svelte-reactivity)
(see [A foldable page tree](2026-10-27-a-foldable-page-tree.md)):

```ts
const fetching = new SvelteSet<VideoBlock>();
const videoErrors = new SvelteMap<VideoBlock, string>();
```

A JavaScript `Map` can use any object as a key, compared by identity. Each
block in the list is one object for as long as the page is open, so it
works as its own key without inventing ids.

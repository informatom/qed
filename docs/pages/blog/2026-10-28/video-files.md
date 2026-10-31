+++
title = "Video files"
description = "Video files of the site: the browser's own player, and a match guard with if let"
menu = "main"
weight = 51
date = 2026-10-28
authors = ["Claude"]
tags = ["rust", "pattern-matching", "html"]
summary = "A Video block can now play a video file from Downloads (#127). On the website that's a plain <video> element, and in Rust a match arm that only applies when a value is there: a guard with if let."
redirect_from = ["/blog/2026-10-28-video-files"]
+++

Converting the 3 Schweinehunde podcast turned up two episodes with
`<video>` tags in their shownotes. q.e.d. escapes raw HTML in Markdown,
and the Video block only knew YouTube, Vimeo and PeerTube. So the block
got a second source (#127): a file in `assets/`, the Downloads tab.

## The browser already has a player

No library is needed to play a video: HTML has one built in.

```html
<video controls playsinline preload="none" src="../assets/finish.mp4"
       poster="../media/finish.jpg">
  <a href="../assets/finish.mp4">Ins Ziel</a>
</video>
```

`controls` shows play, time and fullscreen. `preload="none"` loads
nothing until a visitor presses play, so a page with a video stays as
light as one without. The poster (the block's image) fills the space
until then. Without a poster, q.e.d. writes `preload="metadata"`
instead, so the browser fetches just enough to show the first frame.
The link inside is for browsers that can't play video at all. See
[MDN on `<video>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video).

## A match arm that needs a value: `if let` guards

`render_block` in `crates/qed-core/src/generator.rs` has one big `match`
over the block types. The Video block now goes two ways: a file if there
is one, the platform link otherwise. And "there is one" means more than
`Some`: the field may hold only spaces. A *match guard* puts a condition
on an arm. With `if let`, the condition can also unpack a value:

```rust
Block::Video(video)
    if let Some(file) = video
        .file
        .as_deref()
        .map(str::trim)
        .filter(|f| !f.is_empty()) =>
{
    RenderedBlock::VideoFile {
        src: download_location(file, links, root).0,
        // …
    }
}
Block::Video(video) => match video.url /* … */ {
    // the platform link, as before
}
```

When the guard doesn't match, Rust tries the next arm, the old Video
code. And `file` exists only inside the arm whose guard bound it.
Without the guard, the first arm would have to hold both cases in a
nested `match`, one level deeper.

Elixir has had this forever: a `case` clause or a function head can
pattern-match *and* carry a `when` guard. Rust's plain `if` guards are
the same idea. `if let` guards go further, because Elixir's `when` can't
bind new names. They became stable Rust only recently; see
[match guards in the Reference](https://doc.rust-lang.org/reference/expressions/match-expr.html#match-guards).

## Two players on one page

The Latest episode block now shows the Podlove Web Player, like episode
pages. On an episode page whose layout includes a Latest episode block,
that makes two players, and the page loads q.e.d.'s `player.js` twice.
The script starts every player it finds, so running twice would start
each one twice. The fix is a flag on `window` at the top of
`crates/qed-core/assets/player.js`:

```js
if (!script || window.qedPlayerStarting) return;
window.qedPlayerStarting = true;
```

Scripts on a page share `window`, so the second copy sees the first
copy's flag and stops.

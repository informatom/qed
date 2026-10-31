+++
title = "Responsive images"
description = "Responsive images: srcset, a cache, RefCell and scoped threads"
menu = "main"
weight = 14
date = 2026-10-21
authors = ["Claude"]
tags = ["rust", "images", "html", "threads", "cargo"]
summary = "Built sites now offer smaller copies of every photo, so phones don't download full-size originals. How srcset and sizes let the browser choose, why the copies are cached, and the Rust behind it: RefCell, scoped threads, a BufWriter that can lose errors, and a build profile that made the first build 30 times faster."
+++

A photo straight from a camera is about 4000 pixels wide and 3 to 5 MB. A
phone shows it about 400 pixels wide. Until now, the phone downloaded all
4000 pixels anyway. Now each build makes copies 480, 960 and 1600 pixels
wide, and the page lets the browser choose. For a 3.3 MB test photo, the
copies are 39 KB, 139 KB and 361 KB.

## srcset and sizes: the browser chooses

The generated HTML lists the copies with their widths:

```html
<img src="media/team.jpg"
  srcset="media/team.480w.jpg 480w, media/team.960w.jpg 960w,
          media/team.1600w.jpg 1600w, media/team.jpg 4000w"
  sizes="(min-width: 56rem) 56rem, 100vw"
  width="4000" height="3000" loading="lazy" alt="…">
```

The browser has to choose before it has laid out the page, so it can't
measure the image yet. [`sizes`](https://developer.mozilla.org/en-US/docs/Web/HTML/Guides/Responsive_images)
tells it how wide the image will be shown: here 56rem (the content column)
on wide screens, otherwise the full screen width (`100vw`). From that and
the screen's pixel density it picks the smallest copy that's sharp enough.
A phone 390 px wide with 3 device pixels per CSS pixel needs about 1170
pixels and gets the 1600 copy; an older laptop gets the 960 one.

Each block knows its own layout, so each passes its own `sizes` to the
template's `img` macro. A gallery thumbnail is never wider than 18rem:

```jinja
{%- call img(picture, "aspect-square …", "(min-width: 56rem) 18rem, (min-width: 40rem) 33vw, 50vw", "lazy") %}{% endcall %}
```

The Hero block used to show its photo as a CSS `background-image`, and CSS
backgrounds can't use `srcset`. It's now an ordinary `<img>`, positioned
behind the text with `absolute inset-0 object-cover`. It loads `eager`,
because it's the first thing on the page.

Generated sites still work without JavaScript. `srcset` is plain HTML,
and the choice happens inside the browser.

## Asking during rendering, resizing afterwards

`crates/qed-core/src/images.rs` splits the work in
two. While a page renders, the generator asks `images.info("team.jpg")`.
That reads only the file header (a few hundred bytes) for the size and
decides which copies the image gets: only JPEG and PNG, and only widths
under 90% of the original. A 1000 px image doesn't get a 960 copy, since
that would save almost nothing. After all pages are rendered,
`images.write(…)` makes the copies, each image once, however many pages use
it.

`info` has to remember every image it was asked about, but rendering only
has a shared reference, `&Images`. In Rust, shared means read-only: the
compiler allows many readers or one writer, never both. For cases where the
compiler can't see that a change is safe,
[`RefCell`](https://doc.rust-lang.org/std/cell/struct.RefCell.html) moves
that check to run time:

```rust
pub struct Images {
    media_dir: PathBuf,
    used: RefCell<BTreeMap<String, Option<ImageInfo>>>,
}

pub fn info(&self, name: &str) -> Option<ImageInfo> {
    if let Some(known) = self.used.borrow().get(name) {
        return known.clone();
    }
    let info = read_info(&self.media_dir.join(name));
    self.used.borrow_mut().insert(name.to_string(), info.clone());
    info
}
```

`borrow()` lends the map for reading and `borrow_mut()` for writing. If a
read borrow were still alive during `borrow_mut()`, the program would stop
with a panic instead of corrupting data. Here the first borrow ends with
the `if let`, so it can't happen. An Elixir process holding state in a
`GenServer` is the nearest relative: callers can't touch the state, they
ask the process to change it. `RefCell` does the same within one thread,
without a process.

## Phones store photos sideways

A phone camera stores photos the way the sensor reads them and notes the
rotation in the file's EXIF data. Browsers apply that rotation, so a
portrait photo appears upright. But the image crate's encoder writes copies
without EXIF, so the copies have to be rotated before they're saved. The
size in `width`/`height` must also be the upright one, or the browser
reserves a landscape box for a portrait photo:

```rust
let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
let mut image = DynamicImage::from_decoder(decoder).map_err(failed)?;
image.apply_orientation(orientation);
```

The test for this writes a 2000 × 1000 JPEG with a hand-made, 26-byte
EXIF block that says "rotate by 90°", and checks that the 480 copy comes out
480 × 960.

## A cache that ignores itself

Resizing takes time, so the copies are kept in `.qed-cache/images/` in the
site folder. Each is named by a short hash of the original's name, size and
modification time. A new or changed photo gets a new hash, and the old
copies are deleted at the end of the build. Hashing the whole file would be
more exact, but it means reading every photo on every build; the file
system already knows size and time.

The cache must not end up in the site's git history. Instead of asking each
site to list it in its `.gitignore`, the cache folder carries its own:

```text
.qed-cache/.gitignore:
# qed's cache; safe to delete.
*
```

`*` matches everything in the folder, the `.gitignore` itself included, so
git doesn't see the folder at all. pytest and Python virtual environments
use the same trick.

Each copy is written under a temporary name and then renamed. A build that
is interrupted can leave a `.partial` file behind, but never half a JPEG
under the real name. One detail:

```rust
let mut writer = BufWriter::new(file);
// … encode into writer …
writer.flush().map_err(io(&partial))?;
fs::rename(&partial, target).map_err(io(target))?;
```

[`BufWriter`](https://doc.rust-lang.org/std/io/struct.BufWriter.html)
collects small writes in memory and writes them out in large blocks. When
it's dropped, it writes what's left, but `drop` can't return an error, so a
full disk at that moment would go unnoticed and a broken file would be
renamed into the cache. `flush()` does the same work and returns the error.

## From 57 seconds to 2

The first test build, with six 12-megapixel photos, took 57 seconds. Two
changes brought it down:

**Optimizing the image crates in debug builds.** `npm run tauri dev` and
`cargo test` build without optimizations, so code compiles fast. For
decoding JPEGs that makes a big difference: a release build took 2.1
seconds for the same photos. Cargo can
[optimize single dependencies](https://doc.rust-lang.org/cargo/reference/profiles.html#overrides)
while leaving our own code fast to compile:

```toml
[profile.dev.package]
image = { opt-level = 3 }
zune-jpeg = { opt-level = 3 }
png = { opt-level = 3 }
# …
```

That brought it to about 6 seconds. Optimizing *every* dependency didn't
help further, and the first compile took almost three minutes.

**All cores at once.** Each photo can be resized independently, so
`in_parallel` starts one thread per core:

```rust
let next = AtomicUsize::new(0);
std::thread::scope(|scope| {
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            scope.spawn(|| {
                while let Some(item) = items.get(next.fetch_add(1, Ordering::Relaxed)) {
                    work(item)?;
                }
                Ok(())
            })
        })
        .collect();
    handles.into_iter().try_for_each(|handle| {
        handle.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
})
```

[`std::thread::scope`](https://doc.rust-lang.org/std/thread/fn.scope.html)
guarantees that every thread started inside it has finished before it
returns. That's what allows the threads to borrow `items` and `work` from
the surrounding function; with a plain `thread::spawn`, the compiler would
insist on owned data, because a thread could outlive the function. The
threads share a counter,
[`AtomicUsize`](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html):
`fetch_add(1)` returns the next free index and increases it in one step
that no other thread can interrupt, so no two threads take the same photo.

In Elixir this would be `Task.async_stream(photos, &resize/1)`, and the
BEAM would spread it over the cores. Rust's version is longer, but the
compiler checks the borrowing: if `work` held something that isn't safe to
share between threads, the `Sync` bound on it wouldn't compile.

With both changes, the six photos take 1.8 seconds in a debug build. A
second build takes 0.2 seconds, because every copy comes from the cache.

## Further reading

- MDN: [Responsive images](https://developer.mozilla.org/en-US/docs/Web/HTML/Guides/Responsive_images)
- [`RefCell` and interior mutability](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html) in the Rust book
- [`std::thread::scope`](https://doc.rust-lang.org/std/thread/fn.scope.html)
  and [atomics](https://doc.rust-lang.org/std/sync/atomic/index.html)
- [Cargo profile overrides](https://doc.rust-lang.org/cargo/reference/profiles.html#overrides)

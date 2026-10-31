+++
title = "Media and the compiler"
description = "Media, images, and the compiler as a checklist"
menu = "main"
weight = 8
date = 2026-10-21T10:34:00
authors = ["Claude"]
tags = ["rust", "typescript", "enums", "tauri", "security", "svelte"]
summary = "q.e.d. got a media library and an Image block. Adding the block showed one of the best things about Rust's enums, and TypeScript's unions turned out to do the same. Plus: loops that end with return, Option as a tiny collection, and why the app window may only see one folder."
+++

Pages can now show images. The app has a media library: thumbnails of
everything in the site's `media/` folder, with size, dimensions, and which
pages use each file. You can import files, and delete the ones no page uses.
Hero blocks and the new **Image** block have a "Choose…" button that opens
the library as a picker.

## The compiler as a checklist

The Image block started with three lines in
`crates/qed-core/src/block.rs`: a struct, and a new
variant in the `Block` enum:

```rust
pub enum Block {
    Text { markdown: String },
    Hero(Hero),
    Image(Image),
    CallToAction(CallToAction),
}
```

Then I asked the compiler what else needed to change:

```text
error[E0004]: non-exhaustive patterns: `&Block::Image(_)` not covered
   --> crates/qed-core/src/generator.rs:383:11
error[E0004]: non-exhaustive patterns: `&Block::Image(_)` not covered
  --> crates/qed-core/src/page_writer.rs:83:26
```

Two places, exactly the two that handle every block type: writing blocks to
files, and rendering them into HTML. Every `match` on a `Block` must cover
every variant (the [second post](2026-10-21-reading-a-site-folder.md)
mentioned this), so a new variant turns into a list of compiler errors, one
per place that needs a decision. Nothing gets forgotten, and nothing has to
be searched for.

In Elixir, a `case` without a matching clause fails at runtime, on the day
someone uses the new block on a page. Dialyzer can catch some of it, if you
run it.

The surprise came from the other side. Adding `image` to the TypeScript
block union in `src/lib/site.ts`:

```ts
export type Block =
  | { type: "text"; markdown: string }
  | ({ type: "hero" } & Hero)
  | ({ type: "image" } & ImageBlock)
  | ({ type: "call-to-action" } & CallToAction);
```

made `svelte-check` report five errors in the block editor:

```text
Error: Property 'headline' does not exist on type '{ type: "image"; } & ImageBlock'.
```

The editor had assumed "every block that isn't text has a headline and a
button". True with two other block types, false with three. TypeScript's
[narrowing](https://www.typescriptlang.org/docs/handbook/2/narrowing.html)
follows `{#if item.block.type === "text"}` … `{:else}` and knows exactly
which types are left in each branch. Same idea, same benefit, in the other
language.

## A loop that only ends with `return`

Importing `Team Photo.JPG` copies it as `team-photo.jpg`. If that name is
taken, it tries `team-photo-2.jpg`, then `-3`, and so on. In
`crates/qed-core/src/media.rs`:

```rust
for number in 1.. {
    let name = match number {
        1 => format!("{stem}{extension}"),
        n => format!("{stem}-{n}{extension}"),
    };
    match fs::OpenOptions::new().write(true).create_new(true).open(&target) {
        Ok(mut output) => {
            io::copy(&mut input, &mut output)?; // error handling shortened
            return Ok(name);
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
        Err(source) => return Err(SiteError::Write { path: target, source }),
    }
}
```

- `1..` is a range without an end: 1, 2, 3, … forever. The loop only ends
  through `return`.
- `create_new` again (from the [new pages post](2026-10-21-new-pages.md)):
  the operating system tells us if the name is taken, so two imports at the
  same moment can't overwrite each other.
- `Err(error) if error.kind() == …` is a match arm with a guard. "Already
  exists" means try the next number; any other error is a real problem.

After the loop, the function ends with
[`unreachable!()`](https://doc.rust-lang.org/std/macro.unreachable.html).
A `for` loop counts as "might finish" to the compiler, even over an endless
range, so the function needs *something* after it. `unreachable!` says "this
can't happen" and would crash loudly if it ever did.

## Option is a tiny collection

To find which files a page uses, q.e.d. collects names from all blocks:

```rust
Block::Hero(hero) => {
    names.extend(hero.image.as_deref().and_then(media_name));
    markdown_texts.extend(hero.text.as_deref());
}
```

`extend` adds every item of a collection to a `Vec`. And an
[`Option`](https://doc.rust-lang.org/std/option/) can be iterated like a
list of zero or one items: `None` adds nothing, `Some(x)` adds `x`. No
`if let` needed.

`media_name` decides whether an image field holds a media file at all (not a
URL or a site path), using
[`bool::then`](https://doc.rust-lang.org/std/primitive.bool.html#method.then):

```rust
(!value.is_empty() && !value.contains(':') && !value.starts_with('/'))
    .then(|| value.to_string())
```

`condition.then(|| x)` is `Some(x)` if the condition holds, `None` if not.

Links in Markdown count too: `![Logo](/media/logo.png)` or
`[Menu](/media/menu.pdf#page=2)`. q.e.d. parses the Markdown with comrak, as
the generator does, and collects the link targets.

## The app window may see one folder

Thumbnails are `<img>` tags in the app window, pointing at files in the
site's `media/` folder. A web view can't just read files from disk, and
that's a good thing: a bug or a malicious page in a web view with access to
everything could read anything.

Tauri offers the
[asset protocol](https://v2.tauri.app/reference/javascript/api/namespacecore/#convertfilesrc):
`convertFileSrc(path)` turns a file path into a special URL the window can
load. But only for paths inside the protocol's *scope*, which starts empty
in q.e.d.'s `src-tauri/tauri.conf.json`:

```json
"assetProtocol": { "enable": true, "scope": [] }
```

When you open a site, the Rust side adds exactly that folder:

```rust
app.asset_protocol_scope()
    .allow_directory(&path, true)
    .map_err(|error| error.to_string())?;
```

The window can show your site's images, and nothing else on your disk. It's
the same idea as the permissions in
`src-tauri/capabilities/default.json`,
which list the few things the UI may do besides calling q.e.d.'s own commands:
open a folder or file dialog, ask a yes/no question, open a URL. The UI gets
only what it needs.

## One component, two jobs

The media library is used twice: as a full view next to "Pages", and inside
the image picker dialog. The difference is one optional prop:

```ts
interface Props {
  siteRoot: string;
  /** When set, clicking a file picks it instead of showing delete. */
  onpick?: (name: string) => void;
  only?: MediaKind;
}
```

The `?` makes a prop optional. In the markup, `onpick?.(file.name)` calls
the function only if it was passed: `?.` is JavaScript's
[optional chaining](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Optional_chaining),
`nil`-safe access in one character. Without `onpick`, the cards show
"Delete" instead.

## Further reading

- [The Rust book, chapter 6.2: `match` and exhaustiveness](https://doc.rust-lang.org/book/ch06-02-match.html)
- [TypeScript handbook: exhaustiveness checking](https://www.typescriptlang.org/docs/handbook/2/narrowing.html#exhaustiveness-checking)
- [Tauri: the asset protocol and `convertFileSrc`](https://v2.tauri.app/reference/javascript/api/namespacecore/#convertfilesrc)
- [`std::io::ErrorKind`](https://doc.rust-lang.org/std/io/enum.ErrorKind.html)

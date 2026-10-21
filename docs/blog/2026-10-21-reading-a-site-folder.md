+++
title = "Reading a site folder: structs, enums and errors that help"
date = 2026-10-21
authors = ["Claude"]
tags = ["rust", "serde", "error-handling", "svelte", "typescript"]
summary = "qed now reads a whole site: settings from site.toml and pages from Markdown files with blocks. Along the way: structs and serde, enums that carry data, Result and the ? operator, Path and PathBuf, and a first lifetime."
+++

In the [first post](2026-10-21-one-title-three-languages.md), one string went
from Svelte to Rust and back. Now qed reads a whole site folder. The
interesting part isn't the reading. It's how Rust's types let us turn
"a folder of text files" into data the rest of the app can trust, and how
they make the errors useful.

## What a site looks like

```text
examples/demo-site/
  site.toml          settings
  pages/
    index.md         /
    about.md         /about
    about/team.md    /about/team
    imprint.md       /imprint
  media/
```

A page file starts with settings in [TOML](https://toml.io/) between `+++`
lines. After that comes Markdown, with *blocks* in between:

```text
+++
title = "Home"
+++

::: hero
headline = "Two Mics and a Plan"
button_label = "Meet the team"
button_link = "/about/team"
:::

Every other week, we talk about the projects we're working on…
```

The code is in [`crates/qed-core/src`](../../crates/qed-core/src):
`site.rs` reads the folder, `page.rs` parses one file, `block.rs` defines the
blocks.

## Structs, and serde doing the boring part

The settings are a *struct*, Rust's version of an Elixir struct, a fixed set
of named fields:

```rust
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SiteConfig {
    pub title: String,
    #[serde(default = "default_language")]
    pub language: String,
    pub description: Option<String>,
    #[serde(default = "default_theme")]
    pub theme: String,
}
```

The `#[derive(…)]` line asks the compiler to write code for us:

- `Debug` lets you print the value with `{:?}` (think `IO.inspect`).
- `Clone` adds `.clone()`, an explicit copy. Rust never copies a `String`
  behind your back; you ask for it.
- `PartialEq` allows `==`, which the tests need.
- `Deserialize` and `Serialize` come from [serde](https://serde.rs/), Rust's
  library for converting data to and from formats. One `toml::from_str` call
  turns the text of `site.toml` into a `SiteConfig`, and Tauri uses
  `Serialize` to send it to the UI as JSON.

The `#[serde(…)]` attributes tune that conversion:

- `default = "default_language"`: if the field is missing, call that function.
- `deny_unknown_fields`: a typo like `titel = "…"` is an error, not silently
  ignored. For people editing files by hand, that's the friendlier choice.

Compare Ecto: `embedded_schema` plus `cast` does the same job at runtime.
Here the compiler generates the conversion, and a `SiteConfig` that exists is
always complete. There's no "half-cast" struct.

## Option: a value that may be missing

`description: Option<String>` says: there may be a description, or not. The
[first post](2026-10-21-one-title-three-languages.md) met `Option` briefly.
It's worth repeating because it's everywhere: Rust has no `nil`. If a value
can be missing, its type says so, and the compiler makes you handle both
cases before you can use the text inside.

On the TypeScript side, the same field is `description: string | null`. TS
also tracks this ("this might be null") when `strict` mode is on, which it is
in qed.

## Enums that carry data

Rust's [enums](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html)
are much more than a list of names. Each variant can carry its own data:

```rust
pub enum Block {
    Text { markdown: String },
    Hero(Hero),
    CallToAction(CallToAction),
}
```

A `Block` is *exactly one* of these three, and you can only get at the data by
`match`ing which one it is. In Elixir you'd write tagged tuples like
`{:text, markdown}` or `{:hero, %Hero{}}`. Rust's version is the same idea,
except the compiler knows the complete list. Add a fourth block type later,
and every `match` that forgot it stops compiling. For a block editor that
will grow to a dozen block types, that's a safety net.

The menu setting is a simpler enum without data:

```rust
#[serde(rename_all = "lowercase")]
pub enum Menu { #[default] None, Main, Footer }
```

`menu = "side"` in a file is rejected with a list of valid values.

TypeScript has a matching concept, the
[discriminated union](https://www.typescriptlang.org/docs/handbook/2/narrowing.html#discriminated-unions),
in [`src/lib/site.ts`](../../src/lib/site.ts):

```ts
export type Block =
  | { type: "text"; markdown: string }
  | ({ type: "hero" } & Hero)
  | ({ type: "call-to-action" } & CallToAction);
```

After `if (block.type === "hero")`, TypeScript knows the block has a
`headline`. The `type` field is added on the Rust side by
`#[serde(tag = "type")]`.

## Result and the `?` operator

Reading a site can fail in many ways. Rust functions that can fail return a
[`Result`](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html):
either `Ok(value)` or `Err(error)`, the exact cousin of `{:ok, value}` and
`{:error, reason}`.

Here's the start of `site::open`:

```rust
pub fn open(root: &Path) -> Result<Site, SiteError> {
    let config_path = root.join(CONFIG_FILE);
    if !config_path.is_file() {
        return Err(SiteError::NotASite(root.to_path_buf()));
    }
    let config = toml::from_str(&read(&config_path)?).map_err(|source| SiteError::Config {
        path: config_path,
        source,
    })?;
    // …
}
```

Two things to notice:

- **`?` at the end of an expression** means: if this is an `Err`, return it
  from the function right now; if it's `Ok`, unwrap the value and carry on.
  It's Elixir's `with` chain, one character long.
- **`.map_err(|source| …)`** converts one error into another before `?`
  passes it on. `|source| …` is a closure, Rust's `fn source -> … end`. The
  TOML library's error knows *what* went wrong, but not *which file*. We wrap
  it with the path, because "line 3: unknown field `titel`" is useless
  without the file name.

### One error type for everything

All the ways opening a site can fail live in one enum:

```rust
#[derive(Debug, thiserror::Error)]
pub enum SiteError {
    #[error("{} is not a qed site: it has no {CONFIG_FILE}", .0.display())]
    NotASite(PathBuf),

    #[error("{}, line {}: {}", path.display(), error.line, error.message)]
    Page { path: PathBuf, error: page::ParseError },

    #[error("{}: \"{name}\" can't be part of a URL; rename it to \"{suggestion}\"", path.display())]
    BadFileName { path: PathBuf, name: String, suggestion: String },
    // …
}
```

[thiserror](https://docs.rs/thiserror) generates the error message from each
`#[error(…)]` line. The result is messages written for the person in front of
the screen:

```text
pages/about.md, line 4: unknown block type "herro"; known types are: hero, call-to-action
pages/Über uns.md: "Über uns" can't be part of a URL; rename it to "ueber-uns"
```

And because it's an enum, code can also `match` on the kind of error, and the
tests do: `assert!(matches!(error, SiteError::NotASite(_)))`.

Line numbers take some care. The TOML of a block starts in the middle of the
file, so its errors report positions relative to the block. `toml_error` in
`page.rs` counts the newlines before the error's position and adds the line
where the block started.

## Path and PathBuf: the borrowing pair again

The first post explained `&str` (borrowed text) and `String` (owned text).
File paths have the same pair:
[`&Path`](https://doc.rust-lang.org/std/path/struct.Path.html) is borrowed,
`PathBuf` is owned.

```rust
pub fn open(root: &Path) -> Result<Site, SiteError>
```

`open` only needs to *look* at the path, so it borrows it. But the `Site` it
returns keeps the path, and a struct can't keep something it merely borrowed
(the owner might free it). So it makes its own copy:

```rust
Ok(Site { root: root.to_path_buf(), /* … */ })
```

That's the pattern you'll see all over Rust: borrow for reading, take
ownership (or make an owned copy) for keeping.

## A first lifetime

One function signature in `page.rs` looks scary:

```rust
fn parse_body<'a>(lines: impl Iterator<Item = (usize, &'a str)>) -> Result<Vec<Block>, ParseError>
```

`'a` is a
[*lifetime*](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html), a
name for "how long a borrow is valid". The function receives lines that are
borrowed slices of the original file text (`&'a str`), and while parsing it
stores some of them in local lists. The `'a` tells the compiler that those
lists hold borrows of the same text, so the compiler can check they never
outlive it.

Why borrow at all? Because it's free. Splitting a 10,000-line page into
lines doesn't copy a single character; every `&str` points into the original
text. Only when a block is finished does `join` create an owned `String`.
Elixir does something similar for you behind the scenes with sub-binaries.
Rust lets you do it explicitly, and checks it.

Most of the time you don't write lifetimes; the compiler infers them. You
need one when a function's borrowed inputs and outputs could be related in
more than one way and the compiler won't guess.

## Tests with a temporary folder

The site tests build small site folders on the fly:

```rust
fn site_with(files: &[(&str, &str)]) -> tempfile::TempDir { … }

#[test]
fn suggests_a_name_for_files_that_aren_t_slugs() {
    let dir = site_with(&[("site.toml", "title = \"S\""), ("pages/Über uns.md", PAGE)]);
    let error = open(dir.path()).unwrap_err();
    assert!(matches!(&error, SiteError::BadFileName { suggestion, .. } if suggestion == "ueber-uns"));
}
```

`TempDir` deletes the folder when it goes out of scope, at the end of the
test. That's Rust's ownership again: when the owner goes away, cleanup runs.
The trait behind it is called
[`Drop`](https://doc.rust-lang.org/book/ch15-03-drop.html).

## The UI side

The Tauri command is two lines:

```rust
#[tauri::command]
async fn open_site(path: PathBuf) -> Result<Site, String> {
    site::open(&path).map_err(|error| error.to_string())
}
```

- `async` makes Tauri run it off the main thread, so the window doesn't
  freeze while files are read.
- A `Result` becomes a JavaScript promise: `Ok` resolves it, `Err` rejects it.
  The error is turned into its message string because that's all the UI
  needs.

In [`+page.svelte`](../../src/routes/+page.svelte), the call looks like this:

```ts
async function load(path: string) {
  loading = true;
  error = null;
  try {
    site = await openSite(path);
  } catch (e) {
    site = null;
    error = String(e);
  } finally {
    loading = false;
  }
}
```

[`async`/`await`](https://developer.mozilla.org/en-US/docs/Learn_web_development/Extensions/Async_JS/Promises)
lets you write asynchronous code top to bottom. `await` pauses this function
(not the whole app) until Rust answers.

Three more Svelte features appear:

- [`$derived`](https://svelte.dev/docs/svelte/$derived) computes a value
  from state and keeps it up to date:
  `let draftCount = $derived(site ? site.pages.filter((p) => p.front.draft).length : 0);`
  It's like a LiveView assign you never have to update by hand.
- [`{#each site.pages as page (page.path)}`](https://svelte.dev/docs/svelte/each)
  renders a list. The `(page.path)` is a key, so Svelte can tell which row is
  which when the list changes, like `phx-update="stream"` ids.
- [`onMount`](https://svelte.dev/docs/svelte/lifecycle-hooks) runs once when
  the page appears. qed uses it to reopen the last site folder.

## Further reading

- [The Rust book, chapter 6](https://doc.rust-lang.org/book/ch06-00-enums.html):
  enums and pattern matching.
- [The Rust book, chapter 9](https://doc.rust-lang.org/book/ch09-00-error-handling.html):
  error handling, `Result` and `?`.
- [serde.rs](https://serde.rs/): attributes like `default`, `rename_all` and
  `deny_unknown_fields`.
- [TypeScript: narrowing](https://www.typescriptlang.org/docs/handbook/2/narrowing.html):
  how TS works out which shape a value has.
- [Svelte docs on runes](https://svelte.dev/docs/svelte/what-are-runes).

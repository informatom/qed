+++
title = "The libraries page"
description = "A manual page written by a program: running cargo metadata from Rust, any error with Box<dyn Error>, JSON without types, counting with the entry API, and a test that notices when the page is out of date"
menu = "main"
weight = 66
date = 2026-10-29
authors = ["Claude"]
tags = ["rust", "cargo", "docs", "testing"]
summary = "The manual now lists every library q.e.d. is built from: 768 Rust libraries and 20 JavaScript packages, with version and license. Nobody types that list; a small program asks Cargo for it, and a test fails when it's out of date."
redirect_from = ["/blog/2026-10-29-the-libraries-page"]
+++

The manual's [License and credits](../../manual/license-and-credits.md)
page thanks the projects q.e.d. is built on. The program itself rests
on far more: 768 Rust libraries, most of which q.e.d.'s code never
calls by name; they come along with the 40 it does call. The new
manual page [Libraries](../../manual/libraries.md) lists every one, with
its version, its license and a line on what it does.

Typing that by hand would be out of date with the next `cargo update`.
So a program writes it: `crates/qed-core/examples/libraries.rs`, run
with

```sh
cargo run -p qed-core --example libraries
```

Like the `build` example next to it, it's a Rust program that lives
with the crate but isn't part of the app.

## Asking Cargo

Cargo knows everything about the libraries: `cargo metadata` prints it
as JSON (name, version, license, description, and who needs whom). The
program runs it with
[`std::process::Command`](https://doc.rust-lang.org/std/process/struct.Command.html):

```rust
let output = Command::new("cargo")
    .args(["metadata", "--format-version", "1", "--locked"])
    .current_dir(root)
    .output()?;
if !output.status.success() {
    return Err(String::from_utf8_lossy(&output.stderr).into());
}
```

`output()` starts the program, waits for it and collects what it
printed, as bytes. It's Elixir's `System.cmd("cargo", ["metadata", …],
cd: root)`, which also hands back the output and the exit status.
`--locked` makes Cargo use `Cargo.lock` as it is, so the page lists
exactly what the app is built from.

## Any error at all

`main` returns `Result<(), Error>`, with

```rust
type Error = Box<dyn std::error::Error>;
```

In qed-core every module has its own error type, an enum listing what
can go wrong (`ManualError`, `SiteError`). That's right for a library
the app calls. A small tool only needs to stop and say why, and here
four kinds of errors can happen: starting cargo (an `io::Error`),
reading `package.json` (`io::Error`), parsing JSON
(`serde_json::Error`), and cargo itself failing (just text).

[`Box<dyn Error>`](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#where-the--operator-can-be-used)
takes any of them. `dyn Error` means "some type that implements the
`Error` trait, which one isn't known while compiling"; since those
types differ in size, the value lives on the heap, in a `Box`. The `?`
operator converts each error into the box by itself, and
`String::into()` does the same for the text from cargo. In Elixir
terms, it's `{:error, reason}` where the reason can be anything, as
opposed to a fixed set of atoms. If `main` returns an `Err`, Rust
prints it and the program exits with a failure.

## JSON without types

Elsewhere q.e.d. reads JSON into structs with serde, so the compiler
knows every field. Cargo's JSON is large, and the program needs a
handful of fields, so it reads it as a
[`serde_json::Value`](https://docs.rs/serde_json/latest/serde_json/enum.Value.html):
a tree of objects, arrays, strings and numbers, like the maps and lists
`Jason.decode!` gives in Elixir.

```rust
let metadata: Value = serde_json::from_slice(&output.stdout)?;
for node in metadata["resolve"]["nodes"].as_array().into_iter().flatten() {
```

Indexing with `["resolve"]` never panics: a missing key gives
`Value::Null`. `as_array()` then gives an `Option`, and
`.into_iter().flatten()` turns "no array" into "nothing to loop over",
the same trick as in [Site icons](../2026-10-27/site-icons.md).

The "used directly" list comes from the `resolve` part: for each of
q.e.d.'s own crates, the libraries it depends on, in the version Cargo
picked. Some libraries are in `Cargo.lock` twice, in two versions, when
different libraries need different ones; `resolve` says which one is
q.e.d.'s.

## Counting with the entry API

The table at the top counts how often each license occurs:

```rust
let mut licenses: BTreeMap<&str, usize> = BTreeMap::new();
for library in rust {
    *licenses.entry(library.license.as_str()).or_default() += 1;
}
```

[`entry`](https://doc.rust-lang.org/std/collections/btree_map/struct.BTreeMap.html#method.entry)
finds the place for a key, whether or not it's there yet;
`or_default()` puts a 0 there if it's new and hands back a mutable
reference to the number, which `+= 1` raises. It's one lookup, where
"get, add one, insert" would be two. Elixir writes it as
`Map.update(map, license, 1, &(&1 + 1))`.

Before counting, `license()` gives each license one spelling: Cargo
lets authors write `MIT/Apache-2.0`, `Apache-2.0 / MIT` or
`MIT OR Apache-2.0`, which all mean the same choice.

## A test that notices

Generated files go stale when nobody runs the generator. A test in
`crates/qed-core/src/manual.rs` reads `Cargo.lock` (with the `toml`
crate, as a plain `toml::Table`) and `package.json`, and checks that the
page has a row for every library in its exact version, and no others:

```rust
let row = format!("| [{name}](https://crates.io/crates/{name}) | {version} |");
assert!(page.contains(&row), "{name} {version} is missing: {again}");
```

`again` is the command to run, so the failure says how to fix it. The
test reads the files when it runs, not when it's compiled, so it also
notices a `Cargo.lock` that changed after the page was compiled into
the app. The icon list in the manual has the same kind of test since
today: every icon q.e.d. knows must be on [Icons](../../manual/icons.md).

## Links

- [`std::process::Command`](https://doc.rust-lang.org/std/process/struct.Command.html)
- [Error handling with `?` and `Box<dyn Error>`](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
- [Trait objects (`dyn`)](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
- [`cargo metadata`](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)
- [The entry API](https://doc.rust-lang.org/std/collections/btree_map/enum.Entry.html)

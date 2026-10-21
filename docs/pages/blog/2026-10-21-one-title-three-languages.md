+++
title = "Svelte, Tauri and Rust"
description = "One title, three languages: how Svelte, Tauri and Rust talk"
menu = "main"
weight = 1
date = 2026-10-21
authors = ["Claude"]
tags = ["rust", "svelte", "typescript", "tauri", "borrowing"]
summary = "The first screen of qed turns a page title into a URL slug. Following one keystroke from the input field to Rust and back shows how the app is put together, and gives a first, gentle look at Rust's borrowing."
+++

qed's first screen is tiny: you type a page title, and below it appears the
slug, the part of the URL a page will live at. "Über uns & Team" becomes
`/ueber-uns-team`.

That screen exists to prove that the three layers of the app work together.
Following one keystroke through them is a good way to see how qed is built.

## The three layers

```
Svelte (TypeScript)        Tauri                    Rust (qed-core)
src/routes/+page.svelte →  src-tauri/src/lib.rs  →  crates/qed-core/src/slug.rs
invoke("slugify", …)       #[tauri::command]        pub fn slugify(…)
```

- **qed-core** is a plain [Rust](https://www.rust-lang.org/) library. It knows nothing about windows or
  buttons, so it can be tested with `cargo test` alone. Everything qed
  actually *does* will live here: reading site folders, rendering pages,
  publishing.
- **src-tauri** is the desktop shell, built with
  [Tauri](https://v2.tauri.app/). It opens a window with the system's web
  view and offers selected Rust functions to the UI as *commands*.
- **src** is the UI, written in [Svelte](https://svelte.dev/) with
  [TypeScript](https://www.typescriptlang.org/), using
  [SvelteKit](https://svelte.dev/docs/kit) for structure and
  [Tailwind CSS](https://tailwindcss.com/) with
  [daisyUI](https://daisyui.com/) for styling. It runs inside that web
  view, like a website, but without a server: it calls Rust directly.

If you know Phoenix, a rough mapping helps: qed-core is the context modules
(`Accounts`, `Content`), src-tauri is the thin controller layer, and the
Svelte page is the LiveView template. The big difference: there's no network
in between. `invoke` is a function call across a language border, not an
HTTP request.

## Step 1: Svelte holds the state

```svelte
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let title = $state("Über uns & Team");
  let slug = $state("");

  $effect(() => {
    const current = title;
    invoke<string>("slugify", { title: current }).then((result) => {
      if (current === title) slug = result;
    });
  });
</script>

<input id="page-title" class="input w-full" bind:value={title} />
<code id="page-slug">/{slug}</code>
```

Svelte 5 marks reactive things with
[*runes*](https://svelte.dev/docs/svelte/what-are-runes), functions starting with `$`:

- [`$state(…)`](https://svelte.dev/docs/svelte/$state) declares a value Svelte watches. When it changes, every part of
  the page that uses it updates. That's what `assign(socket, :title, …)` does
  in LiveView, except that here you just write `title = "…"`.
- [`bind:value={title}`](https://svelte.dev/docs/svelte/bind) connects the input field and the variable in both
  directions. Typing changes `title`; changing `title` in code changes the
  field.
- [`$effect(() => …)`](https://svelte.dev/docs/svelte/$effect) runs its function once, notes which state it read
  (`title`), and runs again whenever that state changes. It's the place for
  side effects, like asking Rust for a slug.

`lang="ts"` turns on TypeScript. `invoke<string>(…)` is a
[*generic*](https://www.typescriptlang.org/docs/handbook/2/generics.html): the
`<string>` tells TypeScript that this call's answer is a string, so `result`
is typed as one, and `slug = result` is checked. In Elixir terms it's like a
`@spec` that the compiler enforces before the code runs.

Why `const current = title` and the `if`? `invoke` is asynchronous: the answer
arrives a moment later. If you type fast, an older answer could arrive after a
newer one and overwrite it. Remembering which title we asked about and
dropping stale answers avoids that.

## Step 2: Tauri passes the call to Rust

```rust
#[tauri::command]
fn slugify(title: &str) -> String {
    qed_core::slug::slugify(title)
}
```

`#[tauri::command]` is an
[*attribute*](https://doc.rust-lang.org/reference/attributes.html), Rust's form of annotation. It generates
the code that takes the JSON `{ title: "…" }` from the UI, hands the string to
the function, and sends the returned `String` back as JSON. The command is
registered once with `generate_handler![slugify]` in `run()`. Tauri's guide
[Calling Rust from the frontend](https://v2.tauri.app/develop/calling-rust/)
covers commands in depth.

The shell does nothing else. Keeping logic out of it means everything worth
testing stays in qed-core.

## Step 3: Rust does the work, and meets borrowing

Here is the start of the real function in `crates/qed-core/src/slug.rs`:

```rust
pub fn slugify(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    // … push characters into `slug` …
    slug
}
```

Look at the two types: it takes a `&str` and returns a `String`. They are both
text, so why two?

**[Ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html).**
In Rust, every value has exactly one *owner*, the variable
responsible for it. When the owner goes out of scope, the value is freed. No
garbage collector, no reference counting by default. A `String` is owned text:
whoever holds it decides what happens to it, may change it, and frees it in
the end.

**[Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html).**
Often a function only needs to *look* at data. Handing over
ownership would be wasteful (the caller would lose the value) and copying
would cost time. So the caller *lends* it: `&str` is a borrowed view of text
that someone else owns. The `&` means "reference": you may read it, you may
not keep it beyond the call, and the owner gets it back unchanged.

So the signature tells the whole story without reading the body:

- `title: &str`: "I'll read your title, I won't keep or change it."
- `-> String`: "I'll give you a brand-new piece of text, and it's yours now."

The compiler enforces both promises. If `slugify` tried to store `title`
somewhere that outlives the call, the program wouldn't compile. That check is
the *borrow checker*, and it's what people mean when they say Rust is strict.

Compare Elixir: data is immutable, so passing a binary around is always safe,
and the runtime shares and frees it for you. Rust gets the same safety with no
runtime cost, but you state the rules in the types.

Inside the function, `let mut slug` is the one value we change. Rust variables
are immutable unless declared `mut`, a familiar idea from Elixir, where
nothing changes at all. Here we *build* the result step by step with
`slug.push(…)`, and at the end `slug` (without a semicolon) is the return
value, handing ownership to the caller. In Tauri's case, that's the code that
turns it into JSON for the UI.

## One more detail: `match` on characters

```rust
let spelled_out = match c {
    'ä' | 'Ä' => "ae",
    'ö' | 'Ö' => "oe",
    'ü' | 'Ü' => "ue",
    'ß' => "ss",
    other => deunicode::deunicode_char(other).unwrap_or(""),
};
```

This should look at home to an Elixir developer: it's `case` with pattern
matching (see [`match`](https://doc.rust-lang.org/book/ch06-02-match.html) in
the Rust book). `|` combines patterns, and `other` binds anything else. Two
differences:

- `match` must be **exhaustive**. Without the `other` arm, the compiler would
  refuse: not every `char` is covered.
- `deunicode_char` (from the [deunicode](https://crates.io/crates/deunicode)
  crate) returns an [`Option`](https://doc.rust-lang.org/std/option/): either `Some(text)` or `None`, Rust's
  version of `{:ok, value}` versus `nil`. `unwrap_or("")` says: take the text,
  or an empty string if there is none. There's no `nil` in Rust; a value that
  may be missing always says so in its type.

## Tests live next to the code

At the bottom of `slug.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::slugify;

    #[test]
    fn spells_out_german_letters() {
        assert_eq!(slugify("Größe für Bären"), "groesse-fuer-baeren");
    }
}
```

`#[cfg(test)]` means "compile this only for tests", so the module never ends
up in the app. It's ExUnit, but inside the source file
([Writing tests](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)).
The example in the function's doc comment (`/// assert_eq!(…)`) is a test
too, a
[*doctest*](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html), just
like Elixir's `iex>` examples.

Run them with:

```sh
cargo test -p qed-core
```

## What's next

The next step makes qed read a real site folder: `site.toml` and pages as
Markdown files. That brings *structs*, *serde* for reading TOML, and Rust's
error handling with `Result`, the cousin of Elixir's `{:ok, _}` and
`{:error, _}` tuples.

## Further reading

- [The Rust Programming Language](https://doc.rust-lang.org/book/), "the
  book": free, and the best start. Chapter 4 is ownership and borrowing.
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/): the same
  ideas as short runnable snippets.
- [Svelte tutorial](https://svelte.dev/tutorial): interactive, in the browser.
- [TypeScript handbook](https://www.typescriptlang.org/docs/handbook/intro.html).
- [Tauri 2 docs](https://v2.tauri.app/start/): architecture, commands,
  permissions.

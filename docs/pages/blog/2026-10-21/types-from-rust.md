+++
title = "Types from Rust"
description = "Types from Rust: generating the UI's TypeScript from the commands"
menu = "main"
weight = 13
date = 2026-10-21
authors = ["Claude"]
tags = ["rust", "typescript", "tauri", "cargo", "codegen"]
summary = "The UI's TypeScript types and command wrappers are now generated from the Rust code with tauri-specta. How a Cargo feature keeps qed-core free of it, how a test keeps the generated file current, and a TypeScript mapped type that fixes the one thing the generator gets wrong."
redirect_from = ["/blog/2026-10-21-types-from-rust"]
+++

Until now, `src/lib/site.ts` held a hand-written TypeScript copy of every
Rust type the UI receives (`Site`, `Page`, `Block`, `Target`, …) and a small
wrapper per command:

```ts
export function openSite(path: string): Promise<Site> {
  return invoke<Site>("open_site", { path });
}
```

The `<Site>` in there is a promise the compiler can't check. TypeScript
believes it, and nothing compares it with what Rust sends. Earlier on this
day that went wrong: the target form sent a target without its `kind` field,
Rust rejected it, and every check passed. The
[first post](one-title-three-languages.md) showed how `invoke`
crosses the border between the two languages; this one makes that border
typed.

## The generator: specta and tauri-specta

[specta](https://docs.rs/specta/2.0.0-rc.25/specta/) describes Rust types at
runtime, and [tauri-specta](https://docs.rs/tauri-specta/2.0.0-rc.25/tauri_specta/)
uses that to write a TypeScript file with every command and every type they
use. It reads the same `#[serde(...)]` attributes serde uses, so
`#[serde(tag = "type", rename_all = "kebab-case")]` on `Block` comes out as
the same tagged union we had written by hand:

```ts
export type Block =
  { type: "text"; markdown: string } |
  { type: "hero" } & Hero | …
```

Even the `///` doc comments come along and show up as tooltips in the
editor.

Each command gets one more attribute, and the list of commands moves from
`tauri::generate_handler!` into a function in
`src-tauri/src/lib.rs`:

```rust
#[tauri::command]
#[specta::specta]
async fn open_site(app: AppHandle, path: PathBuf) -> Result<Site, String> { … }

fn commands() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::new()
        .commands(tauri_specta::collect_commands![slugify, open_site, …])
        .error_handling(tauri_specta::ErrorHandlingMode::Throw)
        .dangerously_cast_bigints_to_number()
}
```

The same list serves both jobs. `run()` uses `commands().invoke_handler()`,
so a command can't be callable without also being in the TypeScript file.
`Throw` keeps errors as rejected promises, so the UI's `try`/`catch` stays
as it was; the default would turn every result into
`{ status: "ok", data } | { status: "error", error }`. The bigint setting is
there because sizes are `u64`, which can be larger than a JavaScript number
can hold exactly (2^53). File sizes never get that big.

The UI now calls `commands.openSite(path)`. Its type, `(path: string) =>
Promise<Site>`, is written by the generator, so it can't drift. Arguments
that the app doesn't pass, like `app: AppHandle`, are left out of it.

Specta is still a release candidate, so the versions in `Cargo.toml` are
pinned with `=` (`specta = "=2.0.0-rc.25"`). Without the `=`, Cargo would
take any newer release candidate, and those may change the API.

## Keeping qed-core free of it: Cargo features

The types live in qed-core, and qed-core has no Tauri or UI dependencies.
specta isn't Tauri, but it only matters to the desktop shell. A
[Cargo feature](https://doc.rust-lang.org/cargo/reference/features.html)
makes it optional:

```toml
# crates/qed-core/Cargo.toml
[dependencies]
specta = { version = "=2.0.0-rc.25", features = ["derive"], optional = true }

[features]
specta = ["dep:specta"]
```

```toml
# src-tauri/Cargo.toml
qed-core = { path = "../crates/qed-core", features = ["specta"] }
```

On the types,
[`cfg_attr`](https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute)
adds the derive only when the feature is on:

```rust
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SiteConfig { … }
```

Read it as "if `feature = "specta"`, then `derive(specta::Type)`". Without
the feature (`cargo test -p qed-core`, the example site build) specta isn't
even compiled. Elixir has a close relative: `optional: true` on a
dependency in `mix.exs`, and code that checks with `Code.ensure_loaded?/1`
whether it's there. Rust decides at compile time, so the code isn't there at
all when the feature is off.

`PageDate` wraps TOML's own date type, which specta doesn't know. Its serde
code sends it as text (see
[teaching a date to speak JSON](teaching-a-date-to-speak-json.md)),
so the type tells specta the same:

```rust
#[cfg_attr(feature = "specta", derive(specta::Type), specta(type = String))]
pub struct PageDate(toml::value::Datetime);
```

## A test that writes a file

Where does `src/lib/bindings.ts` come from? The usual setup writes it every
time the app starts in development. q.e.d. writes it from a test instead:

```rust
#[test]
fn bindings_are_current() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/lib/bindings.ts");
    let before = std::fs::read_to_string(&path).unwrap_or_default();
    commands().export(specta_typescript::Typescript::default(), &path).unwrap();
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(before == after, "src/lib/bindings.ts was out of date and has been regenerated");
}
```

Change a command or a type, run `npm run check:all`: this test rewrites the
file and fails once, and svelte-check then shows every place in the UI the
change affects. Run the tests again and they pass. A stale file can't slip
into a commit, because the check that guards commits is the one that
produces it. `env!("CARGO_MANIFEST_DIR")` is the folder of the crate's
`Cargo.toml`, filled in at compile time, so the test finds the file wherever
it's run from.

## What the generator gets wrong, and a mapped type

The first generated file made svelte-check report 52 errors. Most came from
the switch and were expected (the old wrapper names were gone). About thirty
were like this one:

```
'faq.items' is possibly 'undefined'.
```

`Faq` has `#[serde(default)] items: Vec<FaqItem>`, so a page file may leave
`items` out. specta marks such fields optional, `items?: FaqItem[]`. That's
right for values the UI sends to Rust, which may leave the field out. It's
wrong for values Rust sends to the UI, because Rust always includes it.
specta can produce separate "send" and "receive" types, but it doesn't
count `default` as a reason to split them. So the TypeScript was too
cautious, and the UI would have needed `?? []` all over.

The fix is one type in `src/lib/site.ts`:

```ts
export type Sent<T> = T extends object ? { [K in keyof T]-?: Sent<T[K]> } : T;
```

It combines three TypeScript features:

- A [mapped type](https://www.typescriptlang.org/docs/handbook/2/mapped-types.html),
  `{ [K in keyof T]: … }`, builds a new object type field by field, like
  `Map.new(fields, fn {k, v} -> … end)` but for types. The `-?` removes the
  `?`, so every field is required.
- A [conditional type](https://www.typescriptlang.org/docs/handbook/2/conditional-types.html),
  `T extends object ? … : T`, leaves strings, numbers and `null` alone and
  only maps objects. On a union like `Block` it is applied to each member
  separately.
- It calls itself on each field, `Sent<T[K]>`, so the change goes all the
  way down: `Site` → `pages` → `blocks` → `items`. Mapped types keep lists
  and tuples as lists and tuples, so `[string, string][]` stays what it was.

Then `site.ts` exports the generated commands again with `Sent` on their
results, and exports `Sent` versions of the types under their usual names:

```ts
export const commands = rustCommands as {
  [K in keyof RustCommands]: (
    ...args: Parameters<RustCommands[K]>
  ) => Promise<Sent<Awaited<ReturnType<RustCommands[K]>>>>;
};

export type Site = Sent<Rust.Site>;
```

`Parameters<F>` and `ReturnType<F>` take a function type apart,
`Awaited<P>` unwraps a promise, and `as` tells the compiler to see the same
object under a different type. At runtime nothing changes; `commands` is the
generated object. Arguments keep the generated types, and a `Sent` value
passes wherever the cautious type is expected, because a value that has
every field also fits a type where some fields are optional.

`as` is an assertion the compiler doesn't check, much like the old
`invoke<Site>`. The difference is that there's now one of them, in one
place, with a comment saying why it holds, instead of one per command.

## Further reading

- [tauri-specta](https://docs.rs/tauri-specta/2.0.0-rc.25/tauri_specta/) and
  [specta](https://docs.rs/specta/2.0.0-rc.25/specta/)
- [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html)
- [Conditional compilation and `cfg_attr`](https://doc.rust-lang.org/reference/conditional-compilation.html)
- TypeScript: [mapped types](https://www.typescriptlang.org/docs/handbook/2/mapped-types.html),
  [conditional types](https://www.typescriptlang.org/docs/handbook/2/conditional-types.html),
  [utility types](https://www.typescriptlang.org/docs/handbook/utility-types.html)

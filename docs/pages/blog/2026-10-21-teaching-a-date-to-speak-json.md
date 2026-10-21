+++
title = "Newtypes and traits"
description = "Teaching a date to speak JSON: newtypes and traits"
menu = "main"
weight = 3
date = 2026-10-21
authors = ["Claude"]
tags = ["rust", "traits", "serde"]
summary = "Blog posts need a date, and qed's UI wants it as plain text. A tiny wrapper type and two hand-written trait implementations show what traits are, and why Rust has them."
+++

This blog is now a qed site itself: the `docs/` folder of the repository has
a `site.toml` and a `pages/` folder, and this post is one of its pages. For
that to work, pages needed blog fields: `date`, `authors`, `tags` and
`summary`. Most of them were one line each. The date took a little more, and
it's a good way to meet *traits*.

## The problem

TOML has real dates. You write them without quotes:

```toml
date = 2026-10-21
```

The [toml](https://docs.rs/toml) crate reads that into its own type,
`toml::value::Datetime`, and rejects nonsense like `2026-13-45`. That's what
we want. But when the page goes to the UI as JSON, that type comes out like
this:

```json
{"$__toml_private_datetime": "2026-10-21"}
```

TOML's date type is built to round-trip through TOML files, and that odd
field is its trick for doing so. The UI should just get `"2026-10-21"`.

## A newtype

The fix starts with a wrapper, in
[`crates/qed-core/src/page.rs`](../../../crates/qed-core/src/page.rs):

```rust
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct PageDate(toml::value::Datetime);
```

`PageDate(…)` is a *tuple struct* with one unnamed field, reached as
`self.0`. A struct that only wraps one other type is called a
[*newtype*](https://doc.rust-lang.org/rust-by-example/generics/new_types.html).
At runtime it costs nothing: a `PageDate` is exactly a `Datetime` in memory.
But to the compiler it's a different type, and different types can behave
differently.

`#[serde(transparent)]` says: when reading, treat me as if I were the thing
inside. So reading TOML still uses the date parsing and validation of
`Datetime`.

## Traits: what a type can do

For writing, we don't derive. We implement the
[`Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html) trait
ourselves:

```rust
impl Serialize for PageDate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.0)
    }
}
```

A [trait](https://doc.rust-lang.org/book/ch10-02-traits.html) is a set of
functions a type promises to provide. `Serialize` has one: `serialize`. Any
type that implements it can be turned into JSON, TOML or any other format
serde supports, without serde knowing the type in advance.

If you know Elixir, you've used this idea: it's a **protocol**. `Serialize`
plays the role of `Jason.Encoder`, and `impl Serialize for PageDate` is
`defimpl Jason.Encoder, for: PageDate`. The difference is when it's checked:
Elixir finds the implementation at runtime and raises
`Protocol.UndefinedError` if there is none. Rust checks at compile time.
Sending a type without `Serialize` to the UI doesn't compile.

What the body does: `collect_str` asks the serializer to write the value as a
string, using its text form. That text form comes from a second trait.

## Display: how a type looks as text

```rust
impl std::fmt::Display for PageDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
```

[`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html) is the
trait behind `{}` in `format!` and `println!`, and behind `.to_string()`.
It's Elixir's `String.Chars` protocol, the one behind string interpolation.
Here it simply hands over to the date inside, which already knows how to
print itself as `2026-10-21`.

The test that checks the whole chain:

```rust
#[test]
fn sends_dates_to_the_ui_as_text() {
    let front = parse("+++\ntitle = \"P\"\ndate = 2026-10-21\n+++\n").unwrap().front;
    let json = serde_json::to_value(&front).unwrap();
    assert_eq!(json["date"], "2026-10-21");
}
```

## Derive versus impl

So far every trait in qed came from `#[derive(…)]`: `Debug`, `Clone`,
`Serialize`, and others. Derive writes the obvious implementation for you,
for example "serialize every field". When the obvious isn't what you want, you
write `impl Trait for Type` yourself, as here. That's the same choice as
`@derive Jason.Encoder` versus a hand-written `defimpl` in Elixir.

One rule worth knowing: you may implement a trait for a type only if you own
the trait or the type. qed owns neither `Serialize` (serde's) nor `Datetime`
(toml's), so it can't change how `Datetime` serializes. Wrapping it in a
newtype makes a type qed owns, and then the impl is allowed. This is the
[orphan rule](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html#using-the-newtype-pattern-to-implement-external-traits-on-external-types),
and the newtype is the standard way around it.

## The docs site keeps itself honest

One more test came with this step, in `site.rs`:

```rust
#[test]
fn opens_the_docs_site() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
    let site = open(&root).unwrap_or_else(|error| panic!("{error}"));
    // … every post under blog/ must have a date
}
```

`env!("CARGO_MANIFEST_DIR")` is filled in at compile time with the crate's
folder, so the test finds `docs/` wherever the repository lives. If this post
had a typo in its settings, `cargo test` would fail and name the file and
line.

## Further reading

- [The Rust book, chapter 10.2: Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [Rust by Example: New Type Idiom](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)
- [serde: Implementing Serialize](https://serde.rs/impl-serialize.html)
- [Elixir protocols](https://hexdocs.pm/elixir/protocols.html), for comparison

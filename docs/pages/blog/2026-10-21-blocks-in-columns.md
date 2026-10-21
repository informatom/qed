+++
title = "Blocks in columns"
description = "Blocks in columns: a recursive type, a parser that calls itself, and a component that uses itself"
menu = "main"
weight = 17
date = 2026-10-21
authors = ["Claude"]
tags = ["rust", "svelte", "parsing", "recursion"]
summary = "A column can now hold blocks: text, an image, a download, a call to action. The same idea shows up four times: a Rust type that contains itself, a parser that runs on its own output, templates rendered one block at a time, and a Svelte component that includes itself."
+++

Until now a column was a title and some Markdown. Now it holds blocks:
text, an image, a text with image, a download, an FAQ, or a call to action.
Hero, Gallery and Columns are left out, because they need the full width.

In the page file it looks like this:

```text
:::: columns
::: column
title = "Join in"
:::

Suggest a topic or tell us where we got it wrong.

::: call-to-action
button_label = "Suggest a topic"
button_link = "mailto:hello@example.com"
:::
::::
```

The outer container has four colons, so its closing line `::::` can't be
mistaken for the end of a block inside it, which is `:::`. That's the
convention of [markdown-it-container](https://github.com/markdown-it/markdown-it-container),
which other Markdown tools follow too. Inside, a `::: column` line starts
each column and holds its settings (the title). The blocks after it belong
to that column, up to the next `::: column` or the end. That's one extra
level of colons, not one per level, so it stays easy to write by hand.

Nesting appears four times in this change, in four forms.

## A type that contains itself

```rust
pub enum Block {
    Text { markdown: String },
    Columns(Columns),
    // …
}

pub struct Columns { pub items: Vec<Column> }

pub struct Column {
    pub title: Option<String>,
    pub blocks: Vec<Block>,
}
```

A `Block` can hold `Columns`, which hold `Column`s, which hold `Block`s.
Rust needs to know how big every type is at compile time. A type that held
itself *directly* (`struct Column { block: Block }` inside `Block`) would be
infinitely big, and the compiler refuses it with "recursive type has
infinite size". The fix is to put the inner value behind a pointer
([`Box`](https://doc.rust-lang.org/book/ch15-01-box.html)), and `Vec` is
one: it keeps its items on the heap, and a `Vec<Block>` itself is always
three machine words, whatever it holds. So this compiles as it is.

Elixir has no such question: data is always referenced, and a map holding a
list of maps of the same shape is unremarkable.

serde, specta and the TypeScript side follow along. The generated
`bindings.ts` has `Column = { title: string | null, blocks?: Block[] }`, a
type that refers back to `Block`.

## A parser that calls itself

The parser in [`page.rs`](../../../crates/qed-core/src/page.rs) used to
split a page into text and `:::` containers, and rejected a container
inside another. Now `parse_body` does one more thing: when it meets
`:::: columns`, it collects the lines up to the matching `::::` (skipping
colons inside code fences) and hands them to itself:

```rust
fn columns_block(start: usize, lines: &[(usize, &str)]) -> Result<Piece, ParseError> {
    let mut items: Vec<Column> = Vec::new();
    for piece in parse_body(lines, true)? {
        match piece {
            Piece::Column(settings, toml) => items.push(Column { title: settings.title, blocks: Vec::new() }),
            Piece::Block(block, source) => match items.last_mut() {
                Some(column) => column.blocks.push(block),
                None => return Err(ParseError::new(start + 1, "start each column with a ::: column line")),
            },
        }
    }
    // …
}
```

The `true` tells the inner call that it's inside columns: there, a
`::: column` line is allowed and comes back as a `Piece::Column` marker,
and blocks that need the full width are refused with a message that says
why ("a hero block can't go inside a column; it needs the full width").
`columns_block` then walks the pieces in order and starts a new column at
each marker.

The error messages are worth more than usual here, because people type this
by hand. Each likely mistake has its own: `::: columns` with three colons
("columns hold blocks, so they use four colons"), text before the first
`::: column`, a `:::: columns` inside columns, a column that's never
closed. Each one points to the right line. A test goes through all of them.

The line numbers still come out right inside columns, because each line
travels with its number (`(usize, &str)`) instead of being counted again.

## Writing it back

The writer has a promise to keep (see [saving without breaking
things](2026-10-21-saving-without-breaking.md)): what didn't change is
written exactly as it was, comments included. So the parser records how
every block was written, and for columns that record has the same shape as
the content:

```rust
pub(crate) enum BlockSource {
    Text,
    Toml(String),
    Columns(Vec<ColumnSource>),
}

pub(crate) struct ColumnSource {
    pub toml: String,
    pub blocks: Vec<BlockSource>,
}
```

`write_blocks` writes a list of blocks, comparing each with the one at the
same position before. For columns it calls `write_column`, which calls
`write_blocks` again for the column's blocks. A test edits one button label
inside a column and checks that a comment next to it, and a
`title = 'Left'  # short` in single quotes, survive.

## Templates, one block at a time

The site's HTML comes from [askama](https://askama.readthedocs.io/)
templates, which are compiled into Rust code. Everything about a template
is fixed at compile time, so a template can't include itself an unknown
number of times. The block markup used to sit in the middle of `page.html`.
Now it has its own template, `block.html`, and each block is rendered to a
string on its own:

```rust
fn render_html(block: &Block, links: &LinkContext, root: &Path, images: &Images, in_column: bool)
    -> Result<String, askama::Error>
{
    let rendered = render_block(block, links, root, images)?;
    // …
    BlockTemplate { block: &rendered, full_sizes, half_sizes }.render()
}
```

For Columns, `render_block` calls `render_html` for each block inside, so
the column template gets finished HTML strings and only has to place them.
The recursion happens in Rust, where it's allowed. `in_column` tells images
that they have less room, so their `sizes` hint (see [responsive
images](2026-10-21-responsive-images.md)) asks for a smaller copy.

## A component that uses itself

The editor works the same way. The list of blocks with its add, move and
remove buttons and each block's fields moved out of `BlockEditor.svelte`
into [`BlockList.svelte`](../../../src/lib/BlockList.svelte). Inside a
Columns block, each column shows a `BlockList` of its own:

```svelte
<script lang="ts">
  // A component may use itself: each column holds a BlockList of its own.
  import BlockList from "$lib/BlockList.svelte";
</script>

<BlockList bind:blocks={column.blocks} {siteRoot} types={COLUMN_BLOCK_TYPES} nested />
```

In Svelte 5 a component can simply import itself; older versions needed a
special `<svelte:self>` tag. `types` limits the Add buttons to the blocks a
column can hold, and `nested` makes everything a bit smaller.
`bind:blocks` passes the column's list down so changes in it are changes
to the page; the Save button in `BlockEditor` sees them like any other edit.

Before, each block in the editor got a made-up id so Svelte could keep the
right Markdown editor with the right block while they move. With lists
inside lists, that wrapper would have been needed at every level. Instead,
the lists are keyed by the block objects themselves:

```svelte
{#each blocks as block, index (block)}
```

A [keyed each block](https://svelte.dev/docs/svelte/each#Keyed-each-blocks)
accepts any value as key, and objects count as the same key only if they
are the same object. Moving a block moves the object, so Svelte moves its
editor along with it.

## Further reading

- The Rust book on [`Box` and recursive types](https://doc.rust-lang.org/book/ch15-01-box.html)
- [markdown-it-container](https://github.com/markdown-it/markdown-it-container),
  where the colon convention comes from
- Svelte: [keyed each blocks](https://svelte.dev/docs/svelte/each#Keyed-each-blocks)
  and [`bind:` on component props](https://svelte.dev/docs/svelte/$bindable)

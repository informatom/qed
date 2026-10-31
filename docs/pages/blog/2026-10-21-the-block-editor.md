+++
title = "The block editor"
description = "The block editor: closures that run once, and components that talk back"
menu = "main"
weight = 7
date = 2026-10-21
authors = ["Claude"]
tags = ["rust", "closures", "svelte", "components", "overtype"]
summary = "q.e.d. can now edit page content: Markdown text with a live editor, hero and call-to-action fields, adding, removing and reordering blocks. Rust contributes FnOnce and into_iter; Svelte brings two-way props, snippets and a JavaScript library wrapped in a component."
+++

Until now, q.e.d. could change a page's settings but not its content. The
block editor fills that gap. Open a page, and its blocks appear as cards:
text blocks with a Markdown editor, hero and call-to-action blocks with
their fields. Blocks can be added, removed and moved up or down, and "Save"
writes the page file, with the same care for hand-written formatting as
before (see the [previous post on saving](2026-10-21-saving-without-breaking.md)).

## One helper, two kinds of change

Saving settings and saving blocks do almost the same thing: read the file
again, change one part, write it safely, read back what was written. The
only difference is *which* part changes. So the shared work is one function
that takes the change as an argument, in
`crates/qed-core/src/site.rs`:

```rust
fn update_page(
    root: &Path,
    file: &Path,
    change: impl FnOnce(&mut PageContent),
) -> Result<Page, SiteError> {
```

and the two callers shrink to one line each:

```rust
update_page(root, file, |content| content.front = front)
update_page(root, file, |content| content.blocks = blocks)
```

`impl FnOnce(&mut PageContent)` means: any closure that takes a mutable
reference to the page content, and that may be called **once**. Rust
distinguishes three kinds of closures by what they do with what they
capture:

- [`Fn`](https://doc.rust-lang.org/std/ops/trait.Fn.html) only reads it, and
  can be called any number of times.
- [`FnMut`](https://doc.rust-lang.org/std/ops/trait.FnMut.html) changes it,
  and can be called repeatedly, one call at a time.
- [`FnOnce`](https://doc.rust-lang.org/std/ops/trait.FnOnce.html) *uses it
  up*, so it can be called only once.

Our closures move `front` or `blocks` into the page content. After that, the
closure doesn't have them anymore; calling it a second time would have
nothing to move. Rust notices that, and `FnOnce` is the honest description.
In Elixir, every function can be called any number of times, because data is
copied or shared, never moved. Rust's ownership makes "can be called once"
a real property, and the compiler checks it.

### Filtering by pattern

Empty text blocks are dropped before saving:

```rust
let blocks = blocks
    .into_iter()
    .filter(|block| !matches!(block, Block::Text { markdown } if markdown.trim().is_empty()))
    .collect();
```

Two details:

- [`into_iter()`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html)
  instead of `iter()`: the blocks are *moved* through the chain, not
  borrowed, because the function owns them and passes them on. `iter()`
  would yield references, and we'd have to clone each block.
- [`matches!`](https://doc.rust-lang.org/std/macro.matches.html) checks a
  value against a pattern and gives `true` or `false`, here with a guard
  (`if …`). It's `match?/2` from Elixir, guard included.

## Components that talk back: `$bindable`

The editor is built from four Svelte components:

- `src/lib/PageEditor.svelte`: the frame with
  the "Content" and "Settings" tabs.
- `src/lib/BlockEditor.svelte`: the block cards.
- `src/lib/MarkdownEditor.svelte`: a
  Markdown editor for any text field.
- `PageSettings.svelte`, from before.

Props normally flow down: the parent passes data, the child displays it.
Sometimes the child needs to send something back. The settings panel used
callbacks for that (`onsaved`). For a value that changes all the time,
like "are there unsaved changes?", Svelte offers two-way props with
[`$bindable`](https://svelte.dev/docs/svelte/$bindable):

```ts
let { page, siteRoot, onsaved, dirty = $bindable(false) }: Props = $props();
```

The parent binds to it like to an input field:

```svelte
<BlockEditor {page} {siteRoot} {onsaved} bind:dirty={contentDirty} />
```

When the block editor sets `dirty = true`, `contentDirty` in the parent
changes too. That's how the tab shows a dot for unsaved changes, and how the
app asks "Discard them?" before you switch to another page.

## A JavaScript library in a component

The Markdown editor is [OverType](https://github.com/panphora/overtype), a
small library that shows Markdown formatting while you type. It knows
nothing about Svelte; it takes a DOM element and builds its editor inside.
The component's job is to connect the two worlds:

```ts
onMount(() => {
  [editor] = new OverType(element, {
    value,
    toolbar: true,
    autoResize: true,
    onChange: (text: string) => {
      value = text;
    },
  });
  return () => editor?.destroy();
});
```

- `element` is the `<div>` from `bind:this`, as in the
  [new-page dialog](2026-10-21-new-pages.md).
- `onChange` copies every edit into `value`, which is `$bindable`, so the
  parent's block updates.
- A function returned from
  [`onMount`](https://svelte.dev/docs/svelte/lifecycle-hooks#onMount) runs
  when the component is removed: here it tears the editor down. Svelte's
  version of `terminate/2`, for UI pieces.

The other direction, a value changed from outside (say, by "Discard"), is an
effect:

```ts
$effect(() => {
  if (editor && value !== editor.getValue()) editor.setValue(value);
});
```

## Ids that exist only in the UI

Blocks in a page file have no ids, and they don't need any. But the UI does:
when you move a block up, Svelte must know it's the *same* block in a new
place, so its Markdown editor moves along instead of being rebuilt with the
wrong text. So the editor wraps each block with an id when it loads them:

```ts
function wrap(blocks: Block[]): Item[] {
  return $state.snapshot(blocks).map((block) => ({ id: crypto.randomUUID(), block }));
}
```

and uses it as the key in `{#each items as item, index (item.id)}`. The ids
are never saved. Phoenix streams need DOM ids for the same reason.

Moving a block is then plain array surgery:

```ts
function move(index: number, by: number) {
  const [item] = items.splice(index, 1);
  items.splice(index + by, 0, item);
}
```

Svelte 5's `$state` tracks changes inside arrays and objects too, so
`splice` on `items` is enough to update the page. No `assign`, no copying.

## Snippets: small reusable markup

Every field label in the block cards has the same shape: a name, and maybe a
hint. A [snippet](https://svelte.dev/docs/svelte/snippet) is a piece of
markup with parameters, defined right in the component:

```svelte
{#snippet field(label: string, hint: string = "")}
  <span class="text-sm font-medium">
    {label}
    {#if hint}<span class="font-normal text-base-content/50">· {hint}</span>{/if}
  </span>
{/snippet}

{@render field("Image", "file name in media/")}
```

It's like a private function component in Phoenix (`defp label(assigns)`),
without a separate file.

## Further reading

- [The Rust book, chapter 13.1: closures and the `Fn` traits](https://doc.rust-lang.org/book/ch13-01-closures.html)
- [Svelte: `$bindable`](https://svelte.dev/docs/svelte/$bindable) and
  [snippets](https://svelte.dev/docs/svelte/snippet)
- [OverType on GitHub](https://github.com/panphora/overtype)

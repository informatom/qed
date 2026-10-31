+++
title = "Dragging blocks and pages"
description = "Moving blocks and pages by dragging a handle, with pointer events, pointer capture and requestAnimationFrame instead of HTML drag and drop"
menu = "main"
weight = 41
date = 2026-10-27T21:12:00
authors = ["Claude"]
tags = ["svelte", "typescript", "tauri", "ui"]
summary = "Blocks and pages now move by dragging a small grip handle (#043, #120). Browsers have a drag and drop API, but this uses plain pointer events: they work the same in every webview, and pointer capture plus requestAnimationFrame cover the rest."
+++

Until now a block moved one step at a time, with the up and down
arrows. Now each block has a grip handle (⋮⋮) at the left of its header:
press on it, drag, and let go where the block should go (#043). A line in
the gap shows where it will land; near the top or bottom of the window
the page scrolls by itself; Escape cancels.

## Why a handle

The idea came from the owner: drag only from a handle, not from anywhere
on the block. A block is full of things that use the mouse themselves:
Markdown editors where dragging selects text, inputs, image pickers. A
handle keeps the two apart, so a drag is never mistaken for a selection.

## Pointer events, not drag and drop

Browsers have an [HTML drag and drop
API](https://developer.mozilla.org/en-US/docs/Web/API/HTML_Drag_and_Drop_API)
(`draggable`, `dragstart`, `drop`). It's made for dragging between
windows and apps, and that's the problem here: a Tauri window handles
files dropped onto it, and on some systems (Windows in particular) that
takes the drag events away from the page.

[Pointer events](https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events)
are lower level: `pointerdown`, `pointermove`, `pointerup`, the same for
mouse, pen and touch. `BlockList.svelte` puts all three on the handle:

```svelte
<button type="button" class="… touch-none select-none cursor-grab"
  onpointerdown={(e) => dragStart(e, index)} onpointermove={dragMove} onpointerup={dragEnd}
  onpointercancel={() => (drag = null)}><GripVertical class="size-4" /></button>
```

Normally, `pointermove` only reaches an element while the pointer is over
it, and a handle is small. [Pointer
capture](https://developer.mozilla.org/en-US/docs/Web/API/Element/setPointerCapture)
changes that: after `handle.setPointerCapture(event.pointerId)` in
`dragStart`, every move and the final `pointerup` go to the handle,
wherever the pointer is. `touch-none` (CSS `touch-action: none`) stops a
touch screen from scrolling instead, and `event.preventDefault()` stops
the browser from selecting text on the way.

## Where it lands

The state is one small object, `drag = { from, gap }`: the block being
dragged and the gap it would drop into, 0 for above the first block,
`blocks.length` for below the last. Finding the gap is one line of
thinking: the pointer is below every block whose middle is above it.

```ts
drag.gap = cards.slice(0, blocks.length).filter((card) => {
  const box = card.getBoundingClientRect();
  return box.top + box.height / 2 < pointerY;
}).length;
```

`cards` holds the blocks' elements, filled by `bind:this={cards[index]}`
in the `{#each}`. Dropping into the gap right above or below the block
itself changes nothing, so the `moves` value (`$derived`) is false there
and no line shows.

The line is placed with `position: absolute` in the space between two
blocks. If it took up room of its own, the blocks below it would shift,
their middles would move, the gap would change, and the line would jump
back and forth.

## Scrolling while dragging

A page is often taller than the window. While dragging, `autoScroll` runs
once per frame:

```ts
function autoScroll() {
  if (!drag) return;
  if (edgeScroll(scroller, pointerY)) findGap();
  requestAnimationFrame(autoScroll);
}
```

`edgeScroll` (in `src/lib/dragScroll.ts`) scrolls when the pointer is
within 60 pixels of the top or bottom edge, faster the closer it gets,
and says whether it did: then the blocks have moved under the pointer and
the gap is worked out again.

[`requestAnimationFrame`](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame)
calls a function once, just before the browser draws the next frame,
usually 60 times a second. Calling it again at the end makes a loop that
runs in step with the screen and stops by itself once `drag` is `null`.
A `setInterval` would work too, but it runs out of step with drawing and
keeps going in a hidden window. In Elixir, the closest thing is a process
that sends itself a message with `Process.send_after` and handles it in
`handle_info`: a loop driven from outside, one step at a time.

Each list handles its own blocks, so a block inside a column moves
within that column. Moving blocks between columns or in and out of them
is left for later.

## The page tree

Pages in the Pages tab have the same handle (#120). A page can be dropped
in two ways: between two pages, which changes the order, or onto a page,
which moves it below that page. Each row is split in three: the top
third means the gap above it, the bottom third the gap below, the middle
the page itself, shown with a ring.

The order is the pages' `weight`, which menus, page lists and the sidebar
already sort by. After a drop the pages below the same parent get 10, 20,
30, … (`site::set_order` in Rust), so there's room to put a page between
two by hand. For this the tree now shows the site's real order, not the
addresses in alphabetical order. Posts keep their place by date and can't
be reordered; they can still be dropped onto another page.

Moving below another page changes the page's address, so the app asks
first and then uses the same `move_page` as the Move dialog: links are
updated and the old address redirects.

What a drop means is decided in `src/lib/pageTree.ts` by `planDrop`, a
plain function with no Svelte in it: it gets the pages, the dragged page,
the row and the zone, and returns where the page goes and the new order,
or `null` when the drop would change nothing. Keeping that apart from the
pointer handling made it easy to try out on its own with a few sample
pages. In Elixir terms it's the pure function you'd test, and the Svelte
code is the process around it that handles the events.

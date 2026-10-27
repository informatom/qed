+++
title = "An image viewer"
description = "Images open big in the app's Media tab: a modal dialog, arrow keys, and clicks that fall through"
menu = "main"
weight = 38
date = 2026-10-27
authors = ["Claude"]
tags = ["svelte", "css", "dialog"]
summary = "After the sites, the app: a click on an image in the Media tab now shows it as large as the window allows, and the arrow keys step through the others (#118). A modal dialog does most of the work; pointer-events lets clicks beside the image reach the backdrop."
+++

Generated sites open images big with the browser's popover and no
JavaScript ([Pop-ups without scripts](2026-10-27-images-open-big.md)).
The app has no such rule: it's a Svelte program anyway. So the Media
tab's viewer, `src/lib/ImageViewer.svelte`, is a modal
[`<dialog>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/dialog)
like the text viewer next to it. `dialog.showModal()` puts it above
the app, keeps the keyboard inside it, and closes it on Esc, all
by itself.

## Stepping with the arrow keys

The viewer gets the list of images the tab shows (after search and in
the current folder) and the one that was clicked. Which one is shown is
a single number:

```svelte
let index = $state(untrack(() => Math.max(0, images.findIndex((image) => image.name === start))));
let image = $derived(images[index]);

function step(by: number) {
  index = (index + by + images.length) % images.length;
}
```

`image` is `$derived`, so changing `index` is all a step does: Svelte
shows the new image, its name and its size. Adding `images.length`
before the `%` keeps the number positive when stepping back from the
first image, so it wraps round to the last one. JavaScript's `%` keeps
the sign of the left side (`-1 % 5` is `-1`), just like Elixir's
[`rem/2`](https://hexdocs.pm/elixir/Kernel.html#rem/2); Elixir's
[`Integer.mod/2`](https://hexdocs.pm/elixir/Integer.html#mod/2) would
give `4` straight away.

`start` is read inside
[`untrack`](https://svelte.dev/docs/svelte/svelte#untrack): the
viewer takes it once, as a starting point. Without `untrack`, Svelte
would warn that a prop is only captured at its initial value, which is
exactly what's meant here.

The `onkeydown` handler on the dialog catches ← and →, and calls
`preventDefault()` only for those, so Esc and Tab keep working as
usual.

## Clicks that fall through

A click beside the image should close the viewer, like on the sites.
daisyUI's modal has a backdrop for that: a button behind the content,
in the same grid cell. But the viewer's content fills the whole window,
so it covers the backdrop, and every click would land on the content.

[`pointer-events`](https://developer.mozilla.org/en-US/docs/Web/CSS/pointer-events)
solves that:

```svelte
<div class="pointer-events-none col-start-1 row-start-1 flex h-screen w-screen …">
  <button class="pointer-events-auto …" onclick={() => dialog.close()}>
    <img …>
  </button>
  <div class="pointer-events-auto …"><!-- name, size, arrows, close --></div>
</div>
```

`pointer-events-none` makes the big container invisible to the mouse:
clicks go through it to whatever is underneath, here the backdrop,
which closes the dialog. Its children switch it back on with
`pointer-events-auto`, so the image and the buttons below it still get
their clicks. Visually nothing changes; only where clicks land.

`col-start-1 row-start-1` puts the content into the same cell as the
backdrop. daisyUI places the backdrop there explicitly, and CSS grid
would otherwise move the content to the next free cell, below it.

## A title that shrank to one pixel

Right after the viewers came the first bug report: in the text viewer,
showing the long source of an old slide show, the title above the text
was one pixel high. The viewer is a column (`flex flex-col`) of limited
height, with the title, the text and the Close button:

```svelte
<div class="modal-box flex max-h-[85vh] flex-col gap-3">
  <h2 class="truncate …">static/boja-2017/index.html</h2>
  <pre class="min-h-0 flex-1 overflow-auto …">{text}</pre>
  …
</div>
```

When the content is taller than the box, flexbox shrinks the items to
make them fit. It normally stops at an item's content: an item's
smallest size is `min-height: auto`, as tall as what's inside. But
there is an exception. For an item that hides its overflow, `auto`
means zero. Tailwind's `truncate` sets `overflow: hidden` (to cut long
names off with "…"), so the title had no lower limit and gave up all
its height to the text below.
([MDN on the automatic minimum size](https://developer.mozilla.org/en-US/docs/Web/CSS/min-height#auto))

The fix is one class: `shrink-0` (`flex-shrink: 0`) tells flexbox not to
shrink the title at all, and the text area, which has `min-h-0` on
purpose and scrolls, takes all the shrinking. The PDF viewer got the
same class.

The same report found a font nobody had chosen. Tailwind's `font-mono`
lists Mac and Windows fonts (`ui-monospace`, `Menlo`, `Consolas`, …),
and on Linux none of them exists, so the webview fell back to DejaVu
Sans Mono, which looks bold at small sizes. `src/app.css` now sets its
own list in Tailwind's `@theme`, with Linux fonts (`Ubuntu Mono`,
`Noto Sans Mono`) before Consolas, because Linux's fontconfig would
otherwise quietly map Consolas to DejaVu Sans Mono again.

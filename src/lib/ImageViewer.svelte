<script lang="ts">
  import { onMount, untrack } from "svelte";
  import ChevronLeft from "@tabler/icons-svelte-runes/icons/chevron-left";
  import ChevronRight from "@tabler/icons-svelte-runes/icons/chevron-right";
  import Crop from "@tabler/icons-svelte-runes/icons/crop";
  import FocusCentered from "@tabler/icons-svelte-runes/icons/focus-centered";
  import X from "@tabler/icons-svelte-runes/icons/x";
  import { formatSize, libraryUrl, type Focus, type Library, type MediaFile } from "$lib/site";

  interface Props {
    siteRoot: string;
    library: Library;
    /** The images the arrows step through, in the order shown. */
    images: MediaFile[];
    /** The one to start with. */
    start: string;
    onclose: () => void;
    /** Saves where an image is cut to fit (`null`: the middle); offers Crop when set. */
    onfocus?: (name: string, focus: Focus | null) => Promise<void>;
  }

  let { siteRoot, library, images, start, onclose, onfocus }: Props = $props();
  let dialog: HTMLDialogElement;
  let index = $state(untrack(() => Math.max(0, images.findIndex((image) => image.name === start))));
  let image = $derived(images[index]);
  /** Choosing where the image is cut: a click on it sets the spot. */
  let cropping = $state(false);
  /** Focus chosen here, shown at once while it's being saved. */
  let chosen = $state<Record<string, Focus | null>>({});
  let focus = $derived(image ? (image.name in chosen ? chosen[image.name] : (image.focus ?? null)) : null);
  let position = $derived(focus ? `${focus.x}% ${focus.y}%` : "50% 50%");
  let error = $state<string | null>(null);

  function step(by: number) {
    index = (index + by + images.length) % images.length;
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.key === "ArrowLeft") step(-1);
    else if (event.key === "ArrowRight") step(1);
    else return;
    event.preventDefault();
  }

  async function setFocus(next: Focus | null) {
    if (!image || !onfocus) return;
    const name = image.name;
    chosen[name] = next;
    try {
      await onfocus(name, next);
      error = null;
    } catch (e) {
      error = String(e);
    }
  }

  /** The clicked spot, in whole percent of the image. */
  function pick(event: MouseEvent) {
    const box = (event.currentTarget as HTMLElement).querySelector("img")!.getBoundingClientRect();
    const percent = (at: number, from: number, size: number) =>
      Math.round(Math.min(100, Math.max(0, ((at - from) / size) * 100)));
    setFocus({ x: percent(event.clientX, box.left, box.width), y: percent(event.clientY, box.top, box.height) });
  }

  onMount(() => dialog.showModal());
</script>

<!-- Esc closes a modal dialog by itself; so does a click on the image or beside it. -->
<dialog bind:this={dialog} id="image-viewer" class="modal bg-black/85" onclose={onclose} {onkeydown}>
  {#if image}
    <!-- Clicks beside the image fall through to the backdrop, which closes. -->
    <div class="pointer-events-none col-start-1 row-start-1 flex h-screen w-screen flex-col items-center justify-center gap-3 p-6">
      <div class="flex items-center gap-6">
        <!-- The button is as big as the image, so its focus ring hugs it; the
             image leaves room for the bar below (about 6rem), and in crop
             mode for the square beside it. -->
        <button type="button" id="image-viewer-image"
          class={["pointer-events-auto relative block rounded-box outline-offset-4 focus:outline-none focus-visible:outline-2",
            cropping ? "cursor-crosshair" : "cursor-zoom-out"]}
          title={cropping ? "Click the spot that must stay visible" : "Close"}
          onclick={(event) => (cropping ? pick(event) : dialog.close())}>
          <img src={libraryUrl(siteRoot, library, image.name)} alt={image.name}
            class={["block max-h-[calc(100vh-6rem)] rounded-box object-contain shadow-2xl",
              cropping ? "max-w-[calc(100vw-20rem)]" : "max-w-[calc(100vw-3rem)]"]} />
          {#if cropping}
            <span id="image-viewer-focus" aria-hidden="true"
              class="pointer-events-none absolute size-6 -translate-1/2 rounded-full border-2 border-white shadow-[0_0_0_2px_rgb(0_0_0/0.6)]"
              style:left="{focus?.x ?? 50}%" style:top="{focus?.y ?? 50}%"></span>
          {/if}
        </button>
        {#if cropping}
          <div class="pointer-events-auto flex w-56 flex-col gap-2 text-sm text-white/80">
            <img id="image-viewer-square" src={libraryUrl(siteRoot, library, image.name)} alt="How it's cut to a square"
              class="aspect-square w-full rounded-box object-cover shadow-2xl" style:object-position={position} />
            <p>How it's cut to a square in galleries and tiles. Click the spot that must stay visible.</p>
            {#if error}<p id="image-viewer-error" class="text-error">{error}</p>{/if}
          </div>
        {/if}
      </div>
      <div class="pointer-events-auto flex items-center gap-3 text-sm text-white">
        {#if images.length > 1}
          <button type="button" id="image-viewer-previous" class="btn btn-circle btn-ghost btn-sm text-white"
            aria-label="Previous image" title="Previous (←)" onclick={() => step(-1)}><ChevronLeft class="size-5" /></button>
        {/if}
        <span class="font-mono">{image.name}</span>
        <span class="text-white/60">
          {#if image.width && image.height}{image.width}×{image.height} · {/if}{formatSize(image.size)}
          {#if images.length > 1} · {index + 1} of {images.length}{/if}
        </span>
        {#if images.length > 1}
          <button type="button" id="image-viewer-next" class="btn btn-circle btn-ghost btn-sm text-white"
            aria-label="Next image" title="Next (→)" onclick={() => step(1)}><ChevronRight class="size-5" /></button>
        {/if}
        {#if onfocus}
          <button type="button" id="image-viewer-crop" aria-pressed={cropping}
            class={["btn btn-sm", cropping ? "btn-primary" : "btn-ghost text-white"]}
            title="Choose where the image is cut to a square" onclick={() => (cropping = !cropping)}>
            <Crop class="size-4" /> Crop{#if focus && !cropping}<span class="text-white/60">· {focus.x}% {focus.y}%</span>{/if}
          </button>
          {#if cropping}
            <button type="button" id="image-viewer-middle" class="btn btn-ghost btn-sm text-white" disabled={!focus}
              title="Cut around the middle again" onclick={() => setFocus(null)}>
              <FocusCentered class="size-4" /> Middle
            </button>
          {/if}
        {/if}
        <button type="button" id="image-viewer-close" class="btn btn-circle btn-ghost btn-sm text-white"
          aria-label="Close" title="Close (Esc)" onclick={() => dialog.close()}><X class="size-5" /></button>
      </div>
    </div>
  {/if}
  <form method="dialog" class="modal-backdrop"><button>close</button></form>
</dialog>

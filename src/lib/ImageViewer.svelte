<script lang="ts">
  import { onMount, untrack } from "svelte";
  import ChevronLeft from "@lucide/svelte/icons/chevron-left";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import X from "@lucide/svelte/icons/x";
  import { formatSize, libraryUrl, type Library, type MediaFile } from "$lib/site";

  interface Props {
    siteRoot: string;
    library: Library;
    /** The images the arrows step through, in the order shown. */
    images: MediaFile[];
    /** The one to start with. */
    start: string;
    onclose: () => void;
  }

  let { siteRoot, library, images, start, onclose }: Props = $props();
  let dialog: HTMLDialogElement;
  let index = $state(untrack(() => Math.max(0, images.findIndex((image) => image.name === start))));
  let image = $derived(images[index]);

  function step(by: number) {
    index = (index + by + images.length) % images.length;
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.key === "ArrowLeft") step(-1);
    else if (event.key === "ArrowRight") step(1);
    else return;
    event.preventDefault();
  }

  onMount(() => dialog.showModal());
</script>

<!-- Esc closes a modal dialog by itself; so does a click on the image or beside it. -->
<dialog bind:this={dialog} id="image-viewer" class="modal bg-black/85" onclose={onclose} {onkeydown}>
  {#if image}
    <!-- Clicks beside the image fall through to the backdrop, which closes. -->
    <div class="pointer-events-none col-start-1 row-start-1 flex h-screen w-screen flex-col items-center justify-center gap-3 p-6">
      <!-- The button is as big as the image, so its focus ring hugs it; the
           image leaves room for the bar below (about 6rem). -->
      <button type="button" id="image-viewer-image"
        class="pointer-events-auto block cursor-zoom-out rounded-box outline-offset-4 focus:outline-none focus-visible:outline-2"
        title="Close" onclick={() => dialog.close()}>
        <img src={libraryUrl(siteRoot, library, image.name)} alt={image.name}
          class="block max-h-[calc(100vh-6rem)] max-w-[calc(100vw-3rem)] rounded-box object-contain shadow-2xl" />
      </button>
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
        <button type="button" id="image-viewer-close" class="btn btn-circle btn-ghost btn-sm text-white"
          aria-label="Close" title="Close (Esc)" onclick={() => dialog.close()}><X class="size-5" /></button>
      </div>
    </div>
  {/if}
  <form method="dialog" class="modal-backdrop"><button>close</button></form>
</dialog>

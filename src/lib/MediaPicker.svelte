<script lang="ts">
  import { onMount } from "svelte";
  import MediaLibrary from "$lib/MediaLibrary.svelte";
  import type { Library, MediaKind } from "$lib/site";

  interface Props {
    siteRoot: string;
    library?: Library;
    only?: MediaKind;
    onpick: (name: string) => void;
    onclose: () => void;
  }

  let { siteRoot, library = "media", only, onpick, onclose }: Props = $props();
  let dialog: HTMLDialogElement;

  onMount(() => dialog.showModal());

  function pick(name: string) {
    // Report the choice before closing: closing may tell the parent to
    // forget what the picker was opened for.
    onpick(name);
    dialog.close();
  }
</script>

<dialog bind:this={dialog} id="media-picker" class="modal" onclose={onclose}>
  <div class="modal-box flex max-h-[85vh] w-11/12 max-w-6xl flex-col gap-4">
    <h2 class="text-xl font-semibold">Choose {only === "image" ? "an image" : only === "audio" ? "an audio file" : "a file"}</h2>
    <div class="min-h-0 flex-1 overflow-y-auto">
      <MediaLibrary {siteRoot} {library} {only} onpick={pick} />
    </div>
    <div class="modal-action mt-0">
      <button type="button" class="btn btn-ghost" onclick={() => dialog.close()}>Cancel</button>
    </div>
  </div>
  <form method="dialog" class="modal-backdrop"><button>close</button></form>
</dialog>

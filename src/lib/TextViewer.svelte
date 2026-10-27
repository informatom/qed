<script lang="ts">
  import { onMount } from "svelte";
  import { commands, type Library } from "$lib/site";

  interface Props {
    siteRoot: string;
    library: Library;
    /** Path inside the library folder: `meetup7/index.html`. */
    name: string;
    onclose: () => void;
  }

  let { siteRoot, library, name, onclose }: Props = $props();
  let dialog: HTMLDialogElement;
  let text = $state<string | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    dialog.showModal();
    try {
      text = await commands.readLibraryFile(siteRoot, library, name);
    } catch (e) {
      error = String(e);
    }
  });
</script>

<dialog bind:this={dialog} id="text-viewer" class="modal" onclose={onclose}>
  <div class="modal-box flex max-h-[85vh] w-11/12 max-w-6xl flex-col gap-3">
    <h2 class="truncate font-mono text-base font-semibold" title={name}>{library}/{name}</h2>
    {#if error}
      <div role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
    {:else if text === null}
      <span class="loading loading-spinner"></span>
    {:else}
      <!-- Read only: the text is shown, never interpreted as HTML. -->
      <pre id="text-viewer-content"
        class="min-h-0 flex-1 overflow-auto rounded-box bg-base-200 p-3 font-mono text-xs leading-relaxed whitespace-pre">{text}</pre>
    {/if}
    <div class="modal-action mt-0">
      <button type="button" class="btn" onclick={() => dialog.close()}>Close</button>
    </div>
  </div>
  <form method="dialog" class="modal-backdrop"><button>close</button></form>
</dialog>

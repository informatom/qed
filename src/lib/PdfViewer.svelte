<script lang="ts">
  import { onMount } from "svelte";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import { commands, libraryUrl, type Library } from "$lib/site";

  interface Props {
    siteRoot: string;
    library: Library;
    /** Path inside the library folder: `menu.pdf`. */
    name: string;
    onclose: () => void;
  }

  let { siteRoot, library, name, onclose }: Props = $props();
  let dialog: HTMLDialogElement;
  let error = $state<string | null>(null);

  onMount(() => dialog.showModal());

  async function openInApp() {
    error = null;
    try {
      await commands.openLibraryFile(siteRoot, library, name);
    } catch (e) {
      error = String(e);
    }
  }
</script>

<!-- The webview shows PDFs itself: PDF.js in WebKitGTK, Edge's viewer in
     WebView2, PDFKit on macOS. "Open in PDF app" is there for printing, or
     a system without a viewer. -->
<dialog bind:this={dialog} id="pdf-viewer" class="modal" onclose={onclose}>
  <div class="modal-box flex h-[92vh] w-11/12 max-w-6xl flex-col gap-3 p-4">
    <h2 class="shrink-0 truncate font-mono text-base font-semibold" title={name}>{library}/{name}</h2>
    <iframe id="pdf-viewer-frame" src={libraryUrl(siteRoot, library, name)} title={name}
      class="min-h-0 flex-1 rounded-box border border-base-300 bg-base-200"></iframe>
    {#if error}
      <div role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
    {/if}
    <div class="modal-action mt-0">
      <button type="button" id="pdf-viewer-open" class="btn" onclick={openInApp}>
        <ExternalLink class="size-4" /> Open in PDF app
      </button>
      <button type="button" id="pdf-viewer-close" class="btn btn-primary" onclick={() => dialog.close()}>Close</button>
    </div>
  </div>
  <form method="dialog" class="modal-backdrop"><button>close</button></form>
</dialog>

<script lang="ts">
  import { untrack } from "svelte";
  import BlockList from "$lib/BlockList.svelte";
  import { commands, normalizeBlock, type Block, type Page } from "$lib/site";

  interface Props {
    page: Page;
    siteRoot: string;
    onsaved: (page: Page) => void;
    /** Blocks to start with instead of the saved ones (an earlier version). */
    start?: Block[];
    /** Tells the parent whether there are unsaved changes. */
    dirty?: boolean;
  }

  let { page, siteRoot, onsaved, start, dirty = $bindable(false) }: Props = $props();

  // An editable copy. BlockList keys blocks by the objects themselves, so
  // Svelte can tell them apart while they move (the Markdown editors keep
  // their state); `$state.snapshot` makes the copy plain.
  let blocks = $state<Block[]>(untrack(() => $state.snapshot(start ?? page.blocks)));
  let saving = $state(false);
  let error = $state<string | null>(null);

  let edited = $derived(blocks.map(normalizeBlock));
  let changed = $derived(JSON.stringify(edited) !== JSON.stringify(page.blocks.map(normalizeBlock)));

  // Keep the parent informed.
  $effect(() => {
    dirty = changed;
  });

  function discard() {
    blocks = $state.snapshot(page.blocks);
    error = null;
  }

  async function save() {
    saving = true;
    error = null;
    try {
      const saved = await commands.savePageBlocks(siteRoot, page.file, edited);
      // The file may read back slightly differently (empty text dropped,
      // neighbouring text joined), so show what was actually saved.
      blocks = $state.snapshot(saved.blocks);
      onsaved(saved);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<div id="block-editor" class="flex flex-col gap-4">
  <BlockList bind:blocks {siteRoot} />

  <div class="flex flex-wrap items-center justify-end gap-2">
    {#if error}
      <span id="blocks-error" role="alert" class="me-auto text-sm text-error">{error}</span>
    {/if}
    {#if changed}
      <span class="text-sm text-warning">Unsaved changes</span>
      <button type="button" id="discard-blocks" class="btn btn-ghost" onclick={discard} disabled={saving}>
        Discard
      </button>
    {/if}
    <button type="button" id="save-blocks" class="btn btn-primary" onclick={save} disabled={!changed || saving}>
      {#if saving}<span class="loading loading-spinner loading-sm"></span>{/if}
      Save
    </button>
  </div>
</div>

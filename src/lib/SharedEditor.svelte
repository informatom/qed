<script lang="ts">
  import { untrack } from "svelte";
  import { ask } from "@tauri-apps/plugin-dialog";
  import Trash from "@tabler/icons-svelte-runes/icons/trash";
  import X from "@tabler/icons-svelte-runes/icons/x";
  import BlockEditor from "$lib/BlockEditor.svelte";
  import { provideSharedBlocks, sharedBlocks } from "$lib/sharedBlocks";
  import { commands, pageLabel, sharedName, type Page } from "$lib/site";

  interface Props {
    siteRoot: string;
    piece: Page;
    /** URL paths of the pages (and "(footer)") that show it. */
    usedOn: string[];
    onsaved: (piece: Page) => void;
    ondeleted: () => void;
    onclose: () => void;
    dirty?: boolean;
  }

  let { siteRoot, piece, usedOn, onsaved, ondeleted, onclose, dirty = $bindable(false) }: Props = $props();

  let name = $derived(sharedName(piece));

  // The blocks in here can't offer Shared blocks or "Make shared".
  const outer = sharedBlocks();
  provideSharedBlocks({
    get pieces() {
      return outer?.pieces ?? [];
    },
    insidePiece: true,
    get podcast() {
      return outer?.podcast ?? false;
    },
    get contributors() {
      return outer?.contributors ?? [];
    },
    open: (other) => outer?.open(other),
    make: () => Promise.reject(new Error("a shared block can't hold another one")),
  });

  // The parent re-creates this editor for another piece, so the first title is enough.
  let title = $state(untrack(() => piece.front.title));
  let renaming = $state(false);
  let error = $state<string | null>(null);

  async function rename() {
    renaming = true;
    error = null;
    try {
      const saved = await commands.renameShared(siteRoot, name, title);
      title = saved.front.title;
      onsaved(saved);
    } catch (e) {
      error = String(e);
    } finally {
      renaming = false;
    }
  }

  async function remove() {
    const sure = await ask(`Delete the shared block “${piece.front.title}”?`, {
      title: "Delete shared block",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Keep it",
    });
    if (!sure) return;
    try {
      await commands.deleteShared(siteRoot, name);
      ondeleted();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<section id="shared-editor" class="flex min-w-0 flex-col gap-4 rounded-box bg-base-100 p-4 shadow-sm">
  <div class="flex items-start gap-3">
    <div class="flex flex-1 flex-col gap-2">
      <form class="flex flex-wrap items-center gap-2"
        onsubmit={(e) => {
          e.preventDefault();
          rename();
        }}>
        <input id="shared-title" class="input input-ghost min-w-48 flex-1 px-1 text-xl font-semibold" aria-label="Name of the shared block"
          bind:value={title} />
        {#if title.trim() !== piece.front.title}
          <button type="submit" class="btn btn-sm" disabled={!title.trim() || renaming}>Rename</button>
        {/if}
      </form>
      <p class="text-sm text-base-content/60">
        The same blocks on every page that shows it: change them here once.
        {#if usedOn.length}
          Shown on {usedOn.map(pageLabel).join(", ")}.
        {:else}
          No page shows it yet: add a Shared block to a page and pick it.
        {/if}
        Link pages as <code>/impressum</code>, since it can be on any page. Saved in <code>{piece.file}</code>.
      </p>
      {#if error}<p id="shared-error" role="alert" class="text-sm text-error">{error}</p>{/if}
    </div>
    <button type="button" id="delete-shared" class="btn btn-ghost btn-sm btn-square text-error" disabled={usedOn.length > 0}
      title={usedOn.length ? "Still shown on a page; remove it there first" : "Delete this shared block"}
      aria-label="Delete shared block" onclick={remove}><Trash class="size-4" /></button>
    <button type="button" class="btn btn-ghost btn-sm btn-square" aria-label="Close" onclick={onclose}><X class="size-4" /></button>
  </div>
  <BlockEditor saved={piece.blocks} {siteRoot} bind:dirty
    persist={async (blocks) => {
      const saved = await commands.saveShared(siteRoot, name, blocks);
      onsaved(saved);
      return saved.blocks;
    }} />
</section>

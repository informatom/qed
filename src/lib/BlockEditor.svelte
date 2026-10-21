<script lang="ts">
  import { untrack } from "svelte";
  import MarkdownEditor from "$lib/MarkdownEditor.svelte";
  import {
    BLOCK_NAMES,
    newBlock,
    normalizeBlock,
    savePageBlocks,
    type Block,
    type BlockType,
    type Page,
  } from "$lib/site";

  interface Props {
    page: Page;
    siteRoot: string;
    onsaved: (page: Page) => void;
    /** Tells the parent whether there are unsaved changes. */
    dirty?: boolean;
  }

  let { page, siteRoot, onsaved, dirty = $bindable(false) }: Props = $props();

  // Each block gets an id that lives only in the UI, so Svelte can tell blocks
  // apart while they move around (the Markdown editors keep their state).
  interface Item {
    id: string;
    block: Block;
  }

  function wrap(blocks: Block[]): Item[] {
    return $state.snapshot(blocks).map((block) => ({ id: crypto.randomUUID(), block }));
  }

  let items = $state<Item[]>(untrack(() => wrap(page.blocks)));
  let saving = $state(false);
  let error = $state<string | null>(null);

  let edited = $derived(items.map((item) => normalizeBlock(item.block)));
  let changed = $derived(JSON.stringify(edited) !== JSON.stringify(page.blocks.map(normalizeBlock)));

  // Keep the parent informed.
  $effect(() => {
    dirty = changed;
  });

  const TYPES = Object.keys(BLOCK_NAMES) as BlockType[];

  function add(type: BlockType) {
    items.push({ id: crypto.randomUUID(), block: newBlock(type) });
  }

  function remove(index: number) {
    items.splice(index, 1);
  }

  function move(index: number, by: number) {
    const [item] = items.splice(index, 1);
    items.splice(index + by, 0, item);
  }

  function discard() {
    items = wrap(page.blocks);
    error = null;
  }

  async function save() {
    saving = true;
    error = null;
    try {
      const saved = await savePageBlocks(siteRoot, page.file, edited);
      // The file may read back slightly differently (empty text dropped,
      // neighbouring text joined), so show what was actually saved.
      items = wrap(saved.blocks);
      onsaved(saved);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

{#snippet field(label: string, hint: string = "")}
  <span class="text-sm font-medium">
    {label}
    {#if hint}<span class="font-normal text-base-content/50">· {hint}</span>{/if}
  </span>
{/snippet}

<div id="block-editor" class="flex flex-col gap-4">
  {#each items as item, index (item.id)}
    <section id="block-{index}" class="@container card border border-base-300 bg-base-100">
      <div class="card-body gap-3 p-4">
        <header class="flex items-center gap-2">
          <span class="badge badge-soft">{BLOCK_NAMES[item.block.type]}</span>
          <div class="flex-1"></div>
          <button type="button" class="btn btn-ghost btn-xs btn-square" title="Move up"
            disabled={index === 0} onclick={() => move(index, -1)}>↑</button>
          <button type="button" class="btn btn-ghost btn-xs btn-square" title="Move down"
            disabled={index === items.length - 1} onclick={() => move(index, 1)}>↓</button>
          <button type="button" class="btn btn-ghost btn-xs text-error" onclick={() => remove(index)}>
            Remove
          </button>
        </header>

        {#if item.block.type === "text"}
          <MarkdownEditor bind:value={item.block.markdown} placeholder="Write in Markdown…" />
        {:else}
          <div class="grid gap-3 @2xl:grid-cols-2">
            {#if item.block.type === "hero"}
              <label class="flex flex-col gap-1">
                {@render field("Image", "file name in media/")}
                <input class="input w-full" bind:value={item.block.image} placeholder="team.jpg" />
              </label>
            {/if}
            <label class={["flex flex-col gap-1", item.block.type !== "hero" && "@2xl:col-span-2"]}>
              {@render field("Headline")}
              <input class="input w-full" bind:value={item.block.headline} />
            </label>
            <div class="flex flex-col gap-1 @2xl:col-span-2">
              {@render field("Text", "Markdown")}
              <MarkdownEditor
                bind:value={() => item.block.type !== "text" ? (item.block.text ?? "") : "",
                (text) => { if (item.block.type !== "text") item.block.text = text; }}
                minHeight="80px"
              />
            </div>
            <label class="flex flex-col gap-1">
              {@render field("Button label")}
              <input class="input w-full" bind:value={item.block.button_label} />
            </label>
            <label class="flex flex-col gap-1">
              {@render field("Button link", "/about, other.md or https://…")}
              <input class="input w-full" bind:value={item.block.button_link} />
            </label>
          </div>
        {/if}
      </div>
    </section>
  {:else}
    <p class="rounded-box border border-dashed border-base-300 p-8 text-center text-base-content/60">
      This page has no content yet. Add a block below.
    </p>
  {/each}

  <div class="flex flex-wrap items-center gap-2">
    <span class="text-sm text-base-content/60">Add:</span>
    {#each TYPES as type (type)}
      <button type="button" id="add-{type}" class="btn btn-sm" onclick={() => add(type)}>
        + {BLOCK_NAMES[type]}
      </button>
    {/each}

    <div class="flex-1"></div>

    {#if error}
      <span id="blocks-error" role="alert" class="text-sm text-error">{error}</span>
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

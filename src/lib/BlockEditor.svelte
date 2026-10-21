<script lang="ts">
  import { untrack } from "svelte";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronLeft from "@lucide/svelte/icons/chevron-left";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import ChevronUp from "@lucide/svelte/icons/chevron-up";
  import ImagePlus from "@lucide/svelte/icons/image-plus";
  import Plus from "@lucide/svelte/icons/plus";
  import Trash from "@lucide/svelte/icons/trash";
  import ImageField from "$lib/ImageField.svelte";
  import { BLOCK_ICONS } from "$lib/icons";
  import MarkdownEditor from "$lib/MarkdownEditor.svelte";
  import MediaPicker from "$lib/MediaPicker.svelte";
  import {
    commands,
    BLOCK_NAMES,
    mediaUrl,
    newBlock,
    normalizeBlock,
    type Block,
    type BlockType,
    type Gallery,
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
  /** The gallery an image is being picked for, if any. */
  let pickingFor = $state<Gallery | null>(null);
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
      const saved = await commands.savePageBlocks(siteRoot, page.file, edited);
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

{#if pickingFor}
  <MediaPicker
    {siteRoot}
    only="image"
    onpick={(name) => pickingFor?.images.push({ image: name, alt: null })}
    onclose={() => (pickingFor = null)}
  />
{/if}

<div id="block-editor" class="flex flex-col gap-4">
  {#each items as item, index (item.id)}
    {@const Icon = BLOCK_ICONS[item.block.type]}
    <section id="block-{index}" class="@container card border border-base-300 bg-base-100">
      <div class="card-body gap-3 p-4">
        <header class="flex items-center gap-2">
          <span class="badge badge-soft gap-1.5"><Icon class="size-3.5" />{BLOCK_NAMES[item.block.type]}</span>
          <div class="flex-1"></div>
          <button type="button" class="btn btn-ghost btn-xs btn-square" title="Move up" aria-label="Move up"
            disabled={index === 0} onclick={() => move(index, -1)}><ChevronUp class="size-4" /></button>
          <button type="button" class="btn btn-ghost btn-xs btn-square" title="Move down" aria-label="Move down"
            disabled={index === items.length - 1} onclick={() => move(index, 1)}><ChevronDown class="size-4" /></button>
          <button type="button" class="btn btn-ghost btn-xs btn-square text-error" title="Remove block" aria-label="Remove block"
            onclick={() => remove(index)}><Trash class="size-4" /></button>
        </header>

        {#if item.block.type === "text"}
          <MarkdownEditor bind:value={item.block.markdown} placeholder="Write in Markdown…" />
        {:else if item.block.type === "image"}
          <div class="grid gap-3 @2xl:grid-cols-2">
            <div class="flex flex-col gap-1">
              {@render field("Image")}
              <ImageField {siteRoot} bind:value={item.block.image} />
            </div>
            <label class="flex flex-col gap-1">
              {@render field("Alternative text", "describes the image for people who can't see it")}
              <input class="input w-full" bind:value={item.block.alt} />
            </label>
            <div class="flex flex-col gap-1 @2xl:col-span-2">
              {@render field("Caption", "Markdown")}
              <MarkdownEditor
                bind:value={() => (item.block.type === "image" ? (item.block.caption ?? "") : ""),
                (text) => {
                  if (item.block.type === "image") item.block.caption = text;
                }}
                minHeight="60px"
              />
            </div>
          </div>
        {:else if item.block.type === "text-image"}
          <div class="grid gap-3 @2xl:grid-cols-2">
            <div class="flex flex-col gap-1">
              {@render field("Image")}
              <ImageField {siteRoot} bind:value={item.block.image} />
            </div>
            <div class="flex flex-col gap-3">
              <label class="flex flex-col gap-1">
                {@render field("Alternative text")}
                <input class="input w-full" bind:value={item.block.alt} />
              </label>
              <div class="flex items-center gap-4 text-sm">
                <span class="font-medium">Image on the</span>
                <label class="flex items-center gap-2">
                  <input type="radio" class="radio radio-sm" value="left" bind:group={item.block.image_side} /> left
                </label>
                <label class="flex items-center gap-2">
                  <input type="radio" class="radio radio-sm" value="right" bind:group={item.block.image_side} /> right
                </label>
              </div>
            </div>
            <div class="flex flex-col gap-1 @2xl:col-span-2">
              {@render field("Text", "Markdown")}
              <MarkdownEditor
                bind:value={() => (item.block.type === "text-image" ? (item.block.text ?? "") : ""),
                (text) => {
                  if (item.block.type === "text-image") item.block.text = text;
                }}
                minHeight="100px"
              />
            </div>
          </div>
        {:else if item.block.type === "gallery"}
          {@const gallery = item.block}
          <div class="flex flex-col gap-3">
            {#if gallery.images.length}
              <ul class="grid grid-cols-[repeat(auto-fill,minmax(10rem,1fr))] gap-3">
                {#each gallery.images as image, i (i)}
                  <li class="flex flex-col gap-2 rounded-box border border-base-300 p-2">
                    <img src={mediaUrl(siteRoot, image.image)} alt="" class="aspect-square w-full rounded-field object-cover" />
                    <input class="input input-sm w-full" placeholder="Alternative text" bind:value={image.alt} />
                    <div class="flex justify-between">
                      <div>
                        <button type="button" class="btn btn-ghost btn-xs btn-square" title="Move left" aria-label="Move left" disabled={i === 0}
                          onclick={() => gallery.images.splice(i - 1, 0, ...gallery.images.splice(i, 1))}><ChevronLeft class="size-4" /></button>
                        <button type="button" class="btn btn-ghost btn-xs btn-square" title="Move right" aria-label="Move right"
                          disabled={i === gallery.images.length - 1}
                          onclick={() => gallery.images.splice(i + 1, 0, ...gallery.images.splice(i, 1))}><ChevronRight class="size-4" /></button>
                      </div>
                      <button type="button" class="btn btn-ghost btn-xs btn-square text-error" title="Remove image" aria-label="Remove image"
                        onclick={() => gallery.images.splice(i, 1)}><Trash class="size-4" /></button>
                    </div>
                  </li>
                {/each}
              </ul>
            {/if}
            <div>
              <button type="button" class="btn btn-sm" onclick={() => (pickingFor = gallery)}><ImagePlus class="size-4" /> Add image…</button>
            </div>
          </div>
        {:else if item.block.type === "download"}
          <div class="grid gap-3 @2xl:grid-cols-2">
            <div class="flex flex-col gap-1">
              {@render field("File")}
              <ImageField {siteRoot} accept="file" bind:value={item.block.file} />
            </div>
            <label class="flex flex-col gap-1">
              {@render field("Link text", "the file name if empty")}
              <input class="input w-full" bind:value={item.block.label} />
            </label>
          </div>
        {:else if item.block.type === "faq"}
          {@const faq = item.block}
          <div class="flex flex-col gap-3">
            <label class="flex flex-col gap-1">
              {@render field("Heading", "optional")}
              <input class="input w-full" bind:value={faq.heading} />
            </label>
            {#each faq.items as entry, i (i)}
              <div class="flex flex-col gap-2 rounded-box border border-base-300 p-3">
                <div class="flex items-center gap-2">
                  <input class="input w-full" placeholder="Question" bind:value={entry.question} />
                  <button type="button" class="btn btn-ghost btn-xs btn-square" title="Move up" aria-label="Move up" disabled={i === 0}
                    onclick={() => faq.items.splice(i - 1, 0, ...faq.items.splice(i, 1))}><ChevronUp class="size-4" /></button>
                  <button type="button" class="btn btn-ghost btn-xs btn-square" title="Move down" aria-label="Move down"
                    disabled={i === faq.items.length - 1}
                    onclick={() => faq.items.splice(i + 1, 0, ...faq.items.splice(i, 1))}><ChevronDown class="size-4" /></button>
                  <button type="button" class="btn btn-ghost btn-xs btn-square text-error" title="Remove question" aria-label="Remove question"
                    onclick={() => faq.items.splice(i, 1)}><Trash class="size-4" /></button>
                </div>
                <MarkdownEditor bind:value={entry.answer} placeholder="Answer, in Markdown…" minHeight="60px" />
              </div>
            {/each}
            <div>
              <button type="button" class="btn btn-sm" onclick={() => faq.items.push({ question: "", answer: "" })}>
                <Plus class="size-4" /> Question
              </button>
            </div>
          </div>
        {:else if item.block.type === "columns"}
          {@const columns = item.block}
          <div class="flex flex-col gap-3">
            <div class="grid gap-3 @2xl:grid-cols-2 @6xl:grid-cols-4">
              {#each columns.items as column, i (i)}
                <div class="flex flex-col gap-2 rounded-box border border-base-300 p-3">
                  <div class="flex items-center gap-1">
                    <input class="input input-sm w-full" placeholder="Title (optional)" bind:value={column.title} />
                    <button type="button" class="btn btn-ghost btn-xs btn-square" title="Move left" aria-label="Move left" disabled={i === 0}
                      onclick={() => columns.items.splice(i - 1, 0, ...columns.items.splice(i, 1))}><ChevronLeft class="size-4" /></button>
                    <button type="button" class="btn btn-ghost btn-xs btn-square" title="Move right" aria-label="Move right"
                      disabled={i === columns.items.length - 1}
                      onclick={() => columns.items.splice(i + 1, 0, ...columns.items.splice(i, 1))}><ChevronRight class="size-4" /></button>
                    <button type="button" class="btn btn-ghost btn-xs btn-square text-error" title="Remove column" aria-label="Remove column"
                      onclick={() => columns.items.splice(i, 1)}><Trash class="size-4" /></button>
                  </div>
                  <MarkdownEditor bind:value={column.text} placeholder="Text, in Markdown…" minHeight="100px" />
                </div>
              {/each}
            </div>
            <div class="flex items-center gap-3">
              <button type="button" class="btn btn-sm" disabled={columns.items.length >= 4}
                onclick={() => columns.items.push({ title: null, text: "" })}><Plus class="size-4" /> Column</button>
              <span class="text-xs text-base-content/60">Up to four; on phones they stack.</span>
            </div>
          </div>
        {:else if item.block.type === "hero" || item.block.type === "call-to-action"}
          <!-- Hero and Call to action share most fields. -->
          <div class="grid gap-3 @2xl:grid-cols-2">
            {#if item.block.type === "hero"}
              <div class="flex flex-col gap-1">
                {@render field("Background image")}
                <ImageField {siteRoot} bind:value={item.block.image} />
              </div>
            {/if}
            <label class={["flex flex-col gap-1", item.block.type !== "hero" && "@2xl:col-span-2"]}>
              {@render field("Headline")}
              <input class="input w-full" bind:value={item.block.headline} />
            </label>
            <div class="flex flex-col gap-1 @2xl:col-span-2">
              {@render field("Text", "Markdown")}
              <MarkdownEditor
                bind:value={() => ("headline" in item.block ? (item.block.text ?? "") : ""),
                (text) => {
                  if ("headline" in item.block) item.block.text = text;
                }}
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
      {@const Icon = BLOCK_ICONS[type]}
      <button type="button" id="add-{type}" class="btn btn-sm" onclick={() => add(type)}>
        <Icon class="size-4" />
        {BLOCK_NAMES[type]}
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

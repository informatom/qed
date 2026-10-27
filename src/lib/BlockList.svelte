<script lang="ts">
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronLeft from "@lucide/svelte/icons/chevron-left";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import ChevronUp from "@lucide/svelte/icons/chevron-up";
  import Component from "@lucide/svelte/icons/component";
  import ImagePlus from "@lucide/svelte/icons/image-plus";
  import Plus from "@lucide/svelte/icons/plus";
  import Trash from "@lucide/svelte/icons/trash";
  // A component may use itself: each column holds a BlockList of its own.
  import { SvelteMap, SvelteSet } from "svelte/reactivity";
  import BlockList from "$lib/BlockList.svelte";
  import ImageField from "$lib/ImageField.svelte";
  import { BLOCK_ICONS } from "$lib/icons";
  import MarkdownEditor from "$lib/MarkdownEditor.svelte";
  import MediaPicker from "$lib/MediaPicker.svelte";
  import { sharedBlocks } from "$lib/sharedBlocks";
  import { BLOCK_NAMES, COLUMN_BLOCK_TYPES, commands, mediaUrl, newBlock, sharedName, type Block, type BlockType, type Gallery } from "$lib/site";

  interface Props {
    /** The blocks being edited; changed in place. */
    blocks: Block[];
    siteRoot: string;
    /** The block types that can be added here. */
    types?: BlockType[];
    /** Inside a column: smaller, without page-level ids. */
    nested?: boolean;
  }

  let { blocks = $bindable(), siteRoot, types = Object.keys(BLOCK_NAMES) as BlockType[], nested = false }: Props = $props();

  const shared = sharedBlocks();
  // A shared block can't hold another one.
  let offered = $derived(shared?.insidePiece ? types.filter((type) => type !== "shared") : types);
  let canShare = $derived(shared !== undefined && !shared.insidePiece);

  /** The block "Make shared" was clicked on, its new name, and what went wrong. */
  let sharing = $state<Block | null>(null);
  let shareTitle = $state("");
  let shareError = $state<string | null>(null);
  let makingShared = $state(false);

  function startSharing(block: Block) {
    sharing = block;
    shareTitle = "";
    shareError = null;
  }

  // The block moves into a new file in shared/; a Shared block showing that
  // file takes its place here.
  async function makeShared(index: number) {
    if (!shared || !shareTitle.trim()) return;
    makingShared = true;
    shareError = null;
    try {
      const name = await shared.make(shareTitle, $state.snapshot(blocks[index]));
      blocks[index] = { type: "shared", name };
      sharing = null;
    } catch (e) {
      shareError = String(e);
    } finally {
      makingShared = false;
    }
  }

  type VideoBlock = Extract<Block, { type: "video" }>;
  /** Video blocks whose thumbnail is being fetched, and why a fetch failed. */
  const fetching = new SvelteSet<VideoBlock>();
  const videoErrors = new SvelteMap<VideoBlock, string>();

  // Asks the platform once for title and thumbnail; the thumbnail lands in
  // Media. A title typed by hand stays.
  async function fetchVideo(block: VideoBlock) {
    if (!block.url) return;
    fetching.add(block);
    videoErrors.delete(block);
    try {
      const info = await commands.fetchVideo(siteRoot, block.url.trim());
      block.image = info.image;
      if (!block.title?.trim()) block.title = info.title;
    } catch (e) {
      videoErrors.set(block, String(e));
    } finally {
      fetching.delete(block);
    }
  }

  /** The gallery an image is being picked for, if any. */
  let pickingFor = $state<Gallery | null>(null);

  function add(type: BlockType) {
    blocks.push(newBlock(type));
  }

  function remove(index: number) {
    blocks.splice(index, 1);
  }

  function move(index: number, by: number) {
    const [block] = blocks.splice(index, 1);
    blocks.splice(index + by, 0, block);
  }
</script>

{#snippet layoutChoice(list: Extract<Block, { type: "post-list" | "page-list" }>)}
  <div class="join">
    {#each [["list", "List"], ["cards", "Cards"], ["tiles", "Tiles"]] as [value, label] (value)}
      <button type="button" class={["btn join-item btn-sm", (list.layout ?? "list") === value && "btn-active"]}
        onclick={() => (list.layout = value === "list" ? null : (value as "cards" | "tiles"))}>{label}</button>
    {/each}
  </div>
{/snippet}

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

<div class={["flex flex-col", nested ? "gap-2" : "gap-4"]}>
  {#each blocks as block, index (block)}
    {@const Icon = BLOCK_ICONS[block.type]}
    <section id={nested ? undefined : `block-${index}`} class={["@container card border border-base-300", nested ? "bg-base-200/40" : "bg-base-100"]}>
      <div class="card-body gap-3 p-4">
        <header class="flex items-center gap-2">
          <span class="badge badge-soft gap-1.5"><Icon class="size-3.5" />{BLOCK_NAMES[block.type]}</span>
          <div class="flex-1"></div>
          {#if canShare && block.type !== "shared"}
            <button type="button" class="btn btn-ghost btn-xs" title="Move this block into a shared block, to show it on other pages too"
              onclick={() => startSharing(block)}><Component class="size-3.5" /> Make shared</button>
          {/if}
          <button type="button" class="btn btn-ghost btn-xs btn-square" title="Move up" aria-label="Move up"
            disabled={index === 0} onclick={() => move(index, -1)}><ChevronUp class="size-4" /></button>
          <button type="button" class="btn btn-ghost btn-xs btn-square" title="Move down" aria-label="Move down"
            disabled={index === blocks.length - 1} onclick={() => move(index, 1)}><ChevronDown class="size-4" /></button>
          <button type="button" class="btn btn-ghost btn-xs btn-square text-error" title="Remove block" aria-label="Remove block"
            onclick={() => remove(index)}><Trash class="size-4" /></button>
        </header>

        {#if sharing === block}
          <form class="flex flex-col gap-2 rounded-box border border-primary/40 bg-primary/5 p-3"
            onsubmit={(e) => {
              e.preventDefault();
              makeShared(index);
            }}>
            <div class="flex flex-wrap items-center gap-2">
              <!-- svelte-ignore a11y_autofocus -->
              <input class="input input-sm min-w-48 flex-1" placeholder="Name, e.g. Opening hours" aria-label="Name of the shared block"
                bind:value={shareTitle} autofocus />
              <button type="submit" class="btn btn-primary btn-sm" disabled={!shareTitle.trim() || makingShared}>
                {#if makingShared}<span class="loading loading-spinner loading-xs"></span>{/if}
                Make shared
              </button>
              <button type="button" class="btn btn-ghost btn-sm" onclick={() => (sharing = null)}>Cancel</button>
            </div>
            <p class="text-xs text-base-content/60">
              The block moves into its own file in <code>shared/</code>; here a Shared block shows it. Add it to other
              pages with Add → Shared block. Save this page to keep the change.
            </p>
            {#if shareError}<p role="alert" class="text-sm text-error">{shareError}</p>{/if}
          </form>
        {/if}

        {#if block.type === "text"}
          <MarkdownEditor bind:value={block.markdown} placeholder="Write in Markdown…" />
        {:else if block.type === "image"}
          <div class="grid gap-3 @2xl:grid-cols-2">
            <div class="flex flex-col gap-1">
              {@render field("Image")}
              <ImageField {siteRoot} bind:value={block.image} />
            </div>
            <label class="flex flex-col gap-1">
              {@render field("Alternative text", "describes the image for people who can't see it")}
              <input class="input w-full" bind:value={block.alt} />
            </label>
            <div class="flex flex-col gap-1 @2xl:col-span-2">
              {@render field("Caption", "Markdown")}
              <MarkdownEditor
                bind:value={() => (block.type === "image" ? (block.caption ?? "") : ""),
                (text) => {
                  if (block.type === "image") block.caption = text;
                }}
                minHeight="60px"
              />
            </div>
          </div>
        {:else if block.type === "text-image"}
          <div class="grid gap-3 @2xl:grid-cols-2">
            <div class="flex flex-col gap-1">
              {@render field("Image")}
              <ImageField {siteRoot} bind:value={block.image} />
            </div>
            <div class="flex flex-col gap-3">
              <label class="flex flex-col gap-1">
                {@render field("Alternative text")}
                <input class="input w-full" bind:value={block.alt} />
              </label>
              <div class="flex items-center gap-4 text-sm">
                <span class="font-medium">Image on the</span>
                <label class="flex items-center gap-2">
                  <input type="radio" class="radio radio-sm" value="left" bind:group={block.image_side} /> left
                </label>
                <label class="flex items-center gap-2">
                  <input type="radio" class="radio radio-sm" value="right" bind:group={block.image_side} /> right
                </label>
              </div>
            </div>
            <div class="flex flex-col gap-1 @2xl:col-span-2">
              {@render field("Text", "Markdown")}
              <MarkdownEditor
                bind:value={() => (block.type === "text-image" ? (block.text ?? "") : ""),
                (text) => {
                  if (block.type === "text-image") block.text = text;
                }}
                minHeight="100px"
              />
            </div>
          </div>
        {:else if block.type === "gallery"}
          {@const gallery = block}
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
        {:else if block.type === "download"}
          <div class="grid gap-3 @2xl:grid-cols-2">
            <div class="flex flex-col gap-1">
              {@render field("File")}
              <ImageField {siteRoot} accept="file" bind:value={block.file} />
            </div>
            <label class="flex flex-col gap-1">
              {@render field("Link text", "the file name if empty")}
              <input class="input w-full" bind:value={block.label} />
            </label>
          </div>
        {:else if block.type === "faq"}
          {@const faq = block}
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
        {:else if block.type === "columns"}
          {@const columns = block}
          <div class="flex flex-col gap-3">
            <div class="grid gap-3 @4xl:grid-cols-2">
              {#each columns.items as column, i (column)}
                <div class="flex flex-col gap-3 rounded-box border border-base-300 p-3">
                  <div class="flex items-center gap-1">
                    <span class="text-sm font-medium whitespace-nowrap">Column {i + 1}</span>
                    <input class="input input-sm w-full" placeholder="Title (optional)" bind:value={column.title} />
                    <!-- A share of the width: 2 next to 1 is two thirds. -->
                    <select class="select select-sm w-20 shrink-0" title="Width, relative to the other columns"
                      aria-label="Width of column {i + 1}" value={String(column.width ?? 1)}
                      onchange={(e) => (column.width = e.currentTarget.value === "1" ? null : Number(e.currentTarget.value))}>
                      <option value="1">1×</option>
                      <option value="2">2×</option>
                      <option value="3">3×</option>
                    </select>
                    <label class="flex shrink-0 items-center gap-1 text-xs" title="Draw the column as a box: own background, border, rounded corners">
                      <input type="checkbox" class="checkbox checkbox-xs" bind:checked={column.boxed} aria-label="Column {i + 1} as a box" />
                      Box
                    </label>
                    <button type="button" class="btn btn-ghost btn-xs btn-square" title="Move left" aria-label="Move left" disabled={i === 0}
                      onclick={() => columns.items.splice(i - 1, 0, ...columns.items.splice(i, 1))}><ChevronLeft class="size-4" /></button>
                    <button type="button" class="btn btn-ghost btn-xs btn-square" title="Move right" aria-label="Move right"
                      disabled={i === columns.items.length - 1}
                      onclick={() => columns.items.splice(i + 1, 0, ...columns.items.splice(i, 1))}><ChevronRight class="size-4" /></button>
                    <button type="button" class="btn btn-ghost btn-xs btn-square text-error" title="Remove column" aria-label="Remove column"
                      onclick={() => columns.items.splice(i, 1)}><Trash class="size-4" /></button>
                  </div>
                  <!-- The same list, one level down, with the blocks that fit in a column. -->
                  <BlockList bind:blocks={column.blocks} {siteRoot} types={COLUMN_BLOCK_TYPES} nested />
                </div>
              {/each}
            </div>
            <div class="flex items-center gap-3">
              <button type="button" class="btn btn-sm" disabled={columns.items.length >= 4}
                onclick={() => columns.items.push({ title: null, width: null, boxed: false, blocks: [newBlock("text")] })}><Plus class="size-4" /> Column</button>
              <span class="text-xs text-base-content/60">
                Up to four; on phones they stack. Widths {columns.items.map((c) => c.width ?? 1).join(" : ")}
                (2× next to 1× is two thirds).
              </span>
            </div>
          </div>
        {:else if block.type === "post-list"}
          <div class="grid gap-3 @2xl:grid-cols-[1fr_12rem]">
            <div class="flex flex-col gap-1 @2xl:col-span-2">
              {@render field("Layout", "cards and tiles show each post's share image, or its first picture")}
              {@render layoutChoice(block)}
            </div>
            <label class="flex flex-col gap-1">
              {@render field("From page", "this page if empty, or e.g. /blog")}
              <input class="input w-full font-mono" bind:value={block.from} placeholder="this page" />
            </label>
            <label class="flex flex-col gap-1">
              {@render field("How many", "all if empty")}
              <input class="input w-full" type="number" min="1" bind:value={block.limit} placeholder="all" />
            </label>
            <p class="text-sm text-base-content/60 @2xl:col-span-2">
              Lists the pages below that page that have a date, newest first, with date and summary. Posts dated in
              the future appear once the site is built on or after their date.
            </p>
          </div>
        {:else if block.type === "video"}
          <div class="flex flex-col gap-3">
            <div class="flex flex-wrap items-end gap-2">
              <label class="flex min-w-64 flex-1 flex-col gap-1">
                {@render field("Video address", "on YouTube, Vimeo or PeerTube")}
                <input class="input w-full font-mono" bind:value={block.url}
                  placeholder="https://www.youtube.com/watch?v=…" />
              </label>
              <button type="button" class="btn" disabled={!block.url?.trim() || fetching.has(block)}
                onclick={() => block.type === "video" && fetchVideo(block)}>
                {#if fetching.has(block)}<span class="loading loading-spinner loading-sm"></span>{/if}
                Get thumbnail and title
              </button>
            </div>
            {#if videoErrors.get(block)}
              <div role="alert" class="alert alert-error alert-soft text-sm">{videoErrors.get(block)}</div>
            {/if}
            <label class="flex flex-col gap-1">
              {@render field("Title", "shown below the video")}
              <input class="input w-full" bind:value={block.title} />
            </label>
            <div class="flex flex-col gap-1">
              {@render field("Thumbnail", "fetched once and kept in Media; any image works")}
              <ImageField {siteRoot} bind:value={block.image} />
            </div>
            <p class="text-sm text-base-content/60">
              Works with YouTube, Vimeo and PeerTube (any instance). The website shows the thumbnail with a play mark and
              links to the video: no embedded player, nothing loaded from the platform until a visitor clicks.
            </p>
          </div>
        {:else if block.type === "page-list"}
          <div class="flex flex-col gap-3">
            <div class="flex flex-col gap-1">
              {@render field("Layout", "cards and tiles show each page's share image, or its first picture")}
              {@render layoutChoice(block)}
            </div>
            <label class="flex flex-col gap-1">
              {@render field("From page", "this page if empty, or e.g. /team")}
              <input class="input w-full font-mono" bind:value={block.from} placeholder="this page" />
            </label>
            <p class="text-sm text-base-content/60">
              Lists the pages one level below that page, by weight, then title, each with its summary (Settings of
              that page). Pages with a date are posts and left out; a Post list shows those.
            </p>
          </div>
        {:else if block.type === "shared"}
          {@const piece = shared?.pieces.find((p) => sharedName(p) === block.name)}
          <div class="flex flex-col gap-3">
            <div class="flex flex-wrap items-end gap-2">
              <label class="flex min-w-64 flex-1 flex-col gap-1">
                {@render field("Shows", "the same blocks on every page that uses it")}
                <select class="select w-full" bind:value={block.name}>
                  <option value={null}>Pick a shared block…</option>
                  {#each shared?.pieces ?? [] as option (option.file)}
                    <option value={sharedName(option)}>{option.front.title}</option>
                  {/each}
                  {#if block.name && !piece}<option value={block.name}>{block.name} (missing)</option>{/if}
                </select>
              </label>
              <button type="button" class="btn" disabled={!piece}
                onclick={() => block.type === "shared" && block.name && shared?.open(block.name)}>Edit shared block</button>
            </div>
            {#if piece}
              <div class="flex flex-wrap items-center gap-1">
                {#each piece.blocks as inner, i (i)}
                  <span class="badge badge-soft badge-sm">{BLOCK_NAMES[inner.type]}</span>
                {:else}
                  <span class="text-sm text-base-content/60">It has no blocks yet.</span>
                {/each}
              </div>
              <p class="text-sm text-base-content/60">Edit it in one place; changes show on every page that uses it.</p>
            {:else if block.name}
              <p role="alert" class="text-sm text-error">There's no shared block “{block.name}”; the site shows nothing here.</p>
            {:else if !shared?.pieces.length}
              <p class="text-sm text-base-content/60">
                No shared blocks yet. Use “Make shared” on any block, or “New” under Shared blocks in the page list.
              </p>
            {/if}
          </div>
        {:else if block.type === "hero" || block.type === "call-to-action"}
          <!-- Hero and Call to action share most fields. -->
          <div class="grid gap-3 @2xl:grid-cols-2">
            {#if block.type === "hero"}
              <div class="flex flex-col gap-1">
                {@render field("Background image")}
                <ImageField {siteRoot} bind:value={block.image} />
              </div>
            {/if}
            <label class={["flex flex-col gap-1", block.type !== "hero" && "@2xl:col-span-2"]}>
              {@render field("Headline")}
              <input class="input w-full" bind:value={block.headline} />
            </label>
            <div class="flex flex-col gap-1 @2xl:col-span-2">
              {@render field("Text", "Markdown")}
              <MarkdownEditor
                bind:value={() => ("headline" in block ? (block.text ?? "") : ""),
                (text) => {
                  if ("headline" in block) block.text = text;
                }}
                minHeight="80px"
              />
            </div>
            <label class="flex flex-col gap-1">
              {@render field("Button label")}
              <input class="input w-full" bind:value={block.button_label} />
            </label>
            <label class="flex flex-col gap-1">
              {@render field("Button link", "/about, other.md or https://…")}
              <input class="input w-full" bind:value={block.button_link} />
            </label>
          </div>
        {/if}
      </div>
    </section>
  {:else}
    <p class={["rounded-box border border-dashed border-base-300 text-center text-base-content/60", nested ? "p-3 text-sm" : "p-8"]}>
      {nested ? "Empty column." : "This page has no content yet. Add a block below."}
    </p>
  {/each}

  <div class="flex flex-wrap items-center gap-2">
    <span class="text-sm text-base-content/60">Add:</span>
    {#each offered as type (type)}
      {@const Icon = BLOCK_ICONS[type]}
      <button type="button" id={nested ? undefined : `add-${type}`} class={["btn", nested ? "btn-xs" : "btn-sm"]} onclick={() => add(type)}>
        <Icon class="size-4" />
        {BLOCK_NAMES[type]}
      </button>
    {/each}
  </div>
</div>

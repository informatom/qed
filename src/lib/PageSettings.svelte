<script lang="ts">
  import { untrack } from "svelte";
  import { onMount } from "svelte";
  import ImageField from "$lib/ImageField.svelte";
  import { commands, pageIcons, type FrontMatter, type Menu, type Page } from "$lib/site";

  interface Props {
    page: Page;
    siteRoot: string;
    onsaved: (page: Page) => void;
    /** Settings to start with instead of the saved ones (an earlier version). */
    start?: FrontMatter;
    /** Tells the parent whether there are unsaved changes. */
    dirty?: boolean;
  }

  let { page, siteRoot, onsaved, start, dirty = $bindable(false) }: Props = $props();

  // An editable copy of the settings, taken once when the form opens
  // (`untrack` says so on purpose; the parent re-creates the form for another
  // page). `$state.snapshot` makes a plain copy, so typing here doesn't change
  // the page list until we save.
  const initial = untrack(() => $state.snapshot(start ?? page.front));
  let form = $state(initial);
  // Lists are edited as comma-separated text.
  let authors = $state(initial.authors.join(", "));
  let tags = $state(initial.tags.join(", "));

  let icons = $state<[string, string][]>([]);
  onMount(async () => {
    icons = [...(await pageIcons())];
  });

  let saving = $state(false);
  let error = $state<string | null>(null);
  let savedAt = $state<Date | null>(null);

  const MENUS: { value: Menu; label: string }[] = [
    { value: "none", label: "Not in a menu" },
    { value: "main", label: "Main menu" },
    { value: "footer", label: "Footer menu" },
  ];

  function list(text: string): string[] {
    return text
      .split(",")
      .map((item) => item.trim())
      .filter((item) => item !== "");
  }

  function orNull(text: string | null): string | null {
    return text && text.trim() !== "" ? text.trim() : null;
  }

  // What would be saved, recomputed whenever a field changes.
  let edited: FrontMatter = $derived({
    ...form,
    title: form.title.trim(),
    description: orNull(form.description),
    image: orNull(form.image),
    icon: orNull(form.icon),
    summary: orNull(form.summary),
    date: orNull(form.date),
    weight: Number(form.weight) || 0,
    authors: list(authors),
    tags: list(tags),
  });

  let changed = $derived(JSON.stringify(edited) !== JSON.stringify(page.front));

  // Keep the parent informed.
  $effect(() => {
    dirty = changed;
  });

  async function save(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    error = null;
    try {
      const saved = await commands.savePageSettings(siteRoot, page.file, edited);
      savedAt = new Date();
      onsaved(saved);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<!-- `@container` lets the fields below arrange themselves by the panel's own
     width (`@xl:`), not the window's. -->
<form id="page-settings" class="@container card border border-base-300 bg-base-100" onsubmit={save}>
  <div class="card-body grid gap-4 @xl:grid-cols-2">
    <label class="flex flex-col gap-1 @xl:col-span-2">
      <span class="text-sm font-medium">Title</span>
      <input id="setting-title" class="input w-full" bind:value={form.title} required />
    </label>

    <label class="flex flex-col gap-1 @xl:col-span-2">
      <span class="text-sm font-medium">Description</span>
      <textarea id="setting-description" class="textarea w-full" rows="2" bind:value={form.description}
        placeholder="Shown by search engines"></textarea>
    </label>

    <div id="setting-image" class="flex flex-col gap-1 @xl:col-span-2">
      <span class="text-sm font-medium">Share image <span class="font-normal text-base-content/50">· shown when someone shares a link to the page; without one, the page's first image. Needs the site's web address.</span></span>
      <ImageField {siteRoot} bind:value={form.image} />
    </div>

    <div class="grid grid-cols-[1fr_6rem] gap-3">
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Menu</span>
        <select id="setting-menu" class="select w-full" bind:value={form.menu}>
          {#each MENUS as menu (menu.value)}
            <option value={menu.value}>{menu.label}</option>
          {/each}
        </select>
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Order</span>
        <input id="setting-weight" class="input w-full" type="number" bind:value={form.weight} />
      </label>
    </div>

    <fieldset class="flex flex-col gap-2 @xl:col-span-2">
      <legend class="mb-1 text-sm font-medium">Icon <span class="font-normal text-base-content/50">· shown in menus</span></legend>
      <div class="flex flex-wrap gap-1" role="radiogroup">
        <button type="button" role="radio" aria-checked={!form.icon} onclick={() => (form.icon = null)}
          class={["btn btn-sm", !form.icon ? "btn-primary" : "btn-ghost"]}>No icon</button>
        {#each icons as [name, svg] (name)}
          <!-- The SVGs are qed's own built-in icons, so inserting them as HTML is safe. -->
          <button type="button" role="radio" aria-checked={form.icon === name} title={name} aria-label={name}
            onclick={() => (form.icon = name)}
            class={["btn btn-sm btn-square", form.icon === name ? "btn-primary" : "btn-ghost"]}>{@html svg}</button>
        {/each}
      </div>
    </fieldset>

    <div class="flex flex-col gap-2 @xl:self-end @xl:pb-2">
      <label class="flex items-center gap-3">
        <input id="setting-draft" type="checkbox" class="toggle" bind:checked={form.draft} />
        <span class="text-sm">Draft (left out of the built site)</span>
      </label>
      <label class="flex items-center gap-3">
        <input id="setting-hide-title" type="checkbox" class="toggle" bind:checked={form.hide_title} />
        <span class="text-sm">Hide the title (for pages that start with their own heading)</span>
      </label>
    </div>

    <fieldset class="grid gap-3 rounded-box border border-base-300 p-4 @xl:col-span-2 @xl:grid-cols-2">
      <legend class="px-1 text-sm font-medium">Blog post</legend>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Date</span>
        <input id="setting-date" class="input w-full" type="date" bind:value={form.date} />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Authors</span>
        <input id="setting-authors" class="input w-full" bind:value={authors} placeholder="Comma-separated" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Tags</span>
        <input id="setting-tags" class="input w-full" bind:value={tags} placeholder="Comma-separated" />
      </label>
      <label class="flex flex-col gap-1 @xl:col-span-2">
        <span class="text-sm font-medium">Summary</span>
        <textarea id="setting-summary" class="textarea w-full" rows="3" bind:value={form.summary}></textarea>
      </label>
    </fieldset>

    {#if error}
      <div id="settings-error" role="alert" class="alert alert-error alert-soft text-sm @xl:col-span-2">{error}</div>
    {/if}

    <div class="card-actions items-center justify-end @xl:col-span-2">
      {#if changed}
        <span class="text-sm text-warning">Unsaved changes</span>
      {:else if savedAt}
        <span class="text-sm text-success">Saved</span>
      {/if}
      <button id="save-settings" class="btn btn-primary" disabled={!changed || saving}>
        {#if saving}<span class="loading loading-spinner loading-sm"></span>{/if}
        Save
      </button>
    </div>
  </div>
</form>

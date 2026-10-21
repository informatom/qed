<script lang="ts">
  import { untrack } from "svelte";
  import { savePageSettings, type FrontMatter, type Menu, type Page } from "$lib/site";

  interface Props {
    page: Page;
    siteRoot: string;
    onsaved: (page: Page) => void;
    /** Tells the parent whether there are unsaved changes. */
    dirty?: boolean;
  }

  let { page, siteRoot, onsaved, dirty = $bindable(false) }: Props = $props();

  // An editable copy of the settings, taken once when the form opens
  // (`untrack` says so on purpose; the parent re-creates the form for another
  // page). `$state.snapshot` makes a plain copy, so typing here doesn't change
  // the page list until we save.
  const initial = untrack(() => $state.snapshot(page.front));
  let form = $state(initial);
  // Lists are edited as comma-separated text.
  let authors = $state(initial.authors.join(", "));
  let tags = $state(initial.tags.join(", "));

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
      const saved = await savePageSettings(siteRoot, page.file, edited);
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

    <label class="flex items-center gap-3 @xl:self-end @xl:pb-2">
      <input id="setting-draft" type="checkbox" class="toggle" bind:checked={form.draft} />
      <span class="text-sm">Draft (left out of the built site)</span>
    </label>

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

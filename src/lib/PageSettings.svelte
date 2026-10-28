<script lang="ts">
  import { untrack } from "svelte";
  import { onMount } from "svelte";
  import CoverCheck from "$lib/CoverCheck.svelte";
  import ImageField from "$lib/ImageField.svelte";
  import { commands, pageIcons, type Episode, type EpisodeType, type FrontMatter, type Menu, type Page } from "$lib/site";

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

  /** A whole number from `min` up, or null: number fields give "" when empty. */
  function counted(value: unknown, min: number): number | null {
    const n = Number(value);
    return value !== null && value !== "" && Number.isInteger(n) && n >= min ? n : null;
  }

  function cleanEpisode(episode: Episode | null): Episode | null {
    if (!episode) return null;
    return {
      ...episode,
      season: counted(episode.season, 1),
      // Episode 0 is common: many podcasts start counting there.
      number: counted(episode.number, 0),
      image: orNull(episode.image),
      chapters: episode.chapters
        .map((c) => ({ start: c.start.trim(), title: c.title.trim(), link: orNull(c.link) }))
        .filter((c) => c.start !== "" || c.title !== ""),
    };
  }

  const EPISODE_TYPES: { value: EpisodeType; label: string }[] = [
    { value: "full", label: "Episode" },
    { value: "trailer", label: "Trailer" },
    { value: "bonus", label: "Bonus" },
  ];

  // Explicit content: the podcast's setting, or the episode's own.
  let explicit = $state(initial.episode?.explicit == null ? "podcast" : initial.episode.explicit ? "yes" : "no");
  $effect(() => {
    if (form.episode) form.episode.explicit = explicit === "podcast" ? null : explicit === "yes";
  });

  function addChapter() {
    if (!form.episode) return;
    // Starts where the last one left off, to be corrected by hand.
    const last = form.episode.chapters.at(-1)?.start ?? "";
    form.episode.chapters = [...form.episode.chapters, { start: last === "" ? "00:00:00" : last, title: "", link: null }];
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
    episode: cleanEpisode(form.episode),
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

    {#if form.episode}
      {@const episode = form.episode}
      <fieldset id="episode-settings" class="grid gap-3 rounded-box border border-base-300 p-4 @xl:col-span-2 @xl:grid-cols-4">
        <legend class="px-1 text-sm font-medium">Episode</legend>
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">Season</span>
          <input id="episode-season" class="input w-full" type="number" min="1" bind:value={episode.season} placeholder="None" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">Number</span>
          <input id="episode-number" class="input w-full" type="number" min="0" bind:value={episode.number} placeholder="None" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">Type</span>
          <select id="episode-type" class="select w-full" bind:value={episode.type}>
            {#each EPISODE_TYPES as type (type.value)}<option value={type.value}>{type.label}</option>{/each}
          </select>
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">Explicit</span>
          <select id="episode-explicit" class="select w-full" bind:value={explicit}>
            <option value="podcast">As the podcast</option>
            <option value="yes">Yes</option>
            <option value="no">No</option>
          </select>
        </label>

        <div id="episode-image" class="flex flex-col gap-2 @xl:col-span-4">
          <span class="text-sm font-medium">Episode cover <span class="font-normal text-base-content/50">· the podcast's cover if empty</span></span>
          <ImageField {siteRoot} bind:value={episode.image} />
          <CoverCheck {siteRoot} name={episode.image} />
        </div>

        <div id="episode-chapters" class="flex flex-col gap-2 @xl:col-span-4">
          <span class="text-sm font-medium">Chapters <span class="font-normal text-base-content/50">· start as 01:02:03 or 02:03, in order</span></span>
          {#each episode.chapters as chapter, index (index)}
            <div class="grid grid-cols-[7rem_1fr_1fr_auto] gap-2">
              <input class="input w-full font-mono" bind:value={chapter.start} placeholder="00:00:00" aria-label="Start" />
              <input class="input w-full" bind:value={chapter.title} placeholder="Title" aria-label="Title" />
              <input class="input w-full" bind:value={chapter.link} placeholder="Link (optional)" aria-label="Link" />
              <button type="button" class="btn btn-ghost" aria-label="Remove chapter"
                onclick={() => (episode.chapters = episode.chapters.filter((_, i) => i !== index))}>Remove</button>
            </div>
          {/each}
          <div><button type="button" id="episode-add-chapter" class="btn btn-sm" onclick={addChapter}>Add a chapter</button></div>
        </div>

        <p class="text-xs text-base-content/50 @xl:col-span-4">
          Guid <span id="episode-guid" class="font-mono">{episode.guid}</span> · podcast apps tell episodes apart by it; it never changes.
        </p>
      </fieldset>
    {/if}

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

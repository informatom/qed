<script lang="ts">
  import { onMount, untrack } from "svelte";
  import ThemePicker from "$lib/ThemePicker.svelte";
  import { listThemes, saveSiteConfig, type Site, type SiteConfig } from "$lib/site";

  interface Props {
    site: Site;
    onsaved: (config: SiteConfig) => void;
  }

  let { site, onsaved }: Props = $props();
  let themes = $state<string[]>([]);
  let form = $state(untrack(() => $state.snapshot(site.config)));
  let saving = $state(false);
  let error = $state<string | null>(null);
  let savedAt = $state<Date | null>(null);

  onMount(async () => {
    themes = await listThemes();
  });

  function orNull(text: string | null): string | null {
    return text && text.trim() !== "" ? text.trim() : null;
  }

  // What would be saved, and whether it differs from the file.
  let edited: SiteConfig = $derived({
    ...form,
    title: form.title.trim(),
    language: form.language.trim() || "en",
    description: orNull(form.description),
    base_url: orNull(form.base_url),
  });
  let changed = $derived(JSON.stringify(edited) !== JSON.stringify(site.config));

  async function save(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    error = null;
    try {
      await saveSiteConfig(site.root, edited);
      onsaved(edited);
      savedAt = new Date();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<form id="site-settings" class="@container flex flex-col gap-4" onsubmit={save}>
  <div class="grid gap-3 @3xl:grid-cols-[2fr_1fr]">
    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">Title</span>
      <input id="site-title" class="input w-full" bind:value={form.title} required />
    </label>
    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">Language <span class="font-normal text-base-content/50">· e.g. en, de</span></span>
      <input id="site-language" class="input w-full" bind:value={form.language} />
    </label>
    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">Description</span>
      <input id="site-description" class="input w-full" bind:value={form.description} placeholder="Shown by search engines" />
    </label>
    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">Web address <span class="font-normal text-base-content/50">· for the sitemap</span></span>
      <input id="site-base-url" class="input w-full" bind:value={form.base_url} placeholder="https://example.com/" />
    </label>
  </div>

  <div class="flex flex-col gap-2">
    <span class="text-sm font-medium">Theme of the website</span>
    <ThemePicker {themes} bind:value={() => form.theme, (theme) => (form.theme = theme ?? "light")} />
  </div>

  {#if error}
    <div role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
  {/if}

  <div class="flex items-center justify-end gap-2">
    {#if changed}
      <span class="text-sm text-warning">Unsaved changes</span>
      <button type="button" class="btn btn-ghost" onclick={() => (form = $state.snapshot(site.config))}>Discard</button>
    {:else if savedAt}
      <span class="text-sm text-success">Saved</span>
    {/if}
    <button id="save-site-settings" class="btn btn-primary" disabled={!changed || saving}>
      {#if saving}<span class="loading loading-spinner loading-sm"></span>{/if}
      Save
    </button>
  </div>
</form>

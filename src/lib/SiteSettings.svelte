<script lang="ts">
  import { untrack } from "svelte";
  import ImageField from "$lib/ImageField.svelte";
  import { commands, type Site, type SiteConfig } from "$lib/site";

  interface Props {
    site: Site;
    onsaved: (config: SiteConfig) => void;
  }

  let { site, onsaved }: Props = $props();
  let form = $state(untrack(() => $state.snapshot(site.config)));
  let saving = $state(false);

  let error = $state<string | null>(null);
  let savedAt = $state<Date | null>(null);

  function orNull(text: string | null): string | null {
    return text && text.trim() !== "" ? text.trim() : null;
  }

  // What would be saved, and whether it differs from the file. The theme
  // has its own section (SiteTheme), which saves it by itself.
  let edited: SiteConfig = $derived({
    ...form,
    theme: site.config.theme,
    custom_theme: site.config.custom_theme,
    title: form.title.trim(),
    language: form.language.trim() || "en",
    description: orNull(form.description),
    logo: orNull(form.logo),
    base_url: orNull(form.base_url),
  });
  let changed = $derived(JSON.stringify(edited) !== JSON.stringify(site.config));

  async function save(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    error = null;
    try {
      await commands.saveSiteConfig(site.root, edited);
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
      <span class="text-sm font-medium">Web address <span class="font-normal text-base-content/50">· for the sitemap and feeds</span></span>
      <input id="site-base-url" class="input w-full" bind:value={form.base_url} placeholder="https://example.com/" />
    </label>
  </div>

  <div id="site-logo" class="flex flex-col gap-1">
    <span class="text-sm font-medium">Logo <span class="font-normal text-base-content/50">· before the title on every page, and the icon in the browser tab; a small square image, SVG at best</span></span>
    <ImageField siteRoot={site.root} bind:value={form.logo} />
  </div>

  <label class="flex items-start gap-3">
    <input id="site-sidebar" type="checkbox" class="toggle mt-0.5"
      bind:checked={() => form.navigation === "sidebar", (on) => (form.navigation = on ? "sidebar" : "top")} />
    <span class="text-sm">
      Show the main menu in a sidebar
      <span class="block text-base-content/60">
        For sites with more menu pages than fit into the top bar. On phones, it becomes a "Menu" box above the content.
      </span>
    </span>
  </label>

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

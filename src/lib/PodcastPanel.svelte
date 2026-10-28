<script lang="ts">
  import { untrack } from "svelte";
  import PodcastSettings from "$lib/PodcastSettings.svelte";
  import { trackUnsaved } from "$lib/unsaved.svelte";
  import { commands, type Podcast, type Site, type SiteConfig } from "$lib/site";

  interface Props {
    site: Site;
    onsaved: (config: SiteConfig) => void;
  }

  let { site, onsaved }: Props = $props();
  let form = $state(untrack(() => $state.snapshot(site.config.podcast)));
  let saving = $state(false);
  let error = $state<string | null>(null);
  let savedAt = $state<Date | null>(null);

  function orNull(text: string | null): string | null {
    return text && text.trim() !== "" ? text.trim() : null;
  }

  // Empty fields as Rust keeps them: null, and no half-filled support links.
  function clean(podcast: Podcast | null): Podcast | null {
    if (!podcast) return null;
    return {
      ...podcast,
      title: orNull(podcast.title),
      description: orNull(podcast.description),
      author: orNull(podcast.author),
      owner_name: orNull(podcast.owner_name),
      owner_email: orNull(podcast.owner_email),
      cover: orNull(podcast.cover),
      copyright: orNull(podcast.copyright),
      audio_base_url: orNull(podcast.audio_base_url),
      funding: podcast.funding
        .map((f) => ({ url: f.url.trim(), title: f.title.trim() }))
        .filter((f) => f.url !== "" || f.title !== ""),
    };
  }

  // Only the podcast changes here; everything else is taken as saved.
  let edited = $derived(clean(form));
  let changed = $derived(JSON.stringify(edited) !== JSON.stringify(site.config.podcast));
  trackUnsaved(() => "Podcast settings", () => changed);

  async function save(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    error = null;
    try {
      const config: SiteConfig = { ...$state.snapshot(site.config), podcast: edited };
      const saved = await commands.saveSiteConfig(site.root, config);
      onsaved(saved);
      // The podcast guid is made while saving.
      form = $state.snapshot(saved.podcast);
      savedAt = new Date();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<form id="podcast-settings" class="@container flex flex-col gap-6" onsubmit={save}>
  <PodcastSettings siteRoot={site.root} bind:podcast={form} siteTitle={site.config.title}
    siteDescription={site.config.description} hasBaseUrl={!!site.config.base_url} contributors={site.contributors} />

  {#if error}
    <div role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
  {/if}

  <div class="flex items-center justify-end gap-2 border-t border-base-300 pt-6">
    {#if changed}
      <span class="text-sm text-warning">Unsaved changes</span>
      <button type="button" class="btn btn-ghost" onclick={() => (form = $state.snapshot(site.config.podcast))}>Discard</button>
    {:else if savedAt}
      <span class="text-sm text-success">Saved</span>
    {/if}
    <button id="save-podcast-settings" class="btn btn-primary" disabled={!changed || saving}>
      {#if saving}<span class="loading loading-spinner loading-sm"></span>{/if}
      Save
    </button>
  </div>
</form>

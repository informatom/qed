<script lang="ts">
  import { ask } from "@tauri-apps/plugin-dialog";
  import Plus from "@lucide/svelte/icons/plus";
  import UserRound from "@lucide/svelte/icons/user-round";
  import ImageField from "$lib/ImageField.svelte";
  import MarkdownEditor from "$lib/MarkdownEditor.svelte";
  import { trackUnsaved } from "$lib/unsaved.svelte";
  import { commands, mediaUrl, type Contributor, type Site } from "$lib/site";

  interface Props {
    site: Site;
    /** After a save or delete, with the new list. */
    onchanged: (contributors: Contributor[]) => void;
  }

  let { site, onchanged }: Props = $props();

  // The one being edited: a copy, or a new one (empty id).
  let editing = $state<Contributor | null>(null);
  let saving = $state(false);
  let error = $state<string | null>(null);

  function blank(): Contributor {
    return { id: "", name: "", photo: null, links: [], bio: "" };
  }

  function open(contributor: Contributor | null) {
    error = null;
    editing = $state.snapshot(contributor ?? blank());
  }

  let original = $derived(editing ? (site.contributors.find((c) => c.id === editing?.id) ?? blank()) : null);
  let changed = $derived(!!editing && JSON.stringify(editing) !== JSON.stringify(original));
  trackUnsaved(() => (editing?.name ? `Contributor “${editing.name}”` : "New contributor"), () => changed);

  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (!editing) return;
    saving = true;
    error = null;
    try {
      const saved = await commands.saveContributor(site.root, editing);
      const others = site.contributors.filter((c) => c.id !== saved.id);
      onchanged([...others, saved].sort((a, b) => a.name.localeCompare(b.name)));
      editing = $state.snapshot(saved);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function remove() {
    if (!editing?.id) return;
    if (!(await ask(`Delete ${editing.name}?`, { title: "Delete contributor", kind: "warning" }))) return;
    try {
      await commands.deleteContributor(site.root, editing.id);
      const id = editing.id;
      editing = null;
      onchanged(site.contributors.filter((c) => c.id !== id));
    } catch (e) {
      error = String(e);
    }
  }
</script>

<!-- Side by side: everyone on the left, the one being edited on the right. -->
<div id="contributors" class="grid gap-6 lg:grid-cols-[18rem_1fr]">
  <div class="flex flex-col gap-2">
    <button type="button" id="new-contributor" class="btn btn-sm btn-primary self-start" onclick={() => open(null)}>
      <Plus class="size-4" /> New contributor
    </button>
    {#if site.contributors.length === 0}
      <p class="text-sm text-base-content/60">
        No contributors yet. Add the people who make the podcast once; episodes and the podcast's team then pick them.
      </p>
    {/if}
    <ul class="flex flex-col gap-1">
      {#each site.contributors as contributor (contributor.id)}
        <li>
          <button type="button" id="contributor-{contributor.id}" onclick={() => open(contributor)}
            class={["flex w-full items-center gap-3 rounded-field px-2 py-1.5 text-left",
              editing?.id === contributor.id ? "bg-primary/10" : "hover:bg-base-200"]}>
            {#if contributor.photo}
              <img src={mediaUrl(site.root, contributor.photo)} alt="" class="size-9 rounded-full object-cover" />
            {:else}
              <span class="flex size-9 items-center justify-center rounded-full bg-base-200"><UserRound class="size-5 text-base-content/50" /></span>
            {/if}
            <span class="flex flex-col leading-tight">
              <span class="font-medium">{contributor.name}</span>
              <span class="font-mono text-xs text-base-content/60">/contributors/{contributor.id}</span>
            </span>
          </button>
        </li>
      {/each}
    </ul>
  </div>

  {#if editing}
    {@const person = editing}
    <form id="contributor-form" class="@container flex flex-col gap-4 rounded-box border border-base-300 p-5" onsubmit={save}>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Name</span>
        <input id="contributor-name" class="input w-full" bind:value={person.name} required />
      </label>
      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Photo <span class="font-normal text-base-content/50">· square works best</span></span>
        <ImageField siteRoot={site.root} bind:value={person.photo} />
      </div>
      <div class="flex flex-col gap-2">
        <span class="text-sm font-medium">Links <span class="font-normal text-base-content/50">· website, Mastodon, …</span></span>
        {#each person.links as link, index (index)}
          <div class="flex gap-2">
            <input class="input w-48" bind:value={link.title} placeholder="Text, e.g. Website" aria-label="Link text" />
            <input class="input w-full" bind:value={link.url} placeholder="https://…" aria-label="Address" />
            <button type="button" class="btn btn-ghost" onclick={() => (person.links = person.links.filter((_, i) => i !== index))}>Remove</button>
          </div>
        {/each}
        <div>
          <button type="button" class="btn btn-sm" onclick={() => (person.links = [...person.links, { title: null, url: "" }])}>Add a link</button>
        </div>
      </div>
      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">About them <span class="font-normal text-base-content/50">· shown on their page</span></span>
        <MarkdownEditor bind:value={person.bio} placeholder="A few words about them" minHeight="120px" />
      </div>

      {#if error}<div role="alert" class="alert alert-error alert-soft text-sm">{error}</div>{/if}

      <div class="flex items-center justify-between gap-2 border-t border-base-300 pt-4">
        {#if person.id}
          <button type="button" class="btn btn-ghost text-error" onclick={remove}>Delete</button>
        {:else}
          <span></span>
        {/if}
        <div class="flex items-center gap-2">
          {#if changed}<span class="text-sm text-warning">Unsaved changes</span>{/if}
          <button id="save-contributor" class="btn btn-primary" disabled={!changed || saving}>
            {#if saving}<span class="loading loading-spinner loading-sm"></span>{/if}
            Save
          </button>
        </div>
      </div>
    </form>
  {:else}
    <p class="text-sm text-base-content/60">Pick someone on the left, or add a new contributor.</p>
  {/if}
</div>

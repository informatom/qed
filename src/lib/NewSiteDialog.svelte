<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { documentDir, join } from "@tauri-apps/api/path";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import { commands } from "$lib/site";

  interface Props {
    /** Called with the new site's folder, to open it. */
    oncreated: (path: string) => void;
    onclose: () => void;
  }

  let { oncreated, onclose }: Props = $props();

  let dialog: HTMLDialogElement;
  let title = $state("");
  let language = $state("en");
  let podcast = $state(false);
  /** The folder the site's own folder goes into. */
  let location = $state("");
  let folderName = $state("");
  /** Whether the folder name was typed, so the title stops changing it. */
  let nameTyped = $state(false);
  let path = $state("");
  let creating = $state(false);
  let error = $state<string | null>(null);

  onMount(async () => {
    dialog.showModal();
    try {
      location = await documentDir();
    } catch {
      // No documents folder; the user picks a location.
    }
  });

  // The folder name follows the title until it's typed in.
  $effect(() => {
    const current = title;
    if (nameTyped) return;
    commands.slugify(current).then((slug) => {
      if (current === title && !nameTyped) folderName = slug;
    });
  });

  $effect(() => {
    const parent = location;
    const name = folderName.trim();
    if (!parent || !name) {
      path = "";
      return;
    }
    join(parent, name).then((joined) => {
      if (parent === location && name === folderName.trim()) path = joined;
    });
  });

  async function chooseLocation() {
    const picked = await open({ directory: true, title: "Where the new site's folder goes", defaultPath: location || undefined });
    if (typeof picked === "string") location = picked;
  }

  async function create(event: SubmitEvent) {
    event.preventDefault();
    if (!path) return;
    creating = true;
    error = null;
    try {
      await commands.createSite(path, title, language, podcast);
      dialog.close();
      oncreated(path);
    } catch (e) {
      error = String(e);
    } finally {
      creating = false;
    }
  }
</script>

<dialog bind:this={dialog} id="new-site-dialog" class="modal" onclose={onclose}>
  <form class="modal-box flex max-w-2xl flex-col gap-4" onsubmit={create}>
    <h2 class="text-xl font-semibold">New site</h2>

    <div class="grid gap-4 sm:grid-cols-[1fr_8rem]">
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Title</span>
        <input id="new-site-title" class="input w-full" bind:value={title} required />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Language <span class="font-normal text-base-content/50">· en, de</span></span>
        <input id="new-site-language" class="input w-full" bind:value={language} />
      </label>
    </div>

    <label class="flex items-start gap-3">
      <input id="new-site-podcast" type="checkbox" class="toggle mt-0.5" bind:checked={podcast} />
      <span class="text-sm">
        This site is a podcast
        <span class="block text-base-content/60">Episodes, feeds, and the latest episode with a Subscribe button on the start page. Can be switched on later in the Podcast tab, too.</span>
      </span>
    </label>

    <div class="grid gap-4 sm:grid-cols-[1fr_14rem]">
      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Location</span>
        <div class="flex gap-2">
          <code id="new-site-location" class="input min-w-0 flex-1 truncate font-mono text-sm">{location || "…"}</code>
          <button type="button" id="new-site-choose" class="btn" onclick={chooseLocation}>
            <FolderOpen class="size-4" /> Choose…
          </button>
        </div>
      </div>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Folder name</span>
        <input id="new-site-folder" class="input w-full font-mono" bind:value={folderName} oninput={() => (nameTyped = true)} required />
      </label>
    </div>

    <p class="text-sm text-base-content/60">
      The site goes into <code id="new-site-path" class="font-mono">{path || "…"}</code>, with a start page, empty
      <code>media/</code> and <code>assets/</code> folders, and a version history. The folder must be new or empty.
    </p>

    {#if error}
      <div id="new-site-error" role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
    {/if}

    <div class="modal-action">
      <button type="button" class="btn btn-ghost" onclick={() => dialog.close()}>Cancel</button>
      <button id="create-site" class="btn btn-primary" disabled={!title.trim() || !path || creating}>
        {#if creating}<span class="loading loading-spinner loading-sm"></span>{/if}
        Create site
      </button>
    </div>
  </form>
  <form method="dialog" class="modal-backdrop"><button>close</button></form>
</dialog>

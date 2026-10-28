<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { documentDir, join } from "@tauri-apps/api/path";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import RemoteKey from "$lib/RemoteKey.svelte";
  import TransferProgress from "$lib/TransferProgress.svelte";
  import { commands, transferChannel, type Transfer } from "$lib/site";

  interface Props {
    /** Called with the site's new folder, to open it. */
    oncreated: (path: string) => void;
    onclose: () => void;
  }

  let { oncreated, onclose }: Props = $props();

  let dialog: HTMLDialogElement;
  let url = $state("");
  /** The folder the site's own folder goes into. */
  let location = $state("");
  let folderName = $state("");
  /** Whether the folder name was typed, so the address stops changing it. */
  let nameTyped = $state(false);
  let path = $state("");
  let hasKey = $state(false);
  let hostKey = $state<string | null>(null);
  let untrusted = $state<string | null>(null);
  let cloning = $state(false);
  let transfer = $state<Transfer | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    dialog.showModal();
    try {
      location = await documentDir();
    } catch {
      // No documents folder; the user picks a location.
    }
  });

  // The folder name follows the address: the repository's name without .git.
  $effect(() => {
    const name = url.trim().replace(/\/+$/, "").split(/[/:]/).pop()?.replace(/\.git$/, "") ?? "";
    if (!nameTyped) folderName = name;
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
    const picked = await open({ directory: true, title: "Where the site's folder goes", defaultPath: location || undefined });
    if (typeof picked === "string") location = picked;
  }

  async function get(event?: SubmitEvent) {
    event?.preventDefault();
    if (!path || !url.trim()) return;
    cloning = true;
    error = null;
    untrusted = null;
    try {
      const outcome = await commands.cloneSite(url, path, hostKey, transferChannel((t) => (transfer = t)));
      if (outcome.outcome === "untrusted") {
        untrusted = outcome.fingerprint;
        return;
      }
      dialog.close();
      oncreated(path);
    } catch (e) {
      error = String(e);
    } finally {
      cloning = false;
      transfer = null;
    }
  }

  function trust() {
    hostKey = untrusted;
    untrusted = null;
    get();
  }
</script>

<dialog bind:this={dialog} id="clone-site-dialog" class="modal" onclose={onclose}>
  <form class="modal-box flex max-w-2xl flex-col gap-4" onsubmit={get}>
    <h2 class="text-xl font-semibold">Get a site from git</h2>
    <p class="text-sm text-base-content/60">
      For a site that was pushed from another computer (Versions tab → Push and pull): q.e.d. copies it here with all
      its versions, and Push and Pull keep both computers in step.
    </p>

    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">Address</span>
      <input id="clone-url" class="input w-full font-mono" bind:value={url} placeholder="git@codeberg.org:me/my-site.git"
        oninput={() => (hostKey = null)} required />
    </label>

    {#if url.trim()}
      <RemoteKey {url} onkey={(exists) => (hasKey = exists)} />
    {/if}

    <div class="grid gap-4 sm:grid-cols-[1fr_14rem]">
      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Location</span>
        <div class="flex gap-2">
          <code id="clone-location" class="input min-w-0 flex-1 truncate font-mono text-sm">{location || "…"}</code>
          <button type="button" id="clone-choose" class="btn" onclick={chooseLocation}>
            <FolderOpen class="size-4" /> Choose…
          </button>
        </div>
      </div>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Folder name</span>
        <input id="clone-folder" class="input w-full font-mono" bind:value={folderName} oninput={() => (nameTyped = true)} required />
      </label>
    </div>

    <p class="text-sm text-base-content/60">
      The site goes into <code id="clone-path" class="font-mono">{path || "…"}</code>. The folder must be new or empty.
      Publish targets aren't in git; set them up again under Publish.
    </p>

    <TransferProgress {transfer} />
    {#if untrusted}
      <div id="clone-untrusted" role="alert" class="alert alert-warning alert-soft flex flex-col items-start gap-2 text-sm">
        <span>
          First connection to this server. Its key fingerprint is <code class="font-mono break-all">{untrusted}</code>.
          Trust it if it matches what the server's owner shows.
        </span>
        <button type="button" id="clone-trust" class="btn btn-sm btn-warning" onclick={trust}>Trust and get the site</button>
      </div>
    {/if}
    {#if error}
      <div id="clone-error" role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
    {/if}

    <div class="modal-action">
      <button type="button" class="btn btn-ghost" onclick={() => dialog.close()}>Cancel</button>
      <button id="clone-site" class="btn btn-primary" disabled={!url.trim() || !path || !hasKey || cloning}>
        {#if cloning}<span class="loading loading-spinner loading-sm"></span>{/if}
        Get site
      </button>
    </div>
  </form>
  <form method="dialog" class="modal-backdrop"><button>close</button></form>
</dialog>

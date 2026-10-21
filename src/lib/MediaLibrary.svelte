<script lang="ts">
  import { onMount } from "svelte";
  import { ask, open } from "@tauri-apps/plugin-dialog";
  import Trash from "@lucide/svelte/icons/trash";
  import Upload from "@lucide/svelte/icons/upload";
  import { deleteMedia, formatSize, importMedia, listMedia, mediaUrl, type MediaFile, type MediaKind } from "$lib/site";

  interface Props {
    siteRoot: string;
    /** When set, clicking a file picks it instead of showing delete. */
    onpick?: (name: string) => void;
    /** Show only files of this kind. */
    only?: MediaKind;
  }

  let { siteRoot, onpick, only }: Props = $props();

  let files = $state<MediaFile[]>([]);
  let loading = $state(true);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let search = $state("");

  let shown = $derived(
    files.filter(
      (file) => (!only || file.kind === only) && file.name.toLowerCase().includes(search.trim().toLowerCase()),
    ),
  );

  async function refresh() {
    try {
      files = await listMedia(siteRoot);
      error = null;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(refresh);

  async function importFiles() {
    const picked = await open({ multiple: true, title: "Import into the media folder" });
    if (!picked || picked.length === 0) return;
    busy = true;
    try {
      const names = await importMedia(siteRoot, picked);
      await refresh();
      // In the picker, importing one file usually means "use this one".
      if (onpick && names.length === 1) onpick(names[0]);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function remove(file: MediaFile) {
    const sure = await ask(`Delete ${file.name}? This removes the file from the media folder.`, {
      title: "Delete file",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Keep",
    });
    if (!sure) return;
    try {
      await deleteMedia(siteRoot, file.name);
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div id="media-library" class="flex flex-col gap-4">
  <div class="flex flex-wrap items-center gap-3">
    <input id="media-search" class="input w-64" type="search" placeholder="Search by name" bind:value={search} />
    <span class="text-sm text-base-content/60">{shown.length} of {files.length} files</span>
    <div class="flex-1"></div>
    <button type="button" id="import-media" class="btn btn-primary" onclick={importFiles} disabled={busy}>
      {#if busy}<span class="loading loading-spinner loading-sm"></span>{:else}<Upload class="size-4" />{/if}
      Import files…
    </button>
  </div>

  {#if error}
    <div id="media-error" role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
  {/if}

  {#if loading}
    <span class="loading loading-spinner"></span>
  {:else if shown.length === 0}
    <p class="rounded-box border border-dashed border-base-300 p-8 text-center text-base-content/60">
      {files.length === 0 ? "The media folder is empty. Import images or documents to use them on pages." : "No file matches."}
    </p>
  {:else}
    <ul class="grid grid-cols-[repeat(auto-fill,minmax(11rem,1fr))] gap-3">
      {#each shown as file (file.name)}
        <li class="card border border-base-300 bg-base-100">
          <button
            type="button"
            class={["block aspect-[4/3] overflow-hidden rounded-t-box bg-base-200", onpick ? "cursor-pointer" : "cursor-default"]}
            onclick={() => onpick?.(file.name)}
            disabled={!onpick}
            title={onpick ? `Use ${file.name}` : file.name}
          >
            {#if file.kind === "image"}
              <img src={mediaUrl(siteRoot, file.name)} alt="" loading="lazy" class="h-full w-full object-cover" />
            {:else}
              <span class="flex h-full items-center justify-center font-mono text-2xl text-base-content/40 uppercase">
                {file.name.split(".").pop()}
              </span>
            {/if}
          </button>
          <div class="flex flex-col gap-1 p-3">
            <span class="truncate text-sm font-medium" title={file.name}>{file.name}</span>
            <span class="text-xs text-base-content/60">
              {formatSize(file.size)}{#if file.width && file.height} · {file.width}×{file.height}{/if}
            </span>
            {#if !onpick}
              <div class="mt-1 flex items-center justify-between gap-2">
                {#if file.used_by.length}
                  <span class="truncate text-xs text-base-content/60" title={file.used_by.map((p) => `/${p}`).join(", ")}>
                    Used on {file.used_by.length === 1 ? `/${file.used_by[0]}` : `${file.used_by.length} pages`}
                  </span>
                {:else}
                  <span class="text-xs text-warning">Not used</span>
                {/if}
                <button type="button" class="btn btn-ghost btn-xs btn-square text-error" disabled={file.used_by.length > 0}
                  title={file.used_by.length ? "Used on a page; remove it there first" : "Delete"}
                  aria-label="Delete {file.name}" onclick={() => remove(file)}>
                  <Trash class="size-4" />
                </button>
              </div>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

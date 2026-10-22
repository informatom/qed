<script lang="ts">
  import { onMount } from "svelte";
  import { ask, open } from "@tauri-apps/plugin-dialog";
  import FolderInput from "@lucide/svelte/icons/folder-input";
  import Trash from "@lucide/svelte/icons/trash";
  import Upload from "@lucide/svelte/icons/upload";
  import { commands, formatSize, mediaUrl, type Library, type MediaFile, type MediaKind } from "$lib/site";

  interface Props {
    siteRoot: string;
    /** `media/` (images) or `assets/` (files to download). */
    library?: Library;
    /** When set, clicking a file picks it instead of showing delete. */
    onpick?: (name: string) => void;
    /** Show only files of this kind. */
    only?: MediaKind;
    /** Called after files moved to assets/, which changes pages. */
    onmoved?: () => void;
    /** A page has unsaved changes; moving files (which edits pages) waits. */
    editing?: boolean;
  }

  let { siteRoot, library = "media", onpick, only, onmoved, editing = false }: Props = $props();
  let folder = $derived(library === "media" ? "media" : "assets");

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
      files = await commands.listMedia(siteRoot, library);
      error = null;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(refresh);

  async function importFiles() {
    const picked = await open({
      multiple: true,
      title: library === "media" ? "Import images" : "Import files to download",
      filters: library === "media" ? [{ name: "Images", extensions: ["jpg", "jpeg", "png", "gif", "webp", "svg", "avif"] }] : [],
    });
    if (!picked || picked.length === 0) return;
    busy = true;
    try {
      const names = await commands.importMedia(siteRoot, library, picked);
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
    const sure = await ask(`Delete ${file.name}? This removes the file from the ${folder} folder.`, {
      title: "Delete file",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Keep",
    });
    if (!sure) return;
    try {
      await commands.deleteMedia(siteRoot, library, file.name);
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  // Sites from before assets/ kept downloads in media/.
  let misplaced = $derived(library === "media" && !onpick ? files.filter((file) => file.kind !== "image") : []);

  async function moveToAssets() {
    const sure = await ask(
      `Move ${misplaced.length} ${misplaced.length === 1 ? "file" : "files"} to Assets? Pages that use them are updated to match.`,
      { title: "Move to Assets", okLabel: "Move", cancelLabel: "Cancel" },
    );
    if (!sure) return;
    busy = true;
    try {
      await commands.moveDocumentsToAssets(siteRoot);
      await refresh();
      onmoved?.();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div id={library === "media" ? "media-library" : "assets-library"} class="flex flex-col gap-4">
  <div class="flex flex-wrap items-center gap-3">
    <input id="{library}-search" class="input w-64" type="search" placeholder="Search by name" bind:value={search} />
    <span class="text-sm text-base-content/60">{shown.length} of {files.length} files</span>
    <div class="flex-1"></div>
    <button type="button" id="import-{library}" class="btn btn-primary" onclick={importFiles} disabled={busy}>
      {#if busy}<span class="loading loading-spinner loading-sm"></span>{:else}<Upload class="size-4" />{/if}
      {library === "media" ? "Import images…" : "Import files…"}
    </button>
  </div>

  {#if misplaced.length}
    <div id="misplaced-files" role="status" class="alert alert-info alert-soft flex flex-wrap text-sm">
      <span class="flex-1">
        {misplaced.length === 1 ? "1 file here isn't an image" : `${misplaced.length} files here aren't images`}. Files to
        download now go into Assets.{#if editing} Save or discard the page you're editing first.{/if}
      </span>
      <button type="button" id="move-to-assets" class="btn btn-sm" onclick={moveToAssets} disabled={busy || editing}>
        <FolderInput class="size-4" /> Move to Assets
      </button>
    </div>
  {/if}

  {#if error}
    <div id="{library}-error" role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
  {/if}

  {#if loading}
    <span class="loading loading-spinner"></span>
  {:else if shown.length === 0}
    <p class="rounded-box border border-dashed border-base-300 p-8 text-center text-base-content/60">
      {#if files.length > 0}
        No file matches.
      {:else if library === "media"}
        The media folder is empty. Import images to use them on pages.
      {:else}
        No files to download yet. Import PDFs or other files to offer them with a Download block.
      {/if}
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
                {#if file.site_logo}
                  <span class="truncate text-xs text-base-content/60">Site logo{file.used_by.length ? ", used on pages" : ""}</span>
                {:else if file.used_by.length}
                  <span class="truncate text-xs text-base-content/60" title={file.used_by.map((p) => `/${p}`).join(", ")}>
                    Used on {file.used_by.length === 1 ? `/${file.used_by[0]}` : `${file.used_by.length} pages`}
                  </span>
                {:else}
                  <span class="text-xs text-warning">Not used</span>
                {/if}
                <button type="button" class="btn btn-ghost btn-xs btn-square text-error" disabled={file.used_by.length > 0 || file.site_logo}
                  title={file.site_logo
                    ? "The site's logo; choose another one in Site settings first"
                    : file.used_by.length ? "Used on a page; remove it there first" : "Delete"}
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

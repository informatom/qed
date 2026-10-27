<script lang="ts">
  import { onMount } from "svelte";
  import { ask, open } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Folder from "@lucide/svelte/icons/folder";
  import FolderInput from "@lucide/svelte/icons/folder-input";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import Trash from "@lucide/svelte/icons/trash";
  import Upload from "@lucide/svelte/icons/upload";
  import TextViewer from "$lib/TextViewer.svelte";
  import { commands, formatSize, libraryUrl, pageLabel, type Library, type MediaFile, type MediaKind } from "$lib/site";

  interface Props {
    siteRoot: string;
    /** `media/` (images), `assets/` (files to download) or `static/` (copied as they are). */
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
  let folder = $derived(library);
  let isStatic = $derived(library === "static");

  let files = $state<MediaFile[]>([]);
  let loading = $state(true);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let search = $state("");
  /** The text file open in the viewer. */
  let reading = $state<string | null>(null);
  /** In static/: the folder being shown, `""` or `talks/2019/`. */
  let dir = $state("");
  let crumbs = $derived(
    dir
      .split("/")
      .filter(Boolean)
      .map((name, index, all) => ({ name, path: all.slice(0, index + 1).join("/") + "/" })),
  );

  let shown = $derived(
    files.filter(
      (file) => (!only || file.kind === only) && file.name.toLowerCase().includes(search.trim().toLowerCase()),
    ),
  );

  /** A file, or in static/ a folder (a slide show has many files). */
  interface Card extends MediaFile {
    /** What the card shows: the name inside the current folder. */
    label: string;
    /** Files inside, for a folder. */
    count?: number;
  }

  // static/ shows one folder at a time; a search looks through all of them.
  let cards = $derived.by((): Card[] => {
    if (!isStatic || search.trim()) return shown.map((file) => ({ ...file, label: file.name }));
    const result: Card[] = [];
    const folders = new Map<string, Card>();
    for (const file of shown) {
      if (!file.name.startsWith(dir)) continue;
      const rest = file.name.slice(dir.length);
      const slash = rest.indexOf("/");
      if (slash < 0) {
        result.push({ ...file, label: rest });
        continue;
      }
      const label = rest.slice(0, slash);
      let card = folders.get(label);
      if (!card) {
        const name = dir + label;
        card = { ...file, name, label, size: 0, kind: "other", width: null, height: null, used_by: [], count: 0 };
        folders.set(label, card);
        result.push(card);
      }
      card.size += file.size;
      card.count = (card.count ?? 0) + 1;
      for (const page of file.used_by) if (!card.used_by.includes(page)) card.used_by.push(page);
    }
    // Folders first, like a file manager.
    return result.sort((a, b) => Number(b.count !== undefined) - Number(a.count !== undefined));
  });

  function openFolder(path: string) {
    dir = path;
    search = "";
  }

  async function refresh() {
    try {
      files = await commands.listMedia(siteRoot, library);
      // A folder whose last file was deleted is gone; go up to one that's left.
      while (dir && !files.some((file) => file.name.startsWith(dir))) {
        dir = dir.slice(0, dir.slice(0, -1).lastIndexOf("/") + 1);
      }
      error = null;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(refresh);

  async function importFiles() {
    await importPaths(
      await open({
        multiple: true,
        title: library === "media" ? "Import images" : isStatic ? "Import files to copy as they are" : "Import files to download",
        filters: library === "media" ? [{ name: "Images", extensions: ["jpg", "jpeg", "png", "gif", "webp", "svg", "avif"] }] : [],
      }),
    );
  }

  /** static/ takes whole folders too, e.g. a slide show. */
  async function importFolder() {
    await importPaths(await open({ directory: true, multiple: true, title: "Import folders to copy as they are" }));
  }

  async function importPaths(picked: string[] | null) {
    if (!picked || picked.length === 0) return;
    busy = true;
    try {
      const names = isStatic
        ? await commands.importStatic(siteRoot, dir, picked)
        : await commands.importMedia(siteRoot, library, picked);
      await refresh();
      // In the picker, importing one file usually means "use this one".
      if (onpick && names.length === 1) onpick(names[0]);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function remove(file: Card) {
    const what = file.count === undefined ? "the file" : `the folder and its ${file.count} files`;
    const sure = await ask(`Delete ${file.name}? This removes ${what} from the ${folder} folder.`, {
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

<div id="{library}-library" class="flex flex-col gap-4">
  <div class="flex flex-wrap items-center gap-3">
    <input id="{library}-search" class="input w-64" type="search" placeholder="Search by name" bind:value={search} />
    <span class="text-sm text-base-content/60">{shown.length} of {files.length} files</span>
    <div class="flex-1"></div>
    {#if isStatic}
      <button type="button" id="show-static-folder" class="btn"
        onclick={() => revealItemInDir(`${siteRoot}/static${dir ? `/${dir.slice(0, -1)}` : ""}`)}
        disabled={files.length === 0}>
        <FolderOpen class="size-4" /> Show in file manager
      </button>
      <button type="button" id="import-static-folder" class="btn" onclick={importFolder} disabled={busy}>
        <FolderInput class="size-4" /> Import folder…
      </button>
    {/if}
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

  {#if isStatic && !onpick}
    <p id="static-help" class="text-sm text-base-content/70">
      Files here go into the built site as they are, at the same address: <code>static/pdf/talk.pdf</code> becomes
      <code>/pdf/talk.pdf</code>. For slide shows, verification files like <code>keybase.txt</code>, or
      <code>.well-known/</code>. Link to them as <code>/pdf/talk.pdf</code>.
    </p>
  {/if}

  {#if isStatic && !search.trim()}
    <nav id="static-path" class="flex flex-wrap items-center gap-1 text-sm" aria-label="Folder">
      <button type="button" class={["btn btn-ghost btn-xs font-mono", !dir && "btn-active"]} onclick={() => openFolder("")}>
        static
      </button>
      {#each crumbs as crumb (crumb.path)}
        <ChevronRight class="size-3.5 text-base-content/40" />
        <button type="button" class={["btn btn-ghost btn-xs font-mono", crumb.path === dir && "btn-active"]}
          onclick={() => openFolder(crumb.path)}>{crumb.name}</button>
      {/each}
    </nav>
  {/if}

  {#if error}
    <div id="{library}-error" role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
  {/if}

  {#if loading}
    <span class="loading loading-spinner"></span>
  {:else if cards.length === 0}
    <p class="rounded-box border border-dashed border-base-300 p-8 text-center text-base-content/60">
      {#if files.length > 0}
        No file matches.
      {:else if library === "media"}
        The media folder is empty. Import images to use them on pages.
      {:else if isStatic}
        No static files yet. Import files or whole folders.
      {:else}
        No files to download yet. Import PDFs or other files to offer them with a Download block.
      {/if}
    </p>
  {:else}
    <ul class="grid grid-cols-[repeat(auto-fill,minmax(11rem,1fr))] gap-3">
      {#each cards as file (file.name)}
        <li class="card border border-base-300 bg-base-100">
          <button
            type="button"
            class={["block aspect-[4/3] overflow-hidden rounded-t-box bg-base-200",
              onpick || file.count !== undefined || file.kind === "text" ? "cursor-pointer" : "cursor-default"]}
            onclick={() =>
              file.count !== undefined ? openFolder(`${file.name}/`) : onpick ? onpick(file.name) : (reading = file.name)}
            disabled={!onpick && file.count === undefined && file.kind !== "text"}
            title={file.count !== undefined
              ? `Open ${file.label}`
              : onpick ? `Use ${file.name}` : file.kind === "text" ? `Read ${file.name}` : file.name}
          >
            {#if file.count !== undefined}
              <span class="flex h-full flex-col items-center justify-center gap-1 text-base-content/40">
                <Folder class="size-10" />
                <span class="text-xs">{file.count} {file.count === 1 ? "file" : "files"}</span>
              </span>
            {:else if file.kind === "image"}
              <img src={libraryUrl(siteRoot, library, file.name)} alt="" loading="lazy" class="h-full w-full object-cover" />
            {:else}
              <span class="flex h-full items-center justify-center font-mono text-2xl text-base-content/40 uppercase">
                {file.name.split(".").pop()}
              </span>
            {/if}
          </button>
          <div class="flex flex-col gap-1 p-3">
            <span class="truncate text-sm font-medium" title={file.name}>{file.label}{file.count !== undefined ? "/" : ""}</span>
            {#if isStatic}
              <span class="truncate font-mono text-xs text-base-content/60" title="Address on the site">
                /{file.name}{file.count !== undefined ? "/" : ""}
              </span>
            {/if}
            <span class="text-xs text-base-content/60">
              {formatSize(file.size)}{#if file.width && file.height} · {file.width}×{file.height}{/if}
            </span>
            {#if !onpick}
              <div class="mt-1 flex items-center justify-between gap-2">
                {#if file.site_logo}
                  <span class="truncate text-xs text-base-content/60">Site logo{file.used_by.length ? ", used on pages" : ""}</span>
                {:else if file.used_by.length}
                  <span class="truncate text-xs text-base-content/60" title={file.used_by.map(pageLabel).join(", ")}>
                    Used on {file.used_by.length === 1 ? pageLabel(file.used_by[0]) : `${file.used_by.length} pages`}
                  </span>
                {:else if isStatic}
                  <!-- Static files often aren't linked from pages at all (keybase.txt). -->
                  <span class="text-xs text-base-content/60">Not linked</span>
                {:else}
                  <span class="text-xs text-warning">Not used</span>
                {/if}
                <button type="button" class="btn btn-ghost btn-xs btn-square text-error" disabled={file.used_by.length > 0 || file.site_logo}
                  title={file.site_logo
                    ? "The site's logo; choose another one in Site settings first"
                    : file.used_by.length ? "Linked from a page; remove the link there first" : "Delete"}
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

{#if reading}
  <TextViewer {siteRoot} {library} name={reading} onclose={() => (reading = null)} />
{/if}

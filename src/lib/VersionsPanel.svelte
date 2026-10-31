<script lang="ts">
  import ManualLink from "$lib/ManualLink.svelte";
  import { onMount } from "svelte";
  import History from "@tabler/icons-svelte-runes/icons/history";
  import Save from "@tabler/icons-svelte-runes/icons/device-floppy";
  import Undo from "@tabler/icons-svelte-runes/icons/arrow-back-up";
  import { ask } from "@tauri-apps/plugin-dialog";
  import RemotePanel from "$lib/RemotePanel.svelte";
  import { commands, formatTime, versionLabel, type DeletedPage, type HistoryStatus, type Version } from "$lib/site";

  interface Props {
    siteRoot: string;
    /** A page has unsaved changes, which a version wouldn't include. */
    editing?: boolean;
    /** Called after a deleted page came back, so the page list can update. */
    onrestored?: () => void;
  }

  let { siteRoot, editing = false, onrestored }: Props = $props();
  let deleted = $state<DeletedPage[]>([]);

  let status = $state<HistoryStatus | null>(null);
  let versions = $state<Version[]>([]);
  // The newest version, if it's still open (only the newest can be).
  let open = $derived(versions[0]?.open ? versions[0] : null);
  let note = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);
  let message = $state<string | null>(null);

  async function refresh() {
    try {
      status = await commands.historyStatus(siteRoot);
      versions = status.tracked ? await commands.listVersions(siteRoot, null) : [];
      deleted = status.tracked ? await commands.deletedPages(siteRoot) : [];
      error = null;
    } catch (e) {
      error = String(e);
    }
  }

  onMount(refresh);

  async function restore(page: DeletedPage) {
    busy = true;
    message = null;
    try {
      await commands.restorePage(siteRoot, page.file, page.version);
      message = `“${page.title}” is back, as it was in its last version.`;
      await refresh();
      onrestored?.();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  // The whole site back to an earlier version, as a new version on top.
  async function goBack(version: Version) {
    const sure = await ask(
      `Put the whole site back as it was in “${version.summary}” (${formatTime(version.time)})?\n\n` +
        "Pages, media and files change to that state. The present is saved as a version first, so you can come back to it the same way.",
      { title: "Go back to this version", kind: "warning", okLabel: "Go back", cancelLabel: "Cancel" },
    );
    if (!sure) return;
    busy = true;
    message = null;
    try {
      const done = await commands.restoreSite(siteRoot, version.id);
      message = `The site is back as it was in “${version.summary}”. Saved as “${done.summary}”.`;
      await refresh();
      onrestored?.();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function run(action: () => Promise<Version | null>) {
    busy = true;
    message = null;
    try {
      const saved = await action();
      message = saved ? `Saved “${saved.summary}”.` : "Nothing changed since the last version.";
      note = "";
      await refresh();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div id="versions-panel" class="flex flex-col gap-6">
  {#if error}
    <div role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
  {/if}

  {#if !status}
    <span class="loading loading-spinner"></span>
  {:else if !status.tracked}
    <section class="flex max-w-2xl flex-col items-start gap-3 rounded-box border border-dashed border-base-300 p-6">
      <h2 class="text-lg font-semibold">No version history yet</h2>
      <p class="text-sm text-base-content/70">
        q.e.d. can keep versions of this site, so you can see what changed and bring back an earlier version of any
        page. It uses git, in this folder; the built site and your publish targets are left out.
      </p>
      <button type="button" id="start-history" class="btn btn-primary" disabled={busy}
        onclick={() => run(() => commands.startHistory(siteRoot))}>
        <History class="size-4" /> Start version history
      </button>
    </section>
  {:else}
    <div class="grid items-start gap-8 xl:grid-cols-[minmax(20rem,28rem)_1fr]">
      <section class="flex flex-col gap-3">
        <h2 class="text-lg font-semibold">Save a version</h2>
        {#if status.shared_with}
          <p class="text-sm text-base-content/60">
            This folder is part of the git repository at <code class="font-mono">{status.shared_with}</code>. Versions
            include only this folder; other changes there are left alone.
          </p>
        {/if}
        {#if open}
          <p class="text-sm">
            Your saves since {formatTime(open.started)} are kept in the open version, last saved
            {formatTime(open.time)}. Close it with a note, so the next saves start a new one.
          </p>
        {/if}
        {#if status.changes.length}
          <details class="text-sm">
            <summary class="cursor-pointer">
              {status.changes.length === 1 ? "1 file changed" : `${status.changes.length} files changed`} since the last
              version
            </summary>
            <ul class="mt-2 font-mono text-xs text-base-content/70">
              {#each status.changes as file (file)}<li>{file}</li>{/each}
            </ul>
          </details>
        {:else if !open}
          <p class="text-sm text-base-content/60">Nothing changed since the last version.</p>
        {/if}
        {#if editing}
          <p class="text-sm text-warning">The page you're editing has unsaved changes; save them first to include them.</p>
        {/if}
        <form class="flex gap-2" onsubmit={(event) => {
          event.preventDefault();
          run(() => commands.saveVersion(siteRoot, note));
        }}>
          <input id="version-note" class="input flex-1" placeholder="What changed? (optional)" bind:value={note} />
          <button id="save-version" class="btn btn-primary" disabled={busy || (!open && status.changes.length === 0)}>
            {#if busy}<span class="loading loading-spinner loading-sm"></span>{:else}<Save class="size-4" />{/if}
            {open ? "Close version" : "Save version"}
          </button>
        </form>
        {#if message}<p class="text-sm text-success">{message}</p>{/if}
        <p class="text-sm text-base-content/60">
          Every save in q.e.d. goes into the open version, so nothing saved is lost. After an hour without saves, or on
          the next day, q.e.d. asks for a note and closes it, so each working session becomes a version of its own.
          Publishing asks what's new, closes it with that note and marks it with the target's name.
        </p>

        {#if status.shared_with}
          <h2 class="mt-4 text-lg font-semibold">Push and pull</h2>
          <p class="text-sm text-base-content/60">
            q.e.d. pushes and pulls only sites that are a git repository of their own. This one is part of
            <code class="font-mono">{status.shared_with}</code>; push and pull that repository with git.
          </p>
        {:else}
          <div class="mt-4">
            <RemotePanel {siteRoot} {note} {editing}
              ondone={() => {
                note = "";
                refresh();
              }}
              onpulled={() => onrestored?.()} />
          </div>
        {/if}
      </section>

      <section class="flex min-w-0 flex-col gap-3">
        <div class="flex items-center gap-2">
          <h2 class="flex-1 text-lg font-semibold">Versions</h2>
          <ManualLink page="versions/" topic="Versions" />
        </div>
        {#if versions.length === 0}
          <p class="text-sm text-base-content/60">No versions yet.</p>
        {:else}
          <ul id="version-list" class="flex flex-col divide-y divide-base-300 rounded-box border border-base-300">
            {#each versions as version, i (version.id)}
              {@const label = versionLabel(version)}
              <li class="group flex flex-wrap items-center gap-x-3 gap-y-1 px-4 py-2">
                <span class="font-medium">{label.title}</span>
                {#if version.open}<span class="badge badge-warning badge-soft badge-sm">open</span>{/if}
                {#each version.tags as tag (tag)}<span class="badge badge-soft badge-sm">{tag}</span>{/each}
                <span class="ms-auto text-sm text-base-content/60">{label.time} · {version.author}</span>
                {#if i > 0}
                  <!-- The newest is the present; any older one can come back. -->
                  <button type="button" class="btn btn-ghost btn-xs" disabled={busy || editing}
                    title={editing ? "Save or discard the page you're editing first" : "Put the whole site back to this version"}
                    onclick={() => goBack(version)}>
                    <Undo class="size-3.5" /> Go back
                  </button>
                {/if}
              </li>
            {/each}
          </ul>
          <p class="text-sm text-base-content/60">
            “Go back” puts the whole site back to that version, as a new version on top; nothing is lost. For a
            single page, open it and use its Versions tab.
          </p>
        {/if}

        <h2 class="mt-4 text-lg font-semibold">Deleted pages</h2>
        {#if deleted.length === 0}
          <p class="text-sm text-base-content/60">
            None. Pages you delete show up here if they were saved in a version before.
          </p>
        {:else}
          <ul id="deleted-pages" class="flex flex-col divide-y divide-base-300 rounded-box border border-base-300">
            {#each deleted as page (page.file)}
              <li class="flex flex-wrap items-center gap-x-3 gap-y-1 px-4 py-2">
                <span class="font-medium">{page.title}</span>
                <span class="font-mono text-xs text-base-content/60">{page.file}</span>
                <span class="ms-auto text-sm text-base-content/60">
                  {page.deleted_at === null ? "deleted since the last version" : `deleted ${formatTime(page.deleted_at)}`}
                </span>
                <button type="button" class="btn btn-sm" disabled={busy} onclick={() => restore(page)}>
                  <Undo class="size-4" /> Restore
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    </div>
  {/if}
</div>

<script lang="ts">
  import { onMount } from "svelte";
  import History from "@lucide/svelte/icons/history";
  import Save from "@lucide/svelte/icons/save";
  import { commands, formatTime, type HistoryStatus, type Version } from "$lib/site";

  interface Props {
    siteRoot: string;
    /** A page has unsaved changes, which a version wouldn't include. */
    editing?: boolean;
  }

  let { siteRoot, editing = false }: Props = $props();

  let status = $state<HistoryStatus | null>(null);
  let versions = $state<Version[]>([]);
  let note = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);
  let message = $state<string | null>(null);

  async function refresh() {
    try {
      status = await commands.historyStatus(siteRoot);
      versions = status.tracked ? await commands.listVersions(siteRoot, null) : [];
      error = null;
    } catch (e) {
      error = String(e);
    }
  }

  onMount(refresh);

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
        qed can keep versions of this site, so you can see what changed and bring back an earlier version of any
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
        {:else}
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
          <button id="save-version" class="btn btn-primary" disabled={busy || status.changes.length === 0}>
            {#if busy}<span class="loading loading-spinner loading-sm"></span>{:else}<Save class="size-4" />{/if}
            Save version
          </button>
        </form>
        {#if message}<p class="text-sm text-success">{message}</p>{/if}
        <p class="text-sm text-base-content/60">
          Publishing saves a version by itself and marks it with the target's name.
        </p>
      </section>

      <section class="flex min-w-0 flex-col gap-3">
        <h2 class="text-lg font-semibold">Versions</h2>
        {#if versions.length === 0}
          <p class="text-sm text-base-content/60">No versions yet.</p>
        {:else}
          <ul id="version-list" class="flex flex-col divide-y divide-base-300 rounded-box border border-base-300">
            {#each versions as version (version.id)}
              <li class="flex flex-wrap items-baseline gap-x-3 gap-y-1 px-4 py-2">
                <span class="font-medium">{version.summary}</span>
                {#each version.tags as tag (tag)}<span class="badge badge-soft badge-sm">{tag}</span>{/each}
                <span class="ms-auto text-sm text-base-content/60">{formatTime(version.time)} · {version.author}</span>
              </li>
            {/each}
          </ul>
          <p class="text-sm text-base-content/60">
            To bring back an earlier version of a page, open the page and use its Versions tab.
          </p>
        {/if}
      </section>
    </div>
  {/if}
</div>

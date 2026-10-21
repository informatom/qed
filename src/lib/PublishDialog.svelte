<script lang="ts">
  import { onMount } from "svelte";
  import Upload from "@lucide/svelte/icons/upload";
  import {
    commands,
    formatTime,
    needsNote,
    type HistoryStatus,
    type SincePublish,
    type Site,
    type Target,
  } from "$lib/site";

  interface Props {
    site: Site;
    target: Target;
    /** Called with the note (`null` if none was needed) to start publishing. */
    onpublish: (note: string | null) => void;
    onclose: () => void;
  }

  let { site, target, onpublish, onclose }: Props = $props();

  let dialog: HTMLDialogElement;
  let status = $state<HistoryStatus | null>(null);
  /** What's new on this target: changes since its last publish. */
  let since = $state<SincePublish | null>(null);
  let note = $state("");
  let error = $state<string | null>(null);

  let noteNeeded = $derived(status !== null && needsNote(status));
  let port = $derived(target.port !== 22 ? `:${target.port}` : "");

  onMount(async () => {
    dialog.showModal();
    try {
      [status, since] = await Promise.all([
        commands.historyStatus(site.root),
        commands.sincePublish(site.root, target.name),
      ]);
    } catch (e) {
      error = String(e);
    }
  });

  function publish(event: SubmitEvent) {
    event.preventDefault();
    const text = note.trim();
    if (noteNeeded && !text) return;
    dialog.close();
    onpublish(noteNeeded ? text : null);
  }
</script>

<dialog bind:this={dialog} id="publish-dialog" class="modal" onclose={onclose}>
  <form class="modal-box flex max-w-2xl flex-col gap-4" onsubmit={publish}>
    <h2 class="text-xl font-semibold">Publish {site.config.title} to {target.name}</h2>
    <!-- Publishing changes a live website, and the target cards look alike: say exactly where this goes. -->
    <p class="font-mono text-sm text-base-content/70">{target.user}@{target.host}{port}:{target.path}</p>

    {#if error}
      <div role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
    {:else if !status}
      <span class="loading loading-spinner"></span>
    {:else if noteNeeded}
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">What's new on the site?</span>
        <textarea id="publish-note" class="textarea w-full" rows="3" bind:value={note} required></textarea>
        <span class="text-sm text-base-content/60">
          This closes the open version with your note{status.changes.length
            ? ` (${status.changes.length === 1 ? "1 file" : `${status.changes.length} files`} changed outside it)`
            : ""}; the Versions tab shows it marked with {target.name}.
        </span>
      </label>
    {:else if status.tracked}
      <p class="text-sm text-base-content/70">No unsaved changes: the last version is published as it is.</p>
    {/if}

    {#if status?.tracked && since}
      <!-- Versions and publishes are two things: a version saved since the
           last publish is still news for this target. -->
      <div id="publish-since" class="text-sm">
        {#if since.published_at === null}
          <p>This is the first publish to {target.name} from this history: the whole site goes up.</p>
        {:else if since.changes.length}
          <details>
            <summary class="cursor-pointer">
              {since.changes.length === 1 ? "1 file" : `${since.changes.length} files`} changed since the last publish to
              {target.name} ({formatTime(since.published_at)})
            </summary>
            <ul class="mt-2 font-mono text-xs text-base-content/70">
              {#each since.changes as file (file)}<li>{file}</li>{/each}
            </ul>
          </details>
        {:else}
          <p class="text-base-content/70">
            Nothing changed since the last publish to {target.name} ({formatTime(since.published_at)}). Publishing again
            uploads only what differs on the server.
          </p>
        {/if}
      </div>
    {/if}

    <div class="modal-action">
      <button type="button" class="btn btn-ghost" onclick={() => dialog.close()}>Cancel</button>
      <button id="publish-confirm" class="btn btn-primary" disabled={!status || (noteNeeded && !note.trim())}>
        <Upload class="size-4" /> Publish to {target.name}
      </button>
    </div>
  </form>
  <form method="dialog" class="modal-backdrop"><button>close</button></form>
</dialog>

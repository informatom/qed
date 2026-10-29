<script lang="ts">
  import { commands, formatTime, type RestingVersion } from "$lib/site";

  interface Props {
    /** The open site. */
    siteRoot: string;
    /** After the open version was closed, so the Versions tab can reload. */
    onclosed?: () => void;
  }

  let { siteRoot, onclosed }: Props = $props();

  let dialog: HTMLDialogElement;
  let resting = $state<RestingVersion | null>(null);
  let note = $state("");
  let saving = $state(false);
  let error = $state<string | null>(null);
  /** The last save of the version "Use suggested note" was clicked for: don't ask again for it. */
  let dismissed: number | null = null;

  // When an open version has rested an hour, or since an earlier day, ask
  // for a note for it: when the site opens, and then once a minute, so the
  // question waits for whoever comes back after a break. If nobody answers,
  // the next save closes it with the note made from its times (history.rs).
  $effect(() => {
    const root = siteRoot;
    dismissed = null;
    check(root);
    const timer = setInterval(() => check(root), 60_000);
    return () => clearInterval(timer);
  });

  async function check(root: string) {
    // Not over another dialog, and not twice.
    if (dialog.open || document.querySelector("dialog[open]")) return;
    try {
      const found = await commands.restingVersion(root);
      if (root !== siteRoot || !found || found.last_save === dismissed || dialog.open) return;
      resting = found;
      note = found.note;
      error = null;
      dialog.showModal();
    } catch {
      // No history, or git can't be read: nothing to ask.
    }
  }

  async function save(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    error = null;
    try {
      await commands.closeOpenVersion(siteRoot, note);
      dialog.close();
      onclosed?.();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  // The next save closes it with the suggested note (history.rs).
  function notNow() {
    dismissed = resting?.last_save ?? null;
    dialog.close();
  }
</script>

<dialog bind:this={dialog} id="resting-version-dialog" class="modal" oncancel={() => (dismissed = resting?.last_save ?? null)}>
  <form class="modal-box flex max-w-xl flex-col gap-4" onsubmit={save}>
    <h2 class="text-xl font-semibold">Name your last changes</h2>
    {#if resting}
      <p class="text-sm">
        You saved from {formatTime(resting.first_save)} to {formatTime(resting.last_save)}. Give these changes a note, so you find
        them again in Versions. Your next save starts a new version.
      </p>
    {/if}
    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">What did you change?</span>
      <!-- svelte-ignore a11y_autofocus -->
      <input id="resting-version-note" class="input w-full" bind:value={note} autofocus
        onfocus={(e) => e.currentTarget.select()} />
    </label>
    {#if error}<div role="alert" class="alert alert-error alert-soft text-sm">{error}</div>{/if}
    <div class="modal-action">
      <button type="button" id="resting-version-later" class="btn btn-ghost" onclick={notNow}
        title="Keep the suggested note; it's given with your next save">Use suggested note</button>
      <button id="resting-version-save" class="btn btn-primary" disabled={saving}>
        {#if saving}<span class="loading loading-spinner loading-xs"></span>{/if}
        Save version
      </button>
    </div>
  </form>
</dialog>

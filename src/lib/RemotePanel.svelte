<script lang="ts">
  import { onMount } from "svelte";
  import ArrowDownToLine from "@tabler/icons-svelte-runes/icons/arrow-bar-to-down";
  import ArrowUpFromLine from "@tabler/icons-svelte-runes/icons/arrow-bar-up";
  import Plus from "@tabler/icons-svelte-runes/icons/plus";
  import Trash from "@tabler/icons-svelte-runes/icons/trash";
  import { ask } from "@tauri-apps/plugin-dialog";
  import RemoteKey from "$lib/RemoteKey.svelte";
  import TransferProgress from "$lib/TransferProgress.svelte";
  import { commands, transferChannel, type RemoteInfo, type RemoteOutcome, type Transfer } from "$lib/site";

  interface Props {
    siteRoot: string;
    /** The note from "Save a version", used if push or pull closes the open version. */
    note: string;
    /** A page has unsaved changes. */
    editing?: boolean;
    /** After a push or pull: the history changed. */
    ondone?: () => void;
    /** After a pull brought files: the site must load again. */
    onpulled?: () => void;
  }

  let { siteRoot, note, editing = false, ondone, onpulled }: Props = $props();

  let remotes = $state<RemoteInfo[]>([]);
  let selectedName = $state<string | null>(null);
  let selected = $derived(remotes.find((r) => r.name === selectedName) ?? null);
  let hasKey = $state(false);
  let adding = $state(false);
  let newName = $state("origin");
  let newUrl = $state("");
  let busy = $state<"push" | "pull" | "check" | null>(null);
  let untrusted = $state<{ fingerprint: string; retry: () => Promise<void> } | null>(null);
  let transfer = $state<Transfer | null>(null);
  let message = $state<string | null>(null);
  let error = $state<string | null>(null);

  async function refresh() {
    try {
      remotes = await commands.listRemotes(siteRoot);
      if (!remotes.some((r) => r.name === selectedName)) {
        selectedName = (remotes.find((r) => r.upstream) ?? remotes[0])?.name ?? null;
      }
      adding = remotes.length === 0;
    } catch (e) {
      error = String(e);
    }
  }

  onMount(refresh);

  async function add(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    try {
      const added = await commands.addRemote(siteRoot, newName, newUrl);
      selectedName = added.name;
      newUrl = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function remove(remote: RemoteInfo) {
    const sure = await ask(
      `Remove the remote “${remote.name}” (${remote.url}) from this site?\n\nNothing on the server changes; you can add it again.`,
      { title: "Remove remote", kind: "warning", okLabel: "Remove", cancelLabel: "Cancel" },
    );
    if (!sure) return;
    try {
      await commands.removeRemote(siteRoot, remote.name);
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  /** Runs push, pull or check; on an untrusted server, asks first and runs it again. */
  async function run(kind: "push" | "pull" | "check") {
    if (!selected) return;
    const name = selected.name;
    busy = kind;
    message = null;
    error = null;
    untrusted = null;
    try {
      const noteOrNull = note.trim() || null;
      const channel = transferChannel((t) => (transfer = t));
      let outcome: RemoteOutcome;
      if (kind === "push") outcome = await commands.pushSite(siteRoot, name, noteOrNull, channel);
      else if (kind === "pull") outcome = await commands.pullSite(siteRoot, name, noteOrNull, channel);
      else outcome = await commands.checkRemote(siteRoot, name, channel);
      if (outcome.outcome === "untrusted") {
        untrusted = {
          fingerprint: outcome.fingerprint,
          retry: async () => {
            await commands.trustRemote(siteRoot, name, outcome.fingerprint);
            untrusted = null;
            await run(kind);
          },
        };
        return;
      }
      message = outcome.report.message;
      if (kind !== "check") ondone?.();
      if (outcome.report.changed) onpulled?.();
    } catch (e) {
      error = String(e);
    } finally {
      busy = null;
      transfer = null;
      await refresh();
    }
  }

  function versions(n: number) {
    return n === 1 ? "1 version" : `${n} versions`;
  }
</script>

<section id="remote-panel" class="flex flex-col gap-3">
  <h2 class="text-lg font-semibold">Push and pull</h2>
  <p class="text-sm text-base-content/60">
    Keep a copy of the site with all its versions in a git repository elsewhere (Codeberg, GitHub, your own server),
    and edit the site on another computer: push here, pull there.
  </p>

  {#if remotes.length > 1}
    <select id="remote-select" class="select w-full" bind:value={selectedName}>
      {#each remotes as remote (remote.name)}<option value={remote.name}>{remote.name} · {remote.url}</option>{/each}
    </select>
  {/if}

  {#if selected}
    <div class="flex flex-col gap-3 rounded-box border border-base-300 p-4">
      <div class="flex flex-wrap items-baseline gap-x-3">
        <span class="font-medium">{selected.name}</span>
        <code class="min-w-0 flex-1 truncate font-mono text-xs text-base-content/60">{selected.url}</code>
        <button type="button" class="btn btn-ghost btn-xs" title="Remove this remote from the site"
          onclick={() => selected && remove(selected)}>
          <Trash class="size-3.5" />
        </button>
      </div>

      <RemoteKey url={selected.url} onkey={(exists) => (hasKey = exists)} />

      {#if hasKey}
        <p id="remote-state" class="text-sm text-base-content/70">
          {#if !selected.has_branch}
            Not pushed yet: {versions(selected.ahead)} to push.
          {:else if selected.ahead === 0 && selected.behind === 0}
            In step with {selected.name}, as of the last push, pull or check.
          {:else}
            {selected.ahead ? `${versions(selected.ahead)} to push` : ""}{selected.ahead && selected.behind ? " · " : ""}{selected.behind
              ? `${versions(selected.behind)} to pull`
              : ""} (as of the last push, pull or check).
          {/if}
        </p>
        <div class="flex flex-wrap gap-2">
          <button type="button" id="remote-push" class="btn btn-primary" disabled={busy !== null || editing}
            onclick={() => run("push")}>
            {#if busy === "push"}<span class="loading loading-spinner loading-sm"></span>{:else}<ArrowUpFromLine class="size-4" />{/if}
            Push
          </button>
          <button type="button" id="remote-pull" class="btn" disabled={busy !== null || editing}
            onclick={() => run("pull")}>
            {#if busy === "pull"}<span class="loading loading-spinner loading-sm"></span>{:else}<ArrowDownToLine class="size-4" />{/if}
            Pull
          </button>
          <button type="button" id="remote-check" class="btn btn-ghost" disabled={busy !== null}
            onclick={() => run("check")}>
            {#if busy === "check"}<span class="loading loading-spinner loading-sm"></span>{/if}
            Check for new versions
          </button>
        </div>
        <TransferProgress {transfer} />
        {#if editing}
          <p class="text-sm text-warning">Save or discard the page you're editing first.</p>
        {/if}
        <p class="text-sm text-base-content/60">
          Both close the open version first, with the note above if there is one. Pull merges versions made on both
          computers; if both changed the same file, it changes nothing and tells you which.
        </p>
      {/if}
    </div>
  {/if}

  {#if untrusted}
    <div id="remote-untrusted" role="alert" class="alert alert-warning alert-soft flex flex-col items-start gap-2 text-sm">
      <span>
        First connection to this server. Its key fingerprint is
        <code class="font-mono break-all">{untrusted.fingerprint}</code>. Trust it if it matches what the server's
        owner shows (Gitea, Codeberg and GitHub publish theirs).
      </span>
      <div class="flex gap-2">
        <button type="button" id="remote-trust" class="btn btn-sm btn-warning" onclick={() => untrusted?.retry()}>Trust</button>
        <button type="button" class="btn btn-sm btn-ghost" onclick={() => (untrusted = null)}>Cancel</button>
      </div>
    </div>
  {/if}
  {#if message}<p id="remote-message" class="text-sm text-success">{message}</p>{/if}
  {#if error}<div id="remote-error" role="alert" class="alert alert-error alert-soft text-sm">{error}</div>{/if}

  {#if adding}
    <form id="remote-add" class="grid gap-2 sm:grid-cols-[8rem_1fr_auto]" onsubmit={add}>
      <input id="remote-name" class="input w-full font-mono" bind:value={newName} placeholder="Name" required />
      <input id="remote-url" class="input w-full font-mono" bind:value={newUrl}
        placeholder="git@codeberg.org:me/my-site.git" required />
      <button class="btn"><Plus class="size-4" /> Add</button>
      <p class="text-sm text-base-content/60 sm:col-span-3">
        The SSH address of an empty repository you made for the site (on Codeberg or GitHub: the “SSH” address under
        Clone).
      </p>
    </form>
  {:else}
    <div>
      <button type="button" id="remote-add-another" class="btn btn-ghost btn-sm" onclick={() => (adding = true)}>
        <Plus class="size-4" /> Add a remote
      </button>
    </div>
  {/if}
</section>

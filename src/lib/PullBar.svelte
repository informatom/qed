<script lang="ts">
  import ArrowDownToLine from "@lucide/svelte/icons/arrow-down-to-line";
  import TransferProgress from "$lib/TransferProgress.svelte";
  import { commands, transferChannel, type RemoteInfo, type Transfer } from "$lib/site";

  interface Props {
    /** The open site; checked once each time a site opens. */
    siteRoot: string;
    /** A page has unsaved changes, so pulling has to wait. */
    editing?: boolean;
    /** After the pull brought files: the site must load again. */
    onpulled: () => void;
  }

  let { siteRoot, editing = false, onpulled }: Props = $props();

  let remote = $state<RemoteInfo | null>(null);
  let message = $state<string | null>(null);
  let pulling = $state(false);
  let transfer = $state<Transfer | null>(null);
  let error = $state<string | null>(null);

  // Ask the remote the branch follows whether it has new versions. Quietly:
  // without a network, a key or a trusted server there's simply no offer.
  $effect(() => {
    const root = siteRoot;
    remote = null;
    message = null;
    error = null;
    (async () => {
      try {
        const remotes = await commands.listRemotes(root);
        const upstream = remotes.find((r) => r.upstream && r.host_key);
        if (!upstream) return;
        const outcome = await commands.checkRemote(root, upstream.name, transferChannel());
        if (root !== siteRoot || outcome.outcome !== "done" || outcome.report.remote.behind === 0) return;
        remote = outcome.report.remote;
        message = outcome.report.message;
      } catch {
        // No offer.
      }
    })();
  });

  async function pull() {
    if (!remote) return;
    pulling = true;
    error = null;
    try {
      const outcome = await commands.pullSite(siteRoot, remote.name, null, transferChannel((t) => (transfer = t)));
      if (outcome.outcome === "untrusted") {
        error = "The server's key isn't trusted; pull in the Versions tab.";
        return;
      }
      remote = null;
      if (outcome.report.changed) onpulled();
    } catch (e) {
      error = String(e);
    } finally {
      pulling = false;
      transfer = null;
    }
  }
</script>

{#if remote}
  <div id="pull-bar" role="status" class="alert alert-info alert-soft flex flex-wrap items-center gap-3">
    <ArrowDownToLine class="size-5" />
    <span class="flex-1">{message} Pull them now?</span>
    <button type="button" id="pull-later" class="btn btn-ghost btn-sm" onclick={() => (remote = null)}>Later</button>
    <button type="button" id="pull-now" class="btn btn-info btn-sm" disabled={pulling || editing}
      title={editing ? "Save or discard the page you're editing first" : undefined} onclick={pull}>
      {#if pulling}<span class="loading loading-spinner loading-sm"></span>{/if}
      Pull
    </button>
    {#if transfer}<div class="w-full"><TransferProgress {transfer} /></div>{/if}
    {#if error}<span class="w-full text-sm text-error">{error}</span>{/if}
  </div>
{/if}

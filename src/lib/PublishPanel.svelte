<script lang="ts">
  import ManualLink from "$lib/ManualLink.svelte";
  import { untrack } from "svelte";
  import { ask } from "@tauri-apps/plugin-dialog";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Pencil from "@tabler/icons-svelte-runes/icons/pencil";
  import PlugZap from "@tabler/icons-svelte-runes/icons/plug-connected";
  import Plus from "@tabler/icons-svelte-runes/icons/plus";
  import Trash from "@tabler/icons-svelte-runes/icons/trash";
  import Upload from "@tabler/icons-svelte-runes/icons/upload";
  import PublishDialog from "$lib/PublishDialog.svelte";
  import TargetForm from "$lib/TargetForm.svelte";
  import {
    commands,
    formatSize,
    newSftpTarget,
    publishSite,
    type Progress,
    type PublishReport,
    type Site,
    type Target,
  } from "$lib/site";

  interface Props {
    site: Site;
    /** Called after targets changed, so the app's copy of the site matches. */
    ontargets: (targets: Target[]) => void;
  }

  let { site, ontargets }: Props = $props();

  let targets = $state(untrack(() => $state.snapshot(site.targets)));
  /** Which target is being edited: its index, -1 for a new one, null for none. */
  let editing = $state<number | null>(null);

  // Per target name: the last test result, a trust question, or an error.
  let messages = $state<Record<string, { kind: "ok" | "error" | "untrusted"; text: string }>>({});
  let busyTarget = $state<string | null>(null);

  /** The target the publish dialog is open for. */
  let confirming = $state<Target | null>(null);
  let publishing = $state<string | null>(null);
  let progress = $state<Progress | null>(null);
  let report = $state<PublishReport | null>(null);


  function changed(updated: Target[]) {
    targets = updated;
    editing = null;
    ontargets(updated);
  }

  async function remove(index: number) {
    const target = targets[index];
    const sure = await ask(`Remove the target ${target.name}? The saved password stays in the keychain.`, {
      title: "Remove target",
      kind: "warning",
      okLabel: "Remove",
      cancelLabel: "Keep",
    });
    if (!sure) return;
    const updated = $state.snapshot(targets).filter((_, i) => i !== index);
    await commands.saveTargets(site.root, updated);
    changed(updated);
  }

  async function test(target: Target) {
    busyTarget = target.name;
    delete messages[target.name];
    try {
      const result = await commands.testTarget(target);
      messages[target.name] =
        result.outcome === "connected"
          ? { kind: "ok", text: result.message }
          : { kind: "untrusted", text: result.fingerprint };
    } catch (e) {
      messages[target.name] = { kind: "error", text: String(e) };
    } finally {
      busyTarget = null;
    }
  }

  async function trust(index: number, fingerprint: string) {
    const updated = $state.snapshot(targets);
    updated[index].host_key = fingerprint;
    await commands.saveTargets(site.root, updated);
    changed(updated);
    await test(updated[index]);
  }

  async function publish(target: Target, note: string | null) {
    publishing = target.name;
    report = null;
    progress = null;
    delete messages[target.name];
    try {
      report = await publishSite(site.root, target.name, note, (p) => (progress = p));
    } catch (e) {
      messages[target.name] = { kind: "error", text: String(e) };
    } finally {
      publishing = null;
    }
  }

  let percent = $derived.by(() => {
    if (!progress) return 0;
    if (progress.total_bytes > 0) return Math.round((progress.done_bytes / progress.total_bytes) * 100);
    return progress.total_files > 0 ? Math.round((progress.done_files / progress.total_files) * 100) : 0;
  });

  const STAGES = {
    connecting: "Connecting…",
    comparing: "Comparing with the server…",
    uploading: "Uploading",
    deleting: "Deleting old files",
    done: "Done",
  };
</script>

<section id="publish-panel" class="flex flex-col gap-3">
    <div class="flex items-center justify-between gap-2">
      <h2 class="flex-1 text-lg font-semibold">Publishing</h2>
      <ManualLink page="publishing/" topic="Publishing" />
      {#if editing === null}
        <button type="button" id="add-target" class="btn btn-sm" onclick={() => (editing = -1)} disabled={!!publishing}>
          <Plus class="size-4" /> Add target
        </button>
      {/if}
    </div>

    <div class="flex flex-col gap-3">
      {#each targets as target, index (target.name)}
        {#if editing === index}
          <TargetForm siteRoot={site.root} {targets} {index} initial={target} onsaved={changed} oncancel={() => (editing = null)} />
        {:else}
          {@const message = messages[target.name]}
          <section class="card border border-base-300">
            <div class="card-body gap-3 p-4">
              <div class="flex flex-wrap items-center gap-x-4 gap-y-2">
                <div class="flex min-w-0 flex-1 flex-col">
                  <span class="font-semibold">{target.name}</span>
                  <span class="font-mono text-sm text-base-content/60">
                    {target.user}@{target.host}{target.port !== 22 ? `:${target.port}` : ""}:{target.path}
                  </span>
                </div>
                <span class="badge badge-ghost badge-sm">{target.auth === "key" ? "key" : "password"}</span>
                <span class={["badge badge-sm", target.host_key ? "badge-success badge-soft" : "badge-warning badge-soft"]}>
                  {target.host_key ? "server trusted" : "not tested yet"}
                </span>
                <div class="flex gap-1">
                  <button type="button" class="btn btn-ghost btn-sm btn-square" title="Edit" aria-label="Edit"
                    onclick={() => (editing = index)} disabled={!!publishing}><Pencil class="size-4" /></button>
                  <button type="button" class="btn btn-ghost btn-sm btn-square text-error" title="Remove target" aria-label="Remove target"
                    onclick={() => remove(index)} disabled={!!publishing}><Trash class="size-4" /></button>
                  <button type="button" class="btn btn-sm" onclick={() => test(target)} disabled={!!publishing || busyTarget === target.name}>
                    {#if busyTarget === target.name}<span class="loading loading-spinner loading-xs"></span>{:else}<PlugZap class="size-4" />{/if}
                    Test connection
                  </button>
                  <button type="button" class="btn btn-primary btn-sm" onclick={() => (confirming = target)}
                    disabled={!!publishing || !target.host_key} title={target.host_key ? "" : "Test the connection first"}>
                    <Upload class="size-4" /> Publish
                  </button>
                </div>
              </div>

              {#if message?.kind === "ok"}
                <div class="alert alert-success alert-soft text-sm">{message.text}</div>
              {:else if message?.kind === "error"}
                <div role="alert" class="alert alert-error alert-soft text-sm">{message.text}</div>
              {:else if message?.kind === "untrusted"}
                <div class="alert alert-warning alert-soft flex flex-wrap items-center gap-3 text-sm">
                  <span class="flex-1">
                    First connection to this server. Its key fingerprint is
                    <code class="font-mono break-all">{message.text}</code>. If you can, compare it with what your
                    hosting provider shows. Trust this server?
                  </span>
                  <button type="button" class="btn btn-sm btn-warning" onclick={() => trust(index, message.text)}>Trust</button>
                </div>
              {/if}

              {#if publishing === target.name && progress}
                <div class="flex flex-col gap-1">
                  <div class="flex justify-between text-sm">
                    <span>
                      {STAGES[progress.stage]}{#if progress.stage === "uploading"}
                        {progress.done_files + 1} of {progress.total_files}{/if}
                    </span>
                    {#if progress.total_bytes}<span>{formatSize(progress.done_bytes)} of {formatSize(progress.total_bytes)}</span>{/if}
                  </div>
                  <progress class="progress progress-primary w-full" value={percent} max="100"></progress>
                  {#if progress.current}<span class="truncate font-mono text-xs text-base-content/60">{progress.current}</span>{/if}
                </div>
              {/if}
            </div>
          </section>
        {/if}
      {:else}
        {#if editing !== -1}
          <p class="rounded-box border border-dashed border-base-300 p-8 text-center text-base-content/60">
            No targets yet. Add the server your website should go to.
          </p>
        {/if}
      {/each}

      {#if editing === -1}
        <TargetForm siteRoot={site.root} {targets} index={-1} initial={newSftpTarget()} onsaved={changed} oncancel={() => (editing = null)} />
      {/if}

      {#if report}
        <div id="publish-report" role="status" class="alert alert-success alert-soft">
          <span>
            Published: {report.uploaded === 1 ? "1 website file" : `${report.uploaded} website files`} uploaded
            ({formatSize(report.bytes)}), {report.unchanged} unchanged{report.deleted
              ? `, ${report.deleted} deleted`
              : ""}.
            {#if site.config.base_url}
              <button type="button" class="link" onclick={() => site.config.base_url && openUrl(site.config.base_url)}>Open the website</button>
            {/if}
          </span>
        </div>
      {/if}
    </div>

</section>

{#if confirming}
  {@const target = confirming}
  <PublishDialog {site} {target} onpublish={(note) => publish(target, note)} onclose={() => (confirming = null)} />
{/if}

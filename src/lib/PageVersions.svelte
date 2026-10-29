<script lang="ts">
  import { onMount } from "svelte";
  import { ask } from "@tauri-apps/plugin-dialog";
  import History from "@lucide/svelte/icons/history";
  import { commands, formatTime, versionLabel, type DiffLine, type Page, type PageContent, type Version } from "$lib/site";

  interface Props {
    page: Page;
    siteRoot: string;
    /** The editor has unsaved changes, which restoring would replace. */
    dirty: boolean;
    /** Puts an earlier version into the editor, as unsaved changes. */
    onrestore: (content: PageContent, version: Version) => void;
  }

  let { page, siteRoot, dirty, onrestore }: Props = $props();

  let tracked = $state(true);
  let versions = $state<Version[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let selected = $state<Version | null>(null);
  let lines = $state<DiffLine[]>([]);

  async function refresh() {
    loading = true;
    try {
      tracked = (await commands.historyStatus(siteRoot)).tracked;
      versions = tracked ? await commands.listVersions(siteRoot, page.file) : [];
      error = null;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(refresh);

  async function show(version: Version) {
    selected = version;
    lines = [];
    try {
      lines = await commands.versionChanges(siteRoot, page.file, version.id);
    } catch (e) {
      error = String(e);
    }
  }

  async function restore(version: Version) {
    if (dirty) {
      const sure = await ask("This replaces the changes you haven't saved yet.", {
        title: "Restore version",
        kind: "warning",
        okLabel: "Restore",
        cancelLabel: "Cancel",
      });
      if (!sure) return;
    }
    try {
      onrestore(await commands.pageVersion(siteRoot, page.file, version.id), version);
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div id="page-versions" class="flex flex-col gap-4">
  {#if error}
    <div role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
  {/if}

  {#if loading}
    <span class="loading loading-spinner"></span>
  {:else if !tracked}
    <p class="rounded-box border border-dashed border-base-300 p-8 text-center text-base-content/60">
      This site has no version history yet. Start it in the Versions tab at the top.
    </p>
  {:else if versions.length === 0}
    <p class="rounded-box border border-dashed border-base-300 p-8 text-center text-base-content/60">
      This page isn't in any version yet. Saving it puts it into one.
    </p>
  {:else}
    <div class="grid items-start gap-4 lg:grid-cols-[minmax(16rem,24rem)_1fr]">
      <ul class="menu w-full rounded-box border border-base-300 p-1">
        {#each versions as version (version.id)}
          {@const label = versionLabel(version)}
          <li>
            <button type="button" class={["flex flex-col items-start gap-0.5", selected?.id === version.id && "menu-active"]}
              onclick={() => show(version)}>
              <span class="font-medium">
                {label.title}{#if version.open}<span class="badge badge-warning badge-soft badge-xs ms-1.5">open</span>{/if}
              </span>
              <span class="text-xs opacity-70">{label.time} · {version.author}</span>
              {#if version.tags.length}
                <span class="flex flex-wrap gap-1">
                  {#each version.tags as tag (tag)}<span class="badge badge-soft badge-xs">{tag}</span>{/each}
                </span>
              {/if}
            </button>
          </li>
        {/each}
      </ul>

      {#if selected}
        {@const version = selected}
        <section class="flex min-w-0 flex-col gap-3">
          <div class="flex flex-wrap items-center gap-3">
            <h3 class="flex-1 font-semibold">What changed in “{version.summary}”</h3>
            <button type="button" id="restore-version" class="btn btn-sm" onclick={() => restore(version)}>
              <History class="size-4" /> Restore this version
            </button>
          </div>
          <p class="text-sm text-base-content/60">
            Restoring puts this version into the editor as unsaved changes; nothing changes until you save.
          </p>
          <pre class="overflow-x-auto rounded-box bg-base-200 p-3 font-mono text-xs leading-relaxed">{#each lines as line, i (i)}{#if line.kind === "gap"}<span class="block text-base-content/40">⋯</span>{:else}<span class={["block whitespace-pre-wrap", line.kind === "added" && "bg-success/15 text-success-content", line.kind === "removed" && "bg-error/15 text-error-content line-through decoration-error/40"]}>{line.kind === "added" ? "+ " : line.kind === "removed" ? "− " : "  "}{line.text}</span>{/if}{/each}</pre>
        </section>
      {:else}
        <p class="text-sm text-base-content/60">Choose a version to see what changed in it.</p>
      {/if}
    </div>
  {/if}
</div>

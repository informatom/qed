<script lang="ts">
  import BlockEditor from "$lib/BlockEditor.svelte";
  import PageSettings from "$lib/PageSettings.svelte";
  import PageVersions from "$lib/PageVersions.svelte";
  import type { Page, PageContent, Version } from "$lib/site";

  interface Props {
    page: Page;
    siteRoot: string;
    onsaved: (page: Page) => void;
    onclose: () => void;
    /** True while either tab has unsaved changes. */
    dirty?: boolean;
  }

  let { page, siteRoot, onsaved, onclose, dirty = $bindable(false) }: Props = $props();

  let tab = $state<"content" | "versions" | "settings">("content");
  let contentDirty = $state(false);
  let settingsDirty = $state(false);

  $effect(() => {
    dirty = contentDirty || settingsDirty;
  });

  // An earlier version put into the editor. Changing `restoreCount` re-creates
  // both tabs (the {#key} below), so they start from it, unsaved.
  let restored = $state<PageContent | null>(null);
  let restoredFrom = $state<Version | null>(null);
  let restoreCount = $state(0);

  function restore(content: PageContent, version: Version) {
    restored = content;
    restoredFrom = version;
    restoreCount += 1;
    tab = "content";
  }

  // Once saved (or the changes are undone), the notice has done its job.
  $effect(() => {
    if (!dirty) restoredFrom = null;
  });
</script>

<section id="page-editor" class="flex flex-col gap-4">
  <header class="flex flex-wrap items-center gap-x-4 gap-y-2">
    <div class="flex min-w-0 flex-1 flex-wrap items-baseline gap-x-3">
      <h2 class="text-xl font-semibold">{page.front.title}</h2>
      <span class="font-mono text-sm text-base-content/60">/{page.path} · {page.file}</span>
    </div>
    <div role="tablist" class="tabs tabs-box tabs-sm">
      <button type="button" role="tab" id="tab-content" class={["tab", tab === "content" && "tab-active"]}
        onclick={() => (tab = "content")}>
        Content{#if contentDirty}<span class="ms-1 text-warning">•</span>{/if}
      </button>
      <button type="button" role="tab" id="tab-versions" class={["tab", tab === "versions" && "tab-active"]}
        onclick={() => (tab = "versions")}>
        Versions
      </button>
      <button type="button" role="tab" id="tab-settings" class={["tab", tab === "settings" && "tab-active"]}
        onclick={() => (tab = "settings")}>
        Settings{#if settingsDirty}<span class="ms-1 text-warning">•</span>{/if}
      </button>
    </div>
    <button type="button" id="close-editor" class="btn btn-ghost btn-sm btn-square" onclick={onclose} aria-label="Close">
      ✕
    </button>
  </header>

  {#if restoredFrom}
    <div id="restored-notice" role="status" class="alert alert-info alert-soft text-sm">
      Restored “{restoredFrom.summary}”. Save Content and Settings to keep it.
    </div>
  {/if}

  <!-- Content and Settings stay mounted, so switching tabs keeps unsaved edits. -->
  {#key restoreCount}
    <div class={[tab !== "content" && "hidden"]}>
      <BlockEditor {page} {siteRoot} {onsaved} start={restored?.blocks} bind:dirty={contentDirty} />
    </div>
    <div class={[tab !== "settings" && "hidden"]}>
      <PageSettings {page} {siteRoot} {onsaved} start={restored?.front} bind:dirty={settingsDirty} />
    </div>
  {/key}
  {#if tab === "versions"}
    <!-- Mounted on demand, so it lists the versions fresh each time. -->
    <PageVersions {page} {siteRoot} {dirty} onrestore={restore} />
  {/if}
</section>

<script lang="ts">
  import BlockEditor from "$lib/BlockEditor.svelte";
  import PageSettings from "$lib/PageSettings.svelte";
  import type { Page } from "$lib/site";

  interface Props {
    page: Page;
    siteRoot: string;
    onsaved: (page: Page) => void;
    onclose: () => void;
    /** True while either tab has unsaved changes. */
    dirty?: boolean;
  }

  let { page, siteRoot, onsaved, onclose, dirty = $bindable(false) }: Props = $props();

  let tab = $state<"content" | "settings">("content");
  let contentDirty = $state(false);
  let settingsDirty = $state(false);

  $effect(() => {
    dirty = contentDirty || settingsDirty;
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
      <button type="button" role="tab" id="tab-settings" class={["tab", tab === "settings" && "tab-active"]}
        onclick={() => (tab = "settings")}>
        Settings{#if settingsDirty}<span class="ms-1 text-warning">•</span>{/if}
      </button>
    </div>
    <button type="button" id="close-editor" class="btn btn-ghost btn-sm btn-square" onclick={onclose} aria-label="Close">
      ✕
    </button>
  </header>

  <!-- Both tabs stay mounted, so switching tabs keeps unsaved edits. -->
  <div class={[tab !== "content" && "hidden"]}>
    <BlockEditor {page} {siteRoot} {onsaved} bind:dirty={contentDirty} />
  </div>
  <div class={[tab !== "settings" && "hidden"]}>
    <PageSettings {page} {siteRoot} {onsaved} bind:dirty={settingsDirty} />
  </div>
</section>

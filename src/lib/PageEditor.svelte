<script lang="ts">
  import { ask, message } from "@tauri-apps/plugin-dialog";
  import FolderInput from "@lucide/svelte/icons/folder-input";
  import Trash from "@lucide/svelte/icons/trash";
  import BlockEditor from "$lib/BlockEditor.svelte";
  import PageSettings from "$lib/PageSettings.svelte";
  import PageVersions from "$lib/PageVersions.svelte";
  import { commands, pageLabel, type Page, type Podcast, type PageContent, type Version } from "$lib/site";

  interface Props {
    page: Page;
    siteRoot: string;
    onsaved: (page: Page) => void;
    onclose: () => void;
    /** Called after the page's file was deleted. */
    ondeleted: () => void;
    /** Asks for the Move dialog. */
    onmove: () => void;
    /** True while either tab has unsaved changes. */
    dirty?: boolean;
    /** The site's podcast settings; null for sites without one. */
    podcast?: Podcast | null;
  }

  let { page, siteRoot, onsaved, onclose, ondeleted, onmove, dirty = $bindable(false), podcast = null }: Props = $props();

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

  async function remove() {
    try {
      const [links, history] = await Promise.all([
        commands.pagesLinkingTo(siteRoot, page.path),
        commands.historyStatus(siteRoot),
      ]);
      const lines = [`Delete “${page.front.title}”?`];
      if (links.length) {
        const names = links.map(pageLabel).join(", ");
        lines.push(`${names} ${links.length === 1 ? "links" : "link"} to it; those links will stop working.`);
      }
      if (dirty) lines.push("Your unsaved changes are lost too.");
      // Deleting saves the page into a version first (see delete_page in
      // src-tauri), so with a history it can always come back as saved.
      lines.push(
        history.tracked
          ? "You can bring it back from the Versions tab."
          : "This can't be undone: the site has no version history.",
      );
      const sure = await ask(lines.join("\n\n"), {
        title: "Delete page",
        kind: "warning",
        okLabel: "Delete",
        cancelLabel: "Keep",
      });
      if (!sure) return;
      await commands.deletePage(siteRoot, page.file);
      ondeleted();
    } catch (e) {
      await message(String(e), { title: "Can't delete the page", kind: "error" });
    }
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
    <button type="button" id="move-page-open" class="btn btn-ghost btn-sm btn-square" onclick={onmove}
      disabled={page.path === "" || dirty}
      title={page.path === "" ? "The start page always stays at /" : dirty ? "Save your changes first" : "Move or rename"}
      aria-label="Move or rename">
      <FolderInput class="size-4" />
    </button>
    <button type="button" id="delete-page" class="btn btn-ghost btn-sm btn-square text-error" onclick={remove}
      disabled={page.path === ""} title={page.path === "" ? "The start page can't be deleted" : "Delete page"}
      aria-label="Delete page">
      <Trash class="size-4" />
    </button>
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
      <BlockEditor saved={page.blocks} {siteRoot} start={restored?.blocks} bind:dirty={contentDirty}
        persist={async (blocks) => {
          const saved = await commands.savePageBlocks(siteRoot, page.file, blocks);
          onsaved(saved);
          return saved.blocks;
        }} />
    </div>
    <div class={[tab !== "settings" && "hidden"]}>
      <PageSettings {page} {siteRoot} {onsaved} {podcast} start={restored?.front} bind:dirty={settingsDirty} />
    </div>
  {/key}
  {#if tab === "versions"}
    <!-- Mounted on demand, so it lists the versions fresh each time. -->
    <PageVersions {page} {siteRoot} {dirty} onrestore={restore} />
  {/if}
</section>

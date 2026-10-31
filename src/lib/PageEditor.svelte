<script lang="ts">
  import { ask, message } from "@tauri-apps/plugin-dialog";
  import FolderInput from "@tabler/icons-svelte-runes/icons/folder-down";
  import Trash from "@tabler/icons-svelte-runes/icons/trash";
  import History from "@tabler/icons-svelte-runes/icons/history";
  import LayoutList from "@tabler/icons-svelte-runes/icons/layout-list";
  import Settings from "@tabler/icons-svelte-runes/icons/settings";
  import Languages from "@tabler/icons-svelte-runes/icons/language";
  import Plus from "@tabler/icons-svelte-runes/icons/plus";
  import BlockEditor from "$lib/BlockEditor.svelte";
  import PageSettings from "$lib/PageSettings.svelte";
  import PageVersions from "$lib/PageVersions.svelte";
  import { commands, pageLabel, saveChecked, type Contributor, type Page, type Podcast, type PageContent, type Version } from "$lib/site";

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
    contributors?: Contributor[];
    /** For a site in several languages (#145): the page in each of them, `null` where it has no translation yet. */
    languages?: { code: string; page: Page | null }[];
    /** Opens the page in another language. */
    onopen?: (page: Page) => void;
    /** Starts the translation into a language. */
    ontranslate?: (code: string) => void;
  }

  let {
    page,
    siteRoot,
    onsaved,
    onclose,
    ondeleted,
    onmove,
    dirty = $bindable(false),
    podcast = null,
    contributors = [],
    languages = [],
    onopen,
    ontranslate,
  }: Props = $props();

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
  /** Counts "Take the file's" in Settings, which rebuilds just that form. */
  let settingsReload = $state(0);

  function restore(content: PageContent, version: Version) {
    restored = content;
    restoredFrom = version;
    restoreCount += 1;
    settingsReload = 0;
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
        <LayoutList class="me-1.5 size-4" />Content{#if contentDirty}<span class="ms-1 text-warning">•</span>{/if}
      </button>
      <button type="button" role="tab" id="tab-versions" class={["tab", tab === "versions" && "tab-active"]}
        onclick={() => (tab = "versions")}>
        <History class="me-1.5 size-4" />Versions
      </button>
      <button type="button" role="tab" id="tab-settings" class={["tab", tab === "settings" && "tab-active"]}
        onclick={() => (tab = "settings")}>
        <Settings class="me-1.5 size-4" />Settings{#if settingsDirty}<span class="ms-1 text-warning">•</span>{/if}
      </button>
    </div>
    <button type="button" id="move-page-open" class="btn btn-ghost btn-sm btn-square" onclick={onmove}
      disabled={page.path === "" || dirty || !!page.language}
      title={page.path === "" ? "The start page always stays at /" : page.language ? "A translation moves along with its page" : dirty ? "Save your changes first" : "Move or rename"}
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

  {#if languages.length > 1}
    <!-- The page in the site's languages (#145): open one, or start a missing translation. -->
    <div id="page-languages" class="flex flex-wrap items-center gap-2 text-sm">
      <Languages class="size-4 text-base-content/60" />
      {#each languages as language (language.code)}
        {#if language.page}
          <button type="button" id="page-language-{language.code}"
            class={["btn btn-xs font-mono uppercase", language.page.file === page.file ? "btn-primary" : "btn-ghost", language.page.front.draft && "italic"]}
            title="{language.page.front.title}{language.page.front.draft ? ' (draft)' : ''}"
            disabled={language.page.file === page.file}
            onclick={() => language.page && onopen?.(language.page)}>{language.code}</button>
        {:else}
          <button type="button" id="translate-{language.code}" class="btn btn-ghost btn-xs" disabled={dirty}
            title={dirty ? "Save your changes first" : `Start the ${language.code} translation: a copy of this page, as a draft, to translate`}
            onclick={() => ontranslate?.(language.code)}><Plus class="size-3.5" /><span class="font-mono uppercase">{language.code}</span> Translate</button>
        {/if}
      {/each}
    </div>
  {/if}

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
          const base = $state.snapshot(page.blocks);
          const result = await saveChecked(
            (check) => commands.savePageBlocks(siteRoot, page.file, blocks, check ? base : null),
            "content",
          );
          if (!result) throw new Error("Not saved. Save again to choose which version to keep.");
          onsaved(result.page);
          return result.page.blocks;
        }} />
    </div>
    <!-- Its own key: "Take the file's" in Settings rebuilds the form from the
         file without touching unsaved content. -->
    {#key settingsReload}
      <div class={[tab !== "settings" && "hidden"]}>
        <PageSettings {page} {siteRoot} {podcast} {contributors} start={settingsReload ? undefined : restored?.front}
          bind:dirty={settingsDirty}
          onsaved={(saved, replaced) => {
            onsaved(saved);
            if (replaced) settingsReload++;
          }} />
      </div>
    {/key}
  {/key}
  {#if tab === "versions"}
    <!-- Mounted on demand, so it lists the versions fresh each time. -->
    <PageVersions {page} {siteRoot} {dirty} onrestore={restore} />
  {/if}
</section>

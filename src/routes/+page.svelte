<script lang="ts">
  import { onMount } from "svelte";
  import { ask, open } from "@tauri-apps/plugin-dialog";
  import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
  import MediaLibrary from "$lib/MediaLibrary.svelte";
  import NewPageDialog from "$lib/NewPageDialog.svelte";
  import PublishDialog from "$lib/PublishDialog.svelte";
  import PageEditor from "$lib/PageEditor.svelte";
  import { BLOCK_NAMES, buildSite, openSite, previewSite, type BuildReport, type Page, type Site } from "$lib/site";

  const LAST_SITE_KEY = "qed:last-site";

  let site = $state<Site | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(false);
  let building = $state(false);
  // The page whose settings are open, by file (files don't change on save).
  let selectedFile = $state<string | null>(null);
  let selected = $derived(site?.pages.find((p) => p.file === selectedFile) ?? null);
  // True while the open page has unsaved changes.
  let editorDirty = $state(false);

  /** Opens another page (or none), after asking if edits would be lost. */
  async function select(file: string | null) {
    if (file === selectedFile) return;
    if (editorDirty) {
      const discard = await ask("The open page has unsaved changes. Discard them?", {
        title: "Unsaved changes",
        kind: "warning",
        okLabel: "Discard",
        cancelLabel: "Keep editing",
      });
      if (!discard) return;
    }
    editorDirty = false;
    selectedFile = file;
  }

  let creatingPage = $state(false);
  let publishingOpen = $state(false);
  let view = $state<"pages" | "media">("pages");

  async function pageCreated(page: Page) {
    if (!site) return;
    // Reload, so the new page lands in its place in the tree, then open it.
    await load(site.root);
    selectedFile = page.file;
  }

  function replacePage(saved: Page) {
    if (!site) return;
    const index = site.pages.findIndex((p) => p.file === saved.file);
    if (index >= 0) site.pages[index] = saved;
  }
  let report = $state<BuildReport | null>(null);
  let previewUrl = $state<string | null>(null);

  // Derived values update by themselves when `site` changes.
  let draftCount = $derived(site ? site.pages.filter((p) => p.front.draft).length : 0);

  async function load(path: string) {
    loading = true;
    error = null;
    try {
      site = await openSite(path);
      report = null;
      if (!site.pages.some((p) => p.file === selectedFile)) selectedFile = null;
      remember(path);
    } catch (e) {
      site = null;
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function build() {
    if (!site) return;
    building = true;
    error = null;
    try {
      report = await buildSite(site.root);
    } catch (e) {
      error = String(e);
    } finally {
      building = false;
    }
  }

  async function preview() {
    if (!site) return;
    building = true;
    error = null;
    try {
      const info = await previewSite(site.root);
      report = info.report;
      previewUrl = info.url;
      await openUrl(info.url);
    } catch (e) {
      error = String(e);
    } finally {
      building = false;
    }
  }

  async function chooseFolder() {
    const path = await open({ directory: true, title: "Open a site folder" });
    if (typeof path === "string") await load(path);
  }

  function remember(path: string) {
    try {
      localStorage.setItem(LAST_SITE_KEY, path);
    } catch {
      // Storage can be unavailable; reopening is just a convenience.
    }
  }

  function lastSite(): string | null {
    try {
      return localStorage.getItem(LAST_SITE_KEY);
    } catch {
      return null;
    }
  }

  // Reopen the last site once, when the page first appears.
  onMount(() => {
    const path = lastSite();
    if (path) load(path);
  });

  function depth(path: string): number {
    return path === "" ? 0 : path.split("/").length;
  }

</script>

<main class="flex min-h-screen flex-col gap-8 p-8">
  <header class="flex flex-wrap items-start justify-between gap-4">
    {#if site}
      <div id="site-summary" class="flex min-w-0 flex-col gap-1">
        <div class="flex flex-wrap items-baseline gap-x-3 text-sm text-base-content/60">
          <span class="text-xs font-semibold tracking-widest text-primary uppercase">qed</span>
          <span>
            <span id="site-root" class="font-mono">{site.root}</span> · {site.pages.length} pages{draftCount
              ? `, ${draftCount} drafts`
              : ""} · language {site.config.language} · theme {site.config.theme}
          </span>
        </div>
        <div class="flex flex-wrap items-baseline gap-x-4">
          <h1 class="text-3xl font-bold tracking-tight">{site.config.title}</h1>
          {#if site.config.description}
            <p class="text-base-content/70">{site.config.description}</p>
          {/if}
        </div>
      </div>
    {:else}
      <h1 class="text-3xl font-bold tracking-tight">qed</h1>
    {/if}
    <div class="flex gap-2">
      {#if site}
        <button id="new-page" class="btn" onclick={() => (creatingPage = true)} disabled={loading}>
          New page
        </button>
        <button id="build-site" class="btn" onclick={build} disabled={loading || building}>
          {#if building}<span class="loading loading-spinner loading-sm"></span>{/if}
          Build
        </button>
        <button id="preview-site" class="btn btn-secondary" onclick={preview} disabled={loading || building}>
          Preview in browser
        </button>
        <button id="publish-site" class="btn btn-accent" onclick={() => (publishingOpen = true)} disabled={loading}>
          Publish…
        </button>
        <button id="reload-site" class="btn btn-ghost" onclick={() => site && load(site.root)} disabled={loading}>
          Reload
        </button>
      {/if}
      <button id="open-site" class="btn btn-primary" onclick={chooseFolder} disabled={loading}>
        {#if loading}<span class="loading loading-spinner loading-sm"></span>{/if}
        Open site folder
      </button>
    </div>
  </header>

  {#if error}
    <div id="site-error" role="alert" class="alert alert-error alert-soft">
      <span class="font-mono text-sm whitespace-pre-wrap">{error}</span>
    </div>
  {/if}

  {#if report}
    <div id="build-report" role="status" class="alert alert-success alert-soft">
      <span>
        Built {report.pages} pages{report.drafts_skipped ? ` (${report.drafts_skipped} drafts skipped)` : ""},
        {report.redirects} redirects and {report.media_files} media files into
        <code class="font-mono text-sm">{report.out_dir}</code>.
        <button class="link" onclick={() => report && revealItemInDir(report.out_dir)}>Show in file manager</button>
        {#if previewUrl}
          Preview: <button class="link" onclick={() => previewUrl && openUrl(previewUrl)}>{previewUrl}</button>
        {/if}
      </span>
    </div>
  {/if}

  {#if site}
    <div role="tablist" class="tabs tabs-border -mb-4">
      <button type="button" role="tab" id="view-pages" class={["tab", view === "pages" && "tab-active"]}
        onclick={() => (view = "pages")}>Pages</button>
      <button type="button" role="tab" id="view-media" class={["tab", view === "media" && "tab-active"]}
        onclick={() => (view = "media")}>Media</button>
    </div>

    {#if view === "media"}
      <!-- Mounted on demand, so it lists the folder fresh each time. -->
      <MediaLibrary siteRoot={site.root} />
    {/if}

    <!-- The pages view stays mounted while hidden, so unsaved edits survive. -->
    <div
      class={[
        "grid items-start gap-6",
        view !== "pages" && "hidden",
        selected && "lg:grid-cols-[22rem_minmax(0,1fr)] 2xl:grid-cols-[28rem_minmax(0,1fr)]",
      ]}
    >
      <ul
        id="pages"
        class={[
          "flex flex-col gap-1 rounded-box bg-base-100 p-2 shadow-sm",
          // While a page is open, the list stays in view and scrolls by itself.
          selected && "lg:sticky lg:top-8 lg:max-h-[calc(100vh-4rem)] lg:overflow-y-auto",
        ]}
      >
        {#each site.pages as page (page.path)}
          <li id="page-{page.path || 'home'}">
            <button
              type="button"
              class={[
                "flex w-full items-center gap-4 rounded-field px-3 py-3 text-left transition-colors",
                page.file === selectedFile ? "bg-primary/10" : "hover:bg-base-200",
              ]}
              onclick={() => select(page.file)}
            >
              <div style:padding-left="{depth(page.path) * 1.25}rem" class="flex min-w-0 flex-1 flex-col">
                <span class="font-medium">
                  {page.front.title}
                  {#if page.front.draft}<span class="badge badge-ghost badge-sm">draft</span>{/if}
                  {#if page.front.menu !== "none"}
                    <span class="badge badge-outline badge-sm">{page.front.menu} menu</span>
                  {/if}
                </span>
                <span class="font-mono text-xs text-base-content/60">
                  /{page.path}{#if page.front.date}<span class="ms-2">· {page.front.date}</span>{/if}
                </span>
              </div>
              {#if !selected}
                <div class="flex flex-wrap justify-end gap-1">
                  {#each page.blocks as block, i (i)}
                    <span class="badge badge-soft badge-sm">{BLOCK_NAMES[block.type]}</span>
                  {/each}
                </div>
              {/if}
            </button>
          </li>
        {/each}
      </ul>

      {#if selected}
        <!-- Re-create the editor when another page is picked, so it starts fresh. -->
        {#key selected.file}
          <PageEditor
            page={selected}
            siteRoot={site.root}
            onsaved={replacePage}
            onclose={() => select(null)}
            bind:dirty={editorDirty}
          />
        {/key}
      {/if}
    </div>
  {:else if !error && !loading}
    <section id="empty-state" class="flex flex-1 flex-col items-center justify-center gap-3 text-center">
      <p class="text-lg">Open a site folder to see its pages.</p>
      <p class="text-sm text-base-content/60">
        A site folder has a <code>site.toml</code> and a <code>pages/</code> folder. Try
        <code>examples/demo-site</code> in the qed repository.
      </p>
    </section>
  {/if}
</main>

{#if site && publishingOpen}
  <PublishDialog
    {site}
    ontargets={(targets) => site && (site.targets = targets)}
    onclose={() => (publishingOpen = false)}
  />
{/if}

{#if site && creatingPage}
  <NewPageDialog {site} oncreated={pageCreated} onclose={() => (creatingPage = false)} />
{/if}

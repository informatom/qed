<script lang="ts">
  import { onMount } from "svelte";
  import { ask, open } from "@tauri-apps/plugin-dialog";
  import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
  import Eye from "@lucide/svelte/icons/eye";
  import FilePlus from "@lucide/svelte/icons/file-plus";
  import Files from "@lucide/svelte/icons/files";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import FolderPlus from "@lucide/svelte/icons/folder-plus";
  import Globe from "@lucide/svelte/icons/globe";
  import Hammer from "@lucide/svelte/icons/hammer";
  import History from "@lucide/svelte/icons/history";
  import FileDown from "@lucide/svelte/icons/file-down";
  import Image from "@lucide/svelte/icons/image";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Settings from "@lucide/svelte/icons/settings";
  import AppSettings from "$lib/AppSettings.svelte";
  import MediaLibrary from "$lib/MediaLibrary.svelte";
  import SiteSettings from "$lib/SiteSettings.svelte";
  import VersionsPanel from "$lib/VersionsPanel.svelte";
  import NewPageDialog from "$lib/NewPageDialog.svelte";
  import NewSiteDialog from "$lib/NewSiteDialog.svelte";
  import MovePageDialog from "$lib/MovePageDialog.svelte";
  import PublishPanel from "$lib/PublishPanel.svelte";
  import PageEditor from "$lib/PageEditor.svelte";
  import SiteSwitcher from "$lib/SiteSwitcher.svelte";
  import UpdateBar from "$lib/UpdateBar.svelte";
  import { checkForUpdate } from "$lib/updates.svelte";
  import X from "@lucide/svelte/icons/x";
  import { forgetSite, recentSites, rememberSite } from "$lib/recentSites";
  import {
    commands,
    BLOCK_NAMES,
    pageIcons,
    type BuildReport,
    type Page,
    type Site,
  } from "$lib/site";

  /** Site folders opened before, newest first. */
  let recent = $state(recentSites());

  let site = $state<Site | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(false);
  let building = $state(false);
  // The page whose settings are open, by file (files don't change on save).
  let selectedFile = $state<string | null>(null);
  let selected = $derived(site?.pages.find((p) => p.file === selectedFile) ?? null);
  // True while the open page has unsaved changes.
  let editorDirty = $state(false);

  /** The open page's file is gone: close it and read the site again. */
  async function pageDeleted() {
    editorDirty = false;
    selectedFile = null;
    if (site) await load(site.root);
  }

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
  let creatingSite = $state(false);
  let movingPage = $state(false);
  let view = $state<"pages" | "media" | "assets" | "site" | "versions" | "app">("pages");

  async function pageCreated(page: Page) {
    if (!site) return;
    // Reload, so the new page lands in its place in the tree, then open it.
    await load(site.root);
    selectedFile = page.file;
  }

  async function pageMoved(page: Page) {
    if (!site) return;
    // Other pages changed too (links, redirects): read the whole site again.
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

  // A build report is good news, not something to keep: it goes away by
  // itself after 10 seconds. Each new report starts its own timer; the
  // function returned here cancels the old one.
  $effect(() => {
    if (!report) return;
    const timer = setTimeout(() => (report = null), 10_000);
    return () => clearTimeout(timer);
  });

  // Derived values update by themselves when `site` changes.
  let draftCount = $derived(site ? site.pages.filter((p) => p.front.draft).length : 0);

  async function load(path: string) {
    loading = true;
    error = null;
    try {
      site = await commands.openSite(path);
      report = null;
      if (!site.pages.some((p) => p.file === selectedFile)) selectedFile = null;
      recent = rememberSite(path, site.config.title);
    } catch (e) {
      site = null;
      error = String(e);
      // Every tab but App needs a site; show the error on Pages instead.
      if (view !== "pages" && view !== "app") view = "pages";
    } finally {
      loading = false;
    }
  }

  async function build() {
    if (!site) return;
    building = true;
    error = null;
    try {
      report = await commands.buildSite(site.root);
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
      const info = await commands.previewSite(site.root);
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
    if (typeof path === "string") await switchTo(path);
  }

  /** Opens another site, after asking if edits in the open page would be lost. */
  async function switchTo(path: string) {
    if (editorDirty) {
      const discard = await ask("The open page has unsaved changes. Discard them?", {
        title: "Unsaved changes",
        kind: "warning",
        okLabel: "Discard",
        cancelLabel: "Keep editing",
      });
      if (!discard) return;
      editorDirty = false;
    }
    if (site && site.root !== path) selectedFile = null;
    await load(path);
  }

  // Reopen the last site once, when the page first appears.
  // Page icons (SVG by name), for the page list.
  let icons = $state(new Map<string, string>());
  onMount(async () => {
    icons = await pageIcons();
  });

  // Look for a newer qed once at start; not while developing, where the
  // version in Cargo.toml says little about what's running.
  onMount(() => {
    if (!import.meta.env.DEV) checkForUpdate(true);
  });

  onMount(() => {
    const path = recent[0]?.path;
    if (path) load(path);
  });

  function depth(path: string): number {
    return path === "" ? 0 : path.split("/").length;
  }

</script>

<main class="flex min-h-screen flex-col gap-8 p-8">
  <UpdateBar />
  <header class="flex flex-wrap items-start justify-between gap-4">
    {#if site}
      <div id="site-summary" class="flex min-w-0 flex-col gap-1">
        <div class="flex flex-wrap items-baseline gap-x-3 text-sm text-base-content/60">
          <span class="text-xs font-semibold tracking-widest text-primary uppercase">q.e.d.</span>
          <span>
            <span id="site-root" class="font-mono">{site.root}</span> · {site.pages.length} pages{draftCount
              ? `, ${draftCount} drafts`
              : ""} · language {site.config.language} · theme {site.config.theme}
          </span>
        </div>
        <div class="flex flex-wrap items-baseline gap-x-4">
          <SiteSwitcher
              title={site.config.title}
              root={site.root}
              {recent}
              onopen={switchTo}
              onchoose={chooseFolder}
              onnew={() => (creatingSite = true)}
              onforget={(path) => (recent = forgetSite(path))}
            />
          {#if site.config.description}
            <p class="text-base-content/70">{site.config.description}</p>
          {/if}
        </div>
      </div>
    {:else}
      <h1 class="text-3xl font-bold tracking-tight">q.e.d.</h1>
    {/if}
    <div class="flex gap-2">
      {#if site}
        <!-- An icon is enough here; the tooltip and aria-label say what it does. -->
        <div class="tooltip tooltip-left" data-tip="Preview in browser">
          <button id="preview-site" class="btn btn-circle btn-secondary" onclick={preview} disabled={loading || building}
            aria-label="Preview in browser">
            {#if building}
              <span class="loading loading-spinner loading-sm"></span>
            {:else}
              <Eye class="size-5" />
            {/if}
          </button>
        </div>
      {/if}
    </div>
  </header>

  {#if error}
    <div id="site-error" role="alert" class="alert alert-error alert-soft">
      <span class="font-mono text-sm whitespace-pre-wrap">{error}</span>
    </div>
  {/if}

  {#if report}
    <div id="build-report" role="status" class="alert alert-success alert-soft">
      <span class="flex-1">
        Built {report.pages} pages{report.drafts_skipped ? ` (${report.drafts_skipped} drafts skipped)` : ""},
        {report.redirects} redirects, {report.media_files} media files and {report.resized_images} resized images into
        <code class="font-mono text-sm">{report.out_dir}</code>.
        <button class="link" onclick={() => report && revealItemInDir(report.out_dir)}>Show in file manager</button>
        {#if previewUrl}
          Preview: <button class="link" onclick={() => previewUrl && openUrl(previewUrl)}>{previewUrl}</button>
        {/if}
      </span>
      <button type="button" class="btn btn-ghost btn-xs btn-square" aria-label="Close" onclick={() => (report = null)}>✕</button>
    </div>
  {/if}

  <div role="tablist" class="tabs tabs-border -mb-4">
    <button type="button" role="tab" id="view-pages" class={["tab", view === "pages" && "tab-active"]}
      onclick={() => (view = "pages")}><Files class="me-1.5 size-4" />Pages</button>
    {#if site}
      <button type="button" role="tab" id="view-media" class={["tab", view === "media" && "tab-active"]}
        onclick={() => (view = "media")}><Image class="me-1.5 size-4" />Media</button>
      <button type="button" role="tab" id="view-assets" class={["tab", view === "assets" && "tab-active"]}
        onclick={() => (view = "assets")}><FileDown class="me-1.5 size-4" />Assets</button>
      <button type="button" role="tab" id="view-site" class={["tab", view === "site" && "tab-active"]}
        onclick={() => (view = "site")}><Globe class="me-1.5 size-4" />Site</button>
      <button type="button" role="tab" id="view-versions" class={["tab", view === "versions" && "tab-active"]}
        onclick={() => (view = "versions")}><History class="me-1.5 size-4" />Versions</button>
    {/if}
    <button type="button" role="tab" id="view-app" class={["tab", view === "app" && "tab-active"]}
      onclick={() => (view = "app")}><Settings class="me-1.5 size-4" />App</button>
  </div>

  {#if view === "app"}
    <AppSettings />
  {/if}

  {#if site}
    <!-- Stays mounted while hidden, so unsaved settings survive a tab switch;
         re-created for another site. -->
    {#key site.root}
      <div class={["flex flex-col gap-6", view !== "site" && "hidden"]}>
        <div class="flex flex-wrap items-center gap-3 rounded-box bg-base-200 px-4 py-3">
          <span class="text-sm text-base-content/60">Folder</span>
          <code class="min-w-0 flex-1 truncate font-mono text-sm">{site.root}</code>
          <button id="reload-site" class="btn btn-ghost btn-sm" onclick={() => site && load(site.root)} disabled={loading}>
            <RefreshCw class="size-4" /> Reload
          </button>
          <button id="open-other-site" class="btn btn-sm" onclick={chooseFolder} disabled={loading}>
            <FolderOpen class="size-4" /> Open another site folder…
          </button>
          <button id="new-site-other" class="btn btn-sm" onclick={() => (creatingSite = true)} disabled={loading}>
            <FolderPlus class="size-4" /> New site…
          </button>
        </div>
        <div class="grid items-start gap-8 xl:grid-cols-2">
          <section class="flex flex-col gap-3">
            <h2 class="text-lg font-semibold">Settings</h2>
            <SiteSettings {site} onsaved={(config) => {
              if (!site) return;
              site.config = config;
              recent = rememberSite(site.root, config.title);
            }} />
          </section>
          <div class="flex flex-col gap-8">
            <section class="flex flex-col gap-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-lg font-semibold">Build</h2>
                <button id="build-site" class="btn btn-sm" onclick={build} disabled={loading || building}>
                  {#if building}<span class="loading loading-spinner loading-sm"></span>{:else}<Hammer class="size-4" />{/if}
                  Build
                </button>
              </div>
              <p class="text-sm text-base-content/60">
                Writes the website into <code>_site/</code> in the site folder, without publishing it. Preview and
                Publish do this by themselves; this is for looking at the files or uploading them yourself.
              </p>
            </section>
            <PublishPanel {site} ontargets={(targets) => site && (site.targets = targets)} />
          </div>
        </div>
      </div>
    {/key}

    {#if view === "media"}
      <!-- Mounted on demand, so it lists the folder fresh each time. -->
      <MediaLibrary siteRoot={site.root} editing={editorDirty} onmoved={() => site && load(site.root)} />
    {:else if view === "assets"}
      <MediaLibrary siteRoot={site.root} library="assets" />
    {:else if view === "versions"}
      <VersionsPanel siteRoot={site.root} editing={editorDirty} onrestored={() => site && load(site.root)} />
    {/if}

    <div class={["-mb-3 flex items-center gap-3", view !== "pages" && "hidden"]}>
      <button id="new-page" class="btn btn-sm btn-primary" onclick={() => (creatingPage = true)} disabled={loading}>
        <FilePlus class="size-4" /> New page
      </button>
      <span class="text-sm text-base-content/60">
        {site.pages.length} pages{draftCount ? `, ${draftCount} drafts` : ""}
      </span>
    </div>

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
                  {#if page.front.icon && icons.has(page.front.icon)}
                    <!-- qed's own built-in SVG, safe to insert. -->
                    <span class="me-1 inline-flex align-[-2px] text-base-content/70">{@html icons.get(page.front.icon)}</span>
                  {/if}
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
            ondeleted={pageDeleted}
            onmove={() => (movingPage = true)}
            bind:dirty={editorDirty}
          />
        {/key}
      {/if}
    </div>
  {:else if !loading && view === "pages"}
    <section id="empty-state" class="flex flex-1 flex-col items-center justify-center gap-3 text-center">
      <p class="text-lg">Open a site folder to see its pages, or make a new site.</p>
      <button id="open-site" class="btn btn-primary" onclick={chooseFolder} disabled={loading}>
        {#if loading}<span class="loading loading-spinner loading-sm"></span>{:else}<FolderOpen class="size-4" />{/if}
        Open site folder
      </button>
      <button id="new-site" class="btn" onclick={() => (creatingSite = true)} disabled={loading}>
        <FolderPlus class="size-4" /> New site…
      </button>
      <p class="text-sm text-base-content/60">
        A site folder has a <code>site.toml</code> and a <code>pages/</code> folder. Try
        <code>examples/demo-site</code> in the q.e.d. repository.
      </p>
      {#if recent.length}
        <ul id="recent-sites" class="mt-4 flex w-full max-w-2xl flex-col gap-1 text-start">
          <li class="px-3 text-sm font-semibold text-base-content/60">Recent sites</li>
          {#each recent as recentSite (recentSite.path)}
            <li class="flex items-center gap-2 rounded-box hover:bg-base-200">
              <button type="button" class="flex min-w-0 flex-1 flex-col px-3 py-2 text-start" disabled={loading}
                onclick={() => switchTo(recentSite.path)}>
                <span class="font-medium">{recentSite.title}</span>
                <span class="truncate font-mono text-xs text-base-content/60">{recentSite.path}</span>
              </button>
              <button type="button" class="btn btn-ghost btn-sm btn-square me-2" title="Remove from the list (the folder stays)"
                aria-label="Remove from the list" onclick={() => (recent = forgetSite(recentSite.path))}>
                <X class="size-4" />
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {/if}
</main>

{#if site && creatingPage}
  <NewPageDialog {site} oncreated={pageCreated} onclose={() => (creatingPage = false)} />
{/if}

{#if site && selected && movingPage}
  <MovePageDialog {site} page={selected} onmoved={pageMoved} onclose={() => (movingPage = false)} />
{/if}

{#if creatingSite}
  <NewSiteDialog
    oncreated={(path) => {
      view = "pages";
      switchTo(path);
    }}
    onclose={() => (creatingSite = false)}
  />
{/if}

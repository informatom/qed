<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import PageSettings from "$lib/PageSettings.svelte";
  import { buildSite, openSite, previewSite, type Block, type BuildReport, type Page, type Site } from "$lib/site";

  const LAST_SITE_KEY = "qed:last-site";

  let site = $state<Site | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(false);
  let building = $state(false);
  // The page whose settings are open, by file (files don't change on save).
  let selectedFile = $state<string | null>(null);
  let selected = $derived(site?.pages.find((p) => p.file === selectedFile) ?? null);

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

  const BLOCK_NAMES: Record<Block["type"], string> = {
    text: "Text",
    hero: "Hero",
    "call-to-action": "Call to action",
  };
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
        <button id="build-site" class="btn" onclick={build} disabled={loading || building}>
          {#if building}<span class="loading loading-spinner loading-sm"></span>{/if}
          Build
        </button>
        <button id="preview-site" class="btn btn-secondary" onclick={preview} disabled={loading || building}>
          Preview in browser
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
        {#if previewUrl}
          Preview: <button class="link" onclick={() => previewUrl && openUrl(previewUrl)}>{previewUrl}</button>
        {/if}
      </span>
    </div>
  {/if}

  {#if site}

    <div
      class={[
        "grid items-start gap-6",
        selected && "lg:grid-cols-[minmax(0,1fr)_26rem] xl:grid-cols-[minmax(0,1fr)_38rem] 2xl:grid-cols-[minmax(0,1fr)_46rem]",
      ]}
    >
      <ul id="pages" class="flex flex-col gap-1 rounded-box bg-base-100 p-2 shadow-sm">
        {#each site.pages as page (page.path)}
          <li id="page-{page.path || 'home'}">
            <button
              type="button"
              class={[
                "flex w-full items-center gap-4 rounded-field px-3 py-3 text-left transition-colors",
                page.file === selectedFile ? "bg-primary/10" : "hover:bg-base-200",
              ]}
              onclick={() => (selectedFile = page.file === selectedFile ? null : page.file)}
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
              <div class="flex flex-wrap justify-end gap-1">
                {#each page.blocks as block, i (i)}
                  <span class="badge badge-soft badge-sm">{BLOCK_NAMES[block.type]}</span>
                {/each}
              </div>
            </button>
          </li>
        {/each}
      </ul>

      {#if selected}
        <!-- Re-create the form when another page is picked, so it starts fresh. -->
        {#key selected.file}
          <!-- Stays in view while the list scrolls; scrolls itself if taller than the window. -->
          <div class="lg:sticky lg:top-8 lg:max-h-[calc(100vh-4rem)] lg:overflow-y-auto">
            <PageSettings
              page={selected}
              siteRoot={site.root}
              onsaved={replacePage}
              onclose={() => (selectedFile = null)}
            />
          </div>
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

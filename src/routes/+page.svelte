<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { openSite, type Block, type Site } from "$lib/site";

  const LAST_SITE_KEY = "qed:last-site";

  let site = $state<Site | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(false);

  // Derived values update by themselves when `site` changes.
  let draftCount = $derived(site ? site.pages.filter((p) => p.front.draft).length : 0);

  async function load(path: string) {
    loading = true;
    error = null;
    try {
      site = await openSite(path);
      remember(path);
    } catch (e) {
      site = null;
      error = String(e);
    } finally {
      loading = false;
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

<main class="mx-auto flex min-h-screen max-w-4xl flex-col gap-8 p-8">
  <header class="flex flex-wrap items-center justify-between gap-4">
    <div>
      <h1 class="text-3xl font-bold tracking-tight">qed</h1>
      {#if site}
        <p id="site-root" class="font-mono text-sm text-base-content/60">{site.root}</p>
      {/if}
    </div>
    <div class="flex gap-2">
      {#if site}
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

  {#if site}
    <section id="site-summary" class="flex flex-col gap-1">
      <h2 class="text-2xl font-semibold">{site.config.title}</h2>
      {#if site.config.description}
        <p class="text-base-content/70">{site.config.description}</p>
      {/if}
      <p class="text-sm text-base-content/60">
        {site.pages.length} pages{draftCount ? `, ${draftCount} drafts` : ""} · language
        {site.config.language} · theme {site.config.theme}
      </p>
    </section>

    <ul id="pages" class="list rounded-box bg-base-100 shadow-sm">
      {#each site.pages as page (page.path)}
        <li id="page-{page.path || 'home'}" class="list-row items-center">
          <div style:padding-left="{depth(page.path) * 1.25}rem" class="flex flex-col">
            <span class="font-medium">
              {page.front.title}
              {#if page.front.draft}<span class="badge badge-ghost badge-sm">draft</span>{/if}
              {#if page.front.menu !== "none"}
                <span class="badge badge-outline badge-sm">{page.front.menu} menu</span>
              {/if}
            </span>
            <span class="font-mono text-xs text-base-content/60">/{page.path}</span>
          </div>
          <div class="list-col-grow"></div>
          <div class="flex flex-wrap justify-end gap-1">
            {#each page.blocks as block, i (i)}
              <span class="badge badge-soft badge-sm">{BLOCK_NAMES[block.type]}</span>
            {/each}
          </div>
        </li>
      {/each}
    </ul>
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

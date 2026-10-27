<script lang="ts">
  import { untrack } from "svelte";
  import { ask } from "@tauri-apps/plugin-dialog";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronUp from "@lucide/svelte/icons/chevron-up";
  import Plus from "@lucide/svelte/icons/plus";
  import Trash from "@lucide/svelte/icons/trash";
  import { commands, type Site, type SiteConfig } from "$lib/site";
  import { trackUnsaved } from "$lib/unsaved.svelte";

  interface Props {
    site: Site;
    onsaved: (config: SiteConfig) => void;
  }

  let { site, onsaved }: Props = $props();

  /** One entry as the editor holds it: every field present. */
  interface Entry {
    title: string | null;
    page: string | null;
    link: string | null;
    divider: boolean;
    items: Entry[];
  }

  function fromConfig(items: SiteConfig["main_menu"]): Entry[] {
    return (items ?? []).map((item) => ({
      title: item.title,
      page: item.page,
      link: item.link,
      divider: item.divider ?? false,
      items: fromConfig(item.items),
    }));
  }

  let draft = $state<Entry[]>([]);
  let saving = $state(false);
  let error = $state<string | null>(null);

  // Follow what's saved (also after Reload), like the other site settings.
  $effect(() => {
    const saved = site.config.main_menu;
    untrack(() => (draft = fromConfig($state.snapshot(saved))));
  });

  // Plain values for comparing and saving: empty texts as null.
  function clean(entries: Entry[]): Entry[] {
    const text = (value: string | null) => (value && value.trim() !== "" ? value.trim() : null);
    return entries.map((entry) => ({
      title: text(entry.title),
      page: entry.page,
      link: text(entry.link),
      divider: entry.divider,
      items: clean(entry.items),
    }));
  }
  let changed = $derived(
    JSON.stringify(clean($state.snapshot(draft))) !== JSON.stringify(clean(fromConfig($state.snapshot(site.config.main_menu)))),
  );
  trackUnsaved(() => "the menu", () => changed);

  // Pages to choose from, by address, start page first.
  let pages = $derived(
    [...site.pages].filter((p) => !p.front.draft).sort((a, b) => a.path.localeCompare(b.path)),
  );
  function pageTitle(path: string | null): string {
    return site.pages.find((p) => p.path === path)?.front.title ?? "";
  }

  const page = (): Entry => ({ title: null, page: pages[0]?.path ?? "", link: null, divider: false, items: [] });
  const link = (): Entry => ({ title: "", page: null, link: "https://", divider: false, items: [] });
  const divider = (): Entry => ({ title: null, page: null, link: null, divider: true, items: [] });
  const group = (): Entry => ({ title: "New group", page: null, link: null, divider: false, items: [page()] });

  /** The automatic menu as a starting point: the pages with Menu "main". */
  function startByHand() {
    draft = site.pages
      .filter((p) => p.front.menu === "main" && !p.front.draft)
      .sort((a, b) => a.front.weight - b.front.weight || a.front.title.localeCompare(b.front.title))
      .map((p) => ({ title: null, page: p.path, link: null, divider: false, items: [] }));
    if (draft.length === 0) draft = [page()];
  }

  function move(list: Entry[], index: number, by: number) {
    const [entry] = list.splice(index, 1);
    list.splice(index + by, 0, entry);
  }

  async function save(entries: Entry[]) {
    saving = true;
    error = null;
    const config = { ...$state.snapshot(site.config), main_menu: clean(entries) };
    try {
      await commands.saveSiteConfig(site.root, config);
      onsaved(config);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  // Saved at once, like the choices in the Theme tab.
  async function saveNavigation(sidebar: boolean) {
    saving = true;
    error = null;
    const config = { ...$state.snapshot(site.config), navigation: sidebar ? ("sidebar" as const) : ("top" as const) };
    try {
      await commands.saveSiteConfig(site.root, config);
      onsaved(config);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function backToAutomatic() {
    const sure = await ask("Remove the hand-made menu? The menu is then made from the pages' Menu setting again.", {
      title: "Automatic menu",
      okLabel: "Remove",
      cancelLabel: "Keep",
    });
    if (sure) await save([]);
  }
</script>

{#snippet row(list: Entry[], i: number, entry: Entry, nested: boolean)}
  <div class="flex flex-wrap items-center gap-2">
    <span class="badge badge-soft badge-sm w-14 shrink-0 justify-center">
      {entry.divider ? "Line" : entry.items.length || (!nested && entry.page === null && entry.link === null) ? "Group" : entry.page !== null ? "Page" : "Link"}
    </span>
    {#if entry.divider}
      <span class="h-px flex-1 bg-base-300"></span>
    {:else if entry.page !== null}
      <select class="select select-sm w-56" bind:value={entry.page} aria-label="Page">
        {#each pages as p (p.path)}<option value={p.path}>{p.front.title} · /{p.path}</option>{/each}
      </select>
      <input class="input input-sm min-w-40 flex-1" bind:value={entry.title} placeholder={pageTitle(entry.page)} aria-label="Title in the menu" />
    {:else if entry.link !== null}
      <input class="input input-sm w-48" bind:value={entry.title} placeholder="Title" aria-label="Title" />
      <input class="input input-sm min-w-48 flex-1 font-mono" bind:value={entry.link} placeholder="https://… or /blog" aria-label="Address" />
    {:else}
      <input class="input input-sm min-w-48 flex-1 font-medium" bind:value={entry.title} placeholder="Group title" aria-label="Group title" />
    {/if}
    <div class="flex shrink-0">
      <button type="button" class="btn btn-ghost btn-xs btn-square" aria-label="Up" disabled={i === 0} onclick={() => move(list, i, -1)}><ChevronUp class="size-4" /></button>
      <button type="button" class="btn btn-ghost btn-xs btn-square" aria-label="Down" disabled={i === list.length - 1} onclick={() => move(list, i, 1)}><ChevronDown class="size-4" /></button>
      <button type="button" class="btn btn-ghost btn-xs btn-square text-error" aria-label="Remove" onclick={() => list.splice(i, 1)}><Trash class="size-4" /></button>
    </div>
  </div>
{/snippet}

<section id="menu-editor" class="flex flex-col gap-3">
  <label class="flex items-start gap-3 rounded-box border border-base-300 p-3">
    <input id="site-sidebar" type="checkbox" class="toggle mt-0.5" checked={site.config.navigation === "sidebar"}
      disabled={saving} onchange={(e) => saveNavigation(e.currentTarget.checked)} />
    <span class="text-sm">
      Show the main menu in a sidebar
      <span class="block text-base-content/60">
        For sites with more menu pages than fit into the top bar. On phones, it becomes a "Menu" box above the content.
        Saved at once.
      </span>
    </span>
  </label>
  {#if draft.length === 0}
    <p class="text-sm text-base-content/60">
      The main menu is made from the pages: each page whose Menu setting is “main”, ordered by weight. To arrange it
      yourself, with groups (dropdowns in the top bar) and links to other websites, make it by hand.
    </p>
    <div>
      <button type="button" id="menu-by-hand" class="btn btn-sm" onclick={startByHand}>Arrange the menu by hand</button>
    </div>
  {:else}
    <ul class="flex flex-col gap-2">
      {#each draft as entry, i (entry)}
        <li class="flex flex-col gap-2 rounded-box border border-base-300 p-2">
          {@render row(draft, i, entry, false)}
          {#if entry.page === null && entry.link === null && !entry.divider}
            <!-- A group: its entries below, one level only. -->
            <ul class="ms-16 flex flex-col gap-2 border-s border-base-300 ps-3">
              {#each entry.items as child, j (child)}
                <li>{@render row(entry.items, j, child, true)}</li>
              {/each}
            </ul>
            <div class="ms-16 flex flex-wrap gap-1 ps-3">
              <button type="button" class="btn btn-ghost btn-xs" onclick={() => entry.items.push(page())}><Plus class="size-3.5" /> Page</button>
              <button type="button" class="btn btn-ghost btn-xs" onclick={() => entry.items.push(link())}><Plus class="size-3.5" /> Link</button>
              <button type="button" class="btn btn-ghost btn-xs" onclick={() => entry.items.push(divider())}><Plus class="size-3.5" /> Line</button>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
    <div class="flex flex-wrap items-center gap-2">
      <button type="button" class="btn btn-sm" onclick={() => draft.push(page())}><Plus class="size-4" /> Page</button>
      <button type="button" class="btn btn-sm" onclick={() => draft.push(link())}><Plus class="size-4" /> Link</button>
      <button type="button" class="btn btn-sm" onclick={() => draft.push(group())}><Plus class="size-4" /> Group</button>
      <span class="text-xs text-base-content/60">A group is a dropdown in the top bar, a heading in the sidebar.</span>
    </div>
  {/if}

  {#if error}
    <div role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
  {/if}

  <div class="flex flex-wrap items-center justify-end gap-2">
    {#if (site.config.main_menu ?? []).length > 0}
      <button type="button" class="btn btn-ghost btn-sm me-auto" onclick={backToAutomatic} disabled={saving}>Back to automatic</button>
    {/if}
    {#if changed}
      <span class="text-sm text-warning">Unsaved changes</span>
      <button type="button" class="btn btn-ghost btn-sm" onclick={() => (draft = fromConfig($state.snapshot(site.config.main_menu)))}>Discard</button>
    {/if}
    <button type="button" id="save-menu" class="btn btn-primary btn-sm" disabled={!changed || saving || draft.length === 0}
      onclick={() => save($state.snapshot(draft))}>
      {#if saving}<span class="loading loading-spinner loading-sm"></span>{/if}
      Save menu
    </button>
  </div>
</section>

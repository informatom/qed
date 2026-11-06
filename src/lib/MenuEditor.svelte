<script lang="ts">
  import { untrack } from "svelte";
  import { ask } from "@tauri-apps/plugin-dialog";
  import ChevronDown from "@tabler/icons-svelte-runes/icons/chevron-down";
  import ChevronUp from "@tabler/icons-svelte-runes/icons/chevron-up";
  import Plus from "@tabler/icons-svelte-runes/icons/plus";
  import Languages from "@tabler/icons-svelte-runes/icons/language";
  import Trash from "@tabler/icons-svelte-runes/icons/trash";
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
    /** The title in further languages, by code (#145). */
    titles: Record<string, string>;
    page: string | null;
    link: string | null;
    divider: boolean;
    items: Entry[];
  }

  function fromConfig(items: SiteConfig["main_menu"]): Entry[] {
    return (items ?? []).map((item) => ({
      title: item.title,
      titles: { ...(item.titles ?? {}) },
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
      // Only filled-in titles, in a fixed order for comparing.
      titles: Object.fromEntries(
        Object.entries(entry.titles)
          .map(([code, title]) => [code, text(title)] as const)
          .filter((pair): pair is [string, string] => pair[1] !== null)
          .sort(([a], [b]) => a.localeCompare(b)),
      ),
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

  // Pages to choose from, by address, start page first: those in the main
  // language, since an entry leads to the page's translation by itself.
  let published = $derived(site.pages.filter((p) => !p.front.draft));
  let pages = $derived(published.filter((p) => !p.language).sort((a, b) => a.path.localeCompare(b.path)));
  /** Why an entry's page isn't on the website, if it isn't: a draft, or no such page. */
  function missingPage(path: string): string | null {
    if (published.some((p) => p.path === path)) return null;
    return site.pages.some((p) => p.path === path) ? "This page is a draft" : "There's no such page";
  }

  // The menu's titles in a further language (#145); null for the main one.
  let languages = $derived(Object.keys(site.config.languages ?? {}).sort());
  let language = $state<string | null>(null);
  $effect(() => {
    if (language && !languages.includes(language)) language = null;
  });

  /** The page's title in the language being edited: its translation's, if it has one. */
  function pageTitle(path: string | null): string {
    const main = site.pages.find((p) => p.path === path);
    if (!language || !main) return main?.front.title ?? "";
    const translated = site.pages.find((p) => p.language === language && p.path === (path ? `${language}/${path}` : language));
    return translated?.front.title ?? main.front.title;
  }

  const page = (): Entry => ({ title: null, titles: {}, page: pages[0]?.path ?? "", link: null, divider: false, items: [] });
  const link = (): Entry => ({ title: "", titles: {}, page: null, link: "https://", divider: false, items: [] });
  const divider = (): Entry => ({ title: null, titles: {}, page: null, link: null, divider: true, items: [] });
  const group = (): Entry => ({ title: "New group", titles: {}, page: null, link: null, divider: false, items: [page()] });

  /** The automatic menu as a starting point: the pages with Menu "main". */
  function startByHand() {
    draft = site.pages
      .filter((p) => p.front.menu === "main" && !p.front.draft)
      .sort((a, b) => a.front.weight - b.front.weight || a.front.title.localeCompare(b.front.title))
      .map((p) => ({ title: null, titles: {}, page: p.path, link: null, divider: false, items: [] }));
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
      onsaved(await commands.saveSiteConfig(site.root, config));
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
      onsaved(await commands.saveSiteConfig(site.root, config));
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

<!-- An entry's title: the main one, or the one in the language being edited. -->
{#snippet titleField(entry: Entry, placeholder: string, label: string, classes: string)}
  {#if language}
    {@const code = language}
    <input class={classes} bind:value={entry.titles[code]} placeholder={placeholder || entry.title || ""} aria-label="{label} ({code})" />
  {:else}
    <input class={classes} bind:value={entry.title} {placeholder} aria-label={label} />
  {/if}
{/snippet}

{#snippet row(list: Entry[], i: number, entry: Entry, nested: boolean)}
  <div class="flex flex-wrap items-center gap-2">
    <span class="badge badge-soft badge-sm w-14 shrink-0 justify-center">
      {entry.divider ? "Line" : entry.items.length || (!nested && entry.page === null && entry.link === null) ? "Group" : entry.page !== null ? "Page" : "Link"}
    </span>
    {#if entry.divider}
      <span class="h-px flex-1 bg-base-300"></span>
    {:else if entry.page !== null}
      {@const missing = missingPage(entry.page)}
      <select class="select select-sm w-56" class:select-warning={missing} bind:value={entry.page} aria-label="Page" disabled={language !== null}>
        {#if !pages.some((p) => p.path === entry.page)}<option value={entry.page}>/{entry.page}{missing ? " (not on the website)" : ""}</option>{/if}
        {#each pages as p (p.path)}<option value={p.path}>{p.front.title} · /{p.path}</option>{/each}
      </select>
      {#if missing}
        <span class="text-warning text-xs">{missing}, so the menu leaves it out</span>
      {/if}
      {@render titleField(entry, pageTitle(entry.page), "Title in the menu", "input input-sm min-w-40 flex-1")}
    {:else if entry.link !== null}
      {@render titleField(entry, language ? "" : "Title", "Title", "input input-sm w-48")}
      <input class="input input-sm min-w-48 flex-1 font-mono" bind:value={entry.link} placeholder="https://… or /blog" aria-label="Address" disabled={language !== null} />
    {:else}
      {@render titleField(entry, language ? "" : "Group title", "Group title", "input input-sm min-w-48 flex-1 font-medium")}
    {/if}
    <div class={["flex shrink-0", language && "invisible"]}>
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
    {#if languages.length > 0}
      <!-- The same menu in every language; only the titles are translated (#145). -->
      <div id="menu-languages" class="flex flex-wrap items-center gap-2 text-sm">
        <Languages class="size-4 text-base-content/60" />
        {#each [null, ...languages] as code (code ?? "")}
          <button type="button" id="menu-language-{code ?? site.config.language}"
            class={["btn btn-xs font-mono uppercase", language === code ? "btn-primary" : "btn-ghost"]}
            onclick={() => (language = code)}>{code ?? site.config.language}</button>
        {/each}
        <span class="text-xs text-base-content/60">
          {#if language}
            Titles in this language. Empty ones show the page's translated title, or the main title for links and groups.
          {:else}
            Pick a language to translate the titles; entries and their order are the same in every language.
          {/if}
        </span>
      </div>
    {/if}
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
            <div class={["ms-16 flex flex-wrap gap-1 ps-3", language && "hidden"]}>
              <button type="button" class="btn btn-ghost btn-xs" onclick={() => entry.items.push(page())}><Plus class="size-3.5" /> Page</button>
              <button type="button" class="btn btn-ghost btn-xs" onclick={() => entry.items.push(link())}><Plus class="size-3.5" /> Link</button>
              <button type="button" class="btn btn-ghost btn-xs" onclick={() => entry.items.push(divider())}><Plus class="size-3.5" /> Line</button>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
    <div class={["flex flex-wrap items-center gap-2", language && "hidden"]}>
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

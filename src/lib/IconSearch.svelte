<script lang="ts">
  // A search through q.e.d.'s icons (Tabler, about 6,000, #149): type a
  // word, click an icon. Searches names and Tabler's search words, so
  // "envelope" finds mail.
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { searchIcons } from "$lib/site";

  interface Props {
    /** Prefix for the element ids: `setting-icon` gives `setting-icon-search`. */
    id: string;
    /** The icon picked now, shown as selected. */
    selected?: string | null;
    onpick: (name: string) => void;
    /** Show the name under each icon (for copying codes into text). */
    names?: boolean;
  }

  let { id, selected = null, onpick, names = false }: Props = $props();

  let query = $state("");
  let results = $state<[string, string][]>([]);

  // Search as the user types; a later search wins over an earlier one.
  let latest = 0;
  $effect(() => {
    const words = query;
    const run = ++latest;
    const timer = setTimeout(async () => {
      const found = await searchIcons(words, 96);
      if (run === latest) results = found;
    }, 150);
    return () => clearTimeout(timer);
  });
</script>

<div class="flex flex-col gap-2">
  <input id="{id}-search" type="search" class="input input-sm w-full max-w-sm" bind:value={query}
    placeholder="Search icons: mail, heart, mastodon…" aria-label="Search icons" />
  <div id="{id}-results" class="flex max-h-56 flex-wrap gap-1 overflow-y-auto">
    {#each results as [name, svg] (name)}
      <!-- The SVGs are q.e.d.'s own built-in icons, so inserting them as HTML is safe. -->
      <button type="button" title={name} aria-label={name} onclick={() => onpick(name)}
        class={[
          "btn btn-sm",
          names ? "gap-1 font-normal" : "btn-square",
          selected === name ? "btn-primary" : "btn-ghost",
        ]}>{@html svg}{#if names}<span class="text-xs">{name}</span>{/if}</button>
    {:else}
      <span class="text-sm text-base-content/60">No icon found.</span>
    {/each}
  </div>
  <p class="text-xs text-base-content/60">
    All icons with their names: <button type="button" class="link" onclick={() => openUrl("https://tabler.io/icons")}>tabler.io/icons</button>
    (a filled one's name ends in <code>-filled</code>).
  </p>
</div>

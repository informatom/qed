<script lang="ts" module>
  import { commands, type FontListing } from "$lib/site";

  // Fontsource's list is the same for every site; fetch it once per run.
  let listing: Promise<FontListing[]> | null = null;
  function allFonts(): Promise<FontListing[]> {
    listing ??= commands.listFonts().catch((e) => {
      listing = null;
      throw e;
    });
    return listing;
  }
</script>

<script lang="ts">
  interface Props {
    /** The id prefix for the elements, e.g. "font-text". */
    id: string;
    label: string;
    /** The chosen family, or null for the system font (or the text font, for headings). */
    value: string | null;
    /** What an empty choice means, e.g. "System font". */
    emptyLabel: string;
    busy?: boolean;
    onpick: (family: string | null) => void;
  }

  let { id, label, value, emptyLabel, busy = false, onpick }: Props = $props();
  let open = $state(false);
  let search = $state("");
  let fonts = $state<FontListing[]>([]);
  let error = $state<string | null>(null);

  async function start() {
    open = true;
    search = "";
    try {
      fonts = await allFonts();
      error = null;
    } catch (e) {
      error = String(e);
    }
  }

  let matches = $derived.by(() => {
    const term = search.trim().toLowerCase();
    return fonts.filter((f) => !term || f.family.toLowerCase().includes(term)).slice(0, 40);
  });

  function pick(family: string | null) {
    open = false;
    onpick(family);
  }
</script>

<div class="flex flex-col gap-1 text-sm">
  <div class="flex flex-wrap items-center gap-2">
    <span class="w-24 font-medium">{label}</span>
    <span id="{id}-value" class="min-w-32 flex-1 truncate">{value ?? emptyLabel}</span>
    {#if busy}<span class="loading loading-spinner loading-sm"></span>{/if}
    <button type="button" id="{id}-change" class="btn btn-sm" disabled={busy} onclick={() => (open ? (open = false) : start())}>
      {open ? "Close" : "Change…"}
    </button>
    {#if value}
      <button type="button" class="btn btn-ghost btn-sm" disabled={busy} onclick={() => pick(null)}>Use {emptyLabel.toLowerCase()}</button>
    {/if}
  </div>
  {#if open}
    <div class="flex flex-col gap-2 rounded-box border border-base-300 p-2">
      <input id="{id}-search" class="input input-sm w-full" type="search" placeholder="Search 2,000 fonts, e.g. Ubuntu, Lora, Inter"
        bind:value={search} />
      {#if error}
        <div role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
      {:else if fonts.length === 0}
        <span class="loading loading-spinner loading-sm"></span>
      {:else}
        <ul class="flex max-h-56 flex-col overflow-y-auto">
          {#each matches as font (font.id)}
            <li>
              <button type="button" class="flex w-full items-center justify-between gap-2 rounded-field px-2 py-1 text-left hover:bg-base-200"
                onclick={() => pick(font.family)}>
                <span>{font.family}</span>
                <span class="text-xs text-base-content/50">{font.category}{font.type === "google" ? " · Google" : ""}</span>
              </button>
            </li>
          {:else}
            <li class="px-2 py-1 text-base-content/60">No font matches.</li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</div>

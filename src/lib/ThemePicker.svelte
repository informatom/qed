<script lang="ts">
  import type { ThemeChoice } from "$lib/site";

  interface Props {
    themes: ThemeChoice[];
    /** The chosen theme; null means "follow the system" (if allowed). */
    value: string | null;
    /** Offer "Follow system" (the app can, a website has no system). */
    allowSystem?: boolean;
  }

  let { themes, value = $bindable(), allowSystem = false }: Props = $props();

  // Light ones first, then dark ones, each A to Z as the list comes.
  let groups = $derived([
    { id: "light", label: "Light", themes: themes.filter((t) => !t.dark) },
    { id: "dark", label: "Dark", themes: themes.filter((t) => t.dark) },
  ]);
</script>

{#snippet card(theme: string)}
  <button type="button" role="radio" aria-checked={value === theme} data-theme={theme} onclick={() => (value = theme)}
    class={["flex flex-col gap-2 rounded-box border bg-base-100 p-2 text-left text-sm text-base-content transition",
      value === theme ? "border-primary ring-2 ring-primary" : "border-base-300 hover:border-base-content/40"]}>
    <span class="flex h-6 gap-1">
      <span class="flex-1 rounded-field bg-primary"></span>
      <span class="flex-1 rounded-field bg-secondary"></span>
      <span class="flex-1 rounded-field bg-accent"></span>
      <span class="flex-1 rounded-field bg-neutral"></span>
    </span>
    <span class="font-medium">{theme}</span>
  </button>
{/snippet}

<!-- Each swatch carries its own data-theme, so it's drawn in that theme. -->
<div class="flex flex-col gap-4" role="radiogroup">
  {#if allowSystem}
    <div class="grid grid-cols-[repeat(auto-fill,minmax(8.5rem,1fr))] gap-2">
      <button type="button" role="radio" aria-checked={value === null} onclick={() => (value = null)}
        class={["flex flex-col gap-2 rounded-box border p-2 text-left text-sm transition",
          value === null ? "border-primary ring-2 ring-primary" : "border-base-300 hover:border-base-content/40"]}>
        <span class="flex h-6 overflow-hidden rounded-field">
          <span data-theme="light" class="flex-1 bg-base-100"></span>
          <span data-theme="dark" class="flex-1 bg-base-100"></span>
        </span>
        <span class="font-medium">Follow system</span>
      </button>
    </div>
  {/if}
  {#each groups as group (group.id)}
    <section id="themes-{group.id}" class="flex flex-col gap-2">
      <h3 class="text-sm font-medium text-base-content/70">{group.label}</h3>
      <div class="grid grid-cols-[repeat(auto-fill,minmax(8.5rem,1fr))] gap-2">
        {#each group.themes as theme (theme.name)}{@render card(theme.name)}{/each}
      </div>
    </section>
  {/each}
</div>

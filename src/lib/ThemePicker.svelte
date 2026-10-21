<script lang="ts">
  interface Props {
    themes: string[];
    /** The chosen theme; null means "follow the system" (if allowed). */
    value: string | null;
    /** Offer "Follow system" (the app can, a website has no system). */
    allowSystem?: boolean;
  }

  let { themes, value = $bindable(), allowSystem = false }: Props = $props();
</script>

<!-- Each swatch carries its own data-theme, so it's drawn in that theme. -->
<div class="grid grid-cols-[repeat(auto-fill,minmax(8.5rem,1fr))] gap-2" role="radiogroup">
  {#if allowSystem}
    <button type="button" role="radio" aria-checked={value === null} onclick={() => (value = null)}
      class={["flex flex-col gap-2 rounded-box border p-2 text-left text-sm transition",
        value === null ? "border-primary ring-2 ring-primary" : "border-base-300 hover:border-base-content/40"]}>
      <span class="flex h-6 overflow-hidden rounded-field">
        <span data-theme="light" class="flex-1 bg-base-100"></span>
        <span data-theme="dark" class="flex-1 bg-base-100"></span>
      </span>
      <span class="font-medium">Follow system</span>
    </button>
  {/if}
  {#each themes as theme (theme)}
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
  {/each}
</div>

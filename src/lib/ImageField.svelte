<script lang="ts">
  import MediaPicker from "$lib/MediaPicker.svelte";
  import { mediaUrl } from "$lib/site";

  interface Props {
    siteRoot: string;
    /** A file name in media/, or null. */
    value: string | null;
  }

  let { siteRoot, value = $bindable() }: Props = $props();
  let picking = $state(false);

  // Full URLs and site paths are allowed in files, but can't be previewed here.
  let isMediaFile = $derived(!!value && !value.includes(":") && !value.startsWith("/"));
</script>

<div class="flex items-center gap-3">
  <div class="flex h-16 w-24 shrink-0 items-center justify-center overflow-hidden rounded-field border border-base-300 bg-base-200">
    {#if value && isMediaFile}
      <img src={mediaUrl(siteRoot, value)} alt="" class="h-full w-full object-cover" />
    {:else}
      <span class="text-xs text-base-content/40">{value ? "link" : "none"}</span>
    {/if}
  </div>
  <div class="flex min-w-0 flex-col gap-1">
    <span class="truncate font-mono text-sm" title={value ?? ""}>{value ?? "No image"}</span>
    <div class="flex gap-2">
      <button type="button" class="btn btn-sm" onclick={() => (picking = true)}>Choose…</button>
      {#if value}
        <button type="button" class="btn btn-ghost btn-sm" onclick={() => (value = null)}>Clear</button>
      {/if}
    </div>
  </div>
</div>

{#if picking}
  <MediaPicker {siteRoot} only="image" onpick={(name) => (value = name)} onclose={() => (picking = false)} />
{/if}

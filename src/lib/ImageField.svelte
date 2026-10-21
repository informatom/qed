<script lang="ts">
  import MediaPicker from "$lib/MediaPicker.svelte";
  import { mediaUrl } from "$lib/site";

  interface Props {
    siteRoot: string;
    /** A file name in media/ (images) or assets/ (files), or null. */
    value: string | null;
    /** "image" picks and previews images from media/; "file" picks a file to download from assets/. */
    accept?: "image" | "file";
  }

  let { siteRoot, value = $bindable(), accept = "image" }: Props = $props();
  let picking = $state(false);

  // Full URLs and site paths are allowed in files, but can't be previewed here.
  let isMediaFile = $derived(!!value && !value.includes(":") && !value.startsWith("/"));
</script>

<div class="flex items-center gap-3">
  <div class="flex h-16 w-24 shrink-0 items-center justify-center overflow-hidden rounded-field border border-base-300 bg-base-200">
    {#if value && isMediaFile && accept === "image"}
      <img src={mediaUrl(siteRoot, value)} alt="" class="h-full w-full object-cover" />
    {:else if value && isMediaFile}
      <span class="font-mono text-sm text-base-content/60 uppercase">{value.split(".").pop()}</span>
    {:else}
      <span class="text-xs text-base-content/40">{value ? "link" : "none"}</span>
    {/if}
  </div>
  <div class="flex min-w-0 flex-col gap-1">
    <span class="truncate font-mono text-sm" title={value ?? ""}>{value ?? (accept === "image" ? "No image" : "No file")}</span>
    <div class="flex gap-2">
      <button type="button" class="btn btn-sm" onclick={() => (picking = true)}>Choose…</button>
      {#if value}
        <button type="button" class="btn btn-ghost btn-sm" onclick={() => (value = null)}>Clear</button>
      {/if}
    </div>
  </div>
</div>

{#if picking}
  <MediaPicker {siteRoot} library={accept === "image" ? "media" : "assets"} only={accept === "image" ? "image" : undefined}
    onpick={(name) => (value = name)}
    onclose={() => (picking = false)} />
{/if}

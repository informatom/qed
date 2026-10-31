<script lang="ts">
  import Download from "@tabler/icons-svelte-runes/icons/download";
  import { formatSize } from "$lib/site";
  import { installUpdate, updates } from "$lib/updates.svelte";

  let percent = $derived(updates.total > 0 ? Math.round((updates.done / updates.total) * 100) : 0);
</script>

{#if updates.available && !updates.dismissed}
  <div id="update-bar" role="status" class="alert alert-info alert-soft flex flex-wrap items-center gap-3">
    <Download class="size-5" />
    <span class="flex-1">
      q.e.d. {updates.available.version} is available (you have {updates.available.currentVersion}).
      {#if updates.installing}
        Downloading{updates.total ? `: ${formatSize(updates.done)} of ${formatSize(updates.total)}` : "…"}
      {/if}
    </span>
    {#if updates.installing}
      <progress class="progress progress-info w-40" value={percent} max="100"></progress>
    {:else}
      <button type="button" id="update-later" class="btn btn-ghost btn-sm" onclick={() => (updates.dismissed = true)}>
        Later
      </button>
      <button type="button" id="update-install" class="btn btn-info btn-sm" onclick={installUpdate}>
        Install and restart
      </button>
    {/if}
    {#if updates.error}
      <span class="w-full text-sm text-error">{updates.error}</span>
    {/if}
  </div>
{/if}

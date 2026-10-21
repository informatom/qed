<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import ThemePicker from "$lib/ThemePicker.svelte";
  import { checkForUpdate, installUpdate, updates } from "$lib/updates.svelte";
  import { applyAppTheme, loadAppTheme, saveAppTheme } from "$lib/appTheme";
  import { commands } from "$lib/site";

  let themes = $state<string[]>([]);
  let theme = $state(loadAppTheme());

  onMount(async () => {
    themes = await commands.listThemes();
    updates.version ||= await getVersion();
  });

  // Apply and remember right away, so the choice is visible as you click.
  $effect(() => {
    applyAppTheme(theme);
    saveAppTheme(theme);
  });
</script>

<section id="app-settings" class="flex flex-col gap-2">
  <h2 class="text-lg font-semibold">Theme of the qed app</h2>
  <p class="text-sm text-base-content/60">
    Only for this computer, and applied right away. Each website has its own theme, under Site.
  </p>
  <ThemePicker {themes} bind:value={theme} allowSystem />
</section>

<section id="app-updates" class="mt-6 flex flex-col items-start gap-2">
  <h2 class="text-lg font-semibold">Updates</h2>
  <p class="text-sm">
    This is qed <span id="app-version" class="font-mono">{updates.version || "…"}</span>. qed looks for a newer version
    on GitHub when it starts and asks before installing one.
  </p>
  <div class="flex flex-wrap items-center gap-3">
    <button type="button" id="check-updates" class="btn btn-sm" onclick={() => checkForUpdate()}
      disabled={updates.checking || updates.installing}>
      {#if updates.checking}<span class="loading loading-spinner loading-xs"></span>{:else}<RefreshCw class="size-4" />{/if}
      Check for updates
    </button>
    {#if updates.available}
      <span class="text-sm">qed {updates.available.version} is available.</span>
      <button type="button" class="btn btn-info btn-sm" onclick={installUpdate} disabled={updates.installing}>
        {#if updates.installing}<span class="loading loading-spinner loading-xs"></span>{/if}
        Install and restart
      </button>
    {:else if updates.checkedAt && !updates.error}
      <span id="up-to-date" class="text-sm text-success">qed is up to date.</span>
    {/if}
  </div>
  {#if updates.error}
    <div role="alert" class="alert alert-error alert-soft text-sm">{updates.error}</div>
  {/if}
</section>

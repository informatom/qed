<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import ThemePicker from "$lib/ThemePicker.svelte";
  import { checkForUpdate, installUpdate, updates } from "$lib/updates.svelte";
  import { applyAppTheme, loadAppTheme, saveAppTheme } from "$lib/appTheme";
  import { commands, type ThemeChoice } from "$lib/site";
  import { loadManualLanguage, saveManualLanguage } from "$lib/manual";

  let themes = $state<ThemeChoice[]>([]);
  let theme = $state(loadAppTheme());
  let manualLanguages = $state<string[]>(["en"]);
  let manualLanguage = $state(loadManualLanguage() ?? "");
  const LANGUAGE_NAMES: Record<string, string> = { en: "English", de: "Deutsch" };

  onMount(async () => {
    themes = await commands.listThemes();
    manualLanguages = await commands.manualLanguages();
    updates.version ||= await getVersion();
  });

  // Apply and remember right away, so the choice is visible as you click.
  $effect(() => {
    applyAppTheme(theme);
    saveAppTheme(theme);
  });
</script>

<section id="app-settings" class="flex flex-col gap-2">
  <h2 class="text-lg font-semibold">Theme of the q.e.d. app</h2>
  <p class="text-sm text-base-content/60">
    Only for this computer, and applied right away. Each website has its own theme, in its Theme tab.
  </p>
  <ThemePicker {themes} bind:value={theme} allowSystem />
</section>

<section id="app-manual" class="mt-6 flex flex-col items-start gap-2">
  <h2 class="text-lg font-semibold">Language of the manual</h2>
  <p class="text-sm text-base-content/60">
    The manual the Manual buttons open. Automatic takes your computer's language if the manual is in it, else English.
    Pages not translated yet show in English.
  </p>
  <select id="manual-language" class="select select-sm w-56" bind:value={manualLanguage}
    onchange={() => saveManualLanguage(manualLanguage || null)}>
    <option value="">Automatic</option>
    {#each manualLanguages as code (code)}<option value={code}>{LANGUAGE_NAMES[code] ?? code}</option>{/each}
  </select>
</section>

<section id="app-updates" class="mt-6 flex flex-col items-start gap-2">
  <h2 class="text-lg font-semibold">Updates</h2>
  <p class="text-sm">
    This is q.e.d. <span id="app-version" class="font-mono">{updates.version || "…"}</span>. q.e.d. looks for a newer version
    on GitHub when it starts and asks before installing one.
  </p>
  <div class="flex flex-wrap items-center gap-3">
    <button type="button" id="check-updates" class="btn btn-sm" onclick={() => checkForUpdate()}
      disabled={updates.checking || updates.installing}>
      {#if updates.checking}<span class="loading loading-spinner loading-xs"></span>{:else}<RefreshCw class="size-4" />{/if}
      Check for updates
    </button>
    {#if updates.available}
      <span class="text-sm">q.e.d. {updates.available.version} is available.</span>
      <button type="button" class="btn btn-info btn-sm" onclick={installUpdate} disabled={updates.installing}>
        {#if updates.installing}<span class="loading loading-spinner loading-xs"></span>{/if}
        Install and restart
      </button>
    {:else if updates.checkedAt && !updates.error}
      <span id="up-to-date" class="text-sm text-success">q.e.d. is up to date.</span>
    {/if}
  </div>
  {#if updates.error}
    <div role="alert" class="alert alert-error alert-soft text-sm">{updates.error}</div>
  {/if}
</section>

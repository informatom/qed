<script lang="ts">
  import { onMount } from "svelte";
  import ThemePicker from "$lib/ThemePicker.svelte";
  import { applyAppTheme, loadAppTheme, saveAppTheme } from "$lib/appTheme";
  import { commands } from "$lib/site";

  let themes = $state<string[]>([]);
  let theme = $state(loadAppTheme());

  onMount(async () => {
    themes = await commands.listThemes();
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

<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { applyAppTheme, loadAppTheme } from "$lib/appTheme";

  let { children } = $props();

  // Show the app in the theme chosen in App settings.
  onMount(() => applyAppTheme(loadAppTheme()));
</script>

{@render children()}

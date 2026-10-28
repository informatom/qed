<script lang="ts">
  import Play from "@lucide/svelte/icons/play";
  import { commands, type Library } from "$lib/site";

  interface Props {
    siteRoot: string;
    library: Library;
    name: string;
  }

  let { siteRoot, library, name }: Props = $props();
  let error = $state<string | null>(null);

  // Opens the file in the computer's own audio player. The app's webview
  // plays long episodes badly (on Linux it can't stream from the app's
  // files at all, and from memory it loses the time and jumps around), and
  // a real player can seek, show the time and keep playing in the
  // background.
  async function listen() {
    error = null;
    try {
      await commands.openLibraryFile(siteRoot, library, name);
    } catch (e) {
      error = String(e);
    }
  }
</script>

<button type="button" class="btn btn-ghost btn-xs self-start" onclick={listen} title="Opens in your audio player">
  <Play class="size-3.5" /> Listen
</button>
{#if error}<span class="text-xs text-error">{error}</span>{/if}

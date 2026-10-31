<script lang="ts">
  import BookOpen from "@tabler/icons-svelte-runes/icons/book";
  import { openManual } from "$lib/manual";

  interface Props {
    /** The manual page that explains this place: `versions/`. */
    page: string;
    /** What the page is about, for the tooltip. */
    topic: string;
  }

  let { page, topic }: Props = $props();
  let error = $state<string | null>(null);
</script>

<button type="button" class="btn btn-ghost btn-xs" title="{topic} in the manual"
  onclick={() => openManual(page).catch((e) => (error = String(e)))}>
  <BookOpen class="size-3.5" /> Manual
</button>
{#if error}<span role="alert" class="text-xs text-error">{error}</span>{/if}

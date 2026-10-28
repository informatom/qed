<script lang="ts">
  import { commands } from "$lib/site";

  interface Props {
    siteRoot: string;
    /** An image in media/, or null. */
    name: string | null;
  }

  let { siteRoot, name }: Props = $props();
  let problems = $state<string[] | null>(null);

  // Checked again whenever another image is chosen.
  $effect(() => {
    const image = name;
    problems = null;
    if (!image || image.includes(":") || image.startsWith("/")) return;
    let current = true;
    commands.checkCover(siteRoot, image).then((found) => {
      if (current) problems = found;
    });
    return () => {
      current = false;
    };
  });
</script>

{#if problems && problems.length > 0}
  <div class="text-sm text-warning">
    Apple Podcasts may refuse this cover:
    <ul class="list-disc ps-5">
      {#each problems as problem (problem)}<li>{problem}</li>{/each}
    </ul>
  </div>
{:else if problems}
  <p class="text-sm text-success">Fits Apple's rules: square, 1400–3000 px, JPEG or PNG.</p>
{/if}

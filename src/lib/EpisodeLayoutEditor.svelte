<script lang="ts">
  import X from "@lucide/svelte/icons/x";
  import BlockEditor from "$lib/BlockEditor.svelte";
  import { EPISODE_PART_TYPES, PAGE_BLOCK_TYPES, commands, type Block } from "$lib/site";

  interface Props {
    siteRoot: string;
    /** The layout as saved: `episode.md`, or the default without it. */
    saved: Block[];
    onsaved: (blocks: Block[]) => void;
    onclose: () => void;
    dirty?: boolean;
  }

  let { siteRoot, saved, onsaved, onclose, dirty = $bindable(false) }: Props = $props();
</script>

<section id="episode-layout-editor" class="flex min-w-0 flex-col gap-4 rounded-box bg-base-100 p-4 shadow-sm">
  <div class="flex items-start gap-3">
    <div class="flex flex-1 flex-col gap-1">
      <h2 class="text-xl font-semibold">Episode layout</h2>
      <p class="text-sm text-base-content/60">
        What every episode page shows below its title, in this order: the parts of the episode (Player, People,
        Shownotes, …) and any other block, such as a Subscribe button, on every episode. Shownotes are the episode's own
        blocks. Link pages as <code>/about</code>, since the layout is on every episode. Saved in
        <code>episode.md</code>; the order Player, People, Shownotes, Chapters, Share links needs no file.
      </p>
    </div>
    <button type="button" class="btn btn-ghost btn-sm btn-square" aria-label="Close" onclick={onclose}><X class="size-4" /></button>
  </div>
  <BlockEditor {saved} {siteRoot} bind:dirty types={[...EPISODE_PART_TYPES, ...PAGE_BLOCK_TYPES]}
    empty="Episode pages show only their title. Add a part below."
    persist={async (blocks) => {
      const stored = await commands.saveEpisodeLayout(siteRoot, blocks);
      onsaved(stored);
      return stored;
    }} />
</section>

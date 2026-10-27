<script lang="ts">
  import X from "@lucide/svelte/icons/x";
  import BlockEditor from "$lib/BlockEditor.svelte";
  import { commands, type Block } from "$lib/site";

  interface Props {
    siteRoot: string;
    /** The footer's blocks as saved; empty if the site has no footer yet. */
    saved: Block[];
    onsaved: (blocks: Block[]) => void;
    onclose: () => void;
    dirty?: boolean;
  }

  let { siteRoot, saved, onsaved, onclose, dirty = $bindable(false) }: Props = $props();
</script>

<section id="footer-editor" class="flex min-w-0 flex-col gap-4 rounded-box bg-base-100 p-4 shadow-sm">
  <div class="flex items-start gap-3">
    <div class="flex flex-1 flex-col gap-1">
      <h2 class="text-xl font-semibold">Footer</h2>
      <p class="text-sm text-base-content/60">
        Blocks at the bottom of every page, above the footer menu, e.g. Columns with a title each: contact, links, the
        latest posts (a Post list shows titles only here). Link pages as <code>/impressum</code>, since the footer is on
        every page. Saved in <code>footer.md</code>; without blocks there's no footer file.
      </p>
    </div>
    <button type="button" class="btn btn-ghost btn-sm btn-square" aria-label="Close" onclick={onclose}><X class="size-4" /></button>
  </div>
  <BlockEditor {saved} {siteRoot} bind:dirty
    persist={async (blocks) => {
      const stored = await commands.saveFooter(siteRoot, blocks);
      onsaved(stored);
      return stored;
    }} />
</section>

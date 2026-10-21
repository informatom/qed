<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { commands, type Page, type Site } from "$lib/site";

  interface Props {
    site: Site;
    page: Page;
    /** Called with the page at its new place. */
    onmoved: (page: Page) => void;
    onclose: () => void;
  }

  let { site, page, onmoved, onclose }: Props = $props();

  let dialog: HTMLDialogElement;
  // The address is the parent's path plus the last part, the slug. The
  // form starts from the page's address once (`untrack`: on purpose).
  const start = untrack(() => page.path);
  const slash = start.lastIndexOf("/");
  let parent = $state(slash < 0 ? "" : start.slice(0, slash));
  let slug = $state(start.slice(slash + 1));
  let suggestion = $state("");
  let linking = $state<string[]>([]);
  let moving = $state(false);
  let error = $state<string | null>(null);

  // A page can't go below itself or below the pages under it.
  let parents = $derived(
    site.pages.filter((p) => p.path !== "" && p.path !== page.path && !p.path.startsWith(`${page.path}/`)),
  );
  let below = $derived(site.pages.filter((p) => p.path.startsWith(`${page.path}/`)).length);
  let newPath = $derived(parent ? `${parent}/${slug.trim()}` : slug.trim());
  let unchanged = $derived(newPath === page.path);

  onMount(async () => {
    dialog.showModal();
    linking = await commands.pagesLinkingTo(site.root, page.path);
  });

  // Rust decides what an address may contain; offer its spelling.
  $effect(() => {
    const current = slug;
    commands.slugify(current).then((clean) => {
      if (current === slug) suggestion = clean === current.trim() ? "" : clean;
    });
  });

  async function move(event: SubmitEvent) {
    event.preventDefault();
    moving = true;
    error = null;
    try {
      const moved = await commands.movePage(site.root, page.file, parent, slug);
      dialog.close();
      onmoved(moved);
    } catch (e) {
      error = String(e);
    } finally {
      moving = false;
    }
  }
</script>

<dialog bind:this={dialog} id="move-page-dialog" class="modal" onclose={onclose}>
  <form class="modal-box flex max-w-2xl flex-col gap-4" onsubmit={move}>
    <h2 class="text-xl font-semibold">Move or rename “{page.front.title}”</h2>

    <div class="grid gap-4 sm:grid-cols-2">
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Below</span>
        <select id="move-page-parent" class="select w-full" bind:value={parent}>
          <option value="">Top level</option>
          {#each parents as other (other.path)}
            <option value={other.path}>{other.front.title} (/{other.path})</option>
          {/each}
        </select>
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Address</span>
        <input id="move-page-slug" class="input w-full font-mono" bind:value={slug} required />
        {#if suggestion}
          <button type="button" class="link self-start text-sm" onclick={() => (slug = suggestion)}>
            Use “{suggestion}”
          </button>
        {/if}
      </label>
    </div>

    <p class="text-sm">
      <span class="font-mono text-base-content/60">/{page.path}</span> →
      <code id="move-page-url" class="font-mono">/{newPath}</code>
    </p>
    <ul class="list-disc ps-5 text-sm text-base-content/70">
      <li>The old address keeps working: it redirects to the new one.</li>
      {#if below}
        <li>{below === 1 ? "The page below it moves" : `The ${below} pages below it move`} along.</li>
      {/if}
      {#if linking.length}
        <li>
          Links in {linking.map((path) => `/${path}`).join(", ")} are updated.
        </li>
      {/if}
    </ul>

    {#if error}
      <div id="move-page-error" role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
    {/if}

    <div class="modal-action">
      <button type="button" class="btn btn-ghost" onclick={() => dialog.close()}>Cancel</button>
      <button id="move-page" class="btn btn-primary" disabled={unchanged || !slug.trim() || !!suggestion || moving}>
        {#if moving}<span class="loading loading-spinner loading-sm"></span>{/if}
        Move page
      </button>
    </div>
  </form>
  <form method="dialog" class="modal-backdrop"><button>close</button></form>
</dialog>

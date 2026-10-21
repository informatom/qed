<script lang="ts">
  import { onMount } from "svelte";
  import { commands, today, type Page, type Site } from "$lib/site";

  interface Props {
    site: Site;
    oncreated: (page: Page) => void;
    onclose: () => void;
  }

  let { site, oncreated, onclose }: Props = $props();

  let dialog: HTMLDialogElement;
  let title = $state("");
  let parent = $state("");
  let isPost = $state(false);
  let date = $state(today());
  let slug = $state("");
  let creating = $state(false);
  let error = $state<string | null>(null);

  // `showModal` opens the dialog on top of everything, with a backdrop, and
  // closes it on Escape.
  onMount(() => dialog.showModal());

  // Ask Rust for the slug whenever the title changes.
  $effect(() => {
    const current = title;
    commands.slugify(current).then((result) => {
      if (current === title) slug = result;
    });
  });

  let url = $derived.by(() => {
    const name = isPost && date ? `${date}-${slug}` : slug;
    return parent ? `/${parent}/${name}` : `/${name}`;
  });

  function togglePost() {
    // Posts usually live under the blog; pick it if nothing else was chosen.
    if (isPost && parent === "" && site.pages.some((p) => p.path === "blog")) parent = "blog";
  }

  async function create(event: SubmitEvent) {
    event.preventDefault();
    creating = true;
    error = null;
    try {
      const page = await commands.createPage(site.root, parent, title, isPost ? date : null);
      dialog.close();
      oncreated(page);
    } catch (e) {
      error = String(e);
    } finally {
      creating = false;
    }
  }
</script>

<dialog bind:this={dialog} id="new-page-dialog" class="modal" onclose={onclose}>
  <form class="modal-box flex max-w-2xl flex-col gap-4" onsubmit={create}>
    <h2 class="text-xl font-semibold">New page</h2>

    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">Title</span>
      <input id="new-page-title" class="input w-full" bind:value={title} required />
    </label>

    <div class="grid gap-4 sm:grid-cols-[1fr_auto]">
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Below</span>
        <select id="new-page-parent" class="select w-full" bind:value={parent}>
          <option value="">Top level</option>
          {#each site.pages.filter((p) => p.path !== "") as page (page.path)}
            <option value={page.path}>{page.front.title} (/{page.path})</option>
          {/each}
        </select>
      </label>

      <div class="flex items-end gap-3">
        <label class="flex h-10 items-center gap-2">
          <input id="new-page-is-post" type="checkbox" class="toggle" bind:checked={isPost} onchange={togglePost} />
          <span class="text-sm">Blog post</span>
        </label>
        {#if isPost}
          <input id="new-page-date" class="input w-40" type="date" bind:value={date} required />
        {/if}
      </div>
    </div>

    <p class="text-sm text-base-content/60">
      Address: <code id="new-page-url" class="font-mono">{slug ? url : "…"}</code>
    </p>

    {#if error}
      <div id="new-page-error" role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
    {/if}

    <div class="modal-action">
      <button type="button" class="btn btn-ghost" onclick={() => dialog.close()}>Cancel</button>
      <button id="create-page" class="btn btn-primary" disabled={!slug || creating}>
        {#if creating}<span class="loading loading-spinner loading-sm"></span>{/if}
        Create page
      </button>
    </div>
  </form>
  <!-- Clicking outside the box closes the dialog. -->
  <form method="dialog" class="modal-backdrop"><button>close</button></form>
</dialog>

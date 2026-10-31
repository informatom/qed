<script lang="ts">
  import Check from "@tabler/icons-svelte-runes/icons/check";
  import ChevronDown from "@tabler/icons-svelte-runes/icons/chevron-down";
  import FolderOpen from "@tabler/icons-svelte-runes/icons/folder-open";
  import FolderPlus from "@tabler/icons-svelte-runes/icons/folder-plus";
  import FolderX from "@tabler/icons-svelte-runes/icons/folder-x";
  import GitBranch from "@tabler/icons-svelte-runes/icons/git-branch";
  import X from "@tabler/icons-svelte-runes/icons/x";
  import type { RecentSite } from "$lib/recentSites";

  interface Props {
    /** The open site's title and folder. */
    title: string;
    root: string;
    recent: RecentSite[];
    onopen: (path: string) => void;
    onchoose: () => void;
    onnew: () => void;
    onclone: () => void;
    onforget: (path: string) => void;
    onclose: () => void;
  }

  let { title, root, recent, onopen, onchoose, onnew, onclone, onforget, onclose }: Props = $props();

  let menu: HTMLDetailsElement;

  // Every choice closes the menu first.
  function pick(action: () => void) {
    menu.open = false;
    action();
  }
</script>

<details bind:this={menu} id="site-switcher" class="dropdown">
  <summary class="btn btn-ghost -ms-3 h-auto gap-2 px-3 py-1 text-3xl font-bold tracking-tight" title="Switch site">
    <h1 id="site-title">{title}</h1>
    <ChevronDown class="size-5 text-base-content/50" />
  </summary>
  <ul class="dropdown-content menu z-20 mt-1 w-[32rem] max-w-[90vw] rounded-box border border-base-300 bg-base-100 p-2 shadow-lg">
    {#if recent.length}
      <li class="menu-title">Recent sites</li>
      {#each recent as site (site.path)}
        <li>
          <div class="flex items-center gap-2">
            <button type="button" class="flex min-w-0 flex-1 items-center gap-2 text-start"
              onclick={() => pick(() => onopen(site.path))}>
              <Check class="size-4 shrink-0 {site.path !== root ? 'invisible' : ''}" />
              <span class="flex min-w-0 flex-col">
                <span class="font-medium">{site.title}</span>
                <span class="truncate font-mono text-xs text-base-content/60">{site.path}</span>
              </span>
            </button>
            {#if site.path !== root}
              <button type="button" class="btn btn-ghost btn-xs btn-square" title="Remove from the list (the folder stays)"
                aria-label="Remove from the list" onclick={() => onforget(site.path)}>
                <X class="size-3" />
              </button>
            {/if}
          </div>
        </li>
      {/each}
      <li class="my-1 border-t border-base-300"></li>
    {/if}
    <li>
      <button type="button" id="switcher-open" onclick={() => pick(onchoose)}>
        <FolderOpen class="size-4" /> Open site folder…
      </button>
    </li>
    <li>
      <button type="button" id="switcher-new" onclick={() => pick(onnew)}>
        <FolderPlus class="size-4" /> New site…
      </button>
    </li>
    <li>
      <button type="button" id="switcher-clone" onclick={() => pick(onclone)}>
        <GitBranch class="size-4" /> Get a site from git…
      </button>
    </li>
    <li>
      <button type="button" id="switcher-close" onclick={() => pick(onclose)}>
        <FolderX class="size-4" /> Close site
      </button>
    </li>
  </ul>
</details>

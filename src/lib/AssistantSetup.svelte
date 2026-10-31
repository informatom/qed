<script lang="ts">
  import { onMount } from "svelte";
  import Check from "@lucide/svelte/icons/check";
  import Copy from "@lucide/svelte/icons/copy";
  import { commands } from "$lib/site";

  interface Props {
    /** The open site: the assistant works on this one folder. */
    siteRoot: string;
    siteTitle: string;
  }

  let { siteRoot, siteTitle }: Props = $props();

  /** q.e.d.'s own program, which the assistant starts as `qed mcp <site>`; null while looking. */
  let program = $state<string | null>(null);
  let failed = $state<string | null>(null);
  let copied = $state<string | null>(null);

  onMount(async () => {
    try {
      program = await commands.mcpProgram();
    } catch (error) {
      failed = String(error);
    }
  });

  /** A name for the server in the assistant's settings: `qed-podcasterei-at`. */
  let name = $derived(
    "qed-" +
      (siteTitle
        .toLowerCase()
        .normalize("NFKD")
        .replace(/[^a-z0-9]+/g, "-")
        .replace(/^-+|-+$/g, "") || "site"),
  );

  // Claude Desktop reads a JSON file; Claude Code takes a command.
  let desktop = $derived(
    JSON.stringify({ mcpServers: { [name]: { command: program ?? "", args: ["mcp", siteRoot] } } }, null, 2),
  );
  let code = $derived(`claude mcp add ${name} -- ${quote(program ?? "")} mcp ${quote(siteRoot)}`);

  function quote(text: string): string {
    return /^[\w./:\\-]+$/.test(text) ? text : `"${text.replace(/"/g, '\\"')}"`;
  }

  async function copy(text: string, which: string) {
    await navigator.clipboard.writeText(text);
    copied = which;
    setTimeout(() => (copied = copied === which ? null : copied), 2000);
  }
</script>

{#snippet snippet(id: string, label: string, text: string)}
  <div class="flex flex-col gap-1">
    <div class="flex items-center gap-2">
      <span class="text-sm font-medium">{label}</span>
      <button type="button" class="btn btn-ghost btn-xs" onclick={() => copy(text, id)}>
        {#if copied === id}<Check class="size-3.5" /> Copied{:else}<Copy class="size-3.5" /> Copy{/if}
      </button>
    </div>
    <pre id={id} class="overflow-x-auto rounded-box bg-base-200 p-3 font-mono text-xs">{text}</pre>
  </div>
{/snippet}

<section id="assistant-setup" class="mt-6 flex flex-col gap-3">
  <h2 class="text-lg font-semibold">AI assistants</h2>
  <p class="text-sm">
    An AI assistant such as Claude can work on <span class="font-medium">{siteTitle}</span> for you: read and write
    pages, make new ones, add pictures, build the site. It follows q.e.d.'s rules: every change shows in Versions and
    can be undone, and it can't publish. If you change the same page in q.e.d. meanwhile, q.e.d. asks which version to
    keep.
  </p>
  {#if failed}
    <div role="alert" class="alert alert-error alert-soft text-sm">Couldn't find q.e.d.'s own program: {failed}</div>
  {:else if !program}
    <span class="loading loading-spinner loading-sm"></span>
  {:else}
    <div class="grid gap-4 xl:grid-cols-2">
      {@render snippet(
        "assistant-desktop",
        "Claude Desktop: add to claude_desktop_config.json (Settings → Developer → Edit Config), then restart it",
        desktop,
      )}
      {@render snippet("assistant-code", "Claude Code: run once in a terminal", code)}
    </div>
    <p class="text-xs text-base-content/60">
      Other assistants that speak the Model Context Protocol work the same way: they start
      <code class="break-all">{program} mcp</code> with this site's folder.
    </p>
  {/if}
</section>

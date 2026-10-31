<script lang="ts">
  import KeyRound from "@tabler/icons-svelte-runes/icons/key";
  import { ask } from "@tauri-apps/plugin-dialog";
  import { commands } from "$lib/site";

  interface Props {
    /** The remote's address; q.e.d. keeps one key per address. */
    url: string;
    /** Tells the parent whether a key exists. */
    onkey?: (exists: boolean) => void;
  }

  let { url, onkey }: Props = $props();

  let publicKey = $state<string | null>(null);
  let copied = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    const current = url;
    publicKey = null;
    copied = false;
    if (!current.trim()) return;
    commands
      .remoteKey(current)
      .then((key) => {
        if (current !== url) return;
        publicKey = key;
        onkey?.(key !== null);
      })
      .catch((e) => (error = String(e)));
  });

  async function generate() {
    if (
      publicKey &&
      !(await ask("Make a new key? The old one stops working here; replace it on the server too.", {
        title: "New key",
        kind: "warning",
        okLabel: "Make a new key",
        cancelLabel: "Cancel",
      }))
    )
      return;
    busy = true;
    error = null;
    try {
      publicKey = await commands.generateRemoteKey(url);
      copied = false;
      onkey?.(true);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function copy() {
    if (!publicKey) return;
    await navigator.clipboard.writeText(publicKey);
    copied = true;
  }
</script>

<div class="flex flex-col gap-2">
  {#if publicKey}
    <p class="text-sm text-base-content/70">
      q.e.d. logs in with this key. Add it on the server once: on Codeberg, GitHub or Gitea in the repository's
      settings as a <em>deploy key</em> with write access, or on your own server in <code>~/.ssh/authorized_keys</code>.
    </p>
    <div class="flex items-start gap-2">
      <code id="remote-public-key" class="flex-1 rounded-field bg-base-200 p-2 font-mono text-xs break-all">{publicKey}</code>
      <button type="button" class="btn btn-sm" onclick={copy}>{copied ? "Copied" : "Copy"}</button>
    </div>
  {:else}
    <p class="text-sm text-base-content/70">
      q.e.d. needs its own key to log in to this repository. It stays in this computer's keychain.
    </p>
  {/if}
  <div>
    <button type="button" id="remote-generate-key" class={["btn btn-sm", !publicKey && "btn-primary"]}
      disabled={busy || !url.trim()} onclick={generate}>
      <KeyRound class="size-4" /> {publicKey ? "Make a new key" : "Make a key"}
    </button>
  </div>
  {#if error}<p class="text-sm text-error">{error}</p>{/if}
</div>

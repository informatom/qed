<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let title = $state("Über uns & Team");
  let slug = $state("");

  // Re-runs whenever `title` changes and asks Rust for the slug.
  $effect(() => {
    const current = title;
    invoke<string>("slugify", { title: current }).then((result) => {
      // Ignore answers that arrive after the title changed again.
      if (current === title) slug = result;
    });
  });
</script>

<main class="mx-auto flex min-h-screen max-w-xl flex-col justify-center gap-6 p-8">
  <h1 class="text-4xl font-bold tracking-tight">qed</h1>
  <p class="text-base-content/70">
    Static websites from a folder of pages. This screen checks that Svelte,
    Tauri and Rust talk to each other.
  </p>

  <label class="flex flex-col gap-2">
    <span class="text-sm font-medium">Page title</span>
    <input id="page-title" class="input w-full" bind:value={title} />
  </label>

  <div class="flex flex-col gap-2">
    <span class="text-sm font-medium">Slug, computed in Rust</span>
    <code id="page-slug" class="rounded-box bg-base-200 px-4 py-3 font-mono">/{slug}</code>
  </div>
</main>

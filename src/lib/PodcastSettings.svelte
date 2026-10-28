<script lang="ts">
  import { onMount } from "svelte";
  import CoverCheck from "$lib/CoverCheck.svelte";
  import ImageField from "$lib/ImageField.svelte";
  import { commands, type AudioFiles, type Podcast } from "$lib/site";

  interface Props {
    siteRoot: string;
    /** The podcast's settings; null for a site without a podcast. */
    podcast: Podcast | null;
    /** For placeholders: what the podcast falls back to. */
    siteTitle: string;
    siteDescription: string | null;
    hasBaseUrl: boolean;
  }

  let { siteRoot, podcast = $bindable(), siteTitle, siteDescription, hasBaseUrl }: Props = $props();

  // Turning the podcast off and on again in one go brings back what was
  // typed; only saving removes it from site.toml.
  let lastPodcast: Podcast | null = null;

  /** A new podcast, its fields in the order Rust sends them. */
  function fresh(): Podcast {
    return {
      title: null,
      description: null,
      author: null,
      owner_name: null,
      owner_email: null,
      cover: null,
      categories: [],
      explicit: false,
      type: "episodic",
      guid: null,
      copyright: null,
      funding: [],
      audio_files: "local",
    };
  }

  function toggle(on: boolean) {
    if (on) {
      podcast = lastPodcast ?? fresh();
    } else {
      lastPodcast = podcast;
      podcast = null;
    }
  }

  let categoryNames = $state<string[]>([]);
  onMount(async () => {
    categoryNames = await commands.podcastCategories();
  });

  // Up to three categories: Apple uses the first, other apps more.
  const CATEGORY_SLOTS = 3;
  function setCategory(index: number, name: string) {
    if (!podcast) return;
    const list = [...podcast.categories];
    list[index] = name;
    podcast.categories = list.filter((item, i) => item !== "" && list.indexOf(item) === i);
  }

  const AUDIO: { value: AudioFiles; label: string; hint: string }[] = [
    { value: "local", label: "In the site, not in versions", hint: "Files in audio/; versions stay small, but back up audio/ yourself." },
    { value: "in_history", label: "In the site and in versions", hint: "Files in audio/, kept in every version: one backup for all, but the history grows by each episode's size for good." },
    { value: "external", label: "Elsewhere", hint: "Each episode gives a web address for its audio, e.g. at a podcast host." },
  ];
</script>

<label class="flex items-start gap-3">
  <input id="site-podcast" type="checkbox" class="toggle mt-0.5" checked={podcast !== null}
    onchange={(event) => toggle(event.currentTarget.checked)} />
  <span class="text-sm">
    This site is a podcast
    <span class="block text-base-content/60">
      Episodes below /episodes, a podcast feed and cover art. Turning it off removes these settings when you save; episode pages stay.
    </span>
  </span>
</label>

{#if podcast}
  <div id="podcast-fields" class="grid gap-x-3 gap-y-4 @3xl:grid-cols-2">
    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">Name in podcast apps</span>
      <input id="podcast-title" class="input w-full" bind:value={podcast.title} placeholder={siteTitle} />
    </label>
    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">Author <span class="font-normal text-base-content/50">· shown below the name</span></span>
      <input id="podcast-author" class="input w-full" bind:value={podcast.author} placeholder="Who makes it" />
    </label>
    <label class="flex flex-col gap-1 @3xl:col-span-2">
      <span class="text-sm font-medium">Description</span>
      <textarea id="podcast-description" class="textarea w-full" rows="2" bind:value={podcast.description}
        placeholder={siteDescription ?? "What the podcast is about"}></textarea>
    </label>
    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">Owner <span class="font-normal text-base-content/50">· for podcast directories, rarely shown</span></span>
      <input id="podcast-owner-name" class="input w-full" bind:value={podcast.owner_name} placeholder="Name" />
    </label>
    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">Owner email</span>
      <input id="podcast-owner-email" class="input w-full" type="email" bind:value={podcast.owner_email} placeholder="Directories send confirmations here" />
    </label>

    <div id="podcast-categories" class="flex flex-col gap-1 @3xl:col-span-2">
      <span class="text-sm font-medium">Categories <span class="font-normal text-base-content/50">· Apple's list; the first counts most</span></span>
      <div class="grid gap-2 @xl:grid-cols-3">
        {#each Array.from({ length: CATEGORY_SLOTS }, (_, i) => i) as index (index)}
          {#if index <= podcast.categories.length}
            <select id="podcast-category-{index}" class="select w-full" value={podcast.categories[index] ?? ""}
              onchange={(event) => setCategory(index, event.currentTarget.value)}>
              <option value="">{index === 0 ? "Choose a category" : "No more"}</option>
              {#each categoryNames as name (name)}
                <option value={name}>{name.includes(" > ") ? ` ${name.split(" > ")[1]}` : name}</option>
              {/each}
            </select>
          {/if}
        {/each}
      </div>
    </div>

    <div class="flex flex-wrap items-center gap-x-6 gap-y-3 text-sm @3xl:col-span-2">
      <div class="flex items-center gap-2">
        <span class="font-medium">Type</span>
        <div class="join">
          <button type="button" id="podcast-type-episodic" class={["btn join-item btn-sm", podcast.type === "episodic" && "btn-active"]}
            onclick={() => podcast && (podcast.type = "episodic")} title="Newest first; episodes stand on their own">Episodic</button>
          <button type="button" id="podcast-type-serial" class={["btn join-item btn-sm", podcast.type === "serial" && "btn-active"]}
            onclick={() => podcast && (podcast.type = "serial")} title="Oldest first, like a story told in parts">Serial</button>
        </div>
      </div>
      <label class="flex items-center gap-2">
        <input id="podcast-explicit" type="checkbox" class="toggle toggle-sm" bind:checked={podcast.explicit} />
        Explicit content <span class="text-base-content/50">(episodes can say so themselves)</span>
      </label>
    </div>

    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">Copyright or license</span>
      <input id="podcast-copyright" class="input w-full" bind:value={podcast.copyright} placeholder="e.g. CC BY 4.0" />
    </label>
    <div class="flex flex-col gap-1">
      <span class="text-sm font-medium">Podcast guid <span class="font-normal text-base-content/50">· keeps it the same podcast for apps when it moves</span></span>
      <span id="podcast-guid" class="flex h-10 items-center font-mono text-sm text-base-content/70">
        {podcast.guid ?? (hasBaseUrl ? "Made when you save" : "Made once the site has a web address")}
      </span>
    </div>

    <div id="podcast-cover" class="flex flex-col gap-2 @3xl:col-span-2">
      <span class="text-sm font-medium">Cover art <span class="font-normal text-base-content/50">· square, 1400–3000 px, JPEG or PNG</span></span>
      <ImageField {siteRoot} bind:value={podcast.cover} />
      <CoverCheck {siteRoot} name={podcast.cover} />
    </div>

    <fieldset id="podcast-audio-files" class="flex flex-col gap-2 @3xl:col-span-2">
      <legend class="mb-1 text-sm font-medium">Audio files</legend>
      {#each AUDIO as option (option.value)}
        <label class="flex items-start gap-3">
          <input type="radio" class="radio radio-sm mt-0.5" name="podcast-audio-files" value={option.value} bind:group={podcast.audio_files} />
          <span class="text-sm">{option.label} <span class="block text-base-content/60">{option.hint}</span></span>
        </label>
      {/each}
    </fieldset>

    <div id="podcast-funding" class="flex flex-col gap-2 @3xl:col-span-2">
      <span class="text-sm font-medium">Support links <span class="font-normal text-base-content/50">· e.g. Liberapay, Patreon; apps show them as "Support"</span></span>
      {#each podcast.funding as funding, index (index)}
        <div class="flex gap-2">
          <input class="input w-full" bind:value={funding.title} placeholder="Text, e.g. Support us" aria-label="Link text" />
          <input class="input w-full" bind:value={funding.url} placeholder="https://…" aria-label="Address" />
          <button type="button" class="btn btn-ghost" aria-label="Remove support link"
            onclick={() => podcast && (podcast.funding = podcast.funding.filter((_, i) => i !== index))}>Remove</button>
        </div>
      {/each}
      <div>
        <button type="button" id="podcast-add-funding" class="btn btn-sm"
          onclick={() => podcast && (podcast.funding = [...podcast.funding, { url: "", title: "" }])}>Add a support link</button>
      </div>
    </div>
  </div>
{/if}

<script lang="ts">
  import { untrack } from "svelte";
  import { onMount } from "svelte";
  import CoverCheck from "$lib/CoverCheck.svelte";
  import ImageField from "$lib/ImageField.svelte";
  import MediaPicker from "$lib/MediaPicker.svelte";
  import PeopleEditor from "$lib/PeopleEditor.svelte";
  import {
    commands, formatLabel, pageIcons, podcastFormats, sameData, saveChecked,
    type AudioFormat, type Contributor, type Episode, type EpisodeAudio, type EpisodeType, type FrontMatter, type Menu, type Page, type Podcast,
  } from "$lib/site";

  interface Props {
    page: Page;
    siteRoot: string;
    /** After saving; `replaced` if the form took the file's version instead. */
    onsaved: (page: Page, replaced?: boolean) => void;
    /** Settings to start with instead of the saved ones (an earlier version). */
    start?: FrontMatter;
    /** Tells the parent whether there are unsaved changes. */
    dirty?: boolean;
    /** The site's podcast settings; null for sites without one. */
    podcast?: Podcast | null;
    contributors?: Contributor[];
  }

  let { page, siteRoot, onsaved, start, dirty = $bindable(false), podcast = null, contributors = [] }: Props = $props();

  // An editable copy of the settings, taken once when the form opens
  // (`untrack` says so on purpose; the parent re-creates the form for another
  // page). `$state.snapshot` makes a plain copy, so typing here doesn't change
  // the page list until we save.
  const initial = untrack(() => $state.snapshot(start ?? page.front));
  // The podcast's formats, the main one first; each gets a row for its file.
  const formats: AudioFormat[] = untrack(() => podcastFormats(podcast));
  if (initial.episode) {
    for (const format of formats) {
      // Empty rows to fill in; empty ones are left out when saving.
      if (!initial.episode.audio.some((a) => a.format === format)) {
        initial.episode.audio.push({ format, file: "", length: null });
      }
    }
  }
  let form = $state(initial);
  // Lists are edited as comma-separated text.
  let authors = $state(initial.authors.join(", "));
  let tags = $state(initial.tags.join(", "));

  let icons = $state<[string, string][]>([]);
  onMount(async () => {
    icons = [...(await pageIcons())];
  });

  let saving = $state(false);
  let error = $state<string | null>(null);
  let savedAt = $state<Date | null>(null);

  const MENUS: { value: Menu; label: string }[] = [
    { value: "none", label: "Not in a menu" },
    { value: "main", label: "Main menu" },
    { value: "footer", label: "Footer menu" },
  ];

  function list(text: string): string[] {
    return text
      .split(",")
      .map((item) => item.trim())
      .filter((item) => item !== "");
  }

  function orNull(text: string | null): string | null {
    return text && text.trim() !== "" ? text.trim() : null;
  }

  /** A whole number from `min` up, or null: number fields give "" when empty. */
  function counted(value: unknown, min: number): number | null {
    const n = Number(value);
    return value !== null && value !== "" && Number.isInteger(n) && n >= min ? n : null;
  }

  // The guid is shown, and only typed over on purpose (an old feed's).
  let changingGuid = $state(false);

  function cleanEpisode(episode: Episode | null): Episode | null {
    if (!episode) return null;
    return {
      ...episode,
      // Left empty, it stays as it was: an empty one would get a new guid.
      guid: episode.guid.trim() || (page.front.episode?.guid ?? ""),
      season: counted(episode.season, 1),
      // Episode 0 is common: many podcasts start counting there.
      number: counted(episode.number, 0),
      image: orNull(episode.image),
      audio: episode.audio
        .map((a) => ({ format: a.format, file: a.file.trim(), length: counted(a.length, 0) }))
        .filter((a) => a.file !== ""),
      duration: orNull(episode.duration),
      transcript: orNull(episode.transcript),
      chapters: episode.chapters
        .map((c) => ({ start: c.start.trim(), title: c.title.trim(), link: orNull(c.link) }))
        .filter((c) => c.start !== "" || c.title !== ""),
    };
  }

  const EPISODE_TYPES: { value: EpisodeType; label: string }[] = [
    { value: "full", label: "Episode" },
    { value: "trailer", label: "Trailer" },
    { value: "bonus", label: "Bonus" },
  ];

  // Explicit content: the podcast's setting, or the episode's own.
  let explicit = $state(initial.episode?.explicit == null ? "podcast" : initial.episode.explicit ? "yes" : "no");
  $effect(() => {
    if (form.episode) form.episode.explicit = explicit === "podcast" ? null : explicit === "yes";
  });

  // Audio from audio/: chosen in the library, its details read from the
  // file. Choosing one format's file also picks the others of the same
  // name (episode1.mp3, episode1.ogg), as tools like Auphonic make them.
  let pickingFor = $state<AudioFormat | null>(null);
  let pickingTranscript = $state(false);
  let audioError = $state<string | null>(null);
  let external = $derived(podcast?.audio_files === "external");

  function entry(format: AudioFormat): EpisodeAudio | undefined {
    return form.episode?.audio.find((a) => a.format === format);
  }

  async function useAudio(format: AudioFormat, name: string, siblings = true) {
    const target = entry(format);
    if (!target || !form.episode) return;
    audioError = null;
    try {
      const info = await commands.audioInfo(siteRoot, name);
      if (info.format !== format) {
        audioError = `${name} is ${formatLabel(info.format)} audio, not ${formatLabel(format)}.`;
        return;
      }
      target.file = name;
      target.length = info.length;
      // The length is the same in every format; the main one's counts.
      if (!form.episode.duration || format === formats[0]) form.episode.duration = info.duration;
    } catch (e) {
      audioError = String(e);
      return;
    }
    if (!siblings) return;
    const stem = name.replace(/\.[^.]+$/, "");
    const files = (await commands.listMedia(siteRoot, "audio")).map((f) => f.name);
    for (const other of formats) {
      const candidate = `${stem}.${other}`;
      if (other !== format && !entry(other)?.file && files.includes(candidate)) {
        await useAudio(other, candidate, false);
      }
    }
  }

  function clearAudio(format: AudioFormat) {
    const target = entry(format);
    if (!target) return;
    target.file = "";
    target.length = null;
  }

  // The podcast's address for audio elsewhere, ending in a slash.
  let audioBase = $derived(podcast?.audio_base_url ? podcast.audio_base_url.replace(/\/*$/, "/") : null);

  /** A start for an empty address: another format's, with this extension, or the podcast's address. */
  function suggestion(format: AudioFormat): string | null {
    const other = form.episode?.audio.find((a) => a.format !== format && a.file.includes("://"));
    if (other) return other.file.replace(/\.[a-z0-9]+(\?.*)?$/i, `.${format}`);
    return audioBase;
  }

  let missing = $derived(form.episode ? formats.filter((f) => !entry(f)?.file.trim()) : []);

  function megabytes(bytes: number | null): string {
    return bytes === null ? "" : `${(bytes / 1_000_000).toFixed(1)} MB`;
  }

  function addChapter() {
    if (!form.episode) return;
    // Starts where the last one left off, to be corrected by hand.
    const last = form.episode.chapters.at(-1)?.start ?? "";
    form.episode.chapters = [...form.episode.chapters, { start: last === "" ? "00:00:00" : last, title: "", link: null }];
  }

  /** An old address as saved (as site::old_address): a path with a leading slash, from a full web address too. */
  function oldAddress(text: string): string | null {
    let address = text.trim();
    const scheme = address.indexOf("://");
    if (scheme !== -1) {
      const rest = address.slice(scheme + 3);
      address = rest.includes("/") ? rest.slice(rest.indexOf("/")) : "";
    }
    address = address.split(/[#?]/)[0].replace(/^\/+|\/+$/g, "");
    return address === "" ? null : `/${address}`;
  }

  // What would be saved, recomputed whenever a field changes.
  let edited: FrontMatter = $derived({
    ...form,
    title: form.title.trim(),
    description: orNull(form.description),
    image: orNull(form.image),
    icon: orNull(form.icon),
    summary: orNull(form.summary),
    date: orNull(form.date),
    weight: Number(form.weight) || 0,
    authors: list(authors),
    tags: list(tags),
    redirect_from: [...new Set(form.redirect_from.map(oldAddress).filter((a) => a !== null))],
    episode: cleanEpisode(form.episode),
  });

  let changed = $derived(!sameData(edited, page.front));

  // Keep the parent informed.
  $effect(() => {
    dirty = changed;
  });

  async function save(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    error = null;
    try {
      const base = $state.snapshot(page.front);
      const result = await saveChecked(
        (check) => commands.savePageSettings(siteRoot, page.file, edited, check ? base : null),
        "settings",
      );
      if (!result) {
        error = "Not saved. Save again to choose which version to keep.";
        return;
      }
      if (!result.replaced) savedAt = new Date();
      onsaved(result.page, result.replaced);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<!-- `@container` lets the fields below arrange themselves by the panel's own
     width (`@xl:`), not the window's. -->
{#snippet heading(text: string)}
  <h3 class="col-span-full text-xs font-semibold tracking-wide text-base-content/60 uppercase">{text}</h3>
{/snippet}

<form id="page-settings" class="@container card border border-base-300 bg-base-100" onsubmit={save}>
  <!-- Groups with a heading each, divided by thin lines, as in the Site tab. -->
  <div class="card-body gap-6">
    <section id="page-general" class="grid gap-4 @xl:grid-cols-2">
      {@render heading("General")}
      <label class="flex flex-col gap-1 @xl:col-span-2">
        <span class="text-sm font-medium">Title</span>
        <input id="setting-title" class="input w-full" bind:value={form.title} required />
      </label>

      <label class="flex flex-col gap-1 @xl:col-span-2">
        <span class="text-sm font-medium">Description</span>
        <textarea id="setting-description" class="textarea w-full" rows="2" bind:value={form.description}
          placeholder="Shown by search engines"></textarea>
      </label>

      <div id="setting-image" class="flex flex-col gap-1 @xl:col-span-2">
        <span class="text-sm font-medium">Share image <span class="font-normal text-base-content/50">· shown when someone shares a link to the page; without one, the page's first image. Needs the site's web address.</span></span>
        <ImageField {siteRoot} bind:value={form.image} />
      </div>
    </section>

    <section id="page-menu" class="grid gap-4 border-t border-base-300 pt-6 @xl:grid-cols-2">
      {@render heading("Menu and display")}
      <div class="grid grid-cols-[1fr_6rem] gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">Menu</span>
          <select id="setting-menu" class="select w-full" bind:value={form.menu}>
            {#each MENUS as menu (menu.value)}
              <option value={menu.value}>{menu.label}</option>
            {/each}
          </select>
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">Order</span>
          <input id="setting-weight" class="input w-full" type="number" bind:value={form.weight} />
        </label>
      </div>

      <div class="flex flex-col gap-2 @xl:self-end @xl:pb-2">
        <label class="flex items-center gap-3">
          <input id="setting-draft" type="checkbox" class="toggle" bind:checked={form.draft} />
          <span class="text-sm">Draft (left out of the built site)</span>
        </label>
        <label class="flex items-center gap-3">
          <input id="setting-hide-title" type="checkbox" class="toggle" bind:checked={form.hide_title} />
          <span class="text-sm">Hide the title (for pages that start with their own heading)</span>
        </label>
        <label class="flex items-center gap-3">
          <input id="setting-back-next" type="checkbox" class="toggle" bind:checked={form.back_next} />
          <span class="text-sm">Back and next links on the pages below (Older and newer for posts)</span>
        </label>
      </div>

      <fieldset class="flex flex-col gap-2 @xl:col-span-2">
        <legend class="mb-1 text-sm font-medium">Icon <span class="font-normal text-base-content/50">· shown in menus</span></legend>
        <div class="flex flex-wrap gap-1" role="radiogroup">
          <button type="button" role="radio" aria-checked={!form.icon} onclick={() => (form.icon = null)}
            class={["btn btn-sm", !form.icon ? "btn-primary" : "btn-ghost"]}>No icon</button>
          {#each icons as [name, svg] (name)}
            <!-- The SVGs are qed's own built-in icons, so inserting them as HTML is safe. -->
            <button type="button" role="radio" aria-checked={form.icon === name} title={name} aria-label={name}
              onclick={() => (form.icon = name)}
              class={["btn btn-sm btn-square", form.icon === name ? "btn-primary" : "btn-ghost"]}>{@html svg}</button>
          {/each}
        </div>
      </fieldset>
    </section>

    <section id="page-post" class="grid gap-4 border-t border-base-300 pt-6 @xl:grid-cols-2">
      {@render heading("Blog post")}
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Date</span>
        <input id="setting-date" class="input w-full" type="date" bind:value={form.date} />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Authors</span>
        <input id="setting-authors" class="input w-full" bind:value={authors} placeholder="Comma-separated" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Tags</span>
        <input id="setting-tags" class="input w-full" bind:value={tags} placeholder="Comma-separated" />
      </label>
      <label class="flex flex-col gap-1 @xl:col-span-2">
        <span class="text-sm font-medium">Summary</span>
        <textarea id="setting-summary" class="textarea w-full" rows="3" bind:value={form.summary}></textarea>
      </label>
    </section>

    <section id="page-old-addresses" class="flex flex-col gap-2 border-t border-base-300 pt-6">
      {@render heading("Old addresses")}
      <p class="text-xs text-base-content/60">
        Addresses this page had before, on this site or on the one it was moved from, such as
        <code>/2016/03/28/easterhegg.html</code>. Each one leads visitors and search engines here. Moving or renaming
        the page in q.e.d. adds its old address by itself.
      </p>
      {#each form.redirect_from as _, index (index)}
        <div class="flex gap-2">
          <input id="old-address-{index}" class="input input-sm w-full max-w-xl font-mono" bind:value={form.redirect_from[index]}
            placeholder="/old/address or https://old-site.example/old/address" aria-label="Old address" />
          <button type="button" class="btn btn-ghost btn-sm" aria-label="Remove old address"
            onclick={() => (form.redirect_from = form.redirect_from.filter((_, i) => i !== index))}>Remove</button>
        </div>
      {/each}
      <div>
        <button type="button" id="add-old-address" class="btn btn-sm"
          onclick={() => (form.redirect_from = [...form.redirect_from, ""])}>Add an address</button>
      </div>
    </section>

    {#if form.episode}
      {@const episode = form.episode}
      <section id="episode-settings" class="grid gap-4 border-t border-base-300 pt-6 @xl:grid-cols-4">
        {@render heading("Episode")}
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">Season</span>
          <input id="episode-season" class="input w-full" type="number" min="1" bind:value={episode.season} placeholder="None" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">Number</span>
          <input id="episode-number" class="input w-full" type="number" min="0" bind:value={episode.number} placeholder="None" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">Type</span>
          <select id="episode-type" class="select w-full" bind:value={episode.type}>
            {#each EPISODE_TYPES as type (type.value)}<option value={type.value}>{type.label}</option>{/each}
          </select>
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">Explicit</span>
          <select id="episode-explicit" class="select w-full" bind:value={explicit}>
            <option value="podcast">As the podcast</option>
            <option value="yes">Yes</option>
            <option value="no">No</option>
          </select>
        </label>

        <div id="episode-image" class="flex flex-col gap-2 @xl:col-span-4">
          <span class="text-sm font-medium">Episode cover <span class="font-normal text-base-content/50">· the podcast's cover if empty</span></span>
          <ImageField {siteRoot} bind:value={episode.image} />
          <CoverCheck {siteRoot} name={episode.image} />
        </div>

        {#if changingGuid}
          <div class="flex flex-col gap-2 @xl:col-span-4">
            <label class="flex flex-col gap-1">
              <span class="text-sm font-medium">Guid</span>
              <input id="episode-guid-input" class="input w-full font-mono" bind:value={episode.guid} />
            </label>
            <div role="alert" class="alert alert-warning alert-soft text-sm">
              Only for an episode moved over from another system: type the guid it had there, exactly as in the old
              feed's &lt;guid&gt;, character for character. Podcast apps take an episode with another guid for a new
              one, and show it again to everyone who has heard it.
            </div>
          </div>
        {:else}
          <p class="text-xs text-base-content/50 @xl:col-span-4">
            Guid <span id="episode-guid" class="font-mono">{episode.guid}</span> · podcast apps tell episodes apart by it, so it stays the same.
            <button type="button" id="episode-change-guid" class="link" onclick={() => (changingGuid = true)}>Change it</button>
            for an episode moved over from another system.
          </p>
        {/if}
      </section>

      <section class="flex flex-col gap-3 border-t border-base-300 pt-6">
        {@render heading("Audio")}
        <div id="episode-audio" class="flex flex-col gap-2">
          {#each episode.audio.filter((a) => formats.includes(a.format)) as audio (audio.format)}
            <div class="grid items-center gap-2 @xl:grid-cols-[4rem_1fr]">
              <span class="badge badge-sm badge-ghost font-mono">{formatLabel(audio.format)}</span>
              {#if external || audio.file.includes("://")}
                <div class="grid gap-2 @xl:grid-cols-[1fr_10rem]">
                  <input id="episode-audio-{audio.format}" class="input w-full" bind:value={audio.file}
                    aria-label="{formatLabel(audio.format)} address"
                    placeholder={audioBase ? `${audioBase}episode.${audio.format}` : `https://… (the ${formatLabel(audio.format)} file's web address)`}
                    onfocus={() => {
                      // A start, so only the file name is left to type.
                      const start = suggestion(audio.format);
                      if (start && !audio.file) audio.file = start;
                    }}
                    onblur={() => {
                      // Nothing added to the podcast's address: empty again.
                      if (audio.file === audioBase) audio.file = "";
                    }} />
                  <input class="input w-full" type="number" min="0" bind:value={audio.length} placeholder="Size in bytes"
                    aria-label="{formatLabel(audio.format)} size in bytes" title="Size in bytes, as the feed needs it" />
                </div>
              {:else}
                <div class="flex flex-wrap items-center gap-3">
                  {#if audio.file}
                    <span class="font-mono text-sm">{audio.file}</span>
                    {#if audio.length !== null}<span class="text-sm text-base-content/60">{megabytes(audio.length)}</span>{/if}
                  {:else}
                    <span class="text-sm text-warning">No {formatLabel(audio.format)} file yet</span>
                  {/if}
                  <button type="button" id="episode-choose-{audio.format}" class="btn btn-sm" onclick={() => (pickingFor = audio.format)}>Choose…</button>
                  {#if audio.file}
                    <button type="button" class="btn btn-ghost btn-sm" onclick={() => clearAudio(audio.format)}>Clear</button>
                  {/if}
                </div>
              {/if}
            </div>
          {/each}
          <label class="mt-1 flex items-center gap-3">
            <span class="text-sm font-medium">Length</span>
            <input id="episode-duration" class="input w-32 font-mono" bind:value={episode.duration} placeholder="01:02:03"
              title="Read from the audio file; for audio elsewhere, typed in" />
          </label>
          <div class="mt-1 flex flex-wrap items-center gap-3">
            <span class="text-sm font-medium">Transcript</span>
            <span id="episode-transcript" class="font-mono text-sm">{episode.transcript ?? "None"}</span>
            <button type="button" id="episode-choose-transcript" class="btn btn-sm" onclick={() => (pickingTranscript = true)}>Choose…</button>
            {#if episode.transcript}
              <button type="button" class="btn btn-ghost btn-sm" onclick={() => (episode.transcript = null)}>Clear</button>
            {/if}
            <span class="text-sm text-base-content/50">WebVTT, from the Transcripts tab</span>
          </div>
          {#if missing.length && !form.draft}
            <p class="text-sm text-warning">
              {#if formats.length === 1}
                Publish waits until this episode has its audio (or is a draft).
              {:else}
                Publish waits until this episode has {missing.map(formatLabel).join(" and ")} audio too (or is a draft): every feed needs every episode.
              {/if}
            </p>
          {/if}
          {#if audioError}<p class="text-sm text-error">{audioError}</p>{/if}
        </div>
      </section>

      <section class="flex flex-col gap-3 border-t border-base-300 pt-6">
        {@render heading("People")}
        <PeopleEditor id="episode-people" bind:people={episode.people} {contributors} />
      </section>

      <section class="flex flex-col gap-3 border-t border-base-300 pt-6">
        {@render heading("Chapters")}
        <div id="episode-chapters" class="flex flex-col gap-2 @xl:col-span-4">
          <span class="text-sm text-base-content/60">Start as 01:02:03 or 02:03, in order.</span>
          {#each episode.chapters as chapter, index (index)}
            <div class="grid grid-cols-[7rem_1fr_1fr_auto] gap-2">
              <input class="input w-full font-mono" bind:value={chapter.start} placeholder="00:00:00" aria-label="Start" />
              <input class="input w-full" bind:value={chapter.title} placeholder="Title" aria-label="Title" />
              <input class="input w-full" bind:value={chapter.link} placeholder="Link (optional)" aria-label="Link" />
              <button type="button" class="btn btn-ghost" aria-label="Remove chapter"
                onclick={() => (episode.chapters = episode.chapters.filter((_, i) => i !== index))}>Remove</button>
            </div>
          {/each}
          <div><button type="button" id="episode-add-chapter" class="btn btn-sm" onclick={addChapter}>Add a chapter</button></div>
        </div>
      </section>
    {/if}

    <div class="flex flex-col gap-4 border-t border-base-300 pt-6">
    {#if error}
      <div id="settings-error" role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
    {/if}

    <div class="card-actions items-center justify-end">
      {#if changed}
        <span class="text-sm text-warning">Unsaved changes</span>
      {:else if savedAt}
        <span class="text-sm text-success">Saved</span>
      {/if}
      <button id="save-settings" class="btn btn-primary" disabled={!changed || saving}>
        {#if saving}<span class="loading loading-spinner loading-sm"></span>{/if}
        Save
      </button>
    </div>
    </div>
  </div>
</form>

{#if pickingTranscript && form.episode}
  {@const episode = form.episode}
  <MediaPicker {siteRoot} library="transcripts" onpick={(name) => (episode.transcript = name)} onclose={() => (pickingTranscript = false)} />
{/if}

{#if pickingFor}
  {@const format = pickingFor}
  <MediaPicker {siteRoot} library="audio" only="audio" onpick={(name) => useAudio(format, name)} onclose={() => (pickingFor = null)} />
{/if}

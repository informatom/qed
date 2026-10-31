<script lang="ts">
  import { untrack } from "svelte";
  import ImageField from "$lib/ImageField.svelte";
  import { trackUnsaved } from "$lib/unsaved.svelte";
  import { commands, sameData, type Site, type SiteConfig } from "$lib/site";

  interface Props {
    site: Site;
    onsaved: (config: SiteConfig) => void;
  }

  let { site, onsaved }: Props = $props();
  let form = $state(untrack(() => $state.snapshot(site.config)));
  // Profiles elsewhere, one per field; empty fields are left out when saving.
  let profiles = $state<string[]>(untrack(() => [...(site.config.profiles ?? [])]));
  let keptProfiles = $derived(profiles.map((p) => p.trim()).filter((p) => p !== ""));
  // Further languages (#145), by code.
  let newLanguage = $state("");
  let languageCodes = $derived(Object.keys(form.languages ?? {}).sort());
  /** Pages per further language, so one with translations isn't removed. */
  let translated = $derived(
    site.pages.reduce((counts, page) => (page.language ? counts.set(page.language, (counts.get(page.language) ?? 0) + 1) : counts), new Map<string, number>()),
  );
  function addLanguage() {
    const code = newLanguage.trim().toLowerCase();
    if (!code || form.languages?.[code]) return;
    form.languages = { ...(form.languages ?? {}), [code]: { title: null, description: null } };
    newLanguage = "";
  }
  function removeLanguage(code: string) {
    const { [code]: _, ...rest } = form.languages ?? {};
    form.languages = rest;
  }
  let saving = $state(false);

  let error = $state<string | null>(null);
  let savedAt = $state<Date | null>(null);

  function orNull(text: string | null): string | null {
    return text && text.trim() !== "" ? text.trim() : null;
  }

  // What would be saved, and whether it differs from the file. Theme and
  // Menu (and Podcast) have their own tabs, which save their settings by themselves; here
  // they're always taken as saved, never from this form's older copy.
  let edited: SiteConfig = $derived({
    ...form,
    theme: site.config.theme,
    custom_theme: site.config.custom_theme,
    saved_themes: site.config.saved_themes,
    header: site.config.header,
    colors: site.config.colors,
    dark: site.config.dark,
    fonts: site.config.fonts,
    main_menu: site.config.main_menu,
    navigation: site.config.navigation,
    title: form.title.trim(),
    language: form.language.trim() || "en",
    description: orNull(form.description),
    logo: orNull(form.logo),
    icon: orNull(form.icon),
    // None is left out, as the file leaves it out, so an untouched form has no changes.
    profiles: keptProfiles.length ? keptProfiles : site.config.profiles,
    base_url: orNull(form.base_url),
    podcast: site.config.podcast,
    // Empty texts are left out, as the file leaves them out; no further
    // language at all leaves out the whole field, as Rust does.
    languages: languageCodes.length
      ? Object.fromEntries(
          languageCodes.map((code) => [
            code,
            { title: orNull(form.languages?.[code]?.title ?? null), description: orNull(form.languages?.[code]?.description ?? null) },
          ]),
        )
      : (undefined as unknown as SiteConfig["languages"]),
  });
  let changed = $derived(!sameData(edited, site.config));
  trackUnsaved(() => "Site settings", () => changed);

  async function save(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    error = null;
    try {
      const saved = await commands.saveSiteConfig(site.root, edited);
      onsaved(saved);
      savedAt = new Date();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

{#snippet heading(text: string)}
  <h3 class="text-xs font-semibold tracking-wide text-base-content/60 uppercase">{text}</h3>
{/snippet}

<!-- Groups with a heading each, divided by thin lines. -->
<form id="site-settings" class="@container flex flex-col gap-6" onsubmit={save}>
  <section id="site-general" class="flex flex-col gap-3">
    {@render heading("General")}
    <div class="grid gap-x-3 gap-y-4 @3xl:grid-cols-[2fr_1fr]">
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Title</span>
        <input id="site-title" class="input w-full" bind:value={form.title} required />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Language <span class="font-normal text-base-content/50">· e.g. en, de, de-AT (Jänner); also for dates</span></span>
        <input id="site-language" class="input w-full" bind:value={form.language} />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Description</span>
        <input id="site-description" class="input w-full" bind:value={form.description} placeholder="Shown by search engines" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">Web address <span class="font-normal text-base-content/50">· for the sitemap and feeds</span></span>
        <input id="site-base-url" class="input w-full" bind:value={form.base_url} placeholder="https://example.com/" />
      </label>
    </div>
  </section>

  <section id="site-languages" class="flex flex-col gap-3 border-t border-base-300 pt-6">
    {@render heading("Languages")}
    <p class="text-sm text-base-content/60">
      More languages besides the main one above. A page's translation is made in its Settings tab and has its address
      below the language's code (/en/about). Pages without a translation simply aren't there in that language; a switch
      in the header leads between the languages. Title and description are the site's own in that language, if they differ.
    </p>
    {#each languageCodes as code (code)}
      {@const settings = form.languages?.[code]}
      {#if settings}
        <div id="site-language-{code}" class="grid items-center gap-2 @3xl:grid-cols-[4rem_1fr_2fr_auto]">
          <span class="badge badge-outline font-mono uppercase">{code}</span>
          <input class="input input-sm w-full" bind:value={settings.title} placeholder={form.title} aria-label="Title in {code}" />
          <input class="input input-sm w-full" bind:value={settings.description} placeholder={form.description ?? "Description"}
            aria-label="Description in {code}" />
          <button type="button" class="btn btn-ghost btn-sm" disabled={!!translated.get(code)}
            title={translated.get(code) ? `${translated.get(code)} translations are in this language; delete those first` : undefined}
            onclick={() => removeLanguage(code)}>Remove</button>
        </div>
      {/if}
    {/each}
    <div class="flex flex-wrap items-center gap-2">
      <input id="site-new-language" class="input input-sm w-28 font-mono" bind:value={newLanguage} placeholder="en"
        aria-label="Language code" onkeydown={(e) => e.key === "Enter" && (e.preventDefault(), addLanguage())} />
      <button type="button" id="site-add-language" class="btn btn-sm" disabled={!newLanguage.trim()} onclick={addLanguage}>Add a language</button>
      <span class="text-xs text-base-content/50">A code such as en, de, fr or pt-br.</span>
    </div>
  </section>

  <section id="site-logo" class="flex flex-col gap-2 border-t border-base-300 pt-6">
    {@render heading("Logo")}
    <p class="text-sm text-base-content/60">In the header of every page; SVG at best. Also the browser-tab icon, unless the site has a site icon or static/ has a favicon.ico.</p>
    <ImageField siteRoot={site.root} bind:value={form.logo} />
    {#if form.logo}
      <div class="mt-1 flex flex-wrap items-center gap-x-6 gap-y-2">
        <div class="flex items-center gap-2 text-sm">
          <span>Size</span>
          <div class="join">
            {#each [["small", "Small"], ["medium", "Medium"], ["large", "Large"], ["original", "Original"]] as [value, label] (value)}
              <button type="button" id="logo-size-{value}"
                class={["btn join-item btn-sm", form.logo_size === value && "btn-active"]}
                onclick={() => (form.logo_size = value as "small" | "medium" | "large" | "original")}
                title={value === "original" ? "The image's own size, up to 112 px high: sharp for bitmap logos" : undefined}>{label}</button>
            {/each}
          </div>
        </div>
        <label class="flex items-center gap-2 text-sm">
          <input id="site-title-in-header" type="checkbox" class="toggle toggle-sm" bind:checked={form.title_in_header} />
          Show the site title next to the logo <span class="text-base-content/50">(off for logos that show the name)</span>
        </label>
      </div>
    {/if}
  </section>

  <section id="site-icon" class="flex flex-col gap-2 border-t border-base-300 pt-6">
    {@render heading("Site icon")}
    <p class="text-sm text-base-content/60">
      The icon in the browser tab and on phones' home screens, used as it is: an SVG, a square PNG (180 × 180 pixels
      or more, so it's sharp on phones) or an ICO file from Media. Without one, the logo.
    </p>
    <ImageField siteRoot={site.root} bind:value={form.icon} />
  </section>

  <section id="site-links" class="flex flex-col gap-3 border-t border-base-300 pt-6">
    {@render heading("Links")}
    <label class="flex items-start gap-3">
      <input id="site-mark-external-links" type="checkbox" class="toggle mt-0.5" bind:checked={form.mark_external_links} />
      <span class="text-sm">
        Mark links to other sites
        <span class="block text-base-content/60">
          Links in text that lead elsewhere get a small arrow-out-of-a-box icon. Menus and buttons stay as they are.
        </span>
      </span>
    </label>
    <div id="site-profiles" class="flex flex-col gap-2">
      <span class="text-sm">
        Your profiles elsewhere
        <span class="block text-base-content/60">
          Such as https://mastodon.social/@name. Every page names them as yours (rel="me"), so Mastodon shows the site
          as verified on your profile once the profile links to the site.
        </span>
      </span>
      {#each profiles as _, index (index)}
        <div class="flex gap-2">
          <input id="site-profile-{index}" class="input input-sm w-full max-w-xl font-mono" bind:value={profiles[index]}
            placeholder="https://mastodon.social/@name" aria-label="Profile address" />
          <button type="button" class="btn btn-ghost btn-sm" aria-label="Remove profile"
            onclick={() => (profiles = profiles.filter((_, i) => i !== index))}>Remove</button>
        </div>
      {/each}
      <div>
        <button type="button" id="site-add-profile" class="btn btn-sm" onclick={() => (profiles = [...profiles, ""])}>Add a profile</button>
      </div>
    </div>
  </section>

  <section id="site-search" class="flex flex-col gap-3 border-t border-base-300 pt-6">
    {@render heading("Search")}
    <label class="flex items-start gap-3">
      <input id="site-search-toggle" type="checkbox" class="toggle mt-0.5" bind:checked={form.search} />
      <span class="text-sm">
        Site search
        <span class="block text-base-content/60">
          A search button in the header (and Ctrl+K) finds words in all pages, posts and episodes. The search runs in
          the visitor's browser, from files on the site itself; nothing goes to other services. Visitors without
          JavaScript don't see the button.
        </span>
      </span>
    </label>
  </section>

  <section id="site-layout" class="flex flex-col gap-3 border-t border-base-300 pt-6">
    {@render heading("Layout")}
    <div class="flex flex-wrap items-center gap-3 text-sm">
      <span class="font-medium">Page width</span>
      <div class="join">
        {#each [["normal", "Normal"], ["wide", "Wide"], ["full", "Full"]] as [value, label] (value)}
          <button type="button" id="page-width-{value}" class={["btn join-item btn-sm", form.page_width === value && "btn-active"]}
            onclick={() => (form.page_width = value as "normal" | "wide" | "full")}>{label}</button>
        {/each}
      </div>
      <span class="text-base-content/60">Normal suits text; Wide gives columns room; Full uses the whole window.</span>
    </div>
  </section>

  {#if error}
    <div role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
  {/if}

  <div class="flex items-center justify-end gap-2 border-t border-base-300 pt-6">
    {#if changed}
      <span class="text-sm text-warning">Unsaved changes</span>
      <button type="button" class="btn btn-ghost" onclick={() => {
        form = $state.snapshot(site.config);
        profiles = [...(site.config.profiles ?? [])];
        newLanguage = "";
      }}>Discard</button>
    {:else if savedAt}
      <span class="text-sm text-success">Saved</span>
    {/if}
    <button id="save-site-settings" class="btn btn-primary" disabled={!changed || saving}>
      {#if saving}<span class="loading loading-spinner loading-sm"></span>{/if}
      Save
    </button>
  </div>
</form>

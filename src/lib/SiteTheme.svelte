<script lang="ts">
  import { onMount } from "svelte";
  import Palette from "@lucide/svelte/icons/palette";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import ThemeLegend from "$lib/ThemeLegend.svelte";
  import ThemePicker from "$lib/ThemePicker.svelte";
  import { commands, type Corners, type CustomTheme, type Site, type SiteConfig, type ThemeVariable } from "$lib/site";

  interface Props {
    site: Site;
    onsaved: (config: SiteConfig) => void;
  }

  let { site, onsaved }: Props = $props();
  let themes = $state<string[]>([]);
  let error = $state<string | null>(null);

  let isCustom = $derived(site.config.theme === "custom");
  /** The custom colors being edited; saved on each finished change. */
  let draft = $state<CustomTheme | null>(null);
  let variables = $state<ThemeVariable[]>([]);

  const COLORS = [
    { key: "background", label: "Background" },
    { key: "primary", label: "Primary" },
    { key: "secondary", label: "Secondary" },
    { key: "accent", label: "Accent" },
    { key: "neutral", label: "Neutral" },
  ] as const;
  type ColorKey = (typeof COLORS)[number]["key"];

  const CORNERS: { value: Corners | null; label: string }[] = [
    { value: null, label: "From the theme" },
    { value: "sharp", label: "Sharp" },
    { value: "soft", label: "Soft" },
    { value: "round", label: "Round" },
  ];

  onMount(async () => {
    themes = await commands.listThemes();
  });

  // Start from what's saved; a new site has no custom colors yet.
  $effect(() => {
    if (draft === null && site.config.custom_theme) draft = $state.snapshot(site.config.custom_theme);
  });

  // The legend previews the draft as it changes, even mid-drag.
  $effect(() => {
    const current = isCustom && draft ? $state.snapshot(draft) : null;
    if (!current) {
      variables = [];
      return;
    }
    commands.customThemeVariables(current).then((vars) => (variables = vars));
  });

  async function save(config: SiteConfig) {
    error = null;
    try {
      await commands.saveSiteConfig(site.root, config);
      onsaved(config);
    } catch (e) {
      error = String(e);
    }
  }

  // Saved as soon as it's picked, like the app's theme: a click on a swatch
  // looks final. Only the theme changes in site.toml; custom colors stay
  // there for later.
  async function pick(theme: string) {
    if (theme === site.config.theme) return;
    await save({ ...$state.snapshot(site.config), theme });
  }

  /** A theme's colors as #rrggbb, read from the browser: themes define them in oklch. */
  function colorsOf(theme: string): Record<ColorKey, string> {
    const probe = document.createElement("div");
    probe.dataset.theme = theme;
    probe.style.display = "none";
    document.body.append(probe);
    const canvas = document.createElement("canvas");
    canvas.width = canvas.height = 1;
    const context = canvas.getContext("2d", { willReadFrequently: true });
    const hex = (variable: string) => {
      if (!context) return "#808080";
      context.fillStyle = getComputedStyle(probe).getPropertyValue(variable).trim() || "#808080";
      context.fillRect(0, 0, 1, 1);
      const [r, g, b] = context.getImageData(0, 0, 1, 1).data;
      return "#" + [r, g, b].map((c) => c.toString(16).padStart(2, "0")).join("");
    };
    const colors = {
      background: hex("--color-base-100"),
      primary: hex("--color-primary"),
      secondary: hex("--color-secondary"),
      accent: hex("--color-accent"),
      neutral: hex("--color-neutral"),
    };
    probe.remove();
    return colors;
  }

  /** Custom colors starting from a theme, with all of its colors filled in to change. */
  function startFrom(theme: string): CustomTheme {
    return { from: theme, ...colorsOf(theme), corners: draft?.corners ?? null };
  }

  /** Custom colors starting from the theme picked last. */
  async function customize() {
    const base = site.config.theme === "custom" ? (draft?.from ?? "light") : site.config.theme;
    // Colors saved earlier on the same theme are picked up again, not reset.
    if (draft?.from !== base) draft = startFrom(base);
    await saveDraft();
  }

  /** The custom colors saved earlier, as they were. */
  async function backToSaved() {
    await saveDraft();
  }

  async function saveDraft() {
    if (!draft) return;
    await save({ ...$state.snapshot(site.config), theme: "custom", custom_theme: $state.snapshot(draft) });
  }

  async function changeBase(theme: string) {
    draft = startFrom(theme);
    await saveDraft();
  }
</script>

<div class="grid items-start gap-8 xl:grid-cols-2">
  <div id="site-theme" class="flex flex-col gap-3">
    <p class="text-sm text-base-content/60">Saved as you pick it; build or preview again to see it on the website.</p>
    {#if error}
      <div role="alert" class="alert alert-error alert-soft text-sm">{error}</div>
    {/if}

    <section id="custom-theme"
      class={["flex flex-col gap-3 rounded-box border p-3", isCustom ? "border-primary ring-2 ring-primary" : "border-base-300"]}>
      <div class="flex flex-wrap items-center gap-3">
        <Palette class="size-4" />
        <span class="flex-1 font-medium">Custom colors</span>
        {#if !isCustom}
          {#if draft && draft.from !== site.config.theme}
            <button type="button" id="back-to-custom-theme" class="btn btn-ghost btn-sm" onclick={backToSaved}
              title="The custom colors saved earlier, based on {draft.from}">Back to my colors</button>
          {/if}
          <button type="button" id="use-custom-theme" class="btn btn-sm" onclick={customize}>
            Customize {site.config.theme}
          </button>
        {/if}
      </div>

      {#if isCustom && draft}
        <label class="flex flex-wrap items-center gap-3 text-sm">
          <span class="w-28">Start from</span>
          <select id="custom-theme-from" class="select select-sm w-48" value={draft.from}
            onchange={(e) => changeBase(e.currentTarget.value)}>
            {#each themes as theme (theme)}<option value={theme}>{theme}</option>{/each}
          </select>
          <span class="text-xs text-base-content/60">Changing it takes that theme's colors.</span>
        </label>
        <div class="grid gap-2 sm:grid-cols-2">
          {#each COLORS as color (color.key)}
            <label class="flex items-center gap-3 text-sm">
              <span class="w-28">{color.label}</span>
              <!-- `input` previews while dragging, `change` saves once the picker closes. -->
              <input type="color" id="custom-{color.key}" class="h-8 w-14 cursor-pointer rounded-field border border-base-300 bg-base-100"
                value={draft[color.key] ?? "#808080"}
                oninput={(e) => draft && (draft[color.key] = e.currentTarget.value)}
                onchange={saveDraft} />
              <code class="font-mono text-xs text-base-content/60">{draft[color.key] ?? "from theme"}</code>
            </label>
          {/each}
        </div>
        <div class="flex flex-wrap items-center gap-3 text-sm">
          <span class="w-28">Corners</span>
          <div class="join">
            {#each CORNERS as corner (corner.label)}
              <button type="button" class={["btn join-item btn-sm", draft.corners === corner.value && "btn-active"]}
                onclick={() => draft && ((draft.corners = corner.value), saveDraft())}>{corner.label}</button>
            {/each}
          </div>
          <div class="flex-1"></div>
          <button type="button" id="reset-custom-theme" class="btn btn-ghost btn-sm" onclick={() => draft && changeBase(draft.from)}
            title="Take all colors from {draft.from} again">
            <RotateCcw class="size-4" /> Reset to {draft.from}
          </button>
        </div>
      {:else}
        <p class="text-sm text-base-content/60">
          Start from any theme and change its five colors and its corners. Text on each color is made readable by itself.
        </p>
      {/if}
    </section>

    <ThemePicker {themes} bind:value={() => site.config.theme, (theme) => pick(theme ?? "light")} />
  </div>

  <!-- Stays in view while the theme list scrolls. -->
  <div class="@container xl:sticky xl:top-4">
    <ThemeLegend theme={isCustom && draft ? draft.from : site.config.theme} {variables}
      label={isCustom && draft ? `custom, from ${draft.from}` : site.config.theme} />
  </div>
</div>

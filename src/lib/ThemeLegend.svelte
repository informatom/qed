<script lang="ts">
  import SiteSketch from "$lib/SiteSketch.svelte";
  import type { Colors, HeaderStyle, ThemeVariable } from "$lib/site";
  import { usesOf } from "$lib/siteColors";

  interface Props {
    /** The site's theme; the legend is drawn in it. */
    theme: string;
    /** A custom theme's variables on top of `theme`. */
    variables?: ThemeVariable[];
    /** The name to show, e.g. "custom". */
    label?: string;
    header?: HeaderStyle;
    colors?: Colors;
  }

  let { theme, variables = [], label = theme, header = "plain", colors }: Props = $props();
  // Inline variables win over the theme's own, as `:root[data-theme]` does on the site.
  let style = $derived(variables.map((v) => `${v.name}: ${v.value}`).join("; "));

  /** The swatches in ThemePicker, left to right, and where generated sites use them. */
  let LEGEND = $derived([
    { color: "Primary", swatch: "bg-primary text-primary-content", uses: usesOf("primary", colors, header) },
    { color: "Secondary", swatch: "bg-secondary text-secondary-content", uses: usesOf("secondary", colors, header) },
    { color: "Accent", swatch: "bg-accent text-accent-content", uses: usesOf("accent", colors, header) },
    { color: "Neutral", swatch: "bg-neutral text-neutral-content", uses: usesOf("neutral", colors, header) },
  ]);
</script>

<!-- What the four swatches in the theme picker stand for. The box keeps the
     app's look; only the sketch and the numbered squares are drawn in the
     site's theme, so a dark theme doesn't swallow the legend. -->
<aside id="theme-legend" class="flex flex-col gap-3 rounded-box border border-base-300 bg-base-100 p-4 text-sm">
  <span class="font-medium">Where the theme's colors go <span class="font-normal text-base-content/60">· {label}</span></span>
  <div class="grid items-start gap-4 @lg:grid-cols-[minmax(0,15rem)_1fr]">
    <div data-theme={theme} {style} class="bg-transparent">
      <SiteSketch {header} {colors} />
    </div>
    <div class="flex flex-col gap-3">
    {#each LEGEND as entry, i (entry.color)}
      <div class="flex items-start gap-3">
        <span data-theme={theme} {style} class={["mt-0.5 flex size-5 shrink-0 items-center justify-center rounded-field text-[0.65rem] font-semibold", entry.swatch]}>{i + 1}</span>
        <span class="flex flex-col">
          <span class="font-medium">{entry.color}</span>
          <span class="text-base-content/70 first-letter:uppercase">{entry.uses}</span>
        </span>
      </div>
    {/each}
    </div>
  </div>
  <p class="border-t border-base-300 pt-3 text-base-content/70">
    Background, text and borders come from the theme's base colors, shown behind each swatch.
  </p>
</aside>

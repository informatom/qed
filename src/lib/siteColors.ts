// The site's color choices (`[colors]` in site.toml, `Colors` in site.rs):
// which of the theme's colors goes where. Shared by the picker in Site →
// Theme, the legend and the sketch of a page.

import type { Colors, HeaderStyle, Surface, ThemeColor } from "$lib/site";

export type ColorKey = keyof Colors;

export interface ColorSetting {
  key: ColorKey;
  label: string;
  /** How it reads in the legend, after the color's name. */
  uses: string;
  /** Theme colors only, or also plain and shaded. */
  surface: boolean;
  /** What empty means; the defaults are how sites looked before. */
  fallback: Surface;
}

export const COLOR_SETTINGS: ColorSetting[] = [
  { key: "links", label: "Links", uses: "underline of links in text", surface: false, fallback: "primary" },
  { key: "buttons", label: "Buttons", uses: "buttons", surface: false, fallback: "primary" },
  { key: "hero", label: "Hero", uses: "Hero without an image", surface: true, fallback: "secondary" },
  { key: "call_to_action", label: "Call to action", uses: "Call to action box", surface: true, fallback: "primary" },
  { key: "boxes", label: "Boxed columns", uses: "boxed columns", surface: true, fallback: "plain" },
  { key: "content", label: "Content background", uses: "tint of the page background", surface: true, fallback: "plain" },
  { key: "footer", label: "Footer", uses: "footer", surface: true, fallback: "accent" },
];

export const THEME_COLORS: ThemeColor[] = ["primary", "secondary", "accent", "neutral"];
export const SURFACES: Surface[] = ["plain", "shaded", ...THEME_COLORS];

/** The choice in effect for `key`: the one saved, or the default. */
export function colorOf(colors: Colors | undefined, key: ColorKey): Surface {
  const setting = COLOR_SETTINGS.find((s) => s.key === key);
  return colors?.[key] ?? setting?.fallback ?? "plain";
}

/** Whole class names for the app's Tailwind: an area filled with `surface`. */
export const FILLED: Record<Surface, string> = {
  plain: "border border-base-300 bg-base-100 text-base-content",
  shaded: "bg-base-200 text-base-content",
  primary: "bg-primary text-primary-content",
  secondary: "bg-secondary text-secondary-content",
  accent: "bg-accent text-accent-content",
  neutral: "bg-neutral text-neutral-content",
};

/** A button on an area filled with `surface`: plain on its own color (as color_classes in generator.rs). */
export function buttonOn(colors: Colors | undefined, surface: Surface): string {
  const button = colorOf(colors, "buttons");
  if (button === surface) return "bg-base-100";
  return { primary: "bg-primary", secondary: "bg-secondary", accent: "bg-accent", neutral: "bg-neutral" }[button as ThemeColor] ?? "bg-primary";
}

/** The page background: a light tint of a theme color, as on the site. */
export function contentBackground(colors: Colors | undefined): string | null {
  const content = colorOf(colors, "content");
  if (content === "plain") return null;
  if (content === "shaded") return "var(--color-base-200)";
  return `color-mix(in oklab, var(--color-${content}) 10%, var(--color-base-100))`;
}

/** Where a theme color shows up on the site with these choices. */
export function usesOf(color: ThemeColor, colors: Colors | undefined, header: HeaderStyle): string {
  const fixed: Record<ThemeColor, string[]> = {
    primary: header === "plain" ? ["current page in the top menu"] : [],
    secondary: ["dates in Post lists"],
    accent: header === "plain" ? ["stripe along the top of the header", "bar beside quotes"] : ["bar beside quotes"],
    neutral: ["current page in the sidebar menu", "file type in Download blocks"],
  };
  const chosen = COLOR_SETTINGS.filter((s) => colorOf(colors, s.key) === color).map((s) => s.uses);
  if (header === color) chosen.push("the header bar");
  const all = [...chosen, ...fixed[color]];
  return all.length ? all.join(", ") : "nothing else";
}

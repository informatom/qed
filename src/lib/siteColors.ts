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
  /** What empty means on a theme with a dark page background, if that differs. */
  darkFallback?: Surface;
}

export const COLOR_SETTINGS: ColorSetting[] = [
  { key: "links", label: "Links", uses: "underline of links in text", surface: false, fallback: "primary" },
  { key: "buttons", label: "Buttons", uses: "buttons", surface: false, fallback: "primary" },
  { key: "hero", label: "Hero", uses: "Hero without an image", surface: true, fallback: "secondary" },
  { key: "call_to_action", label: "Call to action", uses: "Call to action box", surface: true, fallback: "primary" },
  { key: "boxes", label: "Boxed columns", uses: "boxed columns", surface: true, fallback: "plain" },
  { key: "content", label: "Content background", uses: "tint of the page background", surface: true, fallback: "plain" },
  { key: "footer", label: "Footer", uses: "footer", surface: true, fallback: "accent", darkFallback: "shaded" },
];

export const THEME_COLORS: ThemeColor[] = ["primary", "secondary", "accent", "neutral"];
export const SURFACES: Surface[] = ["plain", "shaded", ...THEME_COLORS];

/** What an empty setting means: on a dark theme, the footer is shaded (as color_classes in generator.rs). */
export function fallbackOf(setting: ColorSetting, dark: boolean): Surface {
  return (dark && setting.darkFallback) || setting.fallback;
}

/** The choice in effect for `key`: the one saved, or the default. */
export function colorOf(colors: Colors | undefined, key: ColorKey, dark = false): Surface {
  const setting = COLOR_SETTINGS.find((s) => s.key === key);
  return colors?.[key] ?? (setting ? fallbackOf(setting, dark) : "plain");
}

/** Whether light text reads better on `#rrggbb` than dark text: a dark page background (as is_light in theme.rs). */
export function isDarkHex(hex: string): boolean | null {
  const match = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(hex.trim());
  if (!match) return null;
  const digits = match[1].length === 3 ? [...match[1]].map((d) => d + d).join("") : match[1];
  const channel = (i: number) => {
    const c = parseInt(digits.slice(i, i + 2), 16) / 255;
    return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  };
  const l = 0.2126 * channel(0) + 0.7152 * channel(2) + 0.0722 * channel(4);
  return (l + 0.05) / 0.05 <= 1.05 / (l + 0.05);
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
export function usesOf(color: ThemeColor, colors: Colors | undefined, header: HeaderStyle, dark = false): string {
  const fixed: Record<ThemeColor, string[]> = {
    primary: header === "plain" ? ["current page in the top menu"] : [],
    secondary: ["dates in Post lists"],
    accent: header === "plain" ? ["stripe along the top of the header", "bar beside quotes"] : ["bar beside quotes"],
    neutral: ["current page in the sidebar menu", "file type in Download blocks"],
  };
  const chosen = COLOR_SETTINGS.filter((s) => colorOf(colors, s.key, dark) === color).map((s) => s.uses);
  if (header === color) chosen.push("the header bar");
  const all = [...chosen, ...fixed[color]];
  return all.length ? all.join(", ") : "nothing else";
}

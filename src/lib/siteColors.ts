// The site's color choices (`[colors]` in site.toml, `Colors` in site.rs):
// which of the theme's colors goes where. Shared by the picker in Site →
// Theme, the legend and the sketch of a page.

import type { Colors, HeaderStyle, Surface, ThemeColor } from "$lib/site";

/** The color choices; `hero_solid` and `call_to_action_solid` are switches of their own. */
export type ColorKey = Exclude<keyof Colors, "hero_solid" | "call_to_action_solid">;

/** The areas that are outlined unless set to solid (#163), and their switch. */
export const SOLID_KEYS: Partial<Record<ColorKey, "hero_solid" | "call_to_action_solid">> = {
  hero: "hero_solid",
  call_to_action: "call_to_action_solid",
};

/** Whether `key` is filled with its color: always, unless it's one that's outlined by default. */
export function isSolid(colors: Colors | undefined, key: ColorKey): boolean {
  const solid = SOLID_KEYS[key];
  return solid ? (colors?.[solid] ?? false) : true;
}

export interface ColorSetting {
  key: ColorKey;
  label: string;
  /** How it reads in the legend, after the color's name. */
  uses: string;
  /** The options it offers. */
  choices: readonly Surface[];
  /** What empty means; the defaults are how sites looked before. */
  fallback: Surface;
  /** What empty means on a theme with a dark page background, if that differs. */
  darkFallback?: Surface;
}

export const THEME_COLORS: ThemeColor[] = ["primary", "secondary", "accent", "neutral"];
export const SURFACES: Surface[] = ["plain", "shaded", ...THEME_COLORS];
/** Text: the text color (plain) or a deep tone of a theme color. */
export const TEXT_COLORS: Surface[] = ["plain", ...THEME_COLORS];

export const COLOR_SETTINGS: ColorSetting[] = [
  { key: "links", label: "Links", uses: "underline of links in text", choices: THEME_COLORS, fallback: "primary" },
  { key: "headings", label: "Headings", uses: "headings, in a deep tone", choices: TEXT_COLORS, fallback: "primary" },
  { key: "buttons", label: "Buttons", uses: "buttons", choices: THEME_COLORS, fallback: "primary" },
  { key: "hero", label: "Hero", uses: "Hero without an image", choices: SURFACES, fallback: "secondary" },
  { key: "call_to_action", label: "Call to action", uses: "Call to action box", choices: SURFACES, fallback: "primary" },
  { key: "boxes", label: "Boxed columns", uses: "boxed columns", choices: SURFACES, fallback: "plain" },
  { key: "content", label: "Content background", uses: "tint of the page background", choices: SURFACES, fallback: "plain" },
  { key: "footer", label: "Footer", uses: "footer", choices: SURFACES, fallback: "plain" },
];

/** A heading's color in a deep tone of `color` (as blocks.css): mixed with the text color. */
export function headingTone(color: Surface): string {
  return color === "plain" || color === "shaded"
    ? "var(--color-base-content)"
    : `color-mix(in oklch, var(--color-${color}) 50%, var(--color-base-content))`;
}

/** What an empty setting means, on a theme with a dark page background too. */
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

/** Whole class names: an area outlined in `surface` (a light shade, a border in the color; blocks.css), or filled for plain and shaded. */
export const OUTLINED: Record<Surface, string> = {
  plain: FILLED.plain,
  shaded: FILLED.shaded,
  primary: "outlined-primary",
  secondary: "outlined-secondary",
  accent: "outlined-accent",
  neutral: "outlined-neutral",
};

/** The classes of the area `key` is drawn in: outlined or filled. */
export function areaOf(colors: Colors | undefined, key: ColorKey, dark = false): string {
  const surface = colorOf(colors, key, dark);
  return isSolid(colors, key) ? FILLED[surface] : OUTLINED[surface];
}

/** A button on an area filled with `surface`: plain on its own color (as color_classes in generator.rs); on an outlined area it stands out anyway. */
export function buttonOn(colors: Colors | undefined, surface: Surface, solid = true): string {
  const button = colorOf(colors, "buttons");
  if (solid && button === surface) return "bg-base-100";
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
    accent: [
      ...(header === "plain" ? ["stripe along the top of the header"] : []),
      ...(colorOf(colors, "footer", dark) === "plain" ? ["line along the top of the footer"] : []),
      "bar beside quotes",
    ],
    neutral: ["current page in the sidebar menu", "file type in Download blocks"],
  };
  const chosen = COLOR_SETTINGS.filter((s) => colorOf(colors, s.key, dark) === color).map((s) => s.uses);
  if (header === color) chosen.push("the header bar");
  const all = [...chosen, ...fixed[color]];
  return all.length ? all.join(", ") : "nothing else";
}

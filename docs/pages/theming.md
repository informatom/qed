+++
title = "Theming notes"
description = "How q.e.d. uses the daisyUI theme colors, and what color systems recommend instead (backlog #163)."
+++

Background for backlog #163 and Phase 3.5 "Theming" in the
[Backlog](backlog.md). Collected on 2026-11-20, decided and built on
2026-11-23 (see Decisions; the items are in [Done](done.md)).

## What prompted it

On ki.mittenin.at in light mode (catppuccin-latte based custom theme):

- Text is black everywhere outside buttons: running text, headings, and
  link text (links only get an underline in the link color).
- The Call to action is set to secondary and is the biggest color patch
  on the page; primary only shows in buttons.
- A Hero in a theme color dominates the start page: on a full HD screen
  it takes about 456 of the 845 px below the browser bar, more than half,
  and more on a phone. In accent, the purple primary button on it has
  little contrast.

## daisyUI's roles

From [daisyUI: Colors](https://daisyui.com/docs/colors/) (v5.8):

| Color | daisyUI's description |
|---|---|
| primary | "The main color of your brand" |
| secondary | "The optional, secondary color of your brand" |
| accent | "The optional, accent color of your brand" |
| neutral | "For not-saturated parts of UI" |
| base-100 | "surface color of page, used for blank backgrounds" (q.e.d.'s *plain*) |
| base-200 | "darker shade, to create elevations" (q.e.d.'s *shaded*) |
| base-300 | "even more darker shade, to create elevations" |
| base-content | "the text color of the page by default" |

Each color has a `-content` color for text on it. daisyUI says nothing
about how much of each color a page should have, but its components
show the intent: brand colors for buttons, badges and links; the `link`
class sets no color (`link-primary` is opt-in); the
[Hero](https://daisyui.com/components/hero/) example uses `bg-base-200`,
not a brand color.

## Where q.e.d. puts them today

Defaults (`color_classes` in crates/qed-core/src/generator.rs): Hero
secondary, Call to action primary, footer accent (light) or shaded
(dark), buttons and link underlines primary. Text stays base-content
everywhere, so that it's readable in every theme (blocks.css): some
themes have primaries too light for text on white. In the
[@tailwindcss/typography](https://github.com/tailwindlabs/tailwindcss-typography)
plugin daisyUI maps prose body text to 80 % base-content, headings and
link text to base-content.

So three brand colors each get a big area, while primary, the main
color, shows only in small places.

## What color systems do

The established guides don't treat a brand color as one shade but as a
scale, with jobs for each part:

| Guide | Shades per color | Big areas | Solid color | Text |
|---|---|---|---|---|
| [Radix Colors](https://www.radix-ui.com/colors/docs/palette-composition/understanding-the-scale) | 12 steps | 1–2 "App / subtle background", 3–5 element backgrounds | 9 "Solid backgrounds", the purest step | 11–12 low- and high-contrast text |
| [Material Design 3](https://developer.android.com/design/ui/mobile/guides/styles/color) | tonal palette | *surfaces* "represent the majority of your app's colors"; *containers* are "less vibrant", to avoid oversaturation | primary for "crucial calls to action" | *on-container* colors, accessible by design |
| [Refactoring UI](https://www.refactoringui.com/previews/building-your-color-palette) | 5–10 | "almost everything in an interface is grey"; ultra-light shades "tint the background" | the base shade "would work well as a button background" | the darkest shade "usually reserved for text" |

Radix's steps in full: 1 app background, 2 subtle background, 3 UI
element background, 4 hovered, 5 active/selected, 6 subtle borders and
separators, 7 UI element border and focus rings, 8 hovered border, 9
solid backgrounds, 10 hovered solid, 11 low-contrast text, 12
high-contrast text.

**The gap:** daisyUI has only the solid step (primary) and the text on
it (primary-content). The light shades for areas and the dark shade
for text are missing. So q.e.d. can offer a color only as a full patch
or not at all, which explains both the shouting patches and the black
text.

## Hero and Call to action

- **Hero:** the usual advice is one clear message and one focused
  button at the top of the page; nothing says it must be a solid patch.
  Options:
  - plain or shaded, big headline, solid primary button (daisyUI's
    example);
  - a *primary container* (Material): a light tone of the primary hue,
    text in a deep tone of the same hue, the solid primary left for the
    button. Not the same as a pale tint with grey text (rejected,
    see the [Backlog](backlog.md));
  - plain with a border in primary (Radix step 7–8);
  - solid, but low.
- **Call to action:** here Material does call for solid primary
  ("crucial calls to action"). A solid band works if it's the only one,
  short, and its button contrasts (q.e.d. already draws it base-100 on
  primary).
- **Height** is a setting of its own, apart from color: a solid band at
  200 px is easier to accept than at 450 px.

## A model to work from

Material 3's roles map onto daisyUI almost one to one and name what's
missing:

| Material 3 | daisyUI today | |
|---|---|---|
| surface, surface-container | base-100, base-200, base-300 (plain, shaded) | there |
| primary, on-primary | primary, primary-content | there |
| primary-container, on-primary-container | – | missing |
| outline | (base-300 only) | missing per color |

The missing shades could be derived per theme in OKLCH with
`color-mix`, as Material derives its tones from the brand color:
container = the color mixed with base-100, on-container = the color
mixed with base-content, checked for contrast (4.5:1). That works for
every daisyUI theme and in dark mode, where the container becomes a
dark tone. Radix's 12-step table serves as the checklist for which
shade does what; Refactoring UI explains why.

## Decisions (2026-11-23)

- **Material 3's role model** is the one to follow (container,
  on-container, outline derived per theme).
- **Big solid areas get a colored border** as a choice: "making it stand
  out, but not kick you in the brain" (user). At least Hero and Call to
  action only: boxes and sections don't get it (user: "that would be
  too much"); the footer gets an accent line on top instead.
  The border **replaces the solid fill as the default**, with a lightly
  colored container shade inside; solid stays a choice.
- **No height setting.** A Hero without an image only takes the height
  its text needs; with an image it keeps room for the picture.
- **New defaults may change existing sites**, production sites included.
  The borders give q.e.d. sites a recognizable look across all palettes,
  and that's intended (user: q.e.d. is open source, whoever wants
  otherwise can fork it).

## Open questions

- Is Material 3's role model the one to follow? (Yes, see above.)
- Which areas could take a container shade (Hero, Call to action,
  boxes, sections), and is it an extra choice or replaces anything?
- Borders in a theme color as a choice for Hero and boxes?
- A height setting for the Hero (and the Call to action)?
- Text in theme colors (#160): with an on-container / dark shade, link
  text and headings could be colored and still readable.
- New defaults (#162) and how to keep the look of sites in production.

## Sources

- [daisyUI: Colors](https://daisyui.com/docs/colors/),
  [Hero](https://daisyui.com/components/hero/),
  [Link](https://daisyui.com/components/link/)
- [Radix Colors: Understanding the scale](https://www.radix-ui.com/colors/docs/palette-composition/understanding-the-scale)
- [Android / Material 3: Color for mobile design](https://developer.android.com/design/ui/mobile/guides/styles/color),
  [Wear OS: Color roles and tokens](https://developer.android.com/design/ui/wear/guides/styles/color/roles-tokens),
  [Material blog: Tone-based surfaces](https://material.io/blog/tone-based-surface-color-m3)
- [Refactoring UI: Building your color palette](https://www.refactoringui.com/previews/building-your-color-palette)
- Hero practice:
  [LogRocket: Hero section examples](https://blog.logrocket.com/ux-design/hero-section-examples-best-practices/),
  [Progress: Hero section best practices](https://www.progress.com/blogs/web-design-best-practices-hero-section)

# pulldown-latex: stylesheet and math font

Math in text (#107): q.e.d. turns LaTeX formulas into MathML with the
[pulldown-latex](https://github.com/carloskiki/pulldown-latex) crate
(0.8.0, MIT license) when it builds a site. Browsers draw MathML
themselves; this stylesheet and font make it look like LaTeX.

## Where it comes from

From the release v0.7.1 of pulldown-latex (2024-11-18, the newest
release with files; the crate is 0.8.0), downloaded on 2026-10-28 from
<https://github.com/carloskiki/pulldown-latex/releases/tag/v0.7.1>:

- `latinmodern-math.woff2`: from `font.zip`, unchanged. Latin Modern
  Math, the math font of LaTeX's Latin Modern family, by the GUST e-foundry
  (GUST Font License, free to use and pass on).
- `math.css`: from `styles.css`.

## Changes

- `styles.css` uses `@namespace m url(…MathML)` and `m|math` selectors.
  q.e.d. appends the rules to the site's `style.css`, where an
  `@namespace` after other rules is ignored, so the selectors are plain
  element names (`math`, `mtd`, …), which match MathML in HTML pages
  just as well.
- Only the math font: the Latin Modern Roman text fonts (`LMRoman12`)
  are left out, so words inside formulas (`\text{…}`) use the site's
  font. The font's address is `math/latinmodern-math.woff2`, with
  `font-display: swap`.
- Added: formulas are 1.15 times the text size; Latin Modern's small
  letters are low and would look small next to the site's font.
- Added: formulas on their own line scroll sideways when they're wider
  than the page (phones).
- The slash through negated operators uses the text color instead of
  black, for dark themes.

## How q.e.d. uses it

`generator.rs` appends `math.css` to the site's `style.css` and copies
the font to `_site/math/` only if a built page contains `<math`. The
browser downloads the font only on pages that show a formula.

## Updating

Download `font.zip` and `styles.css` from a newer release, replace the
font, and redo the changes above in `math.css` (search `m|`). Build the
docs site and look at the post "Math formulas" (fractions, sums,
`\le`, a mistake in red).

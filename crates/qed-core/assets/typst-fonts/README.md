# Fonts for the site's PDF

The fonts Typst sets the whole-site PDF with (#154, `src/pdf.rs`), compiled
into q.e.d. by `include_bytes!`, so the PDF looks the same on every
computer and nothing has to be installed.

- **Where from:** the `typst-assets` crate, version 0.15.1 (the version of
  the `typst` crate q.e.d. uses), folder `files/fonts/`, copied on
  2026-11-06.
- **Which:** Libertinus Serif in four styles (the text), DejaVu Sans Mono
  regular and bold (code), New Computer Modern Math (formulas). The crate
  has more (Libertinus Semibold, New Computer Modern text, the PDF
  standard fonts); they're left out to keep the app small, about 3 MB
  instead of 10.
- **Changed:** nothing.
- **Licenses:** `NOTICE` is the crate's own, unchanged: SIL Open Font
  License 1.1 for Libertinus, the Bitstream Vera license (public domain
  changes) for DejaVu, the GUST Font License for New Computer Modern
  Math.
- **Updating:** after updating `typst`, copy the same files from the new
  `typst-assets` (`~/.cargo/registry/src/*/typst-assets-<version>/files/fonts/`)
  and its `NOTICE`, and change the version above.

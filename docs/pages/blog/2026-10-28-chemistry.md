+++
title = "Chemistry"
description = "Chemistry in text: structure diagrams from SMILES and reaction equations with \\ce, drawn when the site is built"
menu = "main"
weight = 58
date = 2026-10-28T21:03:00
authors = ["Claude"]
tags = ["rust", "markdown", "chemistry", "svg"]
summary = "After math, chemistry (#130): a ```smiles block draws a molecule's structure as SVG, and \\ce{…} in a formula writes chemical formulas and reaction equations the way chemists do. Both happen when the site is built, in pure Rust."
+++

Formulas came first ([Math formulas](2026-10-28-math-formulas.md)), then
the question whether q.e.d. could do chemistry too (#130). It can, in two
ways: structure diagrams of molecules, and chemical formulas and reaction
equations in the line of text.

## Caffeine

```smiles
CN1C=NC2=C1C(=O)N(C(=O)N2C)C
Caffeine, C₈H₁₀N₄O₂
```

That's written like this in a Text block:

````markdown
```smiles
CN1C=NC2=C1C(=O)N(C(=O)N2C)C
Caffeine, C₈H₁₀N₄O₂
```
````

The first line is the molecule in
[SMILES](https://en.wikipedia.org/wiki/Simplified_Molecular_Input_Line_Entry_System),
the one-line notation chemists use: `C` and `N` are carbon and nitrogen
atoms, digits open and close rings, `=` is a double bond, brackets are
branches. Hydrogen is left out; every carbon gets as many as it needs.
You don't have to write SMILES yourself. Wikipedia lists it in the box on
every molecule's page, and so does [PubChem](https://pubchem.ncbi.nlm.nih.gov).
The second line, if there is one, becomes the caption.

A few more, to show what the layout handles: a ring with alternating
double bonds, and four rings fused together.

```smiles
CC(=O)OC1=CC=CC=C1C(=O)O
Aspirin
```

```smiles
C[C@H](CCCC(C)C)[C@H]1CC[C@@H]2[C@@]1(CC[C@H]3[C@H]2CC=C4[C@@]3(CC[C@@H](C4)O)C)C
Cholesterol
```

## Photosynthesis

The most famous equation in chemistry may be the one plants solve every
day:

$$\ce{6CO2 + 6H2O ->[light] C6H12O6 + 6O2}$$

Written as:

```markdown
$$\ce{6CO2 + 6H2O ->[light] C6H12O6 + 6O2}$$
```

`\ce{…}` comes from mhchem, the LaTeX package chemists write equations
with. Inside it, a number after an element is a subscript, a number in
front is a coefficient, and `->` is the reaction arrow; text in brackets
after it goes above the arrow. In the line of text it works with single
dollar signs: water is $\ce{H2O}$, table salt in water is
$\ce{Na+(aq) + Cl-(aq)}$, sulfate is $\ce{SO4^2-}$, and burning lime goes

$$\ce{CaCO3 ->[\Delta] CaO + CO2 ^}$$

q.e.d. understands a part of mhchem: formulas with brackets
(`Ca(OH)2`, `[Cu(NH3)4]^2+`), coefficients (`2H2O`, `1/2O2`), charges
(`Na+`, `SO4^2-`), states (`(aq)`, `(s)`, `(l)`, `(g)`), hydrates
(`CuSO4*5H2O`), the arrows `->`, `<-`, `<->`, `<=>` and `^`/`v` for gas
and precipitate. Anything else in `\ce{…}` is passed on as LaTeX.

## Drawing a molecule in Rust

The diagrams come from [chematic](https://github.com/kent-tokyo/chematic),
a chemistry toolkit in pure Rust. `molecule` in
`crates/qed-core/src/chem.rs` does three steps:

```rust
// Atoms and bonds, from the SMILES.
let mol = chematic_smiles::parse(smiles)?;
// Where each atom goes on the page.
let layout = compute_layout(&mol);
// Lines and letters.
let svg = render_svg_opts(&mol, &layout, &options);
```

The middle one is the hard part. SMILES says which atom is bonded to
which, but not where anything is. `compute_layout` places rings as
regular polygons, fuses them along shared edges and lays chains out in a
zigzag. It takes well under a millisecond for cholesterol.

`kekulize: true` in the options draws benzene rings with alternating
double bonds. In SMILES, aromatic rings are often written in lowercase
(`c1ccccc1`), and without this option they'd come out with dashed
bonds.

The SVG is drawn in black. q.e.d. replaces `"black"` with
`"currentColor"`, the color of the text around it, so molecules stay
visible on dark themes. Oxygen stays red and nitrogen blue, the colors
chemists are used to. Behind each atom label is a small box in the page's
color, so the bonds stop short of the letters. In the site's stylesheet,
`.molecule rect { fill: var(--color-base-100) }` gives the boxes the
theme's background. A CSS rule beats an SVG attribute like
`fill="white"`, so no change to the SVG is needed for that.

## From \ce to LaTeX

pulldown-latex, which turns the formulas into MathML, doesn't know
`\ce`. So `expand_ce` translates it into LaTeX it does know, before the
formula is parsed:

```rust
assert_eq!(
    expand_ce(r"\ce{2H2 + O2 -> 2H2O}"),
    r"{2\,\mathrm{H}_{2} + \mathrm{O}_{2} \longrightarrow 2\,\mathrm{H}_{2}\mathrm{O}}"
);
```

Element symbols go in `\mathrm` (upright, as chemistry wants, while math
sets letters in italics), digits after them become subscripts, and
`\,` is a thin space after a coefficient. The `r"…"` is a
[raw string](https://doc.rust-lang.org/reference/tokens.html#raw-string-literals):
backslashes in it are just backslashes, which spares writing every LaTeX
command as `\\mathrm`.

`species` reads a formula character by character, from a
`Vec<char>` with an index `i` rather than with an iterator. The
decisions look ahead and back: a `(` starts a state like `(aq)` only if
lowercase letters and a `)` follow, and a `+` at the end is a charge
only if nothing but `+` comes after it. An index makes looking around
easy, and with `Vec<char>` each index is a whole character. Indexing a
`String` counts bytes, and `·` or `→` are two or three bytes long.

Mistakes work as for math: an invalid SMILES shows in red where the
diagram would be, with the problem on hover.

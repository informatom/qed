+++
title = "Math formulas"
description = "Math in text: LaTeX becomes MathML when the site is built, with placeholders through the Markdown tree, and Euclid as the test"
menu = "main"
weight = 57
date = 2026-10-28
authors = ["Claude"]
tags = ["rust", "markdown", "mathml", "html"]
summary = "Text blocks can contain formulas now (#107): LaTeX between dollar signs becomes MathML when the site is built, and the browser draws it. No JavaScript, no extra files. In Rust: changing comrak's document tree, and placeholders for HTML the escaping would otherwise spoil."
+++

Text blocks can contain math now (#107). You write the formula in LaTeX,
the notation mathematicians have used for decades, and the built site
shows it typeset. Here's the test the owner suggested: Euclid's proof
that there is no largest prime.

## Euclid's proof

**Theorem.** There are infinitely many prime numbers.

*Proof.* Suppose, for the sake of contradiction, that there are only
finitely many prime numbers, say

$$p_1, p_2, \dots, p_n.$$

Consider the integer $N$ defined as the product of all these primes plus
one:

$$N = (p_1 \cdot p_2 \cdots p_n) + 1.$$

The number $N$ must be divisible by at least one prime number $p$.
However, $N$ leaves a remainder of $1$ when divided by any of the listed
primes $p_i$ (where $1 \le i \le n$).

Therefore, $p$ cannot be any of the primes $p_1, p_2, \dots, p_n$. This
implies there exists a prime $p$ not in our supposedly complete list of
all primes, which is a contradiction.

Thus, our initial assumption must be false, and there are infinitely many
prime numbers. ∎

## How to write it

The proof above is written like this in the Text block:

```markdown
Consider the integer $N$ defined as the product of all these primes plus
one:

$$N = (p_1 \cdot p_2 \cdots p_n) + 1.$$
```

- `$…$` is a formula in the line of text: $a^2 + b^2 = c^2$.
- `$$…$$` on a line of its own is a formula on its own, centered.
- A ```` ```math ```` code block does the same as `$$…$$`, for longer
  formulas over several lines:

```math
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
```

Only formulas are LaTeX; the text around them stays Markdown. A whole
LaTeX document (`\documentclass`, `\begin{theorem}`, `\begin{proof}`)
doesn't work. The original of the proof above was one, and its theorem
and proof became bold and italic text, its `\[ … \]` became `$$ … $$`.

Dollar signs in ordinary text mostly stay what they are. "Tickets cost
\$5 and \$10" has no formula in it: a `$` that has a space or a digit
after it can't end one (the rules come from Pandoc). It goes wrong when a
paragraph has a price and, later on, a dollar sign that can end a
formula. Then everything in between becomes one formula. The first draft
of this very post did that. A backslash prevents it: `\$` is always a
plain dollar sign.

A mistake in a formula doesn't stop the build. The formula shows as its
LaTeX in red, and the problem appears when you point at it:
$\frac{1}$.

The look is LaTeX's: q.e.d. adds the Latin Modern Math font and
pulldown-latex's stylesheet to sites that have a formula, and the
browser loads the font only on pages that show one
(`assets/pulldown-latex/README.md`).

## The browser draws it

[MathML](https://developer.mozilla.org/en-US/docs/Web/MathML) is HTML for
formulas. `$x^2$` becomes

```html
<math display="inline"><semantics><mrow>
  <msup><mi>x</mi><mn>2</mn></msup>
</mrow><annotation encoding="application/x-tex">x^2</annotation></semantics></math>
```

`msup` is "with superscript", `mi` an identifier, `mn` a number.
Chrome, Edge, Firefox and Safari all draw MathML themselves, Chrome since
2023. Other sites often use KaTeX or MathJax instead: JavaScript that
does the typesetting in the visitor's browser, with its own fonts to
load. With MathML the work is done once, when the site is built. The
`annotation` keeps the LaTeX source, so copying a formula copies it too.

The conversion is the crate
[pulldown-latex](https://docs.rs/pulldown-latex), in `mathml` in
`crates/qed-core/src/markdown.rs`:

```rust
let storage = Storage::new();
let mut html = String::new();
push_mathml(&mut html, Parser::new(latex, &storage), config)
```

`Storage` is memory the parser uses while it works (an
[arena](https://docs.rs/bumpalo), like comrak's). The parser borrows
it, so it must live at least as long as the parser does. That's why it
has a `let` of its own instead of being made inside the call.

## Placeholders through the tree

comrak already knows the dollar notation. With `math_dollars` and
`math_code` switched on in `options()`, the formulas arrive as
`NodeValue::Math` nodes in the document tree, and ```` ```math ```` as a
code block with the language `math`. `render` already walks the tree to
rewrite links and images. Now it also collects the math nodes, and
afterwards replaces each one:

```rust
let token = format!("{MATH_MARK}{}{MATH_MARK}", formulas.len());
formulas.push((token.clone(), mathml(&latex, display), display));
if block {
    node.data.borrow_mut().value = NodeValue::Paragraph;
    node.append(arena.alloc(NodeValue::Text(token.into()).into()));
} else {
    node.data.borrow_mut().value = NodeValue::Text(token.into());
}
```

Why not put the MathML straight into the tree? q.e.d. escapes raw HTML in
Markdown (`render.escape`), so a `<script>` typed into a page shows as
text instead of running. That rule applies to HTML nodes in the tree too,
and the MathML would come out as `&lt;math&gt;`. So each formula
becomes a placeholder, plain text that comrak writes out unchanged, and
after formatting `html.replace(token, mathml)` swaps the MathML in.
Images in text get their `srcset` the same way.

`MATH_MARK` is `'\u{E000}'`, the first character of Unicode's
[private use area](https://en.wikipedia.org/wiki/Private_Use_Areas):
characters that mean nothing unless a program agrees on a meaning. No
text contains one by accident, so the placeholders can't collide with
anything written in a page.

`node.data.borrow_mut()` is the
[`RefCell`](https://doc.rust-lang.org/std/cell/struct.RefCell.html) from
[Responsive images](2026-10-21-responsive-images.md): comrak's nodes can
be changed through a shared reference, and the borrow is checked while
the program runs. The loop collects the nodes first and changes them
afterwards, because the first loop already holds a `borrow_mut` on each
node it looks at. A second one at the same time would panic.

//! Chemistry in text: structure diagrams from SMILES, and chemical
//! formulas and equations in math (`\ce{…}`, a subset of LaTeX's mhchem).
//!
//! A ```` ```smiles ```` block in Markdown holds a molecule in SMILES, the
//! one-line notation chemists use (`CCO` is ethanol), and optionally a
//! caption on the next line. When the site is built, chematic draws it as
//! SVG: 2D layout, double bonds, atom labels in their usual colors. No
//! JavaScript, nothing loaded from elsewhere.
//!
//! `\ce{2H2 + O2 -> 2H2O}` inside `$…$` becomes ordinary LaTeX
//! (`2\,\mathrm{H}_{2} + …`) before pulldown-latex turns it into MathML;
//! see [`expand_ce`] for what's understood.

use chematic_depict::{RenderOptions, compute_layout, render_svg_opts};

/// The HTML for a ```` ```smiles ```` block: the molecule as SVG with its
/// caption, or, if the SMILES has a mistake, the SMILES in red with the
/// problem on hover.
pub fn molecule(block: &str) -> String {
    let mut lines = block.lines().map(str::trim).filter(|line| !line.is_empty());
    let smiles = lines.next().unwrap_or_default();
    let caption = lines.collect::<Vec<_>>().join(" ");
    let mol = match chematic_smiles::parse(smiles) {
        Ok(mol) if mol.atom_count() > 0 => mol,
        Ok(_) => return error(smiles, "no atoms"),
        Err(problem) => return error(smiles, &problem.to_string()),
    };
    let options = RenderOptions {
        // Double bonds drawn as such, not as an aromatic ring.
        kekulize: true,
        ..RenderOptions::default()
    };
    let svg = render_svg_opts(&mol, &compute_layout(&mol), &options);
    let label = if caption.is_empty() { smiles } else { &caption };
    // Black in the theme's text color, so the drawing works on dark
    // themes; the boxes behind atom labels get the page color from the
    // stylesheet (`.molecule rect`). The atoms' letters stay out of the
    // search index; the caption is what people search for.
    let svg = svg.replace("\"black\"", "\"currentColor\"").replacen(
        "<svg ",
        &format!(
            "<svg role=\"img\" aria-label=\"{}\" data-pagefind-ignore ",
            attribute(label)
        ),
        1,
    );
    let caption = match caption.is_empty() {
        true => String::new(),
        false => format!("<figcaption>{}</figcaption>", text(&caption)),
    };
    format!("<figure class=\"molecule\">{svg}{caption}</figure>")
}

fn error(source: &str, problem: &str) -> String {
    format!(
        "<p><code class=\"chem-error\" title=\"{}\">{}</code></p>",
        attribute(problem),
        text(source)
    )
}

fn text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn attribute(value: &str) -> String {
    text(value).replace('"', "&quot;")
}

/// Replaces every `\ce{…}` in a formula by plain LaTeX. Understood, as in
/// mhchem:
///
/// - formulas: `H2SO4`, `Ca(OH)2`, `[Cu(NH3)4]^2+`, with numbers after
///   an element or bracket as subscripts
/// - coefficients: `2H2O`, `1/2O2`
/// - charges: `Na+`, `Cl-`, `SO4^2-`, `Fe^{3+}`
/// - states: `(aq)`, `(s)`, `(l)`, `(g)`
/// - hydrates: `CuSO4*5H2O` or `CuSO4·5H2O`
/// - arrows: `->`, `<-`, `<->`, `<=>`, with text above and below for
///   `->` and `<-`: `->[\Delta]`, `->[heat][-H2O]`
/// - `+`, `=`, and `^` and `v` on their own for gas and precipitate
pub fn expand_ce(latex: &str) -> String {
    let mut out = String::new();
    let mut rest = latex;
    while let Some(start) = rest.find("\\ce{") {
        out.push_str(&rest[..start]);
        let inner_start = start + 4;
        let Some(len) = braced(&rest[inner_start..]) else {
            // No closing brace: leave it to the LaTeX parser to complain.
            out.push_str(&rest[start..]);
            return out;
        };
        out.push('{');
        out.push_str(&equation(&rest[inner_start..inner_start + len]));
        out.push('}');
        rest = &rest[inner_start + len + 1..];
    }
    out.push_str(rest);
    out
}

/// The length of the text up to the `}` that closes an already opened
/// brace.
fn braced(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in text.char_indices() {
        match c {
            '{' => depth += 1,
            '}' if depth == 0 => return Some(i),
            '}' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// One `\ce{…}`: tokens separated by spaces; brackets after an arrow
/// belong to it even with spaces inside.
fn equation(ce: &str) -> String {
    tokens(ce)
        .iter()
        .map(|token| token_latex(token))
        .collect::<Vec<_>>()
        .join(" ")
}

fn tokens(ce: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    for c in ce.chars() {
        match c {
            '[' | '{' => depth += 1,
            ']' | '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if c.is_whitespace() && depth == 0 {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn token_latex(token: &str) -> String {
    match token {
        "+" => return "+".to_string(),
        "=" => return "=".to_string(),
        "^" => return "\\uparrow".to_string(),
        "v" => return "\\downarrow".to_string(),
        _ => {}
    }
    for (arrow, plain, labeled) in [
        ("<=>", "\\rightleftharpoons", None),
        ("<->", "\\longleftrightarrow", None),
        ("->", "\\longrightarrow", Some("\\xrightarrow")),
        ("<-", "\\longleftarrow", Some("\\xleftarrow")),
    ] {
        if let Some(labels) = token.strip_prefix(arrow) {
            let labels = arrow_labels(labels);
            return match (labels.as_slice(), labeled) {
                ([], _) | (_, None) => plain.to_string(),
                ([above], Some(command)) => format!("{command}{{{}}}", label(above)),
                ([above, below, ..], Some(command)) => {
                    format!("{command}[{}]{{{}}}", label(below), label(above))
                }
            };
        }
    }
    // A hydrate's parts, joined by a centered dot.
    token
        .split(['*', '·'])
        .map(species)
        .collect::<Vec<_>>()
        .join(" \\cdot ")
}

/// `[above][below]` after an arrow.
fn arrow_labels(text: &str) -> Vec<String> {
    let mut labels = Vec::new();
    let mut rest = text;
    while let Some(inner) = rest.strip_prefix('[') {
        let Some(end) = inner.find(']') else { break };
        labels.push(inner[..end].to_string());
        rest = &inner[end + 1..];
    }
    labels
}

/// Text above or below an arrow: LaTeX if it has commands (`\Delta`), a
/// formula if it looks like one (`-H2O`), else words.
fn label(text: &str) -> String {
    if text.contains('\\') || text.contains('$') {
        text.replace('$', "")
    } else if text.chars().any(|c| c.is_ascii_uppercase()) && !text.contains(' ') {
        species(text)
    } else {
        format!("\\text{{{text}}}")
    }
}

/// One substance: coefficient, formula, charge, state.
fn species(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;

    // A coefficient: digits, maybe a fraction (`1/2`) or decimals.
    let start = i;
    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '/' || chars[i] == '.') {
        i += 1;
    }
    if i > start && i < chars.len() {
        let number: String = chars[start..i].iter().collect();
        match number.split_once('/') {
            Some((top, bottom)) => out.push_str(&format!("\\tfrac{{{top}}}{{{bottom}}}\\,")),
            None => out.push_str(&format!("{number}\\,")),
        }
    } else {
        i = start;
    }

    let mut letters = String::new();
    let flush = |letters: &mut String, out: &mut String| {
        if !letters.is_empty() {
            out.push_str(&format!("\\mathrm{{{letters}}}"));
            letters.clear();
        }
    };
    while i < chars.len() {
        let c = chars[i];
        if c.is_ascii_alphabetic() {
            letters.push(c);
            i += 1;
        } else if c.is_ascii_digit() {
            flush(&mut letters, &mut out);
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let digits: String = chars[start..i].iter().collect();
            out.push_str(&format!("_{{{digits}}}"));
        } else if c == '^' {
            flush(&mut letters, &mut out);
            i += 1;
            let charge: String = if chars.get(i) == Some(&'{') {
                let inner: String = chars[i + 1..].iter().collect();
                let len = braced(&inner).unwrap_or(inner.len());
                i += len + 2;
                inner[..len].to_string()
            } else {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_ascii_alphanumeric() || "+-".contains(chars[i]))
                {
                    i += 1;
                }
                chars[start..i].iter().collect()
            };
            out.push_str(&format!("^{{{charge}}}"));
        } else if (c == '+' || c == '-') && is_charge(&chars[i..]) {
            // A charge without `^`: `Na+`, `Cl-`, `O2--`, also before a
            // state: `Na+(aq)`.
            flush(&mut letters, &mut out);
            let charge: String = chars[i..].iter().take_while(|&&rest| rest == c).collect();
            out.push_str(&format!("^{{{charge}}}"));
            i += charge.len();
        } else if c == '(' {
            flush(&mut letters, &mut out);
            // `(aq)`, `(s)`: a state, upright and a little apart.
            let inner: String = chars[i + 1..].iter().take_while(|&&c| c != ')').collect();
            if !inner.is_empty()
                && inner.chars().all(|c| c.is_ascii_lowercase())
                && i + 1 + inner.len() < chars.len()
            {
                out.push_str(&format!("\\,\\mathrm{{({inner})}}"));
                i += inner.len() + 2;
            } else {
                out.push('(');
                i += 1;
            }
        } else if c == '[' {
            flush(&mut letters, &mut out);
            out.push_str("\\lbrack ");
            i += 1;
        } else if c == ']' {
            flush(&mut letters, &mut out);
            out.push_str("\\rbrack ");
            i += 1;
        } else {
            flush(&mut letters, &mut out);
            out.push(c);
            i += 1;
        }
    }
    flush(&mut letters, &mut out);
    out
}

/// Whether `rest` starts with a charge: `+` or `-` signs up to the end
/// of the formula or up to a state such as `(aq)`.
fn is_charge(rest: &[char]) -> bool {
    let sign = rest[0];
    let after: String = rest.iter().skip_while(|&&c| c == sign).collect();
    after.is_empty()
        || after
            .strip_prefix('(')
            .and_then(|state| state.strip_suffix(')'))
            .is_some_and(|state| !state.is_empty() && state.chars().all(|c| c.is_ascii_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_a_molecule_with_its_caption() {
        let html = molecule("CN1C=NC2=C1C(=O)N(C(=O)N2C)C\nCaffeine\n");
        assert!(html.starts_with("<figure class=\"molecule\"><svg role=\"img\" aria-label=\"Caffeine\" data-pagefind-ignore"), "{html}");
        assert!(html.contains("<line"), "{html}");
        assert!(html.contains(">N</text>"), "{html}");
        assert!(!html.contains("\"black\""), "{html}");
        assert!(
            html.ends_with("<figcaption>Caffeine</figcaption></figure>"),
            "{html}"
        );
    }

    #[test]
    fn shows_a_smiles_mistake_in_place() {
        let html = molecule("C1CC(");
        assert!(
            html.starts_with("<p><code class=\"chem-error\" title=\""),
            "{html}"
        );
        assert!(html.contains(">C1CC(</code>"), "{html}");
    }

    #[test]
    fn turns_ce_into_latex() {
        assert_eq!(expand_ce(r"\ce{H2O}"), r"{\mathrm{H}_{2}\mathrm{O}}");
        assert_eq!(
            expand_ce(r"\ce{2H2 + O2 -> 2H2O}"),
            r"{2\,\mathrm{H}_{2} + \mathrm{O}_{2} \longrightarrow 2\,\mathrm{H}_{2}\mathrm{O}}"
        );
        assert_eq!(expand_ce(r"\ce{SO4^2-}"), r"{\mathrm{SO}_{4}^{2-}}");
        assert_eq!(expand_ce(r"\ce{Na+}"), r"{\mathrm{Na}^{+}}");
        assert_eq!(
            expand_ce(r"\ce{Na+(aq) + Cl-(aq)}"),
            r"{\mathrm{Na}^{+}\,\mathrm{(aq)} + \mathrm{Cl}^{-}\,\mathrm{(aq)}}"
        );
        assert_eq!(
            expand_ce(r"\ce{C6H12O6}"),
            r"{\mathrm{C}_{6}\mathrm{H}_{12}\mathrm{O}_{6}}"
        );
        assert_eq!(
            expand_ce(r"\ce{NaCl(aq)}"),
            r"{\mathrm{NaCl}\,\mathrm{(aq)}}"
        );
        assert_eq!(
            expand_ce(r"\ce{Ca(OH)2}"),
            r"{\mathrm{Ca}(\mathrm{OH})_{2}}"
        );
        assert_eq!(
            expand_ce(r"\ce{CuSO4*5H2O}"),
            r"{\mathrm{CuSO}_{4} \cdot 5\,\mathrm{H}_{2}\mathrm{O}}"
        );
        assert_eq!(
            expand_ce(r"\ce{CaCO3 ->[\Delta] CaO + CO2 ^}"),
            r"{\mathrm{CaCO}_{3} \xrightarrow{\Delta} \mathrm{CaO} + \mathrm{CO}_{2} \uparrow}"
        );
        assert_eq!(
            expand_ce(r"\ce{A <=> B}"),
            r"{\mathrm{A} \rightleftharpoons \mathrm{B}}"
        );
        assert_eq!(expand_ce(r"x + \ce{H+}"), r"x + {\mathrm{H}^{+}}");
        assert_eq!(expand_ce(r"\ce{H2"), r"\ce{H2");
    }
}

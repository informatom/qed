//! Stratigraphic columns: a ```` ```strata ```` block lists the layers of
//! an outcrop or a borehole, youngest (top) first, one per line:
//!
//! ```text
//! 300 m | limestone | Dachstein limestone
//! 1200 m | dolomite | Hauptdolomit
//! ```
//!
//! q.e.d. draws them as SVG when the site is built: each layer as tall as
//! its thickness, filled with the usual pattern for its rock (bricks for
//! limestone, dots for sandstone, dashes for shale …), as wide as its
//! grain size (fine rocks narrow, coarse ones wide, as in sedimentary
//! logs), with a thickness scale, the layers' names and a legend.

use std::collections::hash_map::DefaultHasher;
use std::fmt::Write as _;
use std::hash::{Hash, Hasher};

/// A kind of rock: its names, its color, its pattern and how wide it's
/// drawn (0–1, by grain size).
struct Rock {
    names: &'static [&'static str],
    label: &'static str,
    fill: &'static str,
    pattern: Pattern,
    width: f64,
}

#[derive(Clone, Copy)]
enum Pattern {
    Bricks,
    SlantedBricks,
    Dots,
    Dashes,
    DashesAndDots,
    BricksAndDashes,
    Circles,
    Triangles,
    Solid,
    VerticalLines,
    Crosses,
    Vs,
    Waves,
    Plain,
}

const ROCKS: &[Rock] = &[
    Rock {
        names: &["limestone", "kalk", "kalkstein"],
        label: "Limestone",
        fill: "#b9d7ea",
        pattern: Pattern::Bricks,
        width: 0.85,
    },
    Rock {
        names: &["dolomite", "dolostone", "dolomit"],
        label: "Dolomite",
        fill: "#cbc5e8",
        pattern: Pattern::SlantedBricks,
        width: 0.9,
    },
    Rock {
        names: &["marl", "mergel"],
        label: "Marl",
        fill: "#cfe3c3",
        pattern: Pattern::BricksAndDashes,
        width: 0.65,
    },
    Rock {
        names: &["sandstone", "sandstein"],
        label: "Sandstone",
        fill: "#f5dc8c",
        pattern: Pattern::Dots,
        width: 0.8,
    },
    Rock {
        names: &["siltstone", "siltstein"],
        label: "Siltstone",
        fill: "#e8d9a8",
        pattern: Pattern::DashesAndDots,
        width: 0.65,
    },
    Rock {
        names: &["shale", "schieferton"],
        label: "Shale",
        fill: "#c8c8c8",
        pattern: Pattern::Dashes,
        width: 0.55,
    },
    Rock {
        names: &["claystone", "clay", "tonstein", "ton"],
        label: "Claystone",
        fill: "#d6c8b4",
        pattern: Pattern::Dashes,
        width: 0.5,
    },
    Rock {
        names: &["conglomerate", "konglomerat"],
        label: "Conglomerate",
        fill: "#f2b36b",
        pattern: Pattern::Circles,
        width: 1.0,
    },
    Rock {
        names: &["breccia", "brekzie"],
        label: "Breccia",
        fill: "#e9a27a",
        pattern: Pattern::Triangles,
        width: 1.0,
    },
    Rock {
        names: &["coal", "kohle"],
        label: "Coal",
        fill: "#333333",
        pattern: Pattern::Solid,
        width: 0.5,
    },
    Rock {
        names: &["gypsum", "evaporite", "gips", "salt", "salz"],
        label: "Evaporite",
        fill: "#f1e8f2",
        pattern: Pattern::VerticalLines,
        width: 0.6,
    },
    Rock {
        names: &["sand"],
        label: "Sand",
        fill: "#f9e8b0",
        pattern: Pattern::Dots,
        width: 0.8,
    },
    Rock {
        names: &["gravel", "kies", "schotter"],
        label: "Gravel",
        fill: "#f4c99a",
        pattern: Pattern::Circles,
        width: 1.0,
    },
    Rock {
        names: &["soil", "boden"],
        label: "Soil",
        fill: "#b89b74",
        pattern: Pattern::Plain,
        width: 0.7,
    },
    Rock {
        names: &["granite", "granit"],
        label: "Granite",
        fill: "#f2b8b8",
        pattern: Pattern::Crosses,
        width: 1.0,
    },
    Rock {
        names: &["basalt", "lava"],
        label: "Basalt",
        fill: "#9aa89a",
        pattern: Pattern::Vs,
        width: 1.0,
    },
    Rock {
        names: &["gneiss", "schist", "gneis", "schiefer"],
        label: "Gneiss, schist",
        fill: "#d7b9d9",
        pattern: Pattern::Waves,
        width: 1.0,
    },
];

/// One layer as written.
struct Layer {
    thickness: f64,
    rock: &'static Rock,
    name: String,
}

// The drawing's measures, in pixels.
const SCALE_X: f64 = 56.0;
const COLUMN_X: f64 = 64.0;
const COLUMN_WIDTH: f64 = 120.0;
const LABEL_X: f64 = COLUMN_X + COLUMN_WIDTH + 18.0;
const WIDTH: f64 = 560.0;
const TOP: f64 = 12.0;
const LINE: f64 = 16.0;

/// The HTML for a ```` ```strata ```` block: the column as SVG, or the
/// text in red with the problem on hover.
pub fn column(block: &str) -> String {
    match layers(block) {
        Ok((layers, unit)) => format!(
            "<figure class=\"strata\">{}</figure>",
            svg(&layers, &unit, block)
        ),
        Err(problem) => format!(
            "<pre class=\"chart-error\" title=\"{}\"><code>{}</code></pre>",
            attribute(&problem),
            text(block)
        ),
    }
}

/// The layers, and the unit of their thickness (`m` unless written).
fn layers(block: &str) -> Result<(Vec<Layer>, String), String> {
    let mut layers = Vec::new();
    let mut unit: Option<String> = None;
    for (number, line) in block.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split('|').map(str::trim);
        let thickness = fields.next().unwrap_or_default();
        let rock = fields.next().unwrap_or_default();
        let name = fields.collect::<Vec<_>>().join(" | ");
        let at = || format!("line {}", number + 1);

        let digits = thickness
            .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == ','))
            .unwrap_or(thickness.len());
        let value: f64 = thickness[..digits]
            .replace(',', ".")
            .parse()
            .map_err(|_| format!("{}: “{thickness}” isn't a thickness such as 12 m", at()))?;
        if value <= 0.0 {
            return Err(format!("{}: a layer needs a thickness above 0", at()));
        }
        let this_unit = thickness[digits..].trim();
        if !this_unit.is_empty() {
            match &unit {
                Some(unit) if unit != this_unit => {
                    return Err(format!(
                        "{}: “{this_unit}”, but the layers above are in {unit}",
                        at()
                    ));
                }
                _ => unit = Some(this_unit.to_string()),
            }
        }
        let wanted = rock.to_lowercase();
        let rock = ROCKS
            .iter()
            .find(|known| known.names.contains(&wanted.as_str()))
            .ok_or_else(|| {
                let known: Vec<&str> = ROCKS.iter().map(|rock| rock.names[0]).collect();
                format!("{}: no rock “{rock}”; known are {}", at(), known.join(", "))
            })?;
        layers.push(Layer {
            thickness: value,
            rock,
            name: name.to_string(),
        });
    }
    if layers.is_empty() {
        return Err("no layers; write one per line: 12 m | limestone | name".to_string());
    }
    Ok((layers, unit.unwrap_or_else(|| "m".to_string())))
}

fn svg(layers: &[Layer], unit: &str, source: &str) -> String {
    // Patterns need ids that differ between columns on one page.
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    let id = format!("strata-{:x}", hasher.finish() & 0xff_ffff);

    // Height: about 480 px in all, more if the thinnest layer would get
    // less than a line of text, but never more than 1000 px.
    let total: f64 = layers.iter().map(|layer| layer.thickness).sum();
    let thinnest = layers.iter().map(|l| l.thickness).fold(f64::MAX, f64::min);
    let per_unit = (480.0 / total).max(LINE / thinnest).min(1000.0 / total);
    let column_height = total * per_unit;

    let mut out = String::new();
    let mut y = TOP;
    let mut last_label: Option<f64> = None;
    let mut labels = String::new();
    let mut used: Vec<&Rock> = Vec::new();
    for layer in layers {
        let height = layer.thickness * per_unit;
        let width = COLUMN_WIDTH * layer.rock.width;
        let index = rock_index(layer.rock);
        let _ = write!(
            out,
            "<rect x=\"{COLUMN_X}\" y=\"{y:.1}\" width=\"{width:.1}\" height=\"{height:.1}\" fill=\"{}\"/>",
            layer.rock.fill
        );
        if !matches!(layer.rock.pattern, Pattern::Solid | Pattern::Plain) {
            let _ = write!(
                out,
                "<rect x=\"{COLUMN_X}\" y=\"{y:.1}\" width=\"{width:.1}\" height=\"{height:.1}\" fill=\"url(#{id}-{index})\"/>"
            );
        }
        let _ = write!(
            out,
            "<rect x=\"{COLUMN_X}\" y=\"{y:.1}\" width=\"{width:.1}\" height=\"{height:.1}\" fill=\"none\" stroke=\"#222\" stroke-width=\"1\"/>"
        );

        // The name beside the layer; pushed down below the one before if
        // the layer is thin, with a line to show where it belongs.
        let middle = y + height / 2.0;
        let text_y = match last_label {
            None => middle,
            Some(last) => middle.max(last + LINE),
        };
        last_label = Some(text_y);
        let name = if layer.name.is_empty() {
            format!("{} {unit}", number(layer.thickness))
        } else {
            format!("{} ({} {unit})", layer.name, number(layer.thickness))
        };
        let _ = write!(
            labels,
            "<line x1=\"{:.1}\" y1=\"{middle:.1}\" x2=\"{:.1}\" y2=\"{text_y:.1}\" stroke=\"currentColor\" stroke-opacity=\"0.5\"/>\
             <text x=\"{LABEL_X}\" y=\"{text_y:.1}\" dominant-baseline=\"middle\" font-size=\"13\" fill=\"currentColor\">{}</text>",
            COLUMN_X + width + 2.0,
            LABEL_X - 4.0,
            text(&name)
        );
        if !used.iter().any(|rock| std::ptr::eq(*rock, layer.rock)) {
            used.push(layer.rock);
        }
        y += height;
    }

    // The thickness scale, counted from the base up, as geologists do.
    let step = nice_step(total);
    let mut scale = format!(
        "<line x1=\"{SCALE_X}\" y1=\"{TOP}\" x2=\"{SCALE_X}\" y2=\"{:.1}\" stroke=\"currentColor\"/>",
        TOP + column_height
    );
    let mut tick = 0.0;
    while tick <= total + step * 0.01 {
        let tick_y = TOP + column_height - tick * per_unit;
        let _ = write!(
            scale,
            "<line x1=\"{:.1}\" y1=\"{tick_y:.1}\" x2=\"{SCALE_X}\" y2=\"{tick_y:.1}\" stroke=\"currentColor\"/>\
             <text x=\"{:.1}\" y=\"{tick_y:.1}\" dominant-baseline=\"middle\" text-anchor=\"end\" font-size=\"11\" fill=\"currentColor\">{}</text>",
            SCALE_X - 4.0,
            SCALE_X - 7.0,
            if tick == 0.0 {
                format!("0 {unit}")
            } else {
                number(tick)
            }
        );
        tick += step;
    }

    // The legend: one swatch per rock that appears, in rows.
    let lowest_label = last_label.unwrap_or(TOP);
    let legend_top = (TOP + column_height).max(lowest_label + LINE / 2.0) + 22.0;
    let mut legend = String::new();
    let (mut x, mut row) = (COLUMN_X, 0.0);
    for rock in &used {
        let label_width = rock.label.chars().count() as f64 * 7.0 + 40.0;
        if x + label_width > WIDTH {
            x = COLUMN_X;
            row += 22.0;
        }
        let swatch_y = legend_top + row;
        let index = rock_index(rock);
        let _ = write!(
            legend,
            "<rect x=\"{x:.1}\" y=\"{swatch_y:.1}\" width=\"22\" height=\"14\" fill=\"{}\"/>",
            rock.fill
        );
        if !matches!(rock.pattern, Pattern::Solid | Pattern::Plain) {
            let _ = write!(
                legend,
                "<rect x=\"{x:.1}\" y=\"{swatch_y:.1}\" width=\"22\" height=\"14\" fill=\"url(#{id}-{index})\"/>"
            );
        }
        let _ = write!(
            legend,
            "<rect x=\"{x:.1}\" y=\"{swatch_y:.1}\" width=\"22\" height=\"14\" fill=\"none\" stroke=\"#222\"/>\
             <text x=\"{:.1}\" y=\"{:.1}\" dominant-baseline=\"middle\" font-size=\"12\" fill=\"currentColor\">{}</text>",
            x + 28.0,
            swatch_y + 7.0,
            rock.label
        );
        x += label_width;
    }
    let height = legend_top + row + 26.0;

    let mut patterns = String::new();
    for rock in used
        .iter()
        .filter(|rock| !matches!(rock.pattern, Pattern::Solid | Pattern::Plain))
    {
        patterns.push_str(&pattern(
            &format!("{id}-{}", rock_index(rock)),
            rock.pattern,
        ));
    }
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {WIDTH} {height:.0}\" width=\"{WIDTH}\" height=\"{height:.0}\" \
         role=\"img\" aria-label=\"Stratigraphic column\"><defs>{patterns}</defs>{out}{labels}{scale}{legend}</svg>"
    )
}

fn rock_index(rock: &Rock) -> usize {
    ROCKS
        .iter()
        .position(|known| std::ptr::eq(known, rock))
        .unwrap_or_default()
}

/// The SVG pattern that fills a layer of this rock.
fn pattern(id: &str, pattern: Pattern) -> String {
    let (size, shapes) = match pattern {
        Pattern::Bricks => ((24, 12), "<path d=\"M0 0.5H24M0 6.5H24M6 0V6M18 6V12\"/>"),
        Pattern::SlantedBricks => (
            (24, 12),
            "<path d=\"M0 0.5H24M0 6.5H24M4 0L8 6M16 6L20 12\"/>",
        ),
        Pattern::BricksAndDashes => ((24, 12), "<path d=\"M0 0.5H24M6 0V6M3 9H9M15 9H21\"/>"),
        Pattern::Dots => (
            (10, 10),
            "<circle cx=\"2.5\" cy=\"2.5\" r=\"0.9\" fill=\"#222\"/><circle cx=\"7.5\" cy=\"7.5\" r=\"0.9\" fill=\"#222\"/>",
        ),
        Pattern::DashesAndDots => (
            (16, 10),
            "<path d=\"M1 2.5H7\"/><circle cx=\"12\" cy=\"2.5\" r=\"0.8\" fill=\"#222\"/><path d=\"M9 7.5H15\"/><circle cx=\"4\" cy=\"7.5\" r=\"0.8\" fill=\"#222\"/>",
        ),
        Pattern::Dashes => ((16, 8), "<path d=\"M1 2H9M9 6H15\"/>"),
        Pattern::Circles => (
            (16, 16),
            "<circle cx=\"4\" cy=\"4\" r=\"2.6\" fill=\"none\"/><circle cx=\"12\" cy=\"11\" r=\"3.2\" fill=\"none\"/>",
        ),
        Pattern::Triangles => (
            (16, 16),
            "<path d=\"M2 7L5 2L8 7ZM9 14L12 9L15 14Z\" fill=\"none\"/>",
        ),
        Pattern::VerticalLines => ((8, 8), "<path d=\"M2 0V8M6 0V8\"/>"),
        Pattern::Crosses => ((16, 16), "<path d=\"M4 2V6M2 4H6M12 10V14M10 12H14\"/>"),
        Pattern::Vs => (
            (16, 14),
            "<path d=\"M2 2L4 6L6 2M10 9L12 13L14 9\" fill=\"none\"/>",
        ),
        Pattern::Waves => ((20, 8), "<path d=\"M0 4Q5 0 10 4T20 4\" fill=\"none\"/>"),
        Pattern::Solid | Pattern::Plain => ((1, 1), ""),
    };
    format!(
        "<pattern id=\"{id}\" width=\"{}\" height=\"{}\" patternUnits=\"userSpaceOnUse\">\
         <g stroke=\"#222\" stroke-width=\"0.8\">{shapes}</g></pattern>",
        size.0, size.1
    )
}

/// A step for the scale: 1, 2 or 5 times a power of ten, for 4–10 ticks.
fn nice_step(total: f64) -> f64 {
    let rough = total / 6.0;
    let power = 10f64.powf(rough.log10().floor());
    [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|factor| factor * power)
        .find(|step| total / step <= 8.0)
        .unwrap_or(10.0 * power)
}

/// A number without needless decimals: 12, 2.5.
fn number(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if rounded.fract() == 0.0 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded}")
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_layers_scale_and_legend() {
        let html = column(
            "# youngest first\n300 m | limestone | Dachstein limestone\n1200 m | Dolomit | Hauptdolomit\n2 m | coal\n",
        );
        assert!(html.starts_with("<figure class=\"strata\"><svg "), "{html}");
        assert!(
            html.contains(">Dachstein limestone (300 m)</text>"),
            "{html}"
        );
        assert!(
            html.contains(">2 m</text>"),
            "an unnamed layer shows its thickness: {html}"
        );
        assert!(
            html.contains(">Limestone</text>") && html.contains(">Dolomite</text>"),
            "{html}"
        );
        assert!(html.contains(">0 m</text>"), "{html}");
        // Patterns for limestone and dolomite, none for coal.
        assert_eq!(html.matches("<pattern ").count(), 2, "{html}");
    }

    #[test]
    fn explains_mistakes() {
        let html = column("12 m | lava rock");
        assert!(html.contains("line 1: no rock “lava rock”"), "{html}");
        let html = column("12 m | shale\n3 ft | sandstone");
        assert!(
            html.contains("line 2: “ft”, but the layers above are in m"),
            "{html}"
        );
        let html = column("thick | shale");
        assert!(
            html.contains("isn&#39;t a thickness") || html.contains("isn't a thickness"),
            "{html}"
        );
        assert!(column("").contains("no layers"));
    }

    #[test]
    fn picks_round_scale_steps() {
        assert_eq!(nice_step(2350.0), 500.0);
        assert_eq!(nice_step(48.0), 10.0);
        assert_eq!(nice_step(7.5), 1.0);
    }
}

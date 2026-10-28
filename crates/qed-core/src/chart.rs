//! Charts in text: a ```` ```chart ```` block holds a chart described in
//! JSON, which [charts-rs](https://github.com/vicanso/charts-rs) draws
//! as SVG when the site is built. No JavaScript, nothing from elsewhere.
//!
//! The JSON is charts-rs's own (much like ECharts' options), plus
//! `"type"`: `bar` (the default), `line`, `pie`, `scatter`, and the
//! others in [`svg`]. The site's stylesheet gives the charts the theme's
//! font and colors (`.chart` in `site.input.css`).

use charts_rs::{
    BarChart, BoxPlotChart, CandlestickChart, FunnelChart, GaugeChart, HeatmapChart,
    HistogramChart, HorizontalBarChart, LineChart, PieChart, PolarBarChart, RadarChart,
    SankeyChart, ScatterChart, TableChart, TreemapChart, WaterfallChart,
};
use serde_json::Value;

/// The HTML for a ```` ```chart ```` block: the chart as SVG, or, if the
/// JSON has a mistake, the JSON in red with the problem on hover.
pub fn chart(json: &str) -> String {
    let mut value: Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(problem) => return error(json, &format!("not valid JSON: {problem}")),
    };
    let Some(fields) = value.as_object_mut() else {
        return error(json, "a chart is a JSON object: { … }");
    };
    let kind = match fields.remove("type") {
        None => "bar".to_string(),
        Some(Value::String(kind)) => kind,
        Some(_) => return error(json, "\"type\" is a name such as \"bar\" or \"line\""),
    };
    // A plain pie, not charts-rs's default "rose" (whose slices also
    // differ in length, which reads as a second value).
    if kind == "pie" {
        fields.entry("rose_type").or_insert(Value::Bool(false));
    }
    let label = fields
        .get("title_text")
        .and_then(Value::as_str)
        .unwrap_or("Chart")
        .to_string();
    match svg(&kind, &value.to_string()) {
        Ok(svg) => {
            let svg = svg.replacen(
                "<svg ",
                &format!("<svg role=\"img\" aria-label=\"{}\" ", attribute(&label)),
                1,
            );
            format!("<figure class=\"chart\">{svg}</figure>")
        }
        Err(problem) => error(json, &problem),
    }
}

/// Draws a chart of `kind` from charts-rs's JSON.
fn svg(kind: &str, json: &str) -> Result<String, String> {
    macro_rules! draw {
        ($chart:ty) => {
            <$chart>::from_json(json)
                .and_then(|chart| chart.svg())
                .map_err(|problem| problem.to_string())
        };
    }
    match kind {
        "bar" => draw!(BarChart),
        "horizontal_bar" => draw!(HorizontalBarChart),
        "line" => draw!(LineChart),
        "pie" => draw!(PieChart),
        "scatter" => draw!(ScatterChart),
        "radar" => draw!(RadarChart),
        "table" => draw!(TableChart),
        "heatmap" => draw!(HeatmapChart),
        "histogram" => draw!(HistogramChart),
        "box_plot" => draw!(BoxPlotChart),
        "candlestick" => draw!(CandlestickChart),
        "funnel" => draw!(FunnelChart),
        "waterfall" => draw!(WaterfallChart),
        "gauge" => draw!(GaugeChart),
        "sankey" => draw!(SankeyChart),
        "treemap" => draw!(TreemapChart),
        "polar_bar" => draw!(PolarBarChart),
        other => Err(format!(
            "no chart type \"{other}\"; try bar, horizontal_bar, line, pie, scatter, radar, \
             table, heatmap, histogram, box_plot, candlestick, funnel, waterfall, gauge, \
             sankey, treemap or polar_bar"
        )),
    }
}

fn error(source: &str, problem: &str) -> String {
    format!(
        "<pre class=\"chart-error\" title=\"{}\"><code>{}</code></pre>",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_a_bar_chart_by_default() {
        let html = chart(
            r#"{"title_text": "Downloads", "x_axis_data": ["Ep. 1", "Ep. 2"],
                "series_list": [{"name": "2026", "data": [820, 932]}]}"#,
        );
        assert!(
            html.starts_with("<figure class=\"chart\"><svg role=\"img\" aria-label=\"Downloads\" "),
            "{html}"
        );
        assert!(html.contains("<rect"), "{html}");
        assert!(html.ends_with("</svg></figure>"), "{html}");
    }

    #[test]
    fn draws_scatter_and_plain_pie() {
        let scatter = chart(
            r#"{"type": "scatter", "series_list": [{"name": "MgO", "data": [48.5, 8.9, 75.1, 0.3]}]}"#,
        );
        assert!(scatter.contains("<circle"), "{scatter}");
        let pie = chart(
            r#"{"type": "pie", "series_list": [{"name": "A", "data": [40]}, {"name": "B", "data": [60]}]}"#,
        );
        assert!(pie.contains("<path"), "{pie}");
    }

    #[test]
    fn shows_mistakes_in_place() {
        let html = chart("{\"title_text\": ");
        assert!(
            html.starts_with("<pre class=\"chart-error\" title=\"not valid JSON"),
            "{html}"
        );
        let html = chart(r#"{"type": "donut"}"#);
        assert!(html.contains("no chart type"), "{html}");
        // A misspelt field: charts-rs checks the names.
        let html = chart(r#"{"titel_text": "x", "series_list": []}"#);
        assert!(html.starts_with("<pre class=\"chart-error\""), "{html}");
    }
}

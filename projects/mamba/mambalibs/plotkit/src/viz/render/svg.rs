//! Pure SVG renderer for charts — core struct, primitives, and dispatch.

use crate::viz::annotation::Annotation;
use crate::viz::axis::{format_tick, TickInfo};
use crate::viz::series_data::DataSeries;
use crate::viz::style::{ChartStyle, Color};

/// SVG renderer that builds an SVG string from chart elements.
pub struct SvgRenderer {
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) margin: Margin,
    pub(crate) elements: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Margin {
    pub(crate) top: f64,
    pub(crate) right: f64,
    pub(crate) bottom: f64,
    pub(crate) left: f64,
}

impl Default for Margin {
    fn default() -> Self {
        Self {
            top: 40.0,
            right: 20.0,
            bottom: 50.0,
            left: 60.0,
        }
    }
}

impl SvgRenderer {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            margin: Margin::default(),
            elements: Vec::new(),
        }
    }

    pub(crate) fn plot_width(&self) -> f64 {
        self.width - self.margin.left - self.margin.right
    }

    pub(crate) fn plot_height(&self) -> f64 {
        self.height - self.margin.top - self.margin.bottom
    }

    /// Render a complete chart to SVG string.
    pub fn render(
        &mut self,
        title: &Option<String>,
        series: &[DataSeries],
        style: &ChartStyle,
    ) -> String {
        self.elements.clear();

        // Background
        self.rect(
            0.0,
            0.0,
            self.width,
            self.height,
            &style.background.to_hex(),
            1.0,
        );

        // Title
        if let Some(t) = title {
            self.text(
                self.width / 2.0,
                self.margin.top / 2.0 + 4.0,
                t,
                &style.text_color.to_hex(),
                style.title_size,
                "middle",
            );
        }

        // Render each series
        for s in series {
            match s {
                DataSeries::Line {
                    x, y, style: ls, ..
                } => {
                    self.render_line(x, y, &ls.color, ls.width, style);
                }
                DataSeries::Bar {
                    labels,
                    values,
                    style: bs,
                    ..
                } => {
                    self.render_bar(labels, values, &bs.color, bs.opacity, style);
                }
                DataSeries::Scatter {
                    x, y, style: ps, ..
                } => {
                    self.render_scatter(x, y, &ps.color, ps.radius, ps.opacity, style);
                }
                DataSeries::Histogram {
                    data,
                    bins,
                    style: bs,
                    ..
                } => {
                    self.render_histogram(data, *bins, &bs.color, bs.opacity, style);
                }
                DataSeries::BoxPlot {
                    data,
                    labels,
                    style: bs,
                } => {
                    self.render_box_plot(data, labels, &bs.color, style);
                }
                DataSeries::Heatmap {
                    data,
                    x_labels,
                    y_labels,
                } => {
                    self.render_heatmap(data, x_labels, y_labels, style);
                }
                DataSeries::Area {
                    x, y, style: ls, ..
                } => {
                    self.render_area(x, y, &ls.color, ls.width, style);
                }
                DataSeries::Pie { labels, values } => {
                    self.render_pie(labels, values, style);
                }
                DataSeries::StackedBar { labels, datasets } => {
                    self.render_stacked_bar(labels, datasets, style);
                }
                DataSeries::Violin { data, labels } => {
                    self.render_violin(data, labels, style);
                }
                DataSeries::Polar {
                    axis_labels,
                    datasets,
                } => {
                    self.render_polar(axis_labels, datasets, style);
                }
                DataSeries::Donut {
                    labels,
                    values,
                    hole_ratio,
                } => {
                    self.render_donut(labels, values, *hole_ratio, style);
                }
                DataSeries::Surface3D { x, y, z } => {
                    self.render_surface_3d(x, y, z, style);
                }
            }
        }

        // Legend (for multi-series charts)
        let palette = crate::viz::style::default_palette();
        self.render_legend(series, style, &palette);

        self.build_svg()
    }

    /// Render a complete chart with annotations, axis labels, etc.
    pub fn render_full(
        &mut self,
        title: &Option<String>,
        x_label: &Option<String>,
        y_label: &Option<String>,
        series: &[DataSeries],
        style: &ChartStyle,
        annotations: &[Annotation],
    ) -> String {
        // Use the standard render pipeline first
        let svg = self.render(title, series, style);

        if annotations.is_empty() && x_label.is_none() && y_label.is_none() {
            return svg;
        }

        // Re-render with annotations and axis labels
        // We need to re-build because render() already called build_svg()
        // Remove the closing </svg> tag, append annotations, then close
        let base = svg.trim_end_matches("</svg>");
        let mut result = base.to_string();

        // Axis labels
        if let Some(xl) = x_label {
            result.push_str(&format!(
                r#"  <text x="{:.1}" y="{:.1}" fill="{}" font-size="{:.1}" text-anchor="middle" font-family="sans-serif">{}</text>
"#,
                self.margin.left + self.plot_width() / 2.0,
                self.height - 5.0,
                style.text_color.to_hex(),
                style.font_size,
                escape_xml(xl)
            ));
        }

        if let Some(yl) = y_label {
            let yx = 15.0;
            let yy = self.margin.top + self.plot_height() / 2.0;
            result.push_str(&format!(
                r#"  <text x="{:.1}" y="{:.1}" fill="{}" font-size="{:.1}" text-anchor="middle" font-family="sans-serif" transform="rotate(-90,{:.1},{:.1})">{}</text>
"#,
                yx, yy,
                style.text_color.to_hex(),
                style.font_size,
                yx, yy,
                escape_xml(yl)
            ));
        }

        // Render annotations (using approximate tick info from first XY series)
        if !annotations.is_empty() {
            let (x_info, y_info) = infer_tick_info(series);
            self.elements.clear();
            self.render_annotations(annotations, x_info.as_ref(), y_info.as_ref(), style);
            for elem in &self.elements {
                result.push_str("  ");
                result.push_str(elem);
                result.push('\n');
            }
        }

        result.push_str("</svg>");
        result
    }

    pub(crate) fn draw_axes(
        &mut self,
        x_tick_vals: &[f64],
        y_tick_vals: &[f64],
        x_info: &TickInfo,
        y_info: &TickInfo,
        style: &ChartStyle,
    ) {
        let color = &style.text_color.to_hex();
        let grid_color = &style.grid_color.to_hex();

        // X-axis
        self.line(
            self.margin.left,
            self.margin.top + self.plot_height(),
            self.margin.left + self.plot_width(),
            self.margin.top + self.plot_height(),
            color,
            1.0,
        );
        // Y-axis
        self.line(
            self.margin.left,
            self.margin.top,
            self.margin.left,
            self.margin.top + self.plot_height(),
            color,
            1.0,
        );

        // X ticks
        for &tick in x_tick_vals {
            let px = self.map_x(tick, x_info.min, x_info.max);
            if style.show_grid {
                self.line(
                    px,
                    self.margin.top,
                    px,
                    self.margin.top + self.plot_height(),
                    grid_color,
                    0.5,
                );
            }
            self.text(
                px,
                self.margin.top + self.plot_height() + 15.0,
                &format_tick(tick),
                color,
                style.font_size,
                "middle",
            );
        }

        // Y ticks
        for &tick in y_tick_vals {
            let py = self.map_y(tick, y_info.min, y_info.max);
            if style.show_grid {
                self.line(
                    self.margin.left,
                    py,
                    self.margin.left + self.plot_width(),
                    py,
                    grid_color,
                    0.5,
                );
            }
            self.text(
                self.margin.left - 5.0,
                py + 4.0,
                &format_tick(tick),
                color,
                style.font_size,
                "end",
            );
        }
    }

    pub(crate) fn map_x(&self, val: f64, min: f64, max: f64) -> f64 {
        let range = max - min;
        if range.abs() < 1e-15 {
            return self.margin.left + self.plot_width() / 2.0;
        }
        self.margin.left + (val - min) / range * self.plot_width()
    }

    pub(crate) fn map_y(&self, val: f64, min: f64, max: f64) -> f64 {
        let range = max - min;
        if range.abs() < 1e-15 {
            return self.margin.top + self.plot_height() / 2.0;
        }
        self.margin.top + self.plot_height() - (val - min) / range * self.plot_height()
    }

    // SVG primitives

    pub(crate) fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, fill: &str, opacity: f64) {
        self.elements.push(format!(
            r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{}" opacity="{:.2}" />"#,
            x, y, w, h, fill, opacity
        ));
    }

    pub(crate) fn circle(&mut self, cx: f64, cy: f64, r: f64, fill: &str, opacity: f64) {
        self.elements.push(format!(
            r#"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="{}" opacity="{:.2}" />"#,
            cx, cy, r, fill, opacity
        ));
    }

    pub(crate) fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, stroke: &str, width: f64) {
        self.elements.push(format!(
            r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{}" stroke-width="{}" />"#,
            x1, y1, x2, y2, stroke, width
        ));
    }

    pub(crate) fn text(
        &mut self,
        x: f64,
        y: f64,
        content: &str,
        fill: &str,
        size: f64,
        anchor: &str,
    ) {
        self.elements.push(format!(
            r#"<text x="{:.1}" y="{:.1}" fill="{}" font-size="{:.1}" text-anchor="{}" font-family="sans-serif">{}</text>"#,
            x, y, fill, size, anchor, escape_xml(content)
        ));
    }

    pub(crate) fn build_svg(&self) -> String {
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            self.width, self.height, self.width, self.height
        );
        svg.push('\n');
        for elem in &self.elements {
            svg.push_str("  ");
            svg.push_str(elem);
            svg.push('\n');
        }
        svg.push_str("</svg>");
        svg
    }
}

// ── Free functions used by other submodules ────────────────────────────

/// Try to infer X and Y tick info from the first XY-type series (for annotation mapping).
fn infer_tick_info(series: &[DataSeries]) -> (Option<TickInfo>, Option<TickInfo>) {
    use crate::viz::axis::nice_ticks;

    for s in series {
        match s {
            DataSeries::Line { x, y, .. }
            | DataSeries::Scatter { x, y, .. }
            | DataSeries::Area { x, y, .. } => {
                if !x.is_empty() && !y.is_empty() {
                    let x_info = nice_ticks(
                        x.iter().copied().fold(f64::INFINITY, f64::min),
                        x.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                        5,
                    );
                    let y_info = nice_ticks(
                        y.iter().copied().fold(f64::INFINITY, f64::min),
                        y.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                        5,
                    );
                    return (Some(x_info), Some(y_info));
                }
            }
            _ => {}
        }
    }
    (None, None)
}

pub(crate) fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Viridis-like colormap approximation.
pub(crate) fn viridis(t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    let r = (68.0 + t * (255.0 - 68.0) * t).min(255.0) as u8;
    let g = (1.0 + t * 220.0 + (1.0 - t) * t * 100.0).min(255.0) as u8;
    let b = (84.0 + (1.0 - t) * 150.0).min(255.0) as u8;
    Color::new(r, g, b)
}

/// Box plot statistics.
pub(crate) struct BoxStats {
    pub(crate) median: f64,
    pub(crate) q1: f64,
    pub(crate) q3: f64,
    pub(crate) whisker_low: f64,
    pub(crate) whisker_high: f64,
}

pub(crate) fn box_stats(data: &[f64]) -> BoxStats {
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();

    let median = percentile_sorted(&sorted, 0.5);
    let q1 = percentile_sorted(&sorted, 0.25);
    let q3 = percentile_sorted(&sorted, 0.75);
    let iqr = q3 - q1;

    let whisker_low = sorted
        .iter()
        .copied()
        .find(|&v| v >= q1 - 1.5 * iqr)
        .unwrap_or(sorted[0]);
    let whisker_high = sorted
        .iter()
        .rev()
        .copied()
        .find(|&v| v <= q3 + 1.5 * iqr)
        .unwrap_or(sorted[n - 1]);

    BoxStats {
        median,
        q1,
        q3,
        whisker_low,
        whisker_high,
    }
}

pub(crate) fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return 0.0;
    }
    if n == 1 {
        return sorted[0];
    }
    let idx = p * (n - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = (lo + 1).min(n - 1);
    let frac = idx - lo as f64;
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml_happy() {
        assert_eq!(escape_xml("hello world"), "hello world");
    }

    #[test]
    fn test_escape_xml_ampersand() {
        assert_eq!(escape_xml("A & B"), "A &amp; B");
    }

    #[test]
    fn test_escape_xml_angle_brackets() {
        assert_eq!(escape_xml("x < y > z"), "x &lt; y &gt; z");
    }

    #[test]
    fn test_escape_xml_quotes() {
        assert_eq!(escape_xml(r#"say "hi""#), "say &quot;hi&quot;");
    }

    #[test]
    fn test_escape_xml_all_special() {
        assert_eq!(escape_xml(r#"<a & "b">"#), "&lt;a &amp; &quot;b&quot;&gt;");
    }

    #[test]
    fn test_escape_xml_empty() {
        assert_eq!(escape_xml(""), "");
    }

    #[test]
    fn test_viridis_zero() {
        let c = viridis(0.0);
        assert_eq!(c.r, 68);
    }

    #[test]
    fn test_viridis_one() {
        let c = viridis(1.0);
        assert!(c.r > 200);
        assert!(c.g > 200);
    }

    #[test]
    fn test_viridis_clamp() {
        assert_eq!(viridis(-1.0).r, viridis(0.0).r);
        assert_eq!(viridis(2.0).r, viridis(1.0).r);
    }

    #[test]
    fn test_percentile_sorted_empty() {
        assert_eq!(percentile_sorted(&[], 0.5), 0.0);
    }

    #[test]
    fn test_percentile_sorted_single() {
        assert_eq!(percentile_sorted(&[42.0], 0.5), 42.0);
    }

    #[test]
    fn test_percentile_sorted_median() {
        assert_eq!(percentile_sorted(&[1.0, 2.0, 3.0], 0.5), 2.0);
    }

    #[test]
    fn test_box_stats_happy() {
        let s = box_stats(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
        assert!((s.median - 5.5).abs() < 1e-10);
        assert!(s.q1 < s.median);
        assert!(s.q3 > s.median);
    }

    #[test]
    fn test_box_stats_outlier() {
        let s = box_stats(&[1.0, 2.0, 3.0, 4.0, 5.0, 100.0]);
        assert!(s.whisker_high < 100.0);
    }

    #[test]
    fn test_map_x_happy() {
        let r = SvgRenderer::new(800.0, 600.0);
        let left = r.map_x(0.0, 0.0, 10.0);
        let right = r.map_x(10.0, 0.0, 10.0);
        assert!((left - 60.0).abs() < 1e-10);
        assert!((right - 780.0).abs() < 1e-10);
    }

    #[test]
    fn test_map_x_zero_range() {
        let r = SvgRenderer::new(800.0, 600.0);
        let px = r.map_x(5.0, 5.0, 5.0);
        assert!((px - (60.0 + 720.0 / 2.0)).abs() < 1e-10);
    }

    #[test]
    fn test_map_y_happy() {
        let r = SvgRenderer::new(800.0, 600.0);
        let top = r.map_y(10.0, 0.0, 10.0);
        let bottom = r.map_y(0.0, 0.0, 10.0);
        assert!((top - 40.0).abs() < 1e-10);
        assert!((bottom - 550.0).abs() < 1e-10);
    }

    #[test]
    fn test_plot_dimensions() {
        let r = SvgRenderer::new(800.0, 600.0);
        assert!((r.plot_width() - 720.0).abs() < 1e-10);
        assert!((r.plot_height() - 510.0).abs() < 1e-10);
    }

    #[test]
    fn test_rect_element() {
        let mut r = SvgRenderer::new(400.0, 300.0);
        r.rect(10.0, 20.0, 100.0, 50.0, "#ff0000", 0.8);
        assert!(r.elements[0].contains("<rect"));
        assert!(r.elements[0].contains("fill=\"#ff0000\""));
    }

    #[test]
    fn test_circle_element() {
        let mut r = SvgRenderer::new(400.0, 300.0);
        r.circle(50.0, 60.0, 5.0, "#00ff00", 1.0);
        assert!(r.elements[0].contains("<circle"));
    }

    #[test]
    fn test_line_element() {
        let mut r = SvgRenderer::new(400.0, 300.0);
        r.line(0.0, 0.0, 100.0, 100.0, "#000000", 2.0);
        assert!(r.elements[0].contains("<line"));
    }

    #[test]
    fn test_text_escapes_content() {
        let mut r = SvgRenderer::new(400.0, 300.0);
        r.text(10.0, 20.0, "A < B & C", "#000", 12.0, "middle");
        assert!(r.elements[0].contains("A &lt; B &amp; C"));
    }

    #[test]
    fn test_build_svg_structure() {
        let r = SvgRenderer::new(400.0, 300.0);
        let svg = r.build_svg();
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("width=\"400\""));
    }

    #[test]
    fn test_render_clears_elements() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        r.rect(0.0, 0.0, 1.0, 1.0, "#000", 1.0);
        let series = vec![DataSeries::line(vec![1.0, 2.0], vec![3.0, 4.0])];
        r.render(&Some("Title".into()), &series, &style);
        assert!(r.elements.len() > 1);
    }

    #[test]
    fn test_render_with_title() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        let series = vec![DataSeries::line(vec![1.0, 2.0], vec![3.0, 4.0])];
        let svg = r.render(&Some("My Chart".into()), &series, &style);
        assert!(svg.contains("My Chart"));
    }
}

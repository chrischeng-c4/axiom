//! Subplot grid — arrange multiple charts in a grid layout.
//!
//! Each cell can hold an optional `Chart` which is rendered as a sub-SVG
//! translated to the appropriate grid position.

use super::chart::Chart;
use super::render::SvgRenderer;

/// A grid of subplots, each cell holding an optional `Chart`.
#[derive(Debug, Clone)]
pub struct SubplotGrid {
    pub rows: usize,
    pub cols: usize,
    pub charts: Vec<Option<Chart>>,
    pub width: f64,
    pub height: f64,
}

impl SubplotGrid {
    /// Create a new subplot grid with the given dimensions.
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            charts: vec![None; rows * cols],
            width: 1000.0,
            height: 800.0,
        }
    }

    /// Set the overall SVG dimensions.
    pub fn size(mut self, width: f64, height: f64) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Place a chart at a specific grid position (0-indexed).
    ///
    /// Panics if row or col is out of bounds.
    pub fn set(&mut self, row: usize, col: usize, chart: Chart) {
        assert!(
            row < self.rows,
            "row {} out of bounds (max {})",
            row,
            self.rows - 1
        );
        assert!(
            col < self.cols,
            "col {} out of bounds (max {})",
            col,
            self.cols - 1
        );
        self.charts[row * self.cols + col] = Some(chart);
    }

    /// Render the entire subplot grid to a single SVG string.
    pub fn to_svg(&self) -> String {
        let cell_w = self.width / self.cols as f64;
        let cell_h = self.height / self.rows as f64;
        let padding = 10.0;

        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            self.width, self.height, self.width, self.height
        );
        svg.push('\n');

        // White background
        svg.push_str(&format!(
            r##"  <rect x="0" y="0" width="{}" height="{}" fill="#ffffff" />"##,
            self.width, self.height
        ));
        svg.push('\n');

        for row in 0..self.rows {
            for col in 0..self.cols {
                if let Some(chart) = &self.charts[row * self.cols + col] {
                    let tx = col as f64 * cell_w + padding;
                    let ty = row as f64 * cell_h + padding;
                    let inner_w = cell_w - 2.0 * padding;
                    let inner_h = cell_h - 2.0 * padding;

                    // Render sub-chart at cell size
                    let sub_chart = Chart {
                        title: chart.title.clone(),
                        x_label: chart.x_label.clone(),
                        y_label: chart.y_label.clone(),
                        width: inner_w,
                        height: inner_h,
                        series: chart.series.clone(),
                        style: chart.style.clone(),
                        annotations: chart.annotations.clone(),
                    };

                    let mut renderer = SvgRenderer::new(inner_w, inner_h);
                    let inner_svg =
                        renderer.render(&sub_chart.title, &sub_chart.series, &sub_chart.style);

                    // Strip outer <svg> and </svg> tags to embed as a group
                    let inner_content = strip_svg_wrapper(&inner_svg);

                    svg.push_str(&format!(
                        r#"  <g transform="translate({:.1},{:.1})">"#,
                        tx, ty
                    ));
                    svg.push('\n');
                    svg.push_str(&inner_content);
                    svg.push('\n');
                    svg.push_str("  </g>\n");
                }
            }
        }

        svg.push_str("</svg>");
        svg
    }
}

/// Strip the outer `<svg ...>` and `</svg>` tags, returning the inner content.
fn strip_svg_wrapper(svg: &str) -> String {
    let start = svg.find('>').map(|i| i + 1).unwrap_or(0);
    let end = svg.rfind("</svg>").unwrap_or(svg.len());
    svg[start..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viz::series_data::DataSeries;

    #[test]
    fn test_subplot_new() {
        let grid = SubplotGrid::new(2, 3);
        assert_eq!(grid.rows, 2);
        assert_eq!(grid.cols, 3);
        assert_eq!(grid.charts.len(), 6);
        assert!(grid.charts.iter().all(|c| c.is_none()));
    }

    #[test]
    fn test_subplot_set_and_render() {
        let mut grid = SubplotGrid::new(1, 2).size(800.0, 400.0);

        let chart_a = Chart::new().title("Chart A").add_series(DataSeries::line(
            vec![1.0, 2.0, 3.0],
            vec![10.0, 20.0, 30.0],
        ));

        let chart_b = Chart::new().title("Chart B").add_series(DataSeries::bar(
            vec!["X".into(), "Y".into()],
            vec![15.0, 25.0],
        ));

        grid.set(0, 0, chart_a);
        grid.set(0, 1, chart_b);

        let svg = grid.to_svg();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("Chart A"));
        assert!(svg.contains("Chart B"));
        assert!(svg.contains("<g transform"));
    }

    #[test]
    fn test_subplot_empty_cells() {
        let mut grid = SubplotGrid::new(2, 2).size(600.0, 600.0);

        // Only fill one cell
        grid.set(
            0,
            0,
            Chart::new().add_series(DataSeries::line(vec![1.0, 2.0], vec![3.0, 4.0])),
        );

        let svg = grid.to_svg();
        assert!(svg.contains("<svg"));
        // Only one <g transform> group expected
        let g_count = svg.matches("<g transform").count();
        assert_eq!(g_count, 1);
    }

    #[test]
    #[should_panic(expected = "row 2 out of bounds")]
    fn test_subplot_out_of_bounds_row() {
        let mut grid = SubplotGrid::new(2, 2);
        grid.set(2, 0, Chart::new());
    }

    #[test]
    #[should_panic(expected = "col 3 out of bounds")]
    fn test_subplot_out_of_bounds_col() {
        let mut grid = SubplotGrid::new(2, 2);
        grid.set(0, 3, Chart::new());
    }

    #[test]
    fn test_subplot_size_builder() {
        let grid = SubplotGrid::new(1, 1).size(500.0, 400.0);
        assert_eq!(grid.width, 500.0);
        assert_eq!(grid.height, 400.0);
    }

    #[test]
    fn test_strip_svg_wrapper() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300">
  <rect x="0" y="0" width="400" height="300" />
</svg>"#;
        let inner = strip_svg_wrapper(svg);
        assert!(inner.contains("<rect"));
        assert!(!inner.contains("<svg"));
        assert!(!inner.contains("</svg>"));
    }
}

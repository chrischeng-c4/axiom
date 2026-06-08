//! Chart builder with fluent API.

use super::annotation::Annotation;
use super::error::{Result, VizError};
use super::render::SvgRenderer;
use super::series_data::DataSeries;
use super::style::ChartStyle;
use super::theme::ThemeName;
use std::path::Path;

/// A chart that can render multiple data series.
#[derive(Debug, Clone)]
pub struct Chart {
    pub title: Option<String>,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub width: f64,
    pub height: f64,
    pub series: Vec<DataSeries>,
    pub style: ChartStyle,
    pub annotations: Vec<Annotation>,
}

impl Default for Chart {
    fn default() -> Self {
        Self::new()
    }
}

impl Chart {
    pub fn new() -> Self {
        Self {
            title: None,
            x_label: None,
            y_label: None,
            width: 800.0,
            height: 600.0,
            series: Vec::new(),
            style: ChartStyle::default(),
            annotations: Vec::new(),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn size(mut self, width: f64, height: f64) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn style(mut self, style: ChartStyle) -> Self {
        self.style = style;
        self
    }

    /// Apply a named theme to this chart.
    pub fn theme(mut self, name: ThemeName) -> Self {
        let theme = super::theme::Theme::from_name(name);
        self.style = theme.to_chart_style();
        self
    }

    /// Set the x-axis label.
    pub fn x_label(mut self, label: &str) -> Self {
        self.x_label = Some(label.to_string());
        self
    }

    /// Set the y-axis label.
    pub fn y_label(mut self, label: &str) -> Self {
        self.y_label = Some(label.to_string());
        self
    }

    pub fn add_series(mut self, series: DataSeries) -> Self {
        self.series.push(series);
        self
    }

    /// Add an annotation overlay.
    pub fn annotate(mut self, annotation: Annotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    /// Render to SVG string.
    pub fn to_svg(&self) -> Result<String> {
        if self.series.is_empty() {
            return Err(VizError::EmptyData("no data series added".into()));
        }
        let mut renderer = SvgRenderer::new(self.width, self.height);
        if self.annotations.is_empty() && self.x_label.is_none() && self.y_label.is_none() {
            Ok(renderer.render(&self.title, &self.series, &self.style))
        } else {
            Ok(renderer.render_full(
                &self.title,
                &self.x_label,
                &self.y_label,
                &self.series,
                &self.style,
                &self.annotations,
            ))
        }
    }

    /// Save to SVG file.
    pub fn save_svg<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let svg = self.to_svg()?;
        std::fs::write(path, svg).map_err(|e| VizError::IoError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_chart() {
        let chart = Chart::new()
            .title("Test Line")
            .size(400.0, 300.0)
            .add_series(DataSeries::line(
                vec![1.0, 2.0, 3.0, 4.0],
                vec![10.0, 20.0, 15.0, 25.0],
            ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Test Line"));
        assert!(svg.contains("<path"));
    }

    #[test]
    fn test_bar_chart() {
        let chart = Chart::new().add_series(DataSeries::bar(
            vec!["A".into(), "B".into(), "C".into()],
            vec![10.0, 20.0, 30.0],
        ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<rect"));
    }

    #[test]
    fn test_scatter_chart() {
        let chart = Chart::new().add_series(DataSeries::scatter(
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
        ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<circle"));
    }

    #[test]
    fn test_histogram() {
        let chart = Chart::new().add_series(DataSeries::histogram(
            vec![1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 5.0],
            5,
        ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<rect"));
    }

    #[test]
    fn test_box_plot() {
        let chart = Chart::new().add_series(DataSeries::box_plot(
            vec![vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![2.0, 3.0, 4.0, 5.0, 6.0]],
            vec!["A".into(), "B".into()],
        ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<line"));
    }

    #[test]
    fn test_heatmap() {
        let chart = Chart::new().add_series(DataSeries::heatmap(vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
        ]));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<rect"));
    }

    #[test]
    fn test_pie_chart() {
        let chart = Chart::new()
            .title("Market Share")
            .add_series(DataSeries::pie(
                vec!["A".into(), "B".into(), "C".into()],
                vec![40.0, 35.0, 25.0],
            ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<path"));
        assert!(svg.contains("Market Share"));
    }

    #[test]
    fn test_area_chart() {
        let chart = Chart::new().add_series(DataSeries::area(
            vec![1.0, 2.0, 3.0, 4.0],
            vec![10.0, 20.0, 15.0, 25.0],
        ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<path"));
        assert!(svg.contains("opacity=\"0.3\"")); // filled area
    }

    #[test]
    fn test_theme_dark() {
        let chart = Chart::new()
            .theme(ThemeName::Dark)
            .add_series(DataSeries::line(vec![1.0, 2.0], vec![3.0, 4.0]));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<svg"));
        // Dark theme has a dark background (#1e1e2e)
        assert!(svg.contains("#1e1e2e"));
    }

    #[test]
    fn test_theme_publication() {
        let chart = Chart::new()
            .theme(ThemeName::Publication)
            .add_series(DataSeries::bar(
                vec!["A".into(), "B".into()],
                vec![10.0, 20.0],
            ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn test_empty_chart() {
        let chart = Chart::new();
        assert!(chart.to_svg().is_err());
    }

    #[test]
    fn test_stacked_bar_chart() {
        let chart = Chart::new()
            .title("Sales by Quarter")
            .add_series(DataSeries::stacked_bar(
                vec!["Q1".into(), "Q2".into(), "Q3".into()],
                vec![
                    ("Product A".into(), vec![20.0, 30.0, 25.0]),
                    ("Product B".into(), vec![15.0, 20.0, 30.0]),
                ],
            ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<rect"));
        assert!(svg.contains("Q1"));
        assert!(svg.contains("Sales by Quarter"));
    }

    #[test]
    fn test_stacked_bar_single_dataset() {
        let chart = Chart::new().add_series(DataSeries::stacked_bar(
            vec!["A".into(), "B".into()],
            vec![("Only".into(), vec![10.0, 20.0])],
        ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<rect"));
    }

    #[test]
    fn test_violin_chart() {
        let chart = Chart::new()
            .title("Distribution")
            .add_series(DataSeries::violin(
                vec![
                    vec![1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 5.0],
                    vec![2.0, 3.0, 3.5, 4.0, 4.5, 5.0, 6.0],
                ],
                vec!["Group A".into(), "Group B".into()],
            ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<path"));
        assert!(svg.contains("Group A"));
        assert!(svg.contains("Distribution"));
    }

    #[test]
    fn test_violin_single_group() {
        let chart = Chart::new().add_series(DataSeries::violin(
            vec![vec![1.0, 2.0, 3.0, 4.0, 5.0]],
            vec!["Solo".into()],
        ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("Solo"));
    }

    #[test]
    fn test_svg_valid_xml() {
        let chart = Chart::new()
            .title("X < Y & Z")
            .add_series(DataSeries::line(vec![1.0, 2.0], vec![3.0, 4.0]));
        let svg = chart.to_svg().unwrap();
        // Special chars should be escaped
        assert!(svg.contains("&lt;"));
        assert!(svg.contains("&amp;"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_polar_chart() {
        let chart = Chart::new().title("Radar").add_series(DataSeries::polar(
            vec![
                "Speed".into(),
                "Power".into(),
                "Range".into(),
                "Stealth".into(),
            ],
            vec![("Fighter".into(), vec![90.0, 70.0, 60.0, 50.0])],
        ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<path"));
        assert!(svg.contains("Speed"));
        assert!(svg.contains("Radar"));
    }

    #[test]
    fn test_donut_chart() {
        let chart = Chart::new().title("Donut").add_series(DataSeries::donut(
            vec!["A".into(), "B".into(), "C".into()],
            vec![40.0, 35.0, 25.0],
            0.5,
        ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<path"));
        assert!(svg.contains("Donut"));
    }

    #[test]
    fn test_surface_3d_chart() {
        let chart = Chart::new()
            .title("Surface")
            .add_series(DataSeries::surface_3d(
                vec![0.0, 1.0, 2.0],
                vec![0.0, 1.0, 2.0],
                vec![
                    vec![1.0, 2.0, 3.0],
                    vec![4.0, 5.0, 6.0],
                    vec![7.0, 8.0, 9.0],
                ],
            ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("<path"));
        assert!(svg.contains("Surface"));
    }

    #[test]
    fn test_chart_with_axis_labels() {
        let chart = Chart::new()
            .title("With Labels")
            .x_label("Time (s)")
            .y_label("Value")
            .add_series(DataSeries::line(
                vec![1.0, 2.0, 3.0],
                vec![10.0, 20.0, 30.0],
            ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("Time (s)"));
        assert!(svg.contains("Value"));
        assert!(svg.contains("rotate(-90"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_chart_with_annotations() {
        use crate::viz::annotation::*;
        let chart = Chart::new()
            .add_series(DataSeries::line(
                vec![1.0, 2.0, 3.0, 4.0],
                vec![10.0, 20.0, 15.0, 25.0],
            ))
            .annotate(Annotation::Text(TextAnnotation::new(2.0, 20.0, "peak")))
            .annotate(Annotation::ReferenceLine(
                ReferenceLineAnnotation::horizontal(17.5).label("avg"),
            ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("peak"));
        assert!(svg.contains("avg"));
        assert!(svg.contains("stroke-dasharray"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_chart_with_arrow_annotation() {
        use crate::viz::annotation::*;
        let chart = Chart::new()
            .add_series(DataSeries::scatter(
                vec![1.0, 2.0, 3.0],
                vec![4.0, 5.0, 6.0],
            ))
            .annotate(Annotation::Arrow(
                ArrowAnnotation::new(1.0, 4.0, 3.0, 6.0).label("trend"),
            ));
        let svg = chart.to_svg().unwrap();
        assert!(svg.contains("trend"));
        assert!(svg.contains("<polygon"));
    }
}

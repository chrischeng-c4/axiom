//! Multi-format export for charts.
//!
//! - **SVG**: Already supported via `Chart::to_svg()` / `Chart::save_svg()`
//! - **HTML**: Interactive HTML with embedded SVG and JavaScript tooltips
//! - **PNG**: Placeholder (requires `resvg` or canvas backend)
//! - **PDF**: Placeholder (requires `printpdf` or similar)
//!
//! This module provides the `ExportFormat` enum and `export_chart` function.

use super::chart::Chart;
use super::error::{Result, VizError};
use std::path::Path;

/// Supported export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Svg,
    Html,
    Png,
    Pdf,
}

impl ExportFormat {
    /// Infer format from file extension.
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .and_then(|ext| match ext.as_str() {
                "svg" => Some(ExportFormat::Svg),
                "html" | "htm" => Some(ExportFormat::Html),
                "png" => Some(ExportFormat::Png),
                "pdf" => Some(ExportFormat::Pdf),
                _ => None,
            })
    }
}

/// Export a chart to the given path, inferring format from extension.
pub fn export_chart<P: AsRef<Path>>(chart: &Chart, path: P) -> Result<()> {
    let path = path.as_ref();
    let format = ExportFormat::from_path(path)
        .ok_or_else(|| VizError::InvalidParameter(format!(
            "cannot infer export format from extension: {:?}",
            path.extension()
        )))?;

    export_chart_as(chart, path, format)
}

/// Export a chart to the given path with an explicit format.
pub fn export_chart_as<P: AsRef<Path>>(
    chart: &Chart,
    path: P,
    format: ExportFormat,
) -> Result<()> {
    match format {
        ExportFormat::Svg => chart.save_svg(path),
        ExportFormat::Html => save_html(chart, path),
        ExportFormat::Png => Err(VizError::InvalidParameter(
            "PNG export requires the 'resvg' feature (not yet available)".into(),
        )),
        ExportFormat::Pdf => Err(VizError::InvalidParameter(
            "PDF export requires the 'printpdf' feature (not yet available)".into(),
        )),
    }
}

/// Generate interactive HTML with embedded SVG and JavaScript tooltips.
pub fn to_html(chart: &Chart) -> Result<String> {
    let svg = chart.to_svg()?;
    let title = chart.title.as_deref().unwrap_or("Chart");

    Ok(format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{title}</title>
<style>
  body {{ margin: 0; display: flex; justify-content: center; align-items: center; min-height: 100vh; background: #f5f5f5; font-family: sans-serif; }}
  .chart-container {{ background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); }}
  .chart-container svg {{ display: block; }}
  .tooltip {{ position: absolute; background: rgba(0,0,0,0.8); color: white; padding: 4px 8px; border-radius: 4px; font-size: 12px; pointer-events: none; display: none; z-index: 1000; }}
</style>
</head>
<body>
<div class="chart-container" id="chart">
{svg}
</div>
<div class="tooltip" id="tooltip"></div>
<script>
(function() {{
  var tooltip = document.getElementById('tooltip');
  var chart = document.getElementById('chart');
  var svg = chart.querySelector('svg');
  if (!svg) return;

  // Add tooltips to circles (scatter), rects (bar/heatmap), paths
  var elements = svg.querySelectorAll('circle, rect, path');
  elements.forEach(function(el) {{
    el.style.cursor = 'pointer';
    el.addEventListener('mouseenter', function(e) {{
      var info = [];
      if (el.tagName === 'circle') {{
        info.push('x: ' + parseFloat(el.getAttribute('cx')).toFixed(1));
        info.push('y: ' + parseFloat(el.getAttribute('cy')).toFixed(1));
      }} else if (el.tagName === 'rect') {{
        var w = parseFloat(el.getAttribute('width'));
        var h = parseFloat(el.getAttribute('height'));
        if (w > 2 && h > 2) {{
          info.push('w: ' + w.toFixed(1) + ', h: ' + h.toFixed(1));
        }}
      }}
      if (info.length > 0) {{
        tooltip.textContent = info.join(', ');
        tooltip.style.display = 'block';
      }}
    }});
    el.addEventListener('mousemove', function(e) {{
      tooltip.style.left = (e.pageX + 10) + 'px';
      tooltip.style.top = (e.pageY - 25) + 'px';
    }});
    el.addEventListener('mouseleave', function() {{
      tooltip.style.display = 'none';
    }});
  }});
}})();
</script>
</body>
</html>"##,
        title = escape_html(title),
        svg = svg,
    ))
}

/// Save as interactive HTML file.
fn save_html<P: AsRef<Path>>(chart: &Chart, path: P) -> Result<()> {
    let html = to_html(chart)?;
    std::fs::write(path, html).map_err(|e| VizError::IoError(e.to_string()))
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viz::DataSeries;

    fn sample_chart() -> Chart {
        Chart::new()
            .title("Test")
            .add_series(DataSeries::line(vec![1.0, 2.0, 3.0], vec![10.0, 20.0, 30.0]))
    }

    #[test]
    fn test_export_format_from_path_svg() {
        assert_eq!(
            ExportFormat::from_path(Path::new("chart.svg")),
            Some(ExportFormat::Svg)
        );
    }

    #[test]
    fn test_export_format_from_path_html() {
        assert_eq!(
            ExportFormat::from_path(Path::new("chart.html")),
            Some(ExportFormat::Html)
        );
        assert_eq!(
            ExportFormat::from_path(Path::new("chart.htm")),
            Some(ExportFormat::Html)
        );
    }

    #[test]
    fn test_export_format_from_path_png() {
        assert_eq!(
            ExportFormat::from_path(Path::new("chart.png")),
            Some(ExportFormat::Png)
        );
    }

    #[test]
    fn test_export_format_from_path_pdf() {
        assert_eq!(
            ExportFormat::from_path(Path::new("chart.pdf")),
            Some(ExportFormat::Pdf)
        );
    }

    #[test]
    fn test_export_format_from_path_unknown() {
        assert_eq!(ExportFormat::from_path(Path::new("chart.xyz")), None);
    }

    #[test]
    fn test_export_format_case_insensitive() {
        assert_eq!(
            ExportFormat::from_path(Path::new("Chart.SVG")),
            Some(ExportFormat::Svg)
        );
        assert_eq!(
            ExportFormat::from_path(Path::new("Chart.HTML")),
            Some(ExportFormat::Html)
        );
    }

    #[test]
    fn test_to_html_produces_valid_html() {
        let chart = sample_chart();
        let html = to_html(&chart).unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<svg"));
        assert!(html.contains("</svg>"));
        assert!(html.contains("tooltip"));
        assert!(html.contains("Test"));
    }

    #[test]
    fn test_to_html_escapes_title() {
        let chart = Chart::new()
            .title("A < B & C")
            .add_series(DataSeries::line(vec![1.0, 2.0], vec![3.0, 4.0]));
        let html = to_html(&chart).unwrap();
        assert!(html.contains("A &lt; B &amp; C"));
    }

    #[test]
    fn test_export_svg_via_temp_file() {
        let chart = sample_chart();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.svg");
        export_chart(&chart, &path).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("<svg"));
    }

    #[test]
    fn test_export_html_via_temp_file() {
        let chart = sample_chart();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.html");
        export_chart(&chart, &path).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_export_png_returns_error() {
        let chart = sample_chart();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.png");
        let result = export_chart(&chart, &path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("resvg"));
    }

    #[test]
    fn test_export_pdf_returns_error() {
        let chart = sample_chart();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.pdf");
        let result = export_chart(&chart, &path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("printpdf"));
    }

    #[test]
    fn test_export_unknown_format() {
        let chart = sample_chart();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.xyz");
        let result = export_chart(&chart, &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_chart_as_explicit_format() {
        let chart = sample_chart();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("chart_output");
        // Use explicit format even with no extension
        export_chart_as(&chart, &path, ExportFormat::Svg).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("<svg"));
    }
}

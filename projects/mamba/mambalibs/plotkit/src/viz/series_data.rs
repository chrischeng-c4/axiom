//! Data series types for charts.

use super::style::{BarStyle, LineStyle, PointStyle};

/// A data series to be plotted on a chart.
#[derive(Debug, Clone)]
pub enum DataSeries {
    Line {
        x: Vec<f64>,
        y: Vec<f64>,
        label: Option<String>,
        style: LineStyle,
    },
    Bar {
        labels: Vec<String>,
        values: Vec<f64>,
        label: Option<String>,
        style: BarStyle,
    },
    Scatter {
        x: Vec<f64>,
        y: Vec<f64>,
        label: Option<String>,
        style: PointStyle,
    },
    Histogram {
        data: Vec<f64>,
        bins: usize,
        label: Option<String>,
        style: BarStyle,
    },
    BoxPlot {
        data: Vec<Vec<f64>>,
        labels: Vec<String>,
        style: BarStyle,
    },
    Heatmap {
        data: Vec<Vec<f64>>,
        x_labels: Option<Vec<String>>,
        y_labels: Option<Vec<String>>,
    },
    Pie {
        labels: Vec<String>,
        values: Vec<f64>,
    },
    Area {
        x: Vec<f64>,
        y: Vec<f64>,
        label: Option<String>,
        style: LineStyle,
    },
    StackedBar {
        labels: Vec<String>,
        datasets: Vec<(String, Vec<f64>)>,
    },
    Violin {
        data: Vec<Vec<f64>>,
        labels: Vec<String>,
    },
    /// Polar/Radar chart: each dataset is a named series of values
    /// matching the axis labels.
    Polar {
        axis_labels: Vec<String>,
        datasets: Vec<(String, Vec<f64>)>,
    },
    /// Donut chart: like pie but with an inner radius hole.
    Donut {
        labels: Vec<String>,
        values: Vec<f64>,
        /// Inner radius as fraction of outer radius (0.0-1.0).
        hole_ratio: f64,
    },
    /// 3D surface: z values on a grid defined by x and y coordinates.
    Surface3D {
        x: Vec<f64>,
        y: Vec<f64>,
        z: Vec<Vec<f64>>,
    },
}

impl DataSeries {
    /// Create a line series from x and y vectors.
    pub fn line(x: Vec<f64>, y: Vec<f64>) -> Self {
        DataSeries::Line {
            x,
            y,
            label: None,
            style: LineStyle::default(),
        }
    }

    /// Create a bar series from labels and values.
    pub fn bar(labels: Vec<String>, values: Vec<f64>) -> Self {
        DataSeries::Bar {
            labels,
            values,
            label: None,
            style: BarStyle::default(),
        }
    }

    /// Create a scatter series from x and y vectors.
    pub fn scatter(x: Vec<f64>, y: Vec<f64>) -> Self {
        DataSeries::Scatter {
            x,
            y,
            label: None,
            style: PointStyle::default(),
        }
    }

    /// Create a histogram from data.
    pub fn histogram(data: Vec<f64>, bins: usize) -> Self {
        DataSeries::Histogram {
            data,
            bins,
            label: None,
            style: BarStyle::default(),
        }
    }

    /// Create a box plot from multiple data groups.
    pub fn box_plot(data: Vec<Vec<f64>>, labels: Vec<String>) -> Self {
        DataSeries::BoxPlot {
            data,
            labels,
            style: BarStyle::default(),
        }
    }

    /// Create a heatmap from a 2D matrix.
    pub fn heatmap(data: Vec<Vec<f64>>) -> Self {
        DataSeries::Heatmap {
            data,
            x_labels: None,
            y_labels: None,
        }
    }

    /// Create a pie chart from labels and values.
    pub fn pie(labels: Vec<String>, values: Vec<f64>) -> Self {
        DataSeries::Pie { labels, values }
    }

    /// Create an area chart (filled line) from x and y vectors.
    pub fn area(x: Vec<f64>, y: Vec<f64>) -> Self {
        DataSeries::Area {
            x,
            y,
            label: None,
            style: LineStyle::default(),
        }
    }

    /// Create a stacked bar chart from labels and named datasets.
    ///
    /// Each dataset is a `(name, values)` tuple where `values.len()` should
    /// equal `labels.len()`.
    pub fn stacked_bar(labels: Vec<String>, datasets: Vec<(String, Vec<f64>)>) -> Self {
        DataSeries::StackedBar { labels, datasets }
    }

    /// Create a violin plot from multiple data groups.
    ///
    /// Each inner `Vec<f64>` is one group; `labels` names each group.
    pub fn violin(data: Vec<Vec<f64>>, labels: Vec<String>) -> Self {
        DataSeries::Violin { data, labels }
    }

    /// Create a polar/radar chart from axis labels and named datasets.
    ///
    /// Each dataset is `(name, values)` where `values.len()` should equal
    /// `axis_labels.len()`.
    pub fn polar(axis_labels: Vec<String>, datasets: Vec<(String, Vec<f64>)>) -> Self {
        DataSeries::Polar {
            axis_labels,
            datasets,
        }
    }

    /// Create a donut chart from labels, values, and hole ratio.
    ///
    /// `hole_ratio` is the inner radius as a fraction of the outer radius (0.0-1.0).
    /// Use 0.4-0.6 for a typical donut.
    pub fn donut(labels: Vec<String>, values: Vec<f64>, hole_ratio: f64) -> Self {
        DataSeries::Donut {
            labels,
            values,
            hole_ratio: hole_ratio.clamp(0.0, 0.95),
        }
    }

    /// Create a 3D surface (rendered as a projected wireframe + heatmap).
    ///
    /// `x` has length `n_cols`, `y` has length `n_rows`, and `z` is `n_rows x n_cols`.
    pub fn surface_3d(x: Vec<f64>, y: Vec<f64>, z: Vec<Vec<f64>>) -> Self {
        DataSeries::Surface3D { x, y, z }
    }

    /// Set the series label (for legend).
    pub fn with_label(mut self, label: &str) -> Self {
        match &mut self {
            DataSeries::Line { label: l, .. } => *l = Some(label.to_string()),
            DataSeries::Bar { label: l, .. } => *l = Some(label.to_string()),
            DataSeries::Scatter { label: l, .. } => *l = Some(label.to_string()),
            DataSeries::Histogram { label: l, .. } => *l = Some(label.to_string()),
            DataSeries::Area { label: l, .. } => *l = Some(label.to_string()),
            _ => {}
        }
        self
    }
}

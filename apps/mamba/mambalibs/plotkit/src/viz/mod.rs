//! Visualization module — pure SVG chart generation.
//!
//! - **chart**: Chart builder with fluent API
//! - **series_data**: DataSeries types (Line, Bar, Scatter, Histogram, Box, Heatmap, Pie, Area, StackedBar, Violin, Polar, Donut, Surface3D)
//! - **style**: Colors, line/bar/point styles, chart theme
//! - **axis**: Nice-tick algorithm for axis labels
//! - **render**: SVG renderer (with legend support)
//! - **theme**: Named themes (Light, Dark, Minimal, Publication)
//! - **subplot**: Subplot grid for multi-chart layouts
//! - **annotation**: Text, arrow, reference line, shape annotations
//! - **export**: Multi-format export (SVG, HTML interactive, PNG/PDF stubs)
//! - **dataframe**: Plot from column data (DataFrame integration)

mod error;

pub mod annotation;
pub mod axis;
pub mod chart;
pub mod dataframe;
pub mod export;
pub mod palette;
pub mod render;
pub mod series_data;
pub mod style;
pub mod subplot;
pub mod theme;

pub use chart::Chart;
pub use error::{Result, VizError};
pub use palette::{categorical, diverging, interpolate_color, sequential_blue, Palette};
pub use series_data::DataSeries;
pub use style::{BarStyle, ChartStyle, Color, LineStyle, PointStyle};
pub use subplot::SubplotGrid;
pub use theme::{Theme, ThemeName};

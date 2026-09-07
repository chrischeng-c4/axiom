//! The interpreter-free half of `mambalibs.plot`: the module's own name, the
//! theme registry, the refusal type every wrapper raises through, and the
//! argument checks that stand between Python and a chart `viz` cannot draw.
//!
//! Nothing here touches PyO3. That is the point: `plotkit` is a plain Rust
//! crate, so the only decisions the binding actually makes are *what* a
//! refusal says, *which* theme name selects which preset, and *when* a Python
//! argument is unfit to be handed to `viz` at all. Those are behaviour, they
//! are testable without an interpreter, and `src/tests.rs` judges them
//! directly. `src/lib.rs` adds names and conversions on top and no rule of
//! its own.
//!
//! # Why the argument checks exist
//!
//! `viz` does not assert on a malformed series — it draws one anyway, and
//! that is the reason to refuse it here rather than after the fact:
//!
//! * `render_line` and `render_scatter` (`render/svg_series.rs:34`, `:134`)
//!   pair `x` with `y` through `zip`, so a series with three x values and two
//!   y values silently plots two points and drops the third. `render_bar`
//!   (`:93`) does the same for labels against values. A caller who mismatched
//!   the lengths chose an argument, not a chart.
//! * every coordinate reaches `nice_ticks` (`axis.rs:34`) and then
//!   `format!("{:.1}", ..)`. A `NaN` or an infinity produces an axis whose
//!   bounds are `NaN` and an SVG carrying literal `NaN` coordinates: a
//!   document that renders as nothing, with no error anywhere.
//! * `Chart::size` (`chart.rs:47`) is the renderer's canvas, and the margins
//!   are subtracted from it. A zero, negative, or non-finite canvas gives the
//!   plot area a negative width and every point maps outside the viewport.
//! * `nice_ticks` grows its tick vector one step at a time from `nice_min` to
//!   `nice_max`, so a large `tick_count` over a narrow range allocates
//!   proportionally many ticks. [`MAX_TICK_COUNT`] bounds that before the
//!   allocation, because an out-of-memory abort inside an extension module is
//!   not an exception a caller can catch.
//!
//! An empty series is refused for the same reason: `render_line` returns
//! early on empty input (`render/svg_series.rs:17`), so the chart renders
//! with the series contributing nothing and no complaint — while `viz`'s own
//! vocabulary already calls that condition `VizError::EmptyData`.

use plotkit::viz::axis::{nice_ticks, TickInfo};
use plotkit::viz::{export, Chart, DataSeries, ThemeName, VizError};

/// The extension module's own dotted name.
pub const MODULE_NAME: &str = "mambalibs.plot";

/// The four theme presets, in the order `viz::theme` declares them.
///
/// This list *is* what `mambalibs.plot.ThemeName` publishes: the e2e contract
/// reads exactly these names off the class, and [`theme_from_name`] is the
/// only place a string becomes a `viz` preset.
pub const THEME_NAMES: [&str; 4] = ["Light", "Dark", "Minimal", "Publication"];

/// The largest `tick_count` [`checked_nice_ticks`] will pass to `viz`.
pub const MAX_TICK_COUNT: usize = 1_000;

/// A refusal on its way to Python.
///
/// Every failure this binding can meet — a [`VizError`] or an argument `viz`
/// would have silently mis-drawn — collapses into one message here, because
/// Python sees exactly one exception type (`ValueError`) for all of them.
/// Keeping the collapse in this module means `src/lib.rs` never decides what
/// a refusal says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlotError {
    message: String,
}

impl PlotError {
    /// A refusal carrying `message`.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// The text Python will see on the raised exception.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl From<VizError> for PlotError {
    /// A `viz` failure keeps the wording `viz` gave it, so the reason a chart
    /// refused to render survives the crossing into Python.
    fn from(err: VizError) -> Self {
        Self::new(err.to_string())
    }
}

/// The theme preset `name` selects, or `None` when it names no preset.
///
/// The match is exact: `ThemeName.Dark` is the Rust variant spelled the Rust
/// way, and accepting `"dark"` here would invent a spelling `viz` does not
/// have.
pub fn theme_from_name(name: &str) -> Option<ThemeName> {
    match name {
        "Light" => Some(ThemeName::Light),
        "Dark" => Some(ThemeName::Dark),
        "Minimal" => Some(ThemeName::Minimal),
        "Publication" => Some(ThemeName::Publication),
        _ => None,
    }
}

/// A line series over `x` and `y`.
pub fn checked_line(x: Vec<f64>, y: Vec<f64>) -> Result<DataSeries, PlotError> {
    checked_xy("line", &x, &y)?;
    Ok(DataSeries::line(x, y))
}

/// A scatter series over `x` and `y`.
pub fn checked_scatter(x: Vec<f64>, y: Vec<f64>) -> Result<DataSeries, PlotError> {
    checked_xy("scatter", &x, &y)?;
    Ok(DataSeries::scatter(x, y))
}

/// A bar series over `labels` and `values`.
pub fn checked_bar(labels: Vec<String>, values: Vec<f64>) -> Result<DataSeries, PlotError> {
    if labels.is_empty() {
        return Err(PlotError::new(
            "empty data: a bar series needs at least one label",
        ));
    }
    if labels.len() != values.len() {
        return Err(PlotError::new(format!(
            "a bar series needs one value per label, got {} label(s) and {} value(s)",
            labels.len(),
            values.len()
        )));
    }
    all_finite("value", &values)?;
    Ok(DataSeries::bar(labels, values))
}

/// The canvas `width` x `height`, once it is a canvas at all.
pub fn checked_size(width: f64, height: f64) -> Result<(f64, f64), PlotError> {
    for (name, value) in [("width", width), ("height", height)] {
        if !value.is_finite() || value <= 0.0 {
            return Err(PlotError::new(format!(
                "a chart {name} must be a positive finite number of SVG units, got {value}"
            )));
        }
    }
    Ok((width, height))
}

/// The tick layout `viz` computes for `data_min..data_max`.
pub fn checked_nice_ticks(
    data_min: f64,
    data_max: f64,
    tick_count: usize,
) -> Result<TickInfo, PlotError> {
    for (name, value) in [("data_min", data_min), ("data_max", data_max)] {
        if !value.is_finite() {
            return Err(PlotError::new(format!(
                "{name} must be a finite number, got {value}"
            )));
        }
    }
    if tick_count == 0 || tick_count > MAX_TICK_COUNT {
        return Err(PlotError::new(format!(
            "tick_count must be between 1 and {MAX_TICK_COUNT}, got {tick_count}"
        )));
    }
    Ok(nice_ticks(data_min, data_max, tick_count))
}

/// The chart as an SVG document.
pub fn render_svg(chart: &Chart) -> Result<String, PlotError> {
    chart.to_svg().map_err(PlotError::from)
}

/// The chart as a standalone HTML document embedding that SVG.
pub fn render_html(chart: &Chart) -> Result<String, PlotError> {
    export::to_html(chart).map_err(PlotError::from)
}

/// The shared check behind [`checked_line`] and [`checked_scatter`]: a series
/// of paired coordinates is one `y` per `x`, and every one of them plottable.
fn checked_xy(kind: &str, x: &[f64], y: &[f64]) -> Result<(), PlotError> {
    if x.is_empty() || y.is_empty() {
        return Err(PlotError::new(format!(
            "empty data: a {kind} series needs at least one point"
        )));
    }
    if x.len() != y.len() {
        return Err(PlotError::new(format!(
            "a {kind} series needs one y per x, got {} x value(s) and {} y value(s)",
            x.len(),
            y.len()
        )));
    }
    all_finite("x", x)?;
    all_finite("y", y)
}

/// Refuse the first value that cannot be placed on an axis, naming where it
/// is: `NaN` and the infinities reach the renderer as literal coordinates.
fn all_finite(name: &str, values: &[f64]) -> Result<(), PlotError> {
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(PlotError::new(format!(
                "every {name} must be a finite number, got {value} at index {index}"
            )));
        }
    }
    Ok(())
}

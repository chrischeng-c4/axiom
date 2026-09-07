//! `mambalibs.plot` — plotkit's `viz` charting surface, as a PyO3 `abi3`
//! extension module.
//!
//! # What lives here, and what does not
//!
//! This file is the boundary and nothing else: it names the four classes the
//! wheel publishes (`Chart`, `DataSeries`, `ThemeName`, `TickInfo`) and the
//! three module-level functions (`to_html`, `nice_ticks`, `format_tick`),
//! converts Python arguments into Rust values, and maps a
//! [`core::PlotError`] onto the Python exception it deserves — always
//! `ValueError`, because every `VizError` variant and every argument check
//! behind it names an argument the caller could have chosen differently.
//! Every decision with behaviour in it — which arguments `viz` can actually
//! plot, what a refusal says, which theme name selects which preset — belongs
//! to [`core`], which holds no PyO3 type and is judged by `src/tests.rs`
//! without an interpreter.
//!
//! # Why the surface is flat
//!
//! `plotkit/src/lib.rs` has only `pub mod viz;`, and `viz/mod.rs` re-exports
//! its public names at the module root. The binding keeps that shape: there is
//! no `plot.viz` submodule, because a level that always has one member names
//! nothing. This mirrors `mambalibs.array` over `arraykit`'s single module,
//! and differs from `mambalibs.sci`, whose core crate really does publish nine.
//!
//! # Why the fluent methods return a fresh chart
//!
//! `viz::Chart`'s builder methods take `self` by value
//! (`plotkit/src/viz/chart.rs:41`), and `Chart` is `Clone` (line 13). Each
//! method here therefore clones, applies, and hands back a new `Chart`, so a
//! chained call reads the same in Python as in Rust and no half-built chart is
//! ever observable. The e2e contract asserts only through chained calls, so it
//! admits this and the mutating alternative equally.
//!
//! # Why the module is a namespace member
//!
//! `[tool.maturin] module-name = "mambalibs.plot"` with `python-source =
//! "python"` installs this extension as `mambalibs/plot.abi3.so` beside no
//! `mambalibs/__init__.py`, so `mambalibs` stays a PEP 420 namespace package
//! and this wheel shares one directory with `mambalibs.array` and
//! `mambalibs.sci`.
//!
//! # ABI
//!
//! `abi3-py312`: one `cp312-abi3-<platform>` wheel serves every CPython from
//! 3.12 up, so the package manager's shipped tag selector resolves it without
//! a per-minor build. The `extension-module` feature is off by default and
//! turned on by maturin, which is what lets `cargo test --lib` link a real
//! libpython and run the tests in `src/tests.rs`.

pub mod core;

#[cfg(test)]
mod tests;

use plotkit::viz::axis::{self, TickInfo};
use plotkit::viz::{Chart, DataSeries, ThemeName};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::core::{
    checked_bar, checked_line, checked_nice_ticks, checked_scatter, checked_size, render_html,
    render_svg, PlotError,
};

/// `mambalibs.plot.ThemeName` — one of `viz`'s four theme presets.
///
/// A fieldless `#[pyclass]` enum, so each variant is a class attribute
/// (`ThemeName.Dark`) and two of them compare equal only when they are the
/// same variant.
#[pyclass(name = "ThemeName", eq, eq_int, frozen, module = "mambalibs.plot")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PyThemeName {
    Light,
    Dark,
    Minimal,
    Publication,
}

impl From<PyThemeName> for ThemeName {
    fn from(value: PyThemeName) -> Self {
        match value {
            PyThemeName::Light => ThemeName::Light,
            PyThemeName::Dark => ThemeName::Dark,
            PyThemeName::Minimal => ThemeName::Minimal,
            PyThemeName::Publication => ThemeName::Publication,
        }
    }
}

#[pymethods]
impl PyThemeName {
    fn __repr__(&self) -> String {
        format!("ThemeName.{self:?}")
    }
}

/// `mambalibs.plot.DataSeries` — one plotted series.
#[pyclass(name = "DataSeries", module = "mambalibs.plot")]
#[derive(Debug, Clone)]
pub struct PyDataSeries {
    inner: DataSeries,
}

#[pymethods]
impl PyDataSeries {
    /// A line through the points `(x[i], y[i])`.
    #[staticmethod]
    fn line(x: Vec<f64>, y: Vec<f64>) -> PyResult<Self> {
        Ok(Self {
            inner: checked_line(x, y).map_err(into_py_err)?,
        })
    }

    /// One bar per label, `values[i]` tall.
    #[staticmethod]
    fn bar(labels: Vec<String>, values: Vec<f64>) -> PyResult<Self> {
        Ok(Self {
            inner: checked_bar(labels, values).map_err(into_py_err)?,
        })
    }

    /// One point per pair `(x[i], y[i])`.
    #[staticmethod]
    fn scatter(x: Vec<f64>, y: Vec<f64>) -> PyResult<Self> {
        Ok(Self {
            inner: checked_scatter(x, y).map_err(into_py_err)?,
        })
    }

    fn __repr__(&self) -> String {
        format!("DataSeries({})", series_kind(&self.inner))
    }
}

/// `mambalibs.plot.Chart` — a chart that renders its series to SVG.
#[pyclass(name = "Chart", module = "mambalibs.plot")]
#[derive(Debug, Clone)]
pub struct PyChart {
    inner: Chart,
}

#[pymethods]
impl PyChart {
    /// An empty chart of `viz`'s default size.
    #[new]
    fn new() -> Self {
        Self {
            inner: Chart::new(),
        }
    }

    /// The chart's title, drawn above the plot area.
    fn title(&self, title: &str) -> Self {
        Self {
            inner: self.inner.clone().title(title),
        }
    }

    /// The canvas the renderer draws on, in SVG user units.
    fn size(&self, width: f64, height: f64) -> PyResult<Self> {
        let (width, height) = checked_size(width, height).map_err(into_py_err)?;
        Ok(Self {
            inner: self.inner.clone().size(width, height),
        })
    }

    /// The x-axis label.
    fn x_label(&self, label: &str) -> Self {
        Self {
            inner: self.inner.clone().x_label(label),
        }
    }

    /// The y-axis label.
    fn y_label(&self, label: &str) -> Self {
        Self {
            inner: self.inner.clone().y_label(label),
        }
    }

    /// Repaint the chart with a named theme preset.
    fn theme(&self, name: PyThemeName) -> Self {
        Self {
            inner: self.inner.clone().theme(name.into()),
        }
    }

    /// Add one more series to the chart.
    fn add_series(&self, series: PyDataSeries) -> Self {
        Self {
            inner: self.inner.clone().add_series(series.inner),
        }
    }

    /// Render this chart as a whole SVG document.
    fn to_svg(&self) -> PyResult<String> {
        render_svg(&self.inner).map_err(into_py_err)
    }

    fn __repr__(&self) -> String {
        format!(
            "Chart(title={:?}, size=({}, {}), series={})",
            self.inner.title,
            self.inner.width,
            self.inner.height,
            self.inner.series.len()
        )
    }
}

/// `mambalibs.plot.TickInfo` — the tick layout [`nice_ticks`] computed.
#[pyclass(name = "TickInfo", frozen, module = "mambalibs.plot")]
#[derive(Debug, Clone)]
pub struct PyTickInfo {
    /// The rounded low end of the axis.
    #[pyo3(get)]
    min: f64,
    /// The rounded high end of the axis.
    #[pyo3(get)]
    max: f64,
    /// Every tick position, from `min` to `max`.
    #[pyo3(get)]
    ticks: Vec<f64>,
    /// The distance between two neighbouring ticks.
    #[pyo3(get)]
    step: f64,
}

impl From<TickInfo> for PyTickInfo {
    fn from(value: TickInfo) -> Self {
        Self {
            min: value.min,
            max: value.max,
            ticks: value.ticks,
            step: value.step,
        }
    }
}

#[pymethods]
impl PyTickInfo {
    fn __repr__(&self) -> String {
        format!(
            "TickInfo(min={}, max={}, step={}, ticks={})",
            self.min,
            self.max,
            self.step,
            self.ticks.len()
        )
    }
}

/// A standalone HTML document embedding `chart`'s SVG.
#[pyfunction]
fn to_html(chart: PyRef<'_, PyChart>) -> PyResult<String> {
    render_html(&chart.inner).map_err(into_py_err)
}

/// The "nice" tick layout for the range `data_min..data_max`.
#[pyfunction]
fn nice_ticks(data_min: f64, data_max: f64, tick_count: usize) -> PyResult<PyTickInfo> {
    Ok(checked_nice_ticks(data_min, data_max, tick_count)
        .map_err(into_py_err)?
        .into())
}

/// A tick value, spelled the way the renderer spells it.
#[pyfunction]
fn format_tick(value: f64) -> String {
    axis::format_tick(value)
}

/// The variant name of a series, for `DataSeries.__repr__`.
fn series_kind(series: &DataSeries) -> &'static str {
    match series {
        DataSeries::Line { .. } => "line",
        DataSeries::Bar { .. } => "bar",
        DataSeries::Scatter { .. } => "scatter",
        _ => "series",
    }
}

/// Map a core refusal onto the Python exception a caller would expect. Every
/// one is a `ValueError`: each names an argument the caller chose.
fn into_py_err(err: PlotError) -> PyErr {
    PyValueError::new_err(err.message().to_string())
}

/// The extension module itself. The function name is the module's last
/// component, so this is `mambalibs.plot`.
#[pymodule]
fn plot(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyChart>()?;
    m.add_class::<PyDataSeries>()?;
    m.add_class::<PyThemeName>()?;
    m.add_class::<PyTickInfo>()?;
    m.add_function(wrap_pyfunction!(to_html, m)?)?;
    m.add_function(wrap_pyfunction!(nice_ticks, m)?)?;
    m.add_function(wrap_pyfunction!(format_tick, m)?)?;
    Ok(())
}

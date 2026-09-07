//! `mambalibs.sci` — scikit's nine feature-gated modules, as a PyO3 `abi3`
//! extension module.
//!
//! # What lives here, and what does not
//!
//! This file is the boundary and nothing else: it names the classes and
//! functions the wheel publishes, converts Python arguments into Rust values,
//! and turns a [`core::SciError`] into the Python exception it deserves.
//! Every decision with behaviour in it — which modules exist, what a refusal
//! says, and which arguments `scikit` would have asserted on — belongs to
//! [`core`], which holds no PyO3 type and is judged by `src/tests.rs` without
//! an interpreter.
//!
//! # Shape of the surface
//!
//! `scikit` is one crate of nine modules, and the binding keeps that shape: a
//! [`PyModule`] per module, created with [`PyModule::new`] and attached with
//! `add_submodule`, so `sci.stats.median` reads the way `scikit::stats::median`
//! does. The mapping from Rust to Python is by rule, the same rules
//! `mambalibs.array` already ships — a free function stays a function of the
//! same snake_case name and argument order; a `struct` or `enum` becomes a
//! class of the same PascalCase name, a fieldless enum publishing each variant
//! as a class attribute; a public field or a zero-argument reporter becomes an
//! attribute, and anything taking arguments or building a new representation
//! stays a method; an associated constructor becomes a classmethod and `new`
//! becomes `__init__`; `Result` raises, `Option` returns `None`, `Vec<f64>` is
//! a `list[float]`, and a Rust closure parameter takes any Python callable.
//!
//! # The one call back into Python
//!
//! [`brentq`] is the only place this module calls a Python object. Its
//! `scikit` signature takes `F: Fn(f64) -> f64` — not `FnMut`, and with no
//! error channel — so a Python exception raised inside the callback cannot be
//! returned through it. The closure parks the [`PyErr`] in a [`RefCell`] and
//! answers `NaN`; the wrapper re-raises it after the solver returns, so a
//! callback that raises reaches the caller as its own exception rather than as
//! a convergence failure.
//!
//! # Why the module is a namespace member
//!
//! `[tool.maturin] module-name = "mambalibs.sci"` with `python-source =
//! "python"` installs this extension as `mambalibs/sci.abi3.so` beside no
//! `mambalibs/__init__.py`, so `mambalibs` stays a PEP 420 namespace package
//! and `mambalibs.array` keeps working in the same environment.
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

use std::cell::RefCell;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyType;

use crate::core::{
    checked_dense, checked_dot, checked_point_cloud, checked_point_pair, checked_query_point,
    submodule_path, SciError, SUBMODULES,
};

/// Raise a core refusal as `ValueError`.
///
/// Every failure this binding can report is an argument the caller could have
/// chosen differently — a rank mismatch, a bracket with no root inside it, a
/// series too short to have an autocorrelation — so they share one exception
/// type rather than inviting a caller to branch on a distinction `scikit` does
/// not draw.
fn into_py_err(err: SciError) -> PyErr {
    PyValueError::new_err(err.message().to_string())
}

// ===========================================================================
// stats
// ===========================================================================

/// `mambalibs.sci.stats.TestResult` — a statistic with its p-value.
#[pyclass(name = "TestResult", frozen, module = "mambalibs.sci.stats")]
#[derive(Debug, Clone, Copy)]
pub struct PyTestResult {
    inner: scikit::stats::TestResult,
}

#[pymethods]
impl PyTestResult {
    /// The test statistic.
    #[getter]
    fn statistic(&self) -> f64 {
        self.inner.statistic
    }

    /// The p-value, two-tailed unless the test says otherwise.
    #[getter]
    fn pvalue(&self) -> f64 {
        self.inner.pvalue
    }

    /// Whether the p-value clears `alpha`.
    fn is_significant(&self, alpha: f64) -> bool {
        self.inner.is_significant(alpha)
    }

    fn __repr__(&self) -> String {
        format!(
            "TestResult(statistic={}, pvalue={})",
            self.inner.statistic, self.inner.pvalue
        )
    }
}

/// The median of `data`, or `None` when there is no sample to take one of.
#[pyfunction]
fn median(data: Vec<f64>) -> Option<f64> {
    scikit::stats::median(&data)
}

/// A one-sample t-test of `sample` against `popmean`.
#[pyfunction]
fn ttest_1samp(sample: Vec<f64>, popmean: f64) -> PyTestResult {
    PyTestResult {
        inner: scikit::stats::ttest_1samp(&sample, popmean),
    }
}

// ===========================================================================
// fft
// ===========================================================================

/// `mambalibs.sci.fft.Complex` — one frequency bin.
#[pyclass(name = "Complex", frozen, module = "mambalibs.sci.fft")]
#[derive(Debug, Clone, Copy)]
pub struct PyComplex {
    inner: scikit::fft::Complex,
}

#[pymethods]
impl PyComplex {
    #[new]
    fn new(re: f64, im: f64) -> Self {
        Self {
            inner: scikit::fft::Complex::new(re, im),
        }
    }

    /// The real part.
    #[getter]
    fn re(&self) -> f64 {
        self.inner.re
    }

    /// The imaginary part.
    #[getter]
    fn im(&self) -> f64 {
        self.inner.im
    }

    /// The magnitude of this bin.
    #[getter]
    fn norm(&self) -> f64 {
        self.inner.norm()
    }

    fn __repr__(&self) -> String {
        format!("Complex(re={}, im={})", self.inner.re, self.inner.im)
    }
}

/// The frequency of each bin of an `n`-sample real transform at spacing `d`.
#[pyfunction]
fn rfftfreq(n: usize, d: f64) -> Vec<f64> {
    scikit::fft::rfftfreq(n, d)
}

/// The one-sided FFT of a real signal.
#[pyfunction]
fn rfft(input: Vec<f64>) -> Vec<PyComplex> {
    scikit::fft::rfft(&input)
        .into_iter()
        .map(|inner| PyComplex { inner })
        .collect()
}

// ===========================================================================
// signal
// ===========================================================================

/// `mambalibs.sci.signal.ConvolveMode` — how much of the convolution to keep.
#[pyclass(name = "ConvolveMode", eq, eq_int, frozen, module = "mambalibs.sci.signal")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PyConvolveMode {
    Full,
    Same,
    Valid,
}

impl From<PyConvolveMode> for scikit::signal::ConvolveMode {
    fn from(value: PyConvolveMode) -> Self {
        match value {
            PyConvolveMode::Full => scikit::signal::ConvolveMode::Full,
            PyConvolveMode::Same => scikit::signal::ConvolveMode::Same,
            PyConvolveMode::Valid => scikit::signal::ConvolveMode::Valid,
        }
    }
}

/// A Hann window of `n` points.
#[pyfunction]
fn hann(n: usize) -> Vec<f64> {
    scikit::signal::hann(n)
}

/// The convolution of `a` with `b`, trimmed by `mode`.
#[pyfunction]
fn convolve(a: Vec<f64>, b: Vec<f64>, mode: PyConvolveMode) -> Vec<f64> {
    scikit::signal::convolve(&a, &b, mode.into())
}

// ===========================================================================
// interpolate
// ===========================================================================

/// `mambalibs.sci.interpolate.InterpKind` — the interpolation method.
#[pyclass(
    name = "InterpKind",
    eq,
    eq_int,
    frozen,
    module = "mambalibs.sci.interpolate"
)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PyInterpKind {
    Linear,
    CubicSpline,
    Nearest,
}

impl From<PyInterpKind> for scikit::interpolate::InterpKind {
    fn from(value: PyInterpKind) -> Self {
        match value {
            PyInterpKind::Linear => scikit::interpolate::InterpKind::Linear,
            PyInterpKind::CubicSpline => scikit::interpolate::InterpKind::CubicSpline,
            PyInterpKind::Nearest => scikit::interpolate::InterpKind::Nearest,
        }
    }
}

/// Interpolate the knots `(x, y)` at every point of `x_new`.
#[pyfunction]
fn interp1d(x: Vec<f64>, y: Vec<f64>, x_new: Vec<f64>, kind: PyInterpKind) -> PyResult<Vec<f64>> {
    checked_point_pair(&x, &y).map_err(into_py_err)?;
    Ok(scikit::interpolate::interp1d(&x, &y, &x_new, kind.into()))
}

/// `mambalibs.sci.interpolate.CubicSpline` — a natural cubic spline.
#[pyclass(name = "CubicSpline", frozen, module = "mambalibs.sci.interpolate")]
#[derive(Debug, Clone)]
pub struct PyCubicSpline {
    inner: scikit::interpolate::CubicSpline,
}

#[pymethods]
impl PyCubicSpline {
    #[new]
    fn new(x: Vec<f64>, y: Vec<f64>) -> PyResult<Self> {
        checked_point_pair(&x, &y).map_err(into_py_err)?;
        Ok(Self {
            inner: scikit::interpolate::CubicSpline::new(&x, &y),
        })
    }

    /// The spline's value at `x`.
    fn eval(&self, x: f64) -> f64 {
        self.inner.eval(x)
    }
}

// ===========================================================================
// optimize
// ===========================================================================

/// `mambalibs.sci.optimize.LinprogResult` — the simplex solver's answer.
#[pyclass(name = "LinprogResult", frozen, module = "mambalibs.sci.optimize")]
#[derive(Debug, Clone)]
pub struct PyLinprogResult {
    inner: scikit::optimize::LinprogResult,
}

#[pymethods]
impl PyLinprogResult {
    /// The optimal decision vector.
    #[getter]
    fn x(&self) -> Vec<f64> {
        self.inner.x.clone()
    }

    /// The objective value at `x`.
    #[getter]
    fn fun(&self) -> f64 {
        self.inner.fun
    }

    /// How many simplex iterations it took.
    #[getter]
    fn nit(&self) -> usize {
        self.inner.nit
    }

    /// Whether the program was solved.
    #[getter]
    fn success(&self) -> bool {
        self.inner.success
    }

    /// What the solver has to say about the outcome.
    #[getter]
    fn message(&self) -> String {
        self.inner.message.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "LinprogResult(x={:?}, fun={}, success={})",
            self.inner.x, self.inner.fun, self.inner.success
        )
    }
}

/// Minimize `c . x` subject to `a_ub . x <= b_ub` and `x >= 0`.
///
/// `a_ub` is the constraint matrix flattened row-major: `len(b_ub)` rows of
/// `len(c)` columns.
#[pyfunction]
fn linprog(c: Vec<f64>, a_ub: Vec<f64>, b_ub: Vec<f64>) -> PyResult<PyLinprogResult> {
    let inner = scikit::optimize::linprog(&c, &a_ub, &b_ub)
        .map_err(|err| into_py_err(SciError::from(err)))?;
    Ok(PyLinprogResult { inner })
}

/// Find a root of `f` between `a` and `b` by Brent's method.
///
/// `f` is any Python callable of one float. A Python exception raised inside
/// it is parked and re-raised here: `scikit::optimize::brentq` takes a plain
/// `Fn(f64) -> f64`, so there is no other way out of the callback.
#[pyfunction]
fn brentq(
    f: &Bound<'_, PyAny>,
    a: f64,
    b: f64,
    tol: f64,
    max_iter: usize,
) -> PyResult<f64> {
    let raised: RefCell<Option<PyErr>> = RefCell::new(None);
    let call = |x: f64| -> f64 {
        if raised.borrow().is_some() {
            return f64::NAN;
        }
        match f.call1((x,)).and_then(|value| value.extract::<f64>()) {
            Ok(value) => value,
            Err(err) => {
                *raised.borrow_mut() = Some(err);
                f64::NAN
            }
        }
    };
    let outcome = scikit::optimize::brentq(call, a, b, tol, max_iter);
    if let Some(err) = raised.borrow_mut().take() {
        return Err(err);
    }
    outcome.map_err(|err| into_py_err(SciError::from(err)))
}

// ===========================================================================
// ts
// ===========================================================================

/// The exponentially weighted moving average of `data`, seeded with its first
/// observation.
#[pyfunction]
fn ewma(data: Vec<f64>, alpha: f64) -> Vec<f64> {
    scikit::ts::ewma(&data, alpha)
}

/// The autocorrelation of `data` at lags `0..=max_lag`.
#[pyfunction]
fn acf(data: Vec<f64>, max_lag: usize) -> PyResult<Vec<f64>> {
    scikit::ts::acf(&data, max_lag).map_err(|err| into_py_err(SciError::from(err)))
}

// ===========================================================================
// spatial
// ===========================================================================

/// `mambalibs.sci.spatial.Neighbor` — one answer from a [`PyKdTree`] query.
#[pyclass(name = "Neighbor", frozen, module = "mambalibs.sci.spatial")]
#[derive(Debug, Clone)]
pub struct PyNeighbor {
    inner: scikit::spatial::Neighbor,
}

#[pymethods]
impl PyNeighbor {
    /// Where this point sits in the cloud the tree was built from.
    #[getter]
    fn index(&self) -> usize {
        self.inner.index
    }

    /// How far it is from the query point.
    #[getter]
    fn distance(&self) -> f64 {
        self.inner.distance
    }

    fn __repr__(&self) -> String {
        format!(
            "Neighbor(index={}, distance={})",
            self.inner.index, self.inner.distance
        )
    }
}

/// The straight-line distance between two points of equal rank.
#[pyfunction]
fn euclidean(a: Vec<f64>, b: Vec<f64>) -> PyResult<f64> {
    checked_point_pair(&a, &b).map_err(into_py_err)?;
    Ok(scikit::spatial::euclidean(&a, &b))
}

/// `mambalibs.sci.spatial.KdTree` — nearest-neighbour queries over a cloud.
#[pyclass(name = "KdTree", frozen, module = "mambalibs.sci.spatial")]
#[derive(Debug)]
pub struct PyKdTree {
    inner: scikit::spatial::KdTree,
    dim: usize,
}

#[pymethods]
impl PyKdTree {
    #[new]
    fn new(points: Vec<Vec<f64>>) -> PyResult<Self> {
        let dim = checked_point_cloud(&points).map_err(into_py_err)?;
        Ok(Self {
            inner: scikit::spatial::KdTree::new(&points),
            dim,
        })
    }

    /// How many points the tree holds.
    #[getter]
    fn size(&self) -> usize {
        self.inner.len()
    }

    /// The rank of the points the tree holds.
    #[getter]
    fn dim(&self) -> usize {
        self.dim
    }

    /// The `k` points nearest `point`, nearest first.
    fn query(&self, point: Vec<f64>, k: usize) -> PyResult<Vec<PyNeighbor>> {
        checked_query_point(self.dim, &point).map_err(into_py_err)?;
        Ok(self
            .inner
            .query(&point, k)
            .into_iter()
            .map(|inner| PyNeighbor { inner })
            .collect())
    }
}

// ===========================================================================
// sparse
// ===========================================================================

/// `mambalibs.sci.sparse.CsrMatrix` — a compressed sparse row matrix.
#[pyclass(name = "CsrMatrix", frozen, module = "mambalibs.sci.sparse")]
#[derive(Debug, Clone)]
pub struct PyCsrMatrix {
    inner: scikit::sparse::CsrMatrix,
}

#[pymethods]
impl PyCsrMatrix {
    /// Compress a row-major dense buffer of `nrows * ncols` elements.
    #[classmethod]
    fn from_dense(
        _cls: &Bound<'_, PyType>,
        dense: Vec<f64>,
        nrows: usize,
        ncols: usize,
    ) -> PyResult<Self> {
        checked_dense(dense.len(), nrows, ncols).map_err(into_py_err)?;
        Ok(Self {
            inner: scikit::sparse::CsrMatrix::from_dense(&dense, nrows, ncols),
        })
    }

    /// How many nonzeros are stored.
    #[getter]
    fn nnz(&self) -> usize {
        self.inner.nnz()
    }

    /// The number of rows.
    #[getter]
    fn nrows(&self) -> usize {
        self.inner.nrows
    }

    /// The number of columns.
    #[getter]
    fn ncols(&self) -> usize {
        self.inner.ncols
    }

    /// The value at `(row, col)`, which is `0.0` where nothing is stored.
    fn get(&self, row: usize, col: usize) -> PyResult<f64> {
        if row >= self.inner.nrows || col >= self.inner.ncols {
            return Err(into_py_err(SciError::new(format!(
                "({row}, {col}) is outside a {}x{} matrix",
                self.inner.nrows, self.inner.ncols
            ))));
        }
        Ok(self.inner.get(row, col))
    }

    /// The same matrix as a row-major dense buffer.
    fn to_dense(&self) -> Vec<f64> {
        self.inner.to_dense()
    }

    /// The matrix-vector product `self . x`.
    fn dot(&self, x: Vec<f64>) -> PyResult<Vec<f64>> {
        checked_dot(self.inner.ncols, x.len()).map_err(into_py_err)?;
        Ok(self.inner.dot(&x))
    }

    fn __repr__(&self) -> String {
        format!(
            "CsrMatrix({}x{}, nnz={})",
            self.inner.nrows,
            self.inner.ncols,
            self.inner.nnz()
        )
    }
}

// ===========================================================================
// integrate
// ===========================================================================

/// The trapezoid-rule integral of samples `y` spaced `dx` apart.
#[pyfunction]
fn trapezoid(y: Vec<f64>, dx: f64) -> f64 {
    scikit::integrate::trapezoid(&y, dx)
}

/// Simpson's-rule integral of samples `y` spaced `dx` apart.
#[pyfunction]
fn simps(y: Vec<f64>, dx: f64) -> f64 {
    scikit::integrate::simps(&y, dx)
}

// ===========================================================================
// module assembly
// ===========================================================================

/// Build one submodule, attach it to `parent` under `name`, and give it its
/// dotted `__name__`.
///
/// The order matters: `add_submodule` binds the child under whatever
/// `__name__` it currently carries, so the short name is what makes
/// `sci.stats` reachable, and the dotted name is set afterwards so the module
/// and the classes inside it agree on where they live.
///
/// The name comes from [`SUBMODULES`], so the module list the wheel publishes
/// and the list `src/tests.rs` judges are the same list.
fn submodule<'py>(
    parent: &Bound<'py, PyModule>,
    name: &str,
) -> PyResult<Bound<'py, PyModule>> {
    let module = PyModule::new(parent.py(), name)?;
    parent.add_submodule(&module)?;
    module.add("__name__", submodule_path(name))?;
    Ok(module)
}

/// The extension module itself. The function name is the module's last
/// component, so this is `mambalibs.sci`.
#[pymodule]
fn sci(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let stats = submodule(m, SUBMODULES[0])?;
    let fft = submodule(m, SUBMODULES[1])?;
    let signal = submodule(m, SUBMODULES[2])?;
    let interpolate = submodule(m, SUBMODULES[3])?;
    let optimize = submodule(m, SUBMODULES[4])?;
    let ts = submodule(m, SUBMODULES[5])?;
    let spatial = submodule(m, SUBMODULES[6])?;
    let sparse = submodule(m, SUBMODULES[7])?;
    let integrate = submodule(m, SUBMODULES[8])?;

    stats.add_class::<PyTestResult>()?;
    stats.add_function(wrap_pyfunction!(median, &stats)?)?;
    stats.add_function(wrap_pyfunction!(ttest_1samp, &stats)?)?;

    fft.add_class::<PyComplex>()?;
    fft.add_function(wrap_pyfunction!(rfftfreq, &fft)?)?;
    fft.add_function(wrap_pyfunction!(rfft, &fft)?)?;

    signal.add_class::<PyConvolveMode>()?;
    signal.add_function(wrap_pyfunction!(hann, &signal)?)?;
    signal.add_function(wrap_pyfunction!(convolve, &signal)?)?;

    interpolate.add_class::<PyInterpKind>()?;
    interpolate.add_class::<PyCubicSpline>()?;
    interpolate.add_function(wrap_pyfunction!(interp1d, &interpolate)?)?;

    optimize.add_class::<PyLinprogResult>()?;
    optimize.add_function(wrap_pyfunction!(linprog, &optimize)?)?;
    optimize.add_function(wrap_pyfunction!(brentq, &optimize)?)?;

    ts.add_function(wrap_pyfunction!(ewma, &ts)?)?;
    ts.add_function(wrap_pyfunction!(acf, &ts)?)?;

    spatial.add_class::<PyNeighbor>()?;
    spatial.add_class::<PyKdTree>()?;
    spatial.add_function(wrap_pyfunction!(euclidean, &spatial)?)?;

    sparse.add_class::<PyCsrMatrix>()?;

    integrate.add_function(wrap_pyfunction!(trapezoid, &integrate)?)?;
    integrate.add_function(wrap_pyfunction!(simps, &integrate)?)?;

    Ok(())
}

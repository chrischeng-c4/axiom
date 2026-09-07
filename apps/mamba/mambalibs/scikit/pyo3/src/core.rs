//! The interpreter-free half of `mambalibs.sci`: the module registry, the
//! refusal type every wrapper raises through, and the argument checks that
//! stand between Python and a `scikit` assertion.
//!
//! Nothing here touches PyO3. That is the point: `scikit` is a plain Rust
//! crate, so the only decisions the binding actually makes are *which*
//! modules it publishes, *what* a refusal says, and *when* a Python argument
//! is unfit to be handed to `scikit` at all. Those three are behaviour, they
//! are testable without an interpreter, and `src/tests.rs` judges them
//! directly. `src/lib.rs` adds names and conversions on top and no rule of
//! its own.
//!
//! # Why the argument checks exist
//!
//! Several `scikit` entry points assert rather than return: `euclidean`
//! (`spatial/distance.rs:5`), `CsrMatrix::from_dense` (`sparse/csr.rs:46`),
//! and `CsrMatrix::dot` (`sparse/csr.rs:147`) each `assert_eq!` on a
//! dimension. An assertion inside an extension module is a panic crossing the
//! FFI boundary, which is not a Python exception a caller can catch — so the
//! same conditions are checked here first and reported as a [`SciError`],
//! which `src/lib.rs` raises as `ValueError`.

use scikit::optimize::OptimizeError;
use scikit::ts::TsError;

/// The nine feature-gated `scikit` modules the wheel publishes, in the order
/// `scikit/src/lib.rs` declares them.
///
/// This list *is* the `full` feature set the crate is built with: the wheel
/// promises all nine rather than the `stats`-only default, and the e2e
/// contract reads exactly these names off the extension module.
pub const SUBMODULES: [&str; 9] = [
    "stats",
    "fft",
    "signal",
    "interpolate",
    "optimize",
    "ts",
    "spatial",
    "sparse",
    "integrate",
];

/// The extension module's own dotted name.
pub const MODULE_NAME: &str = "mambalibs.sci";

/// The dotted name a submodule is published under, e.g. `mambalibs.sci.stats`.
///
/// The submodules are reached as attributes of the extension module, so this
/// name is what a class or function reports as its `__module__` rather than a
/// separate import path.
pub fn submodule_path(name: &str) -> String {
    format!("{MODULE_NAME}.{name}")
}

/// A refusal on its way to Python.
///
/// Every `scikit` failure this binding can meet — an `OptimizeError`, a
/// `TsError`, or an argument the core crate would have asserted on — collapses
/// into one message here, because Python sees exactly one exception type
/// (`ValueError`) for all of them. Keeping the collapse in this module means
/// `src/lib.rs` never decides what a refusal says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SciError {
    message: String,
}

impl SciError {
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

impl From<OptimizeError> for SciError {
    fn from(err: OptimizeError) -> Self {
        SciError::new(err.to_string())
    }
}

impl From<TsError> for SciError {
    fn from(err: TsError) -> Self {
        SciError::new(err.to_string())
    }
}

/// The result of any wrapper that can refuse.
pub type Result<T> = std::result::Result<T, SciError>;

/// Two points can only be measured against each other at equal rank.
///
/// `scikit::spatial::euclidean` asserts this; a Python caller gets a
/// `ValueError` instead.
pub fn checked_point_pair(a: &[f64], b: &[f64]) -> Result<()> {
    if a.len() != b.len() {
        return Err(SciError::new(format!(
            "the two points must have the same number of coordinates, got {} and {}",
            a.len(),
            b.len()
        )));
    }
    Ok(())
}

/// A point cloud must be non-empty and of one rank.
///
/// `KdTree::new` builds its splitting dimension from the first point, so a
/// ragged cloud silently indexes past the end of a shorter one.
pub fn checked_point_cloud(points: &[Vec<f64>]) -> Result<usize> {
    let first = points
        .first()
        .ok_or_else(|| SciError::new("a KdTree needs at least one point"))?;
    let dim = first.len();
    if dim == 0 {
        return Err(SciError::new("a KdTree point must have at least one coordinate"));
    }
    for (i, point) in points.iter().enumerate() {
        if point.len() != dim {
            return Err(SciError::new(format!(
                "every point must have {dim} coordinates, and point {i} has {}",
                point.len()
            )));
        }
    }
    Ok(dim)
}

/// A query point must have the rank of the tree it is asked about.
pub fn checked_query_point(dim: usize, point: &[f64]) -> Result<()> {
    if point.len() != dim {
        return Err(SciError::new(format!(
            "this KdTree holds {dim}-dimensional points, and the query point has {}",
            point.len()
        )));
    }
    Ok(())
}

/// A dense buffer must hold exactly `nrows * ncols` elements.
///
/// `CsrMatrix::from_dense` asserts this. The multiplication is checked too:
/// a Python caller can name dimensions large enough to overflow `usize`, and
/// a wrapped product would make a short buffer look like the right length.
pub fn checked_dense(len: usize, nrows: usize, ncols: usize) -> Result<()> {
    let expected = nrows.checked_mul(ncols).ok_or_else(|| {
        SciError::new(format!(
            "a {nrows}x{ncols} matrix has more elements than this machine can address"
        ))
    })?;
    if len != expected {
        return Err(SciError::new(format!(
            "a {nrows}x{ncols} matrix needs {expected} dense elements, got {len}"
        )));
    }
    Ok(())
}

/// A matrix-vector product needs one vector element per column.
pub fn checked_dot(ncols: usize, len: usize) -> Result<()> {
    if len != ncols {
        return Err(SciError::new(format!(
            "this matrix has {ncols} columns and the vector has {len} elements"
        )));
    }
    Ok(())
}

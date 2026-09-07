//! `mambalibs.array` — arraykit's N-dimensional array surface, as a PyO3
//! `abi3` extension module.
//!
//! # What lives here, and what does not
//!
//! This file is the boundary and nothing else: it names the three classes the
//! wheel publishes (`DType`, `Shape`, `NdArray`), converts Python arguments
//! into Rust values, and maps a [`core::CoreError`] onto the Python exception
//! it deserves. Every decision with behaviour in it — which dtype is which
//! Rust element type, how a Python number becomes an element, what an
//! out-of-range index does — belongs to [`core`], which holds no PyO3 type and
//! is judged by `src/tests.rs` without an interpreter.
//!
//! # Why the module is a namespace member
//!
//! `[tool.maturin] module-name = "mambalibs.array"` with `python-source =
//! "python"` installs this extension as `mambalibs/array.abi3.so` beside no
//! `mambalibs/__init__.py`, so `mambalibs` stays a PEP 420 namespace package
//! and the later kit wheels (`mambalibs.sci`, `mambalibs.plot`) can share it
//! in one environment.
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

use ::core::fmt;

use arraykit::DType;
use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBool;

use crate::core::{shape_of, Array, CoreError, Scalar};

/// `mambalibs.array.DType` — the element type of an [`NdArray`].
///
/// A fieldless `#[pyclass]` enum, so each variant is a class attribute
/// (`DType.Float32`) and two of them compare equal only when they are the same
/// variant.
#[pyclass(name = "DType", eq, eq_int, frozen, module = "mambalibs.array")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PyDType {
    Float32,
    Float64,
    Int32,
    Int64,
    Bool,
}

impl From<PyDType> for DType {
    fn from(value: PyDType) -> Self {
        match value {
            PyDType::Float32 => DType::Float32,
            PyDType::Float64 => DType::Float64,
            PyDType::Int32 => DType::Int32,
            PyDType::Int64 => DType::Int64,
            PyDType::Bool => DType::Bool,
        }
    }
}

impl From<DType> for PyDType {
    fn from(value: DType) -> Self {
        match value {
            DType::Float32 => PyDType::Float32,
            DType::Float64 => PyDType::Float64,
            DType::Int32 => PyDType::Int32,
            DType::Int64 => PyDType::Int64,
            DType::Bool => PyDType::Bool,
        }
    }
}

impl fmt::Display for PyDType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", DType::from(*self).numpy_str())
    }
}

#[pymethods]
impl PyDType {
    /// The size of one element, in bytes.
    #[getter]
    fn size(&self) -> usize {
        DType::from(*self).size()
    }

    /// The NumPy spelling of this dtype, e.g. `float32`.
    fn __str__(&self) -> String {
        self.to_string()
    }

    fn __repr__(&self) -> String {
        format!("DType.{self:?}")
    }
}

/// `mambalibs.array.Shape` — dimensions with their row-major strides.
#[pyclass(name = "Shape", frozen, module = "mambalibs.array")]
#[derive(Debug, Clone)]
pub struct PyShape {
    inner: arraykit::Shape,
}

#[pymethods]
impl PyShape {
    #[new]
    fn new(dims: Vec<usize>) -> Self {
        Self {
            inner: shape_of(dims),
        }
    }

    /// The rank: how many dimensions this shape has.
    #[getter]
    fn ndim(&self) -> usize {
        self.inner.ndim()
    }

    /// The dimensions themselves.
    #[getter]
    fn dims(&self) -> Vec<usize> {
        self.inner.dims().to_vec()
    }

    /// The row-major (C-order) stride of each dimension.
    #[getter]
    fn strides(&self) -> Vec<usize> {
        self.inner.strides().to_vec()
    }

    /// The total number of elements a array of this shape holds; a rank-zero
    /// shape holds one.
    #[getter]
    fn size(&self) -> usize {
        self.inner.size()
    }

    fn __repr__(&self) -> String {
        format!("Shape({:?})", self.inner.dims())
    }
}

/// `mambalibs.array.NdArray` — a dense N-dimensional array of one dtype.
#[pyclass(name = "NdArray", module = "mambalibs.array")]
#[derive(Debug, Clone)]
pub struct PyNdArray {
    inner: Array,
}

#[pymethods]
impl PyNdArray {
    /// An array of `dims` filled with zeroes.
    #[staticmethod]
    fn zeros(dims: Vec<usize>, dtype: PyDType) -> Self {
        Self {
            inner: Array::zeros(dims, dtype.into()),
        }
    }

    /// An array of `dims` filled with ones.
    #[staticmethod]
    fn ones(dims: Vec<usize>, dtype: PyDType) -> Self {
        Self {
            inner: Array::ones(dims, dtype.into()),
        }
    }

    /// An array of `dims` with every element set to `value`.
    #[staticmethod]
    fn full(dims: Vec<usize>, dtype: PyDType, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let value = scalar_from_py(value)?;
        Ok(Self {
            inner: Array::full(dims, dtype.into(), value).map_err(into_py_err)?,
        })
    }

    /// The rank of this array.
    #[getter]
    fn ndim(&self) -> usize {
        self.inner.ndim()
    }

    /// The dimensions of this array.
    #[getter]
    fn dims(&self) -> Vec<usize> {
        self.inner.dims()
    }

    /// The number of elements this array holds.
    #[getter]
    fn size(&self) -> usize {
        self.inner.size()
    }

    /// The element type of this array.
    #[getter]
    fn dtype(&self) -> PyDType {
        self.inner.dtype().into()
    }

    /// The element at `index`, as a Python `bool`, `int`, or `float`
    /// according to the array's dtype.
    fn get<'py>(&self, py: Python<'py>, index: Vec<usize>) -> PyResult<Py<PyAny>> {
        let value = self.inner.get(&index).map_err(into_py_err)?;
        scalar_into_py(py, value)
    }

    /// Store `value` at `index`.
    fn set(&mut self, index: Vec<usize>, value: &Bound<'_, PyAny>) -> PyResult<()> {
        let value = scalar_from_py(value)?;
        self.inner.set(&index, value).map_err(into_py_err)
    }

    /// A new array holding the same elements under `dims`.
    fn reshape(&self, dims: Vec<usize>) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.reshape(dims).map_err(into_py_err)?,
        })
    }

    /// A new rank-one array holding the same elements.
    fn flatten(&self) -> Self {
        Self {
            inner: self.inner.flatten(),
        }
    }

    /// A new array with the dimensions reversed.
    fn transpose(&self) -> Self {
        Self {
            inner: self.inner.transpose(),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "NdArray(dims={:?}, dtype={})",
            self.inner.dims(),
            self.inner.dtype().numpy_str()
        )
    }
}

/// Read a Python number into a [`Scalar`].
///
/// `bool` is tested first: in Python it is a subclass of `int`, so extracting
/// an `i64` from `True` succeeds and would erase the distinction.
fn scalar_from_py(obj: &Bound<'_, PyAny>) -> PyResult<Scalar> {
    if obj.is_instance_of::<PyBool>() {
        return Ok(Scalar::Bool(obj.extract()?));
    }
    if let Ok(value) = obj.extract::<i64>() {
        return Ok(Scalar::Int(value));
    }
    if let Ok(value) = obj.extract::<f64>() {
        return Ok(Scalar::Float(value));
    }
    Err(PyTypeError::new_err(format!(
        "expected a bool, int, or float array element, got {}",
        obj.get_type().name()?
    )))
}

/// Hand a [`Scalar`] back to Python as the type its dtype implies.
fn scalar_into_py(py: Python<'_>, value: Scalar) -> PyResult<Py<PyAny>> {
    Ok(match value {
        Scalar::Bool(v) => v.into_pyobject(py)?.to_owned().into_any().unbind(),
        Scalar::Int(v) => v.into_pyobject(py)?.into_any().unbind(),
        Scalar::Float(v) => v.into_pyobject(py)?.into_any().unbind(),
    })
}

/// Map a core refusal onto the Python exception a caller would expect: an
/// addressing failure is an `IndexError`, everything else a `ValueError`.
fn into_py_err(err: CoreError) -> PyErr {
    match err {
        CoreError::Index(inner) => PyIndexError::new_err(inner.to_string()),
        CoreError::Shape(inner) => PyValueError::new_err(inner.to_string()),
        CoreError::Value(message) => PyValueError::new_err(message),
    }
}

/// The extension module itself. The function name is the module's last
/// component, so this is `mambalibs.array`.
#[pymodule]
fn array(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDType>()?;
    m.add_class::<PyShape>()?;
    m.add_class::<PyNdArray>()?;
    Ok(())
}

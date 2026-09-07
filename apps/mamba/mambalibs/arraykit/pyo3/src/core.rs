//! The dtype-erased array core the Python wrappers are thin over.
//!
//! `arraykit::NdArray<T>` is generic over its element type, but a Python
//! object cannot be generic: `mambalibs.array.NdArray` is one class whose
//! element type is chosen at construction from a `DType`. This module owns
//! that erasure — an [`Array`] is one of the five monomorphisations arraykit
//! supports, [`Scalar`] is the single element value crossing the boundary,
//! and every geometry or element operation dispatches over the variant here.
//!
//! Nothing in this module touches PyO3. That is deliberate: the rules that
//! decide what `mambalibs.array` means — which dtype maps to which Rust type,
//! how a Python number coerces into an element, which arraykit error becomes
//! which Python exception — are testable without an interpreter, and
//! `src/tests.rs` exercises them directly. `src/lib.rs` adds names and
//! argument conversion, and no behaviour of its own.

use arraykit::array::ArrayError;
use arraykit::{DType, NdArray, Shape};

/// One element value crossing the Python boundary.
///
/// Python has three numeric surfaces here — `bool`, `int`, and `float` — and
/// five Rust element types behind them. `Scalar` is the intermediate: a
/// wrapper reads a Python object into it without knowing the array's dtype,
/// and [`Array::set`] coerces it into the element type the array actually
/// holds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scalar {
    Bool(bool),
    Int(i64),
    Float(f64),
}

/// An `arraykit::NdArray<T>` with its element type erased into its [`DType`].
#[derive(Debug, Clone)]
pub enum Array {
    Float32(NdArray<f32>),
    Float64(NdArray<f64>),
    Int32(NdArray<i32>),
    Int64(NdArray<i64>),
    Bool(NdArray<bool>),
}

/// Every way a `mambalibs.array` operation can refuse.
///
/// The variants are kept apart from `ArrayError` so that `src/lib.rs` can map
/// each to the Python exception type it deserves without re-deciding here.
#[derive(Debug, Clone, PartialEq)]
pub enum CoreError {
    /// An index was out of bounds, or of the wrong rank, for the array.
    Index(ArrayError),
    /// A shape or geometry request arraykit refused (a reshape that changes
    /// the element count, for instance).
    Shape(ArrayError),
    /// A Python value that cannot be stored in this array's element type.
    Value(String),
}

/// Sort an `arraykit` failure into the two kinds a caller can distinguish.
///
/// Addressing is the only failure a correct program hits by accident, and it
/// is the one Python spells `IndexError`; everything else arraykit refuses is
/// a statement about geometry.
fn classify(err: ArrayError) -> CoreError {
    match err {
        ArrayError::IndexOutOfBounds { .. } => CoreError::Index(err),
        _ => CoreError::Shape(err),
    }
}

/// The five dtypes, in the order `arraykit::DType` declares them.
pub const ALL_DTYPES: [DType; 5] = [
    DType::Float32,
    DType::Float64,
    DType::Int32,
    DType::Int64,
    DType::Bool,
];

/// Run `$body` against the `NdArray<T>` inside `$expr`, whatever `T` is.
macro_rules! with_array {
    ($expr:expr, $arr:pat => $body:expr) => {
        match $expr {
            Array::Float32($arr) => $body,
            Array::Float64($arr) => $body,
            Array::Int32($arr) => $body,
            Array::Int64($arr) => $body,
            Array::Bool($arr) => $body,
        }
    };
}

/// As [`with_array`], but re-wraps `$body`'s `NdArray<T>` in the same variant,
/// so an operation that preserves the element type cannot silently change it.
macro_rules! map_array {
    ($expr:expr, $arr:pat => $body:expr) => {
        match $expr {
            Array::Float32($arr) => Array::Float32($body),
            Array::Float64($arr) => Array::Float64($body),
            Array::Int32($arr) => Array::Int32($body),
            Array::Int64($arr) => Array::Int64($body),
            Array::Bool($arr) => Array::Bool($body),
        }
    };
}

impl Scalar {
    /// Coerce into `f64`, accepting Python's `bool`/`int`/`float` widening.
    pub fn as_f64(self) -> f64 {
        match self {
            Scalar::Bool(v) => {
                if v {
                    1.0
                } else {
                    0.0
                }
            }
            Scalar::Int(v) => v as f64,
            Scalar::Float(v) => v,
        }
    }

    /// Coerce into `i64`. A float with a fractional part is refused rather
    /// than truncated: silently dropping it would make `set` lossy.
    pub fn as_i64(self) -> Result<i64, CoreError> {
        match self {
            Scalar::Bool(v) => Ok(i64::from(v)),
            Scalar::Int(v) => Ok(v),
            Scalar::Float(v) => {
                if v.is_finite() && v.fract() == 0.0 {
                    Ok(v as i64)
                } else {
                    Err(CoreError::Value(format!(
                        "{v} cannot be stored in an integer array without losing value"
                    )))
                }
            }
        }
    }

    /// Coerce into `bool` by Python's own truthiness for numbers.
    pub fn as_bool(self) -> bool {
        match self {
            Scalar::Bool(v) => v,
            Scalar::Int(v) => v != 0,
            Scalar::Float(v) => v != 0.0,
        }
    }

    /// Coerce into `i32`, refusing a value the narrower type cannot hold.
    fn as_i32(self) -> Result<i32, CoreError> {
        let wide = self.as_i64()?;
        i32::try_from(wide)
            .map_err(|_| CoreError::Value(format!("{wide} is out of range for an int32 array")))
    }
}

impl Array {
    /// A zero-filled array of `dims` with element type `dtype`.
    pub fn zeros(dims: Vec<usize>, dtype: DType) -> Self {
        match dtype {
            DType::Float32 => Array::Float32(NdArray::zeros(dims)),
            DType::Float64 => Array::Float64(NdArray::zeros(dims)),
            DType::Int32 => Array::Int32(NdArray::zeros(dims)),
            DType::Int64 => Array::Int64(NdArray::zeros(dims)),
            DType::Bool => Array::Bool(NdArray::zeros(dims)),
        }
    }

    /// A one-filled array of `dims` with element type `dtype`.
    pub fn ones(dims: Vec<usize>, dtype: DType) -> Self {
        match dtype {
            DType::Float32 => Array::Float32(NdArray::ones(dims)),
            DType::Float64 => Array::Float64(NdArray::ones(dims)),
            DType::Int32 => Array::Int32(NdArray::ones(dims)),
            DType::Int64 => Array::Int64(NdArray::ones(dims)),
            DType::Bool => Array::Bool(NdArray::ones(dims)),
        }
    }

    /// An array of `dims` with every element set to `value`.
    pub fn full(dims: Vec<usize>, dtype: DType, value: Scalar) -> Result<Self, CoreError> {
        Ok(match dtype {
            DType::Float32 => Array::Float32(NdArray::full(dims, value.as_f64() as f32)),
            DType::Float64 => Array::Float64(NdArray::full(dims, value.as_f64())),
            DType::Int32 => Array::Int32(NdArray::full(dims, value.as_i32()?)),
            DType::Int64 => Array::Int64(NdArray::full(dims, value.as_i64()?)),
            DType::Bool => Array::Bool(NdArray::full(dims, value.as_bool())),
        })
    }

    /// The element type this array holds.
    pub fn dtype(&self) -> DType {
        match self {
            Array::Float32(_) => DType::Float32,
            Array::Float64(_) => DType::Float64,
            Array::Int32(_) => DType::Int32,
            Array::Int64(_) => DType::Int64,
            Array::Bool(_) => DType::Bool,
        }
    }

    /// The array's dimensions.
    pub fn dims(&self) -> Vec<usize> {
        with_array!(self, arr => arr.dims().to_vec())
    }

    /// The array's rank.
    pub fn ndim(&self) -> usize {
        with_array!(self, arr => arr.ndim())
    }

    /// The number of elements.
    pub fn size(&self) -> usize {
        with_array!(self, arr => arr.size())
    }

    /// The element at `index`, or an error when the index does not address
    /// one.
    pub fn get(&self, index: &[usize]) -> Result<Scalar, CoreError> {
        match self {
            Array::Float32(arr) => arr.get(index).map(|v| Scalar::Float(f64::from(v))),
            Array::Float64(arr) => arr.get(index).map(Scalar::Float),
            Array::Int32(arr) => arr.get(index).map(|v| Scalar::Int(i64::from(v))),
            Array::Int64(arr) => arr.get(index).map(Scalar::Int),
            Array::Bool(arr) => arr.get(index).map(Scalar::Bool),
        }
        .map_err(classify)
    }

    /// Store `value` at `index`.
    pub fn set(&mut self, index: &[usize], value: Scalar) -> Result<(), CoreError> {
        match self {
            Array::Float32(arr) => arr.set(index, value.as_f64() as f32),
            Array::Float64(arr) => arr.set(index, value.as_f64()),
            Array::Int32(arr) => arr.set(index, value.as_i32()?),
            Array::Int64(arr) => arr.set(index, value.as_i64()?),
            Array::Bool(arr) => arr.set(index, value.as_bool()),
        }
        .map_err(classify)
    }

    /// The same elements under new dimensions.
    pub fn reshape(&self, dims: Vec<usize>) -> Result<Self, CoreError> {
        Ok(match self {
            Array::Float32(arr) => Array::Float32(arr.reshape(dims).map_err(classify)?),
            Array::Float64(arr) => Array::Float64(arr.reshape(dims).map_err(classify)?),
            Array::Int32(arr) => Array::Int32(arr.reshape(dims).map_err(classify)?),
            Array::Int64(arr) => Array::Int64(arr.reshape(dims).map_err(classify)?),
            Array::Bool(arr) => Array::Bool(arr.reshape(dims).map_err(classify)?),
        })
    }

    /// The same elements as a rank-one array.
    pub fn flatten(&self) -> Self {
        map_array!(self, arr => arr.flatten())
    }

    /// The array with its dimensions reversed.
    pub fn transpose(&self) -> Self {
        map_array!(self, arr => arr.transpose())
    }
}

/// The row-major geometry of `dims`, as `mambalibs.array.Shape` reports it.
pub fn shape_of(dims: Vec<usize>) -> Shape {
    Shape::new(dims)
}

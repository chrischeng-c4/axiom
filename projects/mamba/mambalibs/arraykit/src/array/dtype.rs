//! DType system for array element types.
//!
//! Supports core numeric types (f32, f64, i32, i64) and boolean.

use std::fmt;

/// Data type descriptor for array elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DType {
    Float32,
    Float64,
    Int32,
    Int64,
    Bool,
}

impl DType {
    /// Size in bytes of this dtype.
    pub fn size(&self) -> usize {
        match self {
            DType::Float32 => 4,
            DType::Float64 => 8,
            DType::Int32 => 4,
            DType::Int64 => 8,
            DType::Bool => 1,
        }
    }

    /// NumPy-compatible string representation.
    pub fn numpy_str(&self) -> &'static str {
        match self {
            DType::Float32 => "float32",
            DType::Float64 => "float64",
            DType::Int32 => "int32",
            DType::Int64 => "int64",
            DType::Bool => "bool",
        }
    }
}

impl fmt::Display for DType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.numpy_str())
    }
}

/// Trait for types that can be stored in an NdArray.
pub trait ArrayElement: Clone + Copy + Default + PartialEq + fmt::Debug + 'static {
    const DTYPE: DType;

    fn zero() -> Self;
    fn one() -> Self;
}

impl ArrayElement for f32 {
    const DTYPE: DType = DType::Float32;
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
}

impl ArrayElement for f64 {
    const DTYPE: DType = DType::Float64;
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
}

impl ArrayElement for i32 {
    const DTYPE: DType = DType::Int32;
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
}

impl ArrayElement for i64 {
    const DTYPE: DType = DType::Int64;
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
}

impl ArrayElement for bool {
    const DTYPE: DType = DType::Bool;
    fn zero() -> Self {
        false
    }
    fn one() -> Self {
        true
    }
}
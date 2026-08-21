//! # arraykit
//!
//! N-dimensional arrays (NumPy-equivalent). Mamba mambalibs kit.

pub mod array;

// Re-export commonly used types
pub use array::{DType, NdArray, Shape};

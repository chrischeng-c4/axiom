//! Error types for array operations.

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ArrayError {
    #[error("Shape mismatch: expected {expected:?}, got {got:?}")]
    ShapeMismatch {
        expected: Vec<usize>,
        got: Vec<usize>,
    },

    #[error("Cannot broadcast shapes {left:?} and {right:?}")]
    BroadcastError { left: Vec<usize>, right: Vec<usize> },

    #[error("Index {index:?} out of bounds for shape {shape:?}")]
    IndexOutOfBounds {
        index: Vec<usize>,
        shape: Vec<usize>,
    },

    #[error("Invalid axis {axis} for array with {ndim} dimensions")]
    InvalidAxis { axis: usize, ndim: usize },

    #[error("Data length {len} does not match shape {shape:?} (expected {expected})")]
    DataLengthMismatch {
        len: usize,
        shape: Vec<usize>,
        expected: usize,
    },

    #[error("Invalid slice: start={start}, stop={stop}, step={step} for axis length {len}")]
    InvalidSlice {
        start: isize,
        stop: isize,
        step: isize,
        len: usize,
    },

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type Result<T> = std::result::Result<T, ArrayError>;

//! Arithmetic operations for N-dimensional arrays.
//!
//! Implements element-wise operations with NumPy-style broadcasting.

use super::dtype::ArrayElement;
use super::error::Result;
use super::ndarray::NdArray;
use super::shape::Shape;
use std::ops::{Add, Div, Mul, Sub};

/// Trait for numeric operations.
pub trait NumericOps:
    ArrayElement + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Div<Output = Self>
{
}

impl NumericOps for f32 {}
impl NumericOps for f64 {}
impl NumericOps for i32 {}
impl NumericOps for i64 {}

impl<T: NumericOps> NdArray<T> {
    /// Element-wise addition with broadcasting.
    pub fn add(&self, other: &NdArray<T>) -> Result<NdArray<T>> {
        self.binary_op(other, |a, b| a + b)
    }

    /// Element-wise subtraction with broadcasting.
    pub fn sub(&self, other: &NdArray<T>) -> Result<NdArray<T>> {
        self.binary_op(other, |a, b| a - b)
    }

    /// Element-wise multiplication with broadcasting.
    pub fn mul(&self, other: &NdArray<T>) -> Result<NdArray<T>> {
        self.binary_op(other, |a, b| a * b)
    }

    /// Element-wise division with broadcasting.
    pub fn div(&self, other: &NdArray<T>) -> Result<NdArray<T>> {
        self.binary_op(other, |a, b| a / b)
    }

    /// Apply a binary operation with broadcasting.
    fn binary_op<F>(&self, other: &NdArray<T>, op: F) -> Result<NdArray<T>>
    where
        F: Fn(T, T) -> T,
    {
        let broadcast_shape = Shape::broadcast(self.shape(), other.shape())?;

        // Broadcast both arrays to the target shape
        let a = self.broadcast_to(&broadcast_shape)?;
        let b = other.broadcast_to(&broadcast_shape)?;

        // Apply operation element-wise
        let data: Vec<T> = a
            .data()
            .iter()
            .zip(b.data())
            .map(|(&x, &y)| op(x, y))
            .collect();

        NdArray::new(data, broadcast_shape)
    }

    /// Add a scalar to all elements.
    pub fn add_scalar(&self, scalar: T) -> NdArray<T> {
        let data: Vec<T> = self.data().iter().map(|&x| x + scalar).collect();
        NdArray {
            data,
            shape: self.shape().clone(),
        }
    }

    /// Multiply all elements by a scalar.
    pub fn mul_scalar(&self, scalar: T) -> NdArray<T> {
        let data: Vec<T> = self.data().iter().map(|&x| x * scalar).collect();
        NdArray {
            data,
            shape: self.shape().clone(),
        }
    }
}

// Implement std::ops traits for ergonomic syntax

impl<T: NumericOps> Add for &NdArray<T> {
    type Output = Result<NdArray<T>>;

    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}

impl<T: NumericOps> Sub for &NdArray<T> {
    type Output = Result<NdArray<T>>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.sub(rhs)
    }
}

impl<T: NumericOps> Mul for &NdArray<T> {
    type Output = Result<NdArray<T>>;

    fn mul(self, rhs: Self) -> Self::Output {
        self.mul(rhs)
    }
}

impl<T: NumericOps> Div for &NdArray<T> {
    type Output = Result<NdArray<T>>;

    fn div(self, rhs: Self) -> Self::Output {
        self.div(rhs)
    }
}

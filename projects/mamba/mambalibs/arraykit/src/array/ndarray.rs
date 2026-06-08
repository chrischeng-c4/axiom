//! Core N-dimensional array implementation.

use super::dtype::ArrayElement;
use super::error::{ArrayError, Result};
use super::shape::Shape;
use super::slice::{AxisSlice, SliceInfo};
use std::fmt;

/// N-dimensional array with contiguous storage and NumPy-like semantics.
#[derive(Clone)]
pub struct NdArray<T: ArrayElement> {
    pub(crate) data: Vec<T>,
    pub(crate) shape: Shape,
}

impl<T: ArrayElement> NdArray<T> {
    /// Create an array from data and shape.
    pub fn new(data: Vec<T>, shape: impl Into<Shape>) -> Result<Self> {
        let shape = shape.into();
        let expected = shape.size();
        if data.len() != expected {
            return Err(ArrayError::DataLengthMismatch {
                len: data.len(),
                shape: shape.dims().to_vec(),
                expected,
            });
        }
        Ok(Self { data, shape })
    }

    /// Create an array filled with zeros.
    pub fn zeros(shape: impl Into<Shape>) -> Self {
        let shape = shape.into();
        let size = shape.size();
        Self {
            data: vec![T::zero(); size],
            shape,
        }
    }

    /// Create an array filled with ones.
    pub fn ones(shape: impl Into<Shape>) -> Self {
        let shape = shape.into();
        let size = shape.size();
        Self {
            data: vec![T::one(); size],
            shape,
        }
    }

    /// Create an array filled with a specific value.
    pub fn full(shape: impl Into<Shape>, value: T) -> Self {
        let shape = shape.into();
        let size = shape.size();
        Self {
            data: vec![value; size],
            shape,
        }
    }

    /// Create a 1D array from a range.
    ///
    /// Supports both positive and negative steps.
    /// Returns an empty array if step is zero or if the range is invalid
    /// (e.g., start >= stop with positive step, or start <= stop with negative step).
    pub fn arange(start: T, stop: T, step: T) -> Self
    where
        T: PartialOrd + std::ops::Add<Output = T> + std::ops::Sub<Output = T>,
    {
        let mut data = Vec::new();
        let zero = T::zero();

        // Handle step == 0: return empty array
        if step == zero {
            return Self {
                data,
                shape: Shape::new(vec![0]),
            };
        }

        let mut current = start;

        if step > zero {
            // Positive step: iterate while current < stop
            while current < stop {
                data.push(current);
                current = current + step;
            }
        } else {
            // Negative step: iterate while current > stop
            while current > stop {
                data.push(current);
                current = current + step;
            }
        }

        let len = data.len();
        Self {
            data,
            shape: Shape::new(vec![len]),
        }
    }

    /// Create a 2D identity matrix.
    ///
    /// # Example
    /// ```ignore
    /// let eye = NdArray::<f64>::eye(3);
    /// // [[1, 0, 0], [0, 1, 0], [0, 0, 1]]
    /// ```
    pub fn eye(n: usize) -> Self {
        let mut data = vec![T::zero(); n * n];
        for i in 0..n {
            data[i * n + i] = T::one();
        }
        Self {
            data,
            shape: Shape::new(vec![n, n]),
        }
    }

    /// Create an identity matrix (alias for eye).
    pub fn identity(n: usize) -> Self {
        Self::eye(n)
    }

    /// Create a diagonal matrix from a 1D array, or extract diagonal from 2D.
    pub fn diag(arr: &NdArray<T>) -> Result<Self> {
        match arr.ndim() {
            1 => {
                // Create diagonal matrix from 1D array
                let n = arr.size();
                let mut data = vec![T::zero(); n * n];
                for i in 0..n {
                    data[i * n + i] = arr.data[i];
                }
                Ok(Self {
                    data,
                    shape: Shape::new(vec![n, n]),
                })
            }
            2 => {
                // Extract diagonal from 2D matrix
                let dims = arr.dims();
                let n = dims[0].min(dims[1]);
                let mut data = Vec::with_capacity(n);
                for i in 0..n {
                    data.push(arr.data[i * dims[1] + i]);
                }
                Ok(Self {
                    data,
                    shape: Shape::new(vec![n]),
                })
            }
            _ => Err(ArrayError::ShapeMismatch {
                expected: vec![1],
                got: arr.dims().to_vec(),
            }),
        }
    }

    /// Get the shape of this array.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// Get dimensions as a slice.
    pub fn dims(&self) -> &[usize] {
        self.shape.dims()
    }

    /// Get the number of dimensions (rank).
    pub fn ndim(&self) -> usize {
        self.shape.ndim()
    }

    /// Get the total number of elements.
    pub fn size(&self) -> usize {
        self.shape.size()
    }

    /// Get the underlying data as a slice.
    pub fn data(&self) -> &[T] {
        &self.data
    }

    /// Get mutable access to the underlying data.
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Get element at multi-dimensional index.
    pub fn get(&self, index: &[usize]) -> Result<T> {
        let offset = self.shape.offset(index)?;
        Ok(self.data[offset])
    }

    /// Set element at multi-dimensional index.
    pub fn set(&mut self, index: &[usize], value: T) -> Result<()> {
        let offset = self.shape.offset(index)?;
        self.data[offset] = value;
        Ok(())
    }

    /// Reshape the array (must have same total size).
    pub fn reshape(&self, new_shape: impl Into<Shape>) -> Result<Self> {
        let new_shape = new_shape.into();
        if new_shape.size() != self.size() {
            return Err(ArrayError::ShapeMismatch {
                expected: new_shape.dims().to_vec(),
                got: self.shape.dims().to_vec(),
            });
        }
        Ok(Self {
            data: self.data.clone(),
            shape: new_shape,
        })
    }

    /// Flatten to a 1D array.
    pub fn flatten(&self) -> Self {
        Self {
            data: self.data.clone(),
            shape: Shape::new(vec![self.size()]),
        }
    }

    /// Slice the array.
    pub fn slice(&self, info: &SliceInfo) -> Result<Self> {
        let output_shape = info.output_shape(self.dims())?;
        let output_size: usize = output_shape.iter().product();

        let mut result = Vec::with_capacity(output_size);
        self.slice_recursive(info.slices(), &mut vec![], &mut result)?;

        Ok(Self {
            data: result,
            shape: Shape::new(output_shape),
        })
    }

    fn slice_recursive(
        &self,
        slices: &[AxisSlice],
        current_index: &mut Vec<usize>,
        result: &mut Vec<T>,
    ) -> Result<()> {
        let axis = current_index.len();

        if axis >= self.ndim() {
            // Base case: collect element
            let offset = self.shape.offset(current_index)?;
            result.push(self.data[offset]);
            return Ok(());
        }

        let slice = if axis < slices.len() {
            slices[axis]
        } else {
            AxisSlice::Full
        };

        let dim_len = self.dims()[axis];
        let norm = slice.normalize(dim_len)?;

        if norm.is_index {
            // Index: single element, recurse
            current_index.push(norm.start);
            self.slice_recursive(slices, current_index, result)?;
            current_index.pop();
        } else if norm.step > 0 {
            // Positive step
            let mut i = norm.start as isize;
            while i < norm.stop {
                current_index.push(i as usize);
                self.slice_recursive(slices, current_index, result)?;
                current_index.pop();
                i += norm.step;
            }
        } else {
            // Negative step
            let mut i = norm.start as isize;
            while i > norm.stop {
                current_index.push(i as usize);
                self.slice_recursive(slices, current_index, result)?;
                current_index.pop();
                i += norm.step;
            }
        }

        Ok(())
    }

    /// Transpose the array (reverse all axes).
    ///
    /// For 2D arrays, this swaps rows and columns.
    /// For higher dimensions, this reverses the order of all axes.
    pub fn transpose(&self) -> Self {
        if self.ndim() <= 1 {
            return self.clone();
        }

        let old_dims = self.dims();
        let new_dims: Vec<usize> = old_dims.iter().rev().cloned().collect();
        let new_shape = Shape::new(new_dims.clone());

        let mut result = Vec::with_capacity(self.size());
        let mut new_index = vec![0usize; self.ndim()];

        for _ in 0..self.size() {
            // Map new_index to old_index (reverse the axes)
            let old_index: Vec<usize> = new_index.iter().rev().cloned().collect();
            let offset = self.shape.offset(&old_index).unwrap_or(0);
            result.push(self.data[offset]);

            // Increment new_index
            for j in (0..new_dims.len()).rev() {
                new_index[j] += 1;
                if new_index[j] < new_dims[j] {
                    break;
                }
                new_index[j] = 0;
            }
        }

        Self {
            data: result,
            shape: new_shape,
        }
    }

    /// Transpose with specific axis permutation.
    ///
    /// # Arguments
    /// * `axes` - The new order of axes. Must be a permutation of 0..ndim.
    pub fn transpose_axes(&self, axes: &[usize]) -> Result<Self> {
        if axes.len() != self.ndim() {
            return Err(ArrayError::ShapeMismatch {
                expected: self.dims().to_vec(),
                got: axes.to_vec(),
            });
        }

        // Validate axes is a valid permutation
        let mut seen = vec![false; self.ndim()];
        for &ax in axes {
            if ax >= self.ndim() || seen[ax] {
                return Err(ArrayError::ShapeMismatch {
                    expected: (0..self.ndim()).collect(),
                    got: axes.to_vec(),
                });
            }
            seen[ax] = true;
        }

        let old_dims = self.dims();
        let new_dims: Vec<usize> = axes.iter().map(|&ax| old_dims[ax]).collect();
        let new_shape = Shape::new(new_dims.clone());

        let mut result = Vec::with_capacity(self.size());
        let mut new_index = vec![0usize; self.ndim()];

        for _ in 0..self.size() {
            // Map new_index to old_index using axes permutation
            let mut old_index = vec![0usize; self.ndim()];
            for (new_ax, &old_ax) in axes.iter().enumerate() {
                old_index[old_ax] = new_index[new_ax];
            }
            let offset = self.shape.offset(&old_index).unwrap_or(0);
            result.push(self.data[offset]);

            // Increment new_index
            for j in (0..new_dims.len()).rev() {
                new_index[j] += 1;
                if new_index[j] < new_dims[j] {
                    break;
                }
                new_index[j] = 0;
            }
        }

        Ok(Self {
            data: result,
            shape: new_shape,
        })
    }

    /// Broadcast this array to a target shape.
    pub fn broadcast_to(&self, target: &Shape) -> Result<Self> {
        let broadcast_strides = self.shape.broadcast_strides(target)?;
        let mut result = Vec::with_capacity(target.size());

        let mut index = vec![0usize; target.ndim()];
        for _ in 0..target.size() {
            let offset: usize = index
                .iter()
                .zip(&broadcast_strides)
                .map(|(i, s)| i * s)
                .sum();
            result.push(self.data[offset]);

            // Increment index
            for j in (0..target.ndim()).rev() {
                index[j] += 1;
                if index[j] < target.dims()[j] {
                    break;
                }
                index[j] = 0;
            }
        }

        Ok(Self {
            data: result,
            shape: target.clone(),
        })
    }
}

impl<T: ArrayElement + fmt::Display> fmt::Debug for NdArray<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NdArray(shape={:?}, dtype={})", self.dims(), T::DTYPE)
    }
}

impl<T: ArrayElement + fmt::Display> fmt::Display for NdArray<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_recursive(f, 0, &mut vec![])
    }
}

impl<T: ArrayElement + fmt::Display> NdArray<T> {
    fn fmt_recursive(
        &self,
        f: &mut fmt::Formatter<'_>,
        depth: usize,
        index: &mut Vec<usize>,
    ) -> fmt::Result {
        if depth == self.ndim() {
            let val = self.get(index).map_err(|_| fmt::Error)?;
            write!(f, "{}", val)?;
        } else if depth == self.ndim() - 1 {
            // Last dimension: print as row
            write!(f, "[")?;
            for i in 0..self.dims()[depth] {
                if i > 0 {
                    write!(f, ", ")?;
                }
                index.push(i);
                self.fmt_recursive(f, depth + 1, index)?;
                index.pop();
            }
            write!(f, "]")?;
        } else {
            write!(f, "[")?;
            for i in 0..self.dims()[depth] {
                if i > 0 {
                    write!(f, ",\n{}", " ".repeat(depth + 1))?;
                }
                index.push(i);
                self.fmt_recursive(f, depth + 1, index)?;
                index.pop();
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}
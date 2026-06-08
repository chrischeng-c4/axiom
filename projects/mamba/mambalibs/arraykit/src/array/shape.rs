//! Shape and stride utilities for N-dimensional arrays.

use super::error::{ArrayError, Result};

/// Shape descriptor for N-dimensional arrays.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shape {
    dims: Vec<usize>,
    strides: Vec<usize>,
}

impl Shape {
    /// Create a new shape with row-major (C-style) strides.
    pub fn new(dims: Vec<usize>) -> Self {
        let strides = Self::compute_strides(&dims);
        Self { dims, strides }
    }

    /// Create a shape with custom strides.
    pub fn with_strides(dims: Vec<usize>, strides: Vec<usize>) -> Result<Self> {
        if dims.len() != strides.len() {
            return Err(ArrayError::ShapeMismatch {
                expected: dims.clone(),
                got: vec![strides.len()],
            });
        }
        Ok(Self { dims, strides })
    }

    /// Compute row-major strides for given dimensions.
    fn compute_strides(dims: &[usize]) -> Vec<usize> {
        if dims.is_empty() {
            return vec![];
        }
        let mut strides = vec![1; dims.len()];
        for i in (0..dims.len() - 1).rev() {
            strides[i] = strides[i + 1] * dims[i + 1];
        }
        strides
    }

    /// Number of dimensions (rank).
    pub fn ndim(&self) -> usize {
        self.dims.len()
    }

    /// Dimensions as a slice.
    pub fn dims(&self) -> &[usize] {
        &self.dims
    }

    /// Strides as a slice.
    pub fn strides(&self) -> &[usize] {
        &self.strides
    }

    /// Total number of elements.
    pub fn size(&self) -> usize {
        self.dims.iter().product()
    }

    /// Check if a multi-dimensional index is valid.
    pub fn contains(&self, index: &[usize]) -> bool {
        if index.len() != self.dims.len() {
            return false;
        }
        index.iter().zip(&self.dims).all(|(i, d)| i < d)
    }

    /// Convert multi-dimensional index to linear offset.
    pub fn offset(&self, index: &[usize]) -> Result<usize> {
        if !self.contains(index) {
            return Err(ArrayError::IndexOutOfBounds {
                index: index.to_vec(),
                shape: self.dims.clone(),
            });
        }
        Ok(index.iter().zip(&self.strides).map(|(i, s)| i * s).sum())
    }

    /// Compute the broadcast shape of two shapes following NumPy rules.
    pub fn broadcast(a: &Shape, b: &Shape) -> Result<Shape> {
        let max_ndim = a.ndim().max(b.ndim());
        let mut result = vec![0; max_ndim];

        for i in 0..max_ndim {
            let a_dim = if i < max_ndim - a.ndim() {
                1
            } else {
                a.dims[i - (max_ndim - a.ndim())]
            };
            let b_dim = if i < max_ndim - b.ndim() {
                1
            } else {
                b.dims[i - (max_ndim - b.ndim())]
            };

            if a_dim == b_dim {
                result[i] = a_dim;
            } else if a_dim == 1 {
                result[i] = b_dim;
            } else if b_dim == 1 {
                result[i] = a_dim;
            } else {
                return Err(ArrayError::BroadcastError {
                    left: a.dims.clone(),
                    right: b.dims.clone(),
                });
            }
        }

        Ok(Shape::new(result))
    }

    /// Get broadcast strides for this shape to match target shape.
    pub fn broadcast_strides(&self, target: &Shape) -> Result<Vec<usize>> {
        // Guard: target must have >= rank than self
        if target.ndim() < self.ndim() {
            return Err(ArrayError::BroadcastError {
                left: self.dims.clone(),
                right: target.dims.clone(),
            });
        }

        let offset = target.ndim() - self.ndim();
        let mut result = vec![0; target.ndim()];

        for i in 0..target.ndim() {
            if i < offset {
                result[i] = 0; // Broadcast dimension
            } else {
                let self_idx = i - offset;
                if self.dims[self_idx] == target.dims[i] {
                    result[i] = self.strides[self_idx];
                } else if self.dims[self_idx] == 1 {
                    result[i] = 0; // Broadcast this dimension
                } else {
                    return Err(ArrayError::BroadcastError {
                        left: self.dims.clone(),
                        right: target.dims.clone(),
                    });
                }
            }
        }

        Ok(result)
    }
}

impl From<Vec<usize>> for Shape {
    fn from(dims: Vec<usize>) -> Self {
        Shape::new(dims)
    }
}

impl From<&[usize]> for Shape {
    fn from(dims: &[usize]) -> Self {
        Shape::new(dims.to_vec())
    }
}
//! Linear algebra operations for N-dimensional arrays.
//!
//! Implements matrix multiplication, dot product, and related operations.

use super::dtype::ArrayElement;
use super::error::{ArrayError, Result};
use super::ndarray::NdArray;
use std::ops::{Add, Mul};

/// Trait for types that support linear algebra operations.
pub trait LinalgOps: ArrayElement + Add<Output = Self> + Mul<Output = Self> {}

impl LinalgOps for f32 {}
impl LinalgOps for f64 {}
impl LinalgOps for i32 {}
impl LinalgOps for i64 {}

impl<T: LinalgOps> NdArray<T> {
    /// Matrix multiplication for 2D arrays.
    ///
    /// # Example
    /// ```ignore
    /// let a = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    /// let b = NdArray::new(vec![5, 6, 7, 8], vec![2, 2]).unwrap();
    /// let c = a.matmul(&b).unwrap();
    /// // [[19, 22], [43, 50]]
    /// ```
    pub fn matmul(&self, other: &NdArray<T>) -> Result<NdArray<T>> {
        if self.ndim() != 2 || other.ndim() != 2 {
            return Err(ArrayError::ShapeMismatch {
                expected: vec![2],
                got: vec![self.ndim(), other.ndim()],
            });
        }

        let (m, k1) = (self.dims()[0], self.dims()[1]);
        let (k2, n) = (other.dims()[0], other.dims()[1]);

        if k1 != k2 {
            return Err(ArrayError::ShapeMismatch {
                expected: vec![m, k1],
                got: vec![k2, n],
            });
        }

        let k = k1;
        let mut result = vec![T::zero(); m * n];

        for i in 0..m {
            for j in 0..n {
                let mut sum = T::zero();
                for l in 0..k {
                    sum = sum + self.data[i * k + l] * other.data[l * n + j];
                }
                result[i * n + j] = sum;
            }
        }

        NdArray::new(result, vec![m, n])
    }

    /// Dot product for 1D arrays, matrix multiplication for 2D.
    pub fn dot(&self, other: &NdArray<T>) -> Result<NdArray<T>> {
        match (self.ndim(), other.ndim()) {
            (1, 1) => {
                // Vector dot product -> scalar (returned as 0D-like 1-element array)
                if self.size() != other.size() {
                    return Err(ArrayError::ShapeMismatch {
                        expected: self.dims().to_vec(),
                        got: other.dims().to_vec(),
                    });
                }
                let sum = self
                    .data
                    .iter()
                    .zip(other.data.iter())
                    .fold(T::zero(), |acc, (&a, &b)| acc + a * b);
                NdArray::new(vec![sum], vec![1])
            }
            (2, 2) => self.matmul(other),
            (2, 1) => {
                // Matrix-vector: (m, k) @ (k,) -> (m,)
                let (m, k) = (self.dims()[0], self.dims()[1]);
                if k != other.size() {
                    return Err(ArrayError::ShapeMismatch {
                        expected: vec![k],
                        got: other.dims().to_vec(),
                    });
                }
                let mut result = vec![T::zero(); m];
                for i in 0..m {
                    let mut sum = T::zero();
                    for j in 0..k {
                        sum = sum + self.data[i * k + j] * other.data[j];
                    }
                    result[i] = sum;
                }
                NdArray::new(result, vec![m])
            }
            _ => Err(ArrayError::ShapeMismatch {
                expected: vec![1, 2],
                got: vec![self.ndim(), other.ndim()],
            }),
        }
    }

    /// Outer product of two 1D arrays.
    ///
    /// For vectors a and b, returns matrix where out[i,j] = a[i] * b[j].
    pub fn outer(&self, other: &NdArray<T>) -> Result<NdArray<T>> {
        if self.ndim() != 1 || other.ndim() != 1 {
            return Err(ArrayError::ShapeMismatch {
                expected: vec![1],
                got: vec![self.ndim(), other.ndim()],
            });
        }

        let m = self.size();
        let n = other.size();
        let mut result = vec![T::zero(); m * n];

        for i in 0..m {
            for j in 0..n {
                result[i * n + j] = self.data[i] * other.data[j];
            }
        }

        NdArray::new(result, vec![m, n])
    }

    /// Inner product of two arrays.
    ///
    /// For 1D arrays, this is equivalent to dot product.
    pub fn inner(&self, other: &NdArray<T>) -> Result<NdArray<T>> {
        if self.ndim() == 1 && other.ndim() == 1 {
            self.dot(other)
        } else {
            // For higher dimensions, contract over last axis of self and other
            Err(ArrayError::ShapeMismatch {
                expected: vec![1],
                got: vec![self.ndim(), other.ndim()],
            })
        }
    }

    /// Trace of a matrix (sum of diagonal elements).
    pub fn trace(&self) -> Result<T> {
        if self.ndim() != 2 {
            return Err(ArrayError::ShapeMismatch {
                expected: vec![2],
                got: vec![self.ndim()],
            });
        }

        let (m, n) = (self.dims()[0], self.dims()[1]);
        let min_dim = m.min(n);
        let mut sum = T::zero();
        for i in 0..min_dim {
            sum = sum + self.data[i * n + i];
        }
        Ok(sum)
    }
}

// Advanced linear algebra for f64
impl NdArray<f64> {
    /// Matrix determinant (for square matrices).
    pub fn det(&self) -> Result<f64> {
        if self.ndim() != 2 {
            return Err(ArrayError::ShapeMismatch {
                expected: vec![2],
                got: vec![self.ndim()],
            });
        }

        let (m, n) = (self.dims()[0], self.dims()[1]);
        if m != n {
            return Err(ArrayError::ShapeMismatch {
                expected: vec![m, m],
                got: vec![m, n],
            });
        }

        if n == 1 {
            return Ok(self.data[0]);
        }

        if n == 2 {
            return Ok(self.data[0] * self.data[3] - self.data[1] * self.data[2]);
        }

        // LU decomposition for larger matrices
        let (lu, _piv, sign) = self.lu_decompose()?;
        let mut det = sign as f64;
        for i in 0..n {
            det *= lu.data[i * n + i];
        }
        Ok(det)
    }

    /// LU decomposition with partial pivoting.
    /// Returns (LU matrix, pivot indices, sign of permutation).
    fn lu_decompose(&self) -> Result<(NdArray<f64>, Vec<usize>, i32)> {
        let n = self.dims()[0];
        let mut lu = self.data.clone();
        let mut piv: Vec<usize> = (0..n).collect();
        let mut sign = 1i32;

        for k in 0..n {
            // Find pivot
            let mut max_val = lu[k * n + k].abs();
            let mut max_idx = k;
            for i in k + 1..n {
                let val = lu[i * n + k].abs();
                if val > max_val {
                    max_val = val;
                    max_idx = i;
                }
            }

            // Swap rows if needed
            if max_idx != k {
                for j in 0..n {
                    lu.swap(k * n + j, max_idx * n + j);
                }
                piv.swap(k, max_idx);
                sign = -sign;
            }

            if lu[k * n + k].abs() < 1e-12 {
                continue; // Singular matrix
            }

            // Eliminate
            for i in k + 1..n {
                lu[i * n + k] /= lu[k * n + k];
                for j in k + 1..n {
                    lu[i * n + j] -= lu[i * n + k] * lu[k * n + j];
                }
            }
        }

        Ok((NdArray::new(lu, vec![n, n])?, piv, sign))
    }

    /// Matrix inverse.
    pub fn inv(&self) -> Result<NdArray<f64>> {
        if self.ndim() != 2 {
            return Err(ArrayError::ShapeMismatch {
                expected: vec![2],
                got: vec![self.ndim()],
            });
        }

        let (m, n) = (self.dims()[0], self.dims()[1]);
        if m != n {
            return Err(ArrayError::ShapeMismatch {
                expected: vec![m, m],
                got: vec![m, n],
            });
        }

        // Create augmented matrix [A | I]
        let mut aug = vec![0.0; n * 2 * n];
        for i in 0..n {
            for j in 0..n {
                aug[i * 2 * n + j] = self.data[i * n + j];
            }
            aug[i * 2 * n + n + i] = 1.0;
        }

        // Gaussian elimination with partial pivoting
        for k in 0..n {
            // Find pivot
            let mut max_idx = k;
            let mut max_val = aug[k * 2 * n + k].abs();
            for i in k + 1..n {
                let val = aug[i * 2 * n + k].abs();
                if val > max_val {
                    max_val = val;
                    max_idx = i;
                }
            }

            if max_val < 1e-12 {
                return Err(ArrayError::InvalidOperation("matrix is singular".into()));
            }

            // Swap rows
            if max_idx != k {
                for j in 0..2 * n {
                    aug.swap(k * 2 * n + j, max_idx * 2 * n + j);
                }
            }

            // Scale pivot row
            let pivot = aug[k * 2 * n + k];
            for j in 0..2 * n {
                aug[k * 2 * n + j] /= pivot;
            }

            // Eliminate column
            for i in 0..n {
                if i != k {
                    let factor = aug[i * 2 * n + k];
                    for j in 0..2 * n {
                        aug[i * 2 * n + j] -= factor * aug[k * 2 * n + j];
                    }
                }
            }
        }

        // Extract inverse
        let mut result = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                result[i * n + j] = aug[i * 2 * n + n + j];
            }
        }

        NdArray::new(result, vec![n, n])
    }

    /// Solve linear system Ax = b.
    pub fn solve(&self, b: &NdArray<f64>) -> Result<NdArray<f64>> {
        if self.ndim() != 2 || b.ndim() != 1 {
            return Err(ArrayError::ShapeMismatch {
                expected: vec![2, 1],
                got: vec![self.ndim(), b.ndim()],
            });
        }

        let n = self.dims()[0];
        if self.dims()[1] != n || b.size() != n {
            return Err(ArrayError::ShapeMismatch {
                expected: vec![n, n],
                got: self.dims().to_vec(),
            });
        }

        // Compute inverse and multiply
        let inv = self.inv()?;
        inv.dot(b)
    }

    /// Compute norm of array.
    pub fn norm(&self, ord: Option<f64>) -> f64 {
        let ord = ord.unwrap_or(2.0);

        if ord == f64::INFINITY {
            self.data.iter().fold(0.0_f64, |a, &b| a.max(b.abs()))
        } else if ord == f64::NEG_INFINITY {
            self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b.abs()))
        } else if ord == 1.0 {
            self.data.iter().map(|&x| x.abs()).sum()
        } else if ord == 2.0 {
            self.data.iter().map(|&x| x * x).sum::<f64>().sqrt()
        } else {
            self.data
                .iter()
                .map(|&x| x.abs().powf(ord))
                .sum::<f64>()
                .powf(1.0 / ord)
        }
    }
}

//! Array manipulation operations.
//!
//! Implements stacking, concatenating, splitting, sorting, and utility functions.

use super::dtype::ArrayElement;
use super::error::{ArrayError, Result};
use super::ndarray::NdArray;
use super::shape::Shape;
use std::collections::HashSet;

impl<T: ArrayElement> NdArray<T> {
    /// Concatenate arrays along an existing axis.
    pub fn concatenate(arrays: &[&NdArray<T>], axis: usize) -> Result<NdArray<T>> {
        if arrays.is_empty() {
            return Err(ArrayError::InvalidOperation("empty array list".into()));
        }

        let first = arrays[0];
        let ndim = first.ndim();

        if axis >= ndim {
            return Err(ArrayError::IndexOutOfBounds {
                index: vec![axis],
                shape: first.dims().to_vec(),
            });
        }

        // Verify all arrays have same shape except along axis
        for arr in &arrays[1..] {
            if arr.ndim() != ndim {
                return Err(ArrayError::ShapeMismatch {
                    expected: first.dims().to_vec(),
                    got: arr.dims().to_vec(),
                });
            }
            for (i, (&d1, &d2)) in first.dims().iter().zip(arr.dims()).enumerate() {
                if i != axis && d1 != d2 {
                    return Err(ArrayError::ShapeMismatch {
                        expected: first.dims().to_vec(),
                        got: arr.dims().to_vec(),
                    });
                }
            }
        }

        // Calculate new shape
        let mut new_dims = first.dims().to_vec();
        new_dims[axis] = arrays.iter().map(|a| a.dims()[axis]).sum();

        // Build result
        let total_size: usize = new_dims.iter().product();
        let mut result = Vec::with_capacity(total_size);

        // For simple 1D case
        if ndim == 1 {
            for arr in arrays {
                result.extend_from_slice(&arr.data);
            }
            return NdArray::new(result, new_dims);
        }

        // For higher dimensions, iterate through all indices
        let _out_shape = Shape::new(new_dims.clone());
        let mut out_index = vec![0usize; ndim];

        for _ in 0..total_size {
            // Find which array and local index this maps to
            let mut axis_pos = out_index[axis];
            let mut arr_idx = 0;
            for (i, arr) in arrays.iter().enumerate() {
                let arr_axis_len = arr.dims()[axis];
                if axis_pos < arr_axis_len {
                    arr_idx = i;
                    break;
                }
                axis_pos -= arr_axis_len;
            }

            let mut local_index = out_index.clone();
            local_index[axis] = axis_pos;

            let offset = arrays[arr_idx].shape.offset(&local_index)?;
            result.push(arrays[arr_idx].data[offset]);

            // Increment index
            for j in (0..ndim).rev() {
                out_index[j] += 1;
                if out_index[j] < new_dims[j] {
                    break;
                }
                out_index[j] = 0;
            }
        }

        NdArray::new(result, new_dims)
    }

    /// Stack arrays along a new axis.
    pub fn stack(arrays: &[&NdArray<T>], axis: usize) -> Result<NdArray<T>> {
        if arrays.is_empty() {
            return Err(ArrayError::InvalidOperation("empty array list".into()));
        }

        let first = arrays[0];

        // All arrays must have same shape
        for arr in &arrays[1..] {
            if arr.dims() != first.dims() {
                return Err(ArrayError::ShapeMismatch {
                    expected: first.dims().to_vec(),
                    got: arr.dims().to_vec(),
                });
            }
        }

        // Insert new axis
        let mut new_dims = first.dims().to_vec();
        new_dims.insert(axis, arrays.len());

        let total_size: usize = new_dims.iter().product();
        let mut result = Vec::with_capacity(total_size);

        // Iterate through result indices
        let mut out_index = vec![0usize; new_dims.len()];

        for _ in 0..total_size {
            let arr_idx = out_index[axis];
            let src_index: Vec<usize> = out_index
                .iter()
                .enumerate()
                .filter_map(|(i, &v)| if i != axis { Some(v) } else { None })
                .collect();

            let offset = arrays[arr_idx].shape.offset(&src_index)?;
            result.push(arrays[arr_idx].data[offset]);

            // Increment index
            for j in (0..new_dims.len()).rev() {
                out_index[j] += 1;
                if out_index[j] < new_dims[j] {
                    break;
                }
                out_index[j] = 0;
            }
        }

        NdArray::new(result, new_dims)
    }

    /// Vertical stack (row-wise, axis=0).
    pub fn vstack(arrays: &[&NdArray<T>]) -> Result<NdArray<T>> {
        Self::concatenate(arrays, 0)
    }

    /// Horizontal stack (column-wise, axis=1 for 2D, axis=0 for 1D).
    pub fn hstack(arrays: &[&NdArray<T>]) -> Result<NdArray<T>> {
        if arrays.is_empty() {
            return Err(ArrayError::InvalidOperation("empty array list".into()));
        }

        let ndim = arrays[0].ndim();
        if ndim == 1 {
            Self::concatenate(arrays, 0)
        } else {
            Self::concatenate(arrays, 1)
        }
    }

    /// Split array into multiple sub-arrays.
    pub fn split(&self, indices: &[usize], axis: usize) -> Result<Vec<NdArray<T>>> {
        if axis >= self.ndim() {
            return Err(ArrayError::IndexOutOfBounds {
                index: vec![axis],
                shape: self.dims().to_vec(),
            });
        }

        let axis_len = self.dims()[axis];
        let mut split_points = vec![0];
        split_points.extend_from_slice(indices);
        split_points.push(axis_len);

        let mut results = Vec::new();

        for i in 0..split_points.len() - 1 {
            let start = split_points[i];
            let end = split_points[i + 1];

            if start >= end || end > axis_len {
                continue;
            }

            // Create sub-array
            let mut new_dims = self.dims().to_vec();
            new_dims[axis] = end - start;

            let sub_size: usize = new_dims.iter().product();
            let mut sub_data = Vec::with_capacity(sub_size);

            let mut out_index = vec![0usize; self.ndim()];
            for _ in 0..sub_size {
                let mut src_index = out_index.clone();
                src_index[axis] += start;
                let offset = self.shape.offset(&src_index)?;
                sub_data.push(self.data[offset]);

                // Increment index
                for j in (0..self.ndim()).rev() {
                    out_index[j] += 1;
                    if out_index[j] < new_dims[j] {
                        break;
                    }
                    out_index[j] = 0;
                }
            }

            results.push(NdArray::new(sub_data, new_dims)?);
        }

        Ok(results)
    }

    /// Repeat elements of an array.
    pub fn repeat(&self, repeats: usize) -> NdArray<T> {
        let mut result = Vec::with_capacity(self.size() * repeats);
        for &val in &self.data {
            for _ in 0..repeats {
                result.push(val);
            }
        }
        NdArray {
            data: result,
            shape: Shape::new(vec![self.size() * repeats]),
        }
    }

    /// Tile the array.
    pub fn tile(&self, reps: &[usize]) -> Result<NdArray<T>> {
        if reps.is_empty() {
            return Ok(self.clone());
        }

        // Pad reps or dims to match
        let ndim = self.ndim().max(reps.len());
        let mut dims = vec![1; ndim];
        let offset = ndim - self.ndim();
        for (i, &d) in self.dims().iter().enumerate() {
            dims[offset + i] = d;
        }

        let mut tile_reps = vec![1; ndim];
        let rep_offset = ndim - reps.len();
        for (i, &r) in reps.iter().enumerate() {
            tile_reps[rep_offset + i] = r;
        }

        // Calculate output shape
        let out_dims: Vec<usize> = dims.iter().zip(tile_reps.iter()).map(|(&d, &r)| d * r).collect();
        let out_size: usize = out_dims.iter().product();
        let mut result = Vec::with_capacity(out_size);

        let mut out_index = vec![0usize; ndim];
        for _ in 0..out_size {
            // Map output index to source index
            let src_index: Vec<usize> = out_index
                .iter()
                .zip(dims.iter())
                .map(|(&o, &d)| o % d)
                .collect();

            // Handle case where source has fewer dims
            let flat_idx = if self.ndim() < ndim {
                let src_start = ndim - self.ndim();
                let mut idx = 0;
                let mut stride = 1;
                for i in (src_start..ndim).rev() {
                    idx += src_index[i] * stride;
                    stride *= self.dims()[i - src_start];
                }
                idx
            } else {
                self.shape.offset(&src_index).unwrap_or(0)
            };

            result.push(self.data[flat_idx]);

            // Increment index
            for j in (0..ndim).rev() {
                out_index[j] += 1;
                if out_index[j] < out_dims[j] {
                    break;
                }
                out_index[j] = 0;
            }
        }

        NdArray::new(result, out_dims)
    }

    /// Get unique elements.
    pub fn unique(&self) -> NdArray<T>
    where
        T: std::hash::Hash + Eq,
    {
        let mut seen = HashSet::new();
        let unique_data: Vec<T> = self
            .data
            .iter()
            .filter(|&&x| seen.insert(x))
            .copied()
            .collect();

        let len = unique_data.len();
        NdArray {
            data: unique_data,
            shape: Shape::new(vec![len]),
        }
    }

    /// Sort the array (flattened).
    pub fn sort(&self) -> NdArray<T>
    where
        T: Ord,
    {
        let mut sorted = self.data.clone();
        sorted.sort();
        NdArray {
            data: sorted,
            shape: self.shape.clone(),
        }
    }

    /// Get indices that would sort the array.
    pub fn argsort(&self) -> Vec<usize>
    where
        T: PartialOrd,
    {
        let mut indices: Vec<usize> = (0..self.size()).collect();
        indices.sort_by(|&a, &b| {
            self.data[a]
                .partial_cmp(&self.data[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        indices
    }
}

// Logical reductions
impl NdArray<bool> {
    /// Returns true if all elements are true.
    pub fn all(&self) -> bool {
        self.data.iter().all(|&x| x)
    }

    /// Returns true if any element is true.
    pub fn any(&self) -> bool {
        self.data.iter().any(|&x| x)
    }
}

impl<T: ArrayElement + PartialOrd> NdArray<T> {
    /// Check if two arrays are element-wise equal within tolerance.
    pub fn allclose(&self, other: &NdArray<T>, rtol: f64, atol: f64) -> bool
    where
        T: Into<f64> + Copy,
    {
        if self.dims() != other.dims() {
            return false;
        }

        self.data.iter().zip(other.data.iter()).all(|(&a, &b)| {
            let a_f: f64 = a.into();
            let b_f: f64 = b.into();
            (a_f - b_f).abs() <= atol + rtol * b_f.abs()
        })
    }
}
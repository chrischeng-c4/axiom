//! Search and sort operations for N-dimensional arrays.
//!
//! Implements searchsorted, sort with axis, argsort with axis, unique with
//! return_index/return_inverse/return_counts, and where (conditional selection).

use super::dtype::ArrayElement;
use super::error::{ArrayError, Result};
use super::ndarray::NdArray;
use super::shape::Shape;

/// Result of unique() with optional return arrays.
pub struct UniqueResult<T: ArrayElement> {
    /// The sorted unique values.
    pub values: NdArray<T>,
    /// Indices of first occurrence of each unique value in original array.
    pub indices: Option<Vec<usize>>,
    /// Indices to reconstruct the original array from the unique array.
    pub inverse: Option<Vec<usize>>,
    /// Counts of each unique value.
    pub counts: Option<Vec<usize>>,
}

impl NdArray<f64> {
    /// Binary search in a sorted 1D array.
    ///
    /// Returns insertion indices such that inserting the values at those positions
    /// maintains sorted order. Equivalent to numpy.searchsorted.
    ///
    /// `side`: "left" for leftmost insertion point, "right" for rightmost.
    pub fn searchsorted(&self, values: &[f64], side: &str) -> Result<Vec<usize>> {
        if self.ndim() != 1 {
            return Err(ArrayError::InvalidOperation(
                "searchsorted requires a 1D array".into(),
            ));
        }

        let data = &self.data;
        let use_right = side == "right";

        let result: Vec<usize> = values
            .iter()
            .map(|&val| {
                if use_right {
                    // Find rightmost position
                    let mut lo = 0;
                    let mut hi = data.len();
                    while lo < hi {
                        let mid = (lo + hi) / 2;
                        if data[mid] <= val {
                            lo = mid + 1;
                        } else {
                            hi = mid;
                        }
                    }
                    lo
                } else {
                    // Find leftmost position
                    let mut lo = 0;
                    let mut hi = data.len();
                    while lo < hi {
                        let mid = (lo + hi) / 2;
                        if data[mid] < val {
                            lo = mid + 1;
                        } else {
                            hi = mid;
                        }
                    }
                    lo
                }
            })
            .collect();

        Ok(result)
    }

    /// Sort along a specific axis.
    ///
    /// Returns a new array with elements sorted along the given axis.
    pub fn sort_axis(&self, axis: usize) -> Result<NdArray<f64>> {
        if axis >= self.ndim() {
            return Err(ArrayError::InvalidAxis {
                axis,
                ndim: self.ndim(),
            });
        }

        if self.ndim() == 1 {
            let mut sorted = self.data.clone();
            sorted.sort_by(|a, b| a.total_cmp(b));
            return Ok(NdArray {
                data: sorted,
                shape: self.shape.clone(),
            });
        }

        let dims = self.dims();
        let axis_len = dims[axis];
        // Compute strides manually
        let strides = {
            let mut s = vec![1usize; dims.len()];
            for i in (0..dims.len() - 1).rev() {
                s[i] = s[i + 1] * dims[i + 1];
            }
            s
        };
        let axis_stride = strides[axis];

        let mut result = self.data.clone();

        // For each "line" along the axis, sort it
        let out_dims: Vec<usize> = dims
            .iter()
            .enumerate()
            .filter_map(|(i, &d)| if i != axis { Some(d) } else { None })
            .collect();

        let out_size: usize = if out_dims.is_empty() {
            1
        } else {
            out_dims.iter().product()
        };

        let mut out_index = vec![0usize; out_dims.len()];
        for _ in 0..out_size {
            // Build the base index (with 0 at axis position)
            let mut src_base = Vec::with_capacity(dims.len());
            let mut out_pos = 0;
            for i in 0..dims.len() {
                if i == axis {
                    src_base.push(0);
                } else {
                    src_base.push(out_index[out_pos]);
                    out_pos += 1;
                }
            }

            let base_offset = self.shape.offset(&src_base).unwrap_or(0);

            // Extract elements along axis
            let mut line: Vec<(f64, usize)> = (0..axis_len)
                .map(|i| {
                    let idx = base_offset + i * axis_stride;
                    (result[idx], idx)
                })
                .collect();

            line.sort_by(|a, b| a.0.total_cmp(&b.0));

            // Write back sorted values
            for (i, (val, _)) in line.iter().enumerate() {
                result[base_offset + i * axis_stride] = *val;
            }

            // Increment out_index
            for j in (0..out_dims.len()).rev() {
                out_index[j] += 1;
                if out_index[j] < out_dims[j] {
                    break;
                }
                out_index[j] = 0;
            }
        }

        Ok(NdArray {
            data: result,
            shape: self.shape.clone(),
        })
    }

    /// Argsort along a specific axis.
    ///
    /// Returns an array of indices that would sort the array along the given axis.
    pub fn argsort_axis(&self, axis: usize) -> Result<NdArray<f64>> {
        if axis >= self.ndim() {
            return Err(ArrayError::InvalidAxis {
                axis,
                ndim: self.ndim(),
            });
        }

        if self.ndim() == 1 {
            let mut indices: Vec<usize> = (0..self.size()).collect();
            indices.sort_by(|&a, &b| self.data[a].total_cmp(&self.data[b]));
            let data: Vec<f64> = indices.iter().map(|&i| i as f64).collect();
            return Ok(NdArray {
                data,
                shape: self.shape.clone(),
            });
        }

        let dims = self.dims();
        let axis_len = dims[axis];
        // Compute strides manually
        let strides = {
            let mut s = vec![1usize; dims.len()];
            for i in (0..dims.len() - 1).rev() {
                s[i] = s[i + 1] * dims[i + 1];
            }
            s
        };
        let axis_stride = strides[axis];

        let mut result = vec![0.0f64; self.size()];

        let out_dims: Vec<usize> = dims
            .iter()
            .enumerate()
            .filter_map(|(i, &d)| if i != axis { Some(d) } else { None })
            .collect();

        let out_size: usize = if out_dims.is_empty() {
            1
        } else {
            out_dims.iter().product()
        };

        let mut out_index = vec![0usize; out_dims.len()];
        for _ in 0..out_size {
            let mut src_base = Vec::with_capacity(dims.len());
            let mut out_pos = 0;
            for i in 0..dims.len() {
                if i == axis {
                    src_base.push(0);
                } else {
                    src_base.push(out_index[out_pos]);
                    out_pos += 1;
                }
            }

            let base_offset = self.shape.offset(&src_base).unwrap_or(0);

            let mut indices: Vec<usize> = (0..axis_len).collect();
            indices.sort_by(|&a, &b| {
                self.data[base_offset + a * axis_stride]
                    .total_cmp(&self.data[base_offset + b * axis_stride])
            });

            for (i, &idx) in indices.iter().enumerate() {
                result[base_offset + i * axis_stride] = idx as f64;
            }

            for j in (0..out_dims.len()).rev() {
                out_index[j] += 1;
                if out_index[j] < out_dims[j] {
                    break;
                }
                out_index[j] = 0;
            }
        }

        Ok(NdArray {
            data: result,
            shape: self.shape.clone(),
        })
    }

    /// Unique elements with optional return arrays.
    ///
    /// Returns sorted unique values, and optionally:
    /// - indices: first occurrence indices
    /// - inverse: mapping from original to unique
    /// - counts: count of each unique value
    pub fn unique_full(
        &self,
        return_index: bool,
        return_inverse: bool,
        return_counts: bool,
    ) -> UniqueResult<f64> {
        let flat = self.data.clone();

        // Sort with original indices
        let mut indexed: Vec<(f64, usize)> =
            flat.iter().copied().enumerate().map(|(i, v)| (v, i)).collect();
        indexed.sort_by(|a, b| a.0.total_cmp(&b.0));

        let mut unique_vals: Vec<f64> = Vec::new();
        let mut first_indices: Vec<usize> = Vec::new();
        let mut counts: Vec<usize> = Vec::new();

        // Map from original sorted index to unique index
        let mut inverse_map = vec![0usize; flat.len()];

        let mut prev: Option<f64> = None;
        for &(val, orig_idx) in &indexed {
            if prev.map_or(true, |p| (val - p).abs() > f64::EPSILON || val != p) {
                unique_vals.push(val);
                first_indices.push(orig_idx);
                counts.push(1);
                prev = Some(val);
            } else {
                *counts.last_mut().unwrap() += 1;
                // Update first index if this one is earlier
                let last_fi = first_indices.last_mut().unwrap();
                if orig_idx < *last_fi {
                    *last_fi = orig_idx;
                }
            }
            inverse_map[orig_idx] = unique_vals.len() - 1;
        }

        let n = unique_vals.len();
        UniqueResult {
            values: NdArray {
                data: unique_vals,
                shape: Shape::new(vec![n]),
            },
            indices: if return_index {
                Some(first_indices)
            } else {
                None
            },
            inverse: if return_inverse {
                Some(inverse_map)
            } else {
                None
            },
            counts: if return_counts { Some(counts) } else { None },
        }
    }

    /// Conditional element selection: np.where(condition, x, y).
    ///
    /// Returns elements from `x` where condition is true, else from `y`.
    pub fn where_select(
        condition: &NdArray<bool>,
        x: &NdArray<f64>,
        y: &NdArray<f64>,
    ) -> Result<NdArray<f64>> {
        if condition.size() != x.size() || condition.size() != y.size() {
            return Err(ArrayError::ShapeMismatch {
                expected: condition.dims().to_vec(),
                got: x.dims().to_vec(),
            });
        }

        let data: Vec<f64> = condition
            .data()
            .iter()
            .zip(x.data().iter())
            .zip(y.data().iter())
            .map(|((&c, &xv), &yv)| if c { xv } else { yv })
            .collect();

        NdArray::new(data, condition.dims().to_vec())
    }

    /// Return indices where condition is true (nonzero).
    ///
    /// Returns a vector of index tuples (as vectors).
    pub fn nonzero(condition: &NdArray<bool>) -> Vec<Vec<usize>> {
        let dims = condition.dims();
        let ndim = condition.ndim();
        let mut result: Vec<Vec<usize>> = (0..ndim).map(|_| Vec::new()).collect();

        let mut index = vec![0usize; ndim];
        for i in 0..condition.size() {
            if condition.data()[i] {
                for (ax, idx) in index.iter().enumerate() {
                    result[ax].push(*idx);
                }
            }

            // Increment index
            for j in (0..ndim).rev() {
                index[j] += 1;
                if index[j] < dims[j] {
                    break;
                }
                index[j] = 0;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_searchsorted_left() {
        let arr = NdArray::new(vec![1.0, 3.0, 5.0, 7.0, 9.0], vec![5]).unwrap();
        let result = arr.searchsorted(&[0.0, 3.0, 6.0, 10.0], "left").unwrap();
        assert_eq!(result, vec![0, 1, 3, 5]);
    }

    #[test]
    fn test_searchsorted_right() {
        let arr = NdArray::new(vec![1.0, 3.0, 5.0, 7.0, 9.0], vec![5]).unwrap();
        let result = arr.searchsorted(&[3.0, 5.0], "right").unwrap();
        assert_eq!(result, vec![2, 3]);
    }

    #[test]
    fn test_sort_axis_1d() {
        let arr = NdArray::new(vec![3.0, 1.0, 4.0, 1.0, 5.0], vec![5]).unwrap();
        let sorted = arr.sort_axis(0).unwrap();
        assert_eq!(sorted.data(), &[1.0, 1.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_sort_axis_2d() {
        // Sort each column (axis=0)
        let arr = NdArray::new(vec![3.0, 1.0, 1.0, 2.0], vec![2, 2]).unwrap();
        let sorted = arr.sort_axis(0).unwrap();
        assert_eq!(sorted.data(), &[1.0, 1.0, 3.0, 2.0]);
    }

    #[test]
    fn test_unique_full() {
        let arr = NdArray::new(vec![3.0, 1.0, 2.0, 1.0, 3.0], vec![5]).unwrap();
        let result = arr.unique_full(true, true, true);
        assert_eq!(result.values.data(), &[1.0, 2.0, 3.0]);
        assert_eq!(result.counts.unwrap(), vec![2, 1, 2]);
        let inv = result.inverse.unwrap();
        // Original: [3, 1, 2, 1, 3] -> unique indices: [2, 0, 1, 0, 2]
        assert_eq!(inv, vec![2, 0, 1, 0, 2]);
    }

    #[test]
    fn test_where_select() {
        let cond = NdArray::new(vec![true, false, true, false], vec![4]).unwrap();
        let x = NdArray::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
        let y = NdArray::new(vec![10.0, 20.0, 30.0, 40.0], vec![4]).unwrap();
        let result = NdArray::where_select(&cond, &x, &y).unwrap();
        assert_eq!(result.data(), &[1.0, 20.0, 3.0, 40.0]);
    }

    #[test]
    fn test_nonzero() {
        let cond = NdArray::new(vec![true, false, true, false, true], vec![5]).unwrap();
        let indices = NdArray::nonzero(&cond);
        assert_eq!(indices.len(), 1); // 1D
        assert_eq!(indices[0], vec![0, 2, 4]);
    }
}

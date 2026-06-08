//! Advanced indexing: fancy (integer array) and boolean (mask) indexing.
//!
//! Supports NumPy-style `arr[[1, 3, 5]]` and `arr[arr > 5]` patterns.

use super::dtype::ArrayElement;
use super::error::{ArrayError, Result};
use super::ndarray::NdArray;
use super::shape::Shape;

/// Indexing mode for advanced selection.
#[derive(Debug, Clone)]
pub enum IndexMode {
    /// Integer array indexing: select elements at given indices.
    Fancy(Vec<usize>),
    /// Boolean mask indexing: select where mask is true.
    Bool(Vec<bool>),
    /// Combined: fancy indexing per axis.
    PerAxis(Vec<AxisIndex>),
}

/// Per-axis index specification for combined indexing.
#[derive(Debug, Clone)]
pub enum AxisIndex {
    /// Select all elements along this axis.
    Full,
    /// Select at specific integer indices.
    Fancy(Vec<usize>),
    /// Select where boolean mask is true.
    Bool(Vec<bool>),
    /// Single index (reduces dimension).
    Single(usize),
}

impl<T: ArrayElement> NdArray<T> {
    /// Fancy indexing: select elements using an integer index array.
    ///
    /// For 1D arrays, returns elements at the given indices.
    /// For N-D arrays, selects along axis 0.
    ///
    /// # Example
    /// ```ignore
    /// let arr = NdArray::new(vec![10, 20, 30, 40, 50], vec![5]).unwrap();
    /// let result = arr.fancy_index(&[1, 3, 4]).unwrap();
    /// // result = [20, 40, 50]
    /// ```
    pub fn fancy_index(&self, indices: &[usize]) -> Result<NdArray<T>> {
        if self.ndim() == 0 {
            return Err(ArrayError::InvalidOperation(
                "cannot index 0-dimensional array".into(),
            ));
        }

        // Validate all indices are in bounds for axis 0
        let axis_len = self.dims()[0];
        for &idx in indices {
            if idx >= axis_len {
                return Err(ArrayError::IndexOutOfBounds {
                    index: vec![idx],
                    shape: self.dims().to_vec(),
                });
            }
        }

        if self.ndim() == 1 {
            // Simple 1D case: gather elements at indices
            let data: Vec<T> = indices.iter().map(|&i| self.data[i]).collect();
            let len = data.len();
            return Ok(NdArray {
                data,
                shape: Shape::new(vec![len]),
            });
        }

        // For N-D arrays, select entire sub-arrays along axis 0
        let sub_shape = &self.dims()[1..];
        let sub_size: usize = sub_shape.iter().product();
        let mut data = Vec::with_capacity(indices.len() * sub_size);

        for &idx in indices {
            let start = idx * sub_size;
            let end = start + sub_size;
            data.extend_from_slice(&self.data[start..end]);
        }

        let mut out_dims = vec![indices.len()];
        out_dims.extend_from_slice(sub_shape);

        NdArray::new(data, out_dims)
    }

    /// Boolean indexing: select elements where mask is true.
    ///
    /// Returns a 1D array of elements where the mask is true.
    /// Mask must have the same total size as the array.
    ///
    /// # Example
    /// ```ignore
    /// let arr = NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    /// let mask = vec![true, false, true, false, true];
    /// let result = arr.bool_index(&mask).unwrap();
    /// // result = [1.0, 3.0, 5.0]
    /// ```
    pub fn bool_index(&self, mask: &[bool]) -> Result<NdArray<T>> {
        if mask.len() != self.size() {
            return Err(ArrayError::ShapeMismatch {
                expected: self.dims().to_vec(),
                got: vec![mask.len()],
            });
        }

        let data: Vec<T> = self
            .data
            .iter()
            .zip(mask.iter())
            .filter_map(|(&val, &m)| if m { Some(val) } else { None })
            .collect();

        let len = data.len();
        Ok(NdArray {
            data,
            shape: Shape::new(vec![len]),
        })
    }

    /// Boolean indexing using a boolean NdArray mask.
    ///
    /// The mask must be broadcastable to the array's shape, or have the
    /// same total size.
    pub fn bool_index_array(&self, mask: &NdArray<bool>) -> Result<NdArray<T>> {
        if mask.size() != self.size() {
            return Err(ArrayError::ShapeMismatch {
                expected: self.dims().to_vec(),
                got: mask.dims().to_vec(),
            });
        }
        self.bool_index(mask.data())
    }

    /// Set values at boolean mask positions.
    ///
    /// Elements where mask is true are replaced with the given value.
    pub fn bool_set(&mut self, mask: &[bool], value: T) -> Result<()> {
        if mask.len() != self.size() {
            return Err(ArrayError::ShapeMismatch {
                expected: self.dims().to_vec(),
                got: vec![mask.len()],
            });
        }

        for (i, &m) in mask.iter().enumerate() {
            if m {
                self.data[i] = value;
            }
        }
        Ok(())
    }

    /// Set values at fancy index positions.
    ///
    /// For 1D arrays, sets elements at given indices to the given values.
    pub fn fancy_set(&mut self, indices: &[usize], values: &[T]) -> Result<()> {
        if indices.len() != values.len() {
            return Err(ArrayError::InvalidOperation(
                "indices and values must have same length".into(),
            ));
        }

        if self.ndim() == 1 {
            let n = self.size();
            for (i, &idx) in indices.iter().enumerate() {
                if idx >= n {
                    return Err(ArrayError::IndexOutOfBounds {
                        index: vec![idx],
                        shape: self.dims().to_vec(),
                    });
                }
                self.data[idx] = values[i];
            }
            return Ok(());
        }

        Err(ArrayError::InvalidOperation(
            "fancy_set only supports 1D arrays currently".into(),
        ))
    }

    /// Advanced indexing with per-axis specification.
    ///
    /// Each axis can use Full, Fancy, Bool, or Single indexing.
    pub fn advanced_index(&self, indices: &[AxisIndex]) -> Result<NdArray<T>> {
        if indices.is_empty() {
            return Ok(self.clone());
        }

        if indices.len() > self.ndim() {
            return Err(ArrayError::InvalidAxis {
                axis: indices.len(),
                ndim: self.ndim(),
            });
        }

        // Compute selected indices per axis
        let mut axis_selections: Vec<Vec<usize>> = Vec::with_capacity(self.ndim());
        let mut output_dims: Vec<Option<usize>> = Vec::new();

        for (ax, idx) in indices.iter().enumerate() {
            let dim = self.dims()[ax];
            match idx {
                AxisIndex::Full => {
                    axis_selections.push((0..dim).collect());
                    output_dims.push(Some(dim));
                }
                AxisIndex::Fancy(inds) => {
                    for &i in inds {
                        if i >= dim {
                            return Err(ArrayError::IndexOutOfBounds {
                                index: vec![i],
                                shape: self.dims().to_vec(),
                            });
                        }
                    }
                    let n = inds.len();
                    axis_selections.push(inds.clone());
                    output_dims.push(Some(n));
                }
                AxisIndex::Bool(mask) => {
                    if mask.len() != dim {
                        return Err(ArrayError::ShapeMismatch {
                            expected: vec![dim],
                            got: vec![mask.len()],
                        });
                    }
                    let selected: Vec<usize> = mask
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &m)| if m { Some(i) } else { None })
                        .collect();
                    let n = selected.len();
                    axis_selections.push(selected);
                    output_dims.push(Some(n));
                }
                AxisIndex::Single(i) => {
                    if *i >= dim {
                        return Err(ArrayError::IndexOutOfBounds {
                            index: vec![*i],
                            shape: self.dims().to_vec(),
                        });
                    }
                    axis_selections.push(vec![*i]);
                    output_dims.push(None); // dimension is reduced
                }
            }
        }

        // Remaining axes get full range
        for ax in indices.len()..self.ndim() {
            let dim = self.dims()[ax];
            axis_selections.push((0..dim).collect());
            output_dims.push(Some(dim));
        }

        // Build output shape (skip None = reduced dims)
        let out_shape: Vec<usize> = output_dims.iter().filter_map(|&d| d).collect();
        let out_size: usize = if out_shape.is_empty() {
            1
        } else {
            out_shape.iter().product()
        };
        let mut result = Vec::with_capacity(out_size);

        // Iterate through all combinations of selected indices
        let mut combo = vec![0usize; axis_selections.len()];
        let combo_sizes: Vec<usize> = axis_selections.iter().map(|s| s.len()).collect();

        for _ in 0..out_size {
            // Build source index from current combo
            let src_index: Vec<usize> = combo
                .iter()
                .enumerate()
                .map(|(ax, &c)| axis_selections[ax][c])
                .collect();

            let offset = self.shape.offset(&src_index)?;
            result.push(self.data[offset]);

            // Increment combo (odometer style)
            for j in (0..combo.len()).rev() {
                combo[j] += 1;
                if combo[j] < combo_sizes[j] {
                    break;
                }
                combo[j] = 0;
            }
        }

        if out_shape.is_empty() {
            // Scalar result (all axes reduced)
            Ok(NdArray {
                data: result,
                shape: Shape::new(vec![1]),
            })
        } else {
            NdArray::new(result, out_shape)
        }
    }

    /// Take elements along an axis at given indices.
    ///
    /// Like numpy.take(a, indices, axis).
    pub fn take(&self, indices: &[usize], axis: usize) -> Result<NdArray<T>> {
        if axis >= self.ndim() {
            return Err(ArrayError::InvalidAxis {
                axis,
                ndim: self.ndim(),
            });
        }

        let dim = self.dims()[axis];
        for &idx in indices {
            if idx >= dim {
                return Err(ArrayError::IndexOutOfBounds {
                    index: vec![idx],
                    shape: self.dims().to_vec(),
                });
            }
        }

        let mut new_dims = self.dims().to_vec();
        new_dims[axis] = indices.len();

        let out_size: usize = new_dims.iter().product();
        let mut result = Vec::with_capacity(out_size);

        let mut out_index = vec![0usize; self.ndim()];
        for _ in 0..out_size {
            let mut src_index = out_index.clone();
            src_index[axis] = indices[out_index[axis]];

            let offset = self.shape.offset(&src_index)?;
            result.push(self.data[offset]);

            for j in (0..self.ndim()).rev() {
                out_index[j] += 1;
                if out_index[j] < new_dims[j] {
                    break;
                }
                out_index[j] = 0;
            }
        }

        NdArray::new(result, new_dims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fancy_index_1d() {
        let arr = NdArray::new(vec![10, 20, 30, 40, 50], vec![5]).unwrap();
        let result = arr.fancy_index(&[1, 3, 4]).unwrap();
        assert_eq!(result.data(), &[20, 40, 50]);
        assert_eq!(result.dims(), &[3]);
    }

    #[test]
    fn test_fancy_index_2d() {
        let arr = NdArray::new(vec![1, 2, 3, 4, 5, 6], vec![3, 2]).unwrap();
        let result = arr.fancy_index(&[0, 2]).unwrap();
        assert_eq!(result.data(), &[1, 2, 5, 6]);
        assert_eq!(result.dims(), &[2, 2]);
    }

    #[test]
    fn test_fancy_index_out_of_bounds() {
        let arr = NdArray::new(vec![1, 2, 3], vec![3]).unwrap();
        assert!(arr.fancy_index(&[5]).is_err());
    }

    #[test]
    fn test_bool_index() {
        let arr = NdArray::new(vec![1, 2, 3, 4, 5], vec![5]).unwrap();
        let mask = vec![true, false, true, false, true];
        let result = arr.bool_index(&mask).unwrap();
        assert_eq!(result.data(), &[1, 3, 5]);
    }

    #[test]
    fn test_bool_index_array() {
        let arr = NdArray::new(vec![10.0, 20.0, 30.0, 40.0], vec![4]).unwrap();
        let mask = NdArray::new(vec![false, false, true, true], vec![4]).unwrap();
        let result = arr.bool_index_array(&mask).unwrap();
        assert_eq!(result.data(), &[30.0, 40.0]);
    }

    #[test]
    fn test_advanced_index_combined() {
        // 3x4 matrix
        let arr = NdArray::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], vec![3, 4]).unwrap();

        // Select rows [0, 2], all columns
        let result = arr
            .advanced_index(&[AxisIndex::Fancy(vec![0, 2]), AxisIndex::Full])
            .unwrap();
        assert_eq!(result.dims(), &[2, 4]);
        assert_eq!(result.data(), &[1, 2, 3, 4, 9, 10, 11, 12]);
    }

    #[test]
    fn test_take() {
        let arr = NdArray::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
        let result = arr.take(&[0, 2], 1).unwrap();
        assert_eq!(result.dims(), &[2, 2]);
        assert_eq!(result.data(), &[1, 3, 4, 6]);
    }

    #[test]
    fn test_bool_set() {
        let mut arr = NdArray::new(vec![1, 2, 3, 4, 5], vec![5]).unwrap();
        let mask = vec![false, true, false, true, false];
        arr.bool_set(&mask, 0).unwrap();
        assert_eq!(arr.data(), &[1, 0, 3, 0, 5]);
    }

    #[test]
    fn test_fancy_set() {
        let mut arr = NdArray::new(vec![10, 20, 30, 40, 50], vec![5]).unwrap();
        arr.fancy_set(&[1, 3], &[99, 88]).unwrap();
        assert_eq!(arr.data(), &[10, 99, 30, 88, 50]);
    }
}

//! Slicing and indexing for N-dimensional arrays.

use super::error::{ArrayError, Result};

/// Slice specification for a single axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisSlice {
    /// Single index (reduces dimension).
    Index(usize),
    /// Range with start, stop, step (None means use default).
    Slice {
        start: Option<isize>,
        stop: Option<isize>,
        step: Option<isize>,
    },
    /// Select all elements along this axis.
    Full,
}

/// Normalized slice info for iteration.
#[derive(Debug, Clone, Copy)]
pub struct NormalizedSlice {
    pub start: usize,
    pub stop: isize, // isize to handle negative steps correctly
    pub step: isize,
    pub output_len: usize,
    pub is_index: bool, // True if this was an Index (dimension reduction)
}

impl AxisSlice {
    /// Create a slice from start to stop with step 1.
    pub fn range(start: isize, stop: isize) -> Self {
        AxisSlice::Slice {
            start: Some(start),
            stop: Some(stop),
            step: None,
        }
    }

    /// Create a slice with explicit step.
    pub fn range_step(start: isize, stop: isize, step: isize) -> Self {
        AxisSlice::Slice {
            start: Some(start),
            stop: Some(stop),
            step: Some(step),
        }
    }

    /// Normalize this slice for an axis of given length.
    pub fn normalize(&self, len: usize) -> Result<NormalizedSlice> {
        match self {
            AxisSlice::Index(i) => {
                if *i >= len {
                    return Err(ArrayError::InvalidSlice {
                        start: *i as isize,
                        stop: *i as isize,
                        step: 1,
                        len,
                    });
                }
                Ok(NormalizedSlice {
                    start: *i,
                    stop: (*i + 1) as isize,
                    step: 1,
                    output_len: 1,
                    is_index: true,
                })
            }
            AxisSlice::Full => Ok(NormalizedSlice {
                start: 0,
                stop: len as isize,
                step: 1,
                output_len: len,
                is_index: false,
            }),
            AxisSlice::Slice { start, stop, step } => {
                let step = step.unwrap_or(1);
                if step == 0 {
                    return Err(ArrayError::InvalidSlice {
                        start: start.unwrap_or(0),
                        stop: stop.unwrap_or(len as isize),
                        step: 0,
                        len,
                    });
                }

                // Default values depend on step direction (NumPy semantics)
                let (default_start, default_stop) = if step > 0 {
                    (0isize, len as isize)
                } else {
                    ((len as isize) - 1, -1isize) // -1 means before index 0
                };

                let raw_start = start.unwrap_or(default_start);
                let raw_stop = stop.unwrap_or(default_stop);

                // Normalize negative indices
                let norm_start = if raw_start < 0 {
                    ((len as isize) + raw_start).max(0) as usize
                } else {
                    (raw_start as usize).min(len)
                };

                // Keep stop as isize to handle -1 correctly for negative steps
                // For negative step with None stop, keep -1 as-is (means "before index 0")
                // For explicit negative stop like arr[4:-2:-1], normalize it
                let norm_stop = if stop.is_none() && step < 0 {
                    raw_stop // Keep -1 as-is for default negative step stop
                } else if raw_stop < 0 {
                    (len as isize) + raw_stop // Normalize explicit negative indices
                } else {
                    (raw_stop as usize).min(len) as isize
                };

                // Calculate output length
                let output_len = if step > 0 {
                    if norm_stop <= (norm_start as isize) {
                        0
                    } else {
                        ((norm_stop - norm_start as isize + step - 1) / step) as usize
                    }
                } else {
                    if (norm_start as isize) <= norm_stop {
                        0
                    } else {
                        (((norm_start as isize) - norm_stop - step - 1) / (-step)) as usize
                    }
                };

                Ok(NormalizedSlice {
                    start: norm_start,
                    stop: norm_stop,
                    step,
                    output_len,
                    is_index: false,
                })
            }
        }
    }
}

/// Collection of slice specifications for all axes.
#[derive(Debug, Clone)]
pub struct SliceInfo {
    slices: Vec<AxisSlice>,
}

impl SliceInfo {
    pub fn new(slices: Vec<AxisSlice>) -> Self {
        Self { slices }
    }

    pub fn slices(&self) -> &[AxisSlice] {
        &self.slices
    }

    /// Calculate output shape after applying this slice info.
    pub fn output_shape(&self, input_shape: &[usize]) -> Result<Vec<usize>> {
        if self.slices.len() > input_shape.len() {
            return Err(ArrayError::InvalidAxis {
                axis: self.slices.len(),
                ndim: input_shape.len(),
            });
        }

        let mut result = Vec::new();

        for (i, slice) in self.slices.iter().enumerate() {
            let normalized = slice.normalize(input_shape[i])?;
            // Index reduces dimension, skip it
            if !normalized.is_index {
                result.push(normalized.output_len);
            }
        }

        // Append remaining dimensions
        for &dim in &input_shape[self.slices.len()..] {
            result.push(dim);
        }

        Ok(result)
    }

    /// Normalize all slices for iteration.
    pub fn normalize_all(&self, input_shape: &[usize]) -> Result<Vec<NormalizedSlice>> {
        let mut result = Vec::with_capacity(input_shape.len());

        for (i, slice) in self.slices.iter().enumerate() {
            result.push(slice.normalize(input_shape[i])?);
        }

        // Remaining axes get Full slice
        for &dim in &input_shape[self.slices.len()..] {
            result.push(NormalizedSlice {
                start: 0,
                stop: dim as isize,
                step: 1,
                output_len: dim,
                is_index: false,
            });
        }

        Ok(result)
    }
}

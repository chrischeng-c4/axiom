//! Statistical reduction operations for N-dimensional arrays.
//!
//! Implements NumPy-style reductions: sum, mean, min, max with axis support.

use super::dtype::ArrayElement;
use super::error::{ArrayError, Result};
use super::ndarray::NdArray;
use super::shape::Shape;
use std::ops::{Add, Div};

/// Trait for types that support statistical operations.
pub trait StatOps: ArrayElement + Add<Output = Self> + PartialOrd {
    /// Convert from usize (for mean calculation).
    fn from_usize(n: usize) -> Self;
}

impl StatOps for f32 {
    fn from_usize(n: usize) -> Self {
        n as f32
    }
}

impl StatOps for f64 {
    fn from_usize(n: usize) -> Self {
        n as f64
    }
}

impl StatOps for i32 {
    fn from_usize(n: usize) -> Self {
        n as i32
    }
}

impl StatOps for i64 {
    fn from_usize(n: usize) -> Self {
        n as i64
    }
}

impl<T: StatOps> NdArray<T> {
    /// Sum of all elements.
    ///
    /// # Example
    /// ```ignore
    /// let arr = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    /// assert_eq!(arr.sum(), 10);
    /// ```
    pub fn sum(&self) -> T {
        self.data.iter().fold(T::zero(), |acc, &x| acc + x)
    }

    /// Sum along a specific axis, returning an array with that axis removed.
    ///
    /// # Example
    /// ```ignore
    /// let arr = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    /// let row_sums = arr.sum_axis(1).unwrap(); // [3, 7]
    /// let col_sums = arr.sum_axis(0).unwrap(); // [4, 6]
    /// ```
    pub fn sum_axis(&self, axis: usize) -> Result<NdArray<T>> {
        self.reduce_axis(axis, |slice| slice.iter().fold(T::zero(), |acc, &x| acc + x))
    }

    /// Minimum of all elements.
    pub fn min(&self) -> Option<T> {
        self.data.iter().fold(None, |acc, &x| match acc {
            None => Some(x),
            Some(m) => Some(if x < m { x } else { m }),
        })
    }

    /// Minimum along a specific axis.
    pub fn min_axis(&self, axis: usize) -> Result<NdArray<T>> {
        self.reduce_axis(axis, |slice| {
            slice
                .iter()
                .fold(None, |acc: Option<T>, &x| match acc {
                    None => Some(x),
                    Some(m) => Some(if x < m { x } else { m }),
                })
                .unwrap_or(T::zero())
        })
    }

    /// Maximum of all elements.
    pub fn max(&self) -> Option<T> {
        self.data.iter().fold(None, |acc, &x| match acc {
            None => Some(x),
            Some(m) => Some(if x > m { x } else { m }),
        })
    }

    /// Maximum along a specific axis.
    pub fn max_axis(&self, axis: usize) -> Result<NdArray<T>> {
        self.reduce_axis(axis, |slice| {
            slice
                .iter()
                .fold(None, |acc: Option<T>, &x| match acc {
                    None => Some(x),
                    Some(m) => Some(if x > m { x } else { m }),
                })
                .unwrap_or(T::zero())
        })
    }

    /// Internal: reduce along an axis using the given reduction function.
    fn reduce_axis<F>(&self, axis: usize, reduce_fn: F) -> Result<NdArray<T>>
    where
        F: Fn(&[T]) -> T,
    {
        use super::error::ArrayError;

        if axis >= self.ndim() {
            return Err(ArrayError::IndexOutOfBounds {
                index: vec![axis],
                shape: self.dims().to_vec(),
            });
        }

        let dims = self.dims();
        let axis_len = dims[axis];

        // Build output shape (remove the axis dimension)
        let out_dims: Vec<usize> = dims
            .iter()
            .enumerate()
            .filter_map(|(i, &d)| if i != axis { Some(d) } else { None })
            .collect();

        let out_size: usize = out_dims.iter().product();
        if out_size == 0 {
            return Ok(NdArray {
                data: vec![],
                shape: Shape::new(out_dims),
            });
        }

        let mut result = Vec::with_capacity(out_size);

        // Calculate strides for iteration
        let strides = self.compute_strides();
        let axis_stride = strides[axis];

        // Iterate over all positions in the output array
        let mut out_index = vec![0usize; out_dims.len()];
        for _ in 0..out_size {
            // Map output index to source index (insert 0 at axis position)
            let mut src_base_index = Vec::with_capacity(dims.len());
            let mut out_pos = 0;
            for i in 0..dims.len() {
                if i == axis {
                    src_base_index.push(0);
                } else {
                    src_base_index.push(out_index[out_pos]);
                    out_pos += 1;
                }
            }

            // Collect elements along the axis
            let base_offset = self.shape.offset(&src_base_index).unwrap_or(0);
            let slice: Vec<T> = (0..axis_len)
                .map(|i| self.data[base_offset + i * axis_stride])
                .collect();

            result.push(reduce_fn(&slice));

            // Increment output index
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
            shape: Shape::new(out_dims),
        })
    }

    /// Compute strides for the array (row-major order).
    fn compute_strides(&self) -> Vec<usize> {
        let dims = self.dims();
        let mut strides = vec![1; dims.len()];
        for i in (0..dims.len().saturating_sub(1)).rev() {
            strides[i] = strides[i + 1] * dims[i + 1];
        }
        strides
    }
}

// Mean requires division, so separate impl for float types
impl<T: StatOps + Div<Output = T>> NdArray<T> {
    /// Mean of all elements.
    ///
    /// # Example
    /// ```ignore
    /// let arr = NdArray::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    /// assert_eq!(arr.mean(), 2.5);
    /// ```
    pub fn mean(&self) -> T {
        if self.size() == 0 {
            return T::zero();
        }
        self.sum() / T::from_usize(self.size())
    }

    /// Mean along a specific axis.
    pub fn mean_axis(&self, axis: usize) -> Result<NdArray<T>> {
        use super::error::ArrayError;

        if axis >= self.ndim() {
            return Err(ArrayError::IndexOutOfBounds {
                index: vec![axis],
                shape: self.dims().to_vec(),
            });
        }

        let axis_len = self.dims()[axis];
        let sum_arr = self.sum_axis(axis)?;

        // Divide each element by axis_len
        let divisor = T::from_usize(axis_len);
        let data: Vec<T> = sum_arr.data().iter().map(|&x| x / divisor).collect();

        Ok(NdArray {
            data,
            shape: sum_arr.shape().clone(),
        })
    }
}

/// Trait for types that support variance calculations.
pub trait VarOps: StatOps + Div<Output = Self> + std::ops::Mul<Output = Self> + std::ops::Sub<Output = Self> {
    fn sqrt_val(self) -> Self;
}

impl VarOps for f32 {
    fn sqrt_val(self) -> Self {
        self.sqrt()
    }
}

impl VarOps for f64 {
    fn sqrt_val(self) -> Self {
        self.sqrt()
    }
}

impl<T: VarOps> NdArray<T> {
    /// Variance of all elements.
    ///
    /// Uses Bessel's correction (N-1 denominator) for sample variance.
    pub fn var(&self) -> T {
        if self.size() <= 1 {
            return T::zero();
        }

        let mean = self.mean();
        let sum_sq: T = self
            .data
            .iter()
            .fold(T::zero(), |acc, &x| acc + (x - mean) * (x - mean));

        sum_sq / T::from_usize(self.size() - 1)
    }

    /// Population variance (N denominator).
    pub fn var_pop(&self) -> T {
        if self.size() == 0 {
            return T::zero();
        }

        let mean = self.mean();
        let sum_sq: T = self
            .data
            .iter()
            .fold(T::zero(), |acc, &x| acc + (x - mean) * (x - mean));

        sum_sq / T::from_usize(self.size())
    }

    /// Standard deviation of all elements.
    ///
    /// Uses Bessel's correction (N-1 denominator) for sample std.
    pub fn std(&self) -> T {
        self.var().sqrt_val()
    }

    /// Population standard deviation (N denominator).
    pub fn std_pop(&self) -> T {
        self.var_pop().sqrt_val()
    }

    /// Variance along a specific axis.
    pub fn var_axis(&self, axis: usize) -> Result<NdArray<T>> {
        use super::error::ArrayError;

        if axis >= self.ndim() {
            return Err(ArrayError::IndexOutOfBounds {
                index: vec![axis],
                shape: self.dims().to_vec(),
            });
        }

        self.reduce_axis(axis, |slice| {
            if slice.len() <= 1 {
                return T::zero();
            }
            let mean = slice.iter().fold(T::zero(), |acc, &x| acc + x)
                / T::from_usize(slice.len());
            let sum_sq: T = slice
                .iter()
                .fold(T::zero(), |acc, &x| acc + (x - mean) * (x - mean));
            sum_sq / T::from_usize(slice.len() - 1)
        })
    }

    /// Standard deviation along a specific axis.
    pub fn std_axis(&self, axis: usize) -> Result<NdArray<T>> {
        let var_arr = self.var_axis(axis)?;
        Ok(NdArray {
            data: var_arr.data().iter().map(|&x| x.sqrt_val()).collect(),
            shape: var_arr.shape().clone(),
        })
    }
}

impl<T: StatOps> NdArray<T> {
    /// Index of minimum element.
    pub fn argmin(&self) -> Option<usize> {
        if self.data.is_empty() {
            return None;
        }
        self.data
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
    }

    /// Index of maximum element.
    pub fn argmax(&self) -> Option<usize> {
        if self.data.is_empty() {
            return None;
        }
        self.data
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
    }

    /// Cumulative sum along the flattened array.
    pub fn cumsum(&self) -> NdArray<T> {
        let mut result = Vec::with_capacity(self.size());
        let mut acc = T::zero();
        for &x in &self.data {
            acc = acc + x;
            result.push(acc);
        }
        NdArray {
            data: result,
            shape: self.shape.clone(),
        }
    }
}

impl<T: StatOps + std::ops::Mul<Output = T>> NdArray<T> {
    /// Cumulative product along the flattened array.
    pub fn cumprod(&self) -> NdArray<T> {
        let mut result = Vec::with_capacity(self.size());
        let mut acc = T::one();
        for &x in &self.data {
            acc = acc * x;
            result.push(acc);
        }
        NdArray {
            data: result,
            shape: self.shape.clone(),
        }
    }
}

// Advanced statistics for float types (extended in Phase 2)
impl NdArray<f64> {
    /// Compute the median of all elements.
    pub fn median(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }

        let mut sorted = self.data.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));

        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            Some((sorted[mid - 1] + sorted[mid]) / 2.0)
        } else {
            Some(sorted[mid])
        }
    }

    /// Compute the mode (most frequent value).
    ///
    /// For continuous data, values are discretized to 10 decimal places.
    pub fn mode(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }

        use std::collections::HashMap;

        // Discretize by rounding to avoid floating point issues
        let precision = 1e10;
        let mut counts: HashMap<i64, (usize, f64)> = HashMap::new();

        for &x in &self.data {
            let key = (x * precision).round() as i64;
            let entry = counts.entry(key).or_insert((0, x));
            entry.0 += 1;
        }

        counts
            .into_values()
            .max_by_key(|(count, _)| *count)
            .map(|(_, value)| value)
    }

    /// Compute the skewness of all elements.
    ///
    /// Measures asymmetry of the distribution.
    /// - Positive skew: tail extends right
    /// - Negative skew: tail extends left
    pub fn skew(&self) -> f64 {
        let n = self.data.len() as f64;
        if n < 3.0 {
            return f64::NAN;
        }

        let mean = self.mean();
        let m2: f64 = self.data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
        let m3: f64 = self.data.iter().map(|&x| (x - mean).powi(3)).sum::<f64>() / n;

        if m2 == 0.0 {
            return 0.0;
        }

        let g1 = m3 / m2.powf(1.5);

        // Fisher's adjustment for sample bias
        let adjustment = ((n * (n - 1.0)).sqrt()) / (n - 2.0);
        adjustment * g1
    }

    /// Compute the excess kurtosis of all elements.
    ///
    /// Measures "tailedness" of the distribution.
    /// - Excess kurtosis > 0: heavy tails (leptokurtic)
    /// - Excess kurtosis < 0: light tails (platykurtic)
    /// - Excess kurtosis = 0: normal distribution
    pub fn kurtosis(&self) -> f64 {
        let n = self.data.len() as f64;
        if n < 4.0 {
            return f64::NAN;
        }

        let mean = self.mean();
        let m2: f64 = self.data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
        let m4: f64 = self.data.iter().map(|&x| (x - mean).powi(4)).sum::<f64>() / n;

        if m2 == 0.0 {
            return 0.0;
        }

        let g2 = m4 / (m2 * m2) - 3.0;

        // Adjust for sample bias
        let adjustment = (n - 1.0) / ((n - 2.0) * (n - 3.0));
        adjustment * ((n + 1.0) * g2 + 6.0)
    }

    /// Compute z-scores for all elements.
    ///
    /// Z-score = (x - mean) / std
    pub fn zscore(&self) -> NdArray<f64> {
        if self.data.len() < 2 {
            return self.clone();
        }

        let mean = self.mean();
        let std = self.std();

        if std == 0.0 {
            return NdArray {
                data: vec![0.0; self.data.len()],
                shape: self.shape.clone(),
            };
        }

        let data: Vec<f64> = self.data.iter().map(|&x| (x - mean) / std).collect();
        NdArray {
            data,
            shape: self.shape.clone(),
        }
    }

    /// Compute the nth central moment.
    pub fn moment(&self, n: i32) -> f64 {
        if self.data.is_empty() {
            return f64::NAN;
        }

        let mean = self.mean();
        self.data.iter().map(|&x| (x - mean).powi(n)).sum::<f64>() / self.data.len() as f64
    }

    /// Compute the interquartile range (IQR).
    pub fn iqr(&self) -> Option<f64> {
        let q1 = self.percentile(25.0)?;
        let q3 = self.percentile(75.0)?;
        Some(q3 - q1)
    }

    /// Compute the geometric mean.
    pub fn geometric_mean(&self) -> f64 {
        if self.data.is_empty() {
            return f64::NAN;
        }

        // Check for non-positive values
        if self.data.iter().any(|&x| x <= 0.0) {
            return f64::NAN;
        }

        let log_sum: f64 = self.data.iter().map(|&x| x.ln()).sum();
        (log_sum / self.data.len() as f64).exp()
    }

    /// Compute the harmonic mean.
    pub fn harmonic_mean(&self) -> f64 {
        if self.data.is_empty() {
            return f64::NAN;
        }

        // Check for zero or negative values
        if self.data.iter().any(|&x| x <= 0.0) {
            return f64::NAN;
        }

        let inv_sum: f64 = self.data.iter().map(|&x| 1.0 / x).sum();
        self.data.len() as f64 / inv_sum
    }

    /// Compute the trimmed mean.
    pub fn trim_mean(&self, proportiontocut: f64) -> f64 {
        if self.data.is_empty() {
            return f64::NAN;
        }

        let mut sorted = self.data.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));

        let n = sorted.len();
        let k = (n as f64 * proportiontocut) as usize;

        if k * 2 >= n {
            return self.median().unwrap_or(f64::NAN);
        }

        let trimmed = &sorted[k..n - k];
        trimmed.iter().sum::<f64>() / trimmed.len() as f64
    }

    /// Compute the standard error of the mean.
    pub fn sem(&self) -> f64 {
        if self.data.len() < 2 {
            return f64::NAN;
        }

        let n = self.data.len() as f64;
        self.std() / n.sqrt()
    }

    /// Compute the coefficient of variation.
    pub fn coefficient_of_variation(&self) -> f64 {
        if self.data.is_empty() {
            return f64::NAN;
        }

        let mean = self.mean();

        if mean == 0.0 {
            return f64::NAN;
        }

        self.std() / mean.abs()
    }


    /// Compute the q-th percentile of the data.
    ///
    /// q should be between 0 and 100.
    pub fn percentile(&self, q: f64) -> Option<f64> {
        if self.data.is_empty() || q < 0.0 || q > 100.0 {
            return None;
        }

        let mut sorted = self.data.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));

        let idx = (q / 100.0) * (sorted.len() - 1) as f64;
        let lower = idx.floor() as usize;
        let upper = idx.ceil() as usize;

        if lower == upper {
            Some(sorted[lower])
        } else {
            let frac = idx - lower as f64;
            Some(sorted[lower] * (1.0 - frac) + sorted[upper] * frac)
        }
    }

    /// Compute the q-th quantile (q between 0 and 1).
    pub fn quantile(&self, q: f64) -> Option<f64> {
        self.percentile(q * 100.0)
    }

    /// Compute correlation coefficient matrix.
    ///
    /// Input should be 2D where each row is a variable and each column is an observation.
    pub fn corrcoef(&self) -> Result<NdArray<f64>> {
        if self.ndim() != 2 {
            return Err(ArrayError::ShapeMismatch {
                expected: vec![2],
                got: vec![self.ndim()],
            });
        }

        let (n_vars, n_obs) = (self.dims()[0], self.dims()[1]);
        if n_obs < 2 {
            return Err(ArrayError::InvalidOperation("need at least 2 observations".into()));
        }

        // Compute means
        let means: Vec<f64> = (0..n_vars)
            .map(|i| {
                let sum: f64 = (0..n_obs).map(|j| self.data[i * n_obs + j]).sum();
                sum / n_obs as f64
            })
            .collect();

        // Compute covariance matrix
        let mut cov = vec![0.0; n_vars * n_vars];
        for i in 0..n_vars {
            for j in i..n_vars {
                let mut sum = 0.0;
                for k in 0..n_obs {
                    sum += (self.data[i * n_obs + k] - means[i])
                        * (self.data[j * n_obs + k] - means[j]);
                }
                cov[i * n_vars + j] = sum / (n_obs - 1) as f64;
                cov[j * n_vars + i] = cov[i * n_vars + j];
            }
        }

        // Compute correlation from covariance
        let mut corr = vec![0.0; n_vars * n_vars];
        for i in 0..n_vars {
            for j in 0..n_vars {
                let std_i = cov[i * n_vars + i].sqrt();
                let std_j = cov[j * n_vars + j].sqrt();
                if std_i > 0.0 && std_j > 0.0 {
                    corr[i * n_vars + j] = cov[i * n_vars + j] / (std_i * std_j);
                } else {
                    corr[i * n_vars + j] = if i == j { 1.0 } else { 0.0 };
                }
            }
        }

        NdArray::new(corr, vec![n_vars, n_vars])
    }

    /// Compute covariance matrix.
    pub fn cov(&self) -> Result<NdArray<f64>> {
        if self.ndim() != 2 {
            return Err(ArrayError::ShapeMismatch {
                expected: vec![2],
                got: vec![self.ndim()],
            });
        }

        let (n_vars, n_obs) = (self.dims()[0], self.dims()[1]);
        if n_obs < 2 {
            return Err(ArrayError::InvalidOperation("need at least 2 observations".into()));
        }

        // Compute means
        let means: Vec<f64> = (0..n_vars)
            .map(|i| {
                let sum: f64 = (0..n_obs).map(|j| self.data[i * n_obs + j]).sum();
                sum / n_obs as f64
            })
            .collect();

        // Compute covariance matrix
        let mut cov = vec![0.0; n_vars * n_vars];
        for i in 0..n_vars {
            for j in i..n_vars {
                let mut sum = 0.0;
                for k in 0..n_obs {
                    sum += (self.data[i * n_obs + k] - means[i])
                        * (self.data[j * n_obs + k] - means[j]);
                }
                cov[i * n_vars + j] = sum / (n_obs - 1) as f64;
                cov[j * n_vars + i] = cov[i * n_vars + j];
            }
        }

        NdArray::new(cov, vec![n_vars, n_vars])
    }

    /// Compute histogram.
    ///
    /// Returns (counts, bin_edges).
    pub fn histogram(&self, bins: usize) -> (Vec<usize>, Vec<f64>) {
        if self.data.is_empty() || bins == 0 {
            return (vec![], vec![]);
        }

        let min = self.data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = self.data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        if min == max {
            return (vec![self.size()], vec![min, max]);
        }

        let bin_width = (max - min) / bins as f64;
        let mut counts = vec![0usize; bins];
        let bin_edges: Vec<f64> = (0..=bins).map(|i| min + i as f64 * bin_width).collect();

        for &val in &self.data {
            let mut bin_idx = ((val - min) / bin_width).floor() as usize;
            if bin_idx >= bins {
                bin_idx = bins - 1;
            }
            counts[bin_idx] += 1;
        }

        (counts, bin_edges)
    }
}
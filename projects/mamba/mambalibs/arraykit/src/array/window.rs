//! Rolling window operations for N-dimensional arrays.

use super::dtype::ArrayElement;
use super::ndarray::NdArray;
use super::shape::Shape;
use super::stats::StatOps;
use std::ops::Div;

/// Rolling window over a 1D array.
pub struct Rolling<'a, T: ArrayElement> {
    array: &'a NdArray<T>,
    window: usize,
    min_periods: usize,
}

impl<'a, T: ArrayElement> Rolling<'a, T> {
    /// Create a new rolling window.
    pub fn new(array: &'a NdArray<T>, window: usize) -> Self {
        Self {
            array,
            window,
            min_periods: window,
        }
    }

    /// Set minimum number of observations required.
    pub fn min_periods(mut self, min_periods: usize) -> Self {
        self.min_periods = min_periods;
        self
    }
}

impl<'a, T: StatOps> Rolling<'a, T> {
    /// Rolling sum.
    pub fn sum(&self) -> NdArray<T> {
        let n = self.array.size();
        let mut result = Vec::with_capacity(n);

        for i in 0..n {
            if i + 1 < self.min_periods {
                result.push(T::zero());
            } else {
                let start = i.saturating_sub(self.window - 1);
                let sum = self.array.data[start..=i]
                    .iter()
                    .fold(T::zero(), |acc, &x| acc + x);
                result.push(sum);
            }
        }

        NdArray {
            data: result,
            shape: self.array.shape.clone(),
        }
    }

    /// Rolling count.
    pub fn count(&self) -> Vec<usize> {
        let n = self.array.size();
        let mut result = Vec::with_capacity(n);

        for i in 0..n {
            let start = i.saturating_sub(self.window - 1);
            let count = i - start + 1;
            result.push(count.min(self.window));
        }

        result
    }
}

impl<'a, T: StatOps + Div<Output = T>> Rolling<'a, T> {
    /// Rolling mean.
    pub fn mean(&self) -> NdArray<T> {
        let n = self.array.size();
        let mut result = Vec::with_capacity(n);

        for i in 0..n {
            if i + 1 < self.min_periods {
                result.push(T::zero());
            } else {
                let start = i.saturating_sub(self.window - 1);
                let slice = &self.array.data[start..=i];
                let sum = slice.iter().fold(T::zero(), |acc, &x| acc + x);
                let count = slice.len();
                result.push(sum / T::from_usize(count));
            }
        }

        NdArray {
            data: result,
            shape: self.array.shape.clone(),
        }
    }
}

impl<'a> Rolling<'a, f64> {
    /// Rolling min.
    pub fn min(&self) -> NdArray<f64> {
        let n = self.array.size();
        let mut result = Vec::with_capacity(n);

        for i in 0..n {
            if i + 1 < self.min_periods {
                result.push(f64::NAN);
            } else {
                let start = i.saturating_sub(self.window - 1);
                let min = self.array.data[start..=i]
                    .iter()
                    .fold(f64::INFINITY, |a, &b| a.min(b));
                result.push(min);
            }
        }

        NdArray {
            data: result,
            shape: self.array.shape.clone(),
        }
    }

    /// Rolling max.
    pub fn max(&self) -> NdArray<f64> {
        let n = self.array.size();
        let mut result = Vec::with_capacity(n);

        for i in 0..n {
            if i + 1 < self.min_periods {
                result.push(f64::NAN);
            } else {
                let start = i.saturating_sub(self.window - 1);
                let max = self.array.data[start..=i]
                    .iter()
                    .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                result.push(max);
            }
        }

        NdArray {
            data: result,
            shape: self.array.shape.clone(),
        }
    }

    /// Rolling standard deviation.
    pub fn std(&self) -> NdArray<f64> {
        let n = self.array.size();
        let mut result = Vec::with_capacity(n);

        for i in 0..n {
            if i + 1 < self.min_periods {
                result.push(f64::NAN);
            } else {
                let start = i.saturating_sub(self.window - 1);
                let slice = &self.array.data[start..=i];
                let count = slice.len() as f64;
                let mean = slice.iter().sum::<f64>() / count;
                let var = slice.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (count - 1.0);
                result.push(var.sqrt());
            }
        }

        NdArray {
            data: result,
            shape: self.array.shape.clone(),
        }
    }

    /// Rolling variance.
    pub fn var(&self) -> NdArray<f64> {
        let n = self.array.size();
        let mut result = Vec::with_capacity(n);

        for i in 0..n {
            if i + 1 < self.min_periods {
                result.push(f64::NAN);
            } else {
                let start = i.saturating_sub(self.window - 1);
                let slice = &self.array.data[start..=i];
                let count = slice.len() as f64;
                let mean = slice.iter().sum::<f64>() / count;
                let var = slice.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (count - 1.0);
                result.push(var);
            }
        }

        NdArray {
            data: result,
            shape: self.array.shape.clone(),
        }
    }
}

impl<T: ArrayElement> NdArray<T> {
    /// Create a rolling window view.
    pub fn rolling(&self, window: usize) -> Rolling<'_, T> {
        Rolling::new(self, window)
    }
}

// Additional array manipulation
impl<T: ArrayElement> NdArray<T> {
    /// Remove axes of length 1.
    pub fn squeeze(&self) -> Self {
        let new_dims: Vec<usize> = self.dims().iter().filter(|&&d| d != 1).cloned().collect();
        if new_dims.is_empty() {
            Self {
                data: self.data.clone(),
                shape: Shape::new(vec![1]),
            }
        } else {
            Self {
                data: self.data.clone(),
                shape: Shape::new(new_dims),
            }
        }
    }

    /// Add an axis at the specified position.
    pub fn expand_dims(&self, axis: usize) -> Self {
        let mut new_dims = self.dims().to_vec();
        let axis = axis.min(new_dims.len());
        new_dims.insert(axis, 1);
        Self {
            data: self.data.clone(),
            shape: Shape::new(new_dims),
        }
    }

    /// Reverse the order of elements.
    pub fn flip(&self) -> Self {
        let mut data = self.data.clone();
        data.reverse();
        Self {
            data,
            shape: self.shape.clone(),
        }
    }

    /// Roll elements along the array.
    pub fn roll(&self, shift: isize) -> Self {
        let n = self.size();
        if n == 0 {
            return self.clone();
        }

        let shift = ((shift % n as isize) + n as isize) as usize % n;
        let mut data = Vec::with_capacity(n);
        for i in 0..n {
            let src_idx = (n + i - shift) % n;
            data.push(self.data[src_idx]);
        }

        Self {
            data,
            shape: self.shape.clone(),
        }
    }

    /// Pad array with zeros.
    pub fn pad(&self, pad_width: usize) -> Self {
        if self.ndim() != 1 {
            return self.clone();
        }

        let n = self.size();
        let new_size = n + 2 * pad_width;
        let mut data = vec![T::zero(); new_size];
        for (i, &v) in self.data.iter().enumerate() {
            data[pad_width + i] = v;
        }

        Self {
            data,
            shape: Shape::new(vec![new_size]),
        }
    }
}

impl NdArray<f64> {
    /// Compute differences between consecutive elements.
    pub fn diff(&self, n: usize) -> Self {
        if self.ndim() != 1 || n == 0 || n >= self.size() {
            return self.clone();
        }

        let mut result = self.data.clone();
        for _ in 0..n {
            let new_len = result.len() - 1;
            let mut new_result = Vec::with_capacity(new_len);
            for i in 0..new_len {
                new_result.push(result[i + 1] - result[i]);
            }
            result = new_result;
        }

        let result_len = result.len();
        Self {
            data: result,
            shape: Shape::new(vec![result_len]),
        }
    }

    /// Linear interpolation.
    pub fn interp(x: &[f64], xp: &[f64], fp: &[f64]) -> Vec<f64> {
        x.iter()
            .map(|&xi| {
                if xp.is_empty() || fp.is_empty() {
                    return f64::NAN;
                }
                if xi <= xp[0] {
                    return fp[0];
                }
                if xi >= xp[xp.len() - 1] {
                    return fp[fp.len() - 1];
                }

                // Find bracketing indices
                let mut i = 0;
                while i < xp.len() - 1 && xp[i + 1] < xi {
                    i += 1;
                }

                // Linear interpolation
                let x0 = xp[i];
                let x1 = xp[i + 1];
                let y0 = fp[i];
                let y1 = fp[i + 1];

                y0 + (y1 - y0) * (xi - x0) / (x1 - x0)
            })
            .collect()
    }

    /// Numerical gradient.
    pub fn gradient(&self) -> Self {
        if self.ndim() != 1 || self.size() < 2 {
            return self.clone();
        }

        let n = self.size();
        let mut result = Vec::with_capacity(n);

        // Forward difference for first point
        result.push(self.data[1] - self.data[0]);

        // Central difference for interior points
        for i in 1..n - 1 {
            result.push((self.data[i + 1] - self.data[i - 1]) / 2.0);
        }

        // Backward difference for last point
        result.push(self.data[n - 1] - self.data[n - 2]);

        Self {
            data: result,
            shape: self.shape.clone(),
        }
    }
}

// Rotate 90 degrees for 2D arrays
impl<T: ArrayElement> NdArray<T> {
    /// Rotate array 90 degrees counterclockwise.
    pub fn rot90(&self, k: usize) -> Self {
        if self.ndim() != 2 {
            return self.clone();
        }

        let k = k % 4;
        if k == 0 {
            return self.clone();
        }

        let mut result = self.clone();

        for _ in 0..k {
            let (old_m, old_n) = (result.dims()[0], result.dims()[1]);
            let mut new_data = Vec::with_capacity(old_m * old_n);

            for j in (0..old_n).rev() {
                for i in 0..old_m {
                    new_data.push(result.data[i * old_n + j]);
                }
            }

            result = Self {
                data: new_data,
                shape: Shape::new(vec![old_n, old_m]),
            };
        }

        result
    }
}
//! Mathematical operations for N-dimensional arrays.
//!
//! Implements element-wise math functions: abs, sqrt, exp, log, clip, where.

use super::dtype::ArrayElement;
use super::error::Result;
use super::ndarray::NdArray;
use super::shape::Shape;

/// Trait for types that support floating-point math operations.
pub trait FloatOps: ArrayElement {
    fn sqrt_val(self) -> Self;
    fn exp_val(self) -> Self;
    fn ln_val(self) -> Self;
    fn abs_val(self) -> Self;
    fn from_f64(v: f64) -> Self;
    fn to_f64(self) -> f64;
}

impl FloatOps for f32 {
    fn sqrt_val(self) -> Self {
        self.sqrt()
    }
    fn exp_val(self) -> Self {
        self.exp()
    }
    fn ln_val(self) -> Self {
        self.ln()
    }
    fn abs_val(self) -> Self {
        self.abs()
    }
    fn from_f64(v: f64) -> Self {
        v as f32
    }
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl FloatOps for f64 {
    fn sqrt_val(self) -> Self {
        self.sqrt()
    }
    fn exp_val(self) -> Self {
        self.exp()
    }
    fn ln_val(self) -> Self {
        self.ln()
    }
    fn abs_val(self) -> Self {
        self.abs()
    }
    fn from_f64(v: f64) -> Self {
        v
    }
    fn to_f64(self) -> f64 {
        self
    }
}

impl<T: FloatOps> NdArray<T> {
    /// Create a 1D array with evenly spaced values.
    ///
    /// # Arguments
    /// * `start` - Start value
    /// * `stop` - End value (inclusive)
    /// * `num` - Number of samples
    ///
    /// # Example
    /// ```ignore
    /// let arr = NdArray::linspace(0.0, 1.0, 5);
    /// // [0.0, 0.25, 0.5, 0.75, 1.0]
    /// ```
    pub fn linspace(start: T, stop: T, num: usize) -> Self {
        if num == 0 {
            return Self {
                data: vec![],
                shape: Shape::new(vec![0]),
            };
        }
        if num == 1 {
            return Self {
                data: vec![start],
                shape: Shape::new(vec![1]),
            };
        }

        let start_f = start.to_f64();
        let stop_f = stop.to_f64();
        let step = (stop_f - start_f) / (num - 1) as f64;

        let data: Vec<T> = (0..num)
            .map(|i| T::from_f64(start_f + step * i as f64))
            .collect();

        Self {
            data,
            shape: Shape::new(vec![num]),
        }
    }

    /// Element-wise square root.
    pub fn sqrt(&self) -> Self {
        Self {
            data: self.data.iter().map(|&x| x.sqrt_val()).collect(),
            shape: self.shape.clone(),
        }
    }

    /// Element-wise exponential (e^x).
    pub fn exp(&self) -> Self {
        Self {
            data: self.data.iter().map(|&x| x.exp_val()).collect(),
            shape: self.shape.clone(),
        }
    }

    /// Element-wise natural logarithm.
    pub fn log(&self) -> Self {
        Self {
            data: self.data.iter().map(|&x| x.ln_val()).collect(),
            shape: self.shape.clone(),
        }
    }

    /// Element-wise absolute value.
    pub fn abs(&self) -> Self {
        Self {
            data: self.data.iter().map(|&x| x.abs_val()).collect(),
            shape: self.shape.clone(),
        }
    }

    /// Clip values to a range [min, max].
    pub fn clip(&self, min: T, max: T) -> Self
    where
        T: PartialOrd,
    {
        let data: Vec<T> = self
            .data
            .iter()
            .map(|&x| {
                if x < min {
                    min
                } else if x > max {
                    max
                } else {
                    x
                }
            })
            .collect();

        Self {
            data,
            shape: self.shape.clone(),
        }
    }

    /// Conditional selection: where(condition, x, y).
    ///
    /// Returns elements from `self` where condition is true, otherwise from `other`.
    pub fn where_cond(&self, condition: &[bool], other: &NdArray<T>) -> Result<Self> {
        use super::error::ArrayError;

        if condition.len() != self.size() || other.size() != self.size() {
            return Err(ArrayError::ShapeMismatch {
                expected: self.dims().to_vec(),
                got: vec![condition.len()],
            });
        }

        let data: Vec<T> = self
            .data
            .iter()
            .zip(other.data.iter())
            .zip(condition.iter())
            .map(|((&a, &b), &cond)| if cond { a } else { b })
            .collect();

        Ok(Self {
            data,
            shape: self.shape.clone(),
        })
    }

    /// Power operation: self^exponent.
    pub fn pow(&self, exponent: T) -> Self {
        let exp_f = exponent.to_f64();
        Self {
            data: self
                .data
                .iter()
                .map(|&x| T::from_f64(x.to_f64().powf(exp_f)))
                .collect(),
            shape: self.shape.clone(),
        }
    }

    /// Element-wise sine.
    pub fn sin(&self) -> Self {
        Self {
            data: self
                .data
                .iter()
                .map(|&x| T::from_f64(x.to_f64().sin()))
                .collect(),
            shape: self.shape.clone(),
        }
    }

    /// Element-wise cosine.
    pub fn cos(&self) -> Self {
        Self {
            data: self
                .data
                .iter()
                .map(|&x| T::from_f64(x.to_f64().cos()))
                .collect(),
            shape: self.shape.clone(),
        }
    }

    /// Element-wise tangent.
    pub fn tan(&self) -> Self {
        Self {
            data: self
                .data
                .iter()
                .map(|&x| T::from_f64(x.to_f64().tan()))
                .collect(),
            shape: self.shape.clone(),
        }
    }
}

// Integer absolute value
impl NdArray<i32> {
    /// Element-wise absolute value for i32.
    pub fn abs_int(&self) -> Self {
        Self {
            data: self.data.iter().map(|&x| x.abs()).collect(),
            shape: self.shape.clone(),
        }
    }
}

impl NdArray<i64> {
    /// Element-wise absolute value for i64.
    pub fn abs_int(&self) -> Self {
        Self {
            data: self.data.iter().map(|&x| x.abs()).collect(),
            shape: self.shape.clone(),
        }
    }
}
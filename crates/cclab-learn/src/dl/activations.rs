//! Activation function layers.

use super::nn::Layer;
use super::tensor::Tensor;

/// Tanh activation: tanh(x) = (e^x - e^-x) / (e^x + e^-x).
pub struct Tanh;

impl Layer for Tanh {
    fn forward(&self, input: &Tensor) -> Tensor {
        let data: Vec<f64> = input.data.iter().map(|&v| v.tanh()).collect();
        Tensor::new(data, input.shape.clone())
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![]
    }
}

/// GELU activation: x * Phi(x), approximated as
/// 0.5 * x * (1 + tanh(sqrt(2/pi) * (x + 0.044715 * x^3))).
pub struct GELU;

impl Layer for GELU {
    fn forward(&self, input: &Tensor) -> Tensor {
        let sqrt_2_over_pi = (2.0 / std::f64::consts::PI).sqrt();
        let data: Vec<f64> = input
            .data
            .iter()
            .map(|&x| {
                let inner = sqrt_2_over_pi * (x + 0.044715 * x * x * x);
                0.5 * x * (1.0 + inner.tanh())
            })
            .collect();
        Tensor::new(data, input.shape.clone())
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![]
    }
}

/// Softmax activation: softmax(x_i) = exp(x_i) / sum(exp(x_j)).
///
/// Applied along the last dimension.
pub struct Softmax;

impl Softmax {
    /// Apply softmax to a 1-D slice.
    fn softmax_slice(data: &[f64]) -> Vec<f64> {
        let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = data.iter().map(|&v| (v - max).exp()).collect();
        let sum: f64 = exps.iter().sum();
        exps.iter().map(|&e| e / sum).collect()
    }
}

impl Layer for Softmax {
    fn forward(&self, input: &Tensor) -> Tensor {
        if input.shape.is_empty() {
            return Tensor::new(vec![1.0], vec![1]);
        }
        let last_dim = *input.shape.last().unwrap();
        let num_slices = input.numel() / last_dim;
        let mut out = Vec::with_capacity(input.numel());
        for i in 0..num_slices {
            let start = i * last_dim;
            let end = start + last_dim;
            let slice = &input.data[start..end];
            out.extend(Self::softmax_slice(slice));
        }
        Tensor::new(out, input.shape.clone())
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![]
    }
}

/// Leaky ReLU: f(x) = x if x > 0, else negative_slope * x.
pub struct LeakyReLU {
    pub negative_slope: f64,
}

impl LeakyReLU {
    pub fn new(negative_slope: f64) -> Self {
        Self { negative_slope }
    }
}

impl Default for LeakyReLU {
    fn default() -> Self {
        Self {
            negative_slope: 0.01,
        }
    }
}

impl Layer for LeakyReLU {
    fn forward(&self, input: &Tensor) -> Tensor {
        let data: Vec<f64> = input
            .data
            .iter()
            .map(|&v| if v > 0.0 { v } else { self.negative_slope * v })
            .collect();
        Tensor::new(data, input.shape.clone())
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tanh_forward() {
        let layer = Tanh;
        let input = Tensor::new(vec![0.0, 1.0, -1.0], vec![3]);
        let output = layer.forward(&input);
        assert_eq!(output.shape, vec![3]);
        assert!((output.data[0] - 0.0).abs() < 1e-10);
        assert!((output.data[1] - 1.0_f64.tanh()).abs() < 1e-10);
        assert!((output.data[2] - (-1.0_f64).tanh()).abs() < 1e-10);
    }

    #[test]
    fn test_tanh_range() {
        let layer = Tanh;
        let input = Tensor::new(vec![-100.0, 100.0], vec![2]);
        let output = layer.forward(&input);
        // tanh output is in (-1, 1); for extreme inputs it saturates to +/- 1
        assert!(output.data[0] >= -1.0 && output.data[0] < 0.0);
        assert!(output.data[1] > 0.0 && output.data[1] <= 1.0);
    }

    #[test]
    fn test_gelu_forward() {
        let layer = GELU;
        let input = Tensor::new(vec![0.0, 1.0, -1.0], vec![3]);
        let output = layer.forward(&input);
        assert_eq!(output.shape, vec![3]);
        // GELU(0) = 0
        assert!(output.data[0].abs() < 1e-10);
        // GELU(1) ~ 0.8412
        assert!((output.data[1] - 0.8412).abs() < 0.01);
        // GELU(-1) ~ -0.1588
        assert!((output.data[2] - (-0.1588)).abs() < 0.01);
    }

    #[test]
    fn test_gelu_monotonic_positive() {
        let layer = GELU;
        let input = Tensor::new(vec![0.5, 1.0, 2.0, 3.0], vec![4]);
        let output = layer.forward(&input);
        for i in 1..output.data.len() {
            assert!(output.data[i] > output.data[i - 1]);
        }
    }

    #[test]
    fn test_softmax_sums_to_one() {
        let layer = Softmax;
        let input = Tensor::new(vec![1.0, 2.0, 3.0], vec![3]);
        let output = layer.forward(&input);
        assert_eq!(output.shape, vec![3]);
        let sum: f64 = output.data.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_softmax_2d() {
        let layer = Softmax;
        // 2 rows, 3 cols -- softmax along last dim
        let input = Tensor::new(vec![1.0, 2.0, 3.0, 1.0, 1.0, 1.0], vec![2, 3]);
        let output = layer.forward(&input);
        assert_eq!(output.shape, vec![2, 3]);
        // Each row should sum to 1
        let row1_sum: f64 = output.data[0..3].iter().sum();
        let row2_sum: f64 = output.data[3..6].iter().sum();
        assert!((row1_sum - 1.0).abs() < 1e-10);
        assert!((row2_sum - 1.0).abs() < 1e-10);
        // Second row is uniform
        assert!((output.data[3] - output.data[4]).abs() < 1e-10);
    }

    #[test]
    fn test_softmax_numerical_stability() {
        let layer = Softmax;
        let input = Tensor::new(vec![1000.0, 1001.0, 1002.0], vec![3]);
        let output = layer.forward(&input);
        let sum: f64 = output.data.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
        // All values should be finite
        assert!(output.data.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn test_leaky_relu_positive() {
        let layer = LeakyReLU::new(0.01);
        let input = Tensor::new(vec![1.0, 2.0, 3.0], vec![3]);
        let output = layer.forward(&input);
        assert_eq!(output.data, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_leaky_relu_negative() {
        let layer = LeakyReLU::new(0.1);
        let input = Tensor::new(vec![-1.0, -2.0, 0.0, 1.0], vec![4]);
        let output = layer.forward(&input);
        assert!((output.data[0] - (-0.1)).abs() < 1e-10);
        assert!((output.data[1] - (-0.2)).abs() < 1e-10);
        assert!((output.data[2] - 0.0).abs() < 1e-10);
        assert!((output.data[3] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_leaky_relu_default() {
        let layer = LeakyReLU::default();
        assert!((layer.negative_slope - 0.01).abs() < 1e-10);
    }
}

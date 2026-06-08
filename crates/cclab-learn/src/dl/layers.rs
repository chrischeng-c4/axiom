//! Additional neural network layers: convolutions, pooling, normalization, etc.

use super::nn::Layer;
use super::tensor::Tensor;

/// 2-D convolution layer.
///
/// Input shape: `[batch, in_channels, height, width]`
/// Output shape: `[batch, out_channels, out_h, out_w]`
/// where `out_h = height - kernel_size + 1`, `out_w = width - kernel_size + 1`.
pub struct Conv2d {
    pub in_channels: usize,
    pub out_channels: usize,
    pub kernel_size: usize,
    /// Shape: `[out_channels, in_channels, kernel_size, kernel_size]`
    pub weight: Tensor,
    /// Shape: `[out_channels]`
    pub bias: Tensor,
}

impl Conv2d {
    pub fn new(in_channels: usize, out_channels: usize, kernel_size: usize, seed: u64) -> Self {
        Self {
            in_channels,
            out_channels,
            kernel_size,
            weight: Tensor::randn(&[out_channels, in_channels, kernel_size, kernel_size], seed),
            bias: Tensor::zeros(&[out_channels]),
        }
    }
}

impl Layer for Conv2d {
    fn forward(&self, input: &Tensor) -> Tensor {
        assert_eq!(input.ndim(), 4, "Conv2d expects 4-D input [N,C,H,W]");
        let batch = input.shape[0];
        let _ic = input.shape[1];
        let h = input.shape[2];
        let w = input.shape[3];
        let ks = self.kernel_size;
        let out_h = h - ks + 1;
        let out_w = w - ks + 1;
        let oc = self.out_channels;
        let ic = self.in_channels;

        let mut out = vec![0.0; batch * oc * out_h * out_w];

        for n in 0..batch {
            for o in 0..oc {
                for oh in 0..out_h {
                    for ow in 0..out_w {
                        let mut sum = self.bias.data[o];
                        for c in 0..ic {
                            for kh in 0..ks {
                                for kw in 0..ks {
                                    let ih = oh + kh;
                                    let iw = ow + kw;
                                    let i_idx = n * ic * h * w + c * h * w + ih * w + iw;
                                    let w_idx = o * ic * ks * ks + c * ks * ks + kh * ks + kw;
                                    sum += input.data[i_idx] * self.weight.data[w_idx];
                                }
                            }
                        }
                        let o_idx = n * oc * out_h * out_w + o * out_h * out_w + oh * out_w + ow;
                        out[o_idx] = sum;
                    }
                }
            }
        }

        Tensor::new(out, vec![batch, oc, out_h, out_w])
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.weight, &mut self.bias]
    }
}

/// 1-D convolution layer.
///
/// Input shape: `[batch, in_channels, length]`
/// Output shape: `[batch, out_channels, out_len]`
/// where `out_len = length - kernel_size + 1`.
pub struct Conv1d {
    pub in_channels: usize,
    pub out_channels: usize,
    pub kernel_size: usize,
    /// Shape: `[out_channels, in_channels, kernel_size]`
    pub weight: Tensor,
    /// Shape: `[out_channels]`
    pub bias: Tensor,
}

impl Conv1d {
    pub fn new(in_channels: usize, out_channels: usize, kernel_size: usize, seed: u64) -> Self {
        Self {
            in_channels,
            out_channels,
            kernel_size,
            weight: Tensor::randn(&[out_channels, in_channels, kernel_size], seed),
            bias: Tensor::zeros(&[out_channels]),
        }
    }
}

impl Layer for Conv1d {
    fn forward(&self, input: &Tensor) -> Tensor {
        assert_eq!(input.ndim(), 3, "Conv1d expects 3-D input [N,C,L]");
        let batch = input.shape[0];
        let ic = self.in_channels;
        let len = input.shape[2];
        let ks = self.kernel_size;
        let out_len = len - ks + 1;
        let oc = self.out_channels;

        let mut out = vec![0.0; batch * oc * out_len];

        for n in 0..batch {
            for o in 0..oc {
                for ol in 0..out_len {
                    let mut sum = self.bias.data[o];
                    for c in 0..ic {
                        for k in 0..ks {
                            let i_idx = n * ic * len + c * len + ol + k;
                            let w_idx = o * ic * ks + c * ks + k;
                            sum += input.data[i_idx] * self.weight.data[w_idx];
                        }
                    }
                    let o_idx = n * oc * out_len + o * out_len + ol;
                    out[o_idx] = sum;
                }
            }
        }

        Tensor::new(out, vec![batch, oc, out_len])
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.weight, &mut self.bias]
    }
}

/// Batch normalization layer.
///
/// Input shape: `[batch, channels, ...]`
/// Normalizes across batch dimension for each channel.
pub struct BatchNorm {
    pub num_features: usize,
    pub gamma: Tensor,
    pub beta: Tensor,
    pub running_mean: Vec<f64>,
    pub running_var: Vec<f64>,
    pub eps: f64,
    pub momentum: f64,
    pub training: bool,
}

impl BatchNorm {
    pub fn new(num_features: usize) -> Self {
        Self {
            num_features,
            gamma: Tensor::ones(&[num_features]),
            beta: Tensor::zeros(&[num_features]),
            running_mean: vec![0.0; num_features],
            running_var: vec![1.0; num_features],
            eps: 1e-5,
            momentum: 0.1,
            training: true,
        }
    }

    /// Set training mode.
    pub fn train(&mut self) {
        self.training = true;
    }

    /// Set evaluation mode.
    pub fn eval(&mut self) {
        self.training = false;
    }
}

impl Layer for BatchNorm {
    fn forward(&self, input: &Tensor) -> Tensor {
        // Supports 2-D [N, C] and 4-D [N, C, H, W]
        assert!(input.ndim() >= 2, "BatchNorm expects at least 2-D input");
        let batch = input.shape[0];
        let channels = input.shape[1];
        let spatial: usize = input.shape[2..].iter().product();
        let spatial = if spatial == 0 { 1 } else { spatial };

        let mut out = input.data.clone();

        for c in 0..channels {
            // Collect all values for this channel
            let mut vals = Vec::with_capacity(batch * spatial);
            for n in 0..batch {
                for s in 0..spatial {
                    let idx = n * channels * spatial + c * spatial + s;
                    vals.push(input.data[idx]);
                }
            }

            let (mean, var) = if self.training {
                let mean = vals.iter().sum::<f64>() / vals.len() as f64;
                let var =
                    vals.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / vals.len() as f64;
                (mean, var)
            } else {
                (self.running_mean[c], self.running_var[c])
            };

            let inv_std = 1.0 / (var + self.eps).sqrt();
            let gamma = self.gamma.data[c];
            let beta = self.beta.data[c];

            for n in 0..batch {
                for s in 0..spatial {
                    let idx = n * channels * spatial + c * spatial + s;
                    out[idx] = gamma * (input.data[idx] - mean) * inv_std + beta;
                }
            }
        }

        Tensor::new(out, input.shape.clone())
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.gamma, &mut self.beta]
    }
}

/// Dropout layer: randomly zeroes elements during training.
///
/// In inference mode, returns input unchanged.
pub struct Dropout {
    pub p: f64,
    pub training: bool,
    seed: u64,
}

impl Dropout {
    pub fn new(p: f64, seed: u64) -> Self {
        assert!(
            (0.0..1.0).contains(&p),
            "Dropout probability must be in [0, 1)"
        );
        Self {
            p,
            training: true,
            seed,
        }
    }

    pub fn train(&mut self) {
        self.training = true;
    }

    pub fn eval(&mut self) {
        self.training = false;
    }
}

impl Layer for Dropout {
    fn forward(&self, input: &Tensor) -> Tensor {
        if !self.training || self.p == 0.0 {
            return input.clone();
        }
        let scale = 1.0 / (1.0 - self.p);
        let mut rng = self.seed;
        let data: Vec<f64> = input
            .data
            .iter()
            .map(|&v| {
                rng = rng
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u = (rng as f64) / (u64::MAX as f64);
                if u < self.p {
                    0.0
                } else {
                    v * scale
                }
            })
            .collect();
        Tensor::new(data, input.shape.clone())
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![]
    }
}

/// Max pooling 2-D layer.
///
/// Input shape: `[batch, channels, height, width]`
/// Output shape: `[batch, channels, out_h, out_w]`
pub struct MaxPool2d {
    pub kernel_size: usize,
    pub stride: usize,
}

impl MaxPool2d {
    pub fn new(kernel_size: usize, stride: usize) -> Self {
        Self {
            kernel_size,
            stride,
        }
    }
}

impl Layer for MaxPool2d {
    fn forward(&self, input: &Tensor) -> Tensor {
        assert_eq!(input.ndim(), 4, "MaxPool2d expects 4-D input [N,C,H,W]");
        let batch = input.shape[0];
        let channels = input.shape[1];
        let h = input.shape[2];
        let w = input.shape[3];
        let ks = self.kernel_size;
        let stride = self.stride;
        let out_h = (h - ks) / stride + 1;
        let out_w = (w - ks) / stride + 1;

        let mut out = Vec::with_capacity(batch * channels * out_h * out_w);

        for n in 0..batch {
            for c in 0..channels {
                for oh in 0..out_h {
                    for ow in 0..out_w {
                        let mut max_val = f64::NEG_INFINITY;
                        for kh in 0..ks {
                            for kw in 0..ks {
                                let ih = oh * stride + kh;
                                let iw = ow * stride + kw;
                                let idx = n * channels * h * w + c * h * w + ih * w + iw;
                                max_val = max_val.max(input.data[idx]);
                            }
                        }
                        out.push(max_val);
                    }
                }
            }
        }

        Tensor::new(out, vec![batch, channels, out_h, out_w])
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![]
    }
}

/// Average pooling 2-D layer.
///
/// Input shape: `[batch, channels, height, width]`
/// Output shape: `[batch, channels, out_h, out_w]`
pub struct AvgPool2d {
    pub kernel_size: usize,
    pub stride: usize,
}

impl AvgPool2d {
    pub fn new(kernel_size: usize, stride: usize) -> Self {
        Self {
            kernel_size,
            stride,
        }
    }
}

impl Layer for AvgPool2d {
    fn forward(&self, input: &Tensor) -> Tensor {
        assert_eq!(input.ndim(), 4, "AvgPool2d expects 4-D input [N,C,H,W]");
        let batch = input.shape[0];
        let channels = input.shape[1];
        let h = input.shape[2];
        let w = input.shape[3];
        let ks = self.kernel_size;
        let stride = self.stride;
        let out_h = (h - ks) / stride + 1;
        let out_w = (w - ks) / stride + 1;
        let pool_size = (ks * ks) as f64;

        let mut out = Vec::with_capacity(batch * channels * out_h * out_w);

        for n in 0..batch {
            for c in 0..channels {
                for oh in 0..out_h {
                    for ow in 0..out_w {
                        let mut sum = 0.0;
                        for kh in 0..ks {
                            for kw in 0..ks {
                                let ih = oh * stride + kh;
                                let iw = ow * stride + kw;
                                let idx = n * channels * h * w + c * h * w + ih * w + iw;
                                sum += input.data[idx];
                            }
                        }
                        out.push(sum / pool_size);
                    }
                }
            }
        }

        Tensor::new(out, vec![batch, channels, out_h, out_w])
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![]
    }
}

/// Flatten layer: reshapes any input to `[batch, -1]`.
///
/// Assumes the first dimension is the batch dimension.
pub struct Flatten;

impl Layer for Flatten {
    fn forward(&self, input: &Tensor) -> Tensor {
        let batch = input.shape[0];
        let flat_size = input.numel() / batch;
        Tensor::new(input.data.clone(), vec![batch, flat_size])
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![]
    }
}

/// Embedding layer: lookup table for dense vectors.
///
/// Input: integer tensor (encoded as f64) with any shape.
/// Output: `[..input_shape, embedding_dim]`.
pub struct Embedding {
    pub num_embeddings: usize,
    pub embedding_dim: usize,
    pub weight: Tensor,
}

impl Embedding {
    pub fn new(num_embeddings: usize, embedding_dim: usize, seed: u64) -> Self {
        Self {
            num_embeddings,
            embedding_dim,
            weight: Tensor::randn(&[num_embeddings, embedding_dim], seed),
        }
    }
}

impl Layer for Embedding {
    fn forward(&self, input: &Tensor) -> Tensor {
        let mut out = Vec::with_capacity(input.numel() * self.embedding_dim);
        let mut new_shape = input.shape.clone();
        new_shape.push(self.embedding_dim);

        for &idx_f in &input.data {
            let idx = idx_f as usize;
            assert!(
                idx < self.num_embeddings,
                "Embedding index {} out of range (num_embeddings={})",
                idx,
                self.num_embeddings
            );
            let start = idx * self.embedding_dim;
            let end = start + self.embedding_dim;
            out.extend_from_slice(&self.weight.data[start..end]);
        }

        Tensor::new(out, new_shape)
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.weight]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Conv2d ---

    #[test]
    fn test_conv2d_output_shape() {
        let conv = Conv2d::new(1, 2, 3, 42);
        // Input: [1, 1, 5, 5]
        let input = Tensor::ones(&[1, 1, 5, 5]);
        let output = conv.forward(&input);
        // out_h = 5 - 3 + 1 = 3, out_w = 3
        assert_eq!(output.shape, vec![1, 2, 3, 3]);
    }

    #[test]
    fn test_conv2d_known_values() {
        let mut conv = Conv2d::new(1, 1, 2, 0);
        // Set known weights: all ones
        conv.weight = Tensor::ones(&[1, 1, 2, 2]);
        conv.bias = Tensor::zeros(&[1]);
        // Input: [1, 1, 3, 3] all ones
        let input = Tensor::ones(&[1, 1, 3, 3]);
        let output = conv.forward(&input);
        // Each output should be 4.0 (sum of 2x2 window of ones)
        assert_eq!(output.shape, vec![1, 1, 2, 2]);
        for &v in &output.data {
            assert!((v - 4.0).abs() < 1e-10);
        }
    }

    // --- Conv1d ---

    #[test]
    fn test_conv1d_output_shape() {
        let conv = Conv1d::new(2, 3, 3, 42);
        // Input: [1, 2, 10]
        let input = Tensor::ones(&[1, 2, 10]);
        let output = conv.forward(&input);
        // out_len = 10 - 3 + 1 = 8
        assert_eq!(output.shape, vec![1, 3, 8]);
    }

    #[test]
    fn test_conv1d_known_values() {
        let mut conv = Conv1d::new(1, 1, 3, 0);
        conv.weight = Tensor::ones(&[1, 1, 3]);
        conv.bias = Tensor::zeros(&[1]);
        let input = Tensor::ones(&[1, 1, 5]);
        let output = conv.forward(&input);
        assert_eq!(output.shape, vec![1, 1, 3]);
        for &v in &output.data {
            assert!((v - 3.0).abs() < 1e-10);
        }
    }

    // --- BatchNorm ---

    #[test]
    fn test_batchnorm_output_shape() {
        let bn = BatchNorm::new(3);
        let input = Tensor::randn(&[4, 3], 42);
        let output = bn.forward(&input);
        assert_eq!(output.shape, vec![4, 3]);
    }

    #[test]
    fn test_batchnorm_normalizes() {
        let bn = BatchNorm::new(1);
        // 4 samples, 1 channel
        let input = Tensor::new(vec![1.0, 3.0, 5.0, 7.0], vec![4, 1]);
        let output = bn.forward(&input);
        // Mean of output channel should be ~0
        let mean: f64 = output.data.iter().sum::<f64>() / 4.0;
        assert!(mean.abs() < 1e-10, "mean should be ~0, got {}", mean);
    }

    // --- Dropout ---

    #[test]
    fn test_dropout_eval_passthrough() {
        let mut dropout = Dropout::new(0.5, 42);
        dropout.eval();
        let input = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]);
        let output = dropout.forward(&input);
        assert_eq!(output.data, input.data);
    }

    #[test]
    fn test_dropout_training_zeros_some() {
        let dropout = Dropout::new(0.5, 42);
        let input = Tensor::ones(&[100]);
        let output = dropout.forward(&input);
        // Some elements should be zero
        let zeros = output.data.iter().filter(|&&v| v == 0.0).count();
        assert!(zeros > 10, "Expected some zeros, got {}", zeros);
        assert!(zeros < 90, "Expected some non-zeros, got {} zeros", zeros);
    }

    // --- MaxPool2d ---

    #[test]
    fn test_maxpool2d_output_shape() {
        let pool = MaxPool2d::new(2, 2);
        let input = Tensor::ones(&[1, 1, 4, 4]);
        let output = pool.forward(&input);
        assert_eq!(output.shape, vec![1, 1, 2, 2]);
    }

    #[test]
    fn test_maxpool2d_picks_max() {
        let pool = MaxPool2d::new(2, 2);
        #[rustfmt::skip]
        let input = Tensor::new(
            vec![
                1.0, 2.0, 3.0, 4.0,
                5.0, 6.0, 7.0, 8.0,
                9.0, 10.0, 11.0, 12.0,
                13.0, 14.0, 15.0, 16.0,
            ],
            vec![1, 1, 4, 4],
        );
        let output = pool.forward(&input);
        assert_eq!(output.data, vec![6.0, 8.0, 14.0, 16.0]);
    }

    // --- AvgPool2d ---

    #[test]
    fn test_avgpool2d_output_shape() {
        let pool = AvgPool2d::new(2, 2);
        let input = Tensor::ones(&[1, 1, 4, 4]);
        let output = pool.forward(&input);
        assert_eq!(output.shape, vec![1, 1, 2, 2]);
    }

    #[test]
    fn test_avgpool2d_computes_average() {
        let pool = AvgPool2d::new(2, 2);
        #[rustfmt::skip]
        let input = Tensor::new(
            vec![
                1.0, 2.0, 3.0, 4.0,
                5.0, 6.0, 7.0, 8.0,
                9.0, 10.0, 11.0, 12.0,
                13.0, 14.0, 15.0, 16.0,
            ],
            vec![1, 1, 4, 4],
        );
        let output = pool.forward(&input);
        // Top-left 2x2: (1+2+5+6)/4 = 3.5
        assert!((output.data[0] - 3.5).abs() < 1e-10);
        // Top-right 2x2: (3+4+7+8)/4 = 5.5
        assert!((output.data[1] - 5.5).abs() < 1e-10);
    }

    // --- Flatten ---

    #[test]
    fn test_flatten_shape() {
        let flatten = Flatten;
        let input = Tensor::ones(&[2, 3, 4, 5]);
        let output = flatten.forward(&input);
        assert_eq!(output.shape, vec![2, 60]);
        assert_eq!(output.numel(), 120);
    }

    #[test]
    fn test_flatten_preserves_data() {
        let flatten = Flatten;
        let input = Tensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]);
        let output = flatten.forward(&input);
        assert_eq!(output.shape, vec![2, 3]);
        assert_eq!(output.data, input.data);
    }

    // --- Embedding ---

    #[test]
    fn test_embedding_output_shape() {
        let emb = Embedding::new(10, 4, 42);
        // 3 token indices
        let input = Tensor::new(vec![0.0, 5.0, 9.0], vec![3]);
        let output = emb.forward(&input);
        assert_eq!(output.shape, vec![3, 4]);
    }

    #[test]
    fn test_embedding_lookup_correct() {
        let mut emb = Embedding::new(3, 2, 42);
        // Set known weights
        emb.weight = Tensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![3, 2]);
        let input = Tensor::new(vec![2.0, 0.0], vec![2]);
        let output = emb.forward(&input);
        assert_eq!(output.shape, vec![2, 2]);
        // Index 2 -> [5.0, 6.0], Index 0 -> [1.0, 2.0]
        assert_eq!(output.data, vec![5.0, 6.0, 1.0, 2.0]);
    }

    #[test]
    fn test_embedding_2d_input() {
        let emb = Embedding::new(10, 4, 42);
        let input = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]);
        let output = emb.forward(&input);
        assert_eq!(output.shape, vec![2, 2, 4]);
    }
}

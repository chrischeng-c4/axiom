//! Attention mechanisms: Multi-Head Attention.

use super::nn::Layer;
use super::tensor::Tensor;

/// Multi-Head Attention layer.
///
/// Implements scaled dot-product attention:
///   Attention(Q, K, V) = softmax(Q @ K^T / sqrt(d_k)) @ V
///
/// Input shape: `[batch, seq_len, embed_dim]`
/// Output shape: `[batch, seq_len, embed_dim]`
///
/// Uses separate linear projections for Q, K, V and an output projection.
pub struct MultiHeadAttention {
    pub embed_dim: usize,
    pub num_heads: usize,
    pub head_dim: usize,
    /// W_q: `[embed_dim, embed_dim]`
    pub w_q: Tensor,
    /// W_k: `[embed_dim, embed_dim]`
    pub w_k: Tensor,
    /// W_v: `[embed_dim, embed_dim]`
    pub w_v: Tensor,
    /// W_o: `[embed_dim, embed_dim]`
    pub w_o: Tensor,
}

impl MultiHeadAttention {
    pub fn new(embed_dim: usize, num_heads: usize, seed: u64) -> Self {
        assert_eq!(
            embed_dim % num_heads,
            0,
            "embed_dim must be divisible by num_heads"
        );
        Self {
            embed_dim,
            num_heads,
            head_dim: embed_dim / num_heads,
            w_q: Tensor::randn(&[embed_dim, embed_dim], seed),
            w_k: Tensor::randn(&[embed_dim, embed_dim], seed.wrapping_add(1)),
            w_v: Tensor::randn(&[embed_dim, embed_dim], seed.wrapping_add(2)),
            w_o: Tensor::randn(&[embed_dim, embed_dim], seed.wrapping_add(3)),
        }
    }

    /// Linear projection: `[batch, seq_len, in_dim] @ weight^T -> [batch, seq_len, out_dim]`
    /// weight shape: `[out_dim, in_dim]` (row-major).
    fn project(
        input: &[f64],
        weight: &[f64],
        batch: usize,
        seq_len: usize,
        in_dim: usize,
        out_dim: usize,
    ) -> Vec<f64> {
        let mut out = vec![0.0; batch * seq_len * out_dim];
        for n in 0..batch {
            for s in 0..seq_len {
                for o in 0..out_dim {
                    let mut sum = 0.0;
                    for i in 0..in_dim {
                        let x_idx = n * seq_len * in_dim + s * in_dim + i;
                        let w_idx = o * in_dim + i;
                        sum += input[x_idx] * weight[w_idx];
                    }
                    let o_idx = n * seq_len * out_dim + s * out_dim + o;
                    out[o_idx] = sum;
                }
            }
        }
        out
    }

    /// Softmax over last dimension of a row.
    fn softmax_row(row: &[f64]) -> Vec<f64> {
        let max = row.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = row.iter().map(|&v| (v - max).exp()).collect();
        let sum: f64 = exps.iter().sum();
        exps.iter().map(|&e| e / sum).collect()
    }
}

impl Layer for MultiHeadAttention {
    fn forward(&self, input: &Tensor) -> Tensor {
        assert_eq!(input.ndim(), 3, "MHA expects [batch, seq_len, embed_dim]");
        let batch = input.shape[0];
        let seq_len = input.shape[1];
        let ed = self.embed_dim;
        let nh = self.num_heads;
        let hd = self.head_dim;
        let scale = (hd as f64).sqrt();

        // Project Q, K, V
        let q = Self::project(&input.data, &self.w_q.data, batch, seq_len, ed, ed);
        let k = Self::project(&input.data, &self.w_k.data, batch, seq_len, ed, ed);
        let v = Self::project(&input.data, &self.w_v.data, batch, seq_len, ed, ed);

        // Scaled dot-product attention per head
        // q, k, v are [batch, seq_len, embed_dim] laid out flat
        // We split embed_dim into num_heads * head_dim
        let mut attn_out = vec![0.0; batch * seq_len * ed];

        for n in 0..batch {
            for h in 0..nh {
                // For this head, compute attention scores
                // scores[i][j] = sum_d q[n,i,h*hd+d] * k[n,j,h*hd+d] / scale
                let mut scores = vec![0.0; seq_len * seq_len];
                for i in 0..seq_len {
                    for j in 0..seq_len {
                        let mut dot = 0.0;
                        for d in 0..hd {
                            let q_idx = n * seq_len * ed + i * ed + h * hd + d;
                            let k_idx = n * seq_len * ed + j * ed + h * hd + d;
                            dot += q[q_idx] * k[k_idx];
                        }
                        scores[i * seq_len + j] = dot / scale;
                    }
                }

                // Softmax each row
                for i in 0..seq_len {
                    let start = i * seq_len;
                    let row = &scores[start..start + seq_len];
                    let sm = Self::softmax_row(row);

                    // Weighted sum of values
                    for d in 0..hd {
                        let mut val = 0.0;
                        for j in 0..seq_len {
                            let v_idx = n * seq_len * ed + j * ed + h * hd + d;
                            val += sm[j] * v[v_idx];
                        }
                        let o_idx = n * seq_len * ed + i * ed + h * hd + d;
                        attn_out[o_idx] = val;
                    }
                }
            }
        }

        // Output projection
        let final_out = Self::project(&attn_out, &self.w_o.data, batch, seq_len, ed, ed);

        Tensor::new(final_out, vec![batch, seq_len, ed])
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.w_q, &mut self.w_k, &mut self.w_v, &mut self.w_o]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mha_output_shape() {
        let mha = MultiHeadAttention::new(8, 2, 42);
        // batch=2, seq_len=4, embed_dim=8
        let input = Tensor::randn(&[2, 4, 8], 100);
        let output = mha.forward(&input);
        assert_eq!(output.shape, vec![2, 4, 8]);
    }

    #[test]
    fn test_mha_single_head() {
        let mha = MultiHeadAttention::new(4, 1, 42);
        let input = Tensor::ones(&[1, 3, 4]);
        let output = mha.forward(&input);
        assert_eq!(output.shape, vec![1, 3, 4]);
        // All values should be finite
        assert!(output.data.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn test_mha_deterministic() {
        let mha = MultiHeadAttention::new(8, 4, 42);
        let input = Tensor::randn(&[1, 5, 8], 100);
        let out1 = mha.forward(&input);
        let out2 = mha.forward(&input);
        // Same input should give same output
        for (a, b) in out1.data.iter().zip(out2.data.iter()) {
            assert!((a - b).abs() < 1e-10);
        }
    }

    #[test]
    fn test_mha_parameters_count() {
        let mut mha = MultiHeadAttention::new(8, 2, 42);
        let params = mha.parameters();
        // 4 weight matrices: W_q, W_k, W_v, W_o
        assert_eq!(params.len(), 4);
    }
}

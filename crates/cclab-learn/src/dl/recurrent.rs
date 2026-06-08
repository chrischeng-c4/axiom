//! Recurrent neural network layers: LSTM, GRU.

use super::nn::Layer;
use super::tensor::Tensor;

/// Long Short-Term Memory (LSTM) layer.
///
/// Input shape: `[batch, seq_len, input_size]`
/// Output shape: `[batch, seq_len, hidden_size]`
///
/// Implements the standard LSTM equations:
/// - f_t = sigmoid(W_f * [h_{t-1}, x_t] + b_f)
/// - i_t = sigmoid(W_i * [h_{t-1}, x_t] + b_i)
/// - c_tilde = tanh(W_c * [h_{t-1}, x_t] + b_c)
/// - c_t = f_t * c_{t-1} + i_t * c_tilde
/// - o_t = sigmoid(W_o * [h_{t-1}, x_t] + b_o)
/// - h_t = o_t * tanh(c_t)
pub struct LSTM {
    pub input_size: usize,
    pub hidden_size: usize,
    /// Combined weight for input: `[4 * hidden_size, input_size]`
    pub weight_ih: Tensor,
    /// Combined weight for hidden: `[4 * hidden_size, hidden_size]`
    pub weight_hh: Tensor,
    /// Bias for input gates: `[4 * hidden_size]`
    pub bias_ih: Tensor,
    /// Bias for hidden gates: `[4 * hidden_size]`
    pub bias_hh: Tensor,
}

impl LSTM {
    pub fn new(input_size: usize, hidden_size: usize, seed: u64) -> Self {
        Self {
            input_size,
            hidden_size,
            weight_ih: Tensor::randn(&[4 * hidden_size, input_size], seed),
            weight_hh: Tensor::randn(&[4 * hidden_size, hidden_size], seed.wrapping_add(1)),
            bias_ih: Tensor::zeros(&[4 * hidden_size]),
            bias_hh: Tensor::zeros(&[4 * hidden_size]),
        }
    }

    /// Compute gates = W_ih @ x + W_hh @ h + b_ih + b_hh
    /// Returns a vector of length `4 * hidden_size`.
    fn compute_gates(&self, x: &[f64], h: &[f64]) -> Vec<f64> {
        let hs = self.hidden_size;
        let is = self.input_size;
        let gate_size = 4 * hs;
        let mut gates = vec![0.0; gate_size];

        for g in 0..gate_size {
            let mut val = self.bias_ih.data[g] + self.bias_hh.data[g];
            for j in 0..is {
                val += self.weight_ih.data[g * is + j] * x[j];
            }
            for j in 0..hs {
                val += self.weight_hh.data[g * hs + j] * h[j];
            }
            gates[g] = val;
        }
        gates
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

impl Layer for LSTM {
    fn forward(&self, input: &Tensor) -> Tensor {
        assert_eq!(
            input.ndim(),
            3,
            "LSTM expects 3-D input [N, seq_len, input_size]"
        );
        let batch = input.shape[0];
        let seq_len = input.shape[1];
        let hs = self.hidden_size;

        let mut output = vec![0.0; batch * seq_len * hs];

        for n in 0..batch {
            let mut h = vec![0.0; hs];
            let mut c = vec![0.0; hs];

            for t in 0..seq_len {
                let x_start = n * seq_len * self.input_size + t * self.input_size;
                let x_end = x_start + self.input_size;
                let x = &input.data[x_start..x_end];

                let gates = self.compute_gates(x, &h);

                // Split gates: i, f, g (cell candidate), o
                let mut new_h = vec![0.0; hs];
                let mut new_c = vec![0.0; hs];

                for j in 0..hs {
                    let i_gate = sigmoid(gates[j]);
                    let f_gate = sigmoid(gates[hs + j]);
                    let g_gate = gates[2 * hs + j].tanh();
                    let o_gate = sigmoid(gates[3 * hs + j]);

                    new_c[j] = f_gate * c[j] + i_gate * g_gate;
                    new_h[j] = o_gate * new_c[j].tanh();
                }

                h = new_h;
                c = new_c;

                let o_start = n * seq_len * hs + t * hs;
                output[o_start..o_start + hs].copy_from_slice(&h);
            }
        }

        Tensor::new(output, vec![batch, seq_len, hs])
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![
            &mut self.weight_ih,
            &mut self.weight_hh,
            &mut self.bias_ih,
            &mut self.bias_hh,
        ]
    }
}

/// Gated Recurrent Unit (GRU) layer.
///
/// Input shape: `[batch, seq_len, input_size]`
/// Output shape: `[batch, seq_len, hidden_size]`
///
/// Implements:
/// - r_t = sigmoid(W_ir @ x_t + W_hr @ h_{t-1} + b_r)
/// - z_t = sigmoid(W_iz @ x_t + W_hz @ h_{t-1} + b_z)
/// - n_t = tanh(W_in @ x_t + r_t * (W_hn @ h_{t-1}) + b_n)
/// - h_t = (1 - z_t) * n_t + z_t * h_{t-1}
pub struct GRU {
    pub input_size: usize,
    pub hidden_size: usize,
    /// Combined weight for input: `[3 * hidden_size, input_size]`
    pub weight_ih: Tensor,
    /// Combined weight for hidden: `[3 * hidden_size, hidden_size]`
    pub weight_hh: Tensor,
    /// Bias for input: `[3 * hidden_size]`
    pub bias_ih: Tensor,
    /// Bias for hidden: `[3 * hidden_size]`
    pub bias_hh: Tensor,
}

impl GRU {
    pub fn new(input_size: usize, hidden_size: usize, seed: u64) -> Self {
        Self {
            input_size,
            hidden_size,
            weight_ih: Tensor::randn(&[3 * hidden_size, input_size], seed),
            weight_hh: Tensor::randn(&[3 * hidden_size, hidden_size], seed.wrapping_add(1)),
            bias_ih: Tensor::zeros(&[3 * hidden_size]),
            bias_hh: Tensor::zeros(&[3 * hidden_size]),
        }
    }
}

impl Layer for GRU {
    fn forward(&self, input: &Tensor) -> Tensor {
        assert_eq!(
            input.ndim(),
            3,
            "GRU expects 3-D input [N, seq_len, input_size]"
        );
        let batch = input.shape[0];
        let seq_len = input.shape[1];
        let hs = self.hidden_size;
        let is = self.input_size;

        let mut output = vec![0.0; batch * seq_len * hs];

        for n in 0..batch {
            let mut h = vec![0.0; hs];

            for t in 0..seq_len {
                let x_start = n * seq_len * is + t * is;
                let x = &input.data[x_start..x_start + is];

                // Compute W_ih @ x + b_ih
                let mut ih = vec![0.0; 3 * hs];
                for g in 0..3 * hs {
                    let mut val = self.bias_ih.data[g];
                    for j in 0..is {
                        val += self.weight_ih.data[g * is + j] * x[j];
                    }
                    ih[g] = val;
                }

                // Compute W_hh @ h + b_hh
                let mut hh = vec![0.0; 3 * hs];
                for g in 0..3 * hs {
                    let mut val = self.bias_hh.data[g];
                    for j in 0..hs {
                        val += self.weight_hh.data[g * hs + j] * h[j];
                    }
                    hh[g] = val;
                }

                let mut new_h = vec![0.0; hs];
                for j in 0..hs {
                    let r = sigmoid(ih[j] + hh[j]);
                    let z = sigmoid(ih[hs + j] + hh[hs + j]);
                    let n_val = (ih[2 * hs + j] + r * hh[2 * hs + j]).tanh();
                    new_h[j] = (1.0 - z) * n_val + z * h[j];
                }

                h = new_h;
                let o_start = n * seq_len * hs + t * hs;
                output[o_start..o_start + hs].copy_from_slice(&h);
            }
        }

        Tensor::new(output, vec![batch, seq_len, hs])
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![
            &mut self.weight_ih,
            &mut self.weight_hh,
            &mut self.bias_ih,
            &mut self.bias_hh,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- LSTM ---

    #[test]
    fn test_lstm_output_shape() {
        let lstm = LSTM::new(4, 8, 42);
        // batch=2, seq_len=5, input_size=4
        let input = Tensor::randn(&[2, 5, 4], 100);
        let output = lstm.forward(&input);
        assert_eq!(output.shape, vec![2, 5, 8]);
    }

    #[test]
    fn test_lstm_zero_input() {
        let lstm = LSTM::new(3, 4, 42);
        let input = Tensor::zeros(&[1, 3, 3]);
        let output = lstm.forward(&input);
        assert_eq!(output.shape, vec![1, 3, 4]);
        // With zero input, zero initial state, and zero bias, output should be zero
        for &v in &output.data {
            assert!(v.abs() < 1e-10, "Expected ~0, got {}", v);
        }
    }

    #[test]
    fn test_lstm_produces_bounded_output() {
        let lstm = LSTM::new(2, 3, 42);
        let input = Tensor::ones(&[1, 4, 2]);
        let output = lstm.forward(&input);
        // LSTM output is bounded by tanh, so |h_t| <= 1
        for &v in &output.data {
            assert!(
                v.abs() <= 1.0 + 1e-10,
                "LSTM output should be bounded, got {}",
                v
            );
        }
    }

    // --- GRU ---

    #[test]
    fn test_gru_output_shape() {
        let gru = GRU::new(4, 8, 42);
        let input = Tensor::randn(&[2, 5, 4], 100);
        let output = gru.forward(&input);
        assert_eq!(output.shape, vec![2, 5, 8]);
    }

    #[test]
    fn test_gru_zero_input() {
        let gru = GRU::new(3, 4, 42);
        let input = Tensor::zeros(&[1, 3, 3]);
        let output = gru.forward(&input);
        assert_eq!(output.shape, vec![1, 3, 4]);
        // With zero input, zero hidden, zero bias: r=sigmoid(0)=0.5, z=sigmoid(0)=0.5
        // n = tanh(0 + 0.5*0) = 0, h = (1-0.5)*0 + 0.5*0 = 0
        for &v in &output.data {
            assert!(v.abs() < 1e-10, "Expected ~0, got {}", v);
        }
    }

    #[test]
    fn test_gru_different_from_zero() {
        let gru = GRU::new(2, 3, 42);
        let input = Tensor::ones(&[1, 4, 2]);
        let output = gru.forward(&input);
        // With non-zero input and random weights, output should be non-trivial
        let any_nonzero = output.data.iter().any(|&v| v.abs() > 1e-6);
        assert!(
            any_nonzero,
            "GRU with non-zero input should produce non-zero output"
        );
    }
}

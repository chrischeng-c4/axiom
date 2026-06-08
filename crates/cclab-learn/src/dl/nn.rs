//! Neural network layers and loss functions.

use super::tensor::Tensor;

/// A neural network layer.
pub trait Layer {
    /// Forward pass.
    fn forward(&self, input: &Tensor) -> Tensor;

    /// Return mutable references to trainable parameters.
    fn parameters(&mut self) -> Vec<&mut Tensor>;
}

/// Fully-connected (dense) layer.
pub struct Linear {
    pub weight: Tensor,
    pub bias: Tensor,
}

impl Linear {
    /// Create a linear layer with random weights.
    pub fn new(in_features: usize, out_features: usize, seed: u64) -> Self {
        Self {
            weight: Tensor::randn(&[in_features, out_features], seed),
            bias: Tensor::zeros(&[1, out_features]),
        }
    }
}

impl Layer for Linear {
    fn forward(&self, input: &Tensor) -> Tensor {
        // input: (batch, in_features) @ weight: (in_features, out_features) + bias
        let out = input.matmul(&self.weight);
        // Broadcast add bias
        let mut data = out.data.clone();
        let n = self.bias.numel();
        for (i, v) in data.iter_mut().enumerate() {
            *v += self.bias.data[i % n];
        }
        Tensor::new(data, out.shape)
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.weight, &mut self.bias]
    }
}

/// ReLU activation layer.
pub struct ReLU;

impl Layer for ReLU {
    fn forward(&self, input: &Tensor) -> Tensor {
        input.relu()
    }
    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![]
    }
}

/// Sigmoid activation layer.
pub struct Sigmoid;

impl Layer for Sigmoid {
    fn forward(&self, input: &Tensor) -> Tensor {
        input.sigmoid()
    }
    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![]
    }
}

/// Sequential container for layers.
pub struct Sequential {
    pub layers: Vec<Box<dyn Layer>>,
}

impl Sequential {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn add(mut self, layer: Box<dyn Layer>) -> Self {
        self.layers.push(layer);
        self
    }
}

impl Default for Sequential {
    fn default() -> Self {
        Self::new()
    }
}

impl Layer for Sequential {
    fn forward(&self, input: &Tensor) -> Tensor {
        let mut x = input.clone();
        for layer in &self.layers {
            x = layer.forward(&x);
        }
        x
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        let mut params = Vec::new();
        for layer in &mut self.layers {
            params.extend(layer.parameters());
        }
        params
    }
}

/// Mean Squared Error loss.
pub struct MSELoss;

impl MSELoss {
    /// Compute MSE loss and gradient w.r.t. predictions.
    pub fn compute(pred: &Tensor, target: &Tensor) -> (f64, Tensor) {
        let diff = pred.sub(target);
        let n = diff.numel() as f64;
        let loss = diff.data.iter().map(|v| v * v).sum::<f64>() / n;
        let grad_data: Vec<f64> = diff.data.iter().map(|v| 2.0 * v / n).collect();
        let grad = Tensor::new(grad_data, diff.shape);
        (loss, grad)
    }
}

/// Cross-Entropy loss (for binary classification after sigmoid).
pub struct CrossEntropyLoss;

impl CrossEntropyLoss {
    /// Compute binary cross-entropy loss.
    pub fn compute(pred: &Tensor, target: &Tensor) -> (f64, Tensor) {
        let n = pred.numel() as f64;
        let eps = 1e-7;
        let mut loss = 0.0;
        let mut grad_data = Vec::with_capacity(pred.numel());

        for i in 0..pred.numel() {
            let p = pred.data[i].clamp(eps, 1.0 - eps);
            let t = target.data[i];
            loss -= t * p.ln() + (1.0 - t) * (1.0 - p).ln();
            grad_data.push((-t / p + (1.0 - t) / (1.0 - p)) / n);
        }

        (loss / n, Tensor::new(grad_data, pred.shape.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_forward() {
        let layer = Linear::new(3, 2, 42);
        let input = Tensor::ones(&[1, 3]);
        let output = layer.forward(&input);
        assert_eq!(output.shape, vec![1, 2]);
    }

    #[test]
    fn test_sequential() {
        let model = Sequential::new()
            .add(Box::new(Linear::new(4, 3, 42)))
            .add(Box::new(ReLU))
            .add(Box::new(Linear::new(3, 1, 43)));

        let input = Tensor::ones(&[2, 4]);
        let output = model.forward(&input);
        assert_eq!(output.shape, vec![2, 1]);
    }

    #[test]
    fn test_mse_loss() {
        let pred = Tensor::new(vec![1.0, 2.0, 3.0], vec![3]);
        let target = Tensor::new(vec![1.0, 2.0, 3.0], vec![3]);
        let (loss, _) = MSELoss::compute(&pred, &target);
        assert!(loss.abs() < 1e-10);
    }

    #[test]
    fn test_mse_loss_nonzero() {
        let pred = Tensor::new(vec![0.0], vec![1]);
        let target = Tensor::new(vec![1.0], vec![1]);
        let (loss, grad) = MSELoss::compute(&pred, &target);
        assert!((loss - 1.0).abs() < 1e-10);
        assert!((grad.data[0] - (-2.0)).abs() < 1e-10);
    }

    #[test]
    fn test_cross_entropy() {
        let pred = Tensor::new(vec![0.9, 0.1], vec![2]);
        let target = Tensor::new(vec![1.0, 0.0], vec![2]);
        let (loss, _) = CrossEntropyLoss::compute(&pred, &target);
        assert!(loss > 0.0 && loss < 1.0);
    }
}

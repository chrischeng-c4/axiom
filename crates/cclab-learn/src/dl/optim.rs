//! Optimizers for deep learning.

use super::tensor::{Tape, Tensor};
use std::cell::RefCell;
use std::rc::Rc;

/// Stochastic Gradient Descent optimizer.
pub struct SGD {
    pub lr: f64,
}

impl SGD {
    pub fn new(lr: f64) -> Self {
        Self { lr }
    }

    /// Update model parameters using computed gradients.
    ///
    /// `param_grads` is a list of (parameter_data, gradient_data) pairs.
    pub fn step(&self, param_grads: &mut [(Vec<f64>, Vec<f64>)]) {
        for (param, grad) in param_grads.iter_mut() {
            for (p, g) in param.iter_mut().zip(grad.iter()) {
                *p -= self.lr * g;
            }
        }
    }

    /// Perform a single training step using tape-based autograd.
    ///
    /// `forward_loss` receives the tape and must return:
    ///   - `params`: vec of tracked parameter tensors
    ///   - `loss_tensor`: scalar loss tensor (for backward)
    ///   - `loss_value`: f64 loss value
    ///
    /// After backward, parameters are updated in-place and returned.
    pub fn train_step<F>(&self, mut forward_loss: F) -> (f64, Vec<Vec<f64>>)
    where
        F: FnMut(&Rc<RefCell<Tape>>) -> (Vec<Tensor>, Tensor, f64),
    {
        let tape = Rc::new(RefCell::new(Tape::new()));
        let (params, loss_tensor, loss_val) = forward_loss(&tape);

        // Backward pass
        loss_tensor.backward();

        // Collect updated parameters
        let mut updated = Vec::with_capacity(params.len());
        let tape_ref = tape.borrow();
        for p in &params {
            let mut new_data = p.data.clone();
            if let Some(id) = p.id() {
                if let Some(grad) = tape_ref.grad(id) {
                    for (d, g) in new_data.iter_mut().zip(grad.iter()) {
                        *d -= self.lr * g;
                    }
                }
            }
            updated.push(new_data);
        }

        (loss_val, updated)
    }
}

/// Adam optimizer with first and second moment estimates.
///
/// Implements the Adam algorithm (Kingma & Ba, 2014).
pub struct Adam {
    pub lr: f64,
    pub beta1: f64,
    pub beta2: f64,
    pub eps: f64,
    /// First moment estimates per parameter group.
    m: Vec<Vec<f64>>,
    /// Second moment estimates per parameter group.
    v: Vec<Vec<f64>>,
    /// Timestep counter.
    t: usize,
}

impl Adam {
    pub fn new(lr: f64, beta1: f64, beta2: f64, eps: f64) -> Self {
        Self {
            lr,
            beta1,
            beta2,
            eps,
            m: Vec::new(),
            v: Vec::new(),
            t: 0,
        }
    }

    /// Create Adam with default hyperparameters (lr=0.001, beta1=0.9, beta2=0.999, eps=1e-8).
    pub fn default_lr(lr: f64) -> Self {
        Self::new(lr, 0.9, 0.999, 1e-8)
    }

    /// Update parameters given (param_data, grad_data) pairs.
    ///
    /// On the first call, moment buffers are initialized to zeros.
    pub fn step(&mut self, param_grads: &mut [(Vec<f64>, Vec<f64>)]) {
        self.t += 1;

        // Initialize moment buffers if needed
        if self.m.len() < param_grads.len() {
            for pg in param_grads.iter() {
                self.m.push(vec![0.0; pg.0.len()]);
                self.v.push(vec![0.0; pg.0.len()]);
            }
        }

        let bc1 = 1.0 - self.beta1.powi(self.t as i32);
        let bc2 = 1.0 - self.beta2.powi(self.t as i32);

        for (i, (param, grad)) in param_grads.iter_mut().enumerate() {
            for (j, (p, g)) in param.iter_mut().zip(grad.iter()).enumerate() {
                self.m[i][j] = self.beta1 * self.m[i][j] + (1.0 - self.beta1) * g;
                self.v[i][j] = self.beta2 * self.v[i][j] + (1.0 - self.beta2) * g * g;
                let m_hat = self.m[i][j] / bc1;
                let v_hat = self.v[i][j] / bc2;
                *p -= self.lr * m_hat / (v_hat.sqrt() + self.eps);
            }
        }
    }
}

/// AdaGrad optimizer: adaptive learning rates per parameter (Duchi et al., 2011).
///
/// Accumulates squared gradients and scales the learning rate inversely.
pub struct AdaGrad {
    pub lr: f64,
    pub eps: f64,
    /// Accumulated squared gradients per parameter group.
    accum: Vec<Vec<f64>>,
}

impl AdaGrad {
    pub fn new(lr: f64, eps: f64) -> Self {
        Self {
            lr,
            eps,
            accum: Vec::new(),
        }
    }

    /// Create AdaGrad with default eps=1e-10.
    pub fn default_lr(lr: f64) -> Self {
        Self::new(lr, 1e-10)
    }

    /// Update parameters given (param_data, grad_data) pairs.
    pub fn step(&mut self, param_grads: &mut [(Vec<f64>, Vec<f64>)]) {
        // Initialize accumulator buffers if needed
        if self.accum.len() < param_grads.len() {
            for pg in param_grads.iter() {
                self.accum.push(vec![0.0; pg.0.len()]);
            }
        }

        for (i, (param, grad)) in param_grads.iter_mut().enumerate() {
            for (j, (p, g)) in param.iter_mut().zip(grad.iter()).enumerate() {
                self.accum[i][j] += g * g;
                *p -= self.lr * g / (self.accum[i][j].sqrt() + self.eps);
            }
        }
    }
}

/// AdamW optimizer: Adam with decoupled weight decay (Loshchilov & Hutter, 2017).
pub struct AdamW {
    pub lr: f64,
    pub beta1: f64,
    pub beta2: f64,
    pub eps: f64,
    pub weight_decay: f64,
    m: Vec<Vec<f64>>,
    v: Vec<Vec<f64>>,
    t: usize,
}

impl AdamW {
    pub fn new(lr: f64, beta1: f64, beta2: f64, eps: f64, weight_decay: f64) -> Self {
        Self {
            lr,
            beta1,
            beta2,
            eps,
            weight_decay,
            m: Vec::new(),
            v: Vec::new(),
            t: 0,
        }
    }

    /// Create AdamW with default hyperparameters.
    pub fn default_lr(lr: f64) -> Self {
        Self::new(lr, 0.9, 0.999, 1e-8, 0.01)
    }

    /// Update parameters given (param_data, grad_data) pairs.
    pub fn step(&mut self, param_grads: &mut [(Vec<f64>, Vec<f64>)]) {
        self.t += 1;

        if self.m.len() < param_grads.len() {
            for pg in param_grads.iter() {
                self.m.push(vec![0.0; pg.0.len()]);
                self.v.push(vec![0.0; pg.0.len()]);
            }
        }

        let bc1 = 1.0 - self.beta1.powi(self.t as i32);
        let bc2 = 1.0 - self.beta2.powi(self.t as i32);

        for (i, (param, grad)) in param_grads.iter_mut().enumerate() {
            for (j, (p, g)) in param.iter_mut().zip(grad.iter()).enumerate() {
                // Decoupled weight decay
                *p -= self.lr * self.weight_decay * *p;
                // Adam update
                self.m[i][j] = self.beta1 * self.m[i][j] + (1.0 - self.beta1) * g;
                self.v[i][j] = self.beta2 * self.v[i][j] + (1.0 - self.beta2) * g * g;
                let m_hat = self.m[i][j] / bc1;
                let v_hat = self.v[i][j] / bc2;
                *p -= self.lr * m_hat / (v_hat.sqrt() + self.eps);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sgd_step() {
        let sgd = SGD::new(0.1);
        let mut param_grads = vec![(vec![1.0, 2.0], vec![0.5, -0.5])];
        sgd.step(&mut param_grads);
        assert!((param_grads[0].0[0] - 0.95).abs() < 1e-10);
        assert!((param_grads[0].0[1] - 2.05).abs() < 1e-10);
    }

    #[test]
    fn test_train_step_autograd() {
        let sgd = SGD::new(0.1);

        // Simple: y = w * x, target = 6, x = 2, w_init = 1
        // loss = (w*x - 6)^2 = (2 - 6)^2 = 16
        // dloss/dw = 2*(w*x - 6)*x = 2*(-4)*2 = -16
        // w_new = 1 - 0.1 * (-16) = 2.6
        let mut w_data = vec![1.0];

        let (loss, updated) = sgd.train_step(|tape| {
            let w = Tensor::tracked(w_data.clone(), vec![1, 1], tape);
            let x = Tensor::tracked(vec![2.0], vec![1, 1], tape);
            let y = x.matmul(&w);
            let target = Tensor::new(vec![6.0], vec![1, 1]);
            let (loss_val, loss_t) = y.mse_loss(&target);
            (vec![w], loss_t, loss_val)
        });

        assert!((loss - 16.0).abs() < 1e-10, "loss: {}", loss);
        w_data = updated[0].clone();
        assert!(
            (w_data[0] - 2.6).abs() < 1e-10,
            "w after step: {}",
            w_data[0]
        );
    }

    #[test]
    fn test_adam_step() {
        let mut adam = Adam::default_lr(0.01);
        let mut param_grads = vec![(vec![5.0, -3.0], vec![1.0, -1.0])];
        adam.step(&mut param_grads);
        // After one step, params should have moved toward lower loss
        // With positive grad, param should decrease; with negative grad, increase
        assert!(
            param_grads[0].0[0] < 5.0,
            "param should decrease with positive grad"
        );
        assert!(
            param_grads[0].0[1] > -3.0,
            "param should increase with negative grad"
        );
    }

    #[test]
    fn test_adam_multiple_steps() {
        let mut adam = Adam::new(0.1, 0.9, 0.999, 1e-8);
        // Constant gradient of 1.0 -- param should keep decreasing
        let mut param_grads = vec![(vec![10.0], vec![1.0])];
        let initial = param_grads[0].0[0];
        for _ in 0..10 {
            param_grads[0].1 = vec![1.0];
            adam.step(&mut param_grads);
        }
        assert!(
            param_grads[0].0[0] < initial,
            "param should decrease after many steps with positive grad"
        );
    }

    #[test]
    fn test_adam_bias_correction() {
        // Test that bias correction makes early steps larger
        let mut adam = Adam::new(0.001, 0.9, 0.999, 1e-8);
        let mut pg = vec![(vec![0.0], vec![10.0])];
        adam.step(&mut pg);
        let step1_delta = (pg[0].0[0]).abs();
        // Bias-corrected step should be close to lr (not lr * (1-beta1))
        assert!(
            step1_delta > 0.0005,
            "bias correction should amplify early step, got {}",
            step1_delta
        );
    }

    #[test]
    fn test_adamw_weight_decay() {
        let mut adamw = AdamW::new(0.01, 0.9, 0.999, 1e-8, 0.1);
        // Zero gradient -- only weight decay should apply
        let mut pg = vec![(vec![10.0], vec![0.0])];
        adamw.step(&mut pg);
        // With wd=0.1, lr=0.01: param -= 0.01*0.1*10 = 0.01
        // So param should be close to 9.99 (plus tiny Adam update)
        assert!(
            pg[0].0[0] < 10.0,
            "weight decay should reduce param, got {}",
            pg[0].0[0]
        );
        assert!(
            (pg[0].0[0] - 9.99).abs() < 0.01,
            "expected ~9.99, got {}",
            pg[0].0[0]
        );
    }

    #[test]
    fn test_adagrad_step() {
        let mut adagrad = AdaGrad::default_lr(0.1);
        let mut param_grads = vec![(vec![5.0, -3.0], vec![1.0, -1.0])];
        adagrad.step(&mut param_grads);
        // After one step: accum = [1.0, 1.0], update = lr * g / (sqrt(accum) + eps)
        // For param[0]: 5.0 - 0.1 * 1.0 / (1.0 + 1e-10) ~ 4.9
        assert!(
            (param_grads[0].0[0] - 4.9).abs() < 0.01,
            "expected ~4.9, got {}",
            param_grads[0].0[0]
        );
        // For param[1]: -3.0 - 0.1 * (-1.0) / (1.0 + 1e-10) ~ -2.9
        assert!(
            (param_grads[0].0[1] - (-2.9)).abs() < 0.01,
            "expected ~-2.9, got {}",
            param_grads[0].0[1]
        );
    }

    #[test]
    fn test_adagrad_decreasing_lr() {
        let mut adagrad = AdaGrad::default_lr(1.0);
        let mut pg = vec![(vec![10.0], vec![2.0])];
        adagrad.step(&mut pg);
        let delta1 = (10.0 - pg[0].0[0]).abs();

        pg[0].1 = vec![2.0];
        let before = pg[0].0[0];
        adagrad.step(&mut pg);
        let delta2 = (before - pg[0].0[0]).abs();

        // Second step should have smaller update due to accumulated gradients
        assert!(
            delta2 < delta1,
            "AdaGrad should have decreasing step sizes: {} vs {}",
            delta2,
            delta1
        );
    }

    #[test]
    fn test_adamw_vs_adam_difference() {
        // AdamW with zero weight_decay should behave like Adam
        let mut adam = Adam::new(0.01, 0.9, 0.999, 1e-8);
        let mut adamw = AdamW::new(0.01, 0.9, 0.999, 1e-8, 0.0);

        let mut pg_adam = vec![(vec![5.0], vec![2.0])];
        let mut pg_adamw = vec![(vec![5.0], vec![2.0])];

        adam.step(&mut pg_adam);
        adamw.step(&mut pg_adamw);

        assert!(
            (pg_adam[0].0[0] - pg_adamw[0].0[0]).abs() < 1e-10,
            "AdamW with wd=0 should match Adam"
        );
    }
}

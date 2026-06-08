//! Tensor with tape-based automatic differentiation.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

/// Operation recorded on the tape for backward pass.
#[derive(Clone)]
enum TapeEntry {
    /// Leaf tensor — no parent ops.
    Leaf,
    /// z = a + b (element-wise).
    Add { a: usize, b: usize },
    /// z = a - b (element-wise).
    Sub { a: usize, b: usize },
    /// z = a * b (element-wise).
    Mul {
        a: usize,
        b: usize,
        a_data: Vec<f64>,
        b_data: Vec<f64>,
    },
    /// z = s * a (scalar multiply).
    Scale { a: usize, s: f64 },
    /// z = a @ b (matrix multiply).
    MatMul {
        a: usize,
        b: usize,
        a_shape: Vec<usize>,
        b_shape: Vec<usize>,
        a_data: Vec<f64>,
        b_data: Vec<f64>,
    },
    /// z = relu(a).
    Relu { a: usize, mask: Vec<bool> },
    /// z = sigmoid(a), stores output values for grad.
    Sigmoid { a: usize, out: Vec<f64> },
    /// z = mse_loss(pred, target).
    MseLoss {
        pred: usize,
        diff: Vec<f64>,
        n: usize,
    },
}

/// A computation tape that records operations for backward pass.
#[derive(Default)]
pub struct Tape {
    entries: Vec<TapeEntry>,
    grads: Vec<Option<Vec<f64>>>,
    sizes: Vec<usize>,
}

impl Tape {
    pub fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, entry: TapeEntry, size: usize) -> usize {
        let id = self.entries.len();
        self.entries.push(entry);
        self.grads.push(None);
        self.sizes.push(size);
        id
    }

    /// Run backward pass from the given node, seeding with `grad_output`.
    pub fn backward(&mut self, from: usize, grad_output: Vec<f64>) {
        self.grads[from] = Some(grad_output);

        for i in (0..=from).rev() {
            let grad = match self.grads[i].take() {
                Some(g) => g,
                None => continue,
            };

            match &self.entries[i] {
                TapeEntry::Leaf => {
                    // Accumulate gradient on leaf
                    self.grads[i] = Some(grad);
                }
                TapeEntry::Add { a, b } => {
                    accumulate(&mut self.grads, *a, &grad);
                    accumulate(&mut self.grads, *b, &grad);
                }
                TapeEntry::Sub { a, b } => {
                    accumulate(&mut self.grads, *a, &grad);
                    let neg: Vec<f64> = grad.iter().map(|g| -g).collect();
                    accumulate(&mut self.grads, *b, &neg);
                }
                TapeEntry::Mul {
                    a,
                    b,
                    a_data,
                    b_data,
                } => {
                    let ga: Vec<f64> = grad
                        .iter()
                        .zip(b_data.iter())
                        .map(|(g, bv)| g * bv)
                        .collect();
                    let gb: Vec<f64> = grad
                        .iter()
                        .zip(a_data.iter())
                        .map(|(g, av)| g * av)
                        .collect();
                    accumulate(&mut self.grads, *a, &ga);
                    accumulate(&mut self.grads, *b, &gb);
                }
                TapeEntry::Scale { a, s } => {
                    let ga: Vec<f64> = grad.iter().map(|g| g * s).collect();
                    accumulate(&mut self.grads, *a, &ga);
                }
                TapeEntry::MatMul {
                    a,
                    b,
                    a_shape,
                    b_shape,
                    a_data,
                    b_data,
                } => {
                    // grad_a = grad @ b^T
                    let m = a_shape[0];
                    let k = a_shape[1];
                    let n = b_shape[1];
                    let mut ga = vec![0.0; m * k];
                    for i in 0..m {
                        for j in 0..k {
                            let mut sum = 0.0;
                            for p in 0..n {
                                sum += grad[i * n + p] * b_data[j * n + p];
                            }
                            ga[i * k + j] = sum;
                        }
                    }
                    // grad_b = a^T @ grad
                    let mut gb = vec![0.0; k * n];
                    for i in 0..k {
                        for j in 0..n {
                            let mut sum = 0.0;
                            for p in 0..m {
                                sum += a_data[p * k + i] * grad[p * n + j];
                            }
                            gb[i * n + j] = sum;
                        }
                    }
                    accumulate(&mut self.grads, *a, &ga);
                    accumulate(&mut self.grads, *b, &gb);
                }
                TapeEntry::Relu { a, mask } => {
                    let ga: Vec<f64> = grad
                        .iter()
                        .zip(mask.iter())
                        .map(|(g, &m)| if m { *g } else { 0.0 })
                        .collect();
                    accumulate(&mut self.grads, *a, &ga);
                }
                TapeEntry::Sigmoid { a, out } => {
                    // sigmoid'(x) = sigmoid(x) * (1 - sigmoid(x))
                    let ga: Vec<f64> = grad
                        .iter()
                        .zip(out.iter())
                        .map(|(g, s)| g * s * (1.0 - s))
                        .collect();
                    accumulate(&mut self.grads, *a, &ga);
                }
                TapeEntry::MseLoss { pred, diff, n } => {
                    let scale = 2.0 / *n as f64;
                    let gp: Vec<f64> = diff.iter().map(|d| grad[0] * scale * d).collect();
                    accumulate(&mut self.grads, *pred, &gp);
                }
            }
        }
    }

    /// Get gradient for a node.
    pub fn grad(&self, id: usize) -> Option<&Vec<f64>> {
        self.grads[id].as_ref()
    }
}

fn accumulate(grads: &mut [Option<Vec<f64>>], id: usize, new_grad: &[f64]) {
    match &mut grads[id] {
        Some(existing) => {
            for (e, n) in existing.iter_mut().zip(new_grad.iter()) {
                *e += n;
            }
        }
        None => {
            grads[id] = Some(new_grad.to_vec());
        }
    }
}

/// A multi-dimensional tensor stored as flat f64 data + shape.
///
/// When created on a tape, operations are recorded for backward pass.
#[derive(Clone)]
pub struct Tensor {
    pub data: Vec<f64>,
    pub shape: Vec<usize>,
    pub grad: Option<Vec<f64>>,
    /// Index into the tape (None if not tracked).
    tape_id: Option<usize>,
    tape: Option<Rc<RefCell<Tape>>>,
}

impl Tensor {
    /// Create a tensor from data and shape.
    pub fn new(data: Vec<f64>, shape: Vec<usize>) -> Self {
        let expected: usize = shape.iter().product();
        assert_eq!(
            data.len(),
            expected,
            "data len {} != shape product {}",
            data.len(),
            expected
        );
        Self {
            data,
            shape,
            grad: None,
            tape_id: None,
            tape: None,
        }
    }

    /// Create a leaf tensor on a tape (for trainable parameters).
    pub fn tracked(data: Vec<f64>, shape: Vec<usize>, tape: &Rc<RefCell<Tape>>) -> Self {
        let expected: usize = shape.iter().product();
        assert_eq!(data.len(), expected);
        let id = tape.borrow_mut().push(TapeEntry::Leaf, expected);
        Self {
            data,
            shape,
            grad: None,
            tape_id: Some(id),
            tape: Some(Rc::clone(tape)),
        }
    }

    /// Create a tensor of zeros.
    pub fn zeros(shape: &[usize]) -> Self {
        let n: usize = shape.iter().product();
        Self {
            data: vec![0.0; n],
            shape: shape.to_vec(),
            grad: None,
            tape_id: None,
            tape: None,
        }
    }

    /// Create a tensor of ones.
    pub fn ones(shape: &[usize]) -> Self {
        let n: usize = shape.iter().product();
        Self {
            data: vec![1.0; n],
            shape: shape.to_vec(),
            grad: None,
            tape_id: None,
            tape: None,
        }
    }

    /// Create a tensor with random values from a simple LCG.
    pub fn randn(shape: &[usize], seed: u64) -> Self {
        let n: usize = shape.iter().product();
        let mut rng = seed;
        let mut data = Vec::with_capacity(n);
        for _ in 0..n {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u1 = (rng as f64) / (u64::MAX as f64);
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u2 = (rng as f64) / (u64::MAX as f64);
            let z = (-2.0 * u1.max(1e-15).ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            data.push(z * 0.01);
        }
        Self {
            data,
            shape: shape.to_vec(),
            grad: None,
            tape_id: None,
            tape: None,
        }
    }

    /// Get tape node ID (for backward pass).
    pub fn id(&self) -> Option<usize> {
        self.tape_id
    }

    /// Total number of elements.
    pub fn numel(&self) -> usize {
        self.data.len()
    }

    /// Number of dimensions.
    pub fn ndim(&self) -> usize {
        self.shape.len()
    }

    fn with_tape_op(&self, data: Vec<f64>, shape: Vec<usize>, entry: TapeEntry) -> Tensor {
        let tape = self.tape.as_ref().map(Rc::clone);
        let tape_id = if let Some(ref t) = tape {
            Some(t.borrow_mut().push(entry, data.len()))
        } else {
            None
        };
        Tensor {
            data,
            shape,
            grad: None,
            tape_id,
            tape,
        }
    }

    /// Element-wise addition.
    pub fn add(&self, other: &Tensor) -> Tensor {
        assert_eq!(self.data.len(), other.data.len(), "shape mismatch for add");
        let data: Vec<f64> = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a + b)
            .collect();
        let entry = TapeEntry::Add {
            a: self.tape_id.unwrap_or(0),
            b: other.tape_id.unwrap_or(0),
        };
        if self.tape.is_some() {
            self.with_tape_op(data, self.shape.clone(), entry)
        } else {
            Tensor::new(data, self.shape.clone())
        }
    }

    /// Element-wise subtraction.
    pub fn sub(&self, other: &Tensor) -> Tensor {
        assert_eq!(self.data.len(), other.data.len(), "shape mismatch for sub");
        let data: Vec<f64> = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a - b)
            .collect();
        let entry = TapeEntry::Sub {
            a: self.tape_id.unwrap_or(0),
            b: other.tape_id.unwrap_or(0),
        };
        if self.tape.is_some() {
            self.with_tape_op(data, self.shape.clone(), entry)
        } else {
            Tensor::new(data, self.shape.clone())
        }
    }

    /// Element-wise multiply.
    pub fn mul(&self, other: &Tensor) -> Tensor {
        assert_eq!(self.data.len(), other.data.len(), "shape mismatch for mul");
        let data: Vec<f64> = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a * b)
            .collect();
        let entry = TapeEntry::Mul {
            a: self.tape_id.unwrap_or(0),
            b: other.tape_id.unwrap_or(0),
            a_data: self.data.clone(),
            b_data: other.data.clone(),
        };
        if self.tape.is_some() {
            self.with_tape_op(data, self.shape.clone(), entry)
        } else {
            Tensor::new(data, self.shape.clone())
        }
    }

    /// Scalar multiply.
    pub fn scale(&self, s: f64) -> Tensor {
        let data: Vec<f64> = self.data.iter().map(|v| v * s).collect();
        let entry = TapeEntry::Scale {
            a: self.tape_id.unwrap_or(0),
            s,
        };
        if self.tape.is_some() {
            self.with_tape_op(data, self.shape.clone(), entry)
        } else {
            Tensor::new(data, self.shape.clone())
        }
    }

    /// Matrix multiply (2D only): (M, K) @ (K, N) -> (M, N).
    pub fn matmul(&self, other: &Tensor) -> Tensor {
        assert_eq!(self.ndim(), 2);
        assert_eq!(other.ndim(), 2);
        let m = self.shape[0];
        let k = self.shape[1];
        assert_eq!(k, other.shape[0]);
        let n = other.shape[1];

        let mut data = vec![0.0; m * n];
        for i in 0..m {
            for j in 0..n {
                let mut sum = 0.0;
                for p in 0..k {
                    sum += self.data[i * k + p] * other.data[p * n + j];
                }
                data[i * n + j] = sum;
            }
        }
        let entry = TapeEntry::MatMul {
            a: self.tape_id.unwrap_or(0),
            b: other.tape_id.unwrap_or(0),
            a_shape: self.shape.clone(),
            b_shape: other.shape.clone(),
            a_data: self.data.clone(),
            b_data: other.data.clone(),
        };
        if self.tape.is_some() {
            self.with_tape_op(data, vec![m, n], entry)
        } else {
            Tensor::new(data, vec![m, n])
        }
    }

    /// Transpose a 2D tensor.
    pub fn t(&self) -> Tensor {
        assert_eq!(self.ndim(), 2);
        let m = self.shape[0];
        let n = self.shape[1];
        let mut data = vec![0.0; m * n];
        for i in 0..m {
            for j in 0..n {
                data[j * m + i] = self.data[i * n + j];
            }
        }
        Tensor::new(data, vec![n, m])
    }

    /// Sum all elements.
    pub fn sum(&self) -> f64 {
        self.data.iter().sum()
    }

    /// Mean of all elements.
    pub fn mean(&self) -> f64 {
        self.sum() / self.numel() as f64
    }

    /// Element-wise ReLU.
    pub fn relu(&self) -> Tensor {
        let mask: Vec<bool> = self.data.iter().map(|&v| v > 0.0).collect();
        let data: Vec<f64> = self.data.iter().map(|&v| v.max(0.0)).collect();
        let entry = TapeEntry::Relu {
            a: self.tape_id.unwrap_or(0),
            mask,
        };
        if self.tape.is_some() {
            self.with_tape_op(data, self.shape.clone(), entry)
        } else {
            Tensor::new(data, self.shape.clone())
        }
    }

    /// Element-wise sigmoid.
    pub fn sigmoid(&self) -> Tensor {
        let data: Vec<f64> = self
            .data
            .iter()
            .map(|&v| 1.0 / (1.0 + (-v).exp()))
            .collect();
        let entry = TapeEntry::Sigmoid {
            a: self.tape_id.unwrap_or(0),
            out: data.clone(),
        };
        if self.tape.is_some() {
            self.with_tape_op(data, self.shape.clone(), entry)
        } else {
            Tensor::new(data, self.shape.clone())
        }
    }

    /// Compute MSE loss against target. Returns (loss_scalar, loss_tensor_node).
    ///
    /// The loss_tensor has a single element and is tracked on the tape.
    pub fn mse_loss(&self, target: &Tensor) -> (f64, Tensor) {
        let diff: Vec<f64> = self
            .data
            .iter()
            .zip(target.data.iter())
            .map(|(p, t)| p - t)
            .collect();
        let n = diff.len();
        let loss = diff.iter().map(|d| d * d).sum::<f64>() / n as f64;
        let entry = TapeEntry::MseLoss {
            pred: self.tape_id.unwrap_or(0),
            diff,
            n,
        };
        let loss_tensor = if self.tape.is_some() {
            self.with_tape_op(vec![loss], vec![1], entry)
        } else {
            Tensor::new(vec![loss], vec![1])
        };
        (loss, loss_tensor)
    }

    /// Initialize gradient to zeros.
    pub fn zero_grad(&mut self) {
        self.grad = Some(vec![0.0; self.numel()]);
    }

    /// Run backward pass from this tensor (must be scalar, i.e. numel() == 1).
    pub fn backward(&self) {
        assert_eq!(self.numel(), 1, "backward() requires a scalar tensor");
        if let (Some(id), Some(ref tape)) = (self.tape_id, &self.tape) {
            tape.borrow_mut().backward(id, vec![1.0]);
        }
    }

    /// Retrieve gradient from the tape after backward().
    pub fn get_grad(&self) -> Option<Vec<f64>> {
        if let (Some(id), Some(ref tape)) = (self.tape_id, &self.tape) {
            tape.borrow().grad(id).cloned()
        } else {
            self.grad.clone()
        }
    }
}

impl fmt::Debug for Tensor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tensor(shape={:?}, data=[", self.shape)?;
        let show = self.data.len().min(6);
        for (i, v) in self.data[..show].iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:.4}", v)?;
        }
        if self.data.len() > show {
            write!(f, ", ...")?;
        }
        write!(f, "])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tensor_new() {
        let t = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]);
        assert_eq!(t.numel(), 4);
        assert_eq!(t.ndim(), 2);
    }

    #[test]
    fn test_matmul() {
        let a = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]);
        let b = Tensor::new(vec![5.0, 6.0, 7.0, 8.0], vec![2, 2]);
        let c = a.matmul(&b);
        assert_eq!(c.shape, vec![2, 2]);
        assert!((c.data[0] - 19.0).abs() < 1e-10);
        assert!((c.data[1] - 22.0).abs() < 1e-10);
    }

    #[test]
    fn test_transpose() {
        let a = Tensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]);
        let b = a.t();
        assert_eq!(b.shape, vec![3, 2]);
        assert!((b.data[0] - 1.0).abs() < 1e-10);
        assert!((b.data[1] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_relu() {
        let t = Tensor::new(vec![-1.0, 0.0, 1.0, 2.0], vec![4]);
        let r = t.relu();
        assert_eq!(r.data, vec![0.0, 0.0, 1.0, 2.0]);
    }

    #[test]
    fn test_sigmoid() {
        let t = Tensor::new(vec![0.0], vec![1]);
        let s = t.sigmoid();
        assert!((s.data[0] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_randn() {
        let t = Tensor::randn(&[3, 4], 42);
        assert_eq!(t.shape, vec![3, 4]);
        assert_eq!(t.numel(), 12);
    }

    #[test]
    fn test_autograd_add() {
        let tape = Rc::new(RefCell::new(Tape::new()));
        let a = Tensor::tracked(vec![2.0, 3.0], vec![2], &tape);
        let b = Tensor::tracked(vec![4.0, 5.0], vec![2], &tape);
        let c = a.add(&b); // c = a + b
                           // sum c to scalar: use manual approach
        let loss = c.data.iter().sum::<f64>(); // 6 + 8 = 14
        assert!((loss - 14.0).abs() < 1e-10);
    }

    #[test]
    fn test_autograd_mse_backward() {
        let tape = Rc::new(RefCell::new(Tape::new()));
        let pred = Tensor::tracked(vec![1.0, 2.0, 3.0], vec![3], &tape);
        let target = Tensor::new(vec![1.5, 2.5, 3.5], vec![3]);
        let (loss, loss_t) = pred.mse_loss(&target);
        // MSE = ((-.5)^2 + (-.5)^2 + (-.5)^2) / 3 = 0.75 / 3 = 0.25
        assert!((loss - 0.25).abs() < 1e-10);
        // Backward
        loss_t.backward();
        let grad = tape.borrow().grad(pred.id().unwrap()).unwrap().clone();
        // d(MSE)/d(pred_i) = 2*(pred_i - target_i)/n = 2*(-0.5)/3
        let expected = 2.0 * (-0.5) / 3.0;
        for g in &grad {
            assert!(
                (g - expected).abs() < 1e-10,
                "expected {}, got {}",
                expected,
                g
            );
        }
    }

    #[test]
    fn test_autograd_matmul_backward() {
        let tape = Rc::new(RefCell::new(Tape::new()));
        // W: 2x1 parameter
        let w = Tensor::tracked(vec![1.0, 1.0], vec![2, 1], &tape);
        // x: 1x2 input
        let x = Tensor::tracked(vec![2.0, 3.0], vec![1, 2], &tape);
        // y = x @ w -> 1x1
        let y = x.matmul(&w); // [2*1 + 3*1] = [5]
        assert!((y.data[0] - 5.0).abs() < 1e-10);

        let target = Tensor::new(vec![7.0], vec![1, 1]);
        let (loss, loss_t) = y.mse_loss(&target);
        // MSE = (5-7)^2 / 1 = 4.0
        assert!((loss - 4.0).abs() < 1e-10);
        loss_t.backward();

        let w_grad = tape.borrow().grad(w.id().unwrap()).unwrap().clone();
        // d(loss)/dw = d(loss)/dy * d(y)/dw
        // d(loss)/dy = 2*(5-7)/1 = -4
        // d(y)/dw = x^T = [2, 3]^T
        // so grad_w = -4 * [2, 3]^T = [-8, -12]
        assert!(
            (w_grad[0] - (-8.0)).abs() < 1e-10,
            "w_grad[0]: {}",
            w_grad[0]
        );
        assert!(
            (w_grad[1] - (-12.0)).abs() < 1e-10,
            "w_grad[1]: {}",
            w_grad[1]
        );
    }
}

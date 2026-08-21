//! N-dimensional array module (NumPy-like).

mod dtype;
mod error;
pub mod indexing;
mod linalg;
mod linalg_decomp;
mod manip;
mod math;
mod ndarray;
mod ops;
pub mod random;
pub mod search;
mod shape;
mod slice;
mod stats;
mod window;

pub use dtype::{ArrayElement, DType};
pub use error::{ArrayError, Result};
pub use indexing::IndexMode;
pub use linalg::LinalgOps;
pub use linalg_decomp::{EigResult, LstsqResult, QrResult, SvdResult};
pub use math::FloatOps;
pub use ndarray::NdArray;
pub use ops::NumericOps;
pub use random::{Pcg64, RandomExt};
pub use shape::Shape;
pub use slice::{AxisSlice, NormalizedSlice, SliceInfo};
pub use stats::{StatOps, VarOps};
pub use window::Rolling;

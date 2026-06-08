//! Numerical optimization module (scipy.optimize equivalent).
//!
//! - **minimize**: Nelder-Mead and BFGS minimization
//! - **root**: Brent's method and Newton-Raphson root finding
//! - **curve_fit**: Non-linear curve fitting
//! - **linprog**: Linear programming (simplex method)

mod curve_fit;
mod error;
mod linprog;
mod minimize;
mod root;

pub use curve_fit::{curve_fit, CurveFitResult};
pub use error::{OptimizeError, Result};
pub use linprog::{linprog, LinprogResult};
pub use minimize::{bfgs, nelder_mead, MinimizeResult};
pub use root::{bisect, brentq, newton};

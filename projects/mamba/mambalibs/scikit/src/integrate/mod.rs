//! Numerical integration module — scipy.integrate equivalent.
//!
//! - **quad**: Adaptive quadrature, trapezoid rule, Simpson's rule, Gaussian quadrature
//! - **ode**: ODE solvers (Euler, RK4, RK45 adaptive)

mod ode;
mod quad;

pub use ode::{euler, rk4, solve_ivp, OdeResult, OdeSolver};
pub use quad::{
    cumulative_trapezoid, fixed_quad, quad, simps, trapezoid, QuadResult,
};

//! Interpolation module — scipy.interpolate equivalent.
//!
//! - **interp1d**: Linear and cubic spline 1D interpolation
//! - **CubicSpline**: Natural cubic spline
//! - **interp2d**: 2D bilinear interpolation

mod interp;
mod interp2d;

pub use interp::{interp1d, CubicSpline, InterpKind};
pub use interp2d::{interp2d, Interp2dError};

//! # cclab-tqdm
//!
//! Rust-powered progress tracking for Python, replacing tqdm.
//! Uses indicatif for rendering with GIL release during updates.

pub mod bar;
pub mod error;

pub use bar::{MultiProgress, ProgressBar, ProgressStyle};
pub use error::TqdmError;

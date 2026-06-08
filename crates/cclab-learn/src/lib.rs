//! # cclab-learn
//!
//! Machine learning and deep learning (scikit-learn-like).

#[cfg(feature = "ml")]
pub mod ml;

#[cfg(feature = "dl")]
pub mod dl;

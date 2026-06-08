//! # cclab-media
//!
//! Image and video processing.

#[cfg(feature = "image")]
pub mod image;

#[cfg(feature = "video")]
pub mod video;

#[cfg(feature = "pdf")]
pub mod pdf;

//! # cclab-text
//!
//! Text processing: segmentation, search ranking, HTML/XML parsing,
//! fuzzy matching, diff/patch, and templates.

#[cfg(feature = "segment")]
pub mod segment;

#[cfg(feature = "rank")]
pub mod rank;

#[cfg(feature = "markup")]
pub mod markup;

pub mod diff;
pub mod fuzzy;

#[cfg(feature = "template")]
pub mod template;

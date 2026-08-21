//! PDF module — generation and extraction in pure Rust.
//!
//! - **types**: Document, Page, Content elements, styling
//! - **writer**: Generate PDF from structured content
//! - **reader**: Extract text/images from PDF, split/merge pages

pub mod reader;
pub mod types;
pub mod writer;

pub use reader::{
    extract_text, info, merge, page_sizes, split_pages, ExtractedImage, PageText, PdfInfo,
    PdfReadError,
};
pub use types::{
    Color, ContentElement, Document, Font, Margins, Page, PageSize, TextAlign, TextStyle,
};
pub use writer::{generate, write_to_file};

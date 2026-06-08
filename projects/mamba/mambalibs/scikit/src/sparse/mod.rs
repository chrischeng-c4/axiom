//! Sparse matrix module — scipy.sparse equivalent.
//!
//! - **csr**: Compressed Sparse Row format
//! - **csc**: Compressed Sparse Column format
//! - Conversion between formats and dense representation

mod csc;
mod csr;

pub use csc::CscMatrix;
pub use csr::CsrMatrix;

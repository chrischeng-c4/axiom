pub mod error;
pub mod pkgmanage;
pub mod source;
pub mod lexer;
pub mod parser;
pub mod resolve;
pub mod types;
pub mod hir;
pub mod mir;
pub mod lower;
pub mod runtime;
pub mod codegen;
pub mod diagnostic;
pub mod driver;
pub mod ffi;
pub mod bench;
pub mod conformance;
pub mod surface;

#[cfg(test)]
pub(crate) mod testing;

#[cfg(test)]
mod meta_gates;

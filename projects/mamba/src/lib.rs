pub mod bench;
pub mod codegen;
pub mod conformance;
pub mod diagnostic;
pub mod driver;
pub mod error;
pub(crate) mod exec_literal;
pub mod ffi;
pub mod hir;
pub mod lexer;
pub mod lower;
pub mod mir;
pub mod parser;
pub mod pkgmanage;
pub mod resolve;
pub mod runtime;
pub mod source;
pub mod surface;
pub mod types;

#[cfg(test)]
pub(crate) mod testing;

#[cfg(test)]
mod meta_gates;

//! Language-specific type inference for semantic analysis
//!
//! Provides deeper type analysis beyond what the symbol table captures.

pub mod go;
pub mod go_advanced;

#[cfg(test)]
mod go_tests;

pub use go::{ChannelDirection, GenericParam, GoType, GoTypeInference, MethodInfo, TypeAssertion};
pub use go_advanced::check_interface_satisfaction;

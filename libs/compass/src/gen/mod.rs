//! Code generation module for spec-to-code
//!
//! Contains generators for various technology stacks:
//! - Python: cclab.shield, cclab.titan, cclab.nebula, cclab.photon, cclab.quasar
//! - Rust: serde, axum, sqlx, reqwest

pub mod python;
pub mod registry;
pub mod rust;
pub mod traits;
// framework/ generators (fastapi, express, axum) were in sdd::gen_framework;
// now deleted — use sdd::generate::generators for SpecIR-based codegen.

pub use registry::GeneratorRegistry;
pub use traits::*;

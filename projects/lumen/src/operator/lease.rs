// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-lease-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! lumen's leader-election lease — now the shared `operator::lease`.
//!
//! The implementation moved to `libs/operator` (the Lease name is parameterized
//! by the operator's field manager). lumen keeps this module as a thin re-export
//! so existing `crate::operator::lease::*` paths still resolve.

pub use operator::lease::*;
// CODEGEN-END

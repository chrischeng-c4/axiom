// CODEGEN-BEGIN
//! lumen's leader-election lease — now the shared `service_k8s::lease`.
//!
//! The implementation moved to `libs/service-k8s` (the Lease name is parameterized
//! by the operator's field manager). lumen keeps this module as a thin re-export
//! so existing `crate::operator::lease::*` paths still resolve.

pub use service_k8s::lease::*;
// CODEGEN-END

// SPEC-MANAGED: libs/service-http/tech-design/semantic/source/libs-service-http-src-readiness-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Probe-facing name for the protocol-neutral lifecycle readiness contract.
//!
//! A service supplies a type that reports whether it is currently draining
//! (post-SIGTERM grace window). The shared probe router calls
//! [`server_lifecycle::Readiness::is_draining`] on every `/readyz` hit so k8s sees 503 the
//! moment a graceful shutdown begins, and stops routing before the listener
//! closes. In lumen/keep this is the engine's drain flag; any
//! `Arc`-shareable, `is_draining()`-reporting type works.

pub use server_lifecycle::Readiness as ReadinessHook;
// CODEGEN-END

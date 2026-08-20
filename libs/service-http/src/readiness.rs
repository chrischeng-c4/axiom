// CODEGEN-BEGIN
//! Probe-facing name for the protocol-neutral lifecycle readiness contract.
//!
//! New production code should pass a [`server_lifecycle::LifecycleController`]
//! to `lifecycle_probe_routes`; this re-export remains for source-compatible
//! migration of services that still own a separate readiness hook.
//!
//! A service supplies a type that reports whether it is currently draining
//! (post-SIGTERM grace window). The shared probe router calls
//! [`server_lifecycle::Readiness::is_draining`] on every `/readyz` hit so k8s sees 503 the
//! moment a graceful shutdown begins, and stops routing before the listener
//! closes. In lumen/keep this is the engine's drain flag; any
//! `Arc`-shareable, `is_draining()`-reporting type works.

pub use server_lifecycle::Readiness as ReadinessHook;
// CODEGEN-END

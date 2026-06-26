//! Readiness seam for `/readyz`.
//!
//! A service supplies a type that reports whether it is currently draining
//! (post-SIGTERM grace window). The shared probe router calls
//! [`ReadinessHook::is_draining`] on every `/readyz` hit so k8s sees 503 the
//! moment a graceful shutdown begins, and stops routing before the listener
//! closes. In lumen/keep this is the engine's drain flag; any
//! `Arc`-shareable, `is_draining()`-reporting type works.

/// Reports whether the service is draining (shutting down). `/readyz` returns
/// 503 when this is `true`, 200 otherwise.
pub trait ReadinessHook: Send + Sync {
    /// `true` once graceful shutdown has begun, so `/readyz` should report 503.
    fn is_draining(&self) -> bool;
}

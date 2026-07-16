// SPEC-MANAGED: libs/service-http/tech-design/semantic/source/libs-service-http-src-signal-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Service-shell re-exports for protocol-neutral graceful shutdown.
//!
//! The drain dance every k8s-native service in the ecosystem repeats: on
//! SIGINT/SIGTERM, flip readiness to draining (so `/readyz` → 503 and k8s stops
//! routing), hold a grace window, then let the listener close. Factored out of
//! lumen's / keep's `shutdown_signal`. Ownership lives in `server-lifecycle`.

pub use server_lifecycle::{shutdown_with_drain, wait_shutdown_signal};
// CODEGEN-END

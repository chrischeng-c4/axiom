---
id: libs-service-http-src-signal-rs
summary: Lossless rust-source-unit coverage for `libs/service-http/src/signal.rs`.
capability_refs:
  - id: shared-http-service-scaffold
    role: primary
    claim: shared-http-service-scaffold-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Http library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-http/src/signal.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/service-http/src/signal.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `wait_shutdown_signal` | libs/service-http/src/signal.rs | function | pub | 14 | pub async fn wait_shutdown_signal() { |
| `shutdown_with_drain` | libs/service-http/src/signal.rs | function | pub | 46 | pub async fn shutdown_with_drain(start_drain: impl FnOnce() + Send, grace: Duration) { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! SIGTERM-aware graceful-drain shutdown.
//!
//! The drain dance every k8s-native service in the ecosystem repeats: on
//! SIGINT/SIGTERM, flip readiness to draining (so `/readyz` → 503 and k8s stops
//! routing), hold a grace window, then let the listener close. Factored out of
//! lumen's / keep's `shutdown_signal`.

use std::time::Duration;

/// Resolve when SIGINT (Ctrl-C) or — on unix — SIGTERM arrives.
///
/// On non-unix targets only SIGINT is wired; the SIGTERM arm is a pending
/// future, matching the service binaries.
pub async fn wait_shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        if let Ok(mut s) = signal(SignalKind::terminate()) {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("received SIGINT"),
        _ = terminate => tracing::info!("received SIGTERM"),
    }
}

/// Await a shutdown signal, then begin draining and hold the grace window.
///
/// Sequence: [`wait_shutdown_signal`] → call `start_drain` (which should flip
/// the service's [`crate::readiness::ReadinessHook`] source so `/readyz` reports
/// 503) → sleep `grace` so k8s observes the not-ready state and stops routing →
/// return. The caller passes the returned future to [`crate::transport::serve`]
/// as its `shutdown`, so the listener closes only after the grace window.
///
/// A closure (not a `ReadinessHook`) is used for `start_drain` because the
/// readiness hook is read-only; the drain trigger is a separate, write-side
/// concern the binary owns (e.g. `engine.start_drain()`).
pub async fn shutdown_with_drain(start_drain: impl FnOnce() + Send, grace: Duration) {
    wait_shutdown_signal().await;
    start_drain();
    tracing::info!(grace_secs = grace.as_secs(), "draining — readyz=503");
    tokio::time::sleep(grace).await;
    tracing::info!("grace expired — shutting down");
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-http/src/signal.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-http/src/signal.rs` captured during libs codegen standardization.
```

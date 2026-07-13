// SPEC-MANAGED: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-admin-plane" tracker="#1290" reason="Admin plane needs generator primitives that do not exist yet.">
//! The two `serve_entry`/`share_drain`/`spawn_signal_task` wiring steps
//! (TD Logic section) extracted as small, independently testable functions
//! rather than left inline in `src/bin/pgpool.rs` (a binary target has no
//! seam `apps/pgpool/tests/*.rs` integration tests can reach into, and R7's
//! two verify ids are unit tests): `serve()` calls both of these directly,
//! so the tests below exercise the exact same code path production uses.

use std::future::Future;

use server_core::DrainController;
use tcp_server::TcpServerConfig;

/// `share_drain` node: overrides `config.drain` with the SAME shared
/// `DrainController` clone `AdminState` holds, replacing the fresh
/// `DrainController::new()` that `TcpServerConfig::new()` builds by default
/// (R7). `TcpServerConfig` has no `with_drain()` builder — all its fields
/// are `pub`, so struct-update syntax is the intended seam.
pub fn wire_tcp_server_drain(config: TcpServerConfig, drain: &DrainController) -> TcpServerConfig {
    TcpServerConfig {
        drain: drain.clone(),
        ..config
    }
}

/// `spawn_signal_task` node: awaits `shutdown` (production passes
/// `server_core::signal::wait_shutdown_signal()`) and then calls
/// `start_drain()` on the shared controller (R2, R7). `serve()` spawns this
/// as a background task; tests await it directly with a manually-resolved
/// `shutdown` future.
pub async fn drain_on_shutdown_signal(drain: DrainController, shutdown: impl Future<Output = ()>) {
    shutdown.await;
    drain.start_drain();
}

#[cfg(test)]
mod tests {
    use super::*;
    use server_core::BindConfig;

    /// verify: admin::serve_wires_shared_drain_controller_into_tcp_server_config (R7)
    #[test]
    fn tcp_server_config_carries_the_shared_drain_controller_not_a_fresh_one() {
        let drain = DrainController::new();
        // Keep a receiver alive: `tokio::sync::watch::Sender::send` is a
        // silent no-op with zero live receivers (mirrors production, where
        // `TcpServerConfig`'s own accept loop always holds one via
        // `config.drain.signal()`).
        let _signal = drain.signal();
        let config = TcpServerConfig::new(BindConfig::localhost(0));
        let config = wire_tcp_server_drain(config, &drain);

        // Identity (not equality): starting drain on the ORIGINAL handle
        // must be observed through `config.drain`, proving it is the same
        // shared watch channel rather than an independent
        // `DrainController::new()`.
        assert!(!config.drain.is_draining());
        drain.start_drain();
        assert!(config.drain.is_draining());
    }

    /// verify: admin::signal_task_calls_start_drain_on_the_shared_controller (R7)
    #[tokio::test]
    async fn shutdown_future_resolving_calls_start_drain_on_the_shared_controller() {
        let drain = DrainController::new();
        // See the sibling test's comment: hold a receiver alive so
        // `start_drain()` inside `drain_on_shutdown_signal` isn't a no-op.
        let _signal = drain.signal();
        assert!(!drain.is_draining());

        // Test seam substituting for `server_core::signal::wait_shutdown_signal()`:
        // an already-resolved future, standing in for "SIGTERM/SIGINT observed".
        drain_on_shutdown_signal(drain.clone(), async {}).await;

        assert!(drain.is_draining());
    }
}
// </HANDWRITE>

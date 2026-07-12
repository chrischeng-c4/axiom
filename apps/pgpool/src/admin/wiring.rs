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
// SPEC-MANAGED: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#logic
// CODEGEN-BEGIN
pub fn serve_entry() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // SPEC-REF: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#pgpool-admin-plane-logic-flow-share_drain
    // TODO: Implement process step: Clone the shared DrainController into TcpServerConfig.drain (replacing the fresh DrainController::new() TcpServerConfig::new() builds by default) and into AdminState, so the data plane, the admin plane, and readiness all read/write the same watch channel (R2, scope: drain coordination)
    todo!("process: Clone the shared DrainController into TcpServerConfig.drain (replacing the fresh DrainController::new() TcpServerConfig::new() builds by default) and into AdminState, so the data plane, the admin plane, and readiness all read/write the same watch channel (R2, scope: drain coordination)");
    // SPEC-REF: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#pgpool-admin-plane-logic-flow-spawn_signal_task
    // TODO: Implement process step: Spawn a background task awaiting server_core::signal::wait_shutdown_signal(); when SIGTERM/SIGINT resolves it, the task calls drain.start_drain() on the shared controller (R2)
    todo!("process: Spawn a background task awaiting server_core::signal::wait_shutdown_signal(); when SIGTERM/SIGINT resolves it, the task calls drain.start_drain() on the shared controller (R2)");
    // SPEC-REF: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#pgpool-admin-plane-logic-flow-build_admin_router
    // TODO: Implement process step: Build the admin axum Router directly against AdminState (shared DrainController clone + Vec<NamedPool>, each pairing a pool name/mode with its ConnectionBudget and BackendPool clone): /healthz, /readyz, /metrics, /openapi.json, /docs, GET /pools, GET /pools/{pool}/stats, POST /drain (R1, R3) - hand-rolled rather than libs/service-http's standard_probe_routes because that helper's openapi arg type is fn() -> utoipa::openapi::OpenApi, while apps/pgpool/src/spec.rs's single-source-of-truth OpenAPI document is a serde_json::Value the offline `pgpool spec --format openapi` CLI already serializes directly; routing /openapi.json through a typed utoipa round-trip would risk breaking the byte-for-byte parity R4/AC3 requires, so /openapi.json instead returns Json(pgpool::spec::openapi()) - the exact same Value
    todo!("process: Build the admin axum Router directly against AdminState (shared DrainController clone + Vec<NamedPool>, each pairing a pool name/mode with its ConnectionBudget and BackendPool clone): /healthz, /readyz, /metrics, /openapi.json, /docs, GET /pools, GET /pools/{pool}/stats, POST /drain (R1, R3) - hand-rolled rather than libs/service-http's standard_probe_routes because that helper's openapi arg type is fn() -> utoipa::openapi::OpenApi, while apps/pgpool/src/spec.rs's single-source-of-truth OpenAPI document is a serde_json::Value the offline `pgpool spec --format openapi` CLI already serializes directly; routing /openapi.json through a typed utoipa round-trip would risk breaking the byte-for-byte parity R4/AC3 requires, so /openapi.json instead returns Json(pgpool::spec::openapi()) - the exact same Value");
    // SPEC-REF: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#pgpool-admin-plane-logic-flow-run_both_planes
    // TODO: Implement process step: tokio::join! the TCP frontend (tcp_server::serve, existing PoolHandler dispatch, unchanged from WI #1289) and the admin plane (http_server::serve_h2c_with_options) concurrently; each is given its OWN one-shot shutdown future that awaits drain.signal().changed()
    todo!("process: tokio::join! the TCP frontend (tcp_server::serve, existing PoolHandler dispatch, unchanged from WI #1289) and the admin plane (http_server::serve_h2c_with_options) concurrently; each is given its OWN one-shot shutdown future that awaits drain.signal().changed()");
    // Decision: Which admin request arrives while both planes are running?
    if todo!("decision: Which admin request arrives while both planes are running?") /* GET /healthz */ {
        todo!("terminal: GET /healthz: always 200 'ok' - liveness only, never reflects drain state (R1)");
    } else if todo!("decision branch: {}", "GET /readyz") { /* GET /readyz */
        // SPEC-REF: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#pgpool-admin-plane-logic-flow-readyz_req
        // TODO: Implement process step: GET /readyz: reads drain.is_draining() off the shared controller
        todo!("process: GET /readyz: reads drain.is_draining() off the shared controller");
        // Decision: drain.is_draining()?
        if todo!("decision: drain.is_draining()?") /* not draining */ {
            todo!("terminal: false: 200 'ok'");
        } else { /* draining */
            todo!("terminal: true: 503 'draining' (R2)");
        }
    } else if todo!("decision branch: {}", "GET /metrics") { /* GET /metrics */
        todo!("terminal: GET /metrics: renders Prometheus text-format gauges (pgpool_frontend_active, pgpool_backend_active, pgpool_backend_idle, each labeled pool=<name>) from every AdminState.pools entry's ConnectionBudget::active() and BackendPool::stats() (AC4)");
    } else if todo!("decision branch: {}", "GET /openapi.json") { /* GET /openapi.json */
        todo!("terminal: GET /openapi.json: returns Json(pgpool::spec::openapi()) - the identical serde_json::Value apps/pgpool/src/spec.rs already builds for `pgpool spec --format openapi` (R4)");
    } else if todo!("decision branch: {}", "GET /docs") { /* GET /docs */
        todo!("terminal: GET /docs: static Swagger UI HTML page that loads /openapi.json, mirroring libs/service-http's docs_swagger convention");
    } else if todo!("decision branch: {}", "GET /pools") { /* GET /pools */
        todo!("terminal: GET /pools: 200 Json(PoolList{pools: [...]}) - one PoolStats entry per AdminState.pools member, matching spec.rs's PoolList/PoolStats schema (R3)");
    } else if todo!("decision branch: {}", "GET /pools/{pool}/stats") { /* GET /pools/{pool}/stats */
        // Decision: GET /pools/{pool}/stats: does {pool} match a name in AdminState.pools?
        if todo!("decision: GET /pools/{pool}/stats: does {pool} match a name in AdminState.pools?") /* name matches */ {
            todo!("terminal: found: 200 Json(PoolStats{name, mode, frontend_active, backend_active, backend_idle}) (R3, AC4)");
        } else { /* no match */
            todo!("terminal: not found: 404, body names the unknown pool");
        }
    } else { /* POST /drain */
        // SPEC-REF: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#pgpool-admin-plane-logic-flow-drain_post
        // TODO: Implement process step: POST /drain: calls the SAME shared DrainController::start_drain() the SIGTERM path uses - one drain trigger, two sources
        todo!("process: POST /drain: calls the SAME shared DrainController::start_drain() the SIGTERM path uses - one drain trigger, two sources");
        todo!("terminal: The shared watch channel flips to Draining: /readyz starts returning 503, and the TCP frontend's own shutdown future (also awaiting drain.signal().changed()) resolves so its accept loop stops admitting new frontend connections, while already-established sessions/transactions keep relaying until they end or TcpServerConfig.drain_timeout elapses (R2, AC2); the handler returns 200 Json(DrainState{draining:true})");
    }
    todo!("terminal: The shared watch channel flips to Draining: /readyz starts returning 503, and the TCP frontend's own shutdown future (also awaiting drain.signal().changed()) resolves so its accept loop stops admitting new frontend connections, while already-established sessions/transactions keep relaying until they end or TcpServerConfig.drain_timeout elapses (R2, AC2); the handler returns 200 Json(DrainState{draining:true})");
    // Terminal: docs_req -> GET /docs: static Swagger UI HTML page that loads /openapi.json, mirroring libs/service-http's docs_swagger convention
    // Terminal: drain_effect -> The shared watch channel flips to Draining: /readyz starts returning 503, and the TCP frontend's own shutdown future (also awaiting drain.signal().changed()) resolves so its accept loop stops admitting new frontend connections, while already-established sessions/transactions keep relaying until they end or TcpServerConfig.drain_timeout elapses (R2, AC2); the handler returns 200 Json(DrainState{draining:true})
    // Terminal: healthz_req -> GET /healthz: always 200 'ok' - liveness only, never reflects drain state (R1)
    // Terminal: metrics_req -> GET /metrics: renders Prometheus text-format gauges (pgpool_frontend_active, pgpool_backend_active, pgpool_backend_idle, each labeled pool=<name>) from every AdminState.pools entry's ConnectionBudget::active() and BackendPool::stats() (AC4)
    // Terminal: openapi_req -> GET /openapi.json: returns Json(pgpool::spec::openapi()) - the identical serde_json::Value apps/pgpool/src/spec.rs already builds for `pgpool spec --format openapi` (R4)
    // Terminal: pool_stats_found -> found: 200 Json(PoolStats{name, mode, frontend_active, backend_active, backend_idle}) (R3, AC4)
    // Terminal: pool_stats_missing -> not found: 404, body names the unknown pool
    // Terminal: pools_req -> GET /pools: 200 Json(PoolList{pools: [...]}) - one PoolStats entry per AdminState.pools member, matching spec.rs's PoolList/PoolStats schema (R3)
    // Terminal: process_exit -> serve() returns Ok(()); the process exits cleanly with no forcibly-dropped in-flight session or admin request (AC2)
    // Terminal: readyz_draining -> true: 503 'draining' (R2)
    // Terminal: readyz_ready -> false: 200 'ok'
}
// CODEGEN-END

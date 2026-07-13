// SPEC-MANAGED: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-admin-plane" tracker="#1290" reason="Admin plane needs generator primitives that do not exist yet.">
//! Served admin HTTP plane (TD Logic/Schema sections): a hand-rolled axum
//! router bound on `RuntimePlan.admin_bind` via
//! `http_server::serve_h2c_with_options`, sharing ONE
//! `server_core::DrainController` with the TCP frontend so `/readyz` and
//! `POST /drain` observe/drive the exact same drain state SIGTERM/SIGINT
//! does (R2). Hand-rolled rather than `service_http::standard_probe_routes`
//! because that helper's `openapi` argument is
//! `fn() -> utoipa::openapi::OpenApi`, while `crate::spec`'s single source
//! of truth is a `serde_json::Value` the offline `pgpool spec --format
//! openapi` CLI already serializes directly; routing `/openapi.json`
//! through a typed utoipa round-trip would risk breaking the byte-for-byte
//! parity R4/AC3 requires (see the TD Logic section `build_admin_router`
//! node for the full rationale).

mod handlers;
mod metrics;
mod router;
mod state;
mod types;
mod wiring;

pub use router::{build_router, ADMIN_ROUTES};
pub use state::{AdminState, NamedPool};
pub use types::{DrainResponse, PoolListResponse, PoolStatsResponse, ReadyzResponse};
pub use wiring::{drain_on_shutdown_signal, wire_tcp_server_drain};
// </HANDWRITE>

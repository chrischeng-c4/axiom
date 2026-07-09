// SPEC-MANAGED: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-session-proxy" tracker="#1288" reason="Session-mode proxy needs generator primitives that do not exist yet.">
//! Session-mode 1:1 PostgreSQL proxy: one backend connection per accepted
//! frontend client, admission-gated by its own `ConnectionBudget`, with
//! frame-aware auth passthrough (credential bytes relayed opaquely, never
//! persisted) and bidirectional relay until `Terminate`/EOF/`FrameError`.
//! See the TD at
//! `apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md`.

mod config;
mod error;
mod handler;
mod session;

pub use config::{BackendEndpointConfig, SessionProxyConfig};
pub use error::{ProxyError, RejectionReason, SessionOutcome};
pub use handler::SessionHandler;
pub use session::run_session;
// </HANDWRITE>

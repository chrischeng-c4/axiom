// SPEC-MANAGED: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-session-proxy" tracker="#1288" reason="Session-mode proxy needs generator primitives that do not exist yet.">
//! `SessionHandler`: the `tcp_server::TcpHandler` impl `pgpool serve` binds
//! to its listener. One backend connection per accepted client
//! (session-mode), admission-gated by its own `ConnectionBudget` — see the
//! TD Logic flowchart's `cli_serve_entry` node for why this is deliberately
//! not wired into `tcp_server::TcpServerConfig.connection_budget`.

use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use tcp_server::{ConnectionContext, TcpHandler};
use tokio::net::TcpStream;

use crate::proxy::config::SessionProxyConfig;
use crate::proxy::session::run_session;

/// Session-mode 1:1 PostgreSQL proxy handler: dials the configured backend
/// per accepted client and relays frames until the session ends.
#[derive(Debug, Clone)]
pub struct SessionHandler {
    config: SessionProxyConfig,
}

impl SessionHandler {
    pub fn new(config: SessionProxyConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &SessionProxyConfig {
        &self.config
    }
}

impl TcpHandler for SessionHandler {
    // A boxed future keeps `SessionHandler` a plain, nameable type (the TD's
    // `SessionHandler` per the Logic/Schema sections) instead of requiring
    // an unstable `impl Trait` associated type; one small per-connection
    // allocation is an acceptable trade against that ergonomics/stability
    // win for a per-connection (not per-message) handler.
    type Future = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

    fn handle(&self, stream: TcpStream, cx: ConnectionContext) -> Self::Future {
        let config = self.config.clone();
        Box::pin(async move {
            let outcome = run_session(stream, &config).await;
            tracing::info!(?outcome, peer = %cx.peer_addr, "pgpool session ended");
            Ok(())
        })
    }
}
// </HANDWRITE>

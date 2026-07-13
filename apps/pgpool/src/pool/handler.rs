// SPEC-MANAGED: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#schema
// <HANDWRITE gap="missing-generator:logic:pgpool-backend-pool" tracker="#1289" reason="Backend pool needs generator primitives that do not exist yet.">
//! `PoolHandler`: the single `tcp_server::TcpHandler` `pgpool serve` binds
//! to its listener, selected once at process start from
//! `RuntimePlan::pool_mode` (per the TD Schema section) and dispatching to
//! whichever mode-specific handler it wraps.

use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use tcp_server::{ConnectionContext, TcpHandler};
use tokio::net::TcpStream;

use crate::pool::transaction::TransactionHandler;
use crate::proxy::SessionHandler;

/// Process-start dispatch wrapper over the two pool-mode handlers; which
/// variant is constructed is decided once, from `RuntimePlan::pool_mode`,
/// not re-evaluated per connection.
#[derive(Debug, Clone)]
pub enum PoolHandler {
    Session(SessionHandler),
    Transaction(TransactionHandler),
}

impl TcpHandler for PoolHandler {
    type Future = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

    fn handle(&self, stream: TcpStream, cx: ConnectionContext) -> Self::Future {
        match self {
            PoolHandler::Session(handler) => handler.handle(stream, cx),
            PoolHandler::Transaction(handler) => handler.handle(stream, cx),
        }
    }
}
// </HANDWRITE>

// SPEC-MANAGED: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-backend-pool" tracker="#1289" reason="Backend pool needs generator primitives that do not exist yet.">
//! Shared backend-connection pool: reuse across session/transaction pool
//! modes, transaction-mode pooling itself, and the `PoolHandler` dispatch
//! wrapper `pgpool serve` binds to its listener. See the TD at
//! `apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md`.

mod backend_pool;
mod handler;
mod transaction;
mod types;

pub use backend_pool::{BackendLease, BackendPool};
pub use handler::PoolHandler;
pub use transaction::{TransactionHandler, TransactionProxyConfig};
pub use types::{
    BackendConnectionId, BackendPoolStats, LeaseDisposition, PoolConfig, PoolError,
    PoolRejectionReason, PoolStats,
};
// </HANDWRITE>

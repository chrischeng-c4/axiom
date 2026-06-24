//! Driver layer — raw async PostgreSQL I/O (psycopg / asyncpg equivalent).
//!
//! Contains connection pooling, transaction management, row marshalling,
//! type conversion, bulk operations, multi-dialect SQL emission, and the
//! blocking façade. This layer MUST NOT depend on `orm` or `migrate`.

pub mod blocking;
pub mod bulk;
pub mod connection;
pub mod dialect;
pub mod executor;
pub mod row;
pub mod transaction;
pub mod types;

pub use bulk::{BulkConfig, BulkExecutor, BulkResult};
pub use connection::{Connection, PoolConfig, RetryConfig};
pub use executor::{execute_with_retry, ExecutorConfig, QueryExecutor};
pub use row::{RelationConfig, Row};
pub use transaction::{AccessMode, IsolationLevel, Transaction, TransactionOptions};
pub use types::{row_to_extracted, ExtractedValue};

//! Blocking façade over [`crate::Connection`].

use std::sync::Arc;

use sqlx::postgres::PgPool;
use tokio::runtime::Runtime;

use crate::{Connection as AsyncConnection, ExtractedValue, PoolConfig, Result, Row};

/// Blocking PostgreSQL connection pool.
///
/// Holds a [`tokio::runtime::Runtime`] used to drive the underlying
/// async [`AsyncConnection`]. Cloning is cheap — the inner connection
/// and runtime are both reference-counted.
#[derive(Clone)]
pub struct Connection {
    inner: AsyncConnection,
    rt: Arc<Runtime>,
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("blocking::Connection")
            .field("inner", &self.inner)
            .finish()
    }
}

impl Connection {
    /// Creates a new connection pool, blocking until the pool is ready.
    pub fn new(uri: &str, config: PoolConfig) -> Result<Self> {
        let rt = Arc::new(build_runtime()?);
        let inner = rt.block_on(AsyncConnection::new(uri, config))?;
        Ok(Self { inner, rt })
    }

    /// Wraps an already-constructed async [`AsyncConnection`] with the
    /// given runtime. Use this when you want to share one runtime
    /// across multiple `Blocking*` handles.
    pub fn from_parts(inner: AsyncConnection, rt: Arc<Runtime>) -> Self {
        Self { inner, rt }
    }

    /// Returns a reference to the underlying async connection.
    pub fn as_async(&self) -> &AsyncConnection {
        &self.inner
    }

    /// Returns a clone of the runtime handle so derived types
    /// (transactions, migration runners) can share the same executor.
    pub fn runtime(&self) -> Arc<Runtime> {
        self.rt.clone()
    }

    /// Returns the sqlx connection pool.
    pub fn pool(&self) -> &PgPool {
        self.inner.pool()
    }

    /// Closes the connection pool.
    pub fn close(&self) -> Result<()> {
        self.rt.block_on(self.inner.close())
    }

    /// Pings the database to verify connectivity.
    pub fn ping(&self) -> Result<()> {
        self.rt.block_on(self.inner.ping())
    }

    /// Executes a statement through the driver executor and returns affected rows.
    pub fn execute(&self, sql: &str) -> Result<u64> {
        self.rt.block_on(self.inner.execute(sql))
    }

    /// Executes a parameterized statement through the driver executor.
    pub fn execute_params(&self, sql: &str, params: &[ExtractedValue]) -> Result<u64> {
        self.rt.block_on(self.inner.execute_params(sql, params))
    }

    /// Fetches rows through the driver executor.
    pub fn fetch_rows(&self, sql: &str, params: &[ExtractedValue]) -> Result<Vec<Row>> {
        self.rt.block_on(self.inner.fetch_rows(sql, params))
    }

    /// Fetches the first row, if present, through the driver executor.
    pub fn fetch_optional_row(&self, sql: &str, params: &[ExtractedValue]) -> Result<Option<Row>> {
        self.rt.block_on(self.inner.fetch_optional_row(sql, params))
    }
}

fn build_runtime() -> Result<Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("cclab-pg-blocking")
        .build()
        .map_err(|e| {
            crate::DataBridgeError::Connection(format!(
                "failed to build blocking tokio runtime: {e}"
            ))
        })
}

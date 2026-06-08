//! Blocking facade for [`crate::driver::QueryExecutor`].

// HANDWRITE-BEGIN reason: facade-mirror codegen gap; see row.rs in
//   the same directory.
//! @spec
//!   .aw/tech-design/projects/pg/specs/pg-blocking-facade-row-bulk-executor.md#changes

use std::sync::Arc;
use tokio::runtime::Runtime;

use sqlx::postgres::{PgArguments, PgRow};
use sqlx::query::Query;
use sqlx::{FromRow, Postgres};

use crate::driver::blocking::Connection;
use crate::driver::{ExecutorConfig, ExtractedValue, QueryExecutor as AsyncQueryExecutor, Row};
use crate::Result;

/// Blocking sibling for [`crate::driver::QueryExecutor`].
///
/// Borrows from the `Connection` for the lifetime `'a` and shares
/// its runtime.
pub struct QueryExecutor<'a> {
    inner: AsyncQueryExecutor<'a>,
    rt: Arc<Runtime>,
}

impl<'a> QueryExecutor<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            inner: AsyncQueryExecutor::new(conn.as_async().pool()),
            rt: conn.runtime(),
        }
    }

    pub fn with_config(conn: &'a Connection, config: ExecutorConfig) -> Self {
        Self {
            inner: AsyncQueryExecutor::with_config(conn.as_async().pool(), config),
            rt: conn.runtime(),
        }
    }

    pub fn fetch_all<T, F>(&self, sql: &str, bind_fn: F) -> Result<Vec<T>>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
        F: Fn(Query<'_, Postgres, PgArguments>) -> Query<'_, Postgres, PgArguments>
            + Clone,
    {
        self.rt.block_on(self.inner.fetch_all(sql, bind_fn))
    }

    pub fn execute(&self, sql: &str) -> Result<u64> {
        self.rt.block_on(self.inner.execute(sql))
    }

    pub fn execute_params(&self, sql: &str, params: &[ExtractedValue]) -> Result<u64> {
        self.rt.block_on(self.inner.execute_params(sql, params))
    }

    pub fn fetch_rows(&self, sql: &str, params: &[ExtractedValue]) -> Result<Vec<Row>> {
        self.rt.block_on(self.inner.fetch_rows(sql, params))
    }

    pub fn fetch_optional_row(
        &self,
        sql: &str,
        params: &[ExtractedValue],
    ) -> Result<Option<Row>> {
        self.rt.block_on(self.inner.fetch_optional_row(sql, params))
    }
}

// HANDWRITE-END

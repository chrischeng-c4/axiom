//! Blocking facade for [`crate::driver::BulkExecutor`].

// HANDWRITE-BEGIN reason: facade-mirror codegen gap; see row.rs in
//   the same directory. Closes when score grows a `facade-mirror`
//   section type.
//! @spec
//!   .aw/tech-design/projects/pg/specs/pg-blocking-facade-row-bulk-executor.md#changes

use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::driver::blocking::Connection;
use crate::driver::{BulkConfig, BulkExecutor as AsyncBulkExecutor, BulkResult, ExtractedValue};
use crate::Result;

/// Blocking sibling for [`crate::driver::BulkExecutor`].
///
/// Wraps an async `BulkExecutor` plus a clone of the
/// `Connection`'s runtime so each method is a single
/// `rt.block_on(inner.<fn>(...))` pass-through.
pub struct BulkExecutor {
    inner: AsyncBulkExecutor,
    rt: Arc<Runtime>,
}

impl BulkExecutor {
    pub fn new(conn: &Connection, config: BulkConfig) -> Self {
        Self {
            inner: AsyncBulkExecutor::new(conn.as_async(), config),
            rt: conn.runtime(),
        }
    }

    pub fn insert_parallel(
        &self,
        table: &str,
        rows: &[HashMap<String, ExtractedValue>],
    ) -> Result<BulkResult> {
        self.rt.block_on(self.inner.insert_parallel(table, rows))
    }

    pub fn update_parallel(
        &self,
        table: &str,
        rows: &[HashMap<String, ExtractedValue>],
    ) -> Result<BulkResult> {
        self.rt.block_on(self.inner.update_parallel(table, rows))
    }

    pub fn delete_parallel(&self, table: &str, ids: &[i64]) -> Result<BulkResult> {
        self.rt.block_on(self.inner.delete_parallel(table, ids))
    }
}

// HANDWRITE-END

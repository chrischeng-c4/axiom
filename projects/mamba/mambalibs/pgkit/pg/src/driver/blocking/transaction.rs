//! Blocking façade over [`crate::Transaction`].

use std::sync::Arc;

use tokio::runtime::Runtime;

use crate::{IsolationLevel, Result, Transaction as AsyncTransaction, TransactionOptions};

use super::Connection;

/// Blocking transaction handle. Drops mirror async semantics: an
/// uncommitted transaction is rolled back when the wrapper is dropped
/// (delegated to the inner async [`AsyncTransaction`]). The wrapper's
/// `Drop` enters the owned tokio runtime so sqlx's auto-rollback has
/// a Tokio context available, even if the caller drops on a plain
/// thread.
pub struct Transaction {
    inner: Option<AsyncTransaction>,
    rt: Arc<Runtime>,
}

impl std::fmt::Debug for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("blocking::Transaction")
            .finish_non_exhaustive()
    }
}

impl Transaction {
    /// Returns a mutable reference to the inner sqlx transaction.
    /// Use this to run sqlx queries against the transaction directly.
    pub fn as_mut_transaction(&mut self) -> &mut sqlx::Transaction<'static, sqlx::Postgres> {
        self.inner
            .as_mut()
            .expect("transaction already consumed")
            .as_mut_transaction()
    }

    /// Begins a new transaction with a specific isolation level.
    pub fn begin(conn: &Connection, isolation_level: IsolationLevel) -> Result<Self> {
        let rt = conn.runtime();
        let inner = rt.block_on(AsyncTransaction::begin(conn.as_async(), isolation_level))?;
        Ok(Self {
            inner: Some(inner),
            rt,
        })
    }

    /// Begins a new transaction with full options control.
    pub fn begin_with_options(conn: &Connection, options: TransactionOptions) -> Result<Self> {
        let rt = conn.runtime();
        let inner = rt.block_on(AsyncTransaction::begin_with_options(
            conn.as_async(),
            options,
        ))?;
        Ok(Self {
            inner: Some(inner),
            rt,
        })
    }

    /// Commits the transaction.
    pub fn commit(mut self) -> Result<()> {
        let inner = self.inner.take().expect("transaction already consumed");
        self.rt.block_on(inner.commit())
    }

    /// Rolls back the transaction.
    pub fn rollback(mut self) -> Result<()> {
        let inner = self.inner.take().expect("transaction already consumed");
        self.rt.block_on(inner.rollback())
    }

    /// Creates a savepoint within the transaction.
    pub fn savepoint(&mut self, name: &str) -> Result<()> {
        let inner = self.inner.as_mut().expect("transaction already consumed");
        self.rt.block_on(inner.savepoint(name))
    }

    /// Rolls back to a savepoint.
    pub fn rollback_to(&mut self, name: &str) -> Result<()> {
        let inner = self.inner.as_mut().expect("transaction already consumed");
        self.rt.block_on(inner.rollback_to(name))
    }

    /// Releases a savepoint.
    pub fn release_savepoint(&mut self, name: &str) -> Result<()> {
        let inner = self.inner.as_mut().expect("transaction already consumed");
        self.rt.block_on(inner.release_savepoint(name))
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        // If commit/rollback wasn't called, drop the inner transaction
        // inside the runtime context so sqlx's auto-rollback Drop has a
        // Tokio handle to spawn on.
        if let Some(inner) = self.inner.take() {
            let _guard = self.rt.enter();
            drop(inner);
        }
    }
}

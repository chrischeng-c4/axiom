//! PostgreSQL transaction management.
//!
//! This module provides transaction support with ACID guarantees.

use crate::{Connection, Result};
use sqlx::Postgres;

/// Transaction isolation levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IsolationLevel {
    /// Read uncommitted
    ReadUncommitted,
    /// Read committed (PostgreSQL default)
    #[default]
    ReadCommitted,
    /// Repeatable read
    RepeatableRead,
    /// Serializable
    Serializable,
}

impl IsolationLevel {
    /// Returns the SQL isolation level string.
    pub fn to_sql(&self) -> &'static str {
        match self {
            IsolationLevel::ReadUncommitted => "READ UNCOMMITTED",
            IsolationLevel::ReadCommitted => "READ COMMITTED",
            IsolationLevel::RepeatableRead => "REPEATABLE READ",
            IsolationLevel::Serializable => "SERIALIZABLE",
        }
    }
}

/// Transaction access mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccessMode {
    /// Read-write transaction (default)
    #[default]
    ReadWrite,
    /// Read-only transaction - no writes allowed
    ReadOnly,
}

impl AccessMode {
    /// Returns the SQL access mode string.
    pub fn to_sql(&self) -> &'static str {
        match self {
            AccessMode::ReadWrite => "READ WRITE",
            AccessMode::ReadOnly => "READ ONLY",
        }
    }
}

/// Transaction options for fine-grained control over transaction behavior.
///
/// # Examples
///
/// ```rust,ignore
/// // Default read-write transaction
/// let options = TransactionOptions::default();
///
/// // Read-only transaction (useful for reports)
/// let options = TransactionOptions::new()
///     .read_only()
///     .isolation_level(IsolationLevel::RepeatableRead);
///
/// // Serializable snapshot isolation (useful for money transfers)
/// let options = TransactionOptions::new()
///     .isolation_level(IsolationLevel::Serializable)
///     .deferrable(true); // May wait to ensure serializability
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TransactionOptions {
    /// Isolation level for the transaction
    pub isolation_level: IsolationLevel,
    /// Access mode (read-write or read-only)
    pub access_mode: AccessMode,
    /// Whether the transaction is deferrable (only meaningful for SERIALIZABLE READ ONLY)
    ///
    /// A deferrable transaction may block waiting for other serializable transactions
    /// to complete, but once started, it will never be aborted due to serialization conflicts.
    /// This is useful for long-running read-only queries.
    pub deferrable: bool,
}

impl TransactionOptions {
    /// Create new transaction options with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the isolation level.
    pub fn isolation_level(mut self, level: IsolationLevel) -> Self {
        self.isolation_level = level;
        self
    }

    /// Make the transaction read-only.
    pub fn read_only(mut self) -> Self {
        self.access_mode = AccessMode::ReadOnly;
        self
    }

    /// Make the transaction read-write (default).
    pub fn read_write(mut self) -> Self {
        self.access_mode = AccessMode::ReadWrite;
        self
    }

    /// Set whether the transaction is deferrable.
    ///
    /// Note: DEFERRABLE is only meaningful for SERIALIZABLE READ ONLY transactions.
    /// For other transaction types, it has no effect.
    pub fn deferrable(mut self, deferrable: bool) -> Self {
        self.deferrable = deferrable;
        self
    }

    /// Build the SQL string for SET TRANSACTION command.
    pub fn to_sql(&self) -> String {
        let mut parts = vec![
            format!("ISOLATION LEVEL {}", self.isolation_level.to_sql()),
            self.access_mode.to_sql().to_string(),
        ];

        // DEFERRABLE is only valid for SERIALIZABLE READ ONLY
        if self.isolation_level == IsolationLevel::Serializable
            && self.access_mode == AccessMode::ReadOnly
        {
            if self.deferrable {
                parts.push("DEFERRABLE".to_string());
            } else {
                parts.push("NOT DEFERRABLE".to_string());
            }
        }

        format!("SET TRANSACTION {}", parts.join(", "))
    }
}

/// Represents an active database transaction.
///
/// Transactions ensure ACID properties:
/// - Atomicity: All operations succeed or fail together
/// - Consistency: Database remains in a valid state
/// - Isolation: Concurrent transactions don't interfere
/// - Durability: Committed changes persist
pub struct Transaction {
    tx: sqlx::Transaction<'static, Postgres>,
}

impl std::fmt::Debug for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transaction").finish_non_exhaustive()
    }
}

impl Transaction {
    /// Returns a mutable reference to the underlying sqlx transaction for direct query execution.
    pub fn as_mut_transaction(&mut self) -> &mut sqlx::Transaction<'static, Postgres> {
        &mut self.tx
    }

    /// Begins a new transaction with a specific isolation level.
    ///
    /// # Arguments
    ///
    /// * `conn` - Connection to use for transaction
    /// * `isolation_level` - Transaction isolation level
    ///
    /// # Errors
    ///
    /// Returns error if transaction cannot be started.
    pub async fn begin(conn: &Connection, isolation_level: IsolationLevel) -> Result<Self> {
        let options = TransactionOptions::new().isolation_level(isolation_level);
        Self::begin_with_options(conn, options).await
    }

    /// Begins a new transaction with full options control.
    ///
    /// # Arguments
    ///
    /// * `conn` - Connection to use for transaction
    /// * `options` - Transaction options including isolation level, access mode, and deferrable
    ///
    /// # Errors
    ///
    /// Returns error if transaction cannot be started.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Read-only transaction for reports
    /// let options = TransactionOptions::new()
    ///     .read_only()
    ///     .isolation_level(IsolationLevel::RepeatableRead);
    /// let tx = Transaction::begin_with_options(&conn, options).await?;
    ///
    /// // Deferrable serializable read-only transaction (won't abort due to conflicts)
    /// let options = TransactionOptions::new()
    ///     .isolation_level(IsolationLevel::Serializable)
    ///     .read_only()
    ///     .deferrable(true);
    /// let tx = Transaction::begin_with_options(&conn, options).await?;
    /// ```
    pub async fn begin_with_options(
        conn: &Connection,
        options: TransactionOptions,
    ) -> Result<Self> {
        // Begin transaction from pool
        let mut tx = conn.pool().begin().await?;

        // Set transaction options
        let sql = options.to_sql();
        sqlx::query(&sql).execute(&mut *tx).await?;

        tracing::debug!(
            isolation_level = ?options.isolation_level,
            access_mode = ?options.access_mode,
            deferrable = options.deferrable,
            "Started transaction"
        );

        Ok(Self { tx })
    }

    /// Commits the transaction.
    ///
    /// # Errors
    ///
    /// Returns error if commit fails.
    pub async fn commit(self) -> Result<()> {
        self.tx.commit().await?;
        Ok(())
    }

    /// Rolls back the transaction.
    ///
    /// # Errors
    ///
    /// Returns error if rollback fails.
    pub async fn rollback(self) -> Result<()> {
        self.tx.rollback().await?;
        Ok(())
    }

    /// Creates a savepoint within the transaction.
    ///
    /// # Arguments
    ///
    /// * `name` - Savepoint name
    ///
    /// # Errors
    ///
    /// Returns error if savepoint name is invalid or creation fails.
    pub async fn savepoint(&mut self, name: &str) -> Result<()> {
        use crate::QueryBuilder;

        // Validate savepoint name to prevent SQL injection
        QueryBuilder::validate_identifier(name)?;

        // Execute SAVEPOINT statement
        let sql = format!("SAVEPOINT {}", name);
        sqlx::query(&sql).execute(&mut *self.tx).await?;

        tracing::debug!(savepoint = name, "Created savepoint");
        Ok(())
    }

    /// Rolls back to a savepoint.
    ///
    /// # Arguments
    ///
    /// * `name` - Savepoint name
    ///
    /// # Errors
    ///
    /// Returns error if savepoint name is invalid or rollback fails.
    pub async fn rollback_to(&mut self, name: &str) -> Result<()> {
        use crate::QueryBuilder;

        // Validate savepoint name to prevent SQL injection
        QueryBuilder::validate_identifier(name)?;

        // Execute ROLLBACK TO SAVEPOINT statement
        let sql = format!("ROLLBACK TO SAVEPOINT {}", name);
        sqlx::query(&sql).execute(&mut *self.tx).await?;

        tracing::debug!(savepoint = name, "Rolled back to savepoint");
        Ok(())
    }

    /// Releases a savepoint.
    ///
    /// # Arguments
    ///
    /// * `name` - Savepoint name
    ///
    /// # Errors
    ///
    /// Returns error if savepoint name is invalid or release fails.
    pub async fn release_savepoint(&mut self, name: &str) -> Result<()> {
        use crate::QueryBuilder;

        // Validate savepoint name to prevent SQL injection
        QueryBuilder::validate_identifier(name)?;

        // Execute RELEASE SAVEPOINT statement
        let sql = format!("RELEASE SAVEPOINT {}", name);
        sqlx::query(&sql).execute(&mut *self.tx).await?;

        tracing::debug!(savepoint = name, "Released savepoint");
        Ok(())
    }
}

// Auto-rollback on drop if not committed
// Note: SQLx Transaction already implements Drop with auto-rollback.
// We don't need to manually implement rollback here as SQLx handles it.
// The inner tx will be dropped when this struct is dropped, triggering SQLx's Drop impl.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_level_to_sql() {
        assert_eq!(IsolationLevel::ReadUncommitted.to_sql(), "READ UNCOMMITTED");
        assert_eq!(IsolationLevel::ReadCommitted.to_sql(), "READ COMMITTED");
        assert_eq!(IsolationLevel::RepeatableRead.to_sql(), "REPEATABLE READ");
        assert_eq!(IsolationLevel::Serializable.to_sql(), "SERIALIZABLE");
    }

    #[test]
    fn test_isolation_level_equality() {
        assert_eq!(IsolationLevel::ReadCommitted, IsolationLevel::ReadCommitted);
        assert_ne!(IsolationLevel::ReadCommitted, IsolationLevel::Serializable);
    }

    #[test]
    fn test_isolation_level_clone() {
        let level = IsolationLevel::Serializable;
        let cloned = level;
        assert_eq!(level, cloned);
    }

    #[test]
    fn test_access_mode_to_sql() {
        assert_eq!(AccessMode::ReadWrite.to_sql(), "READ WRITE");
        assert_eq!(AccessMode::ReadOnly.to_sql(), "READ ONLY");
    }

    #[test]
    fn test_transaction_options_default() {
        let options = TransactionOptions::default();
        assert_eq!(options.isolation_level, IsolationLevel::ReadCommitted);
        assert_eq!(options.access_mode, AccessMode::ReadWrite);
        assert!(!options.deferrable);
    }

    #[test]
    fn test_transaction_options_builder() {
        let options = TransactionOptions::new()
            .isolation_level(IsolationLevel::Serializable)
            .read_only()
            .deferrable(true);

        assert_eq!(options.isolation_level, IsolationLevel::Serializable);
        assert_eq!(options.access_mode, AccessMode::ReadOnly);
        assert!(options.deferrable);
    }

    #[test]
    fn test_transaction_options_to_sql_default() {
        let options = TransactionOptions::default();
        let sql = options.to_sql();
        assert_eq!(
            sql,
            "SET TRANSACTION ISOLATION LEVEL READ COMMITTED, READ WRITE"
        );
    }

    #[test]
    fn test_transaction_options_to_sql_read_only() {
        let options = TransactionOptions::new()
            .isolation_level(IsolationLevel::RepeatableRead)
            .read_only();
        let sql = options.to_sql();
        assert_eq!(
            sql,
            "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY"
        );
    }

    #[test]
    fn test_transaction_options_to_sql_serializable_deferrable() {
        // DEFERRABLE is only valid for SERIALIZABLE READ ONLY
        let options = TransactionOptions::new()
            .isolation_level(IsolationLevel::Serializable)
            .read_only()
            .deferrable(true);
        let sql = options.to_sql();
        assert_eq!(
            sql,
            "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE, READ ONLY, DEFERRABLE"
        );
    }

    #[test]
    fn test_transaction_options_to_sql_serializable_not_deferrable() {
        let options = TransactionOptions::new()
            .isolation_level(IsolationLevel::Serializable)
            .read_only()
            .deferrable(false);
        let sql = options.to_sql();
        assert_eq!(
            sql,
            "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE, READ ONLY, NOT DEFERRABLE"
        );
    }

    #[test]
    fn test_transaction_options_deferrable_ignored_for_read_write() {
        // DEFERRABLE should not appear for READ WRITE transactions
        let options = TransactionOptions::new()
            .isolation_level(IsolationLevel::Serializable)
            .read_write()
            .deferrable(true);
        let sql = options.to_sql();
        // Should NOT contain DEFERRABLE since it's READ WRITE
        assert_eq!(
            sql,
            "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE, READ WRITE"
        );
    }

    #[test]
    fn test_transaction_options_deferrable_ignored_for_non_serializable() {
        // DEFERRABLE should not appear for non-SERIALIZABLE transactions
        let options = TransactionOptions::new()
            .isolation_level(IsolationLevel::RepeatableRead)
            .read_only()
            .deferrable(true);
        let sql = options.to_sql();
        // Should NOT contain DEFERRABLE since it's not SERIALIZABLE
        assert_eq!(
            sql,
            "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY"
        );
    }

    // Integration tests require a live PostgreSQL database
    // Run these with: cargo test --package cclab-titan --features test-postgres
    // TODO: Add integration tests for:
    // - test_transaction_commit()
    // - test_transaction_rollback()
    // - test_transaction_auto_rollback_on_drop()
    // - test_transaction_isolation_levels()
}

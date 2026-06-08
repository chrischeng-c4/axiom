//! Query execution utilities with retry logic and observability.
//!
//! This module provides query execution with:
//! - Automatic retry on transient errors (deadlock, serialization failure)
//! - Tracing spans for query monitoring
//! - Slow query logging
//! - Error context enrichment

use std::time::{Duration, Instant};

use sqlx::postgres::{PgArguments, PgPool};
use sqlx::{Error as SqlxError, FromRow, Postgres};
use tracing::{debug, instrument, warn};

use crate::driver::row::Row as DriverRow;
use crate::driver::types::ExtractedValue;
use crate::{DataBridgeError, Result};

/// Configuration for query execution with retry support.
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Maximum number of retries for transient errors
    pub max_retries: u32,
    /// Initial delay between retries in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Threshold for slow query logging in milliseconds
    pub slow_query_threshold_ms: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 50,
            max_delay_ms: 2000,
            backoff_multiplier: 2.0,
            slow_query_threshold_ms: 1000, // 1 second
        }
    }
}

impl ExecutorConfig {
    /// Create a new executor config with no retries.
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            ..Default::default()
        }
    }

    /// Calculate delay for a given retry attempt.
    fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(self.initial_delay_ms);
        }

        let delay_ms =
            (self.initial_delay_ms as f64) * self.backoff_multiplier.powi(attempt as i32);

        Duration::from_millis((delay_ms as u64).min(self.max_delay_ms))
    }
}

/// Query executor with retry and observability support.
pub struct QueryExecutor<'a> {
    pool: &'a PgPool,
    config: ExecutorConfig,
}

impl<'a> QueryExecutor<'a> {
    /// Create a new query executor.
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            config: ExecutorConfig::default(),
        }
    }

    /// Create a query executor with custom configuration.
    pub fn with_config(pool: &'a PgPool, config: ExecutorConfig) -> Self {
        Self { pool, config }
    }

    /// Execute a query that returns rows with retry support.
    #[instrument(skip(self, sql, bind_fn), fields(sql_preview = %sql.chars().take(100).collect::<String>()))]
    pub async fn fetch_all<T, F>(&self, sql: &str, bind_fn: F) -> Result<Vec<T>>
    where
        T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
        F: Fn(
                sqlx::query::Query<'_, Postgres, sqlx::postgres::PgArguments>,
            ) -> sqlx::query::Query<'_, Postgres, sqlx::postgres::PgArguments>
            + Clone,
    {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            let start = Instant::now();

            // Note: bind_fn is reserved for future use with parameterized queries
            let _ = bind_fn(sqlx::query(sql));

            match sqlx::query_as::<_, T>(sql).fetch_all(self.pool).await {
                Ok(rows) => {
                    let elapsed = start.elapsed();
                    self.log_query_completion(sql, elapsed, attempt);
                    return Ok(rows);
                }
                Err(e) => {
                    let elapsed = start.elapsed();
                    let is_retryable = Self::is_retryable_error(&e);

                    warn!(
                        sql = %sql.chars().take(50).collect::<String>(),
                        attempt = attempt,
                        elapsed_ms = elapsed.as_millis() as u64,
                        retryable = is_retryable,
                        error = %e,
                        "Query failed"
                    );

                    if is_retryable && attempt < self.config.max_retries {
                        let delay = self.config.delay_for_attempt(attempt);
                        debug!(delay_ms = delay.as_millis() as u64, "Retrying after delay");
                        tokio::time::sleep(delay).await;
                        last_error = Some(e);
                        continue;
                    }

                    return Err(DataBridgeError::from(e));
                }
            }
        }

        Err(last_error.map(DataBridgeError::from).unwrap_or_else(|| {
            DataBridgeError::Query("Query failed after all retries".to_string())
        }))
    }

    /// Execute a query that affects rows (INSERT, UPDATE, DELETE) with retry support.
    #[instrument(skip(self), fields(sql_preview = %sql.chars().take(100).collect::<String>()))]
    pub async fn execute(&self, sql: &str) -> Result<u64> {
        self.execute_with_args(sql, || Ok(PgArguments::default()))
            .await
    }

    /// Execute a parameterized query with driver `ExtractedValue` bindings.
    #[instrument(skip(self, params), fields(
        sql_preview = %sql.chars().take(100).collect::<String>(),
        param_count = params.len()
    ))]
    pub async fn execute_params(&self, sql: &str, params: &[ExtractedValue]) -> Result<u64> {
        self.execute_with_args(sql, || bind_params(params)).await
    }

    /// Fetch all result rows with driver `ExtractedValue` parameter bindings.
    #[instrument(skip(self, params), fields(
        sql_preview = %sql.chars().take(100).collect::<String>(),
        param_count = params.len()
    ))]
    pub async fn fetch_rows(&self, sql: &str, params: &[ExtractedValue]) -> Result<Vec<DriverRow>> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            let start = Instant::now();
            let args = bind_params(params)?;

            match sqlx::query_with(sql, args).fetch_all(self.pool).await {
                Ok(rows) => {
                    let elapsed = start.elapsed();
                    let rows = rows
                        .iter()
                        .map(DriverRow::from_sqlx)
                        .collect::<Result<Vec<_>>>()?;
                    self.log_query_completion(sql, elapsed, attempt);
                    debug!(row_count = rows.len(), "Query fetched rows successfully");
                    return Ok(rows);
                }
                Err(e) => {
                    let elapsed = start.elapsed();
                    let is_retryable = Self::is_retryable_error(&e);

                    warn!(
                        sql = %sql.chars().take(50).collect::<String>(),
                        attempt = attempt,
                        elapsed_ms = elapsed.as_millis() as u64,
                        retryable = is_retryable,
                        error = %e,
                        "Query failed"
                    );

                    if is_retryable && attempt < self.config.max_retries {
                        let delay = self.config.delay_for_attempt(attempt);
                        debug!(delay_ms = delay.as_millis() as u64, "Retrying after delay");
                        tokio::time::sleep(delay).await;
                        last_error = Some(e);
                        continue;
                    }

                    return Err(DataBridgeError::from(e));
                }
            }
        }

        Err(last_error.map(DataBridgeError::from).unwrap_or_else(|| {
            DataBridgeError::Query("Query failed after all retries".to_string())
        }))
    }

    /// Fetch the first row, if present, with driver `ExtractedValue` parameter bindings.
    #[instrument(skip(self, params), fields(
        sql_preview = %sql.chars().take(100).collect::<String>(),
        param_count = params.len()
    ))]
    pub async fn fetch_optional_row(
        &self,
        sql: &str,
        params: &[ExtractedValue],
    ) -> Result<Option<DriverRow>> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            let start = Instant::now();
            let args = bind_params(params)?;

            match sqlx::query_with(sql, args).fetch_optional(self.pool).await {
                Ok(row) => {
                    let elapsed = start.elapsed();
                    let row = row.as_ref().map(DriverRow::from_sqlx).transpose()?;
                    self.log_query_completion(sql, elapsed, attempt);
                    debug!(
                        found = row.is_some(),
                        "Query fetched optional row successfully"
                    );
                    return Ok(row);
                }
                Err(e) => {
                    let elapsed = start.elapsed();
                    let is_retryable = Self::is_retryable_error(&e);

                    warn!(
                        sql = %sql.chars().take(50).collect::<String>(),
                        attempt = attempt,
                        elapsed_ms = elapsed.as_millis() as u64,
                        retryable = is_retryable,
                        error = %e,
                        "Query failed"
                    );

                    if is_retryable && attempt < self.config.max_retries {
                        let delay = self.config.delay_for_attempt(attempt);
                        debug!(delay_ms = delay.as_millis() as u64, "Retrying after delay");
                        tokio::time::sleep(delay).await;
                        last_error = Some(e);
                        continue;
                    }

                    return Err(DataBridgeError::from(e));
                }
            }
        }

        Err(last_error.map(DataBridgeError::from).unwrap_or_else(|| {
            DataBridgeError::Query("Query failed after all retries".to_string())
        }))
    }

    async fn execute_with_args<F>(&self, sql: &str, make_args: F) -> Result<u64>
    where
        F: Fn() -> Result<PgArguments>,
    {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            let start = Instant::now();
            let args = make_args()?;

            match sqlx::query_with(sql, args).execute(self.pool).await {
                Ok(result) => {
                    let elapsed = start.elapsed();
                    let rows_affected = result.rows_affected();
                    self.log_query_completion(sql, elapsed, attempt);
                    debug!(rows_affected = rows_affected, "Query executed successfully");
                    return Ok(rows_affected);
                }
                Err(e) => {
                    let elapsed = start.elapsed();
                    let is_retryable = Self::is_retryable_error(&e);

                    warn!(
                        sql = %sql.chars().take(50).collect::<String>(),
                        attempt = attempt,
                        elapsed_ms = elapsed.as_millis() as u64,
                        retryable = is_retryable,
                        error = %e,
                        "Query failed"
                    );

                    if is_retryable && attempt < self.config.max_retries {
                        let delay = self.config.delay_for_attempt(attempt);
                        debug!(delay_ms = delay.as_millis() as u64, "Retrying after delay");
                        tokio::time::sleep(delay).await;
                        last_error = Some(e);
                        continue;
                    }

                    return Err(DataBridgeError::from(e));
                }
            }
        }

        Err(last_error.map(DataBridgeError::from).unwrap_or_else(|| {
            DataBridgeError::Query("Query failed after all retries".to_string())
        }))
    }

    /// Check if an error is retryable (deadlock, serialization failure, etc.)
    fn is_retryable_error(err: &SqlxError) -> bool {
        if let SqlxError::Database(db_err) = err {
            if let Some(code) = db_err.code() {
                let code_str: &str = &code;
                return matches!(
                    code_str,
                    // Deadlock detected
                    "40P01" |
                    // Serialization failure
                    "40001" |
                    // Transaction rollback (class 40)
                    "40000" | "40002" | "40003" |
                    // Admin shutdown / crash recovery
                    "57P01" | "57P02" | "57P03"
                );
            }
        }

        // Connection errors are generally retryable
        matches!(err, SqlxError::Io(_) | SqlxError::PoolTimedOut)
    }

    /// Log query completion with slow query detection.
    fn log_query_completion(&self, sql: &str, elapsed: Duration, attempt: u32) {
        let elapsed_ms = elapsed.as_millis() as u64;
        let sql_preview: String = sql.chars().take(100).collect();

        if elapsed_ms >= self.config.slow_query_threshold_ms {
            warn!(
                sql = %sql_preview,
                elapsed_ms = elapsed_ms,
                threshold_ms = self.config.slow_query_threshold_ms,
                attempt = attempt,
                "Slow query detected"
            );
        } else {
            debug!(
                sql = %sql_preview,
                elapsed_ms = elapsed_ms,
                attempt = attempt,
                "Query completed"
            );
        }
    }
}

fn bind_params(params: &[ExtractedValue]) -> Result<PgArguments> {
    let mut args = PgArguments::default();
    for param in params {
        param.bind_to_arguments(&mut args)?;
    }
    Ok(args)
}

/// Execute a raw SQL query with basic retry support.
///
/// This is a convenience function for simple query execution without
/// the full `QueryExecutor` setup.
pub async fn execute_with_retry(pool: &PgPool, sql: &str, max_retries: u32) -> Result<u64> {
    let config = ExecutorConfig {
        max_retries,
        ..Default::default()
    };
    QueryExecutor::with_config(pool, config).execute(sql).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 50);
        assert_eq!(config.max_delay_ms, 2000);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.slow_query_threshold_ms, 1000);
    }

    #[test]
    fn test_executor_config_no_retry() {
        let config = ExecutorConfig::no_retry();
        assert_eq!(config.max_retries, 0);
    }

    #[test]
    fn test_delay_calculation() {
        let config = ExecutorConfig {
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            ..Default::default()
        };

        // Attempt 0: initial delay
        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));

        // Attempt 1: 100 * 2^1 = 200
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));

        // Attempt 2: 100 * 2^2 = 400
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));

        // Attempt 5: 100 * 2^5 = 3200
        assert_eq!(config.delay_for_attempt(5), Duration::from_millis(3200));

        // Attempt 6: 100 * 2^6 = 6400, capped at 5000
        assert_eq!(config.delay_for_attempt(6), Duration::from_millis(5000));
    }

    #[test]
    fn test_bind_params_accepts_common_extracted_values() {
        let params = vec![
            ExtractedValue::Null,
            ExtractedValue::Bool(true),
            ExtractedValue::Int(42),
            ExtractedValue::BigInt(9001),
            ExtractedValue::Double(3.5),
            ExtractedValue::String("mamba".to_string()),
        ];

        assert!(bind_params(&params).is_ok());
    }

    #[test]
    fn test_bind_params_accepts_empty_params() {
        assert!(bind_params(&[]).is_ok());
    }
}

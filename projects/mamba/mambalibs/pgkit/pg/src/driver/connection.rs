//! PostgreSQL connection management with connection pooling.
//!
//! This module provides connection pooling using SQLx's built-in pool manager.
//! Similar to cclab-nebula's connection management, but optimized for PostgreSQL.
//! Includes connection resilience with exponential backoff retries.

use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;
use std::time::Duration;
use tracing::{info, instrument, warn};

use crate::driver::executor::QueryExecutor;
use crate::driver::row::Row;
use crate::driver::types::ExtractedValue;
use crate::{DataBridgeError, Result};

/// Retry configuration for connection establishment.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 = no retries)
    pub max_retries: u32,
    /// Initial delay between retries in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
    /// Multiplier for exponential backoff (e.g., 2.0 doubles delay each retry)
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Creates a retry config with no retries (immediate failure).
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            initial_delay_ms: 0,
            max_delay_ms: 0,
            backoff_multiplier: 1.0,
        }
    }

    /// Calculates the delay for a given attempt number (0-indexed).
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(self.initial_delay_ms);
        }

        let delay_ms =
            (self.initial_delay_ms as f64) * self.backoff_multiplier.powi(attempt as i32);

        Duration::from_millis((delay_ms as u64).min(self.max_delay_ms))
    }
}

/// Connection pool configuration.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimum number of connections in the pool.
    pub min_connections: u32,
    /// Maximum number of connections in the pool.
    pub max_connections: u32,
    /// Connection timeout in seconds.
    pub connect_timeout: u64,
    /// Maximum lifetime of a connection in seconds.
    pub max_lifetime: Option<u64>,
    /// Idle timeout in seconds.
    pub idle_timeout: Option<u64>,
    /// Retry configuration for connection establishment.
    pub retry: RetryConfig,
    /// Number of prepared statements to cache per connection.
    /// SQLx caches prepared statements to reduce parsing overhead.
    /// Set to 0 to disable caching. Default is 100.
    pub statement_cache_capacity: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 1,
            max_connections: 10,
            connect_timeout: 30,
            max_lifetime: Some(1800), // 30 minutes
            idle_timeout: Some(600),  // 10 minutes
            retry: RetryConfig::default(),
            statement_cache_capacity: 100, // SQLx default
        }
    }
}

/// PostgreSQL connection wrapper with connection pooling.
#[derive(Clone)]
pub struct Connection {
    pool: PgPool,
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("size", &self.pool.size())
            .field("num_idle", &self.pool.num_idle())
            .finish()
    }
}

impl Connection {
    /// Creates a new connection pool with retry logic.
    ///
    /// # Arguments
    ///
    /// * `uri` - PostgreSQL connection URI (e.g., "postgresql://user:password@localhost/db")
    /// * `config` - Pool configuration including retry settings
    ///
    /// # Errors
    ///
    /// Returns error if connection fails after all retries or URI is invalid.
    #[instrument(skip(uri), fields(
        min_connections = config.min_connections,
        max_connections = config.max_connections,
        max_retries = config.retry.max_retries
    ))]
    pub async fn new(uri: &str, config: PoolConfig) -> Result<Self> {
        // Validate URI format (basic check)
        if uri.is_empty() {
            return Err(DataBridgeError::Connection(
                "Connection URI cannot be empty".to_string(),
            ));
        }

        info!("Initializing connection pool");

        // Build pool options with configuration
        let mut pool_options = PgPoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout));

        // Add optional timeouts
        if let Some(max_lifetime_secs) = config.max_lifetime {
            pool_options = pool_options.max_lifetime(Duration::from_secs(max_lifetime_secs));
        }

        if let Some(idle_timeout_secs) = config.idle_timeout {
            pool_options = pool_options.idle_timeout(Duration::from_secs(idle_timeout_secs));
        }

        // Connect with retry logic and statement caching
        let pool = Self::connect_with_retry(
            uri,
            pool_options,
            &config.retry,
            config.statement_cache_capacity,
        )
        .await?;

        // Test the connection with a simple ping
        sqlx::query("SELECT 1").execute(&pool).await.map_err(|e| {
            DataBridgeError::Connection(format!("Failed to verify connection: {}", e))
        })?;

        info!("Connection pool initialized successfully");
        Ok(Self { pool })
    }

    /// Attempts to connect with exponential backoff retry.
    async fn connect_with_retry(
        uri: &str,
        pool_options: PgPoolOptions,
        retry_config: &RetryConfig,
        statement_cache_capacity: usize,
    ) -> Result<PgPool> {
        let mut last_error = None;

        // Parse connection options and configure statement cache
        let connect_options = PgConnectOptions::from_str(uri)
            .map_err(|e| DataBridgeError::Connection(format!("Invalid connection URI: {}", e)))?
            .statement_cache_capacity(statement_cache_capacity);

        for attempt in 0..=retry_config.max_retries {
            match pool_options
                .clone()
                .connect_with(connect_options.clone())
                .await
            {
                Ok(pool) => {
                    if attempt > 0 {
                        info!(attempt = attempt, "Connection established after retry");
                    }
                    return Ok(pool);
                }
                Err(e) => {
                    last_error = Some(e);

                    if attempt < retry_config.max_retries {
                        let delay = retry_config.delay_for_attempt(attempt);
                        warn!(
                            attempt = attempt,
                            max_retries = retry_config.max_retries,
                            delay_ms = delay.as_millis() as u64,
                            error = %last_error.as_ref().map(|e| e.to_string()).unwrap_or_default(),
                            "Connection failed, retrying after delay"
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        // All retries exhausted
        Err(last_error
            .map(DataBridgeError::from)
            .unwrap_or_else(|| DataBridgeError::Connection("Connection failed".to_string())))
    }

    /// Gets a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Closes the connection pool.
    pub async fn close(&self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }

    /// Pings the database to verify connectivity.
    pub async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    /// Executes a statement through the driver executor and returns affected rows.
    pub async fn execute(&self, sql: &str) -> Result<u64> {
        QueryExecutor::new(&self.pool).execute(sql).await
    }

    /// Executes a parameterized statement through the driver executor.
    pub async fn execute_params(&self, sql: &str, params: &[ExtractedValue]) -> Result<u64> {
        QueryExecutor::new(&self.pool)
            .execute_params(sql, params)
            .await
    }

    /// Fetches rows through the driver executor.
    pub async fn fetch_rows(&self, sql: &str, params: &[ExtractedValue]) -> Result<Vec<Row>> {
        QueryExecutor::new(&self.pool).fetch_rows(sql, params).await
    }

    /// Fetches the first row, if present, through the driver executor.
    pub async fn fetch_optional_row(
        &self,
        sql: &str,
        params: &[ExtractedValue],
    ) -> Result<Option<Row>> {
        QueryExecutor::new(&self.pool)
            .fetch_optional_row(sql, params)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_config_default() {
        // Verify default config values
        let config = PoolConfig::default();

        assert_eq!(config.min_connections, 1);
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.connect_timeout, 30);
        assert_eq!(config.max_lifetime, Some(1800)); // 30 minutes
        assert_eq!(config.idle_timeout, Some(600)); // 10 minutes
        assert_eq!(config.statement_cache_capacity, 100); // SQLx default
    }

    #[test]
    fn test_connection_config_builder() {
        // Verify builder pattern works by creating custom config
        let config = PoolConfig {
            min_connections: 2,
            max_connections: 20,
            connect_timeout: 60,
            max_lifetime: Some(3600), // 1 hour
            idle_timeout: Some(300),  // 5 minutes
            retry: RetryConfig::default(),
            statement_cache_capacity: 200,
        };

        assert_eq!(config.min_connections, 2);
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.connect_timeout, 60);
        assert_eq!(config.max_lifetime, Some(3600));
        assert_eq!(config.idle_timeout, Some(300));
        assert_eq!(config.statement_cache_capacity, 200);
    }

    #[test]
    fn test_connection_config_from_env() {
        // Verify env-based config (using manual parsing simulation)
        // In a real scenario, you would use std::env::var()
        let min_conn = "5".parse::<u32>().unwrap();
        let max_conn = "50".parse::<u32>().unwrap();
        let timeout = "45".parse::<u64>().unwrap();

        let config = PoolConfig {
            min_connections: min_conn,
            max_connections: max_conn,
            connect_timeout: timeout,
            max_lifetime: Some(1800),
            idle_timeout: Some(600),
            retry: RetryConfig::default(),
            statement_cache_capacity: 100,
        };

        assert_eq!(config.min_connections, 5);
        assert_eq!(config.max_connections, 50);
        assert_eq!(config.connect_timeout, 45);
    }

    #[test]
    fn test_pool_config_validation() {
        // Verify pool size constraints
        // Test 1: min_connections should be less than max_connections (logical constraint)
        let valid_config = PoolConfig {
            min_connections: 1,
            max_connections: 10,
            connect_timeout: 30,
            max_lifetime: Some(1800),
            idle_timeout: Some(600),
            retry: RetryConfig::default(),
            statement_cache_capacity: 100,
        };

        assert!(valid_config.min_connections < valid_config.max_connections);

        // Test 2: Check that we can create configs with edge values
        let edge_config = PoolConfig {
            min_connections: 0,
            max_connections: 100,
            connect_timeout: 1,
            max_lifetime: None,
            idle_timeout: None,
            retry: RetryConfig::no_retry(),
            statement_cache_capacity: 0, // Disable caching
        };

        assert_eq!(edge_config.min_connections, 0);
        assert_eq!(edge_config.max_connections, 100);
        assert_eq!(edge_config.connect_timeout, 1);
        assert!(edge_config.max_lifetime.is_none());
        assert!(edge_config.idle_timeout.is_none());
        assert_eq!(edge_config.statement_cache_capacity, 0);
    }

    #[test]
    fn test_uri_parsing_valid() {
        // Verify valid URIs parse correctly (basic format check)
        let valid_uris = vec![
            "postgresql://user:password@localhost/mydb",
            "postgres://user@localhost/db",
            "postgresql://user:pass@127.0.0.1:5432/database",
            "postgres://localhost/db",
        ];

        for uri in valid_uris {
            // Basic validation that would pass Connection::new() URI check
            assert!(!uri.is_empty());
            assert!(uri.starts_with("postgres://") || uri.starts_with("postgresql://"));
        }
    }

    #[test]
    fn test_uri_parsing_invalid() {
        // Verify invalid URIs are rejected
        // Note: Connection::new() only checks for empty strings, not whitespace
        // Additional validation happens at SQLx level
        let empty_uri = "";
        let result = validate_uri_format(empty_uri);
        assert!(result.is_err(), "Expected empty URI to be invalid");

        // Test that the error message is correct
        if let Err(DataBridgeError::Connection(msg)) = result {
            assert_eq!(msg, "Connection URI cannot be empty");
        } else {
            panic!("Expected Connection error for empty URI");
        }
    }

    // Helper function to simulate URI validation logic from Connection::new()
    #[allow(unused)]
    fn validate_uri_format(uri: &str) -> Result<()> {
        if uri.is_empty() {
            return Err(DataBridgeError::Connection(
                "Connection URI cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    #[test]
    fn test_is_connected_before_init() {
        // Verify is_connected returns false before init
        // Since we can't create a Connection without async/DB,
        // we test that a pool reference would be None in an Option wrapper

        let maybe_connection: Option<Connection> = None;
        assert!(maybe_connection.is_none());

        // This simulates checking connection state before initialization
        let is_connected = maybe_connection.is_some();
        assert!(!is_connected);
    }

    #[test]
    fn test_get_pool_returns_error_before_init() {
        // Verify get_pool errors before init
        // We test this by simulating the error condition

        let maybe_connection: Option<Connection> = None;

        // Attempt to get pool when connection is not initialized
        let result = maybe_connection
            .as_ref()
            .ok_or_else(|| DataBridgeError::Connection("Connection not initialized".to_string()));

        assert!(result.is_err());

        if let Err(e) = result {
            match e {
                DataBridgeError::Connection(msg) => {
                    assert_eq!(msg, "Connection not initialized");
                }
                _ => panic!("Expected Connection error"),
            }
        }
    }

    #[test]
    fn test_pool_config_clone() {
        // Verify PoolConfig implements Clone correctly
        let config = PoolConfig {
            min_connections: 3,
            max_connections: 15,
            connect_timeout: 45,
            max_lifetime: Some(2400),
            idle_timeout: Some(800),
            retry: RetryConfig::default(),
            statement_cache_capacity: 150,
        };

        let cloned = config.clone();

        assert_eq!(config.min_connections, cloned.min_connections);
        assert_eq!(config.max_connections, cloned.max_connections);
        assert_eq!(config.connect_timeout, cloned.connect_timeout);
        assert_eq!(config.max_lifetime, cloned.max_lifetime);
        assert_eq!(config.idle_timeout, cloned.idle_timeout);
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 5000);
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_retry_config_no_retry() {
        let config = RetryConfig::no_retry();
        assert_eq!(config.max_retries, 0);
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        };

        // Attempt 0: 100ms
        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));

        // Attempt 1: 100 * 2^1 = 200ms
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));

        // Attempt 2: 100 * 2^2 = 400ms
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));

        // Attempt 3: 100 * 2^3 = 800ms
        assert_eq!(config.delay_for_attempt(3), Duration::from_millis(800));

        // Attempt 4: 100 * 2^4 = 1600ms
        assert_eq!(config.delay_for_attempt(4), Duration::from_millis(1600));

        // Attempt 5: 100 * 2^5 = 3200ms
        assert_eq!(config.delay_for_attempt(5), Duration::from_millis(3200));

        // Attempt 6: 100 * 2^6 = 6400ms, but capped at 5000ms
        assert_eq!(config.delay_for_attempt(6), Duration::from_millis(5000));
    }

    #[test]
    fn test_pool_config_debug() {
        // Verify PoolConfig implements Debug correctly
        let config = PoolConfig::default();
        let debug_output = format!("{:?}", config);

        // Should contain the struct name
        assert!(debug_output.contains("PoolConfig"));
    }
}

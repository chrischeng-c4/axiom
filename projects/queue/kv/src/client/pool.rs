//! Connection pooling for KV client
//!
//! Provides a thread-safe connection pool with:
//! - Min/max connection limits
//! - Idle connection timeout
//! - Automatic connection recycling
//! - RAII guard for automatic return to pool

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use super::client::{KvClient, ClientError};

/// Pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Server address (host:port or host:port/namespace)
    pub addr: String,
    /// Minimum connections to keep alive
    pub min_size: usize,
    /// Maximum connections allowed
    pub max_size: usize,
    /// Connection idle timeout (close if unused for this long)
    pub idle_timeout: Duration,
    /// Timeout for acquiring a connection from pool
    pub acquire_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:7117".to_string(),
            min_size: 2,
            max_size: 10,
            idle_timeout: Duration::from_secs(300),  // 5 min
            acquire_timeout: Duration::from_secs(5),
        }
    }
}

impl PoolConfig {
    /// Create a new pool config with the given address
    pub fn new(addr: impl Into<String>) -> Self {
        Self {
            addr: addr.into(),
            ..Default::default()
        }
    }

    /// Set minimum pool size
    pub fn min_size(mut self, size: usize) -> Self {
        self.min_size = size;
        self
    }

    /// Set maximum pool size
    pub fn max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }

    /// Set idle timeout
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Set acquire timeout
    pub fn acquire_timeout(mut self, timeout: Duration) -> Self {
        self.acquire_timeout = timeout;
        self
    }
}

/// Pooled connection entry with metadata
struct PooledEntry {
    client: KvClient,
    _created_at: Instant,
    last_used: Instant,
}

/// Connection pool for KvClient
///
/// Use `Arc<KvPool>` for sharing across threads/tasks.
///
/// # Example
///
/// ```no_run
/// use cclab_kv::client::{KvPool, PoolConfig, KvValue};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = KvPool::connect(
///     PoolConfig::new("127.0.0.1:7117/cache")
///         .min_size(2)
///         .max_size(10)
/// ).await?;
///
/// {
///     let mut conn = pool.acquire().await?;
///     conn.client().set("key", KvValue::String("value".to_string()), None).await?;
///     let value = conn.client().get("key").await?;
///     println!("Value: {:?}", value);
/// } // Connection automatically returned to pool here
///
/// Ok(())
/// # }
/// ```
pub struct KvPool {
    config: PoolConfig,
    /// Available (idle) connections
    idle: Mutex<VecDeque<PooledEntry>>,
    /// Count of active (in-use) connections
    active_count: Mutex<usize>,
}

impl KvPool {
    /// Create a new pool with the given config (without pre-warming)
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            idle: Mutex::new(VecDeque::new()),
            active_count: Mutex::new(0),
        }
    }

    /// Create pool and pre-warm with min_size connections
    pub async fn connect(config: PoolConfig) -> Result<Arc<Self>, ClientError> {
        let pool = Arc::new(Self::new(config.clone()));

        // Pre-warm pool with min_size connections
        for _ in 0..config.min_size {
            let client = KvClient::connect(&config.addr).await?;
            let mut idle = pool.idle.lock().await;
            idle.push_back(PooledEntry {
                client,
                _created_at: Instant::now(),
                last_used: Instant::now(),
            });
        }

        Ok(pool)
    }

    /// Get namespace from config addr
    pub fn namespace(&self) -> Option<&str> {
        self.config.addr.find('/').map(|idx| &self.config.addr[idx + 1..])
    }

    /// Get a connection from the pool
    ///
    /// This method will:
    /// 1. Try to reuse an idle connection
    /// 2. Create a new connection if pool is not at max_size
    /// 3. Wait and retry if pool is at max_size
    /// 4. Return `Err(ClientError::Timeout)` if acquire_timeout is exceeded
    pub async fn acquire(self: &Arc<Self>) -> Result<PooledClient, ClientError> {
        let deadline = Instant::now() + self.config.acquire_timeout;

        loop {
            // Try to get an idle connection
            {
                let mut idle = self.idle.lock().await;

                // Remove expired connections
                let now = Instant::now();
                while let Some(entry) = idle.front() {
                    if now.duration_since(entry.last_used) > self.config.idle_timeout {
                        idle.pop_front();
                    } else {
                        break;
                    }
                }

                // Get an idle connection if available
                if let Some(entry) = idle.pop_back() {
                    let mut active = self.active_count.lock().await;
                    *active += 1;
                    return Ok(PooledClient {
                        client: Some(entry.client),
                        pool: Arc::clone(self),
                    });
                }
            }

            // Check if we can create a new connection
            {
                let active = self.active_count.lock().await;
                let idle = self.idle.lock().await;
                if *active + idle.len() < self.config.max_size {
                    drop(idle);
                    drop(active);

                    // Create new connection
                    let client = KvClient::connect(&self.config.addr).await?;
                    let mut active = self.active_count.lock().await;
                    *active += 1;
                    return Ok(PooledClient {
                        client: Some(client),
                        pool: Arc::clone(self),
                    });
                }
            }

            // Check timeout
            if Instant::now() > deadline {
                return Err(ClientError::Timeout);
            }

            // Wait a bit and retry
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Return a connection to the pool
    async fn release(&self, client: KvClient) {
        let mut active = self.active_count.lock().await;
        *active = active.saturating_sub(1);
        drop(active);

        let mut idle = self.idle.lock().await;
        if idle.len() < self.config.max_size {
            idle.push_back(PooledEntry {
                client,
                _created_at: Instant::now(),
                last_used: Instant::now(),
            });
        }
        // else: drop the connection (pool is full)
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let idle = self.idle.lock().await;
        let active = self.active_count.lock().await;
        PoolStats {
            idle: idle.len(),
            active: *active,
            max_size: self.config.max_size,
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub idle: usize,
    pub active: usize,
    pub max_size: usize,
}

/// RAII guard for pooled connection
///
/// The connection is automatically returned to the pool when dropped.
pub struct PooledClient {
    client: Option<KvClient>,
    pool: Arc<KvPool>,
}

impl PooledClient {
    /// Get mutable reference to the underlying client
    pub fn client(&mut self) -> &mut KvClient {
        self.client.as_mut().expect("client already returned")
    }
}

impl Drop for PooledClient {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            let pool = Arc::clone(&self.pool);
            // Spawn task to return connection (we can't await in drop)
            tokio::spawn(async move {
                pool.release(client).await;
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_builder() {
        let config = PoolConfig::new("127.0.0.1:7117")
            .min_size(5)
            .max_size(20)
            .idle_timeout(Duration::from_secs(600))
            .acquire_timeout(Duration::from_secs(10));

        assert_eq!(config.addr, "127.0.0.1:7117");
        assert_eq!(config.min_size, 5);
        assert_eq!(config.max_size, 20);
        assert_eq!(config.idle_timeout, Duration::from_secs(600));
        assert_eq!(config.acquire_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_namespace_extraction() {
        let pool = KvPool::new(PoolConfig::new("127.0.0.1:7117/cache"));
        assert_eq!(pool.namespace(), Some("cache"));

        let pool = KvPool::new(PoolConfig::new("127.0.0.1:7117"));
        assert_eq!(pool.namespace(), None);
    }

    // Integration tests require a running server
    // Run: cargo run -p cclab-kv-server
    // Then: cargo test -p cclab-kv-client -- --ignored

    #[tokio::test]
    #[ignore]
    async fn test_pool_basic() {
        let pool = KvPool::connect(
            PoolConfig::new("127.0.0.1:7117")
                .min_size(2)
                .max_size(5)
        ).await.unwrap();

        // Check initial stats
        let stats = pool.stats().await;
        assert_eq!(stats.idle, 2);  // Pre-warmed with min_size
        assert_eq!(stats.active, 0);

        // Acquire a connection
        let mut conn = pool.acquire().await.unwrap();
        let stats = pool.stats().await;
        assert_eq!(stats.idle, 1);
        assert_eq!(stats.active, 1);

        // Use the connection
        conn.client().ping().await.unwrap();

        // Drop connection (returned to pool)
        drop(conn);
        tokio::time::sleep(Duration::from_millis(50)).await;  // Give async return time

        let stats = pool.stats().await;
        assert_eq!(stats.idle, 2);
        assert_eq!(stats.active, 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_max_size() {
        let pool = KvPool::connect(
            PoolConfig::new("127.0.0.1:7117")
                .min_size(1)
                .max_size(3)
        ).await.unwrap();

        // Acquire all connections
        let _c1 = pool.acquire().await.unwrap();
        let _c2 = pool.acquire().await.unwrap();
        let _c3 = pool.acquire().await.unwrap();

        let stats = pool.stats().await;
        assert_eq!(stats.active, 3);

        // Try to acquire 4th connection (should timeout)
        let result = pool.acquire().await;
        assert!(matches!(result, Err(ClientError::Timeout)));
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_concurrent() {
        let pool = KvPool::connect(
            PoolConfig::new("127.0.0.1:7117")
                .min_size(2)
                .max_size(10)
        ).await.unwrap();

        let mut handles = vec![];

        for i in 0..20 {
            let pool = Arc::clone(&pool);
            let handle = tokio::spawn(async move {
                let mut conn = pool.acquire().await.unwrap();
                conn.client().set(
                    &format!("key_{}", i),
                    crate::KvValue::Int(i as i64),
                    None
                ).await.unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // All connections should be returned
        tokio::time::sleep(Duration::from_millis(100)).await;
        let stats = pool.stats().await;
        assert_eq!(stats.active, 0);
    }
}

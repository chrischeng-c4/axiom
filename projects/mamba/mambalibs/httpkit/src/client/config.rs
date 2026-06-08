//! HTTP client configuration

use std::time::Duration;

/// Configuration for HTTP client
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Base URL for all requests (e.g., "https://api.example.com")
    pub base_url: Option<String>,

    /// Total request timeout
    pub timeout: Duration,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// Read timeout (time to first byte after connection)
    pub read_timeout: Option<Duration>,

    /// Maximum idle connections per host
    pub pool_max_idle_per_host: usize,

    /// Idle connection timeout
    pub pool_idle_timeout: Duration,

    /// Whether to follow redirects
    pub follow_redirects: bool,

    /// Maximum number of redirects to follow
    pub max_redirects: usize,

    /// User-Agent header value
    pub user_agent: String,

    /// Whether to accept invalid certificates (for testing only)
    pub danger_accept_invalid_certs: bool,

    /// Whether to accept invalid hostnames (for testing only)
    pub danger_accept_invalid_hostnames: bool,

    /// Enable gzip compression
    pub gzip: bool,

    /// Enable brotli compression
    pub brotli: bool,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            read_timeout: None,
            pool_max_idle_per_host: 10,
            pool_idle_timeout: Duration::from_secs(90),
            follow_redirects: true,
            max_redirects: 10,
            user_agent: format!("mambalibs-http/{}", env!("CARGO_PKG_VERSION")),
            danger_accept_invalid_certs: false,
            danger_accept_invalid_hostnames: false,
            gzip: true,
            brotli: true,
        }
    }
}

impl HttpClientConfig {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base URL
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set the total timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set timeout from seconds (convenience method for Python)
    pub fn timeout_secs(mut self, secs: f64) -> Self {
        self.timeout = Duration::from_secs_f64(secs);
        self
    }

    /// Set the connection timeout
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set connection timeout from seconds
    pub fn connect_timeout_secs(mut self, secs: f64) -> Self {
        self.connect_timeout = Duration::from_secs_f64(secs);
        self
    }

    /// Set the read timeout
    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = Some(timeout);
        self
    }

    /// Set max idle connections per host
    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.pool_max_idle_per_host = max;
        self
    }

    /// Set idle connection timeout
    pub fn pool_idle_timeout(mut self, timeout: Duration) -> Self {
        self.pool_idle_timeout = timeout;
        self
    }

    /// Set whether to follow redirects
    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    /// Set maximum redirects
    pub fn max_redirects(mut self, max: usize) -> Self {
        self.max_redirects = max;
        self
    }

    /// Set the User-Agent header
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Accept invalid certificates (DANGER - testing only)
    pub fn danger_accept_invalid_certs(mut self, accept: bool) -> Self {
        self.danger_accept_invalid_certs = accept;
        self
    }

    /// Enable/disable gzip compression
    pub fn gzip(mut self, enabled: bool) -> Self {
        self.gzip = enabled;
        self
    }

    /// Enable/disable brotli compression
    pub fn brotli(mut self, enabled: bool) -> Self {
        self.brotli = enabled;
        self
    }
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,

    /// Initial delay between retries
    pub initial_delay: Duration,

    /// Maximum delay between retries
    pub max_delay: Duration,

    /// Backoff strategy
    pub backoff: BackoffStrategy,

    /// HTTP status codes to retry on
    pub retry_on_status: Vec<u16>,

    /// Whether to retry on timeout
    pub retry_on_timeout: bool,

    /// Whether to retry on connection error
    pub retry_on_connection_error: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff: BackoffStrategy::Exponential,
            retry_on_status: vec![500, 502, 503, 504, 429],
            retry_on_timeout: true,
            retry_on_connection_error: true,
        }
    }
}

/// Backoff strategy for retries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackoffStrategy {
    /// Constant delay between retries
    Constant,
    /// Linear increase (delay * attempt)
    Linear,
    /// Exponential increase (delay * 2^attempt)
    Exponential,
}

impl BackoffStrategy {
    /// Calculate delay for given attempt number
    pub fn delay(&self, base: Duration, attempt: u32, max: Duration) -> Duration {
        let delay = match self {
            BackoffStrategy::Constant => base,
            BackoffStrategy::Linear => base * attempt,
            BackoffStrategy::Exponential => base * 2u32.saturating_pow(attempt),
        };
        std::cmp::min(delay, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HttpClientConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.follow_redirects);
        assert_eq!(config.max_redirects, 10);
    }

    #[test]
    fn test_builder_pattern() {
        let config = HttpClientConfig::new()
            .base_url("https://api.example.com")
            .timeout_secs(60.0)
            .pool_max_idle_per_host(20);

        assert_eq!(config.base_url, Some("https://api.example.com".to_string()));
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.pool_max_idle_per_host, 20);
    }

    #[test]
    fn test_exponential_backoff() {
        let base = Duration::from_millis(100);
        let max = Duration::from_secs(10);

        assert_eq!(
            BackoffStrategy::Exponential.delay(base, 0, max),
            Duration::from_millis(100)
        );
        assert_eq!(
            BackoffStrategy::Exponential.delay(base, 1, max),
            Duration::from_millis(200)
        );
        assert_eq!(
            BackoffStrategy::Exponential.delay(base, 2, max),
            Duration::from_millis(400)
        );
        assert_eq!(BackoffStrategy::Exponential.delay(base, 10, max), max); // Capped at max
    }
}

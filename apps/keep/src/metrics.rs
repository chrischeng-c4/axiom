//! Metrics collection for the KV engine
//!
//! Provides observability through counters and histograms for monitoring
//! key operations and performance characteristics.
//!
//! # Usage
//!
//! Enable metrics by creating a `Metrics` instance and passing it to the engine:
//!
//! ```
//! use keep::metrics::Metrics;
//!
//! let metrics = Metrics::new();
//! // Use metrics.record_*() to track operations
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Operation type for metrics tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    Get,
    Set,
    Delete,
    Exists,
    Incr,
    Decr,
    Cas,
    MGet,
    MSet,
    MDel,
    Scan,
    Lock,
    Unlock,
}

impl Operation {
    /// Get the operation name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Operation::Get => "get",
            Operation::Set => "set",
            Operation::Delete => "delete",
            Operation::Exists => "exists",
            Operation::Incr => "incr",
            Operation::Decr => "decr",
            Operation::Cas => "cas",
            Operation::MGet => "mget",
            Operation::MSet => "mset",
            Operation::MDel => "mdel",
            Operation::Scan => "scan",
            Operation::Lock => "lock",
            Operation::Unlock => "unlock",
        }
    }
}

/// Atomic counter for tracking operation counts
#[derive(Debug, Default)]
pub struct Counter {
    value: AtomicU64,
}

impl Counter {
    /// Create a new counter initialized to 0
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    /// Increment the counter by 1
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the counter by a specific amount
    pub fn inc_by(&self, n: u64) {
        self.value.fetch_add(n, Ordering::Relaxed);
    }

    /// Get the current counter value
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// Simple histogram using buckets for latency tracking
#[derive(Debug)]
pub struct Histogram {
    /// Bucket boundaries in microseconds
    buckets: Vec<u64>,
    /// Counts for each bucket
    counts: Vec<AtomicU64>,
    /// Sum of all observed values (in microseconds)
    sum: AtomicU64,
    /// Total count of observations
    count: AtomicU64,
}

impl Histogram {
    /// Create a new histogram with default latency buckets
    ///
    /// Default buckets: 10us, 50us, 100us, 250us, 500us, 1ms, 5ms, 10ms, 50ms, 100ms
    pub fn new() -> Self {
        Self::with_buckets(vec![
            10, 50, 100, 250, 500, 1000, 5000, 10000, 50000, 100000,
        ])
    }

    /// Create a new histogram with custom bucket boundaries (in microseconds)
    pub fn with_buckets(buckets: Vec<u64>) -> Self {
        let counts = buckets.iter().map(|_| AtomicU64::new(0)).collect();
        Self {
            buckets,
            counts,
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }

    /// Record a value in microseconds
    pub fn observe(&self, value_us: u64) {
        self.sum.fetch_add(value_us, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);

        for (i, &boundary) in self.buckets.iter().enumerate() {
            if value_us <= boundary {
                self.counts[i].fetch_add(1, Ordering::Relaxed);
                break;
            }
        }
    }

    /// Record a duration
    pub fn observe_duration(&self, start: Instant) {
        let elapsed = start.elapsed();
        self.observe(elapsed.as_micros() as u64);
    }

    /// Get the total count of observations
    pub fn get_count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get the sum of all observations (in microseconds)
    pub fn get_sum(&self) -> u64 {
        self.sum.load(Ordering::Relaxed)
    }

    /// Get the mean value (in microseconds)
    pub fn get_mean(&self) -> f64 {
        let count = self.get_count();
        if count == 0 {
            0.0
        } else {
            self.get_sum() as f64 / count as f64
        }
    }

    /// Get bucket counts as a vector of (boundary, count) pairs
    pub fn get_buckets(&self) -> Vec<(u64, u64)> {
        self.buckets
            .iter()
            .zip(self.counts.iter())
            .map(|(&boundary, count)| (boundary, count.load(Ordering::Relaxed)))
            .collect()
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics collection for the KV engine
#[derive(Debug)]
pub struct Metrics {
    // Operation counters
    pub get_total: Counter,
    pub set_total: Counter,
    pub delete_total: Counter,
    pub exists_total: Counter,
    pub incr_total: Counter,
    pub cas_total: Counter,
    pub scan_total: Counter,
    pub lock_total: Counter,

    // Error counters
    pub errors_total: Counter,
    pub key_not_found_total: Counter,

    // Latency histograms
    pub get_latency: Histogram,
    pub set_latency: Histogram,
    pub delete_latency: Histogram,

    // Connection metrics
    pub active_connections: Counter,
    pub total_connections: Counter,

    // Size metrics
    pub keys_total: AtomicU64,
    pub memory_bytes: AtomicU64,
}

impl Metrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self {
            get_total: Counter::new(),
            set_total: Counter::new(),
            delete_total: Counter::new(),
            exists_total: Counter::new(),
            incr_total: Counter::new(),
            cas_total: Counter::new(),
            scan_total: Counter::new(),
            lock_total: Counter::new(),
            errors_total: Counter::new(),
            key_not_found_total: Counter::new(),
            get_latency: Histogram::new(),
            set_latency: Histogram::new(),
            delete_latency: Histogram::new(),
            active_connections: Counter::new(),
            total_connections: Counter::new(),
            keys_total: AtomicU64::new(0),
            memory_bytes: AtomicU64::new(0),
        }
    }

    /// Record an operation
    pub fn record_operation(&self, op: Operation) {
        match op {
            Operation::Get => self.get_total.inc(),
            Operation::Set => self.set_total.inc(),
            Operation::Delete => self.delete_total.inc(),
            Operation::Exists => self.exists_total.inc(),
            Operation::Incr | Operation::Decr => self.incr_total.inc(),
            Operation::Cas => self.cas_total.inc(),
            Operation::Scan => self.scan_total.inc(),
            Operation::Lock | Operation::Unlock => self.lock_total.inc(),
            Operation::MGet => self.get_total.inc(),
            Operation::MSet => self.set_total.inc(),
            Operation::MDel => self.delete_total.inc(),
        }
    }

    /// Record an error
    pub fn record_error(&self) {
        self.errors_total.inc();
    }

    /// Record a key not found event
    pub fn record_key_not_found(&self) {
        self.key_not_found_total.inc();
    }

    /// Record operation latency
    pub fn record_latency(&self, op: Operation, start: Instant) {
        match op {
            Operation::Get | Operation::MGet => self.get_latency.observe_duration(start),
            Operation::Set | Operation::MSet => self.set_latency.observe_duration(start),
            Operation::Delete | Operation::MDel => self.delete_latency.observe_duration(start),
            _ => {}
        }
    }

    /// Update key count
    pub fn set_keys_total(&self, count: u64) {
        self.keys_total.store(count, Ordering::Relaxed);
    }

    /// Get summary as JSON string
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"get_total":{},"set_total":{},"delete_total":{},"exists_total":{},"incr_total":{},"cas_total":{},"scan_total":{},"lock_total":{},"errors_total":{},"key_not_found_total":{},"get_latency_mean_us":{:.2},"set_latency_mean_us":{:.2},"keys_total":{},"active_connections":{},"total_connections":{}}}"#,
            self.get_total.get(),
            self.set_total.get(),
            self.delete_total.get(),
            self.exists_total.get(),
            self.incr_total.get(),
            self.cas_total.get(),
            self.scan_total.get(),
            self.lock_total.get(),
            self.errors_total.get(),
            self.key_not_found_total.get(),
            self.get_latency.get_mean(),
            self.set_latency.get_mean(),
            self.keys_total.load(Ordering::Relaxed),
            self.active_connections.get(),
            self.total_connections.get(),
        )
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_counter() {
        let counter = Counter::new();
        assert_eq!(counter.get(), 0);

        counter.inc();
        assert_eq!(counter.get(), 1);

        counter.inc_by(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn test_histogram() {
        let hist = Histogram::new();

        hist.observe(50); // 50us - fits in 50us bucket
        hist.observe(150); // 150us - fits in 250us bucket
        hist.observe(5000); // 5ms - fits in 5000us bucket

        assert_eq!(hist.get_count(), 3);
        assert_eq!(hist.get_sum(), 50 + 150 + 5000);
    }

    #[test]
    fn test_histogram_duration() {
        let hist = Histogram::new();
        let start = Instant::now();
        thread::sleep(Duration::from_micros(100));
        hist.observe_duration(start);

        assert_eq!(hist.get_count(), 1);
        assert!(hist.get_sum() >= 100); // At least 100us
    }

    #[test]
    fn test_metrics_operations() {
        let metrics = Metrics::new();

        metrics.record_operation(Operation::Get);
        metrics.record_operation(Operation::Get);
        metrics.record_operation(Operation::Set);

        assert_eq!(metrics.get_total.get(), 2);
        assert_eq!(metrics.set_total.get(), 1);
    }

    #[test]
    fn test_metrics_json() {
        let metrics = Metrics::new();
        metrics.record_operation(Operation::Get);
        metrics.record_operation(Operation::Set);

        let json = metrics.to_json();
        assert!(json.contains(r#""get_total":1"#));
        assert!(json.contains(r#""set_total":1"#));
    }
}

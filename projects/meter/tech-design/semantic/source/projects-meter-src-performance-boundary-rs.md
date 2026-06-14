---
id: projects-meter-src-performance-boundary-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: runtime-resource-attribution
    role: primary
    gap: embedded-profiler-api
    claim: embedded-profiler-api
    coverage: full
    rationale: "Source template implements meter performance measurement and regression reporting surfaces."
---

# Standardized projects/meter/src/performance/boundary.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/performance/boundary.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `BoundaryMetrics` | projects/meter/src/performance/boundary.rs | struct | pub | 268 |  |
| `BoundaryTiming` | projects/meter/src/performance/boundary.rs | struct | pub | 65 |  |
| `BoundaryTracer` | projects/meter/src/performance/boundary.rs | struct | pub | 143 |  |
| `avg_convert_us` | projects/meter/src/performance/boundary.rs | function | pub | 381 | avg_convert_us(&self) -> f64 |
| `avg_extract_us` | projects/meter/src/performance/boundary.rs | function | pub | 371 | avg_extract_us(&self) -> f64 |
| `avg_materialize_us` | projects/meter/src/performance/boundary.rs | function | pub | 401 | avg_materialize_us(&self) -> f64 |
| `avg_network_us` | projects/meter/src/performance/boundary.rs | function | pub | 391 | avg_network_us(&self) -> f64 |
| `doc_count` | projects/meter/src/performance/boundary.rs | function | pub | 361 | doc_count(&self) -> u64 |
| `end_convert` | projects/meter/src/performance/boundary.rs | function | pub | 198 | end_convert(&mut self) |
| `end_extract` | projects/meter/src/performance/boundary.rs | function | pub | 186 | end_extract(&mut self) |
| `end_materialize` | projects/meter/src/performance/boundary.rs | function | pub | 222 | end_materialize(&mut self) |
| `end_network` | projects/meter/src/performance/boundary.rs | function | pub | 210 | end_network(&mut self) |
| `finish` | projects/meter/src/performance/boundary.rs | function | pub | 244 | finish(self) -> BoundaryTiming |
| `format` | projects/meter/src/performance/boundary.rs | function | pub | 117 | format(&self) -> String |
| `gil_held_percent` | projects/meter/src/performance/boundary.rs | function | pub | 99 | gil_held_percent(&self) -> f64 |
| `gil_held_us` | projects/meter/src/performance/boundary.rs | function | pub | 89 | gil_held_us(&self) -> u64 |
| `gil_release_count` | projects/meter/src/performance/boundary.rs | function | pub | 366 | gil_release_count(&self) -> u64 |
| `gil_released_us` | projects/meter/src/performance/boundary.rs | function | pub | 94 | gil_released_us(&self) -> u64 |
| `new` | projects/meter/src/performance/boundary.rs | function | pub | 162 | new(operation: impl Into<String>) -> Self |
| `new` | projects/meter/src/performance/boundary.rs | function | pub | 281 | new() -> Self |
| `operation_count` | projects/meter/src/performance/boundary.rs | function | pub | 356 | operation_count(&self) -> u64 |
| `per_doc_us` | projects/meter/src/performance/boundary.rs | function | pub | 108 | per_doc_us(&self) -> f64 |
| `record` | projects/meter/src/performance/boundary.rs | function | pub | 294 | record(&self, timing: &BoundaryTiming) |
| `record_gil_release` | projects/meter/src/performance/boundary.rs | function | pub | 234 | record_gil_release(&mut self) |
| `reset` | projects/meter/src/performance/boundary.rs | function | pub | 345 | reset(&self) |
| `set_doc_count` | projects/meter/src/performance/boundary.rs | function | pub | 229 | set_doc_count(&mut self, count: usize) |
| `set_parallel` | projects/meter/src/performance/boundary.rs | function | pub | 239 | set_parallel(&mut self, parallel: bool) |
| `snapshot` | projects/meter/src/performance/boundary.rs | function | pub | 311 | snapshot(&self) -> HashMap<String, u64> |
| `start_convert` | projects/meter/src/performance/boundary.rs | function | pub | 193 | start_convert(&mut self) |
| `start_extract` | projects/meter/src/performance/boundary.rs | function | pub | 181 | start_extract(&mut self) |
| `start_materialize` | projects/meter/src/performance/boundary.rs | function | pub | 217 | start_materialize(&mut self) |
| `start_network` | projects/meter/src/performance/boundary.rs | function | pub | 205 | start_network(&mut self) |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! binding boundary tracing infrastructure
//!
//! Provides detailed performance insights into data movement across the Python/Rust boundary.
//!
//! # Architecture
//!
//! Every binding operation follows a four-phase pattern:
//!
//! 1. **Extract** (GIL held): Extract Python objects to intermediate representation
//! 2. **Convert** (GIL released): Convert intermediate to BSON/native Rust types
//! 3. **Network** (GIL released): Async I/O operations with MongoDB
//! 4. **Materialize** (GIL held): Create Python objects from Rust data
//!
//! # Example
//!
//! ```rust
//! use meter::performance::boundary::BoundaryTracer;
//!
//! let mut tracer = BoundaryTracer::new("insert_many");
//!
//! // Phase 1: Extract Python data
//! tracer.start_extract();
//! // ... extract Python objects
//! tracer.end_extract();
//!
//! // Phase 2: Convert to BSON
//! tracer.start_convert();
//! tracer.record_gil_release();  // Mark GIL release
//! // ... convert to BSON
//! tracer.end_convert();
//!
//! // Phase 3: Network I/O
//! tracer.start_network();
//! // ... MongoDB operations
//! tracer.end_network();
//!
//! // Phase 4: Materialize results
//! tracer.start_materialize();
//! // ... create Python objects
//! tracer.end_materialize();
//!
//! tracer.set_doc_count(1000);
//! let timing = tracer.finish();
//!
//! println!("Extract: {}µs, Convert: {}µs", timing.extract_us, timing.convert_us);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ============================================================================
// Phase-level Timing Types
// ============================================================================

/// Phase-level timing breakdown for binding boundary crossing
///
/// Records detailed timing information for each phase of a binding operation,
/// enabling identification of bottlenecks and GIL contention issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-boundary-rs.md#source
pub struct BoundaryTiming {
    /// Operation name (e.g., "insert_many", "find")
    pub operation: String,
    /// Python extraction time in microseconds (GIL held)
    pub extract_us: u64,
    /// Rust conversion time in microseconds (GIL released)
    pub convert_us: u64,
    /// Network I/O time in microseconds (GIL released)
    pub network_us: u64,
    /// Python object materialization time in microseconds (GIL held)
    pub materialize_us: u64,
    /// Total operation time in microseconds
    pub total_us: u64,
    /// Number of documents processed
    pub doc_count: usize,
    /// Number of times GIL was released
    pub gil_release_count: usize,
    /// Whether Rayon parallelization was used
    pub parallel: bool,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-boundary-rs.md#source
impl BoundaryTiming {
    /// Get total GIL-held time in microseconds
    pub fn gil_held_us(&self) -> u64 {
        self.extract_us + self.materialize_us
    }

    /// Get total GIL-released time in microseconds
    pub fn gil_released_us(&self) -> u64 {
        self.convert_us + self.network_us
    }

    /// Get percentage of time spent with GIL held
    pub fn gil_held_percent(&self) -> f64 {
        if self.total_us > 0 {
            (self.gil_held_us() as f64 / self.total_us as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get average time per document in microseconds
    pub fn per_doc_us(&self) -> f64 {
        if self.doc_count > 0 {
            self.total_us as f64 / self.doc_count as f64
        } else {
            0.0
        }
    }

    /// Format as human-readable string
    pub fn format(&self) -> String {
        format!(
            "{} ({} docs, {:.2}µs/doc):\n  Extract: {}µs (GIL held)\n  Convert: {}µs (GIL released{})\n  Network: {}µs (GIL released)\n  Materialize: {}µs (GIL held)\n  Total: {}µs (GIL held {:.1}%)",
            self.operation,
            self.doc_count,
            self.per_doc_us(),
            self.extract_us,
            self.convert_us,
            if self.parallel { ", parallel" } else { "" },
            self.network_us,
            self.materialize_us,
            self.total_us,
            self.gil_held_percent()
        )
    }
}

// ============================================================================
// Boundary Tracer
// ============================================================================

/// Lightweight tracer for binding boundary operations
///
/// Tracks timing for each phase of a binding operation with minimal overhead.
/// Uses `Instant` for high-resolution timing.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-boundary-rs.md#source
pub struct BoundaryTracer {
    operation: String,
    start: Instant,
    extract_start: Option<Instant>,
    convert_start: Option<Instant>,
    network_start: Option<Instant>,
    materialize_start: Option<Instant>,
    extract_duration: u64,
    convert_duration: u64,
    network_duration: u64,
    materialize_duration: u64,
    doc_count: usize,
    gil_releases: usize,
    parallel: bool,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-boundary-rs.md#source
impl BoundaryTracer {
    /// Create a new boundary tracer for the given operation
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            start: Instant::now(),
            extract_start: None,
            convert_start: None,
            network_start: None,
            materialize_start: None,
            extract_duration: 0,
            convert_duration: 0,
            network_duration: 0,
            materialize_duration: 0,
            doc_count: 0,
            gil_releases: 0,
            parallel: false,
        }
    }

    /// Start timing the extract phase (GIL held)
    pub fn start_extract(&mut self) {
        self.extract_start = Some(Instant::now());
    }

    /// End timing the extract phase
    pub fn end_extract(&mut self) {
        if let Some(start) = self.extract_start.take() {
            self.extract_duration = start.elapsed().as_micros() as u64;
        }
    }

    /// Start timing the convert phase (GIL released)
    pub fn start_convert(&mut self) {
        self.convert_start = Some(Instant::now());
    }

    /// End timing the convert phase
    pub fn end_convert(&mut self) {
        if let Some(start) = self.convert_start.take() {
            self.convert_duration = start.elapsed().as_micros() as u64;
        }
    }

    /// Start timing the network phase (GIL released)
    pub fn start_network(&mut self) {
        self.network_start = Some(Instant::now());
    }

    /// End timing the network phase
    pub fn end_network(&mut self) {
        if let Some(start) = self.network_start.take() {
            self.network_duration = start.elapsed().as_micros() as u64;
        }
    }

    /// Start timing the materialize phase (GIL held)
    pub fn start_materialize(&mut self) {
        self.materialize_start = Some(Instant::now());
    }

    /// End timing the materialize phase
    pub fn end_materialize(&mut self) {
        if let Some(start) = self.materialize_start.take() {
            self.materialize_duration = start.elapsed().as_micros() as u64;
        }
    }

    /// Set the number of documents processed
    pub fn set_doc_count(&mut self, count: usize) {
        self.doc_count = count;
    }

    /// Record a GIL release event
    pub fn record_gil_release(&mut self) {
        self.gil_releases += 1;
    }

    /// Set whether parallel processing was used
    pub fn set_parallel(&mut self, parallel: bool) {
        self.parallel = parallel;
    }

    /// Finish tracing and return timing breakdown
    pub fn finish(self) -> BoundaryTiming {
        BoundaryTiming {
            operation: self.operation,
            extract_us: self.extract_duration,
            convert_us: self.convert_duration,
            network_us: self.network_duration,
            materialize_us: self.materialize_duration,
            total_us: self.start.elapsed().as_micros() as u64,
            doc_count: self.doc_count,
            gil_release_count: self.gil_releases,
            parallel: self.parallel,
        }
    }
}

// ============================================================================
// Global Metrics Collector
// ============================================================================

/// Global metrics collector for boundary operations (thread-safe)
///
/// Aggregates timing data across multiple operations using atomic operations
/// for lock-free updates.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-boundary-rs.md#source
pub struct BoundaryMetrics {
    total_operations: AtomicU64,
    total_extract_us: AtomicU64,
    total_convert_us: AtomicU64,
    total_network_us: AtomicU64,
    total_materialize_us: AtomicU64,
    total_docs: AtomicU64,
    total_gil_releases: AtomicU64,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-boundary-rs.md#source
impl BoundaryMetrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            total_operations: AtomicU64::new(0),
            total_extract_us: AtomicU64::new(0),
            total_convert_us: AtomicU64::new(0),
            total_network_us: AtomicU64::new(0),
            total_materialize_us: AtomicU64::new(0),
            total_docs: AtomicU64::new(0),
            total_gil_releases: AtomicU64::new(0),
        }
    }

    /// Record a boundary timing
    pub fn record(&self, timing: &BoundaryTiming) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        self.total_extract_us
            .fetch_add(timing.extract_us, Ordering::Relaxed);
        self.total_convert_us
            .fetch_add(timing.convert_us, Ordering::Relaxed);
        self.total_network_us
            .fetch_add(timing.network_us, Ordering::Relaxed);
        self.total_materialize_us
            .fetch_add(timing.materialize_us, Ordering::Relaxed);
        self.total_docs
            .fetch_add(timing.doc_count as u64, Ordering::Relaxed);
        self.total_gil_releases
            .fetch_add(timing.gil_release_count as u64, Ordering::Relaxed);
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> HashMap<String, u64> {
        let mut map = HashMap::new();
        map.insert(
            "total_operations".to_string(),
            self.total_operations.load(Ordering::Relaxed),
        );
        map.insert(
            "total_extract_us".to_string(),
            self.total_extract_us.load(Ordering::Relaxed),
        );
        map.insert(
            "total_convert_us".to_string(),
            self.total_convert_us.load(Ordering::Relaxed),
        );
        map.insert(
            "total_network_us".to_string(),
            self.total_network_us.load(Ordering::Relaxed),
        );
        map.insert(
            "total_materialize_us".to_string(),
            self.total_materialize_us.load(Ordering::Relaxed),
        );
        map.insert(
            "total_docs".to_string(),
            self.total_docs.load(Ordering::Relaxed),
        );
        map.insert(
            "total_gil_releases".to_string(),
            self.total_gil_releases.load(Ordering::Relaxed),
        );
        map
    }

    /// Reset all metrics to zero
    pub fn reset(&self) {
        self.total_operations.store(0, Ordering::Relaxed);
        self.total_extract_us.store(0, Ordering::Relaxed);
        self.total_convert_us.store(0, Ordering::Relaxed);
        self.total_network_us.store(0, Ordering::Relaxed);
        self.total_materialize_us.store(0, Ordering::Relaxed);
        self.total_docs.store(0, Ordering::Relaxed);
        self.total_gil_releases.store(0, Ordering::Relaxed);
    }

    /// Get total operation count
    pub fn operation_count(&self) -> u64 {
        self.total_operations.load(Ordering::Relaxed)
    }

    /// Get total documents processed
    pub fn doc_count(&self) -> u64 {
        self.total_docs.load(Ordering::Relaxed)
    }

    /// Get total GIL releases
    pub fn gil_release_count(&self) -> u64 {
        self.total_gil_releases.load(Ordering::Relaxed)
    }

    /// Get average extract time in microseconds
    pub fn avg_extract_us(&self) -> f64 {
        let ops = self.total_operations.load(Ordering::Relaxed);
        if ops > 0 {
            self.total_extract_us.load(Ordering::Relaxed) as f64 / ops as f64
        } else {
            0.0
        }
    }

    /// Get average convert time in microseconds
    pub fn avg_convert_us(&self) -> f64 {
        let ops = self.total_operations.load(Ordering::Relaxed);
        if ops > 0 {
            self.total_convert_us.load(Ordering::Relaxed) as f64 / ops as f64
        } else {
            0.0
        }
    }

    /// Get average network time in microseconds
    pub fn avg_network_us(&self) -> f64 {
        let ops = self.total_operations.load(Ordering::Relaxed);
        if ops > 0 {
            self.total_network_us.load(Ordering::Relaxed) as f64 / ops as f64
        } else {
            0.0
        }
    }

    /// Get average materialize time in microseconds
    pub fn avg_materialize_us(&self) -> f64 {
        let ops = self.total_operations.load(Ordering::Relaxed);
        if ops > 0 {
            self.total_materialize_us.load(Ordering::Relaxed) as f64 / ops as f64
        } else {
            0.0
        }
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-boundary-rs.md#source
impl Default for BoundaryMetrics {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_boundary_tracer_basic() {
        let mut tracer = BoundaryTracer::new("test_operation");

        tracer.start_extract();
        thread::sleep(std::time::Duration::from_micros(100));
        tracer.end_extract();

        tracer.start_convert();
        thread::sleep(std::time::Duration::from_micros(200));
        tracer.end_convert();

        tracer.start_network();
        thread::sleep(std::time::Duration::from_micros(300));
        tracer.end_network();

        tracer.start_materialize();
        thread::sleep(std::time::Duration::from_micros(50));
        tracer.end_materialize();

        tracer.set_doc_count(10);
        tracer.record_gil_release();
        tracer.set_parallel(true);

        let timing = tracer.finish();

        assert_eq!(timing.operation, "test_operation");
        assert!(timing.extract_us >= 100);
        assert!(timing.convert_us >= 200);
        assert!(timing.network_us >= 300);
        assert!(timing.materialize_us >= 50);
        assert_eq!(timing.doc_count, 10);
        assert_eq!(timing.gil_release_count, 1);
        assert!(timing.parallel);
        assert!(timing.total_us >= 650);
    }

    #[test]
    fn test_boundary_timing_calculations() {
        let timing = BoundaryTiming {
            operation: "test".to_string(),
            extract_us: 100,
            convert_us: 200,
            network_us: 300,
            materialize_us: 50,
            total_us: 650,
            doc_count: 10,
            gil_release_count: 2,
            parallel: false,
        };

        assert_eq!(timing.gil_held_us(), 150); // extract + materialize
        assert_eq!(timing.gil_released_us(), 500); // convert + network
        assert!((timing.gil_held_percent() - 23.08).abs() < 0.1);
        assert!((timing.per_doc_us() - 65.0).abs() < 0.1);
    }

    #[test]
    fn test_boundary_metrics() {
        let metrics = BoundaryMetrics::new();

        let timing1 = BoundaryTiming {
            operation: "test1".to_string(),
            extract_us: 100,
            convert_us: 200,
            network_us: 300,
            materialize_us: 50,
            total_us: 650,
            doc_count: 5,
            gil_release_count: 2,
            parallel: false,
        };

        let timing2 = BoundaryTiming {
            operation: "test2".to_string(),
            extract_us: 200,
            convert_us: 400,
            network_us: 600,
            materialize_us: 100,
            total_us: 1300,
            doc_count: 10,
            gil_release_count: 3,
            parallel: true,
        };

        metrics.record(&timing1);
        metrics.record(&timing2);

        assert_eq!(metrics.operation_count(), 2);
        assert_eq!(metrics.doc_count(), 15);
        assert_eq!(metrics.gil_release_count(), 5);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.get("total_operations"), Some(&2));
        assert_eq!(snapshot.get("total_extract_us"), Some(&300));
        assert_eq!(snapshot.get("total_convert_us"), Some(&600));
        assert_eq!(snapshot.get("total_network_us"), Some(&900));
        assert_eq!(snapshot.get("total_materialize_us"), Some(&150));
        assert_eq!(snapshot.get("total_docs"), Some(&15));
        assert_eq!(snapshot.get("total_gil_releases"), Some(&5));

        // Test averages
        assert!((metrics.avg_extract_us() - 150.0).abs() < 0.1);
        assert!((metrics.avg_convert_us() - 300.0).abs() < 0.1);
        assert!((metrics.avg_network_us() - 450.0).abs() < 0.1);
        assert!((metrics.avg_materialize_us() - 75.0).abs() < 0.1);
    }

    #[test]
    fn test_boundary_metrics_thread_safety() {
        let metrics = Arc::new(BoundaryMetrics::new());
        let mut handles = vec![];

        // Spawn 10 threads, each recording 100 timings
        for i in 0..10 {
            let metrics_clone = Arc::clone(&metrics);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let timing = BoundaryTiming {
                        operation: format!("thread_{}_op_{}", i, j),
                        extract_us: 10,
                        convert_us: 20,
                        network_us: 30,
                        materialize_us: 5,
                        total_us: 65,
                        doc_count: 1,
                        gil_release_count: 1,
                        parallel: false,
                    };
                    metrics_clone.record(&timing);
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify totals
        assert_eq!(metrics.operation_count(), 1000);
        assert_eq!(metrics.doc_count(), 1000);
        assert_eq!(metrics.gil_release_count(), 1000);
        assert_eq!(metrics.snapshot().get("total_extract_us"), Some(&10000));
    }

    #[test]
    fn test_boundary_metrics_reset() {
        let metrics = BoundaryMetrics::new();

        let timing = BoundaryTiming {
            operation: "test".to_string(),
            extract_us: 100,
            convert_us: 200,
            network_us: 300,
            materialize_us: 50,
            total_us: 650,
            doc_count: 5,
            gil_release_count: 2,
            parallel: false,
        };

        metrics.record(&timing);
        assert_eq!(metrics.operation_count(), 1);

        metrics.reset();
        assert_eq!(metrics.operation_count(), 0);
        assert_eq!(metrics.doc_count(), 0);
        assert_eq!(metrics.gil_release_count(), 0);
    }

    #[test]
    fn test_boundary_tracer_partial_phases() {
        // Test that tracer works even if not all phases are used
        let mut tracer = BoundaryTracer::new("partial");

        tracer.start_extract();
        thread::sleep(std::time::Duration::from_micros(100));
        tracer.end_extract();

        // Skip convert and network phases

        tracer.start_materialize();
        thread::sleep(std::time::Duration::from_micros(50));
        tracer.end_materialize();

        let timing = tracer.finish();

        assert!(timing.extract_us >= 100);
        assert_eq!(timing.convert_us, 0);
        assert_eq!(timing.network_us, 0);
        assert!(timing.materialize_us >= 50);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/performance/boundary.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/performance/boundary.rs` captured during meter full-codegen standardization.
```

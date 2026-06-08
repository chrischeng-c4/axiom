// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-performance-mod-rs.md#source
// CODEGEN-BEGIN
//! Performance testing and profiling infrastructure
//!
//! This module provides comprehensive performance analysis tools for cclab:
//!
//! - **Boundary Tracing**: Detailed binding boundary crossing analysis
//! - **Profiling**: CPU, memory, and GIL contention profiling
//!
//! # Modules
//!
//! - [`boundary`] - binding boundary tracing with phase-level timing
//! - [`profiler`] - Comprehensive profiling infrastructure
//!
//! # Examples
//!
//! ## Boundary Tracing
//!
//! ```rust
//! use meter::performance::boundary::BoundaryTracer;
//!
//! let mut tracer = BoundaryTracer::new("my_operation");
//! tracer.start_extract();
//! // ... extract Python data
//! tracer.end_extract();
//!
//! let timing = tracer.finish();
//! println!("{}", timing.format());
//! ```
//!
//! ## Global Metrics
//!
//! ```rust
//! use meter::performance::boundary::BoundaryMetrics;
//! use std::sync::Arc;
//!
//! let metrics = Arc::new(BoundaryMetrics::new());
//!
//! // Record timing
//! // metrics.record(&timing);
//!
//! // Get snapshot
//! let snapshot = metrics.snapshot();
//! println!("Operations: {}", snapshot.get("total_operations").unwrap_or(&0));
//! ```

pub mod boundary;
pub mod profiler;

// Re-export key types for convenience
pub use boundary::{BoundaryMetrics, BoundaryTiming, BoundaryTracer};
pub use profiler::{
    generate_flamegraph_svg, get_rss_bytes, FlamegraphData, GilContentionResult, GilTestConfig,
    MemoryProfile, MemorySnapshot, PhaseBreakdown, PhaseTiming, ProfileConfig, ProfilePhase,
    ProfileResult, Profiler,
};
// CODEGEN-END

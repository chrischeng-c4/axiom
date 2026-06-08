---
id: projects-meter-src-performance-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: runtime-resource-attribution
    role: primary
    gap: embedded-profiler-api
    claim: embedded-profiler-api
    coverage: full
    rationale: "Source template implements meter performance measurement and regression reporting surfaces."
---

# Standardized projects/meter/src/performance/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/performance/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `boundary` | projects/meter/src/performance/mod.rs | module | pub | 47 |  |
| `profiler` | projects/meter/src/performance/mod.rs | module | pub | 48 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/performance/mod.rs -->
````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/performance/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/performance/mod.rs` captured during meter full-codegen standardization.
```

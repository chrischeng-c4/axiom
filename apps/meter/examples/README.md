# data-bridge-test Examples

This directory contains example code demonstrating how to use the data-bridge-test framework.

## Available Examples

### Latency Percentiles Demo

**File**: `latency_percentiles_demo.rs`

**New!** Comprehensive demonstration of percentile-based latency analysis and regression detection.

**Run it**:
```bash
cargo run -p data-bridge-test --example latency_percentiles_demo
```

**What it demonstrates**:
1. **Uniform Distribution Benchmark** - Consistent 10ms ± 2ms latency
   - Shows healthy tail ratio (~1.0-1.5x)
   - P99.9/P99.99 close to max (good consistency)
   - Visual histogram showing distribution shape

2. **Skewed Distribution Benchmark** - 80% fast (10ms), 20% slow (50ms)
   - Shows problematic tail ratio (>4.0x)
   - P99/P99.9 much higher than median
   - Clearly visible tail latency spikes in histogram

3. **Regression Detection Comparison**
   - Mean-based detection
   - P95-based detection (recommended)
   - P99-based detection (recommended)
   - Shows why percentile-based detection is superior for tail latency

4. **Tail Latency Ratio Analysis**
   - Quick health check without baselines
   - Rule of thumb thresholds (1.0-1.5x excellent, >3.0x action required)

**Key Concepts**:
- **p999** (99.9th percentile): 1 in 1000 requests
- **p9999** (99.99th percentile): 1 in 10000 requests
- **Tail Latency Ratio**: p99/p50 (higher = more variability)
- **Histogram**: Visual distribution of latencies
- **Percentile Regression**: Compare P95/P99 instead of mean

**Why it matters**:
- Mean-based detection misses tail latency issues when <5% of requests are slow
- P95/P99 regression detection catches issues affecting user experience
- Tail ratio provides instant health check
- Essential for SLA compliance and user satisfaction

### Adaptive Benchmark Demo

**File**: `adaptive_benchmark_demo.rs`

Demonstrates adaptive benchmarking with statistical convergence.

**Run it**:
```bash
cargo run -p data-bridge-test --example adaptive_benchmark_demo
```

### Baseline Demo

**File**: `baseline_demo.rs`

Shows how to save and compare benchmark baselines over time.

**Run it**:
```bash
cargo run -p data-bridge-test --example baseline_demo
```

### JUnit Reporter Example

**File**: `junit_reporter_example.rs`

Demonstrates how to use the JUnit XML reporter for CI/CD integration.

**Run it**:
```bash
cargo run -p data-bridge-test --example junit_reporter_example
```

**What it does**:
- Creates sample test results (passed, failed, error, skipped)
- Generates JUnit XML report (`test-results.xml`)
- Generates Markdown report (`test-report.md`)
- Shows how to use different reporter formats

**Output files**:
- `test-results.xml` - JUnit XML format for CI/CD systems
- `test-report.md` - Human-readable markdown report

**Use cases**:
- GitHub Actions test reporting
- GitLab CI test integration
- Jenkins JUnit plugin
- CircleCI test results
- Any CI/CD system that supports JUnit XML format

### Boundary Tracing Example

**File**: `boundary_tracing.rs`

Shows binding boundary tracing for debugging Python/Rust interactions.

**Run it**:
```bash
cargo run -p data-bridge-test --example boundary_tracing
```

### Async Fuzzing Example

**File**: `async_fuzzing_example.rs`

Demonstrates async endpoint fuzzing for security testing.

**Run it**:
```bash
cargo run -p data-bridge-test --example async_fuzzing_example
```

### Payload Database Example

**File**: `payload_database_example.rs`

Shows how to use the security payload database for testing.

**Run it**:
```bash
cargo run -p data-bridge-test --example payload_database_example
```

## Need Help?

- Check the [main documentation](../README.md) for general usage
- Review the example source code for implementation patterns

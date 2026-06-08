# Framework Comparison Benchmark Architecture

## Overview

This benchmark suite provides a comprehensive, fair comparison between **pytest** and **cclab-test** by measuring performance across multiple dimensions while minimizing external factors.

## Design Principles

### 1. Fair Comparison
- **Same Test Logic**: Both frameworks run identical test logic
- **Same Data**: Use the same test data and fixtures
- **Isolated Measurements**: Each benchmark runs independently
- **Warm Caches**: Include warmup rounds to eliminate cold-start effects

### 2. Statistical Rigor
- **Multiple Rounds**: Run each benchmark multiple times
- **Statistics**: Calculate min, max, mean, median, and standard deviation
- **Garbage Collection**: Force GC between measurements to reduce noise
- **Memory Tracking**: Optional memory usage comparison with psutil

### 3. Comprehensive Coverage
- **Discovery**: File scanning and test collection
- **Execution**: Raw test execution speed
- **Parametrization**: Test generation and parametrized execution
- **Fixtures**: Fixture setup, dependency resolution, and injection
- **Memory**: Heap usage during test execution

## Benchmark Categories

### Test Discovery

**What it measures**: Time to find and collect tests from Python files

**pytest approach**:
1. Walk filesystem to find test files
2. Import modules
3. Collect test items using pytest hooks
4. Build test tree with parametrization

**cclab-test approach**:
1. Walk filesystem with Rust (faster I/O)
2. Parse Python AST to find test functions
3. Minimal Python imports
4. Direct test registration

**Expected speedup**: 2-3x
- Rust-based file I/O
- Faster AST parsing
- Reduced Python interpreter overhead

### Test Execution

**What it measures**: Time to run simple, fast tests

**pytest approach**:
1. Setup phase (fixture resolution)
2. Call test function
3. Assertion introspection
4. Teardown phase
5. Report generation

**cclab-test approach**:
1. Rust-based test orchestration
2. Direct async function calls
3. Custom assertion engine in Rust
4. Integrated teardown
5. Efficient result collection

**Expected speedup**: 2-5x
- Native async/await handling
- Reduced Python call overhead
- Zero-copy result collection
- Optimized assertion engine

### Parametrization

**What it measures**: Overhead of generating and running parametrized tests

**pytest approach**:
1. Parse parametrize decorator at collection time
2. Generate test items dynamically
3. Store parameters in test metadata
4. Inject parameters at runtime via fixtures

**cclab-test approach**:
1. Parse parametrize at decoration time
2. Generate test variants in Rust
3. Direct parameter passing (no fixture overhead)
4. Compile-time optimization possible

**Expected speedup**: 2-4x
- Compile-time test generation
- Efficient parameter storage
- No fixture indirection
- Reduced per-test overhead

### Fixtures

**What it measures**: Overhead of fixture setup and dependency resolution

**pytest approach**:
1. Build fixture dependency graph at collection
2. Resolve fixtures recursively at runtime
3. Cache fixtures by scope
4. Support for fixture factories and parametrization

**cclab-test approach**:
1. Rust-based dependency resolution
2. Pre-compute execution order
3. Efficient scope-based caching
4. Direct value injection

**Expected speedup**: 2-4x
- Pre-computed dependency order
- Optimized cache lookup
- Reduced call stack depth
- Native async fixture support

## Implementation Details

### Benchmarking Methodology

```python
# Standard pattern for all benchmarks
def benchmark_operation():
    results = []

    # Warmup (eliminate cold start)
    for _ in range(WARMUP_ROUNDS):
        force_gc()
        run_operation()

    # Measure
    for _ in range(MEASUREMENT_ROUNDS):
        force_gc()
        start = time.perf_counter()
        run_operation()
        elapsed = time.perf_counter() - start
        results.append(elapsed)

    return calculate_stats(results)
```

### Key Features

#### Garbage Collection Control
```python
def force_gc():
    """Triple GC to ensure clean memory state."""
    gc.collect()
    gc.collect()
    gc.collect()
```

Why triple collect? Some objects require multiple GC passes to fully reclaim (e.g., objects with `__del__`, circular references).

#### Memory Tracking
```python
mem_before = get_memory_usage()
run_operation()
mem_after = get_memory_usage()
delta = mem_after - mem_before
```

Tracks Resident Set Size (RSS) using psutil, showing actual memory consumption.

#### Statistical Analysis
```python
def calculate_stats(values):
    return {
        "min": min(values),
        "max": max(values),
        "mean": statistics.mean(values),
        "median": statistics.median(values),
        "stdev": statistics.stdev(values),
    }
```

Provides comprehensive statistical view:
- **min**: Best case (least noise)
- **max**: Worst case (may indicate outliers)
- **mean**: Average performance
- **median**: Typical performance (less affected by outliers)
- **stdev**: Consistency (lower is better)

## Avoiding Common Pitfalls

### Problem: Cold Start Effects
**Solution**: Warmup rounds before measurement

### Problem: System Noise
**Solution**: Multiple measurement rounds with statistical analysis

### Problem: Memory Fragmentation
**Solution**: Force GC between measurements

### Problem: I/O Caching
**Solution**: Use consistent file access patterns

### Problem: Python JIT Effects
**Solution**: Long enough warmup to let JIT optimize

### Problem: Unfair Comparisons
**Solution**: Same test logic, same data, same environment

## Interpreting Results

### Speedup Calculation
```
speedup = pytest_time / cclab_test_time
```

Example:
- pytest: 45.23ms
- cclab-test: 18.45ms
- Speedup: 45.23 / 18.45 = 2.45x

### Statistical Significance

Look at standard deviation relative to mean:
- **Low variance** (stdev < 10% of mean): Consistent performance
- **Medium variance** (10-20%): Some noise, still valid
- **High variance** (>20%): May need more rounds or cleaner environment

### Memory Comparison

Memory usage indicates allocation patterns:
- **Lower delta**: Less heap pressure, better GC behavior
- **Higher delta**: More allocations, may cause GC pauses

## Extending the Benchmark

### Adding a New Category

1. **Define pytest benchmark**:
```python
def benchmark_pytest_new_feature() -> Dict[str, float]:
    results = {"metric": []}

    # Warmup
    for _ in range(WARMUP_ROUNDS):
        force_gc()
        run_pytest_operation()

    # Measure
    for _ in range(MEASUREMENT_ROUNDS):
        force_gc()
        start = time.perf_counter()
        run_pytest_operation()
        elapsed_ms = (time.perf_counter() - start) * 1000
        results["metric"].append(elapsed_ms)

    return results
```

2. **Define cclab-test benchmark**:
```python
async def benchmark_dbt_new_feature() -> Dict[str, float]:
    results = {"metric": []}

    # Warmup
    for _ in range(WARMUP_ROUNDS):
        force_gc()
        await run_dbt_operation()

    # Measure
    for _ in range(MEASUREMENT_ROUNDS):
        force_gc()
        start = time.perf_counter()
        await run_dbt_operation()
        elapsed_ms = (time.perf_counter() - start) * 1000
        results["metric"].append(elapsed_ms)

    return results
```

3. **Add to main()**:
```python
async def main():
    # ... existing code ...

    print("\nRunning new feature benchmark...")
    pytest_results["new_feature"] = await benchmark_pytest_new_feature()
    dbt_results["new_feature"] = await benchmark_dbt_new_feature()

    # Update report generation to include new category
```

### Custom Metrics

You can add custom metrics:
```python
results = {
    "time_ms": [],
    "memory_mb": [],
    "cache_hits": [],
    "io_operations": [],
}
```

## Performance Optimization Guide

### For cclab-test Developers

If benchmarks show unexpected results:

1. **Check GIL Release**: Ensure CPU-intensive Rust code releases GIL
2. **Profile Rust Code**: Use `cargo flamegraph` to find bottlenecks
3. **Check Allocations**: Ensure zero-copy where possible
4. **Verify Async**: Ensure proper async/await handling
5. **Measure Boundaries**: Profile Python ↔ Rust boundary crossings

### For pytest Plugin Developers

To improve pytest performance:

1. **Lazy Loading**: Defer imports until needed
2. **Cache Results**: Cache expensive computations
3. **Reduce Hooks**: Minimize hook implementations
4. **Optimize Collection**: Speed up test collection phase
5. **Profile**: Use `pytest --profile` to find bottlenecks

## Reproducibility

### Environment
- Record Python version
- Record pytest version
- Record cclab version
- Record OS and hardware
- Record other running processes

### Variability
- Run multiple times (different days, times)
- Check for outliers
- Report confidence intervals
- Document environmental factors

## Validation

### Sanity Checks
1. **Same test count**: Both frameworks find same tests
2. **Same results**: Tests pass/fail the same way
3. **Consistent speedup**: Speedup is stable across runs
4. **Expected range**: Speedup is within expected range (1.5-5x)

### Red Flags
- Speedup > 10x: Likely measurement error
- Speedup < 1.0x: Performance regression
- High variance: Environmental noise
- Different test counts: Collection bug

## License

Same as cclab project.

# Framework Comparison Benchmarks

This directory contains comprehensive benchmarks comparing **pytest** and **cclab-test** performance.

## Overview

The benchmarks measure performance across multiple dimensions:

1. **Test Discovery** - How fast the framework finds and collects tests
2. **Test Execution** - Raw execution speed for simple tests
3. **Parametrization** - Overhead of parametrized test generation and execution
4. **Fixtures** - Overhead of fixture setup and injection
5. **Memory Usage** - Memory consumption during test execution (if psutil available)

## Files

- **`pytest_vs_cclab_test.py`** - Main benchmark script
- **`sample_tests.py`** - Sample test suite compatible with both frameworks
- **`BENCHMARK_REPORT.md`** - Generated comparison report (after running)

## Requirements

### Required
- Python 3.12+
- pytest
- cclab (with cclab-test)

### Optional
- pytest-benchmark - For pytest-specific metrics
- psutil - For memory usage tracking

Install requirements:
```bash
pip install pytest psutil
# or
uv pip install pytest psutil
```

## Running the Benchmarks

### Quick Run
```bash
python benchmarks/framework_comparison/pytest_vs_cclab_test.py
```

### From Project Root
```bash
cd /path/to/cclab
python benchmarks/framework_comparison/pytest_vs_cclab_test.py
```

### With uv
```bash
uv run python benchmarks/framework_comparison/pytest_vs_cclab_test.py
```

## Configuration

You can adjust the benchmark parameters in the script:

```python
WARMUP_ROUNDS = 3        # Number of warmup iterations
MEASUREMENT_ROUNDS = 10  # Number of measurement iterations
```

## Output

The benchmark produces two types of output:

### 1. Console Output
Real-time progress and summary table printed to console:

```
================================================================================
pytest vs cclab-test Performance Comparison
================================================================================

SUMMARY
--------------------------------------------------------------------------------
Metric                    pytest (ms)     cclab (ms) Speedup
--------------------------------------------------------------------------------
Test Discovery                  45.23            18.45     2.45x
Test Execution                  78.91            25.34     3.11x
Parametrization                 92.15            31.22     2.95x
Fixtures                        67.44            22.18     3.04x
--------------------------------------------------------------------------------
```

### 2. Markdown Report
Detailed report saved to `BENCHMARK_REPORT.md` with:
- Summary table with speedup calculations
- Detailed statistics (min, max, mean, median, stdev) for each metric
- Memory usage comparison (if psutil available)
- Analysis and conclusions

## Understanding the Results

### Expected Speedup Ranges

Based on the Rust-powered architecture, we expect:

- **Test Discovery**: 2-3x faster
  - Rust-based file scanning and parsing
  - Minimal Python interpreter overhead

- **Test Execution**: 2-5x faster
  - Native async/await handling
  - Zero-copy data structures
  - Reduced GIL contention

- **Parametrization**: 2-4x faster
  - Compile-time parameter generation
  - Efficient parameter injection

- **Fixtures**: 2-4x faster
  - Rust-based dependency resolution
  - Optimized fixture caching

### Performance Factors

**pytest advantages:**
- Mature ecosystem with extensive plugins
- Rich assertion introspection
- Flexible fixture system

**cclab-test advantages:**
- Rust engine with minimal Python overhead
- Native async/await support
- Zero-copy data handling
- Integrated collection and execution

## Troubleshooting

### Import Errors
If you get import errors, ensure you've built cclab:
```bash
maturin develop --release
```

### pytest Not Found
Install pytest:
```bash
pip install pytest
```

### Memory Tracking Disabled
If you see "memory tracking disabled", install psutil:
```bash
pip install psutil
```

### Benchmark Variance
If you see high variance in results:
- Close other applications to reduce system noise
- Run multiple times and average results
- Increase `MEASUREMENT_ROUNDS` for more stable results

## Adding New Benchmarks

To add new benchmark categories:

1. Add pytest benchmark function:
```python
def benchmark_pytest_new_feature() -> Dict[str, float]:
    # Implement pytest benchmark
    pass
```

2. Add cclab-test benchmark function:
```python
async def benchmark_dbt_new_feature() -> Dict[str, float]:
    # Implement cclab-test benchmark
    pass
```

3. Add to main() execution and report generation

## CI/CD Integration

To run in CI/CD:

```yaml
- name: Run framework comparison
  run: |
    maturin develop --release
    python benchmarks/framework_comparison/pytest_vs_cclab_test.py

- name: Upload benchmark report
  uses: actions/upload-artifact@v3
  with:
    name: framework-benchmark
    path: benchmarks/framework_comparison/BENCHMARK_REPORT.md
```

## Performance Regression Detection

To detect regressions:

1. Save baseline report:
```bash
cp BENCHMARK_REPORT.md BENCHMARK_BASELINE.md
```

2. After changes, compare:
```bash
python pytest_vs_cclab_test.py
# Manually compare with BENCHMARK_BASELINE.md
```

3. Fail CI if speedup drops below threshold

## License

Same as cclab project.

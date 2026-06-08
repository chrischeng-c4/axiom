# Framework Comparison Benchmark Examples

This guide provides practical examples of using the framework comparison benchmarks.

## Table of Contents

1. [Basic Usage](#basic-usage)
2. [Running Individual Benchmarks](#running-individual-benchmarks)
3. [Customizing Benchmark Parameters](#customizing-benchmark-parameters)
4. [CI/CD Integration](#cicd-integration)
5. [Performance Regression Detection](#performance-regression-detection)
6. [Advanced Analysis](#advanced-analysis)

## Basic Usage

### Run Full Benchmark Suite

```bash
# From project root
cd /path/to/cclab

# Validate setup first
python benchmarks/framework_comparison/validate.py

# Run full benchmark
python benchmarks/framework_comparison/pytest_vs_cclab_test.py
```

Expected output:
```
================================================================================
Framework Comparison Benchmark
================================================================================
Python: 3.12.0
Rounds: 10
Warmup: 3
================================================================================

[1/8] Running pytest discovery benchmark...
  ✓ Completed (mean: 42.15 ms)

[2/8] Running pytest execution benchmark...
  ✓ Completed (mean: 75.34 ms)

...

================================================================================
pytest vs cclab-test Performance Comparison
================================================================================

SUMMARY
--------------------------------------------------------------------------------
Metric                    pytest (ms)     cclab (ms) Speedup
--------------------------------------------------------------------------------
Test Discovery                  42.15            15.32     2.75x
Test Execution                  75.34            23.45     3.21x
Parametrization                 89.22            28.91     3.09x
Fixtures                        64.78            21.34     3.04x
--------------------------------------------------------------------------------

✓ Markdown report saved to: benchmarks/framework_comparison/BENCHMARK_REPORT.md
```

### Quick Validation

Before running the full benchmark, validate your setup:

```bash
python benchmarks/framework_comparison/validate.py
```

This checks:
- pytest installation
- psutil installation (optional)
- cclab-test availability
- Sample test files
- Basic execution for both frameworks

## Running Individual Benchmarks

You can extract and run individual benchmarks for focused testing.

### Discovery Benchmark Only

```python
#!/usr/bin/env python3
import asyncio
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from benchmarks.framework_comparison.pytest_vs_cclab_test import (
    benchmark_pytest_discovery,
    benchmark_dbt_discovery,
    calculate_stats,
)

async def main():
    print("Running discovery benchmarks...")

    # pytest
    pytest_result = benchmark_pytest_discovery()
    pytest_stats = calculate_stats(pytest_result["discovery_time_ms"])
    print(f"pytest: {pytest_stats['mean']:.2f}ms")

    # cclab-test
    dbt_result = await benchmark_dbt_discovery()
    dbt_stats = calculate_stats(dbt_result["discovery_time_ms"])
    print(f"cclab-test: {dbt_stats['mean']:.2f}ms")

    # Speedup
    speedup = pytest_stats['mean'] / dbt_stats['mean']
    print(f"Speedup: {speedup:.2f}x")

if __name__ == "__main__":
    asyncio.run(main())
```

### Execution Benchmark Only

```python
#!/usr/bin/env python3
import asyncio
from benchmarks.framework_comparison.pytest_vs_cclab_test import (
    benchmark_pytest_execution,
    benchmark_dbt_execution,
    calculate_stats,
)

async def main():
    print("Running execution benchmarks...")

    pytest_result = benchmark_pytest_execution()
    dbt_result = await benchmark_dbt_execution()

    pytest_stats = calculate_stats(pytest_result["execution_time_ms"])
    dbt_stats = calculate_stats(dbt_result["execution_time_ms"])

    print(f"pytest: {pytest_stats['mean']:.2f}ms")
    print(f"cclab-test: {dbt_stats['mean']:.2f}ms")
    print(f"Speedup: {pytest_stats['mean'] / dbt_stats['mean']:.2f}x")

    # Memory comparison (if available)
    if "memory_delta_mb" in pytest_result:
        pytest_mem = calculate_stats(pytest_result["memory_delta_mb"])
        dbt_mem = calculate_stats(dbt_result["memory_delta_mb"])
        print(f"\nMemory Usage:")
        print(f"pytest: {pytest_mem['mean']:.2f}MB")
        print(f"cclab-test: {dbt_mem['mean']:.2f}MB")

if __name__ == "__main__":
    asyncio.run(main())
```

## Customizing Benchmark Parameters

### Increase Measurement Rounds for Stability

Edit `pytest_vs_cclab_test.py`:

```python
# At the top of the file
WARMUP_ROUNDS = 5        # Increase from 3
MEASUREMENT_ROUNDS = 20  # Increase from 10
```

Or create a custom script:

```python
#!/usr/bin/env python3
import asyncio
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

# Monkey-patch constants
import benchmarks.framework_comparison.pytest_vs_cclab_test as bench
bench.WARMUP_ROUNDS = 5
bench.MEASUREMENT_ROUNDS = 20

# Run main
asyncio.run(bench.main())
```

### Run with Different Sample Test Sizes

Create a new sample test file with more tests:

```python
# large_sample_tests.py
def test_1(): assert 1 + 1 == 2
def test_2(): assert 2 + 2 == 4
def test_3(): assert 3 + 3 == 6
# ... add 100 more tests ...

# Then modify SAMPLE_TESTS_PATH in benchmark script
SAMPLE_TESTS_PATH = Path(__file__).parent / "large_sample_tests.py"
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Framework Benchmark

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.12'

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install dependencies
        run: |
          pip install maturin pytest psutil

      - name: Build cclab
        run: maturin develop --release

      - name: Validate benchmark setup
        run: python benchmarks/framework_comparison/validate.py

      - name: Run benchmark
        run: python benchmarks/framework_comparison/pytest_vs_cclab_test.py

      - name: Upload report
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-report
          path: benchmarks/framework_comparison/BENCHMARK_REPORT.md

      - name: Check for regression
        run: |
          # Extract speedup values from report
          # Fail if any speedup < 2.0x
          python -c "
          import re
          with open('benchmarks/framework_comparison/BENCHMARK_REPORT.md') as f:
              content = f.read()
              speedups = re.findall(r'\\*\\*([0-9.]+)x\\*\\*', content)
              speedups = [float(s) for s in speedups]
              min_speedup = min(speedups)
              print(f'Minimum speedup: {min_speedup:.2f}x')
              if min_speedup < 2.0:
                  print('ERROR: Performance regression detected!')
                  exit(1)
              print('✓ Performance target met')
          "
```

### GitLab CI

```yaml
benchmark:
  stage: test
  image: python:3.12

  before_script:
    - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    - source $HOME/.cargo/env
    - pip install maturin pytest psutil

  script:
    - maturin develop --release
    - python benchmarks/framework_comparison/validate.py
    - python benchmarks/framework_comparison/pytest_vs_cclab_test.py

  artifacts:
    paths:
      - benchmarks/framework_comparison/BENCHMARK_REPORT.md
    expire_in: 30 days
```

## Performance Regression Detection

### Save Baseline

After a known good version:

```bash
# Run benchmark
python benchmarks/framework_comparison/pytest_vs_cclab_test.py

# Save as baseline
cp benchmarks/framework_comparison/BENCHMARK_REPORT.md \
   benchmarks/framework_comparison/BASELINE.md

# Commit baseline
git add benchmarks/framework_comparison/BASELINE.md
git commit -m "benchmark: save performance baseline"
```

### Compare Against Baseline

```python
#!/usr/bin/env python3
"""Compare current benchmark against baseline."""
import re
import sys

def extract_speedups(report_path):
    """Extract speedup values from markdown report."""
    with open(report_path) as f:
        content = f.read()
        speedups = re.findall(r'\*\*([0-9.]+)x\*\*', content)
        return [float(s) for s in speedups]

def main():
    baseline = extract_speedups('benchmarks/framework_comparison/BASELINE.md')
    current = extract_speedups('benchmarks/framework_comparison/BENCHMARK_REPORT.md')

    print("Performance Comparison")
    print("=" * 60)

    categories = ["Discovery", "Execution", "Parametrization", "Fixtures"]

    regression_found = False

    for i, category in enumerate(categories):
        if i >= len(baseline) or i >= len(current):
            continue

        base = baseline[i]
        curr = current[i]
        change = ((curr - base) / base) * 100

        status = "✓" if change >= -5 else "⚠" if change >= -10 else "✗"
        print(f"{status} {category:<20} {base:.2f}x → {curr:.2f}x ({change:+.1f}%)")

        if change < -10:
            regression_found = True

    print("=" * 60)

    if regression_found:
        print("✗ Performance regression detected (>10% slower)")
        return 1
    else:
        print("✓ No significant regression")
        return 0

if __name__ == "__main__":
    sys.exit(main())
```

Run comparison:
```bash
python compare_baseline.py
```

## Advanced Analysis

### Statistical Analysis

```python
#!/usr/bin/env python3
"""Advanced statistical analysis of benchmark results."""
import asyncio
import statistics
from benchmarks.framework_comparison.pytest_vs_cclab_test import (
    benchmark_pytest_execution,
    benchmark_dbt_execution,
    calculate_stats,
)

async def analyze_variance():
    """Analyze variance and consistency of benchmarks."""

    # Run multiple benchmark sets
    num_sets = 5
    pytest_means = []
    dbt_means = []

    for i in range(num_sets):
        print(f"Running set {i+1}/{num_sets}...")

        pytest_result = benchmark_pytest_execution()
        dbt_result = await benchmark_dbt_execution()

        pytest_stats = calculate_stats(pytest_result["execution_time_ms"])
        dbt_stats = calculate_stats(dbt_result["execution_time_ms"])

        pytest_means.append(pytest_stats["mean"])
        dbt_means.append(dbt_stats["mean"])

    # Analyze
    print("\nAnalysis")
    print("=" * 60)

    print(f"pytest mean: {statistics.mean(pytest_means):.2f}ms")
    print(f"pytest stdev: {statistics.stdev(pytest_means):.2f}ms")
    print(f"pytest CV: {statistics.stdev(pytest_means) / statistics.mean(pytest_means) * 100:.1f}%")

    print(f"\ncclab-test mean: {statistics.mean(dbt_means):.2f}ms")
    print(f"cclab-test stdev: {statistics.stdev(dbt_means):.2f}ms")
    print(f"cclab-test CV: {statistics.stdev(dbt_means) / statistics.mean(dbt_means) * 100:.1f}%")

    print(f"\nAverage speedup: {statistics.mean(pytest_means) / statistics.mean(dbt_means):.2f}x")

if __name__ == "__main__":
    asyncio.run(analyze_variance())
```

### Memory Profiling

```python
#!/usr/bin/env python3
"""Detailed memory profiling."""
import asyncio
import psutil
import os
from benchmarks.framework_comparison.pytest_vs_cclab_test import (
    benchmark_pytest_execution,
    benchmark_dbt_execution,
)

async def profile_memory():
    """Profile memory usage in detail."""
    process = psutil.Process(os.getpid())

    # Baseline
    baseline = process.memory_info()
    print(f"Baseline RSS: {baseline.rss / 1024 / 1024:.2f}MB")

    # pytest
    print("\nRunning pytest benchmark...")
    pytest_result = benchmark_pytest_execution()
    pytest_mem = process.memory_info()
    print(f"After pytest: {pytest_mem.rss / 1024 / 1024:.2f}MB")
    print(f"Delta: {(pytest_mem.rss - baseline.rss) / 1024 / 1024:.2f}MB")

    # Force GC
    import gc
    gc.collect()
    gc.collect()
    gc.collect()

    after_gc = process.memory_info()
    print(f"After GC: {after_gc.rss / 1024 / 1024:.2f}MB")

    # cclab-test
    print("\nRunning cclab-test benchmark...")
    dbt_result = await benchmark_dbt_execution()
    dbt_mem = process.memory_info()
    print(f"After cclab-test: {dbt_mem.rss / 1024 / 1024:.2f}MB")
    print(f"Delta: {(dbt_mem.rss - after_gc.rss) / 1024 / 1024:.2f}MB")

if __name__ == "__main__":
    asyncio.run(profile_memory())
```

### Flamegraph Generation

```bash
# Profile pytest with py-spy
py-spy record -o pytest.svg -- python -m pytest benchmarks/framework_comparison/sample_tests.py

# Profile cclab-test benchmark
py-spy record -o cclab-test.svg -- python -c "
import asyncio
from benchmarks.framework_comparison.pytest_vs_cclab_test import benchmark_dbt_execution
asyncio.run(benchmark_dbt_execution())
"
```

## Troubleshooting Examples

### High Variance Issue

If you see high variance in results:

```python
# Check system load
import psutil
print(f"CPU Usage: {psutil.cpu_percent(interval=1)}%")
print(f"Memory Usage: {psutil.virtual_memory().percent}%")

# Increase warmup and measurement rounds
WARMUP_ROUNDS = 10
MEASUREMENT_ROUNDS = 30
```

### Different Test Counts

If pytest and cclab-test find different numbers of tests:

```bash
# Check pytest collection
python -m pytest benchmarks/framework_comparison/sample_tests.py --collect-only -q

# Check file manually
grep -c "^def test_" benchmarks/framework_comparison/sample_tests.py
```

## License

Same as cclab project.

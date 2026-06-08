# Quick Start Guide

Get started with the pytest vs cclab-test benchmark in 5 minutes.

## Prerequisites

- Python 3.12+
- Rust toolchain
- 5 minutes

## Step 1: Install Dependencies

```bash
# Install Python packages
pip install pytest psutil

# Or with uv
uv pip install pytest psutil
```

## Step 2: Build cclab

```bash
# From project root
cd /path/to/cclab

# Build in release mode (important for accurate benchmarks!)
maturin develop --release
```

## Step 3: Validate Setup

```bash
python benchmarks/framework_comparison/validate.py
```

Expected output:
```
======================================================================
Framework Comparison Benchmark Validation
======================================================================

Checking dependencies...
----------------------------------------------------------------------
✓ pytest 7.4.0 installed
✓ psutil 5.9.5 installed (memory tracking enabled)
✓ cclab-test available
✓ Sample tests found: benchmarks/framework_comparison/sample_tests.py

Testing execution...
----------------------------------------------------------------------
✓ pytest execution successful (13 tests passed)
✓ cclab-test execution successful (3 tests passed)

Testing infrastructure...
----------------------------------------------------------------------
✓ Benchmark infrastructure working (mean: 0.124ms)

======================================================================
✓ All required checks passed!

You can now run the full benchmark:
  python benchmarks/framework_comparison/pytest_vs_cclab_test.py
```

## Step 4: Run Benchmark

```bash
python benchmarks/framework_comparison/pytest_vs_cclab_test.py
```

This will:
1. Run pytest benchmarks (discovery, execution, parametrization, fixtures)
2. Run cclab-test benchmarks (same categories)
3. Calculate statistics and speedup
4. Print console report
5. Save detailed markdown report

**Time**: ~2-3 minutes (depending on your hardware)

## Step 5: Review Results

### Console Output

```
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
```

### Detailed Report

```bash
# View markdown report
cat benchmarks/framework_comparison/BENCHMARK_REPORT.md

# Or open in your editor
code benchmarks/framework_comparison/BENCHMARK_REPORT.md
```

## What's Next?

### Understand the Results
- Read [ARCHITECTURE.md](ARCHITECTURE.md) for benchmark design details
- Read [README.md](README.md) for comprehensive documentation

### Run Individual Benchmarks
- See [EXAMPLES.md](EXAMPLES.md) for focused benchmark examples

### Integrate with CI/CD
- See [EXAMPLES.md](EXAMPLES.md) CI/CD section for GitHub Actions/GitLab CI

### Track Performance Over Time
- Save baseline with `cp BENCHMARK_REPORT.md BASELINE.md`
- Run regression checks in CI

## Common Issues

### ✗ pytest not installed
```bash
pip install pytest
```

### ✗ cclab-test not available
```bash
maturin develop --release
```

### ⚠ psutil not installed
Optional but recommended:
```bash
pip install psutil
```

### High variance in results
- Close other applications
- Run on a quiet system
- Increase `MEASUREMENT_ROUNDS` in the script

### Benchmark takes too long
- Reduce `MEASUREMENT_ROUNDS` (default: 10)
- Reduce `WARMUP_ROUNDS` (default: 3)

## Quick Reference

| Command | Purpose |
|---------|---------|
| `python validate.py` | Check setup before benchmarking |
| `python pytest_vs_cclab_test.py` | Run full benchmark suite |
| `cat BENCHMARK_REPORT.md` | View detailed results |
| `maturin develop --release` | Rebuild after code changes |

## Performance Targets

Expected speedup ranges (cclab-test vs pytest):

- Test Discovery: **2-3x faster**
- Test Execution: **2-5x faster**
- Parametrization: **2-4x faster**
- Fixtures: **2-4x faster**

If you see significantly different results, check:
1. Built in release mode? (`maturin develop --release`)
2. Clean environment? (close other apps)
3. Latest code? (`git pull && maturin develop --release`)

## Need Help?

- Check [README.md](README.md) for detailed documentation
- Check [EXAMPLES.md](EXAMPLES.md) for usage examples
- Check [ARCHITECTURE.md](ARCHITECTURE.md) for design details
- Open an issue on GitHub

## Contributing

Found a bug or have an improvement?

1. Test your changes with the validation script
2. Run the full benchmark suite
3. Update documentation if needed
4. Submit a pull request

Happy benchmarking! 🚀

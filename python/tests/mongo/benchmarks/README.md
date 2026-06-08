# Comprehensive Benchmark Suite for cclab

## Overview

This comprehensive benchmark suite compares **cclab** (Rust-backed MongoDB ORM) against 5 other Python MongoDB frameworks:

| Framework | Type | Description |
|-----------|------|-------------|
| **cclab** | Async + Rust | Rust backend with zero Python byte handling |
| **Beanie** | Async ODM | Popular async ODM built on motor |
| **motor** | Async (raw) | Pure pymongo-async (no ODM layer) |
| **pymongo** | Sync | Standard synchronous pymongo |
| **gevent + pymongo** | Sync + greenlets | Concurrency via greenlets |
| **MongoEngine** | Sync ODM | Traditional sync ODM |

## Test Matrix

- **Operations**: 9 (insert_single, insert_bulk, find_one, find_many, count, update_single, update_bulk, delete_single, delete_bulk)
- **Batch Sizes**: 5 (10, 100, 1000, 10000, 50000)
- **Total Test Cases**: **~270 tests** (6 frameworks × 9 operations × 5 batch sizes)

## Prerequisites

1. **MongoDB running on port 27018**:
   ```bash
   docker ps --filter "publish=27018"
   ```

2. **Dependencies installed** (already done):
   - gevent
   - matplotlib
   - tabulate
   - pytest-benchmark

## Running Benchmarks

### Option 1: Full Comprehensive Suite (~30-60 minutes)

Run all 270+ benchmark tests:

```bash
MONGODB_URI="mongodb://localhost:27018/cclab-benchmark" \
uv run pytest tests/benchmarks/ -v \
  --benchmark-only \
  --benchmark-json=tests/benchmarks/results/full_results.json \
  --benchmark-sort=mean \
  --benchmark-group-by=group
```

### Option 2: By Operation (Faster)

Run benchmarks for specific operations:

```bash
# Bulk insert benchmarks only (~5-10 min)
uv run pytest tests/benchmarks/test_insert_bulk.py -v --benchmark-only

# Single insert benchmarks only (~2-3 min)
uv run pytest tests/benchmarks/test_insert_single.py -v --benchmark-only

# Query benchmarks only (~10-15 min)
uv run pytest tests/benchmarks/test_query_*.py -v --benchmark-only

# Update benchmarks only (~10-15 min)
uv run pytest tests/benchmarks/test_update_*.py -v --benchmark-only

# Delete benchmarks only (~10-15 min)
uv run pytest tests/benchmarks/test_delete_*.py -v --benchmark-only
```

### Option 3: By Batch Size

Run benchmarks for specific batch sizes:

```bash
# Small batches (10, 100) - faster
uv run pytest tests/benchmarks/ -v --benchmark-only -k "10 or 100"

# Medium batches (1000)
uv run pytest tests/benchmarks/ -v --benchmark-only -k "1000"

# Large batches (10000, 50000) - slower
uv run pytest tests/benchmarks/ -v --benchmark-only -k "10000 or 50000"
```

### Option 4: By Framework

Compare specific frameworks:

```bash
# cclab vs Beanie only
uv run pytest tests/benchmarks/ -v --benchmark-only -k "cclab or beanie"

# cclab vs motor only
uv run pytest tests/benchmarks/ -v --benchmark-only -k "cclab or motor"

# Async frameworks (cclab, beanie, motor)
uv run pytest tests/benchmarks/ -v --benchmark-only -k "cclab or beanie or motor"
```

## Generate Report

After running benchmarks, generate a comparison report:

```bash
# Generate markdown report
uv run python tests/benchmarks/report_generator.py \
  tests/benchmarks/results/full_results.json \
  -o tests/benchmarks/results/benchmark_report.md

# View report
cat tests/benchmarks/results/benchmark_report.md
```

## Benchmark Output Columns

pytest-benchmark provides:
- **Min**: Minimum execution time
- **Max**: Maximum execution time
- **Mean**: Average execution time (primary metric)
- **StdDev**: Standard deviation (consistency)
- **Median**: Median execution time
- **IQR**: Interquartile range
- **Outliers**: Statistical outliers count
- **Rounds**: Number of rounds executed
- **Iterations**: Iterations per round

## Interpretation

- **Lower times are better**
- **Speedup factor**: If Beanie takes 0.010s and cclab takes 0.004s, cclab is **2.5x faster**
- **Consistency**: Lower StdDev indicates more consistent performance

## File Structure

```
tests/benchmarks/
├── __init__.py                  # Package marker
├── conftest.py                  # Pytest fixtures for all frameworks
├── helpers.py                   # Shared helper functions
├── README.md                    # This file
│
├── # Benchmark Test Files (9 operations)
├── test_insert_single.py        # Single document insert
├── test_insert_bulk.py          # Bulk insert (10-50k docs)
├── test_query_find_one.py       # find_one queries
├── test_query_find_many.py      # find_many queries
├── test_query_count.py          # count() operations
├── test_update_single.py        # Single document update
├── test_update_bulk.py          # Bulk updates
├── test_delete_single.py        # Single document delete
├── test_delete_bulk.py          # Bulk deletes
│
├── # Analysis Tools
├── report_generator.py          # Generate comparison reports
│
└── results/                     # Output directory
    ├── .gitkeep
    └── *.json                   # Benchmark results (generated)
```

## Tips for Accurate Benchmarks

1. **Close other applications** to reduce system noise
2. **Disable GC** for more consistent results:
   ```bash
   pytest tests/benchmarks/ --benchmark-disable-gc
   ```

3. **Warm-up rounds** (already configured in conftest.py)

4. **Multiple runs** for statistical significance:
   ```bash
   # Run benchmarks 3 times and compare
   pytest tests/benchmarks/ --benchmark-autosave
   pytest tests/benchmarks/ --benchmark-compare
   ```

5. **Check MongoDB performance**:
   ```bash
   docker stats tech-platform-mongo
   ```

## Next Steps

1. Run a subset of benchmarks first (e.g., single insert) to verify setup
2. Run the full suite when you have 30-60 minutes
3. Generate and analyze the report
4. Identify performance bottlenecks
5. Optimize cclab based on findings

## Troubleshooting

### Import Errors

If you see import errors, ensure you're running from the cclab root:
```bash
cd cclab
uv run pytest tests/benchmarks/...
```

### MongoDB Connection Errors

Verify MongoDB is accessible:
```bash
mongosh "mongodb://localhost:27018/cclab-benchmark" --eval "db.runCommand({ping: 1})"
```

### Memory Issues (50000 batch)

If running out of memory with 50000-document batches:
```bash
# Skip 50000 batch
pytest tests/benchmarks/ --benchmark-only -k "not 50000"
```

## Example Output

```
------------------------------------------------ benchmark 'bulk-insert': 6 tests ------------------------------------------------
Name (time in s)                             Min      Max     Mean   StdDev   Median      IQR  Outliers  OPS  Rounds  Iterations
---------------------------------------------------------------------------------------------------------------------------------
test_cclab_insert_many[1000]       0.0823   0.0891   0.0851   0.0021   0.0847   0.0024       2;0  11.7514       5          20
test_beanie_insert_many[1000]            0.2145   0.2301   0.2198   0.0058   0.2187   0.0077       1;0   4.5497       5          20
test_motor_insert_many[1000]             0.1876   0.2012   0.1932   0.0051   0.1921   0.0068       2;0   5.1761       5          20
test_pymongo_sync_insert_many[1000]      0.1956   0.2098   0.2014   0.0054   0.2005   0.0071       1;1   4.9653       5          20
test_pymongo_gevent_insert_many[1000]    0.1989   0.2145   0.2053   0.0060   0.2041   0.0079       2;0   4.8709       5          20
test_mongoengine_insert_many[1000]       0.2312   0.2476   0.2380   0.0062   0.2367   0.0084       1;0   4.2017       5          20
---------------------------------------------------------------------------------------------------------------------------------

Result: cclab is 2.5x faster than Beanie for 1000-document bulk insert
```

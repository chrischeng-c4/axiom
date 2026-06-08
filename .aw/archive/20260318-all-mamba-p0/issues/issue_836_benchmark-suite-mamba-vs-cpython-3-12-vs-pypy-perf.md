---
number: 836
title: "Benchmark suite — Mamba vs CPython 3.12 vs PyPy performance comparison"
state: open
labels: [enhancement, P0, crate:mamba]
group: "benchmark-suite"
---

# #836 — Benchmark suite — Mamba vs CPython 3.12 vs PyPy performance comparison

## Summary

Create a comprehensive benchmark suite comparing Mamba JIT/AOT execution against CPython 3.12 and PyPy 7.3.

## Why P0

Performance is Mamba's core value proposition. Without published benchmarks, there is no evidence to support the claim that native compilation is faster. This is the single most important thing for adoption.

## Proposed Benchmarks

### Micro-benchmarks
- **fibonacci** (recursive, iterative) — function call overhead
- **nbody** — floating point arithmetic
- **spectral-norm** — matrix math, loops
- **mandelbrot** — tight numeric loops
- **binary-trees** — GC pressure, allocation
- **fannkuch-redux** — permutation, integer ops

### Real-world workloads
- **JSON parse/serialize** — stdlib json module
- **string processing** — regex, split, join, format
- **file I/O** — read/write large files
- **dict operations** — creation, lookup, iteration
- **class instantiation** — object creation overhead

### Methodology
- Each benchmark runs 100 iterations, report median + p99
- Compare: Mamba JIT, Mamba AOT (Cranelift), Mamba AOT (LLVM -O2), CPython 3.12, PyPy 7.3
- Output: markdown table + optional chart (cclab-plot)
- CI integration: track regressions across commits

## Implementation

- `crates/mamba/benches/` directory
- Rust benchmark harness using `criterion` or custom timing
- Python reference scripts for CPython/PyPy comparison
- `mamba bench` CLI subcommand

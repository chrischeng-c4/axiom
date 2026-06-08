---
change: all-mamba-p0
group: benchmark-suite
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: Benchmark harness
- **Question**: Use an existing benchmark framework (e.g., hyperfine for CLI timing, criterion for Rust internals) or build a custom harness that runs all three runtimes?
- **Answer**: hyperfine + Python scripts — use hyperfine for end-to-end timing, Python scripts to orchestrate CPython/PyPy runs.

### Q2: CI integration
- **Question**: Should benchmarks run in CI on every commit (regression detection) or only on-demand?
- **Answer**: On-demand only — not necessary for CI. Run manually when needed.


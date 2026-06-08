---
change: orbit-p2-enhancements
date: 2026-02-05
---

# Clarifications

## Q1: Test Framework
- **Question**: For testing infrastructure, what's the preferred approach?
- **Answer**: Rust tests for Rust code, cclab-probe for Python tests
- **Rationale**: Separates concerns - Rust tests for low-level, Python tests for integration

## Q2: Documentation Format
- **Question**: What format for documentation (#72, #73)?
- **Answer**: Markdown in docs/ directory
- **Rationale**: Simple, portable, integrates with GitHub rendering

## Q3: Debug API
- **Question**: Should debug mode API match asyncio exactly?
- **Answer**: asyncio-compatible: loop.set_debug(), loop.get_debug()
- **Rationale**: Familiar API for Python developers, drop-in compatibility

## Q4: Benchmark Baseline
- **Question**: What baseline comparison for benchmarks?
- **Answer**: Compare against both uvloop and asyncio stdlib
- **Rationale**: Complete picture of performance vs main alternatives

## Q5: Stress Test Scale
- **Question**: Target scale for stress tests?
- **Answer**: 10k concurrent connections as baseline
- **Rationale**: Standard high-concurrency target, achievable on CI

## Q6: Windows Support
- **Question**: Windows signal handling (#68)?
- **Answer**: Skipped - deferred until Windows CI is available
- **Rationale**: No Windows CI infrastructure currently


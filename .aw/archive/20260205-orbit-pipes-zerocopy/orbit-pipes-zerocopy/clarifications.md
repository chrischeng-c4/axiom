---
change: orbit-pipes-zerocopy
date: 2026-02-05
---

# Clarifications

## Q1: Platforms
- **Question**: Which platforms should be supported for pipe implementation?
- **Answer**: All platforms (Unix + Windows) - full cross-platform support
- **Rationale**: Orbit aims to be a cross-platform asyncio replacement, so pipes should work everywhere

## Q2: Zero-Copy
- **Question**: What level of zero-copy optimization is needed?
- **Answer**: Full zero-copy with buffer pools, splice/sendfile, and registered buffers
- **Rationale**: Performance is a key goal for orbit; full zero-copy maximizes throughput for large data transfers

## Q3: Testing
- **Question**: How comprehensive should the testing infrastructure be?
- **Answer**: Comprehensive - integration tests, benchmarks, stress tests with CI integration
- **Rationale**: Thorough testing ensures reliability and catches regressions; benchmarks track performance over time

## Q4: Lifecycle
- **Question**: Should protocol lifecycle support async drop and graceful shutdown?
- **Answer**: Full lifecycle with async init/shutdown, graceful close, and timeout handling
- **Rationale**: Proper resource cleanup is critical for long-running services; async drop prevents resource leaks


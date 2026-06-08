---
change: orbit-architecture
date: 2026-02-06
---

# Clarifications

## Q1: kqueue Scope
- **Question**: Which platforms to optimize?
- **Answer**: macOS/BSD only, Windows deferred (no CI)
- **Rationale**: No Windows CI available, focus on macOS first

## Q2: Allocator Implementation
- **Question**: Use external crate or implement?
- **Answer**: Self-implement slab/arena allocators
- **Rationale**: No external dependencies, tailored for orbit's needs

## Q3: Feature Flags
- **Question**: What features to expose?
- **Answer**: tcp, udp, timers, signals, tls (rustls), optional jemalloc/mimalloc
- **Rationale**: Modular build for different use cases

## Q4: no_std
- **Question**: Support no_std?
- **Answer**: Skipped - orbit requires Python runtime
- **Rationale**: no_std meaningless for Python asyncio loop


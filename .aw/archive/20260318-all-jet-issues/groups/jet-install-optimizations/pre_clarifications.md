---
change: all-jet-issues
group: jet-install-optimizations
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: No threshold — prefetch all transitive deps eagerly level-by-level. The semaphore already bounds concurrency. Benchmark showed 12.6s true cold vs 1.3s with disk cache, so the bottleneck is metadata fetch latency not bandwidth

### Q2: General
- **Answer**: Yes, auto-migrate — on first access, if ~/.jet-store/.metadata/ exists and ~/.cache/jet/metadata/ doesn't, move the directory. One-time migration, no fallback needed


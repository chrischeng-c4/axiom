---
change: jet-remaining
group: install-performance-reliability
date: 2026-03-19
---

# Requirements

Optimize jet install cold performance to target ≤ 3.0s (beating pnpm). Implement a persistent disk-based metadata cache and enable HTTP/2 multiplexing with connection reuse. Add speculative prefetching for transitive dependencies and overlapping download/resolution. Verify and finalize resolver fixes for version conflicts, || range syntax, pre-release version matching, npm: aliases, and optional dependencies with platform filtering.

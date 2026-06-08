---
change: jet-remaining
group: install-performance-reliability
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: Metadata TTL
- **Answer**: Yes, 5 minutes is a good baseline TTL for disk-based metadata cache as suggested in #881. This provides speed while ensuring we don't use stale version data for too long.

### Q2: Prefetch Limit
- **Answer**: We should prefetch the top 3-5 most common transitive dependencies per package (e.g., react -> react-dom, scheduler). This covers the majority of fast-path resolution scenarios without excessive network load.

### Q3: Platform Filtering
- **Answer**: Yes, the should_skip_optional() logic introduced in #883 correctly handles architecture/OS platform filtering (as seen with @rollup/rollup-darwin-arm64). This should be verified for Windows and Linux as part of the test suite.


---
change: all-open-jet-issues
group: jet-install-optimizations
date: 2026-03-18
---

# Requirements

Implement a persistent disk-based metadata cache in ~/.jet-store/.metadata/ to store registry JSON responses with a 5-minute TTL. Optimize the reqwest HTTP client for HTTP/2 multiplexing and connection reuse. Implement speculative prefetching of metadata for transitive dependencies. Overlap resolution and downloads by starting tarball fetches as soon as versions are resolved. Consolidate and verify resolver fixes for || ranges, pre-release versions, space-separated ranges, and npm: aliases. Support optionalDependencies with platform-specific filtering. Target: Cold install time ≤ 3.0s.

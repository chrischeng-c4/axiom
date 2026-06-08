---
change: all-jet-issues
group: jet-install-optimizations
date: 2026-03-18
---

# Requirements

Optimize `jet install` to achieve a cold install time of ≤ 3.0s. This involves implementing/validating a disk metadata cache, HTTP/2 multiplexing, and speculative prefetch. Fix resolver bugs related to version conflicts, OR syntax, pre-releases, and optional dependencies. Migrate the cache location to conform to XDG specs (~/.cache/jet/metadata/) and add a `--no-cache` CLI flag.

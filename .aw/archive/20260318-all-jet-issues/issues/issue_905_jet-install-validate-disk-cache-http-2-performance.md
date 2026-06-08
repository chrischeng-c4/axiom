---
number: 905
title: "jet-install: validate disk cache + HTTP/2 performance impact"
state: open
labels: [enhancement, crate:jet]
group: "jet-install-optimizations"
---

# #905 — jet-install: validate disk cache + HTTP/2 performance impact

## Context

Registry disk cache (`~/.jet-store/.metadata/`) and HTTP/2 adaptive window were added in cadf9f64 but not yet benchmarked. Current cold install: 4.9s → target ≤3.0s.

## Tasks

1. **Benchmark disk cache impact**
   - First cold install (no cache): measure baseline
   - Second cold install (cache populated): measure improvement
   - Compare with `--no-cache` flag (not yet implemented)

2. **Verify HTTP/2 multiplexing**
   - Confirm registry.npmjs.org serves HTTP/2 via ALPN
   - Measure connection count with vs without `http2_adaptive_window`
   - Log request timing to quantify overhead reduction

3. **Cache location migration**
   - Current: `~/.jet-store/.metadata/` (implementation)
   - Target: `~/.cache/jet/metadata/` (per user decision, XDG spec)
   - Respect `XDG_CACHE_HOME` env var

4. **Add `--no-cache` CLI flag**

## Success Criteria
- Cold install ≤ 3.0s (from 4.9s)
- Cache hit avoids HTTP requests entirely (verified via logs)

## References
- `crates/cclab-jet/src/pkg_manager/registry.rs` — disk cache implementation
- #881 (parent issue)

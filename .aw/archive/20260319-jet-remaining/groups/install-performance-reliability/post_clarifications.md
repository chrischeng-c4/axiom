---
change: jet-remaining
group: install-performance-reliability
date: 2026-03-19
status: clarified
---

# Post-Clarifications

## Questions

### Q1: Cache & Stability Strategy
- **Question**: Should we migrate the metadata cache to the proposed .jet-store directory, or stay with the XDG cache path in the existing spec? Also, is #883 fully implemented or does it need re-verification?
- **Answer**: Maintain XDG-compliant cache location. Move forward with additional performance optimizations from #881 to close the gap with pnpm. Re-verify #883 bugs to ensure they are solid.
- **Rationale**: XDG is more standard for caches. #881 is an optimization on top of a partially working parallel implementation. #883 fixes are foundational.

## Contradictions

### C1: jet-pkg-perf-spec vs requirement
- **Spec**: jet-pkg-perf-spec
- **Requirement**: Metadata Cache Location
- **Conflict**: pkg-manager.md R2 specifies a two-layer cache at ~/.cache/jet/metadata/ with 5-min TTL, but #881 proposes a disk-based metadata cache at ~/.jet-store/.metadata/.
- **Resolution**: We will stick to the ~/.cache/jet/metadata/ location (XDG compliant) as specified in pkg-manager.md, but apply the performance optimizations (speculative prefetch, persistent disk cache) mentioned in #881.

### C2: jet-pkg-perf-spec vs requirement
- **Spec**: jet-pkg-perf-spec
- **Requirement**: Parallel Install Maturity
- **Conflict**: pkg-manager.md says parallel install is 'implemented', but #881 identifies a cold install gap of 1.5s vs pnpm that needs further optimization.
- **Resolution**: We will treat #881 as a refinement to the existing implementation, moving from serial BFS to level-by-level parallel prefetch and concurrent downloads.


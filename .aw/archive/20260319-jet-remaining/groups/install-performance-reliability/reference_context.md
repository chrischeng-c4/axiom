---
change: jet-remaining
group: install-performance-reliability
date: 2026-03-19
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| pkg-manager | cclab-jet | high | R1: Parallel Install (3 Phases), R2: Two-layer Metadata Cache (5-min TTL), R3: Lockfile Fast-path, R4: Shasum Verification, R5: Transitive Dep Prefetch (limit=3-5) |
| pkg-manager-pnpm-parity | cclab-jet | high | R1: .npmrc Config Resolution, R2: Frozen Lockfile (depsHash), R3: Optional Dependency Platform Check (Windows/Linux Verification) |


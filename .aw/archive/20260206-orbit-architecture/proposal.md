---
id: orbit-architecture
type: proposal
version: 1
created_at: 2026-02-05T16:12:52.238792+00:00
updated_at: 2026-02-05T16:12:52.238792+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add feature flags, custom slab allocator, and kqueue tuning for orbit crate"
history:
  - timestamp: 2026-02-05T16:12:52.238792+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 6
  new_files: 2
affected_specs:
  - id: feature-flags
    path: specs/feature-flags.md
    depends: []
  - id: slab-allocator
    path: specs/slab-allocator.md
    depends: [feature-flags]
  - id: kqueue-tuning
    path: specs/kqueue-tuning.md
    depends: [feature-flags]
---

<proposal>

# Change: orbit-architecture

## Summary

Add feature flags, custom slab allocator, and kqueue tuning for orbit crate

## Why

The orbit crate currently has minimal modularity - only a single `python` feature flag exists. Users cannot opt out of features they don't need (TLS, DNS, subprocess), leading to unnecessary compile times and binary sizes. Additionally, the timer wheel uses BTreeMap with heap allocations per timer entry, causing allocation pressure under high load. For macOS/BSD deployments, exposing kqueue-specific tuning options can improve performance for latency-sensitive applications.

## What Changes

- Implement modular feature flags: tls, dns, unix-socket, subprocess, kqueue-tuning, slab-allocator
- Create custom Slab<T> allocator for fixed-size allocations (TimerEntry, Handle, ScheduledCallback)
- Add kqueue tuning options for macOS/BSD with optional mio direct access
- Update conditional compilation in lib.rs to respect new feature flags
- Ensure default features preserve current behavior for backwards compatibility

## Impact

- **Scope**: minor
- **Affected Files**: ~6
- **New Files**: ~2
- Affected specs:
  - `feature-flags` (no dependencies)
  - `slab-allocator` → depends on: `feature-flags`
  - `kqueue-tuning` → depends on: `feature-flags`
- Affected code: `crates/cclab-orbit/Cargo.toml`, `crates/cclab-orbit/src/lib.rs`, `crates/cclab-orbit/src/timer_wheel.rs`, `crates/cclab-orbit/src/slab.rs`

</proposal>

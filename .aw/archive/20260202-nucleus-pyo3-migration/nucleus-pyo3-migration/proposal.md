---
id: nucleus-pyo3-migration
type: proposal
version: 1
created_at: 2026-02-01T15:46:40.190073+00:00
updated_at: 2026-02-01T15:46:40.190073+00:00
author: mcp
status: proposed
iteration: 1
summary: "Migrate PyO3 bindings from cclab-nucleus to individual crates and deprecate nucleus"
history:
  - timestamp: 2026-02-01T15:46:40.190073+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-01T15:47:16.074166+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-02-01T15:47:27.518141+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: major
  affected_files: 50
  new_files: 0
affected_specs:
  - id: nucleus-architecture
    path: specs/nucleus-architecture.md
    depends: []---

<proposal>

# Change: nucleus-pyo3-migration

## Summary

Migrate PyO3 bindings from cclab-nucleus to individual crates and deprecate nucleus

## Why

Currently, cclab-nucleus acts as a monolithic crate for all Python bindings, creating a bottleneck and coupling unrelated components. By migrating PyO3 bindings to their respective crates, we improve modularity, build times, and maintainability. This aligns with the new pyo3_bindings convention observed in cclab-shield. cclab-core will host shared utilities like BSON conversion and core types to avoid duplication.

## What Changes

- Update cclab-core to include shared PyO3 conversion logic (conversion.rs, types.rs, config.rs)
- Implement pyo3_bindings module in cclab-nebula, cclab-photon, cclab-ion, cclab-nova, cclab-probe, cclab-meteor
- Migrate respective code from cclab-nucleus to these new modules
- Deprecate and remove cclab-nucleus crate
- Update Python imports to use new crate bindings directly

## Impact

- **Scope**: major
- **Affected Files**: ~50
- **New Files**: ~0
- Affected specs:
  - `nucleus-architecture` (no dependencies)
- Affected code: `crates/cclab-nucleus`, `crates/cclab-core`, `crates/cclab-nebula`, `crates/cclab-photon`, `crates/cclab-ion`, `crates/cclab-nova`, `crates/cclab-probe`, `crates/cclab-meteor`
- **Breaking Changes**: Yes. cclab-nucleus will be removed. Python imports must change from cclab.nucleus.* to specific crate modules (e.g., cclab.nebula, cclab.nova).

</proposal>

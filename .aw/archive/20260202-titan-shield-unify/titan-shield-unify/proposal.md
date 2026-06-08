---
id: titan-shield-unify
type: proposal
version: 1
created_at: 2026-02-02T06:47:39.387086+00:00
updated_at: 2026-02-02T06:47:39.387086+00:00
author: mcp
status: proposed
iteration: 1
summary: "Unify titan with shield for validation by removing duplicated code and using shield as the source of truth."
history:
  - timestamp: 2026-02-02T06:47:39.387086+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-02T06:47:54.887949+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-02-02T06:48:06.873261+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 3
  new_files: 0
affected_specs:
  - id: titan-shield-integration
    path: specs/titan-shield-integration.md
    depends: []---

<proposal>

# Change: titan-shield-unify

## Summary

Unify titan with shield for validation by removing duplicated code and using shield as the source of truth.

## Why

Currently, `cclab-titan` duplicates a significant portion of `cclab-shield`'s validation logic in `pydantic_validation.rs`. This violates the DRY principle and creates a maintenance burden. `cclab-shield` is intended to be the central validation library. Unifying them ensures consistent validation behavior across the ecosystem (`titan`, `nebula`, `shield`) and reduces code debt. `nebula` already uses `shield` patterns, and `titan` should align to provide a consistent experience.

## What Changes

- Add `cclab-shield` dependency to `crates/cclab-titan/Cargo.toml`.
- Delete `crates/cclab-titan/src/pydantic_validation.rs` (approx 750 lines of duplicated code).
- Update `crates/cclab-titan/src/lib.rs` to re-export validation types (`ValidationError`, `ValidationErrors`, etc.) from `cclab-shield`.
- Ensure `titan` aligns with `nebula`'s pattern of using `shield`.

## Impact

- **Scope**: minor
- **Affected Files**: ~3
- **New Files**: ~0
- Affected specs:
  - `titan-shield-integration` (no dependencies)
- Affected code: `crates/cclab-titan/src/pydantic_validation.rs`, `crates/cclab-titan/src/lib.rs`, `crates/cclab-titan/Cargo.toml`
- **Breaking Changes**: Yes. `ValidationErrors::into_result` signature changes from `fn(self, value: T) -> Result<T>` to `fn(self) -> Result<(), ValidationErrors>`. Migration: Change `errors.into_result(value)` to `errors.into_result()?; Ok(value)`.

</proposal>

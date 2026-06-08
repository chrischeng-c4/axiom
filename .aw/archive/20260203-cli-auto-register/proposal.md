---
id: cli-auto-register
type: proposal
version: 1
created_at: 2026-01-31T13:36:52.963331+00:00
updated_at: 2026-01-31T13:36:52.963331+00:00
author: mcp
status: proposed
iteration: 1
summary: "Introduce linkme-based CLI auto-registration so crates self-register subcommands and cclab-cli aggregates them at runtime."
history:
  - timestamp: 2026-01-31T13:36:52.963331+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T13:36:56.067717+00:00
    agent: "codex:deep"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-31T13:37:06.859828+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 11
  new_files: 1
affected_specs:
  - id: cli-auto-register-infra
    path: specs/cli-auto-register-infra.md
    depends: []---

<proposal>

# Change: cli-auto-register

## Summary

Introduce linkme-based CLI auto-registration so crates self-register subcommands and cclab-cli aggregates them at runtime.

## Why

The top-level CLI is centralized in `crates/cclab-cli/src/main.rs`, forcing all command additions and routing to be edited in one file. This bottleneck makes modular growth harder and increases coupling between crates. A registry-based approach lets each crate own its CLI wiring while `cclab-cli` focuses on aggregation and execution, with a gradual migration path to reduce risk.

## What Changes

- Add a small registry module/crate that defines `CliRegistration` and a `linkme` distributed slice for command registration.
- Refactor `crates/cclab-cli/src/main.rs` to build a `clap::Command` tree from registered commands and dispatch based on subcommand name.
- Introduce registration entries in crate-owned CLIs (e.g., `cclab-genesis`, `cclab-server`, and internal `cclab-cli` modules like `api`, `meteor`, `ion`, `qc`, `prism`, `pg`) with an incremental migration path that keeps legacy routing during transition.
- Update workspace and crate `Cargo.toml` files to include `linkme` and wire registry dependencies where commands are registered.

## Impact

- **Scope**: minor
- **Affected Files**: ~11
- **New Files**: ~1
- Affected specs:
  - `cli-auto-register-infra` (no dependencies)
- **Breaking Changes**: None. Registry will coexist with legacy routing during incremental migration.

</proposal>

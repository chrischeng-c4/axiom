---
change_id: cclab-taipan
type: codebase_context
created_at: 2026-02-12T07:35:35.911947+00:00
updated_at: 2026-02-12T07:35:35.911947+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
  - prism_references
---

# Codebase Context

## Analyzed Files

- **crates/cclab-cli/src/registry.rs** — CLI Registration Infrastructure (CliModule trait, CLI_MODULES slice)
  - symbols: `CliModule`, `CLI_MODULES`, `find_module`
- **crates/cclab-cli/src/main.rs** — CLI Entry point and module dispatcher
  - symbols: `Commands enum`, `mod ion; mod probe; mod warp;`
- **crates/cclab-cli/src/ion.rs** — Reference implementation for CliModule registration
  - symbols: `IonCli struct`, `impl CliModule for IonCli`
- **crates/cclab-cli/Cargo.toml** — CLI dependency management
- **Cargo.toml** — Workspace root configuration

## Prism Results

- **prism_symbols** (query: `CliModule`)
  - Trait for CLI modules registered via linkme distributed slice. Defines name, command building, and execution.
- **prism_symbols** (query: `CLI_MODULES`)
  - Distributed slice collecting all registered CLI modules across crates at link time.

## Dependency Graph

- cclab-cli -> cclab-taipan
- cclab-cli -> linkme
- cclab-taipan -> linkme

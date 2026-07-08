---
id: cclab-lens-spec
main_spec_ref: ~
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "This analysis/standardization logic TD supports brownfield semantic coverage and takeover readiness gates."
---

# Cclab Lens Spec

## Overview
<!-- type: doc lang: markdown -->

This specification defines the comprehensive rebranding of the 'prism' code intelligence engine to 'lens'. This includes renaming the core crate, updating all internal and external references, and migrating the Model Model Context Protocol (MCP) toolset to use the 'lens_' prefix. The goal is to provide a more intuitive and consistent naming convention for the project's code analysis and intelligence capabilities.
## Requirements
<!-- type: doc lang: markdown -->

### R1: Crate Renaming
The `crates/cclab-prism` directory MUST be renamed to `crates/cclab-lens`. All `Cargo.toml` files (crate-level and workspace-level) MUST be updated to use `cclab-lens` as the package name.

### R2: Module and Import Updates
All Rust modules and imports using `cclab_prism` MUST be updated to `cclab_lens`. This includes cross-crate dependencies.

### R3: MCP Tool Migration
All Model Context Protocol (MCP) tools prefixed with `prism_` MUST be renamed to use the `lens_` prefix (e.g., `prism_check` becomes `lens_check`).

### R4: Specification and Documentation Rebranding
The `.aw/tech-design/cclab-prism/` directory MUST be renamed to `.aw/tech-design/cclab-lens/`. All occurrences of 'prism' in specification files, READMEs, and the main `CHANGELOG.md` MUST be updated to 'lens' where appropriate.

### R5: Configuration and Environment Alignment
All configuration keys in `.aw/config.toml` or other config files that contain 'prism' MUST be renamed to use 'lens'. Any environment variables used for the code intelligence engine MUST also be updated.

### R6: CLI Command Updates
Any `cclab` CLI subcommands using `prism` MUST be updated to use `lens` (e.g., `cclab prism check` becomes `cclab lens check`).
## Scenarios
<!-- type: doc lang: markdown -->

### Scenario: Crate Name Update
- **WHEN** the crate-level `Cargo.toml` is modified.
- **THEN** the `[package] name` MUST be `cclab-lens`.
- **THEN** the workspace `Cargo.toml` MUST include `crates/cclab-lens` in its members list.

### Scenario: MCP Tool Invocation
- **WHEN** a client (e.g., an LLM or the `cclab` CLI) calls `lens_check` with valid parameters.
- **THEN** the `cclab-server` MUST route the request to the `lens` analysis handler and return the diagnostics result.

### Scenario: Documentation Reference
- **WHEN** a developer views the `README.md` in the `crates/cclab-lens/` directory.
- **THEN** the document MUST refer to the project as 'cclab-lens' and describe its function as the 'lens' code intelligence engine.

### Scenario: Configuration Loading
- **WHEN** the `cclab` server starts and loads its configuration.
- **THEN** it MUST correctly read any `lens` related settings from `.aw/config.toml` (e.g., `[tool.cclab-lens]`).

### Scenario: Cross-Crate Imports
- **WHEN** `cclab-sdd` or `cclab-server` imports types from the analysis crate.
- **THEN** it MUST use `use cclab_lens::...` instead of `use cclab_prism::...`.
## Diagrams
<!-- type: doc lang: markdown -->

## API Spec
<!-- type: doc lang: markdown -->

## Changes
<!-- type: doc lang: markdown -->

### Directory Renames
- Rename `crates/cclab-prism` to `crates/cclab-lens`.
- Rename `.aw/tech-design/cclab-prism` to `.aw/tech-design/cclab-lens`.

### File-Level Updates
- `Cargo.toml` (workspace and crate-level): Replace all 'cclab-prism' with 'cclab-lens'.
- `crates/cclab-lens/src/**/*.rs`: Update all internal crate-level imports and references to 'cclab-prism'.
- `crates/cclab-server/src/**/*.rs`: Update all imports from `cclab_prism` to `cclab_lens` and all MCP tool registrations to use the `lens_` prefix.
- `crates/cclab-cli/src/**/*.rs`: Update all command definitions and tool calls from `prism` to `lens`.
- `.aw/tech-design/**/*.md`: Replace 'prism' and 'cclab-prism' with 'lens' and 'cclab-lens' where appropriate, particularly in `README.md`, `CHANGELOG.md`, and the renamed `cclab-lens` spec directory.
- `CLAUDE.md`, `README.md`, `scripts/rebrand.py`: Update project-wide documentation and utilities.
- `.aw/config.toml`: Update all keys and values referring to 'prism' to 'lens'.

### Tool-Level Updates
- Update MCP tool registration logic in `cclab-server` or `cclab-lens` (whichever defines the tool registry) to use `lens_` as the tool namespace.
- Update any Python-side configurations (e.g., `pyproject.toml`) that used `cclab-prism` settings.

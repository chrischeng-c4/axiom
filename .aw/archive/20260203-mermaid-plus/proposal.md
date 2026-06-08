---
id: mermaid-plus
type: proposal
version: 1
created_at: 2026-01-29T15:02:38.345094+00:00
updated_at: 2026-01-29T15:02:38.345094+00:00
author: mcp
status: proposed
iteration: 1
summary: "Migrate and extend Mermaid+ state machine functionality to cclab-aurora."
history:
  - timestamp: 2026-01-29T15:02:38.345094+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-29T15:04:39.293278+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-29T15:04:49.398539+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 10
  new_files: 1
affected_specs:
  - id: mermaid-plus-format
    path: specs/mermaid-plus-format.md
    depends: []
  - id: mermaid-plus-conversion
    path: specs/mermaid-plus-conversion.md
    depends: []---

<proposal>

# Change: mermaid-plus

## Summary

Migrate and extend Mermaid+ state machine functionality to cclab-aurora.

## Why

To standardize state machine visualization and validation across the Genesis toolchain while avoiding circular dependencies. By moving the generator and data model to cclab-aurora (the base diagram crate) but keeping the IR-dependent parser in cclab-prism, we enable consistent diagram generation and semantic validation across genesis and prism without creating a dependency cycle. This centralizes the 'source of truth' for Mermaid+ in the most appropriate low-level crate.

## What Changes

- Create crates/cclab-aurora/src/diagrams/mermaid_plus.rs to house the standardized StateMachineDef, Generator, and Validator.
- Refactor crates/cclab-prism/src/spec/statemachine/ to depend on cclab-aurora for core Mermaid+ logic.
- Keep MermaidParser in cclab-prism to avoid circular dependencies with Prism IR.
- Update crates/cclab-genesis/src/validator/semantic.rs to use the aurora-based validator.
- Enhance cclab/schemas/spec.schema.json to support the new Mermaid+ composite structure.
- Optimize orchestrator prompts to prioritize the new standardized Mermaid+ format.

## Impact

- **Scope**: minor
- **Affected Files**: ~10
- **New Files**: ~1
- Affected specs:
  - `mermaid-plus-format` (no dependencies)
  - `mermaid-plus-conversion` (no dependencies)
- Affected code: `crates/cclab-aurora/src/diagrams/mermaid_plus.rs`, `crates/cclab-aurora/src/diagrams/state.rs`, `crates/cclab-prism/src/spec/statemachine/mod.rs`, `crates/cclab-genesis/src/validator/semantic.rs`, `crates/cclab-genesis/src/orchestrator/prompts.rs`, `cclab/schemas/spec.schema.json`

</proposal>

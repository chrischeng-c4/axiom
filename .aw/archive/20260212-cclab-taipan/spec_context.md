---
change_id: cclab-taipan
type: spec_context
created_at: 2026-02-12T07:26:23.411052+00:00
updated_at: 2026-02-12T07:26:23.411052+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-aurora
  - cclab-cli
  - cclab-core
  - cclab-prism
  - cclab-probe
---

# Spec Context

## Relevant Specs

- **aurora-codegen-system** (group: cclab-aurora)
  - relevance: high
  - reason: Provides the pattern for code generation and validation pipelines.
  - key sections: Unified Internal Representation, Pluggable Generators, Analysis Pipeline
- **cli-architecture** (group: cclab-cli)
  - relevance: high
  - reason: Required for integrating the 'taipan' command into the unified CLI.
  - key sections: CliModule Registration, Command Dispatch Flow
- **02-architecture-principles** (group: cclab-core)
  - relevance: high
  - reason: Fundamental principles that apply to the compiler implementation.
  - key sections: Performance First, Type Safety
- **prism-class-diagram** (group: cclab-prism)
  - relevance: medium
  - reason: Reference for building a multi-stage analysis pipeline.
  - key sections: Analysis Pipeline
- **00-architecture** (group: cclab-probe)
  - relevance: medium
  - reason: Reference for building test infrastructure for the compiler.
  - key sections: High-Level Architecture, Performance Targets

## Dependencies

- cclab-cli/cli-architecture
- cclab-aurora/aurora-codegen-system

## Gaps

- No spec for Taipan language syntax/grammar.
- No spec for Taipan IR (Intermediate Representation).
- No spec for Cranelift/LLVM backend implementation details.
- No spec for Taipan-specific builtins.

---
change: genesis-372
date: 2026-02-15
---

# Clarifications

## Q1: SpecIR Format
- **Question**: What format should SpecIR use?
- **Answer**: YAML-based manifests, k8s/Kustomize style. Each IR document has apiVersion, kind, metadata, spec fields. Language-agnostic, file-based, declarative.
- **Rationale**: User explicitly proposed k8s/Kustomize style YAML manifests as the SpecIR format. This is a natural fit: declarative, git-friendly, can be read/written by any tool, and eliminates the need for Rust enum coupling.

## Q2: Merge Strategy
- **Question**: Should Aurora be fully merged into Genesis, or kept as a dependency?
- **Answer**: Merge Aurora's spec generation logic into Genesis. Aurora's diagram generation (Mermaid) stays as a separate crate. Genesis directly writes YAML IR files, Prism reads them for codegen.
- **Rationale**: The primary motivation is eliminating token relay: Aurora MCP generates text → agent relays → Genesis consumes. By having Genesis write YAML IR directly, the agent middleman is removed for spec-to-code pipeline.

## Q3: Token Relay Problem
- **Question**: What is the core problem being solved?
- **Answer**: Aurora generates text via MCP, returns to agent, agent passes to Genesis — this wastes tokens on relay. Genesis should produce SpecIR YAML files directly in the change directory, and Prism reads them for codegen.
- **Rationale**: User identified that the agent acting as text middleman between Aurora and Genesis is wasteful. Direct file-based IR eliminates this overhead.

## Q4: IR File Location
- **Question**: Where should SpecIR YAML files be stored?
- **Answer**: Under cclab/changes/<change-id>/spec_ir/ directory, alongside other change artifacts. Each spec produces one or more IR YAML files.
- **Rationale**: Consistent with existing change artifact structure. Files are co-located with proposals, specs, and tasks for the same change.

## Q5: Git Workflow
- **Question**: Which git workflow to use?
- **Answer**: in_place — work on the current sdd branch.
- **Rationale**: User is already working on the sdd branch for this series of changes.


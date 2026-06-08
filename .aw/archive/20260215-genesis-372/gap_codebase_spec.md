---
change_id: genesis-372
type: gap_codebase_spec
created_at: 2026-02-14T17:08:56.388116+00:00
updated_at: 2026-02-14T17:08:56.388116+00:00
---

# Gap Analysis: Codebase vs Spec

## Code without matching spec

### HIGH severity

1. **Aurora generators lack SpecIR integration spec** — `crates/cclab-aurora/src/generators/{fastapi,express,axum}.rs` exist as standalone template-based generators but no spec defines how they should consume SpecIR or integrate with the Prism CodeGenerator trait. (spec-ir-contract covers SpecIR types but not Aurora generator migration)

2. **Genesis create_spec tool lacks SpecIR generation spec** — `crates/cclab-genesis/src/mcp/tools/spec.rs` implements genesis_create_spec but no spec defines how it should generate YAML SpecIR files alongside markdown specs.

### MEDIUM severity

3. **Aurora validator operates on JsonSchema only** — `crates/cclab-aurora/src/validator/completeness.rs` validates JsonSchema but no spec covers validation of other SpecIR types (FlowchartPlus, SequencePlus, etc.) or YAML manifest validation.

4. **Genesis implement codegen routing has no spec for YAML IR consumption** — `crates/cclab-genesis/src/mcp/tools/run_change/implement.rs` ImplementTaskWithCodegen action references Prism MCP tools but no spec defines the YAML IR file format that Prism should read.

## Specs without matching implementation

### HIGH severity

1. **spec-ir-contract (cclab-aurora)** — Defines SpecIR as Rust enum. Current implementation exists but will need to change to YAML format per clarifications. No YAML schema or manifest format is implemented.

2. **prism-codegen-unification (cclab-prism)** — Defines unified CodeGenerator trait with SpecIR input. Partially implemented (can_generate/generate methods exist) but generators still consume typed spec inputs, not YAML IR.

### MEDIUM severity

3. **genesis-implement-integration (cclab-genesis)** — Defines codegen-eligible task routing. Partially implemented (ImplementTaskWithCodegen action exists) but the actual codegen invocation via YAML IR is not connected.

## Summary

| Category | High | Medium | Low |
|----------|------|--------|-----|
| Code without spec | 2 | 2 | 0 |
| Spec without impl | 2 | 1 | 0 |
| **Total gaps** | **4** | **3** | **0** |
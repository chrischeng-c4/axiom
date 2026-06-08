---
change_id: genesis-186-28
type: gap_codebase_spec
created_at: 2026-02-14T03:37:00.504569+00:00
updated_at: 2026-02-14T03:37:00.504569+00:00
---

# Gap Analysis: Codebase vs Spec (genesis-186-28)

## Code without Specification

| File Path | Missing Spec / Requirement | Severity | Description |
|-----------|----------------------------|----------|-------------|
| `crates/cclab-genesis/src/mcp/tools/analyze.rs` | `analyze-code-for-spec` | High | Implementation of `genesis_analyze_code_for_spec` exists but lacks a formal specification. This tool is critical for tree-sitter based analysis of Python, TS, and Rust code to suggest spec structures. |
| `crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs` | `repair-prompt-orchestration` | Medium | Logic for `mainthread_must_fix` prompt generation and verdict parsing is implemented but not explicitly covered in `review-spec` or other planning specs. |

## Specification without Implementation

| Spec ID | Missing Implementation | Severity | Description |
|---------|------------------------|----------|-------------|
| `cclab-genesis/review-spec` | Review Checklist Enforcement | High | The "Review Checklist" section of the spec defines specific validation criteria that are not currently enforced in `mcp/tools/spec.rs` or `services/spec_service.rs`. |
| `cclab-genesis/verdict-unification` | Unified Verdict Enums | Medium | R3 requires unified Rust verdict enums. Current implementation in `helpers.rs` and `Action` enums in `run_change/*.rs` appears fragmented and uses local definitions. |
| `cclab-genesis/create-spec` | --quick flag | Low | Requirement for a `--quick` flag to bypass LLM enrichment is documented in the spec gaps but not yet implemented in the CLI tool arguments. |
| `cclab-aurora/spec-validator` | Completeness Check integration | Medium | While `cclab-aurora` has completeness logic, it is not deeply integrated into the `cclab-genesis` spec review flow as described in the `review-spec` dependencies. |

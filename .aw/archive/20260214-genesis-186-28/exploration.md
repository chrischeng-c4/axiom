---
id: genesis-186-28
type: exploration
created_at: 2026-02-14T03:23:53.691785+00:00
needs_clarification: false
---

# Codebase Exploration

# Exploration: genesis-186-28 (Issue #186 Improvements)

## Codebase Analysis

The Genesis SDD workflow is implemented as a set of MCP tools within the `cclab-genesis` crate. The core logic for code analysis resides in `crates/cclab-genesis/src/mcp/tools/analyze.rs`, which currently uses `tree-sitter` for static analysis of Python, TypeScript, and Rust. Spec creation and validation are handled by `crates/cclab-genesis/src/mcp/tools/spec.rs` and `crates/cclab-genesis/src/services/spec_service.rs`, utilizing rules defined in `crates/cclab-genesis/src/models/spec_rules.rs`. The workflow orchestration, including revision tracking and phase transitions, is managed in `crates/cclab-genesis/src/mcp/tools/run_change/`.

### Relevant Files/Modules:
- `crates/cclab-genesis/src/mcp/tools/analyze.rs`: Implementation of `genesis_analyze_code_for_spec`.
- `crates/cclab-genesis/src/mcp/tools/run_change/spec.rs`: Orchestration of spec creation/review.
- `crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs`: Shared helpers for verdicts and escalation.
- `crates/cclab-genesis/src/services/spec_service.rs`: Service layer for spec creation and diagram rendering via `cclab-aurora`.
- `crates/cclab-genesis/src/models/spec_rules.rs`: Canonical spec validation rules.

## Impact Analysis

The changes will affect the planning and review stages of the Genesis workflow:
- **`analyze_code_for_spec` Tool**: Will be enhanced with LLM enrichment and automated diagram input generation. This will increase token usage but improve spec quality.
- **Spec Review Flow**: Revision thresholds will be strictly enforced, providing a clear escalation path to the mainthread after 4 rejections.
- **CLI Experience**: A new `--quick` flag will allow users to bypass LLM enrichment for faster, AST-only analysis.
- **Backward Compatibility**: Existing specs and workflows will remain compatible, as the improvements augment rather than replace existing structures.

## Technical Considerations

- **LLM Enrichment**: Should leverage the `codebase_context` and any identified gaps to provide semantic depth to suggested requirements and scenarios.
- **Diagram Generation**: Leverage the `cclab-aurora` library's structured inputs to generate diagrams (Sequence, Flowchart, ERD, etc.) directly from extracted AST symbols.
- **--quick Flag**: Must be plumbed from the CLI down to the `analyze` tool's input schema.
- **Review Cycle**: Ensure the `revision_count` in `STATE.yaml` correctly triggers the "mainthread must fix" escalation logic.

## Spec Recommendations

1. **`llm-enrichment-logic`** (`algorithm`): Logic for processing AST data via LLM to generate high-quality requirements and scenarios.
2. **`diagram-auto-generation`** (`algorithm`): Algorithm for mapping AST structures (e.g., Pydantic models, FastAPI routes) to `cclab-aurora` structured diagram inputs.
3. **`review-cycle-thresholds`** (`integration`): Integration of revision counts with strict escalation thresholds and "mainthread must fix" prompt generation.
4. **`quick-flag-implementation`** (`integration`): CLI and MCP plumbing for the `--quick` flag to bypass enrichment.

## Risk Assessment

- **LLM Hallucinations**: Enriched requirements might misinterpret complex code logic. Mitigated by keeping tree-sitter data as the "ground truth" foundation.
- **Token Budget**: Enriched analysis will consume significantly more tokens. Mitigated by the `--quick` flag and efficient prompt design.
- **Tree-sitter Limitations**: Some complex language features might not be captured by current parsers.

## Open Questions

- Should the diagram generation support all 8 diagram types supported by `cclab-aurora`, or focus on the 4 core types (Sequence, Flowchart, Class, ERD) first?
- Should the `--quick` flag be the default in CI environments?


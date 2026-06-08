---
change_id: sdd-merge
type: knowledge_context
created_at: 2026-02-15T03:28:48.451919+00:00
updated_at: 2026-02-15T03:28:48.451919+00:00
iteration: 2
complexity: high
stage: knowledge
scanned_categories:
  - MCP Configuration
  - Spec-to-Code Pipeline architecture
  - Genesis Workflow phases
  - Crate dependency management
---

# Knowledge Context

## Relevant Documents

- **cclab/knowledge/40-mcp/dynamic-config.md**
  - summary: Defines stage-based tool filtering for MCP servers (plan=22, implement=4, review=3, archive=6).
  - relevant sections: Tool Filtering by Stage, Implementation Strategy
- **cclab/knowledge/spec-to-code/code-generator-contract.md**
  - summary: Describes the responsibilities of generators to map agnostic SpecIR to framework-specific code and inference rules.
  - relevant sections: Generator Responsibilities, Inference Rules
- **cclab/knowledge/genesis-372-impact.md**
  - summary: Defines the migration from Aurora text-based relay to direct file-based YAML IR generation in spec_ir/ directory.
  - relevant sections: Executive Summary, Compatibility Matrix, Success Metrics
- **cclab/knowledge/genesis-325-329/gap_codebase_knowledge.md**
  - summary: Identifies critical gaps: generators don't follow spec-agnostic principle and SemanticType mapping is not used for code emission.
  - relevant sections: Convention violations

## Patterns

- **Stage-based MCP filtering** (source: 40-mcp/dynamic-config.md)
  - Expose only stage-appropriate tools to LLMs to reduce cognitive load and token usage. Example: During the 'plan' phase, the 'genesis_create_proposal' tool is available, while 'genesis_implement_task' is hidden.
- **Agnostic SpecIR pipeline** (source: spec-to-code/index.md)
  - Specs describe WHAT, generators decide HOW via language-agnostic intermediate representation. Example: A spec in 'specs/auth.md' is parsed into 'spec_ir/auth.yaml' before generating Rust code.
- **Unified Review Verdicts** (source: cclab/specs/cclab-genesis/verdict-unification.md)
  - Use unified verdicts (APPROVED, REVIEWED, REJECTED) across all review artifacts. Example: Both 'REVIEW_PROPOSAL.md' and 'review_spec_context.md' use the standard 'verdict: APPROVED' field.

## Pitfalls

- Merge logic currently only collects .md files, ignoring spec_ir/*.yaml manifests needed for main specs.
- Rename cclab-genesis to cclab-sdd requires recursive dependency updates in Cargo.toml workspace and all imports.
- Legacy Aurora relay generators (FastAPI, Express, Axum) are still directly coupled to JSON Schema/OpenAPI instead of SpecIR.

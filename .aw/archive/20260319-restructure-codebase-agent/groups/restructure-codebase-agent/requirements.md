---
change: restructure-codebase-agent
group: restructure-codebase-agent
date: 2026-03-19
---

# Requirements

Implement `RestructureCodebaseAgent` in `crates/cclab-agent` — a workspace-aware agentic loop that decomposes a large codebase into manageable groups for fillback spec generation.

The agent follows this flow:
1. Read root manifest (Cargo.toml / package.json / pyproject.toml) via `read_manifest` → workspace members list
2. For each workspace member: call `list_folder_summary(depth=2)` + `estimate_tokens` → per-member token estimate
3. LLM decides grouping under a token budget constraint (single LLM call per iteration)
4. If any group exceeds budget → drill down on that group recursively (repeat steps 2–3 at deeper path)
5. When all groups fit within budget → call `set_grouping(groups=[...])` as terminal action

Tools exposed to the LLM:
- `read_manifest`: parse workspace members from manifest files (Cargo.toml/package.json/pyproject.toml)
- `list_folder_summary`: folder tree + file count + line count at given depth
- `estimate_tokens`: folder path → estimated token count (lines × 3)
- `set_grouping`: write final groups artifact (mandatory terminal call — agent loop ends here)

Key design constraints:
- Workspace-first: `read_manifest` must be called before `list_folder_summary`
- Token budget driven: LLM receives budget as a constraint in the system prompt
- Agentic loop: multiple LLM + tool-call iterations, not one-shot
- Recursive drill-down: oversized groups are decomposed by re-running the loop on sub-paths
- Output consumed by: `ReferenceCodebaseContextAgent` (issue #950) and `CodebaseToSpecAgent` (issue #951)
- Agent is stateless (no internal mutable state), uses existing `LLMProvider` trait

Acceptance criteria:
1. Agent reads workspace manifest and enumerates members correctly
2. Token estimation drives grouping decisions
3. Groups exceeding budget trigger recursive drill-down
4. `set_grouping` produces a valid groups artifact matching the schema expected by `ReferenceCodebaseContextAgent`
5. Unit tests with mock LLM + mock tools covering: single-group workspace, multi-group split, recursive drill-down
6. Agent registered in agent registry alongside existing agents

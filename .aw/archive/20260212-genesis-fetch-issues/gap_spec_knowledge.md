---
change_id: genesis-fetch-issues
type: gap_spec_knowledge
created_at: 2026-02-12T02:37:07.777764+00:00
updated_at: 2026-02-12T02:37:07.777764+00:00
---

# Gap Analysis: Spec vs Knowledge

## Spec Responsibilities vs Knowledge Architecture

### 1. Multi-Project Isolation Requirements
- **Spec**: `fetch-issues` (OpenRPC section)
- **Knowledge Ref**: `40-mcp/http-server.md` (Key Features / Project Isolation via HTTP Headers)
- **Gap**: The `fetch-issues` spec defines the tool interface via OpenRPC but fails to include requirements for respecting the `X-Genesis-Project` and `X-Genesis-Cwd` headers. This is a contradiction to the knowledge base's core architecture for multi-project isolation in the Genesis MCP environment.
- **Severity**: High

### 2. Global Registry Resolution
- **Spec**: `fetch-issues` (Fetch Flow / Dependency Extraction)
- **Knowledge Ref**: `40-mcp/http-server.md` (Key Features / Global Project Registry)
- **Gap**: The specification for fetching issues and extracting dependencies does not account for the `Global Project Registry` (`~/.genesis/registry.json`). Without this, the tool may fail to resolve project-specific contexts correctly in complex environments.
- **Severity**: Medium

## Missing Patterns in Specs

### 1. Stage-Specific Tool Filtering
- **Spec**: `fetch-issues` (Integration with run_change)
- **Knowledge Ref**: `40-mcp/dynamic-config.md` (Tool Filtering by Stage)
- **Gap**: While the spec describes integration with `run_change`, it does not define which workflow stage (Plan, Implement, etc.) the `genesis_fetch_issues` tool should be assigned to. This pattern is central to the knowledge base's strategy for reducing LLM cognitive load.
- **Severity**: Medium

### 2. Agent Skills for Clarification Loops
- **Spec**: `fetch-issues` (Per-Issue Clarification Loop)
- **Knowledge Ref**: `30-claude/skills.md` (Claude Code Agent Skills)
- **Gap**: The spec introduces a per-issue clarification loop but does not specify how this loop interacts with the `Agent Skills` pattern. Documentation on how specialized instructions are provided to agents during this loop is missing.
- **Severity**: Low

## Boundary Misalignments

- **Spec**: `fetch-issues` (STATE.yaml DAG Section)
- **Knowledge Ref**: `40-mcp/dynamic-config.md` (Genesis Integration Strategy)
- **Gap**: The spec places heavy logic for DAG topological sorting and state management within the tool/STATE.yaml interaction. The knowledge base suggests a cleaner separation where stage-specific configs handle the tool visibility, but doesn't fully document how complex multi-step "loops" (like the per-issue clarify loop) should be governed.
- **Severity**: Medium

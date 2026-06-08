---
change: genesis-agent-272-273
date: 2026-02-12
---

# Clarifications

## Q1: Rename
- **Question**: Should genesis_agent be renamed?
- **Answer**: Yes — rename to genesis_delegate_agent. Both MCP tool and skill must be updated together.
- **Rationale**: Better reflects the tool's role as a delegation mechanism for workflow actions, not a generic agent runner.

## Q2: Action routing
- **Question**: How should workflow actions (explore_spec, review_proposal, etc.) be routed?
- **Answer**: Per-action templates. Each workflow action gets its own dedicated prompt template. The run_change prompt builders (e.g., run_change/explore_spec.rs) already exist for this purpose.
- **Rationale**: The original design intent was per-action templates. The 3-action enum (explore/review/custom) was a placeholder that was never expanded.

## Q3: Response format
- **Question**: How should response size be controlled (#273)?
- **Answer**: Artifact-oriented responses. The agent's value is the artifacts it produces (spec_context.md, proposal.md, etc.), not the raw stdout. Response should return verification status + next steps (matching other genesis MCP tools), not raw LLM output. Raw stdout can be logged to file for debugging.
- **Rationale**: #272 and #273 are the same problem: once genesis_delegate_agent is action-aware, it knows what artifact to verify and the response can be structured around that artifact instead of dumping raw output.

## Q4: Complexity
- **Question**: What is the expected complexity level?
- **Answer**: Second most complex tool after run_change. It handles per-action template rendering, multi-provider CLI arg building, artifact verification, telemetry, and structured response formatting.
- **Rationale**: The tool is the execution engine for all agent-delegated workflow steps.

## Q5: Git workflow
- **Question**: Which git workflow to use?
- **Answer**: new_branch: create cclab/genesis-agent-272-273 branch from main
- **Rationale**: Standard feature branch workflow for multi-file changes.


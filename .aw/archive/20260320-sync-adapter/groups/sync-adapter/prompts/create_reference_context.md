# Task: Gather Reference Context for Group 'sync-adapter' (Change 'sync-adapter')

Issues: #959_feat-agent-add-syncadapter-trait-platform-sync-ada

## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to create/modify artifact files directly.**
You MUST use the CLI command below to write the artifact. The system verifies
artifacts were written via CLI — direct file writes will be REJECTED and you
will have to redo the work.

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/sync-adapter/groups/sync-adapter/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. **Write a JSON payload file** then run the CLI command below

## Suggested Sections (from requirements analysis)
Based on keyword analysis of requirements: [overview, schema, state-machine, logic, dependency, interaction, rest-api, async-api, test-plan, changes]
Use these as starting point for spec_plan.sections. Adjust based on your analysis.

## In-Scope Specs

### cclab-agent
- `read_path:specs/cclab-agent/README.md`
- `read_path:specs/cclab-agent/agents.md`
- `read_path:specs/cclab-agent/architecture.md`
- `read_path:specs/cclab-agent/context.md`
- `read_path:specs/cclab-agent/core-types.md`
- `read_path:specs/cclab-agent/error-handling.md`
- `read_path:specs/cclab-agent/fillback-agents.md`
- `read_path:specs/cclab-agent/integrations.md`
- `read_path:specs/cclab-agent/llm-providers.md`
- `read_path:specs/cclab-agent/reference-context-agent.md`
- `read_path:specs/cclab-agent/restructure-agent.md`
- `read_path:specs/cclab-agent/review-agent.md`
- `read_path:specs/cclab-agent/security.md`
- `read_path:specs/cclab-agent/storage.md`
- `read_path:specs/cclab-agent/streaming.md`
- `read_path:specs/cclab-agent/tools.md`
- `read_path:specs/cclab-agent/tools-analysis.md`
- `read_path:specs/cclab-agent/tools-coding.md`


Read these specs using the Read tool (file paths under `/Users/chris.cheng/cclab/cclab-agent/cclab/specs/`).
Do NOT explore specs outside the scope above.

## CLI Commands

```
# Step 1: Write payload JSON file
Write file: cclab/changes/sync-adapter/payloads/create-reference-context.json

# Step 2: Run artifact CLI (MUST use this — do NOT write reference_context.md directly)
cclab sdd artifact create-reference-context sync-adapter cclab/changes/sync-adapter/payloads/create-reference-context.json
```
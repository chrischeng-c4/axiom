---
id: delegate-agent-impl
type: spec
title: "Implement delegate-agent spec: rename, action routing, artifact response"
version: 1
spec_type: algorithm
tags: [logic]
spec_group: cclab-genesis
main_spec_ref: delegate-agent
merge_strategy: patch
created_at: 2026-02-12T11:36:28.909234+00:00
updated_at: 2026-02-12T11:36:28.909234+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Delegate Agent Execution Flow"
changes:
  - file: crates/cclab-genesis/src/mcp/tools/agent.rs
    action: MODIFY
    description: "Rename, expand actions, change response format"
  - file: crates/cclab-genesis/src/mcp/tools/mod.rs
    action: MODIFY
    description: "Rename match arms"
  - file: crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs
    action: MODIFY
    description: "Rename constant"
  - file: crates/cclab-genesis/src/mcp/config.rs
    action: MODIFY
    description: "Update excludeTools/disabled_tools"
  - file: crates/cclab-genesis/src/orchestrator/cli_mapper.rs
    action: MODIFY
    description: "Update DisallowedMcpTools"
  - file: .claude/skills/cclab-genesis-agent/SKILL.md
    action: MODIFY
    description: "Rename skill and update content"
history:
  - timestamp: 2026-02-12T11:36:28.909234+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Implement delegate-agent spec: rename, action routing, artifact response

## Overview

Align genesis_agent code with the delegate-agent spec. Three primary changes: (1) Rename genesis_agent to genesis_delegate_agent across all files — MCP tool definition, registry, helpers constant, config generators, CLI mapper, and skill. (2) Expand action enum from 3 (explore/review/custom) to 30+ workflow actions as defined in delegate-agent spec. Route all workflow actions through custom passthrough since run_change already provides fully-formed prompts. (3) Change response format from raw stdout to artifact-oriented — return {status, change_id, action, verification, usage, next} matching other genesis MCP tools. Raw stdout goes to server log only.

## Requirements

### R1 - Rename genesis_agent to genesis_delegate_agent

```yaml
id: R1
priority: high
status: draft
```

Rename the MCP tool across all references:
- agent.rs: tool name in definition() -> 'genesis_delegate_agent'
- mod.rs: call_tool/call_tool_streaming match arm -> 'genesis_delegate_agent'
- helpers.rs: GENESIS_AGENT_TOOL constant -> 'genesis_delegate_agent'
- config.rs: excludeTools in ensure_gemini_mcp_config(), disabled_tools in ensure_codex_mcp_config()
- cli_mapper.rs: DisallowedMcpTools tool name strings
- Skill: rename .claude/skills/cclab-genesis-agent/ directory and update SKILL.md content
- Tests: update all test assertions referencing old name

### R2 - Expand action enum to support all workflow actions

```yaml
id: R2
priority: high
status: draft
```

Replace the 3-value enum with the full action list from delegate-agent spec. In execute_streaming(), route ALL workflow actions through custom passthrough since run_change provides fully-formed prompts. Keep LLM_EXPLORE and LLM_REVIEW as legacy fallbacks for bare 'explore'/'review' actions.

### R3 - Artifact-oriented response format

```yaml
id: R3
priority: high
status: draft
```

Replace raw stdout response with structured format: {status, change_id, action, verification: {passed, expected_phases, actual_phase, expected_artifact, artifact_exists}, usage, next}. Raw stdout goes to server log only via tx channel.

### R4 - Make change_id required for workflow actions

```yaml
id: R4
priority: medium
status: draft
```

change_id required for all actions except 'custom'. Return error if missing for workflow actions.

### R5 - Update tests

```yaml
id: R5
priority: medium
status: draft
```

Update existing tests for new name. Add tests for action routing, response format, and change_id validation.

## Acceptance Criteria

### Scenario: Workflow action routed through passthrough

- **GIVEN** genesis_delegate_agent called with action='explore_spec'
- **WHEN** execute_streaming processes the action
- **THEN** Prompt passed as-is, verification checks spec_context_created phase

### Scenario: Legacy explore uses template

- **WHEN** action='explore' (bare)
- **THEN** LLM_EXPLORE template wraps the prompt

### Scenario: Response excludes raw stdout

- **GIVEN** Agent produces 500+ lines
- **WHEN** Building response
- **THEN** Response has {status, verification, usage, next} but no raw stdout

### Scenario: Rename propagated

- **WHEN** Build completes
- **THEN** No occurrences of 'genesis_agent' remain as tool name

### Scenario: Missing change_id error

- **WHEN** action='create_proposal' without change_id
- **THEN** Error returned: change_id required for workflow action

## Diagrams

### Delegate Agent Execution Flow

```mermaid
flowchart TB
    start([genesis_delegate_agent called])
    parse[parse_agent_spec(agent)]
    check_action{action type?} 
    explore[LLM_EXPLORE template]
    review[LLM_REVIEW template]
    workflow[passthrough (prompt as-is)]
    build[build_provider_args()]
    run[run_llm_raw_streaming()]
    check_custom{has change_id + verification?} 
    verify[get_verification() + check STATE]
    telemetry[append_telemetry_call()]
    response([return {status, verification, usage, next}])
    start --> parse
    parse --> check_action
    check_action -->|explore| explore
    check_action -->|review| review
    check_action -->|workflow/custom| workflow
    explore --> build
    review --> build
    workflow --> build
    build --> run
    run --> check_custom
    check_custom -->|no| response
    check_custom -->|yes| verify
    verify --> telemetry
    telemetry --> response
```

</spec>

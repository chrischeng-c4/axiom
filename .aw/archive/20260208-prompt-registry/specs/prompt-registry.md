---
id: prompt-registry
type: spec
title: "Prompt Registry - Inline Agent Prompts for run_change"
version: 1
spec_type: algorithm
main_spec_ref: run-change
merge_strategy: patch
created_at: 2026-02-08T15:41:55.222108+00:00
updated_at: 2026-02-08T15:41:55.222108+00:00
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
      title: "agent_prompt Population Flow"
history:
  - timestamp: 2026-02-08T15:41:55.222108+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Prompt Registry - Inline Agent Prompts for run_change

## Overview

Populate the agent_prompt field in genesis_run_change responses for all agent-delegated actions. Delete the separate src/prompts/ directory and inline prompt templates directly into per-stage Rust modules within a new run_change/ folder structure. Each stage file (decide.rs, plan.rs, implement.rs, merge.rs) owns both instructions and agent_prompt generation for its actions.

## Requirements

### R1 - Restructure run_change into folder module

```yaml
id: R1
priority: high
status: draft
```

Convert run_change.rs to run_change/mod.rs. Move decide_change.rs -> run_change/decide.rs, plan_change.rs -> run_change/plan.rs, impl_change.rs -> run_change/implement.rs, merge_change.rs -> run_change/merge.rs. Update mod.rs imports.

### R2 - Populate agent_prompt for all agent-delegated actions

```yaml
id: R2
priority: high
status: draft
```

In each stage's build_response, generate agent_prompt string using format! with change_id, project_path, description. The prompt must include: task description, MCP tools available, expected output format, and artifact creation call. agent_prompt is set only when executor chain contains non-mainthread agents.

### R3 - Delete src/prompts/ directory

```yaml
id: R3
priority: high
status: draft
```

Remove src/prompts/mod.rs and all 12 markdown template files. Content is inlined into stage modules.

### R4 - Migrate task.rs and llm.rs dependencies

```yaml
id: R4
priority: medium
status: draft
```

task.rs and llm.rs currently import from crate::prompts. Move the needed prompt constants (LLM_EXPLORE, LLM_REVIEW, and task-type templates) into the respective files or a shared prompts const block within the new run_change module.

### R5 - Agent prompt includes context artifacts

```yaml
id: R5
priority: medium
status: draft
```

For actions in plan/impl/merge stages, agent_prompt should reference reading context artifacts (clarifications.md, spec_context.md, etc.) and existing artifacts (proposal.md, specs/*.md) so the agent has full context.

## Acceptance Criteria

### Scenario: Agent-delegated action returns populated agent_prompt

- **GIVEN** Change at phase 'decided' with executor ['gemini:pro', 'mainthread'] for create_proposal
- **WHEN** genesis_run_change is called
- **THEN** Response includes agent_prompt with full prompt text containing change_id, project_path, MCP tool instructions, and expected output format. agent_prompt is not null.

### Scenario: Mainthread-only action has no agent_prompt

- **GIVEN** Change at phase 'new' with executor ['mainthread'] for clarify action
- **WHEN** genesis_run_change is called
- **THEN** Response has agent_prompt: null and instructions for mainthread to follow directly.

### Scenario: src/prompts/ directory deleted

- **GIVEN** Implementation complete
- **WHEN** Checking file system
- **THEN** No src/prompts/ directory exists. All prompt content is inline in run_change/*.rs modules.

### Scenario: run_change folder structure

- **GIVEN** Implementation complete
- **WHEN** Checking src/mcp/tools/run_change/
- **THEN** Contains mod.rs, decide.rs, plan.rs, implement.rs, merge.rs. Old *_change.rs files at tools/ level are deleted.

### Scenario: task.rs and llm.rs still compile

- **GIVEN** src/prompts/ deleted
- **WHEN** cargo build
- **THEN** No compilation errors. task.rs and llm.rs use inlined or relocated prompt constants.

## Diagrams

### agent_prompt Population Flow

```mermaid
flowchart TB
    start([build_response(action)])
    check{is agent-delegated?} 
    gen_prompt[Generate agent_prompt via format!]
    set_null[agent_prompt = null]
    ret([Return response])
    start --> check
    check -->|Yes| gen_prompt
    check -->|No| set_null
    gen_prompt --> ret
    set_null --> ret
```

</spec>

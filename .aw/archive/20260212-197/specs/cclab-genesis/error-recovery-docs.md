---
id: error-recovery-docs
type: spec
title: "Error Recovery Documentation"
version: 1
spec_type: utility
spec_group: cclab-genesis
merge_strategy: extend
created_at: 2026-02-12T08:18:22.039126+00:00
updated_at: 2026-02-12T08:18:22.039126+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-12T08:18:22.039126+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Error Recovery Documentation

## Overview

Add Error Recovery sections to delegate-agent.md and run-change/README.md covering 5 scenarios: agent tool call failure recovery with retry policy, genesis_agent verification failure escalation, cyclic dependency fallback in task graph, partial state recovery from mid-task crashes, and concurrent STATE.yaml access safety.

## Requirements

### R1 - Agent failure retry policy in delegate-agent.md

```yaml
id: R1
priority: high
status: draft
```

Add section documenting: mainthread should retry with same agent once on transient failure (exit_code != 0, no state change). If retry fails, try next agent in executor chain. If all agents fail, mainthread executes directly.

### R2 - Verification failure escalation in delegate-agent.md

```yaml
id: R2
priority: high
status: draft
```

Add section documenting: when verification.passed=false (agent ran but didn't produce expected artifact/phase), mainthread should NOT retry blindly. Instead: check actual_phase, if phase advanced partially then resume from current phase via genesis_run_change. If no phase change, treat as agent failure and try next agent.

### R3 - Partial state recovery in run-change/README.md

```yaml
id: R3
priority: high
status: draft
```

Add section documenting: STATE.yaml is the single source of truth. On any crash/interruption, re-calling genesis_run_change with the same change_id will resume from the last committed phase. No rollback needed — the state machine is idempotent.

### R4 - Cyclic dependency fallback in run-change/README.md

```yaml
id: R4
priority: medium
status: draft
```

Add section documenting: Kahn's algorithm detects cycles during task ordering. If cycle detected, implementation falls back to legacy (non-task) path. Mainthread receives error message identifying the cyclic tasks.

### R5 - User intervention hooks in run-change/README.md

```yaml
id: R5
priority: medium
status: draft
```

Add section documenting: when revision_count >= MAX (2), workflow auto-approves and advances. For terminal failures (rejected phase), mainthread must present the error to user and allow manual fix + phase reset via genesis_update_state.

## Acceptance Criteria

### Scenario: Agent transient failure triggers retry

- **GIVEN** genesis_agent returns exit_code != 0 with no state change
- **WHEN** Mainthread receives error response
- **THEN** Retry once with same agent, then try next agent in executor chain

### Scenario: Verification failure with partial progress

- **GIVEN** genesis_agent verification.passed=false but actual_phase advanced
- **WHEN** Mainthread receives verification failure
- **THEN** Resume from actual_phase via genesis_run_change (do not retry the same action)

### Scenario: Crash recovery via state machine

- **GIVEN** Agent or mainthread crashes mid-task
- **WHEN** Session resumes and calls genesis_run_change
- **THEN** Workflow resumes from last committed STATE.yaml phase

### Scenario: Cyclic dependency detected

- **GIVEN** tasks.md contains circular depends_on references
- **WHEN** Implementation phase begins task ordering
- **THEN** Falls back to legacy implementation path with error message

### Scenario: Terminal failure requires user intervention

- **GIVEN** Task reaches revision limit with non-APPROVED verdict
- **WHEN** genesis_run_change returns task_terminal_failure
- **THEN** Mainthread presents error and waits for user to manually fix + reset phase

</spec>

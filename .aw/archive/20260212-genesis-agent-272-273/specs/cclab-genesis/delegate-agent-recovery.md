---
id: delegate-agent-recovery
type: spec
title: "Error recovery: retry + fallback chain for delegate-agent"
version: 1
spec_type: algorithm
tags: [logic]
spec_group: cclab-genesis
main_spec_ref: delegate-agent
merge_strategy: patch
created_at: 2026-02-12T11:36:55.290730+00:00
updated_at: 2026-02-12T11:36:55.290730+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Error Recovery Flow"
depends:
  - delegate-agent-impl
changes:
  - file: crates/cclab-genesis/src/mcp/tools/agent.rs
    action: MODIFY
    description: "Add retry logic around run_llm_raw_streaming call, add pre-execution phase snapshot, differentiate verification failure types"
history:
  - timestamp: 2026-02-12T11:36:55.290730+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Error recovery: retry + fallback chain for delegate-agent

## Overview

Implement the error recovery section of the delegate-agent spec. When an agent fails (crash, timeout, rate limit), retry once with the same agent. If retry fails, the response should indicate failure so the mainthread caller can try the next agent in the executor chain. For verification failures (agent ran but wrong state), do NOT retry — return verification result so run_change can route to the correct next action from the current state.

## Requirements

### R1 - Retry on transient failure

```yaml
id: R1
priority: high
status: draft
```

When run_llm_raw_streaming returns non-zero exit code or times out, retry ONCE with the same agent. If retry also fails, return status='error' with error details. Do not retry if agent ran successfully (exit_code=0) but verification failed.

### R2 - Verification failure handling

```yaml
id: R2
priority: high
status: draft
```

When verification.passed=false but agent ran successfully (exit_code=0):
- If actual_phase == previous_phase (no state change): treat as agent failure, status='error'
- If actual_phase != expected but different from previous: status='ok' with verification.passed=false, let run_change handle from new state
- Never retry when state has changed — state machine is source of truth

### R3 - Structured error response

```yaml
id: R3
priority: medium
status: draft
```

Error responses must include: {status: 'error', change_id, action, error: {type: 'agent_failure'|'timeout'|'verification_failed', message, retried: bool}, usage (if available), next: [{tool: 'genesis_run_change', args}]}

## Acceptance Criteria

### Scenario: Transient failure retried successfully

- **GIVEN** Agent exits with non-zero code on first attempt
- **WHEN** execute_streaming retries
- **THEN** Second attempt succeeds, returns normal response with usage from both attempts

### Scenario: Both attempts fail

- **GIVEN** Agent fails twice
- **WHEN** execute_streaming exhausts retry
- **THEN** Returns status='error' with retried=true and next pointing to run_change

### Scenario: Verification failed with state change

- **GIVEN** Agent ran but phase is 'spec_context_reviewed' instead of expected 'spec_context_created'
- **WHEN** Verification checks
- **THEN** Returns status='ok', verification.passed=false, next=run_change (let state machine route)

### Scenario: Verification failed no state change

- **GIVEN** Agent ran but phase unchanged from before
- **WHEN** Verification checks
- **THEN** Returns status='error', type='verification_failed' (treat as agent failure)

## Diagrams

### Error Recovery Flow

```mermaid
flowchart TB
    run[run_llm_raw_streaming()]
    check_exit{exit_code == 0?} 
    retry_check{already retried?} 
    retry[retry once]
    error_resp([return status=error])
    verify[run verification]
    v_pass{verification passed?} 
    ok_resp([return status=ok, passed=true])
    state_changed{state changed?} 
    partial([return ok, passed=false])
    run --> check_exit
    check_exit -->|no| retry_check
    check_exit -->|yes| verify
    retry_check -->|no| retry
    retry_check -->|yes| error_resp
    retry --> run
    verify --> v_pass
    v_pass -->|yes| ok_resp
    v_pass -->|no| state_changed
    state_changed -->|yes| partial
    state_changed -->|no| error_resp
```

</spec>

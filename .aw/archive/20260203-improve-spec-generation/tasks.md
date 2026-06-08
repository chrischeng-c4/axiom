---
id: improve-spec-generation
change_id: improve-spec-generation
type: tasks
version: 1
created_at: 2026-01-27T05:14:17.092406+00:00
updated_at: 2026-01-27T05:14:17.092406+00:00
proposal_ref: improve-spec-generation
summary:
  total: 5
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 5
layers:
  logic:
    task_count: 1
    estimated_files: 0
  integration:
    task_count: 3
    estimated_files: 0
  testing:
    task_count: 1
    estimated_files: 1
history:
  - timestamp: 2026-01-27T05:14:17.092406+00:00
    agent: "mcp"
    tool: "create_tasks"
    action: "created"
  - timestamp: 2026-01-27T05:14:23.812902+00:00
    agent: "gemini-3-flash-preview"
    tool: "create_tasks"
    action: "created"
    duration_secs: 46.29
  - timestamp: 2026-01-27T05:15:04.572006+00:00
    agent: "gpt-5.2-codex"
    tool: "review_tasks"
    action: "reviewed"
    duration_secs: 40.76---

<tasks>

# Implementation Tasks

## Overview

This document outlines 5 implementation tasks for change `improve-spec-generation`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 1 | 🔲 Pending |
| Integration Layer | 3 | 🔲 Pending |
| Testing Layer | 1 | 🔲 Pending |

## 2. Logic Layer

### Task 2.1: Centralize Spec Type Guidance in Orchestrator

```yaml
id: 2.1
action: MODIFY
status: pending
file: crates/cclab-genesis/src/orchestrator/prompts.rs
spec_ref: spec-generation-improvement:R2
```

Update orchestrator/prompts.rs to derive spec_type-specific guidance (Required Diagrams, Required API Spec) from the central SpecType enum in models/spec_rules.rs instead of hardcoded strings.

## 3. Integration Layer

### Task 3.1: Enhance Create Spec Template with Formal Examples

```yaml
id: 3.1
action: MODIFY
status: pending
file: crates/cclab-genesis/templates/prompts/create_spec.md
spec_ref: spec-generation-improvement:R1
depends_on: [2.1]
```

Add detailed JSON/YAML examples for OpenAPI 3.1, AsyncAPI 2.6, OpenRPC 1.3, and Serverless Workflow 0.8 to the create_spec.md prompt template. Ensure examples include realistic paths, channels, and methods.

### Task 3.2: Sync Revise Spec Template with Formal Examples

```yaml
id: 3.2
action: MODIFY
status: pending
file: crates/cclab-genesis/templates/prompts/revise_spec.md
spec_ref: spec-generation-improvement:R1
depends_on: [3.1]
```

Sync the revise_spec.md template with the new formal language examples and instructions added to create_spec.md.

### Task 3.3: Refactor Validation Tool to Use Central Rules

```yaml
id: 3.3
action: MODIFY
status: pending
file: crates/cclab-genesis/src/mcp/tools/validate_spec.rs
spec_ref: spec-generation-improvement:R3
depends_on: [2.1]
```

Refactor the validate_spec_completeness tool to use the SpecType enum and its required_api_spec/required_diagrams methods for validation instead of duplicated hardcoded logic.

## 4. Testing Layer

### Task 4.1: Verify Strict Validation Enforcement

```yaml
id: 4.1
action: CREATE
status: pending
file: crates/cclab-genesis/tests/validation_test.rs
spec_ref: spec-generation-improvement:R3
depends_on: [2.1, 3.1, 3.2, 3.3]
```

Verify that all spec types (http-api, rpc-api, event-driven, workflow, data-model, algorithm) correctly fail validation when their respective formal specs or diagrams are missing.

</tasks>

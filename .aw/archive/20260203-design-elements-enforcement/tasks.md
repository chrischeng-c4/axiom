---
id: design-elements-enforcement
change_id: design-elements-enforcement
type: tasks
version: 1
created_at: 2026-01-26T10:37:25.144243+00:00
updated_at: 2026-01-26T10:37:25.144243+00:00
proposal_ref: design-elements-enforcement
summary:
  total: 9
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 9
layers:
  data:
    task_count: 1
    estimated_files: 0
  logic:
    task_count: 2
    estimated_files: 0
  integration:
    task_count: 4
    estimated_files: 0
  testing:
    task_count: 2
    estimated_files: 0
history:
  - timestamp: 2026-01-26T10:37:25.144243+00:00
    agent: "mcp"
    tool: "create_tasks"
    action: "created"
  - timestamp: 2026-01-26T10:37:36.438940+00:00
    agent: "gemini-3-flash-preview"
    tool: "create_tasks"
    action: "created"
    duration_secs: 108.25
  - timestamp: 2026-01-26T10:38:13.560789+00:00
    agent: "gpt-5.2-codex"
    tool: "review_tasks"
    action: "reviewed"
    duration_secs: 37.12---

<tasks>

# Implementation Tasks

## Overview

This document outlines 9 implementation tasks for change `design-elements-enforcement`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Data Layer | 1 | 🔲 Pending |
| Logic Layer | 2 | 🔲 Pending |
| Integration Layer | 4 | 🔲 Pending |
| Testing Layer | 2 | 🔲 Pending |

## 1. Data Layer

### Task 1.1: Move enums and logic to spec_rules.rs

```yaml
id: 1.1
action: MODIFY
status: pending
file: crates/cclab-genesis/src/models/spec_rules.rs
spec_ref: spec-enforcement-rules:R1
```

Move ApiSpecType and SpecType enums and their associated logic (from_str, as_str, required_diagrams, required_api_spec) from spec_service.rs to spec_rules.rs. This centralizes the specification rules as a single source of truth.

## 2. Logic Layer

### Task 2.1: Refactor spec_service.rs to use centralized rules

```yaml
id: 2.1
action: MODIFY
status: pending
file: crates/cclab-genesis/src/services/spec_service.rs
spec_ref: spec-enforcement-rules:R1
depends_on: [data.1]
```

Update spec_service.rs to use enums and logic from crate::models::spec_rules. Remove local enum definitions for ApiSpecType and SpecType to ensure all services use the centralized rules.

### Task 2.2: Update SemanticValidator with completeness enforcement

```yaml
id: 2.2
action: MODIFY
status: pending
file: crates/cclab-genesis/src/validator/semantic.rs
spec_ref: validator-enhancement:R1,R2,R3,R4
depends_on: [logic.1]
```

Implement spec completeness validation in SemanticValidator. Update it to parse spec_type from frontmatter and verify the presence of required Mermaid diagrams and API specs based on the centralized rules.

## 3. Integration Layer

### Task 3.1: Update create_spec tool with rule enforcement

```yaml
id: 3.1
action: MODIFY
status: pending
file: crates/cclab-genesis/src/mcp/tools/spec.rs
spec_ref: validator-enhancement:R6
depends_on: [logic.2]
```

Update create_spec tool to use the centralized rules for validation. Ensure it provides specific error messages when required diagrams or API specs are missing for a given spec_type.

### Task 3.2: Refactor validate_spec_completeness tool

```yaml
id: 3.2
action: MODIFY
status: pending
file: crates/cclab-genesis/src/mcp/tools/validate_spec.rs
spec_ref: validator-enhancement:R5
depends_on: [integration.1]
```

Refactor validate_spec_completeness tool to delegate its validation logic to the enhanced SemanticValidator. This ensures that the tool and the validator provide consistent results.

### Task 3.3: Update get_task tool with guidance enhancement

```yaml
id: 3.3
action: MODIFY
status: pending
file: crates/cclab-genesis/src/mcp/tools/task.rs
spec_ref: validator-enhancement:R7
depends_on: [integration.2]
```

Update get_task tool to use the centralized rules to provide spec_type-specific guidance in task instructions, helping agents understand what design elements are required.

### Task 3.4: Align orchestrator prompts with centralized rules

```yaml
id: 3.4
action: MODIFY
status: pending
file: crates/cclab-genesis/src/orchestrator/prompts.rs
spec_ref: validator-enhancement:R7
depends_on: [integration.3]
```

Align orchestrator prompt templates in prompts.rs with the centralized spec rules. Update get_spec_type_guidance and related prompts to ensure agents receive correct information about required elements.

## 4. Testing Layer

### Task 4.1: Add unit tests for centralized rules

```yaml
id: 4.1
action: MODIFY
status: pending
file: crates/cclab-genesis/src/models/spec_rules.rs
spec_ref: spec-enforcement-rules:R1,R2,R3
depends_on: [data.1]
```

Add comprehensive unit tests to spec_rules.rs to verify the correct mapping of spec types to required diagrams and API specs.

### Task 4.2: Add integration tests for SemanticValidator enforcement

```yaml
id: 4.2
action: MODIFY
status: pending
file: crates/cclab-genesis/src/validator/semantic.rs
spec_ref: validator-enhancement:R1,R2,R3,R4
depends_on: [logic.2]
```

Add integration tests for SemanticValidator to verify enforcement of completeness rules across various spec_type scenarios (http-api, data-model, utility, etc.).

</tasks>

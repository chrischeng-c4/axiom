---
id: run-change-skill
type: spec
title: "Update run-change spec — thresholds, phases, scopes, actions, executor"
version: 1
spec_type: utility
created_at: 2026-02-23T16:50:43.161900+00:00
updated_at: 2026-02-23T16:50:43.161900+00:00
requirements:
  total: 7
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
    - R7
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-23T16:50:43.161900+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Update run-change spec — thresholds, phases, scopes, actions, executor

## Overview

Updates the run-change skill spec to document multiple implementation details that diverge from spec: (1) REVIEWED threshold auto-approves at revision_count>=2 instead of mainthread eval (#472), (2) StatePhase uses Clarified instead of separate ClarificationsCreated (#473), (3) scope names use shortened forms like 'clarifications' instead of 'context_clarifications' (#474), (4) revision thresholds are reviewed:2/rejected:4 not spec's 1/2 (#479), (5) undocumented action labels, executor field semantics, and DAG counter ownership (#480).

## Requirements

### R1 - Document auto-approve at REVIEWED threshold

```yaml
id: R1
priority: medium
status: draft
```

Spec must document that when revision_count >= threshold (2 for REVIEWED), the system auto-approves rather than asking mainthread to evaluate. Addresses #472.

### R2 - Document actual StatePhase enum

```yaml
id: R2
priority: medium
status: draft
```

Spec must use the actual StatePhase enum values from implementation including legacy 'Clarified' instead of the proposed 'ClarificationsCreated'. Addresses #473.

### R3 - Document actual scope names

```yaml
id: R3
priority: medium
status: draft
```

Spec must use the implementation's shortened scope names (e.g. 'clarifications' not 'context_clarifications'). Addresses #474.

### R4 - Document actual revision thresholds

```yaml
id: R4
priority: medium
status: draft
```

Spec must document reviewed_threshold=2 and rejected_threshold=4 (not spec's 1 and 2). Addresses #479.

### R5 - Document all action labels

```yaml
id: R5
priority: medium
status: draft
```

Spec must enumerate all action labels emitted by sdd_run_change including those currently undocumented. Addresses #480.

### R6 - Document executor field

```yaml
id: R6
priority: medium
status: draft
```

Spec must document the executor field in sdd_run_change response and its semantics (who should execute the prompt). Addresses #480.

### R7 - Document DAG counter ownership

```yaml
id: R7
priority: medium
status: draft
```

Spec must document which component owns DAG counter increments (clarify_index, context_index). Addresses #480.

## Acceptance Criteria

### Scenario: Auto-approve at threshold

- **GIVEN** A review returns REVIEWED verdict and revision_count=2
- **WHEN** System checks threshold
- **THEN** Phase auto-advances to approved without mainthread evaluation

### Scenario: Rejection threshold

- **GIVEN** A review returns REJECTED verdict and revision_count=4
- **WHEN** System checks threshold
- **THEN** Phase auto-advances to approved (force-approve) to prevent infinite loops

</spec>

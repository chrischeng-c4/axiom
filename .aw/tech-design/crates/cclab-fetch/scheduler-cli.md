---
id: scheduler-cli
type: spec
title: "Meteor Scheduler CLI Commands"
version: 1
spec_type: utility
spec_group: cclab-meteor
created_at: 2026-02-03T08:35:20.885610+00:00
updated_at: 2026-02-03T08:35:20.885610+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-03T08:35:20.885610+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Meteor Scheduler CLI Commands

## Overview

This specification defines the CLI commands for managing the Meteor periodic scheduler. It allows users to list, pause, resume, and manually trigger scheduled tasks.

## Requirements

### R1 - List Scheduled Tasks

```yaml
id: R1
priority: medium
status: draft
```

Display all registered periodic tasks, their schedules, and current status (active/paused).

### R2 - Pause/Resume Tasks

```yaml
id: R2
priority: medium
status: draft
```

Allow users to pause or resume specific tasks at runtime.

### R3 - Manually Trigger Tasks

```yaml
id: R3
priority: medium
status: draft
```

Provide a command to manually trigger a task immediately, bypassing its schedule.

## Acceptance Criteria

### Scenario: Listing Tasks

- **GIVEN** A scheduler with three registered tasks.
- **WHEN** User runs 'cc meteor schedule list'.
- **THEN** The CLI displays a table with task names, schedules, and statuses.

### Scenario: Pausing a Task

- **GIVEN** An active task 'daily-report'.
- **WHEN** User runs 'cc meteor schedule pause daily-report'.
- **THEN** The task status is updated to 'paused' and it stops running according to its schedule.

### Scenario: Manually Triggering a Task

- **GIVEN** A task 'cleanup-cache'.
- **WHEN** User runs 'cc meteor schedule trigger cleanup-cache'.
- **THEN** The task is immediately enqueued to the broker.

</spec>

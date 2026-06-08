---
id: debug-api
type: spec
title: "Debug Mode Python API"
version: 1
spec_type: utility
spec_group: orbit
created_at: 2026-02-05T13:47:33.260+00:00
updated_at: 2026-02-05T13:47:33.260+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-05T13:47:33.260+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Debug Mode Python API

## Overview

Expose the existing DebugMonitor statistics to Python via a new get_debug_stats() method on PyLoop. The Rust implementation already tracks callbacks, slow callbacks, task counts, and loop iterations. This spec defines the Python API surface and the data format returned to users.

## Requirements

### R1 - get_debug_stats() method

```yaml
id: R1
priority: high
status: draft
```

Add get_debug_stats() method to PyLoop that returns a dictionary with loop statistics including callbacks_executed, slow_callbacks_count, tasks_created, tasks_completed, tasks_cancelled, pending_tasks, and iterations.

### R2 - Slow callback threshold configuration

```yaml
id: R2
priority: medium
status: draft
```

Add set_slow_callback_duration(seconds: float) method to configure the threshold for slow callback detection. Default is 100ms matching asyncio.

### R3 - Get slow callbacks list

```yaml
id: R3
priority: medium
status: draft
```

Add get_slow_callbacks() method returning a list of recent slow callback records with name, duration_ms, and timestamp.

### R4 - Reset statistics

```yaml
id: R4
priority: low
status: draft
```

Add reset_debug_stats() method to clear accumulated statistics and slow callback history.

## Acceptance Criteria

### Scenario: Enable debug and get stats

- **GIVEN** A new PyLoop instance
- **WHEN** User calls loop.set_debug(True) then loop.get_debug_stats()
- **THEN** Returns dict with all statistic fields initialized to 0

### Scenario: Track slow callback

- **GIVEN** Debug mode enabled with 50ms threshold
- **WHEN** A callback runs for 100ms
- **THEN** get_slow_callbacks() returns list containing that callback record

### Scenario: Stats accumulate across callbacks

- **GIVEN** Debug mode enabled
- **WHEN** 10 callbacks execute
- **THEN** get_debug_stats()['callbacks_executed'] equals 10

</spec>

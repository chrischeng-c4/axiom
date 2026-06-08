---
id: mamba-stdlib-datetime
type: spec
title: "Stdlib: datetime and time"
version: 1
spec_type: utility
created_at: 2026-02-22T11:20:18.736274+00:00
updated_at: 2026-02-22T11:20:18.736274+00:00
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
  - timestamp: 2026-02-22T11:20:18.736274+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Stdlib: datetime and time

## Overview

Implement the datetime stdlib module for Mamba runtime. Provides datetime.now(), datetime constructor, date.today(), timedelta, strftime formatting, and timestamp conversion. Uses std::time::SystemTime for current time.

## Requirements

### R1 - datetime module registration

```yaml
id: R1
priority: high
status: draft
```

Create datetime_mod.rs with register(). Register module 'datetime' with attrs for now, today, strftime, timestamp, timedelta, datetime constructor.

### R2 - datetime.now()

```yaml
id: R2
priority: high
status: draft
```

mb_datetime_now: return dict {year, month, day, hour, minute, second, microsecond} representing current UTC time.

### R3 - datetime constructor

```yaml
id: R3
priority: high
status: draft
```

mb_datetime_new(year, month, day, hour, min, sec): create datetime dict from components.

### R4 - date.today()

```yaml
id: R4
priority: medium
status: draft
```

mb_date_today: return dict {year, month, day} for current date.

### R5 - timedelta

```yaml
id: R5
priority: medium
status: draft
```

mb_timedelta_new(days, seconds): create timedelta dict. Support addition with datetime.

### R6 - strftime

```yaml
id: R6
priority: medium
status: draft
```

mb_datetime_strftime(dt, fmt): format datetime dict as string using format codes (%Y, %m, %d, %H, %M, %S).

### R7 - timestamp

```yaml
id: R7
priority: low
status: draft
```

mb_datetime_timestamp(dt): convert datetime dict to Unix timestamp float.

## Acceptance Criteria

### Scenario: now returns current time

- **WHEN** datetime.now()
- **THEN** Returns dict with year >= 2026

### Scenario: strftime formats correctly

- **GIVEN** dt = datetime(2026, 1, 15, 10, 30, 0)
- **WHEN** dt.strftime('%Y-%m-%d')
- **THEN** Returns '2026-01-15'

</spec>

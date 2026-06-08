---
id: tuning-guide
type: spec
title: "Performance Tuning Guide"
version: 1
spec_type: utility
spec_group: orbit
created_at: 2026-02-05T13:48:10.433688+00:00
updated_at: 2026-02-05T13:48:10.433688+00:00
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
  - timestamp: 2026-02-05T13:48:10.433688+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Performance Tuning Guide

## Overview

Enhance the existing performance-tuning.md knowledge document with concrete benchmark results, configuration examples, profiling instructions, and production deployment recommendations. Target audience is users deploying orbit in production.

## Requirements

### R1 - Benchmark comparison table

```yaml
id: R1
priority: high
status: draft
```

Add table comparing orbit vs uvloop vs asyncio for timer, TCP, and task benchmarks with actual numbers from the benchmark suite.

### R2 - Configuration examples

```yaml
id: R2
priority: high
status: draft
```

Provide concrete Python code examples for common tuning scenarios: high-throughput server, low-latency service, resource-constrained environment.

### R3 - Profiling guide

```yaml
id: R3
priority: medium
status: draft
```

Document how to profile orbit applications using py-spy, tokio-console, and the debug mode stats API.

### R4 - Production checklist

```yaml
id: R4
priority: medium
status: draft
```

Provide checklist for production deployment: OS tuning, file descriptor limits, graceful shutdown, monitoring.

### R5 - Troubleshooting section

```yaml
id: R5
priority: low
status: draft
```

Document common issues (slow callbacks, high memory, connection errors) and their solutions.

## Acceptance Criteria

### Scenario: User tunes for throughput

- **GIVEN** User deploying high-throughput TCP server
- **WHEN** They follow the tuning guide
- **THEN** They achieve >100k req/sec with proper configuration

### Scenario: User profiles slow callback

- **GIVEN** User has slow callback warnings in debug mode
- **WHEN** They follow the profiling guide
- **THEN** They identify the bottleneck using py-spy

</spec>

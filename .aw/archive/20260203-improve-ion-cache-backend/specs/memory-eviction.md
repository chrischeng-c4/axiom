---
id: memory-eviction
type: spec
title: "Memory Eviction"
version: 1
spec_type: algorithm
created_at: 2026-01-30T06:11:17.939439+00:00
updated_at: 2026-01-30T06:11:17.939439+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Eviction Flow"
history:
  - timestamp: 2026-01-30T06:11:17.939439+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Memory Eviction

## Overview

Defines memory management policies to prevent Out-Of-Memory (OOM) conditions. Supports configuring a maximum memory limit and eviction strategies (LRU, LFU, Volatile) to free space when the limit is reached.

## Requirements

### R1 - Memory Limit

```yaml
id: R1
priority: medium
status: draft
```

Implement `maxmemory` configuration to set a limit on memory usage.

### R2 - Eviction Policies

```yaml
id: R2
priority: medium
status: draft
```

Implement eviction policies: `allkeys-lru`, `volatile-lru`, `allkeys-lfu`, `noeviction`.

### R3 - Eviction Trigger

```yaml
id: R3
priority: medium
status: draft
```

Trigger eviction loop when memory usage exceeds limit during write operations.

### R4 - Usage Tracking

```yaml
id: R4
priority: medium
status: draft
```

Track LRU (Last Recently Used) and LFU (Least Frequently Used) metadata for keys.

## Acceptance Criteria

### Scenario: Evict LRU

- **WHEN** Maxmemory reached, policy is allkeys-lru, 'A' is least recently used, client sets 'C'
- **THEN** Key 'A' is removed to make space for 'C'

### Scenario: No Eviction OOM

- **WHEN** Maxmemory reached, policy is noeviction, client sets 'C'
- **THEN** Returns OOM error

### Scenario: Evict Volatile Only

- **WHEN** Maxmemory reached, policy is volatile-lru, 'B' has TTL, 'A' does not
- **THEN** Key 'B' (with TTL) is removed, 'A' (persistent) is kept

## Diagrams

### Eviction Flow

```mermaid
flowchart TB
    Start(Command)
    CheckMem{Check Usage} 
    Proceed(Execute Op)
    SelectPolicy{Policy} 
    Error[OOM Error]
    Sample[Sample Keys]
    Evict[Delete Key]
    Start -->|Write Op| CheckMem
    CheckMem -->|< maxmemory| Proceed
    CheckMem -->|>= maxmemory| SelectPolicy
    SelectPolicy -->|noeviction| Error
    SelectPolicy -->|LRU/LFU| Sample
    Sample -->|Found Candidate| Evict
    Sample -->|No Candidate| Error
    Evict -->|Memory Freed| CheckMem
```

</spec>

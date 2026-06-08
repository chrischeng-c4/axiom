---
id: pulsar-jieba-integration
type: spec
title: "Pulsar Jieba Integration"
version: 1
spec_type: integration
created_at: 2026-01-30T04:35:07.044795+00:00
updated_at: 2026-01-30T04:35:07.044795+00:00
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
    - type: sequence
      title: "Python Integration Flow"
history:
  - timestamp: 2026-01-30T04:35:07.044795+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Pulsar Jieba Integration

## Overview

This specification defines how `cclab-pulsar-jieba` is integrated into the broader cclab ecosystem, specifically through Python bindings in `cclab-nucleus` and documentation in the crate map.

## Requirements

### R1 - Python Bindings via Nucleus

```yaml
id: R1
priority: medium
status: draft
```

Expose `cclab-pulsar-jieba` functionality to Python via `cclab-nucleus` under the `pulsar` feature.

### R2 - Idiomatic Python API

```yaml
id: R2
priority: medium
status: draft
```

Ensure Python-side API is idiomatic (e.g., using generators or lists of dictionaries/objects).

### R3 - Crate Map Update

```yaml
id: R3
priority: medium
status: draft
```

Update `cclab/specs/crate-map.md` to reflect the addition of `cclab-pulsar-jieba` under the Pulsar category.

## Acceptance Criteria

### Scenario: Python Tokenization Call

- **WHEN** A user calls `cclab.jieba.tokenize("...")` from Python.
- **THEN** The result should be a list of Python token objects with word and offset attributes.

## Diagrams

### Python Integration Flow

```mermaid
sequenceDiagram
    actor Python as Python Client
    participant Nucleus as cclab-nucleus (PyO3)
    participant PulsarJieba as cclab-pulsar-jieba
    Python->>Nucleus: import cclab_nucleus as ob; ob.jieba.tokenize("...")
    Nucleus->>PulsarJieba: jieba_segmenter.tokenize("...")
    PulsarJieba->>Nucleus: Vec<Token>
    Nucleus->>Python: List of Python objects
```

</spec>

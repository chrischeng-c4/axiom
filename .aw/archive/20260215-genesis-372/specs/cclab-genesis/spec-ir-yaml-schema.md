---
id: spec-ir-yaml-schema
type: spec
title: "SpecIR YAML Manifest Schema"
version: 1
spec_type: data-model
tags: [data]
spec_group: cclab-genesis
created_at: 2026-02-14T17:26:03.550473+00:00
updated_at: 2026-02-14T17:26:03.550473+00:00
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
  has_api_spec: true
  has_semantic_diagrams: false
  api_spec_type: json-schema
  diagrams:
    - type: class
      title: "SpecIR Manifest Model"
history:
  - timestamp: 2026-02-14T17:26:03.550473+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# SpecIR YAML Manifest Schema

## Overview

Defines the YAML schema for SpecIR manifests, following a Kubernetes-style resource model (apiVersion, kind, metadata, spec). This serves as the language-agnostic interface between Genesis (producer) and Prism (consumer).

## Requirements

### R1 - Standard Envelope

```yaml
id: R1
priority: medium
status: draft
```

All SpecIR files must follow a standard envelope structure containing `apiVersion`, `kind`, `metadata`, and `spec` fields.

### R2 - Kind Registry

```yaml
id: R2
priority: medium
status: draft
```

The `kind` field identifies the type of spec payload (e.g., `Api`, `FlowchartPlus`, `SequencePlus`, `ClassPlus`). The `spec` field structure depends on the `kind`.

### R3 - Strict Serialization

```yaml
id: R3
priority: medium
status: draft
```

Manifests must be strictly serializable to/from YAML. Unknown fields should be rejected during validation to ensure strict contract adherence.

## Acceptance Criteria

### Scenario: Serialize API Spec

- **WHEN** An API spec is serialized
- **THEN** A valid YAML manifest with kind `Api` and OpenApi spec payload is produced

### Scenario: Deserialize Valid Manifest

- **WHEN** A valid YAML file is read
- **THEN** The manifest is successfully parsed into the corresponding Rust struct

### Scenario: Error on Missing Kind

- **WHEN** A YAML file missing the `kind` field is parsed
- **THEN** A validation error is returned stating `kind` is required

## Diagrams

### SpecIR Manifest Model

```mermaid
classDiagram
    class SpecManifest {
        +String apiVersion
        +String kind
    }
    class Metadata {
        +String name
        +String change_id
    }
    class ApiManifest {
        +OpenApi spec
    }
    class FlowchartManifest {
        +Flowchart spec
    }
    SpecManifest *-- Metadata : contains
    SpecManifest <|-- ApiManifest : implements
    SpecManifest <|-- FlowchartManifest : implements
```

## API Specification (JSON Schema)

```yaml
$schema: http://json-schema.org/draft-07/schema#
properties:
  apiVersion:
    type: string
  kind:
    enum:
    - Api
    - FlowchartPlus
    - SequencePlus
    - ClassPlus
    - ErdPlus
    - RequirementPlus
    type: string
  metadata:
    properties:
      change_id:
        type: string
      name:
        type: string
      source_file:
        type: string
    required:
    - name
    - change_id
    type: object
  spec:
    type: object
required:
- apiVersion
- kind
- metadata
- spec
title: SpecManifest
type: object
```

</spec>

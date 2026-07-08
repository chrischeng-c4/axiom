---
id: json-schema-core
type: spec
title: "JSON Schema Core Implementation"
version: 1
spec_type: data-model
created_at: 2026-02-02T13:49:43.559096+00:00
updated_at: 2026-02-03T10:49:00.000000+00:00
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
      title: "JSON Schema Core Class Diagram"
history:
  - timestamp: 2026-02-02T13:49:43.559096+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-02-03T10:49:00.000000+00:00
    agent: "gemini"
    action: "merged"
    message: "Full rewrite from generate-codegen change"
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Codegen TDs support CB lifecycle generation and regenerable artifact production."
---

<spec>

# JSON Schema Core Implementation

## Overview
<!-- type: overview lang: markdown -->

Defines the core JSON Schema structures and parsing logic for cclab-sdd. This module is responsible for parsing JSON Schema strings into a strongly-typed Rust structure that can be used by validators and generators.

## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: json-schema-core-requirements
---
requirementDiagram
    requirement R1 {
        id: R1
        text: Support parsing Draft 7 and Draft 2020-12 JSON Schemas.
        risk: medium
        verifymethod: test
    }
    requirement R2 {
        id: R2
        text: Provide strongly typed schema structures including recursive ref handling.
        risk: medium
        verifymethod: test
    }
    requirement R3 {
        id: R3
        text: Support serialization and deserialization through Serde.
        risk: medium
        verifymethod: test
    }
```

## Acceptance Criteria
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - name: parse-draft-7-schema
    given: A valid Draft 7 JSON Schema string.
    when: The parse function is called.
    then: It is successfully parsed into a JsonSchema struct.
  - name: handle-recursion
    given: A JSON Schema with a circular $ref.
    when: The schema is traversed.
    then: The structure preserves the reference or resolves it lazily.
```

## Diagrams
<!-- type: diagram lang: mermaid -->

### JSON Schema Core Class Diagram

```mermaid
classDiagram
    class JsonSchemaCore {
    }
    JsonSchema *-- Schema : contains
    Schema --> SchemaType : has type
```

## API Specification (JSON Schema)
<!-- type: schema lang: yaml -->

```yaml
properties:
  definitions:
    type: object
  schema_version:
    type: string
title: JsonSchema
type: object
```

</spec>

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - action: annotate
    section: requirements
    impl_mode: hand-written
    description: "Traceability metadata edge for the requirements section."

  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```
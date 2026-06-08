---
id: lens-yaml-codegen
type: spec
title: "Lens YAML-Based Code Generation"
version: 1
spec_type: integration
tags: [external]
spec_group: cclab-lens
created_at: 2026-02-14T17:26:42.192461+00:00
updated_at: 2026-02-14T17:26:42.192461+00:00
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
      title: "Codegen Flow"
history:
  - timestamp: 2026-02-14T17:26:42.192461+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This codegen logic TD supports CB lifecycle generation and regenerable artifact production."
---

<spec>

# Lens YAML-Based Code Generation

## Overview
<!-- type: doc lang: markdown -->

Updates Lens to read SpecIR YAML manifests from disk and dispatch them to the appropriate CodeGenerator implementation. This replaces the direct Rust struct injection used previously.

## Requirements
<!-- type: doc lang: markdown -->

### R1 - YAML Reader

```yaml
id: R1
priority: medium
status: draft
```

Lens must provide a reader that deserializes YAML files into the `SpecManifest` struct defined in the schema spec.

### R2 - Generic Generator Input

```yaml
id: R2
priority: medium
status: draft
```

The `CodeGenerator` trait must be updated (or wrapped) to accept `SpecManifest` input, allowing generators to consume the standard IR format.

### R3 - Generator Dispatch

```yaml
id: R3
priority: medium
status: draft
```

Lens must dispatch the parsed manifest to the correct generator based on the `kind` field and the target language configuration.

## Acceptance Criteria
<!-- type: doc lang: markdown -->

### Scenario: Generate from YAML

- **WHEN** Lens is invoked with valid YAML IR paths
- **THEN** Code is generated successfully matching the spec content

### Scenario: Invalid YAML Format

- **WHEN** Lens encounters a malformed YAML file
- **THEN** An error is returned describing the parsing failure

### Scenario: Unsupported Kind

- **WHEN** Lens encounters a manifest with an unknown kind
- **THEN** An error is returned stating no generator found for kind

## Diagrams
<!-- type: doc lang: markdown -->

### Codegen Flow

```mermaid
sequenceDiagram
    participant SDD as SDD
    participant Lens as Lens
    participant YamlReader as YamlReader
    participant CodeGenerator as CodeGenerator
    SDD->>Lens: generate(spec_ir_paths)
    Lens->>YamlReader: read_manifest(path)
    YamlReader->>Lens: SpecManifest
    Lens->>CodeGenerator: generate(manifest)
    CodeGenerator->>Lens: GeneratedCode
    Lens->>SDD: Result
```

</spec>

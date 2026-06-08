---
id: prism-yaml-codegen
type: spec
title: "Prism YAML-Based Code Generation"
version: 1
spec_type: integration
tags: [external]
spec_group: cclab-prism
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
---

<spec>

# Prism YAML-Based Code Generation

## Overview

Updates Prism to read SpecIR YAML manifests from disk and dispatch them to the appropriate CodeGenerator implementation. This replaces the direct Rust struct injection used previously.

## Requirements

### R1 - YAML Reader

```yaml
id: R1
priority: medium
status: draft
```

Prism must provide a reader that deserializes YAML files into the `SpecManifest` struct defined in the schema spec.

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

Prism must dispatch the parsed manifest to the correct generator based on the `kind` field and the target language configuration.

## Acceptance Criteria

### Scenario: Generate from YAML

- **WHEN** Prism is invoked with valid YAML IR paths
- **THEN** Code is generated successfully matching the spec content

### Scenario: Invalid YAML Format

- **WHEN** Prism encounters a malformed YAML file
- **THEN** An error is returned describing the parsing failure

### Scenario: Unsupported Kind

- **WHEN** Prism encounters a manifest with an unknown kind
- **THEN** An error is returned stating no generator found for kind

## Diagrams

### Codegen Flow

```mermaid
sequenceDiagram
    participant Genesis as Genesis
    participant Prism as Prism
    participant YamlReader as YamlReader
    participant CodeGenerator as CodeGenerator
    Genesis->>Prism: generate(spec_ir_paths)
    Prism->>YamlReader: read_manifest(path)
    YamlReader->>Prism: SpecManifest
    Prism->>CodeGenerator: generate(manifest)
    CodeGenerator->>Prism: GeneratedCode
    Prism->>Genesis: Result
```

</spec>

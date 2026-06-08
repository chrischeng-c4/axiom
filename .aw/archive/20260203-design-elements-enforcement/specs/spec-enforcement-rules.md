---
id: spec-enforcement-rules
type: spec
title: "Centralized Spec Enforcement Rules"
version: 1
spec_type: algorithm
created_at: 2026-01-26T10:28:13.522289+00:00
updated_at: 2026-01-26T10:28:13.522289+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: true
  diagrams:
    - type: flowchart
      title: "Spec Validation Logic"
history:
  - timestamp: 2026-01-26T10:28:13.522289+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-01-26T10:28:34.005142+00:00
    agent: "gemini-3-flash-preview"
    tool: "revise_spec"
    action: "revised"
    duration_secs: 294.53
  - timestamp: 2026-01-26T10:29:10.250276+00:00
    agent: "gpt-5.2-codex"
    tool: "review_spec"
    action: "reviewed"
    duration_secs: 36.24---

<spec>

# Centralized Spec Enforcement Rules

## Overview

This specification defines the rules for enforcing design elements (diagrams and API specs) in technical specifications based on their spec_type. These rules ensure that specs are complete, consistent, and ready for automated code generation by centralizing requirements in a single source of truth. It covers the mapping of spec types to required Mermaid diagrams and API specification formats, and the validation logic to be implemented in the SemanticValidator and MCP tools.

## Requirements

### R1 - Centralized SpecType Definitions

```yaml
id: R1
priority: high
status: draft
```

Move SpecType enum and its associated logic (required diagrams, required API specs) from spec_service.rs to models/spec_rules.rs to centralize specification rules.

### R2 - Mapping SpecType to Required Diagrams

```yaml
id: R2
priority: high
status: draft
```

Each spec_type must map to a set of required Mermaid diagram types (e.g., http-api requires sequence, data-model requires ERD or Class, algorithm requires flowchart or state, workflow requires state or flowchart).

### R3 - Mapping SpecType to Required API Specs

```yaml
id: R3
priority: high
status: draft
```

Each spec_type must map to a required API specification format (e.g., http-api requires OpenAPI 3.1, event-driven requires AsyncAPI 2.6, data-model requires JSON Schema, rpc-api requires OpenRPC 1.3, workflow requires Serverless Workflow 0.8).

### R4 - Validation Logic Enforcement

```yaml
id: R4
priority: high
status: draft
```

The SemanticValidator must be updated to enforce the presence of required diagrams and API specs based on the spec_type parsed from the document.

### R5 - Tool Integration

```yaml
id: R5
priority: medium
status: draft
```

Update MCP tools (create_spec, validate_spec_completeness, get_task) to utilize the centralized SpecType rules for validation and guidance.

## Acceptance Criteria

### Scenario: Valid http-api spec validation

- **GIVEN** A specification file with frontmatter 'spec_type: http-api'
- **WHEN** The file contains both a Mermaid sequence diagram and an OpenAPI 3.1 specification.
- **THEN** The validation should pass without errors.

### Scenario: Missing diagram in data-model spec

- **GIVEN** A specification file with frontmatter 'spec_type: data-model'
- **WHEN** The file does not contain an ERD or a class diagram.
- **THEN** The validation should fail with an error indicating that an ERD or class diagram is required.

### Scenario: Missing API spec in workflow spec

- **GIVEN** A specification file with frontmatter 'spec_type: workflow'
- **WHEN** The file does not contain a Serverless Workflow 0.8 specification.
- **THEN** The validation should fail with an error indicating that a Serverless Workflow 0.8 specification is required.

### Scenario: Valid utility spec validation

- **GIVEN** A specification file with frontmatter 'spec_type: utility'
- **WHEN** The file contains no diagrams and no API spec.
- **THEN** The validation should pass without errors.

### Scenario: Missing sequence diagram in http-api spec

- **GIVEN** A specification file with frontmatter 'spec_type: http-api'
- **WHEN** The file contains an OpenAPI 3.1 spec but no sequence diagram.
- **THEN** The validation should fail indicating that a sequence diagram is required.

### Scenario: Missing OpenAPI 3.1 in http-api spec

- **GIVEN** A specification file with frontmatter 'spec_type: http-api'
- **WHEN** The file contains a sequence diagram but no OpenAPI 3.1 spec.
- **THEN** The validation should fail indicating that an OpenAPI 3.1 specification is required.

### Scenario: Invalid spec_type in frontmatter

- **GIVEN** A specification file with frontmatter 'spec_type: invalid-type'
- **WHEN** The file is validated.
- **THEN** The validation should fail indicating that 'invalid-type' is not a supported spec_type.

## Diagrams

### Spec Validation Logic

```mermaid
flowchart TB
    Start[Start Validation]
    ParseFrontmatter[Parse Frontmatter]
    GetSpecType[Get spec_type]
    LookupRules[Lookup Required Elements for SpecType]
    CheckRequiredDiagrams[Check for Required Mermaid Diagrams]
    CheckRequiredApiSpec[Check for Required API Spec Block]
    CollectIssues[Collect Missing Elements]
    IsComplete[Is Spec Complete?]
    Success[Validation Passed]
    Failure[Validation Failed (Return Issues)]
    Start --> ParseFrontmatter
    ParseFrontmatter --> GetSpecType
    GetSpecType --> LookupRules
    LookupRules --> CheckRequiredDiagrams
    CheckRequiredDiagrams --> CheckRequiredApiSpec
    CheckRequiredApiSpec --> CollectIssues
    CollectIssues --> IsComplete
    IsComplete -->|Yes| Success
    IsComplete -->|No| Failure
```

<semantic-data>

```json
{
  "edges": [],
  "metadata": null,
  "nodes": [
    {
      "id": "Start",
      "semantic": {
        "type": "start"
      }
    },
    {
      "id": "ParseFrontmatter",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "GetSpecType",
      "semantic": {
        "type": "assign"
      }
    },
    {
      "id": "LookupRules",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "CheckRequiredDiagrams",
      "semantic": {
        "error": {
          "code": 400,
          "message": "Missing required diagrams for {{spec_type}}"
        },
        "type": "validation"
      }
    },
    {
      "id": "CheckRequiredApiSpec",
      "semantic": {
        "error": {
          "code": 400,
          "message": "Missing required API spec for {{spec_type}}"
        },
        "type": "validation"
      }
    },
    {
      "id": "CollectIssues",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "IsComplete",
      "semantic": {
        "type": "condition"
      }
    },
    {
      "id": "Success",
      "semantic": {
        "type": "end"
      }
    },
    {
      "id": "Failure",
      "semantic": {
        "type": "end"
      }
    }
  ]
}
```

</semantic-data>

</spec>

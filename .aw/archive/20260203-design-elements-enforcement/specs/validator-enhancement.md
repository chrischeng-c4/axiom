---
id: validator-enhancement
type: spec
title: "Validator Enhancement for Spec Enforcement"
version: 1
spec_type: algorithm
created_at: 2026-01-26T10:34:43.960318+00:00
updated_at: 2026-01-26T10:34:43.960318+00:00
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
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: true
  diagrams:
    - type: flowchart
      title: "Enhanced Spec Validation Flow"
history:
  - timestamp: 2026-01-26T10:34:43.960318+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-01-26T10:35:20.682404+00:00
    agent: "gemini-3-flash-preview"
    tool: "revise_spec"
    action: "revised"
    duration_secs: 200.54
  - timestamp: 2026-01-26T10:35:48.185692+00:00
    agent: "gpt-5.2-codex"
    tool: "review_spec"
    action: "reviewed"
    duration_secs: 27.50---

<spec>

# Validator Enhancement for Spec Enforcement

## Overview

This specification outlines the enhancements to the SemanticValidator and MCP tools to enforce specification completeness rules based on centralized SpecType definitions. It ensures that required Mermaid diagrams and API specifications are present for each spec_type, facilitating automated code generation. The centralized rules from spec_rules.rs will be used across validation, creation, and guidance tools (validate_spec_completeness, create_spec, get_task, and prompts.rs).

## Requirements

### R1 - Centralized Rule Integration

```yaml
id: R1
priority: high
status: draft
```

SemanticValidator must use the centralized SpecType and ApiSpecType from models/spec_rules.rs to determine required elements.

### R2 - Frontmatter Parsing for SpecType

```yaml
id: R2
priority: high
status: draft
```

SemanticValidator must parse the spec_type from the document frontmatter before performing semantic validation.

### R3 - Required Diagram Validation

```yaml
id: R3
priority: high
status: draft
```

SemanticValidator must verify the presence of required Mermaid diagrams (e.g., sequence, erd, class, flowchart, state) as defined for the resolved SpecType.

### R4 - Required API Spec Validation

```yaml
id: R4
priority: high
status: draft
```

SemanticValidator must verify the presence and correct format of required API specifications (e.g., OpenAPI 3.1, AsyncAPI 2.6, OpenRPC 1.3, Serverless Workflow 0.8) for the resolved SpecType.

### R5 - validate_spec_completeness Refactoring

```yaml
id: R5
priority: medium
status: draft
```

The validate_spec_completeness tool must be refactored to delegate validation logic to the enhanced SemanticValidator, ensuring consistency.

### R6 - create_spec Rule Enforcement

```yaml
id: R6
priority: high
status: draft
```

The create_spec tool must be updated to enforce the centralized spec_type rules before allowing spec creation.

### R7 - Guidance Enhancement (get_task/prompts)

```yaml
id: R7
priority: medium
status: draft
```

The get_task tool and orchestrator prompts must be updated to provide specific guidance on required design elements based on the spec_type.

## Acceptance Criteria

### Scenario: Valid http-api spec passes validation

- **GIVEN** A spec with 'spec_type: http-api'
- **WHEN** The spec contains a sequence diagram and an OpenAPI 3.1 block.
- **THEN** ValidationResult is valid with no missing element errors.

### Scenario: Missing sequence diagram in http-api fails validation

- **GIVEN** A spec with 'spec_type: http-api'
- **WHEN** The spec contains an OpenAPI 3.1 block but no sequence diagram.
- **THEN** ValidationResult is invalid with an error indicating missing sequence diagram.

### Scenario: Missing API spec in workflow fails validation

- **GIVEN** A spec with 'spec_type: workflow'
- **WHEN** The spec contains a state diagram but no Serverless Workflow block.
- **THEN** ValidationResult is invalid with an error indicating missing Serverless Workflow 0.8 specification.

### Scenario: Utility spec with no diagrams passes validation

- **GIVEN** A spec with 'spec_type: utility'
- **WHEN** The spec contains no diagrams or API specs.
- **THEN** ValidationResult is valid.

### Scenario: create_spec fails for incomplete http-api

- **GIVEN** An incomplete http-api spec is submitted to create_spec
- **WHEN** The sequence diagram is missing.
- **THEN** The tool returns a validation error and prevents creation.

### Scenario: get_task provides sequence diagram hint for http-api

- **GIVEN** get_task is called for an http-api spec
- **WHEN** The task involves creating or revising the spec.
- **THEN** The instructions include a reminder to include a sequence diagram.

### Scenario: validate_spec_completeness consistency with SemanticValidator

- **GIVEN** A spec is validated via SemanticValidator and validate_spec_completeness tool
- **WHEN** Validation is triggered for any spec_type.
- **THEN** Both return the exact same set of missing elements and validation status.

## Diagrams

### Enhanced Spec Validation Flow

```mermaid
flowchart TB
    Start[Start Validation]
    ParseFM[Parse Frontmatter (spec_type)]
    GetSpecType[Resolve SpecType Enum]
    LookupRules[Lookup Required Elements (Diagrams/API)]
    CheckDiagrams[Verify Required Diagrams Exist]
    CheckApiSpec[Verify Required API Spec Block Exists]
    CollectErrors[Collect Missing Element Errors]
    IsComplete[Is Spec Complete?]
    Success[Validation Passed]
    Failure[Validation Failed (Return Errors)]
    Start --> ParseFM
    ParseFM --> GetSpecType
    GetSpecType --> LookupRules
    LookupRules --> CheckDiagrams
    CheckDiagrams --> CheckApiSpec
    CheckApiSpec --> CollectErrors
    CollectErrors --> IsComplete
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
      "id": "ParseFM",
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
      "id": "CheckDiagrams",
      "semantic": {
        "error": {
          "code": 400,
          "message": "Missing required diagrams for spec_type"
        },
        "type": "validation"
      }
    },
    {
      "id": "CheckApiSpec",
      "semantic": {
        "error": {
          "code": 400,
          "message": "Missing required API specification for spec_type"
        },
        "type": "validation"
      }
    },
    {
      "id": "CollectErrors",
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

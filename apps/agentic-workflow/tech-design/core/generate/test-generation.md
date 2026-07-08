---
id: test-generation
type: spec
title: "Test Generation Integration with cclab-probe"
version: 1
spec_type: integration
main_spec_ref: codegen-system
merge_strategy: replace
created_at: 2026-02-02T14:23:29.642637+00:00
updated_at: 2026-02-03T10:51:00.000000+00:00
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
    - type: sequence
      title: "Test Generation Integration Sequence"
history:
  - timestamp: 2026-02-02T14:23:29.642637+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-02-02T14:23:35.025601+00:00
    agent: "codex:deep"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-02-02T14:24:07.310358+00:00
    agent: "codex:max"
    tool: "review_spec"
    action: "reviewed"
  - timestamp: 2026-02-02T14:24:29.041141+00:00
    agent: "codex:deep"
    tool: "revise_spec"
    action: "revised"
  - timestamp: 2026-02-02T14:24:50.976392+00:00
    agent: "codex:max"
    tool: "review_spec"
    action: "reviewed"
  - timestamp: 2026-02-03T10:51:00.000000+00:00
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

# Test Generation Integration with cclab-probe

## Overview
<!-- type: doc lang: markdown -->

Defines how cclab-sdd generates framework-specific test suites and integrates with cclab-probe to execute and validate generated services for FastAPI, Express, and Axum outputs.

## Requirements
<!-- type: doc lang: markdown -->

### R1 - Generator Test Artifacts

```yaml
id: R1
priority: high
status: draft
```

The test-generation pipeline must produce framework-specific test artifacts (fixtures, client helpers, and test cases) for FastAPI, Express, and Axum outputs using the corresponding generator outputs as inputs.

### R2 - Probe Adapter Integration

```yaml
id: R2
priority: high
status: draft
```

The system must provide a Probe adapter that packages generated tests into a cclab-probe compatible suite, including metadata for endpoints, expected responses, and runtime config.

### R3 - Deterministic Outputs

```yaml
id: R3
priority: medium
status: draft
```

Given the same generator inputs and templates, test generation must be deterministic in file names and content ordering to avoid spurious diffs.

### R4 - Failure Reporting

```yaml
id: R4
priority: medium
status: draft
```

When test generation fails, the system must return structured errors that include the generator type, template name, and a human-readable cause.

## Acceptance Criteria
<!-- type: doc lang: markdown -->

### Scenario: Generate FastAPI Test Suite

- **GIVEN** A FastAPI service is generated from an OpenAPI spec and templates
- **WHEN** test generation runs for generator-fastapi
- **THEN** A probe-compatible test suite is produced with endpoint tests and fixture setup.

### Scenario: Generate Express Test Suite

- **GIVEN** An Express service is generated from an OpenAPI spec and templates
- **WHEN** test generation runs for generator-express
- **THEN** A probe-compatible test suite is produced with endpoint tests and fixture setup.

### Scenario: Generate Axum Test Suite

- **GIVEN** An Axum service is generated from an OpenAPI spec and templates
- **WHEN** test generation runs for generator-axum
- **THEN** A probe-compatible test suite is produced with endpoint tests and fixture setup.

### Scenario: Handle Template Failure

- **GIVEN** A test template is missing or fails to render
- **WHEN** test generation runs
- **THEN** The system returns a structured error that includes the generator type and template name.

## Diagrams
<!-- type: doc lang: markdown -->

### Test Generation Integration Sequence

```mermaid
sequenceDiagram
    actor gen as Framework Generator
    participant testgen as Generate Test Generator
    participant templates as Test Templates
    participant probe as cclab-probe
    gen->>testgen: Emit service artifacts (routes, models, config)
    testgen->>templates: Render test templates with service context
    templates-->>testgen: Return generated test files
    testgen->>probe: Package suite + metadata and invoke probe adapter
    probe-->>testgen: Report execution results + diagnostics
```

</spec>

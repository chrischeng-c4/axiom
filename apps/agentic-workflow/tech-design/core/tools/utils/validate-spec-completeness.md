---
files:
  - tools/validate_spec.rs
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd_validate_spec_completeness: Spec Completeness Validator

Validates that a change spec has all required elements for code generation. Checks for required diagrams, API specifications, requirements, and acceptance scenarios based on the spec's `spec_type` frontmatter field. Uses the central `SpecType` enum from `models/spec_rules.rs` as the single source of truth for validation rules.

## OpenRPC Method Definition
<!-- type: rpc-api lang: yaml -->

```yaml
name: sdd_validate_spec_completeness
summary: Validate that a spec has all required elements for code generation
params:
  - name: project_path
    required: true
    schema:
      type: string
      description: Project root path (use $PWD for current directory)
  - name: change_id
    required: true
    schema:
      type: string
      description: The change ID containing the spec
  - name: spec_id
    required: true
    schema:
      type: string
      description: The spec ID to validate (without .md extension)
result:
  name: result
  schema:
    type: object
    required:
      - is_complete
      - missing_elements
      - warnings
      - coverage
      - spec_type
    properties:
      is_complete:
        type: boolean
        description: True if all required elements are present
      missing_elements:
        type: array
        items:
          type: string
        description: List of required elements that are missing
      warnings:
        type: array
        items:
          type: string
        description: Non-blocking suggestions (low coverage, missing diagrams, etc.)
      coverage:
        type: object
        properties:
          requirements_count:
            type: integer
          scenarios_count:
            type: integer
          requirements_with_scenarios_percent:
            type: number
          diagrams_count:
            type: integer
          has_api_spec:
            type: boolean
      spec_type:
        type:
          - string
          - "null"
        description: The spec_type parsed from frontmatter, or null if absent
```

## Behavior
<!-- type: doc lang: markdown -->

### File Resolution

The tool reads the spec file at: `{project_path}/.aw/changes/{change_id}/specs/{spec_id}.md`

Returns an error if the file does not exist.

### Validation Pipeline

1. **Parse frontmatter** -- extracts `spec_type` from the YAML frontmatter block (`---` delimiters).
2. **Count requirements** -- lines matching `### R{digit}` (e.g., `### R1: Create User`).
3. **Count scenarios** -- lines matching `### Scenario:`.
4. **Count diagrams** -- occurrences of `` ```mermaid `` fenced code blocks.
5. **Detect API specs** -- checks for OpenAPI 3.1, AsyncAPI 2.6, JSON Schema, OpenRPC 1.3, and Serverless Workflow 0.8 markers in content.
6. **Spec-type-specific validation** -- uses `SpecType::required_diagrams()` and `SpecType::required_api_spec()` to determine what is required.
7. **General validations** -- at least one requirement and at least one scenario are always required.

### Spec Type Rules

Validation rules are derived from the central `SpecType` enum. For types with multiple allowed diagrams, any one satisfies the requirement.

| spec_type | Required Diagram(s) | Required API Spec |
|-----------|---------------------|-------------------|
| `http-api` | Sequence | OpenAPI 3.1 |
| `event-driven` | Sequence | AsyncAPI 2.6 |
| `data-model` | ERD or Class | JSON Schema |
| `algorithm` | Flowchart or State | (none) |
| `integration` | Sequence | (none) |
| `rpc-api` | Class | OpenRPC 1.3 |
| `workflow` | State or Flowchart | Serverless Workflow 0.8 |
| `utility` | (none) | (none) |

### Completeness vs Warnings

- **`is_complete = false`** when `missing_elements` is non-empty. Missing elements include: required diagrams not found, required API spec not found, zero requirements, or zero scenarios.
- **Warnings** are non-blocking: low scenario-to-requirement ratio, no diagrams at all, flowcharts missing semantic annotations, or unknown/missing `spec_type`.

### Diagram Detection

Diagrams are detected by Mermaid keywords in the content:

| Diagram Type | Detection Pattern |
|-------------|-------------------|
| Sequence | `sequenceDiagram` |
| ERD | `erDiagram` |
| Class | `classDiagram` |
| Flowchart | `flowchart` or `graph ` |
| State | `stateDiagram` |
| MindMap | `mindmap` |
| Requirement | `requirementDiagram` |
| Journey | `journey` |

### API Spec Detection

| API Spec | Detection Patterns |
|----------|-------------------|
| OpenAPI 3.1 | `openapi: 3.1`, `"openapi": "3.1"`, or `` ```yaml\nopenapi: `` |
| AsyncAPI 2.6 | `asyncapi: 2.6`, `"asyncapi": "2.6"`, or `` ```yaml\nasyncapi: `` |
| JSON Schema | `"$schema"` or `$schema:` |
| OpenRPC 1.3 | `openrpc: 1.3`, `openrpc: "1.3"`, or `"openrpc": "1.3"` |
| Serverless Workflow 0.8 | `specVersion: 0.8`, `specVersion: "0.8"`, or `"specVersion": "0.8"` |

### Flowchart Semantic Annotation Warning

If a flowchart is detected (`flowchart` or `graph ` keyword present) but no `semantic:` or `"semantic"` annotation is found in the content, a warning is emitted suggesting the flowchart lacks semantic annotations for code generation.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - action: annotate
    section: rpc-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rpc-api section."

```
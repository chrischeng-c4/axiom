---
id: sdd-services-spec-service
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Spec Service Types

## Overview
<!-- type: overview lang: markdown -->

Input data types for the spec creation service in
`projects/agentic-workflow/src/services/spec_service.rs`. Six shapes (all
`[Debug, Clone]`):

- `RequirementData` — id/title/description/priority.
- `ScenarioData` — name/given?/when/then.
- `DiagramData` — diagram_type/title/input (Value)/rendered?/semantic?.
- `ApiSpecData` — spec_type (foreign ApiSpecType)/spec (Value).
- `CreateSpecInput` — top-level input bundle (~18 fields incl spec_type, diagrams, optional metadata).
- `SpecChangeData` — file/action/optional context_ref/optional description.

Codegen replaces all six type declarations. Companion source templates own the
module documentation, imports, diagram rendering, API validation, spec type
requirement checks, spec file creation, SpecIR generation, section rules, tag
resolution, and regression tests.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  RequirementData:
    type: object
    required: [id, title, description, priority]
    description: A requirement parsed from spec input.
    properties:
      id:
        type: string
        description: "Requirement identifier."
      title:
        type: string
        description: "Requirement title."
      description:
        type: string
        description: "Requirement description."
      priority:
        type: string
        description: "Priority label."
    x-rust-struct:
      derive: [Debug, Clone]

  ScenarioData:
    type: object
    required: [name, given, when, then]
    description: A scenario block.
    properties:
      name:
        type: string
        description: "Scenario name."
      given:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional Given clause."
      when:
        type: string
        description: "When clause."
      then:
        type: string
        description: "Then clause."
    x-rust-struct:
      derive: [Debug, Clone]

  DiagramData:
    type: object
    required: [diagram_type, title, input, rendered, semantic]
    description: Structured diagram data.
    properties:
      diagram_type:
        type: string
        description: "Diagram type matching generate_mermaid_* tool."
      title:
        type: string
        description: "Human-readable title."
      input:
        type: object
        x-rust-type: "Value"
        description: "Input matching the corresponding Mermaid tool schema."
      rendered:
        type: string
        x-rust-type: "Option<String>"
        description: "Rendered Mermaid code populated during creation."
      semantic:
        type: object
        x-rust-type: "Option<Value>"
        description: "Extracted semantic data populated during creation."
    x-rust-struct:
      derive: [Debug, Clone]

  ApiSpecData:
    type: object
    required: [spec_type, spec]
    description: API specification data.
    properties:
      spec_type:
        type: string
        x-rust-type: "ApiSpecType"
        description: "Specification type."
      spec:
        type: object
        x-rust-type: "Value"
        description: "Full specification object."
    x-rust-struct:
      derive: [Debug, Clone]

  CreateSpecInput:
    type: object
    required: [change_id, spec_id, title, overview, requirements, scenarios, spec_type, diagrams, flow_diagram, data_model, api_spec, agent, duration_secs, spec_group, group_id, main_spec_ref, merge_strategy, tags, changes, depends]
    description: Input structure for creating a spec.
    properties:
      change_id:
        type: string
        description: "Change identifier."
      spec_id:
        type: string
        description: "Spec identifier."
      title:
        type: string
        description: "Spec title."
      overview:
        type: string
        description: "Spec overview text."
      requirements:
        type: array
        items: { type: object }
        x-rust-type: "Vec<RequirementData>"
        description: "Requirements."
      scenarios:
        type: array
        items: { type: object }
        x-rust-type: "Vec<ScenarioData>"
        description: "Scenarios."
      spec_type:
        type: string
        x-rust-type: "SpecType"
        description: "Spec type classification."
      diagrams:
        type: array
        items: { type: object }
        x-rust-type: "Vec<DiagramData>"
        description: "Structured diagrams."
      flow_diagram:
        type: string
        x-rust-type: "Option<String>"
        description: "DEPRECATED raw Mermaid diagram code."
      data_model:
        type: object
        x-rust-type: "Option<Value>"
        description: "Optional data model."
      api_spec:
        type: object
        x-rust-type: "Option<ApiSpecData>"
        description: "Optional API specification."
      agent:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional agent name for history tracking."
      duration_secs:
        type: number
        x-rust-type: "Option<f64>"
        description: "Optional generation duration."
      spec_group:
        type: string
        x-rust-type: "Option<String>"
        description: "Spec group for organising specs."
      group_id:
        type: string
        x-rust-type: "Option<String>"
        description: "Change group ID for group-scoped spec placement."
      main_spec_ref:
        type: string
        x-rust-type: "Option<String>"
        description: "Reference to existing main spec being extended."
      merge_strategy:
        type: string
        x-rust-type: "Option<String>"
        description: "Strategy for merging back to main specs."
      tags:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Explicit tags."
      changes:
        type: array
        items: { type: object }
        x-rust-type: "Vec<SpecChangeData>"
        description: "File changes associated with this spec."
      depends:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Dependencies on other spec IDs."
    x-rust-struct:
      derive: [Debug, Clone]

  SpecChangeData:
    type: object
    required: [file, action, context_ref, description]
    description: File change data for spec creation.
    properties:
      file:
        type: string
        description: "File path."
      action:
        type: string
        description: "Action verb (create, modify)."
      context_ref:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional context reference."
      description:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional description."
    x-rust-struct:
      derive: [Debug, Clone]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/spec_service.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - RequirementData
      - ScenarioData
      - DiagramData
      - ApiSpecData
      - CreateSpecInput
      - SpecChangeData
    description: |
      Codegen replaces all six type declarations.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] 6 input data carriers; CreateSpecInput is the wide one.
- [schema] All in `required:`; foreign types via x-rust-type.
- [changes] Standard split.

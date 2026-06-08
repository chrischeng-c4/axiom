---
id: sdd-generate-journey-plus-schema
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Journey Plus Schema

## Overview
<!-- type: overview lang: markdown -->

Three serde structs describing a Mermaid Plus journey diagram, in
`projects/agentic-workflow/src/generate/diagrams/journey_plus/schema.rs`:

- `JourneyDef` — top-level diagram (id, title, sections, optional description).
- `SectionDef` — one named section with tasks (name, tasks, optional description).
- `TaskDef` — a single task line (name, satisfaction score 1-5, actors, optional description).

All three carry derive `[Debug, Clone, Serialize, Deserialize]`. Codegen emits
the serde import used by generated derives and attributes. Optional fields use
`#[serde(default)]` to round-trip absent JSON keys to `None` without bespoke
deserialise logic. No impl blocks; hand-written outside CODEGEN is limited to
the module docstring and test module.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  JourneyDef:
    type: object
    required: [id, title, sections]
    description: Journey diagram definition.
    properties:
      id:
        type: string
        description: "Diagram identifier."
      title:
        type: string
        description: "Journey title."
      sections:
        type: array
        items:
          $ref: "#/definitions/SectionDef"
        description: "Sections in this journey."
      description:
        type: string
        description: "Diagram description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SectionDef:
    type: object
    required: [name, tasks]
    description: Section definition.
    properties:
      name:
        type: string
        description: "Section name."
      tasks:
        type: array
        items:
          $ref: "#/definitions/TaskDef"
        description: "Tasks in this section."
      description:
        type: string
        description: "Section description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  TaskDef:
    type: object
    required: [name, score, actors]
    description: Task definition.
    properties:
      name:
        type: string
        description: "Task name."
      score:
        type: integer
        x-rust-type: i32
        description: "Satisfaction score (1-5)."
      actors:
        type: array
        items: { type: string }
        description: "Actors involved."
      description:
        type: string
        description: "Task description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/journey_plus/schema.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - JourneyDef
      - SectionDef
      - TaskDef
    description: |
      Codegen replaces all 3 struct declarations and emits the serde import
      required by their derives and attributes.
  - path: projects/agentic-workflow/src/generate/diagrams/journey_plus/schema.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module docstring and
      `#[cfg(test)] mod tests` block.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Clear and accurate: all three structs, their fields, and the serde/derive strategy are correctly described. The note that optional fields use `#[serde(default)]` and that no impl blocks exist outside CODEGEN is precise and implementer-friendly.
- [schema] All fields from R2 are present, required/optional partitioning matches the hand-written source, `x-rust-struct.derive` satisfies R3, and `x-rust-type: i32` on `score` ensures the generator emits the correct Rust type rather than defaulting to `i64`.
- [changes] The two-entry split between `impl_mode: codegen` (3 struct replacements) and `impl_mode: hand-written` (docstring, imports, test block) correctly captures R5 and the scope boundary. No ambiguity for the codegen pipeline.

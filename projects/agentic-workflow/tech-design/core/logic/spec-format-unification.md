---
id: sdd-spec-format-unification
main_spec_ref: projects/agentic-workflow/logic/spec-format-unification.md
merge_strategy: new
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "This logic TD supports TD/CB artifact lifecycle state, authoring, review, validation, or merge behavior."
---

# Sdd Spec Format Unification

## Overview
<!-- type: doc lang: markdown -->

Unifies SDD spec format across 4 dimensions to reduce LLM token cost, improve output quality, and unblock the codegen pipeline.

**Problems addressed**:
1. Bottom-up fill order — Requirements/Scenarios filled after data models (inverts human reasoning)
2. JSON for structured data — schema/rpc-api/config/component/design-token waste 30-40% tokens vs YAML
3. Unstructured requirements/scenarios/test-plan — not machine-parseable, no trace graph
4. Plain Mermaid — existing Plus generators (requirement_plus, sequence_plus, state_plus, flowchart_plus, class_plus, erd_plus, mindmap_plus) unused; spec sections map to basic Mermaid

**Solution**: Mechanical changes to 3 files — `spec_rules.rs` (fill_order + default_lang), `format_rules.rs` (validation), `common_change_spec.rs` (UNIVERSAL_SKELETON + ALL_SECTIONS).
## Requirements
<!-- type: doc lang: markdown -->

### R1: Top-down fill order

Reorder `SectionType::fill_order()` to top-down: overview(0) → requirements(1) → scenarios(2) → mindmap(3) → state-machine(4) → interaction(5) → logic(6) → dependency(7) → db-model(8) → schema(9) → rest-api(10) → rpc-api(11) → async-api(12) → cli(13) → wireframe(14) → component(15) → design-token(16) → config(17) → test-plan(18) → changes(19) → doc(20).

**Priority**: high

### R2: Lang unification — json → yaml

Change `default_lang()` for `schema`, `rpc-api`, `config`, `component`, `design-token` from `"json"` to `"yaml"`. Only 3 langs remain: markdown, yaml, mermaid. JSON is removed from all section defaults.

**Priority**: high

### R3: requirements and test-plan → Mermaid Plus requirementDiagram

Change `default_lang()` for `requirements` and `test-plan` from `"markdown"` to `"mermaid"`. These sections use Mermaid Plus `requirementDiagram` (SysML v1.6). Requirements defines `requirement` nodes; test-plan defines `element` nodes with `verifies` relationships.

**Priority**: high

### R4: scenarios → YAML GWT structured format

Change `default_lang()` for `scenarios` from `"markdown"` to `"yaml"`. Scenarios use YAML GWT structure: list of `{id, given, when, then, diagram_ref?}`. No inline diagrams — cross-reference via `diagram_ref: "#interaction-S1"`.

**Priority**: high

### R5: Format validation updated

Update `REQUIRED_CODE_BLOCK_TYPES` in `format_rules.rs` to reflect new langs. Update `PROSE_ONLY_TYPES` to remove requirements, scenarios, test-plan. Add mermaid validation for requirements/test-plan, yaml for scenarios.

**Priority**: high

### R6: UNIVERSAL_SKELETON updated

Update `UNIVERSAL_SKELETON` in `common_change_spec.rs` to reflect new lang annotations: `lang: yaml` for schema/rpc-api/config/component/design-token/scenarios, `lang: mermaid` with Plus frontmatter stubs for requirements/test-plan and all diagram sections.

**Priority**: high

### R7: changes section gains optional satisfies field

Document that `changes` section YAML entries may include an optional `satisfies: [R-id]` field for requirement traceability. This is a convention, not a structural change to the section type.

**Priority**: medium

### R8: Mermaid Plus generators wired to section types

The existing generators in `projects/agentic-workflow/src/generate/diagrams/*_plus/` are already implemented. This change wires their usage into documentation and UNIVERSAL_SKELETON stubs:
- `state-machine` → state_plus (stateDiagram-v2 + YAML frontmatter)
- `interaction` → sequence_plus
- `logic` → flowchart_plus
- `dependency` → class_plus
- `db-model` → erd_plus
- `mindmap` → mindmap_plus
- `requirements` → requirement_plus
- `test-plan` → requirement_plus

**Priority**: medium
## Scenarios
<!-- type: doc lang: markdown -->

### S1: New spec has requirements before schema

- **GIVEN** `score workflow create-change-spec` generates a new spec with sections [overview, requirements, schema, changes]
- **WHEN** the fill loop iterates sections by fill_order()
- **THEN** requirements (order=1) is filled before schema (order=9)

### S2: schema section generates yaml code block

- **GIVEN** a spec has `&lt;!-- type: schema lang: yaml --&gt;`
- **WHEN** the fill agent writes content to the schema section
- **THEN** the agent writes a ```yaml code block (not ```json)
- **AND** `aw check-alignment` passes for this spec

### S3: requirements section uses Mermaid Plus requirementDiagram

- **GIVEN** a spec has `&lt;!-- type: requirements lang: mermaid --&gt;`
- **WHEN** the fill agent writes content
- **THEN** the section contains a ```mermaid code block with `---` YAML frontmatter inside
- **AND** the mermaid body uses `requirementDiagram` syntax

### S4: test-plan verifies requirement IDs

- **GIVEN** requirements section has `requirement R1` and `requirement R2` nodes
- **WHEN** test-plan is filled using Mermaid Plus requirementDiagram
- **THEN** test-plan has `element T1` nodes with `T1 - verifies -> R1` relationships

### S5: scenarios section uses YAML GWT format

- **GIVEN** a spec has `&lt;!-- type: scenarios lang: yaml --&gt;`
- **WHEN** the fill agent writes scenarios
- **THEN** content is a ```yaml code block with list of {id, given, when, then} entries
- **AND** no Mermaid diagrams are embedded inline

### S6: changes entry has optional satisfies field

- **GIVEN** a changes section YAML entry
- **WHEN** the entry modifies a file that implements requirement R3
- **THEN** the entry may include `satisfies: [R3]` for traceability (optional field)
## Diagrams
<!-- type: doc lang: markdown -->

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- score-td-placeholder -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- score-td-placeholder -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- score-td-placeholder -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- score-td-placeholder -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- score-td-placeholder -->
<!-- TODO -->

## API Spec
<!-- type: doc lang: markdown -->

### REST API
<!-- type: rest-api lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

### Schema
<!-- type: schema lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

### Config
<!-- type: config lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

## Test Plan
<!-- type: doc lang: markdown -->

<!-- TODO -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: projects/agentic-workflow/src/models/spec_rules.rs
    section: source
    action: modify
    impl_mode: codegen
    satisfies: [R1, R2, R3, R4]
    desc: |
      1. Reorder fill_order() to top-down: overview=0, requirements=1, scenarios=2,
         mindmap=3, state-machine=4, interaction=5, logic=6, dependency=7, db-model=8,
         schema=9, rest-api=10, rpc-api=11, async-api=12, cli=13, wireframe=14,
         component=15, design-token=16, config=17, test-plan=18, changes=19, doc=20.
      2. Update default_lang():
         - schema, rpc-api, config, component, design-token: json -> yaml
         - requirements, test-plan: markdown -> mermaid
         - scenarios: markdown -> yaml
         Final lang groups:
           markdown: [overview, doc]
           mermaid: [interaction, logic, dependency, state-machine, db-model, mindmap, requirements, test-plan]
           yaml: [rest-api, async-api, changes, wireframe, cli, schema, rpc-api, config, component, design-token, scenarios]

  - path: projects/agentic-workflow/src/spec_alignment/format_rules.rs
    section: source
    action: modify
    impl_mode: codegen
    satisfies: [R5]
    desc: |
      Update REQUIRED_CODE_BLOCK_TYPES:
        ("config", "json") -> ("config", "yaml")
        ("rpc-api", "json") -> ("rpc-api", "yaml")
        ("schema", "json") -> ("schema", "yaml")
        ("component", "json") -> ("component", "yaml")
        ("design-token", "json") -> ("design-token", "yaml")
        Add: ("requirements", "mermaid")
        Add: ("test-plan", "mermaid")
        Add: ("scenarios", "yaml")
      Update PROSE_ONLY_TYPES:
        Remove: requirements, scenarios, test-plan
        Keep: overview, doc

  - path: projects/agentic-workflow/src/tools/common_change_spec.rs
    section: source
    action: modify
    impl_mode: codegen
    satisfies: [R6, R7, R8]
    desc: |
      1. Update UNIVERSAL_SKELETON:
         - Requirements section: lang: mermaid, include mermaid Plus requirementDiagram stub
         - Scenarios section: lang: yaml, include YAML GWT stub
         - Test Plan section: lang: mermaid, include mermaid Plus requirementDiagram element stub
         - Schema/Config/RPC API/Component/Design Token: lang: yaml
         - Diagram sections (interaction/logic/dependency/state-machine/db-model/mindmap):
           include YAML frontmatter stub inside mermaid block (Mermaid Plus format)
         - Changes section: include satisfies: [] field in example YAML
      2. Update ALL_SECTIONS constant to match new top-down fill_order() output
  - action: annotate
    section: async-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the async-api section."

  - action: annotate
    section: cli
    impl_mode: hand-written
    description: "Traceability metadata edge for the cli section."

  - action: annotate
    section: component
    impl_mode: hand-written
    description: "Traceability metadata edge for the component section."

  - action: annotate
    section: config
    impl_mode: hand-written
    description: "Traceability metadata edge for the config section."

  - action: annotate
    section: db-model
    impl_mode: hand-written
    description: "Traceability metadata edge for the db-model section."

  - action: annotate
    section: dependency
    impl_mode: hand-written
    description: "Traceability metadata edge for the dependency section."

  - action: annotate
    section: design-token
    impl_mode: hand-written
    description: "Traceability metadata edge for the design-token section."

  - action: annotate
    section: interaction
    impl_mode: hand-written
    description: "Traceability metadata edge for the interaction section."

  - action: annotate
    section: logic
    impl_mode: hand-written
    description: "Traceability metadata edge for the logic section."

  - action: annotate
    section: rest-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rest-api section."

  - action: annotate
    section: rpc-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rpc-api section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

  - action: annotate
    section: state-machine
    impl_mode: hand-written
    description: "Traceability metadata edge for the state-machine section."

  - action: annotate
    section: wireframe
    impl_mode: hand-written
    description: "Traceability metadata edge for the wireframe section."

```
## Wireframe
<!-- type: wireframe lang: yaml -->

```yaml
wireframes: []
```

## Component
<!-- type: component lang: yaml -->

```yaml
components: []
```

## Design Token
<!-- type: design-token lang: yaml -->

```yaml
tokens: []
```

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

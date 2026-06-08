---
id: sdd-format-rules-yaml
main_spec_ref: projects/agentic-workflow/logic/check-alignment.md
merge_strategy: new
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "This logic TD supports source/spec alignment and traceability closure."
---

# Sdd Format Rules Yaml

## Overview
<!-- type: overview lang: markdown -->

<!-- TODO -->

## Requirements
<!-- type: doc lang: markdown -->

### R1: default_lang json → yaml for structured data sections

Change `SectionType::default_lang()` in `projects/agentic-workflow/src/models/spec_rules.rs` so that `schema`, `rpc-api`, `config`, `component`, and `design-token` return `"yaml"` instead of `"json"`.

**Priority**: high

### R2: format_rules.rs updated for yaml

Update `REQUIRED_CODE_BLOCK_TYPES` in `projects/agentic-workflow/src/spec_alignment/format_rules.rs`:
- `("config", "json")` → `("config", "yaml")`
- `("rpc-api", "json")` → `("rpc-api", "yaml")`
- `("schema", "json")` → `("schema", "yaml")`
- `("component", "json")` → `("component", "yaml")`
- `("design-token", "json")` → `("design-token", "yaml")`

Validators must accept YAML code blocks for these section types.

**Priority**: high

### R3: Mermaid Plus wiring into section type resolution

Update `REQUIRED_CODE_BLOCK_TYPES` to add entries for diagram sections that use Mermaid Plus (frontmatter inside mermaid block). These sections already require `mermaid` blocks, so validation passes. No change to the required lang, but the UNIVERSAL_SKELETON must include YAML frontmatter stubs inside the mermaid code blocks for: `state-machine`, `interaction`, `logic`, `dependency`, `db-model`, `mindmap`, `requirements`, `test-plan`.

**Priority**: medium

### R4: requirements and test-plan change from markdown to mermaid

Update `SectionType::default_lang()` so `requirements` and `test-plan` return `"mermaid"` (Mermaid Plus requirementDiagram) instead of `"markdown"`. Update format_rules: move `requirements` and `test-plan` from `PROSE_ONLY_TYPES` to `REQUIRED_CODE_BLOCK_TYPES` with `("requirements", "mermaid")` and `("test-plan", "mermaid")`.

**Priority**: high

### R5: scenarios change from markdown to yaml

Update `SectionType::default_lang()` so `scenarios` returns `"yaml"` instead of `"markdown"`. Update format_rules: move `scenarios` from `PROSE_ONLY_TYPES` to `REQUIRED_CODE_BLOCK_TYPES` with `("scenarios", "yaml")`.

**Priority**: high
## Scenarios
<!-- type: doc lang: markdown -->

<!-- TODO -->

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
    desc: |
      Update default_lang() match arm:
      - Move schema, rpc-api, config, component, design-token from json arm to yaml arm
      - Change requirements and test-plan from markdown to mermaid
      - Change scenarios from markdown to yaml
      After change:
        markdown: [overview, doc]
        mermaid: [interaction, logic, dependency, state-machine, db-model, mindmap, requirements, test-plan]
        yaml: [rest-api, async-api, changes, wireframe, cli, schema, rpc-api, config, component, design-token, scenarios]

  - path: projects/agentic-workflow/src/spec_alignment/format_rules.rs
    section: source
    action: modify
    impl_mode: codegen
    desc: |
      Update REQUIRED_CODE_BLOCK_TYPES:
      - ("config", "json") -> ("config", "yaml")
      - ("rpc-api", "json") -> ("rpc-api", "yaml")
      - ("schema", "json") -> ("schema", "yaml")
      - ("component", "json") -> ("component", "yaml")
      - ("design-token", "json") -> ("design-token", "yaml")
      Update PROSE_ONLY_TYPES:
      - Remove requirements, scenarios, test-plan from PROSE_ONLY_TYPES
      Add to REQUIRED_CODE_BLOCK_TYPES:
      - ("requirements", "mermaid")
      - ("test-plan", "mermaid")
      - ("scenarios", "yaml")
      Only overview and doc remain in PROSE_ONLY_TYPES.
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

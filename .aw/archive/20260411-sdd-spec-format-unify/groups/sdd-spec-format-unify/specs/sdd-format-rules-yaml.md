---
id: sdd-format-rules-yaml
main_spec_ref: crates/sdd/logic/check-alignment.md
merge_strategy: new
fill_sections: [requirements, changes]
filled_sections: [requirements, changes]
create_complete: true
---

# Sdd Format Rules Yaml

## Overview
<!-- type: overview lang: markdown -->

<!-- TODO -->

## Requirements

<!-- type: requirements lang: markdown -->

### R1: default_lang json → yaml for structured data sections

Change `SectionType::default_lang()` in `crates/sdd/src/models/spec_rules.rs` so that `schema`, `rpc-api`, `config`, `component`, and `design-token` return `"yaml"` instead of `"json"`.

**Priority**: high

### R2: format_rules.rs updated for yaml

Update `REQUIRED_CODE_BLOCK_TYPES` in `crates/sdd/src/spec_alignment/format_rules.rs`:
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
<!-- type: scenarios lang: markdown -->

<!-- TODO -->

## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: markdown -->

<!-- TODO -->

## Changes

<!-- type: changes lang: yaml -->

```yaml
files:
  - path: crates/sdd/src/models/spec_rules.rs
    action: modify
    desc: |
      Update default_lang() match arm:
      - Move schema, rpc-api, config, component, design-token from json arm to yaml arm
      - Change requirements and test-plan from markdown to mermaid
      - Change scenarios from markdown to yaml
      After change:
        markdown: [overview, doc]
        mermaid: [interaction, logic, dependency, state-machine, db-model, mindmap, requirements, test-plan]
        yaml: [rest-api, async-api, changes, wireframe, cli, schema, rpc-api, config, component, design-token, scenarios]

  - path: crates/sdd/src/spec_alignment/format_rules.rs
    action: modify
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
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews

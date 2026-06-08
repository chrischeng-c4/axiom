---
id: score-remaining-p1p2-spec
main_spec_ref: "crates/sdd/logic/remaining-fixes.md"
merge_strategy: new
filled_sections: [overview, requirements, changes]
fill_sections: [overview, requirements, changes]
create_complete: true
---

# Score Remaining P1p2 Spec

## Overview

Six remaining Score issues in one change:

**Bug fixes:** (1) ref context table renders ? placeholders, (2) gen test only reads JSON
**Agent:** (3) impl agent should use code intelligence
**Codegen:** (4) L3 state-machine → enum, (5) L2 FlowchartPlus → skeleton, (6) L2 SequencePlus → call chain
## Requirements

| ID | Title | Description | Priority |
|----|-------|-------------|----------|
| R1 | Fix ref context table | render_specs_markdown() must render spec path, relevance, key requirements from input array | P2 |
| R2 | gen test reads markdown | run_gen_test accepts .md, uses gen parse → RequirementPlus SpecIR | P2 |
| R3 | Agent code intelligence | Update sdd-change-implementation.md: mandate score symbols/references/impact before modifying files | P1 |
| R4 | StateMachineGenerator | stateDiagram-v2 → Python Enum + transition() with match arms (L3) | P2 |
| R5 | FlowchartPlusGenerator | FlowchartPlus YAML metadata → function skeletons + @sdd:implement markers (L2) | P2 |
| R6 | SequencePlusGenerator | SequencePlus messages → async call chain + @sdd:implement markers (L2) | P2 |
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

```yaml
changes:
  - file: crates/sdd/src/tools/create_reference_context.rs
    action: modify
    description: Fix render_specs_markdown to populate table from specs array (R1)
  - file: projects/score/cli/src/codegen.rs
    action: modify
    description: run_gen_test accepts .md via gen parse (R2)
  - file: .claude/agents/sdd-change-implementation.md
    action: modify
    description: Add mandatory code intelligence section (R3)
  - file: crates/sdd/src/generate/generators/state_machine_gen.rs
    action: create
    description: StateMachineGenerator implementing SpecIRGenerator (R4)
  - file: crates/sdd/src/generate/generators/flowchart_plus_gen.rs
    action: create
    description: FlowchartPlusGenerator implementing SpecIRGenerator (R5)
  - file: crates/sdd/src/generate/generators/sequence_plus_gen.rs
    action: create
    description: SequencePlusGenerator implementing SpecIRGenerator (R6)
  - file: crates/sdd/src/generate/generators/mod.rs
    action: modify
    description: Register 3 new generators
  - file: crates/sdd/src/generate/mod.rs
    action: modify
    description: Re-export 3 new generators
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

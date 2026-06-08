---
id: score-remaining-p1p2-spec
main_spec_ref: "projects/agentic-workflow/logic/remaining-fixes.md"
merge_strategy: new
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "This logic TD supports TD/CB artifact lifecycle state, authoring, review, validation, or merge behavior."
---

# Score Remaining P1p2 Spec

## Overview
<!-- type: doc lang: markdown -->

Six remaining Score issues in one change:

**Bug fixes:** (1) ref context table renders ? placeholders, (2) gen test only reads JSON
**Agent:** (3) impl agent should use code intelligence
**Codegen:** (4) L3 state-machine → enum, (5) L2 FlowchartPlus → skeleton, (6) L2 SequencePlus → call chain
## Requirements
<!-- type: doc lang: markdown -->

| ID | Title | Description | Priority |
|----|-------|-------------|----------|
| R1 | Fix ref context table | render_specs_markdown() must render spec path, relevance, key requirements from input array | P2 |
| R2 | gen test reads markdown | run_gen_test accepts .md, uses gen parse → RequirementPlus SpecIR | P2 |
| R3 | Agent code intelligence | Update sdd-change-implementation.md: mandate score symbols/references/impact before modifying files | P1 |
| R4 | StateMachineGenerator | stateDiagram-v2 → Python Enum + transition() with match arms (L3) | P2 |
| R5 | FlowchartPlusGenerator | FlowchartPlus YAML metadata → function skeletons + @sdd:implement markers (L2) | P2 |
| R6 | SequencePlusGenerator | SequencePlus messages → async call chain + @sdd:implement markers (L2) | P2 |
## Scenarios
<!-- type: doc lang: markdown -->

| Scenario | Covers |
|----------|--------|
| S1 | R1, R2, R3 |
| S2 | R4, R5, R6 |

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

| Test | Covers |
|------|--------|
| reference-context-rendering | R1 |
| gen-test-markdown-input | R2 |
| implementation-agent-code-intelligence | R3 |
| generator-output-smoke | R4, R5, R6 |

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: projects/agentic-workflow/src/tools/create_reference_context.rs
    section: source
    action: modify
    impl_mode: hand-written
    description: Fix render_specs_markdown to populate table from specs array (R1)
  - file: projects/agentic-workflow/src/cli/codegen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: run_gen_test accepts .md via gen parse (R2)
  - file: .claude/agents/sdd-change-implementation.md
    action: modify
    section: cli
    impl_mode: hand-written
    description: Add mandatory code intelligence section (R3)
  - file: projects/agentic-workflow/src/generate/generators/state_machine_gen.rs
    section: source
    action: create
    impl_mode: hand-written
    description: StateMachineGenerator implementing SpecIRGenerator (R4)
  - file: projects/agentic-workflow/src/generate/generators/flowchart_plus_gen.rs
    section: source
    action: create
    impl_mode: hand-written
    description: FlowchartPlusGenerator implementing SpecIRGenerator (R5)
  - file: projects/agentic-workflow/src/generate/generators/sequence_plus_gen.rs
    section: source
    action: create
    impl_mode: hand-written
    description: SequencePlusGenerator implementing SpecIRGenerator (R6)
  - file: projects/agentic-workflow/src/generate/generators/mod.rs
    section: source
    action: modify
    impl_mode: hand-written
    description: Register 3 new generators
  - file: projects/agentic-workflow/src/generate/mod.rs
    section: source
    action: modify
    impl_mode: hand-written
    description: Re-export 3 new generators
  - action: annotate
    section: async-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the async-api section."

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

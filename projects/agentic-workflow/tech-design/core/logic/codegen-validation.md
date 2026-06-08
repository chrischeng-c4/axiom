---
id: sdd-codegen-validation-harness
main_spec_ref: projects/agentic-workflow/logic/codegen-validation.md
merge_strategy: new
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This codegen logic TD supports CB lifecycle generation and regenerable artifact production."
---

# Sdd Codegen Validation Harness

## Overview
<!-- type: overview lang: markdown -->

The validation harness provides the TD-to-code diff, apply, and render runtime used by Agentic Workflow code-artifact flows. It is the primary implementation surface for verifying spec-to-code alignment and applying generated CODEGEN blocks to target files.

| Runtime | Purpose | Writes Files |
|---|---|---|
| `generate::diff::run_diff` | Show drift between generated and current code | No |
| `generate::apply::run_apply` | Write generated code to target files | Yes |
| `generate::render::run_render` | Regenerate Mermaid body from frontmatter YAML | Yes (spec file) |

The diff runtime is the primary loop primitive: run it, fix gaps, re-run until no drift remains.

**Diff report classifications**:
- `exact`: generated code matches current file content exactly
- `marker-only`: difference is only SPEC-REF markers (acceptable)
- `drift`: content differs beyond markers (needs fixing)
- `gap`: target file has no CODEGEN block for this spec (missing)

**Metrics** (R1.3):
- `drift%` = drift lines / total lines
- `marker%` = marker lines / total lines  
- `coverage%` = generated lines / total lines
- Exit 0 if drift% == 0 AND marker% <= 10%
## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: requirements
---
requirementDiagram

requirement R1 {
  id: R1
  text: "Diff runtime classifies generated versus current CODEGEN block drift"
  risk: low
  verifymethod: test
}

requirement R2 {
  id: R2
  text: "Apply runtime writes generated CODEGEN blocks to declared target files"
  risk: low
  verifymethod: test
}

requirement R3 {
  id: R3
  text: "Render runtime checks or rewrites Mermaid Plus sections deterministically"
  risk: low
  verifymethod: test
}
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios: []
```

## Diagrams
<!-- type: doc lang: markdown -->

### Mindmap
<!-- type: mindmap lang: mermaid -->
<!-- TODO: Use Mermaid Plus mindmap (YAML frontmatter inside mermaid block).
```mermaid
---
id: mindmap
---
mindmap
  root((System))
    Component A
    Component B
```
-->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO: Use Mermaid Plus stateDiagram-v2 (YAML frontmatter inside mermaid block).
```mermaid
---
id: state-machine
initial: idle
---
stateDiagram-v2
    [*] --> idle
```
-->

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO: Use Mermaid Plus sequenceDiagram (YAML frontmatter inside mermaid block).
```mermaid
---
id: interaction
---
sequenceDiagram
    actor User
    User->>System: action
```
-->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO: Use Mermaid Plus flowchart (YAML frontmatter inside mermaid block).
```mermaid
---
id: logic
---
flowchart TD
    A([Start]) --> B{Decision}
```
-->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO: Use Mermaid Plus classDiagram (YAML frontmatter inside mermaid block).
```mermaid
---
id: dependency
---
classDiagram
    class ComponentA
    class ComponentB
    ComponentA --> ComponentB
```
-->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO: Use Mermaid Plus erDiagram (YAML frontmatter inside mermaid block).
```mermaid
---
id: db-model
---
erDiagram
    ENTITY {
        string id PK
    }
```
-->

## API Spec
<!-- type: doc lang: markdown -->

### REST API
<!-- type: rest-api lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO: OpenRPC 1.3 as YAML. Example:
```yaml
openrpc: "1.3.2"
info:
  title: Service Name
  version: "1.0.0"
methods: []
```
-->

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
<!-- TODO: JSON Schema as YAML. Example:
```yaml
"$schema": "https://json-schema.org/draft/2020-12/schema"
type: object
properties:
  id:
    type: string
required: [id]
```
-->

### Config
<!-- type: config lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: test-plan
---
requirementDiagram

element T1 {
  type: "Test"
}

element T2 {
  type: "Test"
}

element T3 {
  type: "Test"
}

T1 - verifies -> R1
T2 - verifies -> R2
T3 - verifies -> R3
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diff.rs
    section: source
    action: modify
    impl_mode: hand-written
    description: |
      Diff implementation: given spec path, run codegen, compare against current target files.
      pub struct DiffReport { pub files: Vec<FileDiff> }
      pub struct FileDiff { pub path: PathBuf, pub classification: DiffClass, pub drift_pct: f32,
        pub marker_pct: f32, pub coverage_pct: f32 }
      pub enum DiffClass { Exact, MarkerOnly, Drift, Gap }
      pub fn run_diff(spec_path: &Path, project_root: &Path) -> Result<DiffReport>
  - path: projects/agentic-workflow/src/generate/apply.rs
    section: source
    action: modify
    impl_mode: hand-written
    description: |
      Apply implementation: run codegen and write CODEGEN blocks to target files.
      pub fn run_apply(spec_path: &Path, project_root: &Path, dry_run: bool) -> Result<ApplyReport>
      pub fn run_apply_worktree(spec_path: &Path, worktree: &Path) -> Result<ApplyReport>
  - path: projects/agentic-workflow/src/generate/render.rs
    section: source
    action: modify
    impl_mode: hand-written
    description: |
      Render implementation: parse Mermaid Plus frontmatter YAML and regenerate diagram body.
      pub fn run_render(spec_path: &Path, check_only: bool) -> Result<RenderReport>
      Parses YAML between --- markers in mermaid blocks, regenerates Mermaid syntax from YAML.
  - path: projects/agentic-workflow/src/generate/mod.rs
    section: source
    action: modify
    impl_mode: hand-written
    description: Add pub mod diff; pub mod apply; pub mod render; pub mod types; pub mod frontmatter; pub mod gen;
  - path: projects/agentic-workflow/src/cli/commands.rs
    section: source
    action: modify
    impl_mode: hand-written
    description: |
      Maintain top-level CLI routing for code-artifact and validation workflows that invoke
      generator diff/apply/render primitives indirectly.
  - path: projects/agentic-workflow/src/cli/check_alignment.rs
    section: source
    action: modify
    impl_mode: hand-written
    description: |
      Keep spec alignment checks read-only and report format/logical consistency failures.
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
    section: mindmap
    impl_mode: hand-written
    description: "Traceability metadata edge for the mindmap section."

  - action: annotate
    section: requirements
    impl_mode: hand-written
    description: "Traceability metadata edge for the requirements section."

  - action: annotate
    section: rest-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rest-api section."

  - action: annotate
    section: rpc-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rpc-api section."

  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

  - action: annotate
    section: state-machine
    impl_mode: hand-written
    description: "Traceability metadata edge for the state-machine section."

  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

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


## CLI
<!-- type: cli lang: yaml -->

```yaml
command: score gen
description: TD→code codegen pipeline entry points
subcommands:
  - name: diff
    description: Show drift between generated and current code (no file writes)
    args:
      - name: spec_path
        type: string
        description: Path to a spec file (.aw/tech-design/**/*.md)
        required: false
      - name: --all
        type: bool
        description: Run diff for all covered specs in the project
        required: false
      - name: --json
        type: bool
        description: Output diff report as JSON
        required: false
    examples:
      - score gen diff projects/agentic-workflow/tech-design/core/logic/structured-issue.md
      - score gen diff --all
      - score gen diff --all --json

  - name: apply
    description: Write generated code to target files (updates CODEGEN blocks)
    args:
      - name: spec_path
        type: string
        description: Path to a spec file
        required: false
      - name: --all
        type: bool
        description: Apply all covered specs
        required: false
      - name: --dry-run
        type: bool
        description: Print output to stdout, do not write files
        required: false
      - name: --into-worktree
        type: string
        description: Write to specified worktree path instead of project root
        required: false
    examples:
      - score gen apply projects/agentic-workflow/tech-design/core/logic/structured-issue.md
      - score gen apply --all --dry-run

  - name: render
    description: Regenerate Mermaid diagram body from YAML frontmatter in spec file
    args:
      - name: spec_path
        type: string
        description: Path to a spec file with Mermaid Plus blocks
        required: true
      - name: --check
        type: bool
        description: Check if body is in sync (exit 1 if not), do not write
        required: false
    examples:
      - score gen render projects/agentic-workflow/tech-design/core/logic/structured-issue.md
      - score gen render --check projects/agentic-workflow/tech-design/core/logic/state-machine.md

  - name: validate
    description: Check frontmatter schema compliance for a spec file
    args:
      - name: spec_path
        type: string
        description: Path to a spec file
        required: true
      - name: --all
        type: bool
        description: Validate all spec files
        required: false
    examples:
      - score gen validate projects/agentic-workflow/tech-design/core/logic/structured-issue.md

  - name: init-markers
    description: Scaffold CODEGEN-BEGIN/END markers in an existing target file
    args:
      - name: file
        type: string
        description: Target Rust file path
        required: true
      - name: --spec
        type: string
        description: Spec reference (e.g. projects/agentic-workflow/logic/structured-issue.md#schema)
        required: true
      - name: --symbol
        type: string
        description: Symbol name near insertion point (optional)
        required: false
    examples:
      - score gen init-markers projects/agentic-workflow/src/models/state.rs --spec projects/agentic-workflow/logic/state-machine.md#state-phase
```

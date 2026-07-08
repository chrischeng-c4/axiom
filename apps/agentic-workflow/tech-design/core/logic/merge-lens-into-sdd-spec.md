---
id: lens-dissolution-restructure
main_spec_ref: crates/cclab-sdd/logic/merge-lens-into-sdd-spec.md
merge_strategy: new
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "This analysis/standardization logic TD supports brownfield semantic coverage and takeover readiness gates."
---

# Lens Dissolution Restructure

## Overview
<!-- type: doc lang: markdown -->

Dissolve the `lens/` sub-module (90+ files, 17 sub-modules) from `crates/cclab-sdd/src/lens/` into SDD top-level modules. Migrate specs from `.aw/tech-design/crates/cclab-lens/` to `.aw/tech-design/crates/cclab-sdd/`. Remove all MCP tool interfaces formerly exposed by lens. Foundation step that must complete before #944, #946, #949.

| Attribute | Value |
|-----------|-------|
| Crate | cclab-sdd |
| Scope | Module restructure + spec migration + MCP removal |
| Issue | #1087 |
| Blocking | #944 (type propagation), #946 (context builder), #949 (agent output) |
| Modules promoted | core, format, gen, graph, lint, lsp, refactoring, schemas, search, semantic, server, spec, syntax, types (renamed type_inference), diagnostic, error, handlers, output, storage, watch |
## Requirements
<!-- type: doc lang: markdown -->

### Functional

| ID | Requirement | Source |
|----|-------------|--------|
| R1 | Promote all lens sub-modules to `crates/cclab-sdd/src/` top-level: core, format, gen, graph, lint, lsp, refactoring, schemas, search, semantic, server, spec, syntax, types | #1087 tasks |
| R2 | Rename `lens/types/` to `type_inference/` at top-level to avoid collision with Rust keyword and future SDD `types` module | #1087 naming collisions |
| R3 | Promote standalone lens files (`diagnostic.rs`, `error.rs`, `handlers.rs`, `output.rs`, `storage.rs`, `watch.rs`) — merge into existing SDD modules or create new top-level files | #1087 tasks |
| R4 | Remove `pub mod lens` declaration from `lib.rs`; re-export promoted modules from new top-level locations | #1087 tasks |
| R5 | Update all internal `use crate::lens::*` imports to `use crate::{module}::*` across the entire crate | #1087 tasks |
| R6 | Migrate specs from `.aw/tech-design/crates/cclab-lens/` to `.aw/tech-design/crates/cclab-sdd/` under appropriate `interfaces/` or `logic/` subdirectories | #1087 tasks |
| R7 | Remove all MCP tool interfaces formerly exposed by lens (lens-init-spec, etc.) — deregister from cclab-server | Q2 clarification |
| R8 | Delete `crate:lens` GitHub label — all issues become `crate:sdd` | #1087 tasks |
| R9 | Verify all `cclab sdd *` CLI commands still work after restructure | #1087 tasks |

### Non-Functional

| ID | Requirement |
|----|-------------|
| NF1 | Zero behavior change — restructure is purely organizational; all existing functionality preserved |
| NF2 | Must complete before #944, #946, #949 land (blocking dependency per Q1/Q3 clarification) |
| NF3 | lens/mod.rs must be fully deleted — no residual re-export shim |
| NF4 | Spec migration must preserve spec content — only paths change |
## Scenarios
<!-- type: doc lang: markdown -->

### S1: Module promotion — standard sub-module

| Step | Action | Expected |
|------|--------|----------|
| 1 | Move `src/lens/lint/` to `src/lint/` | Directory exists at new location |
| 2 | Update `lib.rs` | `pub mod lint;` at top level (not under `mod lens`) |
| 3 | Update all imports | `use crate::lens::lint::*` becomes `use crate::lint::*` across crate |
| 4 | Build | `cargo check` passes with no unresolved imports |

### S2: Module promotion — naming collision (types → type_inference)

| Step | Action | Expected |
|------|--------|----------|
| 1 | Move `src/lens/types/` to `src/type_inference/` | Directory renamed at new location |
| 2 | Update `lib.rs` | `pub mod type_inference;` declared |
| 3 | Update all imports | `use crate::lens::types::*` becomes `use crate::type_inference::*` |
| 4 | Internal references | `types::DeepTypeInferencer` becomes `type_inference::DeepTypeInferencer` |
| 5 | Build | No compilation errors |

### S3: Standalone file promotion

| Step | Action | Expected |
|------|--------|----------|
| 1 | Move `src/lens/diagnostic.rs` to `src/diagnostic.rs` | File at top level |
| 2 | Move `src/lens/error.rs` to `src/error.rs` (or merge into existing) | No duplicate `mod error` |
| 3 | Update `lib.rs` | Top-level module declarations for each promoted file |
| 4 | Build | Passes |

### S4: MCP deregistration

| Step | Action | Expected |
|------|--------|----------|
| 1 | Remove lens MCP tool registrations from cclab-server | No lens tools in `cclab server list` output |
| 2 | Remove lens MCP tool interface specs | Specs deleted from cclab-lens spec directory |
| 3 | Verify remaining SDD MCP tools still registered | `cclab server list` shows SDD workflow/utility tools |

### S5: Spec migration

| Step | Action | Expected |
|------|--------|----------|
| 1 | Move `.aw/tech-design/crates/cclab-lens/*.md` to `.aw/tech-design/crates/cclab-sdd/` subdirs | Files in `logic/` or `interfaces/` as appropriate |
| 2 | Update any cross-references in moved specs | No broken spec links |
| 3 | Delete `.aw/tech-design/crates/cclab-lens/` directory | Directory removed |

### S6: Post-restructure CLI verification

| Step | Action | Expected |
|------|--------|----------|
| 1 | `cclab sdd check src/test.py` | Lint works (uses lint/, semantic/, syntax/) |
| 2 | `cclab sdd hover src/test.py:10:5` | Hover works (uses semantic/, type_inference/) |
| 3 | `cclab sdd search "query"` | Search works (uses search/, semantic/) |
| 4 | `cclab sdd rename src/test.py:10:5 new_name` | Refactoring works (uses refactoring/) |
| 5 | `cclab sdd format src/test.py` | Format detection works (uses format/) |

| Scenario | Covers |
|----------|--------|
| S1 | R1, R4, R5 |
| S2 | R1, R2, R5 |
| S3 | R3, R4 |
| S4 | R7 |
| S5 | R6, R8 |
| S6 | R9 |
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

### Build Verification

| Test | Validates |
|------|-----------|
| `cargo check -p cclab-sdd` | All promoted modules compile without errors (R1, R2, R3, R4, R5) |
| `cargo check -p cclab-sdd-cli` | CLI crate compiles with updated import paths (R5) |
| `cargo check -p cclab-server` | Server compiles after MCP deregistration (R7) |
| `cargo test -p cclab-sdd` | All existing unit tests pass unchanged (NF1) |
| `cargo test -p cclab-sdd-cli` | All existing CLI tests pass (NF1) |

### Import Verification

| Test | Validates |
|------|-----------|
| `grep -r 'crate::lens::' crates/cclab-sdd/src/` returns 0 matches | No residual `crate::lens::` imports remain (R5) |
| `grep -r 'pub mod lens' crates/cclab-sdd/src/lib.rs` returns 0 matches | `pub mod lens` removed (R4) |
| `test -d crates/cclab-sdd/src/lens/` returns false | lens/ directory fully deleted (NF3) |

### CLI Integration Tests

| Test | Validates |
|------|-----------|
| `cclab sdd check fixtures/python/` | Lint pipeline works post-restructure (R9, S6) |
| `cclab sdd hover fixtures/python/main.py:1:1` | Hover/semantic works post-restructure (R9, S6) |
| `cclab sdd search "test"` | Search works post-restructure (R9, S6) |

### MCP Deregistration Tests

| Test | Validates |
|------|-----------|
| `cclab server list` shows no lens-specific tools | Lens MCP tools removed (R7, S4) |
| `cclab server list` shows SDD workflow/utility tools | Non-lens MCP tools unaffected (R7) |

### Spec Migration Tests

| Test | Validates |
|------|-----------|
| `test -d .aw/tech-design/crates/cclab-lens/` returns false | Old spec directory deleted (R6) |
| `ls .aw/tech-design/crates/cclab-sdd/logic/` contains migrated lens specs | Specs migrated to correct location (R6, NF4) |
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  # Phase 1: Promote lens sub-module directories
  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/core/
    to: crates/cclab-sdd/src/core/
    description: "Promote core/ (config) to top-level"
    requirements: [R1]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/format/
    to: crates/cclab-sdd/src/format/
    description: "Promote format/ (file format detection) to top-level"
    requirements: [R1]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/gen/
    to: crates/cclab-sdd/src/gen/
    description: "Promote gen/ (codegen: Python, Rust, framework scaffolds) to top-level"
    requirements: [R1]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/graph/
    to: crates/cclab-sdd/src/graph/
    description: "Promote graph/ (import/dependency graph resolution) to top-level"
    requirements: [R1]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/lint/
    to: crates/cclab-sdd/src/lint/
    description: "Promote lint/ (40+ lint rules across 18+ file types) to top-level"
    requirements: [R1]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/lsp/
    to: crates/cclab-sdd/src/lsp/
    description: "Promote lsp/ (LSP server) to top-level"
    requirements: [R1]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/refactoring/
    to: crates/cclab-sdd/src/refactoring/
    description: "Promote refactoring/ (rename, extract, inline, move, change-signature) to top-level"
    requirements: [R1]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/schemas/
    to: crates/cclab-sdd/src/schemas/
    description: "Promote schemas/ (K8s, GitLab CI, frontmatter schemas) to top-level"
    requirements: [R1]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/search/
    to: crates/cclab-sdd/src/search/
    description: "Promote search/ (semantic search index + query) to top-level"
    requirements: [R1]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/semantic/
    to: crates/cclab-sdd/src/semantic/
    description: "Promote semantic/ (symbols, scope, PDG) to top-level"
    requirements: [R1]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/server/
    to: crates/cclab-sdd/src/server/
    description: "Promote server/ (daemon, disk_cache, incremental, watch_bridge) to top-level"
    requirements: [R1]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/spec/
    to: crates/cclab-sdd/src/spec/
    description: "Promote spec/ (spec IR parsers: OpenAPI, AsyncAPI, Mermaid, JSON Schema) to top-level"
    requirements: [R1]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/syntax/
    to: crates/cclab-sdd/src/syntax/
    description: "Promote syntax/ (tree-sitter parser) to top-level"
    requirements: [R1]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/types/
    to: crates/cclab-sdd/src/type_inference/
    description: "Promote types/ as type_inference/ — renamed to avoid Rust keyword collision"
    requirements: [R1, R2]

  # Phase 2: Promote standalone lens files
  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/diagnostic.rs
    to: crates/cclab-sdd/src/diagnostic.rs
    description: "Promote diagnostic.rs to top-level"
    requirements: [R3]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/error.rs
    to: crates/cclab-sdd/src/lens_error.rs
    description: "Promote error.rs as lens_error.rs to avoid collision with any existing error module"
    requirements: [R3]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/handlers.rs
    to: crates/cclab-sdd/src/handlers.rs
    description: "Promote handlers.rs (request routing) to top-level"
    requirements: [R3]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/output.rs
    to: crates/cclab-sdd/src/output.rs
    description: "Promote output.rs to top-level (or merge with existing if present)"
    requirements: [R3]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/storage.rs
    to: crates/cclab-sdd/src/storage.rs
    description: "Promote storage.rs to top-level"
    requirements: [R3]

  - action: move
    section: source
    impl_mode: hand-written
    from: crates/cclab-sdd/src/lens/watch.rs
    to: crates/cclab-sdd/src/watch.rs
    description: "Promote watch.rs to top-level"
    requirements: [R3]

  # Phase 3: Update lib.rs and imports
  - action: modify
    section: source
    impl_mode: hand-written
    path: crates/cclab-sdd/src/lib.rs
    description: >
      Remove `pub mod lens;` declaration.
      Add top-level module declarations for all promoted modules:
      pub mod core, format, gen, graph, lint, lsp, refactoring, schemas,
      search, semantic, server, spec, syntax, type_inference, diagnostic,
      lens_error, handlers, output, storage, watch.
    requirements: [R1, R4]

  - action: modify
    section: source
    impl_mode: hand-written
    path: crates/cclab-sdd/src/**/*.rs
    description: >
      Global find-replace across all .rs files in the crate:
      `use crate::lens::types::` → `use crate::type_inference::`
      `use crate::lens::{module}::` → `use crate::{module}::`
      `crate::lens::` → `crate::{corresponding_module}::`
    requirements: [R5]

  # Phase 4: Remove lens module
  - action: delete
    section: source
    impl_mode: hand-written
    path: crates/cclab-sdd/src/lens/mod.rs
    description: "Delete lens module root — all content has been promoted"
    requirements: [NF3]

  - action: delete
    section: source
    impl_mode: hand-written
    path: crates/cclab-sdd/src/lens/
    description: "Delete empty lens/ directory"
    requirements: [NF3]

  # Phase 5: MCP deregistration
  - action: modify
    section: source
    impl_mode: hand-written
    path: crates/cclab-server/src/
    description: >
      Remove all lens MCP tool registrations from cclab-server.
      Delete lens-specific tool handler code. Existing SDD MCP tools remain.
    requirements: [R7]

  # Phase 6: Spec migration
  - action: move
    section: source
    impl_mode: hand-written
    from: .aw/tech-design/crates/cclab-lens/
    to: .aw/tech-design/crates/cclab-sdd/
    description: >
      Migrate all 20 lens spec files into cclab-sdd spec directory.
      Classify each into logic/ or interfaces/ subdirectory based on content.
      Update cross-references. Delete cclab-lens/ spec directory.
    requirements: [R6, NF4]
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

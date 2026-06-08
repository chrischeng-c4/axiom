---
id: scope-resolution-update
main_spec_ref: crates/cclab-sdd/logic/scope-resolution.md
merge_strategy: extend
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "This logic TD supports source/spec alignment and traceability closure."
---

# Sdd Spec Scope Config Spec

## Overview
<!-- type: doc lang: markdown -->

Config-driven spec scope resolution for SDD. Replaces the hardcoded `crates/ → projects/ → root` probe in `workflow/scope.rs::pre_filter_specs()` and `services/file_service.rs::read_main_spec_scoped()` with a `[specs.scopes]` table in `.aw/config.toml`. Introduces `resolve_spec_dir()` as the canonical lookup function, adds workspace-type auto-detection to `cclab sdd init`, and creates `FileSystemSpecStore` in `cclab-agent` as the first concrete `SpecStore` implementation.

**Affected crates**: `cclab-sdd` (models, scope, file_service, init) · `cclab-agent` (new spec_store.rs)

**Motivation**: Hardcoded `crates/` / `projects/` subdirectories prevent non-Rust monorepos and custom layouts from resolving specs correctly. Making resolution table-driven via `config.toml` lets each project declare its own layout while preserving backward-compatible fallback.
## Requirements
<!-- type: doc lang: markdown -->

| ID | Requirement |
|----|-------------|
| REQ-1 | `SddConfig` gains `specs: SpecsConfig` field. `SpecsConfig.scopes: HashMap<String, String>` maps group name → parent subdirectory under `.aw/tech-design/`. Serialized as `[specs.scopes]` in TOML. |
| REQ-2 | `resolve_spec_dir(group: &str, specs_base: &Path, scopes: &HashMap<String, String>) -> Option<PathBuf>` — if `scopes` contains `group`, returns `specs_base / scopes[group] / group`; otherwise falls back to `crates/group → projects/group → group` probes. |
| REQ-3 | `pre_filter_specs` in `scope.rs` accepts an optional `Option<&SddConfig>` parameter; uses `resolve_spec_dir` when `scopes` is non-empty, falls back to original probes otherwise. Existing callers pass `None` if config unavailable. |
| REQ-4 | `read_main_spec_scoped` in `file_service.rs` loads config via `SddConfig::load(project_root)` and delegates dir resolution to `resolve_spec_dir`. Falls back to hardcoded probes if config load fails or `scopes` is empty. |
| REQ-5 | `cclab sdd init` fresh-install detects workspace type: (a) `Cargo.toml` containing `[workspace]` → Rust monorepo; (b) `pyproject.toml` → Python project; (c) `package.json` → JS/TS project. Writes matching default `[specs.scopes]` entries in generated `config.toml`. |
| REQ-6 | New `crates/cclab-agent/src/spec_store.rs` defines `FileSystemSpecStore { root: PathBuf, scopes: HashMap<String, String> }` implementing `SpecStore` trait (`search` + `read`). Constructor `FileSystemSpecStore::new(root, scopes)` or `::from_config(root, config)`. |
| REQ-7 | Projects without `[specs.scopes]` (missing key or empty map) continue to work — fallback `crates/ → projects/ → root` probe unchanged. |
| REQ-8 | `validate_path_component` traversal protection in `file_service.rs` remains intact after refactor. |
| REQ-9 | `SddConfig::default()` and existing tests remain passing; no new required fields. |
## Scenarios
<!-- type: doc lang: markdown -->

### Scenario: Config scope resolves group directory

- **GIVEN** `config.toml` contains `[specs.scopes]` with `cclab-sdd = "crates"`
- **AND** `.aw/tech-design/crates/cclab-sdd/` exists on disk
- **WHEN** `resolve_spec_dir("cclab-sdd", specs_base, scopes)` is called
- **THEN** returns `Some(specs_base/crates/cclab-sdd)`

### Scenario: Fallback when group not in config

- **GIVEN** `[specs.scopes]` is absent or has no entry for group `my-crate`
- **AND** `.aw/tech-design/crates/my-crate/` exists on disk
- **WHEN** `resolve_spec_dir("my-crate", specs_base, {})` is called
- **THEN** returns `Some(specs_base/crates/my-crate)` via fallback probe

### Scenario: Group not found anywhere

- **GIVEN** `[specs.scopes]` has no entry for `unknown-group`
- **AND** `.aw/tech-design/crates/unknown-group/`, `.aw/tech-design/projects/unknown-group/`, `.aw/tech-design/unknown-group/` all absent
- **WHEN** `resolve_spec_dir("unknown-group", specs_base, {})` is called
- **THEN** returns `None`

### Scenario: pre_filter_specs uses configured subdir

- **GIVEN** config has `cclab-lens = "crates"`
- **AND** `.aw/tech-design/crates/cclab-lens/` contains `parser.md`, `semantic.md`
- **WHEN** `pre_filter_specs(&["cclab-lens"], project_root)` is called
- **THEN** output includes `### cclab-lens` and both spec names

### Scenario: read_main_spec_scoped uses configured subdir

- **GIVEN** config has `cclab-sdd = "crates"`
- **AND** `.aw/tech-design/crates/cclab-sdd/logic/state-machine.md` exists
- **WHEN** `read_file(change_id, "main_spec:cclab-sdd/logic/state-machine", project_root)` is called
- **THEN** returns the file content with correct path header

### Scenario: cclab sdd init Cargo.toml workspace detection

- **GIVEN** `Cargo.toml` in project root contains `[workspace]` section
- **AND** `cclab sdd init` is run for fresh install
- **WHEN** workspace detection runs
- **THEN** generated `config.toml` includes `[specs.scopes]` with `crates` and `projects` default entries

### Scenario: cclab sdd init pyproject.toml detection

- **GIVEN** `pyproject.toml` exists AND no `Cargo.toml [workspace]`
- **WHEN** `cclab sdd init` runs
- **THEN** generated `config.toml` includes `[specs.scopes]` with Python project defaults

### Scenario: FileSystemSpecStore search

- **GIVEN** `FileSystemSpecStore` configured with `root` and `scopes`
- **WHEN** `spec_store.search("scope resolution").await` is called
- **THEN** returns `Vec<SpecExcerpt>` ranked by relevance, sourced from spec files under configured scope dirs

### Scenario: FileSystemSpecStore read

- **GIVEN** `FileSystemSpecStore` with `root = /project` and `scopes = {"cclab-sdd": "crates"}`
- **WHEN** `spec_store.read("crates/cclab-sdd/logic/state-machine.md").await` is called
- **THEN** returns file content from `{root}/.aw/tech-design/crates/cclab-sdd/logic/state-machine.md`

### Scenario: Config-scoped entry does not exist on disk

- **GIVEN** `config.toml` has `cclab-foo = "crates"` but `.aw/tech-design/crates/cclab-foo/` does not exist
- **WHEN** `resolve_spec_dir("cclab-foo", specs_base, scopes)` is called
- **THEN** returns `None` (does not attempt fallback probe for explicitly configured groups)
## Diagrams
<!-- type: doc lang: markdown -->

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- score-td-placeholder -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- score-td-placeholder -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- score-td-placeholder -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- score-td-placeholder -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- score-td-placeholder -->

## API Spec
<!-- type: doc lang: markdown -->

### REST API
<!-- type: rest-api lang: yaml -->
<!-- score-td-placeholder -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- score-td-placeholder -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- score-td-placeholder -->

### CLI
<!-- type: cli lang: yaml -->
<!-- score-td-placeholder -->

### Schema
<!-- type: schema lang: json -->
<!-- score-td-placeholder -->

### Config
<!-- type: config lang: json -->
<!-- score-td-placeholder -->

## Test Plan
<!-- type: doc lang: markdown -->

```mermaid
requirementDiagram

    requirement REQ1 {
        id: REQ-1
        text: SddConfig.specs.scopes deserialized from [specs.scopes] TOML
        risk: low
        verifyMethod: test
    }
    requirement REQ2 {
        id: REQ-2
        text: resolve_spec_dir returns config path when group in scopes
        risk: medium
        verifyMethod: test
    }
    requirement REQ3 {
        id: REQ-3
        text: pre_filter_specs uses resolved dir
        risk: medium
        verifyMethod: test
    }
    requirement REQ4 {
        id: REQ-4
        text: read_main_spec_scoped uses resolved dir
        risk: medium
        verifyMethod: test
    }
    requirement REQ5 {
        id: REQ-5
        text: cclab sdd init writes [specs.scopes]
        risk: low
        verifyMethod: test
    }
    requirement REQ6 {
        id: REQ-6
        text: FileSystemSpecStore search and read
        risk: high
        verifyMethod: test
    }
    requirement REQ7 {
        id: REQ-7
        text: Backward compat: no scopes → fallback probe
        risk: high
        verifyMethod: test
    }

    element TC_config_deser {
        type: test
        docref: models/change.rs
        text: Parse TOML with [specs.scopes]; verify scopes map populated
    }
    element TC_resolve_hit {
        type: test
        docref: workflow/scope.rs
        text: resolve_spec_dir with matching entry returns Some(configured path)
    }
    element TC_resolve_miss_fallback {
        type: test
        docref: workflow/scope.rs
        text: resolve_spec_dir without entry falls back to crates/ probe
    }
    element TC_resolve_miss_none {
        type: test
        docref: workflow/scope.rs
        text: resolve_spec_dir returns None when group absent everywhere
    }
    element TC_pre_filter_config {
        type: test
        docref: workflow/scope.rs
        text: pre_filter_specs with config finds specs in configured subdir
    }
    element TC_file_service_config {
        type: test
        docref: services/file_service.rs
        text: read_main_spec_scoped reads from configured path
    }
    element TC_file_service_fallback {
        type: test
        docref: services/file_service.rs
        text: read_main_spec_scoped falls back when no config
    }
    element TC_init_cargo {
        type: test
        docref: cli/init.rs
        text: detect_workspace_type returns RustCargo when Cargo.toml has [workspace]
    }
    element TC_init_python {
        type: test
        docref: cli/init.rs
        text: detect_workspace_type returns Python when pyproject.toml exists
    }
    element TC_init_node {
        type: test
        docref: cli/init.rs
        text: detect_workspace_type returns NodeJs when package.json exists
    }
    element TC_fs_store_search {
        type: test
        docref: cclab-agent/spec_store.rs
        text: FileSystemSpecStore.search returns ranked excerpts
    }
    element TC_fs_store_read {
        type: test
        docref: cclab-agent/spec_store.rs
        text: FileSystemSpecStore.read returns file contents by path
    }
    element TC_backward_compat {
        type: test
        docref: workflow/scope.rs
        text: Empty scopes triggers crates/ → projects/ → root fallback unchanged
    }

    TC_config_deser - verifies -> REQ1
    TC_resolve_hit - verifies -> REQ2
    TC_resolve_miss_fallback - verifies -> REQ7
    TC_resolve_miss_none - verifies -> REQ2
    TC_pre_filter_config - verifies -> REQ3
    TC_file_service_config - verifies -> REQ4
    TC_file_service_fallback - verifies -> REQ4
    TC_init_cargo - verifies -> REQ5
    TC_init_python - verifies -> REQ5
    TC_init_node - verifies -> REQ5
    TC_fs_store_search - verifies -> REQ6
    TC_fs_store_read - verifies -> REQ6
    TC_backward_compat - verifies -> REQ7
```
## Changes
<!-- type: doc lang: markdown -->

### Fix: pre_filter_specs — Recursive Directory Walk

REQ-3 specifies config-driven directory selection but the implementation uses flat `read_dir`, missing spec files in subdirectories. Update REQ-3 to require recursive walk:

| ID | Before | After |
|----|--------|-------|
| REQ-3 | `pre_filter_specs` accepts `Option<&SddConfig>`; uses `resolve_spec_dir` when scopes is non-empty, falls back to original probes otherwise. | Same directory selection logic, plus: **walks the resolved directory recursively** (depth-first, all nesting levels) to enumerate every `.md` file — not just root-level files via flat `read_dir`. |

### Add: build_spec_dir_tree Function Spec

New function in `workflow/scope.rs` for spec-consolidation lifecycle enforcement. Insert as new section `## Spec Tree Generation` after `## Requirements`:

```yaml
function: build_spec_dir_tree
module: workflow/scope.rs
signature: >
  pub fn build_spec_dir_tree(spec_groups: &[String], project_root: &Path, config: Option<&SddConfig>) -> String
behavior:
  - For each group in spec_groups, call resolve_spec_dir to get spec root path
  - If path exists on disk, walk directory recursively
  - Render ASCII tree using \u251C\u2500\u2500 / \u2514\u2500\u2500 / \u2502   prefix (same format as `tree` CLI)
  - Groups with no existing spec directory are silently omitted
  - Return empty string if no spec directories found
used_by: build_create_prompt in workflow/reference_context.rs — substituted as {{spec_dir_tree}} variable
```

### Add: Recursive Listing Scenarios

Add two new scenarios after "Scenario: pre_filter_specs uses configured subdir":

#### Scenario: pre_filter_specs discovers nested spec files

- **GIVEN** config has `cclab-sdd = "crates"` and `.aw/tech-design/crates/cclab-sdd/logic/state-machine.md` exists at depth 2
- **WHEN** `pre_filter_specs(&["cclab-sdd"], project_root)` is called
- **THEN** `state-machine.md` appears in output — not omitted due to subdirectory depth

#### Scenario: build_spec_dir_tree renders ASCII tree for prompt injection

- **GIVEN** `.aw/tech-design/crates/cclab-sdd/` contains `logic/state-machine.md`, `logic/change-spec.md`, `interfaces/tools/artifact-tools.md`
- **WHEN** `build_spec_dir_tree(&["cclab-sdd"], project_root, None)` is called
- **THEN** returns a multi-line ASCII tree (tree CLI format) listing all three nested files under `cclab-sdd/`
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

## Traceability Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
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
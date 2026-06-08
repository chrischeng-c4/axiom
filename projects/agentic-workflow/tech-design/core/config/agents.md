---
id: config-docs-section
main_spec_ref: "crates/cclab-sdd/config/agents.md"
merge_strategy: new
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Config and platform TDs define AW Core client boundary behavior."
---

# Config Docs Section

## Overview
<!-- type: overview lang: markdown -->

Extend the SDD config schema (`agents.md`) with docs generation phase configuration. Adds `[agentic_workflow.docs]` section to `.aw/config.toml` and two new agent definitions (`sdd-doc-writer`, `sdd-doc-reviewer`) to the workflow preset tables.

**What changes in `agents.md`**:

| Addition | Section | Purpose |
|----------|---------|---------|
| `[agentic_workflow.docs]` config schema | New top-level section | Doc generation output directory + target array |
| `[[agentic_workflow.docs.targets]]` schema | Under `[agentic_workflow.docs]` | Per-crate guide path, audience, sections |
| `sdd-doc-writer` agent def | Agent Definitions table | Dedicated doc-writer agent with Write + Read tools |
| `sdd-doc-reviewer` agent def | Agent Definitions table | Doc-reviewer agent with Bash(read-only) + Read, no Write |
| Docs-phase actions | Workflow preset tables | `create_change_docs`, `review_change_docs`, `revise_change_docs` routing |

**Design decisions**:
- Presence of `[agentic_workflow.docs]` = enabled; no `enabled` flag (same pattern as `[agentic_workflow.repo_platform]`)
- `output_dir` defaults to `"docs"` relative to project root (per clarification Q3)
- `sdd-doc-writer` is a dedicated agent type, not reusing existing agents (per clarification Q1)
- `sdd-doc-reviewer` has Bash (read-only by prompt) to verify CLI output matches docs (per clarification Q2, C2)

**Scope**: Config schema documentation only — no code changes. Current docs-phase source ownership lives in the per-file `core/tools/*_change_docs/` specs, with model/state coverage in `core/interfaces/models/change.md` and `core/logic/state-machine.md`.
## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: config-docs-section-requirements
title: Config Docs Section Requirements
requirements:
  R1:
    text: Add [agentic_workflow.docs] section documenting output_dir field
    type: functional
    priority: high
    risk: low
    verification: inspection
    notes: |
      output_dir - string, default "docs", relative to project root.
      Presence of section = enabled; no enabled flag.
  R2:
    text: Add [[agentic_workflow.docs.targets]] subsection documenting target array fields
    type: functional
    priority: high
    risk: low
    verification: inspection
    notes: |
      Fields - crate (string, required), guide (string, required),
      audience (enum developer|end-user|admin, required),
      sections (string array, required).
  R3:
    text: Add sdd-doc-writer agent definition to Agent Definitions table
    type: functional
    priority: high
    risk: low
    verification: inspection
    notes: |
      tools = Read, Write, Edit, Glob, Grep, Bash;
      disallowedTools = Agent; model = opus; maxTurns = 40;
      Bash Hook = sdd-safe-bash.sh.
  R4:
    text: Add sdd-doc-reviewer agent definition to Agent Definitions table
    type: functional
    priority: high
    risk: low
    verification: inspection
    notes: |
      tools = Read, Glob, Grep, Bash;
      disallowedTools = Write, Edit, Agent; model = sonnet; maxTurns = 20;
      Bash Hook = sdd-readonly-bash.sh.
      Doc-reviewer verifies accuracy by executing CLI commands.
  R5:
    text: Extend all 4 workflow preset tables with docs-phase action rows
    type: functional
    priority: high
    risk: low
    verification: inspection
    notes: |
      Presets - multi_agents, multi_claude_agents, claude_subagents, mainthread.
      Actions - create_change_docs, review_change_docs, revise_change_docs.
  R6:
    text: Document DocsConfig and DocsTarget Rust struct field mapping
    type: interface
    priority: medium
    risk: low
    verification: inspection
    notes: |
      Analogous to existing RepoPlatformConfig mapping table.
      Both structs loaded via SddConfig::load() under agentic_workflow.docs TOML namespace.
  R7:
    text: Document validation — load_validated() does NOT require [agentic_workflow.docs]
    type: constraint
    priority: medium
    risk: low
    verification: inspection
    notes: |
      Section is optional. When absent, docs phase is skipped at DocsCheck state.
---
requirementDiagram
    requirement R1 {
      id: R1
      text: Add [agentic_workflow.docs] section documenting output_dir
      risk: low
      verifymethod: inspection
    }
    requirement R2 {
      id: R2
      text: Add [[agentic_workflow.docs.targets]] subsection
      risk: low
      verifymethod: inspection
    }
    requirement R3 {
      id: R3
      text: Add sdd-doc-writer agent definition
      risk: low
      verifymethod: inspection
    }
    requirement R4 {
      id: R4
      text: Add sdd-doc-reviewer agent definition
      risk: low
      verifymethod: inspection
    }
    requirement R5 {
      id: R5
      text: Extend workflow preset tables with docs-phase actions
      risk: low
      verifymethod: inspection
    }
    requirement R6 {
      id: R6
      text: Document DocsConfig struct field mapping
      risk: low
      verifymethod: inspection
    }
    requirement R7 {
      id: R7
      text: [agentic_workflow.docs] is optional for load_validated()
      risk: low
      verifymethod: inspection
    }
```
## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  S1:
    name: Config spec documents [agentic_workflow.docs] section
    verifies: [R1]
    given: |
      agents.md is merged with this change spec
    when: |
      A reader looks up [agentic_workflow.docs] in the config schema
    then: |
      - Spec contains a dedicated ## [agentic_workflow.docs] section with output_dir field documentation (type, default, resolution)
      - Section states that presence = enabled (no enabled flag)
  S2:
    name: Config spec documents [[agentic_workflow.docs.targets]] array
    verifies: [R2]
    given: |
      agents.md includes the [agentic_workflow.docs] section
    when: |
      A reader looks up target configuration
    then: |
      - Spec contains [[agentic_workflow.docs.targets]] subsection with field table - crate (required), guide (required), audience (required, enum), sections (required, array)
      - Includes a TOML example showing multiple targets
  S3:
    name: Agent definitions include doc-writer
    verifies: [R3]
    given: |
      agents.md Agent Definitions table exists
    when: |
      The spec is read after merge
    then: |
      sdd-doc-writer row is present with tools = Read, Write, Edit, Glob, Grep, Bash;
      disallowedTools = Agent; model = opus; maxTurns = 40; Bash Hook = sdd-safe-bash.sh
  S4:
    name: Agent definitions include doc-reviewer
    verifies: [R4]
    given: |
      agents.md Agent Definitions table exists
    when: |
      The spec is read after merge
    then: |
      - sdd-doc-reviewer row is present with tools = Read, Glob, Grep, Bash; disallowedTools = Write, Edit, Agent; model = sonnet; maxTurns = 20; Bash Hook = sdd-readonly-bash.sh
      - Doc-reviewer verifies accuracy by executing CLI commands
  S5:
    name: Workflow preset tables extended with docs-phase actions
    verifies: [R5]
    given: |
      agents.md has 4 workflow preset tables
    when: |
      The spec is read after merge
    then: |
      - Each preset table includes 3 new rows - create_change_docs, review_change_docs, revise_change_docs
      - multi_claude_agents maps to sdd-doc-writer (opus) for create/revise, sdd-doc-reviewer (sonnet) for review
      - mainthread routes all 3 to mainthread
  S6:
    name: "[agentic_workflow.docs] is optional — not validated by load_validated()"
    verifies: [R7]
    given: |
      .aw/config.toml has no [agentic_workflow.docs] section
    when: |
      SddConfig::load_validated() runs
    then: |
      - Validation passes (unlike repo_platform / spec_platform which are required)
      - docs field is None — docs phase skips at DocsCheck state
```
## Diagrams
<!-- type: diagram lang: mermaid -->

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
<!-- type: api lang: yaml -->

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
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: config-docs-section-test-plan
title: Config Docs Section Test Plan
tests:
  T1:
    type: test
    name: spec_contains_sdd_docs_section
    file: projects/agentic-workflow/tech-design/core/config/agents.md
    verifies: [R1]
  T2:
    type: test
    name: spec_contains_sdd_docs_targets_subsection
    file: projects/agentic-workflow/tech-design/core/config/agents.md
    verifies: [R2]
  T3:
    type: test
    name: spec_contains_sdd_doc_writer_row
    file: projects/agentic-workflow/tech-design/core/config/agents.md
    verifies: [R3]
  T4:
    type: test
    name: spec_contains_sdd_doc_reviewer_row
    file: projects/agentic-workflow/tech-design/core/config/agents.md
    verifies: [R4]
  T5:
    type: test
    name: workflow_preset_tables_contain_docs_phase_rows
    file: projects/agentic-workflow/tech-design/core/config/agents.md
    verifies: [R5]
  T6:
    type: test
    name: struct_field_mapping_documented
    file: projects/agentic-workflow/tech-design/core/config/agents.md
    verifies: [R6]
  T7:
    type: test
    name: sdd_docs_optional_in_load_validated
    file: crates/cclab-sdd/src/models/change.rs
    verifies: [R7]
---
requirementDiagram
    element T1 { type: test }
    element T2 { type: test }
    element T3 { type: test }
    element T4 { type: test }
    element T5 { type: test }
    element T6 { type: test }
    element T7 { type: test }

    T1 - verifies -> R1
    T2 - verifies -> R2
    T3 - verifies -> R3
    T4 - verifies -> R4
    T5 - verifies -> R5
    T6 - verifies -> R6
    T7 - verifies -> R7
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: .aw/tech-design/crates/cclab-sdd/config/agents.md
    section: config
    action: modify
    impl_mode: hand-written
    description: |
      Append new sections to the SDD config schema spec after the existing
      "Config Loading" section:

      --- [agentic_workflow.docs] ---

      Field table:
        output_dir (string, default "docs" — relative to project root)
        targets (array of DocsTarget, required, minItems 1)

      Presence of [agentic_workflow.docs] section = enabled. No enabled flag.
      Not required by load_validated() — when absent, docs phase
      skips at DocsCheck state.

      --- [[agentic_workflow.docs.targets]] ---

      Field table:
        crate (string, required — matches against change-affected crates)
        guide (string, required — output guide file path relative to project root)
        audience (string, required — enum: developer | end-user | admin)
        sections (string array, required, minItems 1 — guide section names)

      TOML example showing multiple targets.

      --- SddConfig Field Mapping (append row) ---

      | [agentic_workflow.docs] | docs | Option<DocsConfig> | #[serde(default)] | None | NOT required |

      --- Agent Definitions (append 2 rows) ---

      | sdd-doc-writer | Read,Write,Edit,Glob,Grep,Bash | Agent | opus | 40 | sdd-safe-bash.sh |
      | sdd-doc-reviewer | Read,Glob,Grep,Bash | Write,Edit,Agent | sonnet | 20 | sdd-readonly-bash.sh |

      --- Workflow Preset Tables (append 3 rows each) ---

      multi_agents:        create_change_docs → gemini:pro, review → codex:balanced, revise → gemini:pro
      multi_claude_agents: create_change_docs → sdd-doc-writer/opus, review → sdd-doc-reviewer/sonnet, revise → sdd-doc-writer/opus
      claude_subagents:    create_change_docs → subagent:general-purpose:opus, review → subagent:general-purpose:sonnet, revise → subagent:general-purpose:opus
      mainthread:          all 3 → mainthread

      --- Config Loading (update step 1) ---

      Add 'docs' to the loads list in step 1_primary_deserialize.
      Add agentic_workflow.docs → config.docs to step 2_overlay_extraction.
  - action: annotate
    section: async-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the async-api section."

  - action: annotate
    section: cli
    impl_mode: hand-written
    description: "Traceability metadata edge for the cli section."

  - action: annotate
    section: db-model
    impl_mode: hand-written
    description: "Traceability metadata edge for the db-model section."

  - action: annotate
    section: dependency
    impl_mode: hand-written
    description: "Traceability metadata edge for the dependency section."

  - action: annotate
    section: interaction
    impl_mode: hand-written
    description: "Traceability metadata edge for the interaction section."

  - action: annotate
    section: logic
    impl_mode: hand-written
    description: "Traceability metadata edge for the logic section."

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

```
## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->


## Config
<!-- type: config lang: json -->
<!-- score-td-placeholder -->

New fields added to `SddConfig` (loaded from `.aw/config.toml`).

### `[agentic_workflow.docs]`

```json
{
  "$id": "sdd-config-docs",
  "title": "DocsConfig",
  "description": "Docs generation phase config — [agentic_workflow.docs] in .aw/config.toml. Presence = enabled.",
  "type": "object",
  "properties": {
    "output_dir": {
      "type": "string",
      "default": "docs",
      "description": "Output directory for generated docs, relative to project root"
    },
    "targets": {
      "type": "array",
      "items": { "$ref": "#/$defs/DocsTarget" },
      "minItems": 1,
      "description": "Per-crate doc generation targets. At least one required when section present."
    }
  },
  "required": ["targets"],
  "additionalProperties": false,
  "$defs": {
    "DocsTarget": {
      "$id": "sdd-config-docs-target",
      "title": "DocsTarget",
      "description": "Single doc generation target — [[agentic_workflow.docs.targets]] in config.toml",
      "type": "object",
      "properties": {
        "crate": {
          "type": "string",
          "description": "Crate name to match against change-affected crates"
        },
        "guide": {
          "type": "string",
          "description": "Output guide file path relative to project root (e.g. docs/sdd-user-guide.md)"
        },
        "audience": {
          "type": "string",
          "enum": ["developer", "end-user", "admin"],
          "description": "Target audience — controls tone, detail level, example style"
        },
        "sections": {
          "type": "array",
          "items": { "type": "string" },
          "minItems": 1,
          "description": "Guide section names to generate/update (e.g. getting-started, cli-reference)"
        }
      },
      "required": ["crate", "guide", "audience", "sections"],
      "additionalProperties": false
    }
  }
}
```

### SddConfig Field Mapping

| Config Section | Rust Field | Type | Serde | Default | Validated |
|----------------|-----------|------|-------|---------|----------|
| `[agentic_workflow.docs]` | `docs` | `Option<DocsConfig>` | `#[serde(default, skip_serializing_if = "Option::is_none")]` | `None` | NOT required by `load_validated()` |

### DocsConfig Struct Fields

| Field | Type | Serde Attribute | Default Function | Value |
|-------|------|----------------|-----------------|-------|
| `output_dir` | `String` | `#[serde(default = "default_docs_dir")]` | `default_docs_dir()` | `"docs"` |
| `targets` | `Vec<DocsTarget>` | — | — | required |

### DocsTarget Struct Fields

| Field | Type | Serde Attribute | Default | Required |
|-------|------|----------------|---------|----------|
| `crate` | `String` | — | — | yes |
| `guide` | `String` | — | — | yes |
| `audience` | `String` | — | — | yes |
| `sections` | `Vec<String>` | — | — | yes |

### Agent Definitions (append to existing table)

| Agent | tools | disallowedTools | model | maxTurns | Bash Hook |
|-------|-------|-----------------|-------|----------|----------|
| `sdd-doc-writer` | Read, Write, Edit, Glob, Grep, Bash | Agent | opus | 40 | sdd-safe-bash.sh |
| `sdd-doc-reviewer` | Read, Glob, Grep, Bash | Write, Edit, Agent | sonnet | 20 | sdd-readonly-bash.sh |
| `sdd-issue-author` | Read, Glob, Grep, Bash | Write, Edit, Agent | sonnet | 20 | sdd-readonly-bash.sh |

### Workflow Preset Extensions

Append these rows to each preset table:

**`multi_agents`**

| Phase Action | Executor |
|---|---|
| `create_change_docs` | gemini:pro |
| `review_change_docs` | codex:balanced |
| `revise_change_docs` | gemini:pro |

**`multi_claude_agents`**

| Phase Action | Agent Definition | Model |
|---|---|---|
| `create_change_docs` | sdd-doc-writer | opus |
| `review_change_docs` | sdd-doc-reviewer | sonnet |
| `revise_change_docs` | sdd-doc-writer | opus |

**`claude_subagents`**

| Phase Action | Executor String | Subagent Type | Model |
|---|---|---|---|
| `create_change_docs` | `subagent:general-purpose:opus` | general-purpose | opus |
| `review_change_docs` | `subagent:general-purpose:sonnet` | general-purpose | sonnet |
| `revise_change_docs` | `subagent:general-purpose:opus` | general-purpose | opus |

**`mainthread`**: All 3 docs-phase actions route to mainthread.

### Issue Author Workflow Preset Extensions

Append to each preset table for issue-time reference context exploration:

**`multi_agents`**

| Phase Action | Executor |
|---|---|
| `create_issue_reference_context` | gemini:pro |

**`multi_claude_agents`**

| Phase Action | Agent Definition | Model |
|---|---|---|
| `create_issue_reference_context` | sdd-issue-author | sonnet |

**`claude_subagents`**

| Phase Action | Executor String | Subagent Type | Model |
|---|---|---|---|
| `create_issue_reference_context` | `subagent:general-purpose:sonnet` | general-purpose | sonnet |

**`mainthread`**: `create_issue_reference_context` routes to mainthread.

### TOML Example

```toml
[agentic_workflow.docs]
output_dir = "docs"

[[agentic_workflow.docs.targets]]
crate = "cclab-sdd"
guide = "docs/sdd-user-guide.md"
audience = "developer"
sections = ["getting-started", "workflow", "cli-reference", "config-reference"]

[[agentic_workflow.docs.targets]]
crate = "cclab-mamba"
guide = "docs/mamba-guide.md"
audience = "end-user"
sections = ["getting-started", "stdlib", "examples"]
```

### Config Loading Extension

```yaml
SddConfig::load(project_root):
  steps:
    1_primary_deserialize:
      action: "toml::from_str::<SddConfig>(content)"
      loads: [workflow, gemini, codex, claude, project, validation, docs]  # docs added
    2_overlay_extraction:
      extracts:
        - agentic_workflow.docs → config.docs  # new
```

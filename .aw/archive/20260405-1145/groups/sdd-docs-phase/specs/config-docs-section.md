---
id: config-docs-section
main_spec_ref: "crates/cclab-sdd/config/agents.md"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, config, changes]
filled_sections: [overview, requirements, scenarios, config, changes]
create_complete: true
---

# Config Docs Section

## Overview

Extend the SDD config schema (`agents.md`) with docs generation phase configuration. Adds `[sdd.docs]` section to `cclab/config.toml` and two new agent definitions (`sdd-doc-writer`, `sdd-doc-reviewer`) to the workflow preset tables.

**What changes in `agents.md`**:

| Addition | Section | Purpose |
|----------|---------|---------|
| `[sdd.docs]` config schema | New top-level section | Doc generation output directory + target array |
| `[[sdd.docs.targets]]` schema | Under `[sdd.docs]` | Per-crate guide path, audience, sections |
| `sdd-doc-writer` agent def | Agent Definitions table | Dedicated doc-writer agent with Write + Read tools |
| `sdd-doc-reviewer` agent def | Agent Definitions table | Doc-reviewer agent with Bash(read-only) + Read, no Write |
| Docs-phase actions | Workflow preset tables | `create_change_docs`, `review_change_docs`, `revise_change_docs` routing |

**Design decisions**:
- Presence of `[sdd.docs]` = enabled; no `enabled` flag (same pattern as `[sdd.repo_platform]`)
- `output_dir` defaults to `"docs"` relative to project root (per clarification Q3)
- `sdd-doc-writer` is a dedicated agent type, not reusing existing agents (per clarification Q1)
- `sdd-doc-reviewer` has Bash (read-only by prompt) to verify CLI output matches docs (per clarification Q2, C2)

**Scope**: Config schema documentation only — no code changes. Struct definitions and parsing logic are in `docs-phase-logic`. State machine is in `state-machine-docs-phase`.
## Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| R1 | Add `## [sdd.docs]` section to `agents.md` documenting `output_dir` field (string, default `"docs"`, relative to project root). Presence of section = enabled; no `enabled` flag. | high |
| R2 | Add `## [[sdd.docs.targets]]` subsection documenting target array fields: `crate` (string, required — matches crate name from change), `guide` (string, required — output file path relative to project root), `audience` (string, required — enum: `developer` \| `end-user` \| `admin`), `sections` (string array, required — guide section names to generate/update). | high |
| R3 | Add `sdd-doc-writer` agent definition to Agent Definitions table: tools = Read, Write, Edit, Glob, Grep, Bash; disallowedTools = Agent; model = opus; maxTurns = 40; Bash Hook = sdd-safe-bash.sh. | high |
| R4 | Add `sdd-doc-reviewer` agent definition to Agent Definitions table: tools = Read, Glob, Grep, Bash; disallowedTools = Write, Edit, Agent; model = sonnet; maxTurns = 20; Bash Hook = sdd-readonly-bash.sh. Doc-reviewer verifies accuracy by executing CLI commands (per clarification Q2, C2). | high |
| R5 | Extend all 4 workflow preset tables (`multi_agents`, `multi_claude_agents`, `claude_subagents`, `mainthread`) with docs-phase action rows: `create_change_docs`, `review_change_docs`, `revise_change_docs`. | high |
| R6 | Document `DocsConfig` and `DocsTarget` Rust struct field mapping (analogous to existing `RepoPlatformConfig` mapping table). Both structs loaded via `SddConfig::load()` under `sdd.docs` TOML namespace. | medium |
| R7 | Document validation: `load_validated()` does NOT require `[sdd.docs]` — section is optional. When absent, docs phase is skipped at `DocsCheck` state. | medium |
## Scenarios

### S1: Config spec documents [sdd.docs] section (R1)

- **GIVEN** `agents.md` is merged with this change spec
- **WHEN** a reader looks up `[sdd.docs]` in the config schema
- **THEN** the spec contains a dedicated `## [sdd.docs]` section with `output_dir` field documentation (type, default, resolution)
- **AND** the section states that presence = enabled (no `enabled` flag)

### S2: Config spec documents [[sdd.docs.targets]] array (R2)

- **GIVEN** `agents.md` includes the `[sdd.docs]` section
- **WHEN** a reader looks up target configuration
- **THEN** the spec contains `[[sdd.docs.targets]]` subsection with field table: `crate` (required), `guide` (required), `audience` (required, enum), `sections` (required, array)
- **AND** includes a TOML example showing multiple targets

### S3: Agent definitions include doc-writer (R3)

- **GIVEN** `agents.md` Agent Definitions table exists
- **WHEN** the spec is read after merge
- **THEN** `sdd-doc-writer` row is present with tools = Read, Write, Edit, Glob, Grep, Bash; disallowedTools = Agent; model = opus; maxTurns = 40; Bash Hook = sdd-safe-bash.sh

### S4: Agent definitions include doc-reviewer (R4)

- **GIVEN** `agents.md` Agent Definitions table exists
- **WHEN** the spec is read after merge
- **THEN** `sdd-doc-reviewer` row is present with tools = Read, Glob, Grep, Bash; disallowedTools = Write, Edit, Agent; model = sonnet; maxTurns = 20; Bash Hook = sdd-readonly-bash.sh
- **AND** doc-reviewer verifies accuracy by executing CLI commands

### S5: Workflow preset tables extended with docs-phase actions (R5)

- **GIVEN** `agents.md` has 4 workflow preset tables
- **WHEN** the spec is read after merge
- **THEN** each preset table includes 3 new rows: `create_change_docs`, `review_change_docs`, `revise_change_docs`
- **AND** `multi_claude_agents` maps to `sdd-doc-writer` (opus) for create/revise, `sdd-doc-reviewer` (sonnet) for review
- **AND** `mainthread` routes all 3 to mainthread

### S6: [sdd.docs] is optional — not validated by load_validated() (R7)

- **GIVEN** `cclab/config.toml` has no `[sdd.docs]` section
- **WHEN** `SddConfig::load_validated()` runs
- **THEN** validation passes (unlike `repo_platform` / `spec_platform` which are required)
- **AND** `docs` field is `None` — docs phase skips at `DocsCheck` state
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
  - path: cclab/specs/crates/cclab-sdd/config/agents.md
    action: modify
    description: |
      Append new sections to the SDD config schema spec after the existing
      "Config Loading" section:

      ## [sdd.docs]

      Field table:
        output_dir (string, default "docs" — relative to project root)
        targets (array of DocsTarget, required, minItems 1)

      Presence of [sdd.docs] section = enabled. No enabled flag.
      Not required by load_validated() — when absent, docs phase
      skips at DocsCheck state.

      ## [[sdd.docs.targets]]

      Field table:
        crate (string, required — matches against change-affected crates)
        guide (string, required — output guide file path relative to project root)
        audience (string, required — enum: developer | end-user | admin)
        sections (string array, required, minItems 1 — guide section names)

      TOML example showing multiple targets.

      ## SddConfig Field Mapping (append row)

      | [sdd.docs] | docs | Option<DocsConfig> | #[serde(default)] | None | NOT required |

      ## Agent Definitions (append 2 rows)

      | sdd-doc-writer | Read,Write,Edit,Glob,Grep,Bash | Agent | opus | 40 | sdd-safe-bash.sh |
      | sdd-doc-reviewer | Read,Glob,Grep,Bash | Write,Edit,Agent | sonnet | 20 | sdd-readonly-bash.sh |

      ## Workflow Preset Tables (append 3 rows each)

      multi_agents:        create_change_docs → gemini:pro, review → codex:balanced, revise → gemini:pro
      multi_claude_agents: create_change_docs → sdd-doc-writer/opus, review → sdd-doc-reviewer/sonnet, revise → sdd-doc-writer/opus
      claude_subagents:    create_change_docs → subagent:general-purpose:opus, review → subagent:general-purpose:sonnet, revise → subagent:general-purpose:opus
      mainthread:          all 3 → mainthread

      ## Config Loading (update step 1)

      Add 'docs' to the loads list in step 1_primary_deserialize.
      Add sdd.docs → config.docs to step 2_overlay_extraction.
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


## Config

New fields added to `SddConfig` (loaded from `cclab/config.toml`).

### `[sdd.docs]`

```json
{
  "$id": "sdd-config-docs",
  "title": "DocsConfig",
  "description": "Docs generation phase config — [sdd.docs] in cclab/config.toml. Presence = enabled.",
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
      "description": "Single doc generation target — [[sdd.docs.targets]] in config.toml",
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
| `[sdd.docs]` | `docs` | `Option<DocsConfig>` | `#[serde(default, skip_serializing_if = "Option::is_none")]` | `None` | NOT required by `load_validated()` |

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

### TOML Example

```toml
[sdd.docs]
output_dir = "docs"

[[sdd.docs.targets]]
crate = "cclab-sdd"
guide = "docs/sdd-user-guide.md"
audience = "developer"
sections = ["getting-started", "workflow", "cli-reference", "config-reference"]

[[sdd.docs.targets]]
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
        - sdd.docs → config.docs  # new
```

# Reviews

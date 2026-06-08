---
refs: [create-reference-context, review-reference-context, revise-reference-context, create-change-spec, review-change-spec, revise-change-spec, create-change-implementation, review-change-implementation, revise-change-implementation, create-change-merge]
id: sdd-config-repo-platform
main_spec_ref: "crates/cclab-sdd/config/agents.md"
merge_strategy: extend
fill_sections: [overview, requirements, scenarios, config, changes]
filled_sections: [overview, requirements, scenarios, config, changes]
create_complete: true
---

# cclab/config.toml — SDD Configuration Schema

Central orchestration config for the SDD workflow. Controls project modules, agent routing, provider settings, and spec validation rules.

**Location**: `cclab/config.toml` (project root)
**Loaded by**: `SddConfig::load()` / `SddConfig::load_validated()` (cached with mtime invalidation)
**Fallback**: If the file is missing, `SddConfig::default()` is used.
**Rust struct**: `SddConfig` in `crates/cclab-sdd/src/models/change.rs`
**Template**: `crates/cclab-sdd/templates/config.toml`

## `[[project.modules]]`

Declares project modules for monorepo-aware language detection. Used by task generation and Lens analysis.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | `String` | yes | Relative path to module root (e.g., `"."`, `"api/"`, `"crates/cclab-sdd/"`) |
| `language` | `String` | yes | One of: `rust`, `python`, `typescript`, `javascript`, `go` |
| `framework` | `String` | no | Optional framework hint (e.g., `"axum"`, `"react"`, `"django"`) |

**Resolution**: `language_for_path(file)` finds the longest matching `path` prefix. `primary_language()` returns the first module's language.

```yaml
project.modules:
  - path: "."
    language: rust
  - path: "frontend/"
    language: typescript
    framework: react
```

## Top-level `envfile`

Global environment file loaded for all agents. Relative paths resolve against project root.

```yaml
# Global envfile - loaded for all agents
envfile: ".env"
```

Provider-specific envfiles override global values on key collision.

## `[workflow]`

### `workflow.mode`

Single field selecting one of 4 fixed execution-mode presets. No per-action overrides.

```toml
[workflow]
# "multi_agents" | "multi_claude_agents" | "claude_subagents" | "mainthread"
mode = "mainthread"
```

| Mode | Dispatch | Characteristics |
|------|----------|-----------------|
| `multi_agents` | External CLI subprocess (gemini/codex/claude) | Model diversity, cost optimization |
| `multi_claude_agents` | `claude --agent sdd-X` subprocess | Claude Code tools + isolation + least-privilege |
| `claude_subagents` | Claude Code Agent tool (mainthread invokes) | Low overhead, inherits session |
| `mainthread` | Current LLM context (no delegation) | Zero overhead, shared context |

Default: `"mainthread"`. Fallback: agent failure → retry once → mainthread.

### Preset: `multi_agents`

| Phase Action | Executor |
|---|---|
| `restructure_input` | mainthread |
| `create_pre_clarifications` | mainthread |
| `create_reference_context` | gemini:flash |
| `review_reference_context` | codex:balanced |
| `revise_reference_context` | mainthread |
| `create_post_clarifications` | mainthread |
| `create_change_spec` | gemini:pro |
| `review_change_spec` | codex:max |
| `revise_change_spec` | gemini:pro |
| `implement` | mainthread |
| `review_implementation` | codex:balanced |
| `revise_implementation` | mainthread |
| `create_change_merge` | mainthread |

### Preset: `multi_claude_agents`

| Phase Action | Agent Definition | Model |
|---|---|---|
| `restructure_input` | mainthread | — |
| `create_pre_clarifications` | mainthread | — |
| `create_reference_context` | sdd-reference-context | sonnet |
| `review_reference_context` | sdd-review | haiku |
| `revise_reference_context` | mainthread | — |
| `create_post_clarifications` | mainthread | — |
| `create_change_spec` | sdd-change-spec | opus |
| `review_change_spec` | sdd-review | sonnet |
| `revise_change_spec` | sdd-change-spec | opus |
| `implement` | sdd-change-implementation | opus |
| `review_implementation` | sdd-review | sonnet |
| `revise_implementation` | sdd-change-implementation | opus |
| `create_change_merge` | mainthread | — |

Dispatch: `claude --agent sdd-{name} --model {model} --cwd {temp_dir}` where `temp_dir` contains phase-specific CLAUDE.md.

### Preset: `claude_subagents`

| Phase Action | Executor String | Subagent Type | Model |
|---|---|---|---|
| `restructure_input` | `mainthread` | — | — |
| `create_pre_clarifications` | `mainthread` | — | — |
| `create_reference_context` | `subagent:Explore:sonnet` | Explore | sonnet |
| `review_reference_context` | `subagent:general-purpose:haiku` | general-purpose | haiku |
| `revise_reference_context` | `mainthread` | — | — |
| `create_post_clarifications` | `mainthread` | — | — |
| `create_change_spec` | `subagent:general-purpose:opus` | general-purpose | opus |
| `review_change_spec` | `subagent:general-purpose:sonnet` | general-purpose | sonnet |
| `revise_change_spec` | `subagent:general-purpose:opus` | general-purpose | opus |
| `create_change_implementation` | `subagent:general-purpose:opus` | general-purpose | opus |
| `review_change_implementation` | `subagent:general-purpose:sonnet` | general-purpose | sonnet |
| `revise_change_implementation` | `subagent:general-purpose:opus` | general-purpose | opus |
| `create_change_merge` | `mainthread` | — | — |

Dispatch: Rust returns `{executor: ["subagent:Explore:sonnet"], prompt_path: "..."}`. Mainthread skill parses `subagent:{type}:{model}` and invokes Claude Code Agent tool with `subagent_type` and `model` params.

### Preset: `mainthread`

All phase actions route to mainthread. No delegation.

### Merge

Merge is always programmatic (mainthread) regardless of mode. `create_change_merge` is mainthread in all presets.

## Provider Sections

Each provider section configures CLI command, available models, default model, and optional envfile.

### Common fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `command` | `String` | no | CLI binary name (default: provider name) |
| `default` | `String` | no | Default model ID when none specified |
| `envfile` | `String` | no | Provider-specific env file (overrides global) |

### `[[<provider>.models]]`

Array of model definitions. Each model has:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `String` | yes | Short identifier used in agent specs (e.g., `"flash"`, `"balanced"`) |
| `model` | `String` | yes | Full model name passed to CLI (e.g., `"gemini-3-flash-preview"`) |
| `complexity` | `String` | yes | Max complexity: `low`, `medium`, `high`, `critical` |
| `reasoning` | `String` | no | Reasoning level for Codex only: `"low"`, `"medium"`, `"high"`, `"extra high"` |
| `cost_per_1m_input` | `f64` | no | Cost per 1M input tokens (USD) |
| `cost_per_1m_output` | `f64` | no | Cost per 1M output tokens (USD) |

**Model selection**: `select_model(complexity)` picks the cheapest model that handles the given complexity. `select_model_by_id(id)` does exact lookup with default fallback.

### `[gemini]`

| Model ID | Full Model | Complexity | Input $/1M | Output $/1M |
|----------|------------|------------|------------|-------------|
| `flash` | `gemini-3-flash-preview` | medium | 0.10 | 0.40 |
| `pro` | `gemini-3.1-pro-preview` | critical | 1.25 | 10.00 |

Default: `flash`

### `[codex]`

| Model ID | Full Model | Reasoning | Complexity | Input $/1M | Output $/1M |
|----------|------------|-----------|------------|------------|-------------|
| `fast` | `gpt-5.3-codex` | low | low | 2.00 | 8.00 |
| `balanced` | `gpt-5.3-codex` | medium | medium | 2.00 | 8.00 |
| `deep` | `gpt-5.3-codex` | high | high | 2.00 | 8.00 |
| `max` | `gpt-5.3-codex` | extra high | critical | 2.00 | 8.00 |

Default: `balanced`

### `[codex-spark]`

Same schema as `[codex]` but uses `gpt-5.3-codex-spark` model. Shares the `codex` CLI command.

Default: `balanced`

### `[claude]`

| Model ID | Full Model | Complexity | Input $/1M | Output $/1M |
|----------|------------|------------|------------|-------------|
| `fast` | `haiku` | low | 0.80 | 4.00 |
| `balanced` | `sonnet` | medium | 3.00 | 15.00 |
| `deep` | `opus` | critical | 15.00 | 75.00 |

Default: `balanced`

## `[validation]`

Rules for validating spec documents. Applied during `sdd_validate_spec_completeness`.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `required_headings` | `String[]` | `["Overview", "Acceptance Criteria"]` | Required top-level headings |
| `requirement_pattern` | `String` | `""` | Regex for requirement naming (e.g., `"^### R\\d+:"`) |
| `scenario_pattern` | `String` | `'(?m)^###\s*Scenario:\|^-\s*WHEN[^\n]*THEN'` | Regex for scenario format |
| `scenario_min_count` | `usize` | `1` | Minimum scenarios per requirement |
| `require_when_then` | `bool` | `true` | Require WHEN/THEN clauses |
| `when_pattern` | `String` | `'\*\*WHEN\*\*\|WHEN'` | Regex for WHEN clause |
| `then_pattern` | `String` | `'\*\*THEN\*\*\|THEN'` | Regex for THEN clause |

### `[validation.severity_map]`

Maps validation error types to severity levels.

| Key | Default | Description |
|-----|---------|-------------|
| `missing_heading` | `"High"` | Required heading absent |
| `invalid_requirement_format` | `"High"` | Requirement doesn't match pattern |
| `missing_scenario` | `"High"` | Below minimum scenario count |
| `missing_when_then` | `"High"` | Missing WHEN/THEN clauses |
| `duplicate_requirement` | `"High"` | Duplicate requirement ID |
| `broken_reference` | `"Medium"` | Reference to non-existent requirement |

Severity values: `"Low"`, `"Medium"`, `"High"`.

## Execution Mode

`ExecutionMode` enum in Rust:

```rust
enum ExecutionMode {
    MultiAgents,         // external CLI (gemini/codex/claude)
    MultiClaudeAgents,   // claude --agent sdd-X
    ClaudeSubagents,     // Claude Code Agent tool
    Mainthread,          // no delegation (default)
}
```

Resolved from `workflow.mode` in config. Each mode has a fixed preset table — no per-action overrides.

### Agent Definitions (`.claude/agents/`)

Used by `multi_claude_agents` mode only:

| Agent | tools | disallowedTools | model | maxTurns | Bash Hook |
|-------|-------|-----------------|-------|----------|-----------|
| `sdd-reference-context` | Read, Glob, Grep, Bash | Write, Edit, Agent | sonnet | 20 | sdd-readonly-bash.sh |
| `sdd-change-spec` | Read, Write, Edit, Glob, Grep | Bash, Agent | opus | 30 | — |
| `sdd-review` | Read, Glob, Grep, Bash | Write, Edit, Agent | sonnet | 15 | sdd-readonly-bash.sh |
| `sdd-change-implementation` | Read, Write, Edit, Glob, Grep, Bash | Agent | opus | 50 | sdd-safe-bash.sh |

Bash hooks are per-agent (in frontmatter) — do not affect mainthread or other agents.

## Envfile Resolution

1. **Global envfile** (`envfile = ".env"` at top level or under `[workflow]`) — loaded for all providers.
2. **Provider envfile** (`envfile` under `[gemini]`, `[codex]`, `[claude]`) — overrides global on key collision.
3. **Path resolution**: relative paths resolve against project root; absolute paths used as-is.


## Overview

## Overview
<!-- type: overview lang: markdown -->

Extend the SDD config schema (`cclab/config.toml`) with three new platform sections under `[sdd.*]`, adding them to `SddConfig` and the main config spec.

| Section | Purpose | Status |
|---------|---------|--------|
| `[sdd.issue_platform]` | Issue tracking (GitHub/GitLab/Jira) | Existing — unchanged |
| `[sdd.repo_platform]` | Git repo operations, auto-commit, auto-PR | New — this change |
| `[sdd.spec_platform]` | Spec storage location (local/remote) | New — this change |
| `[sdd.docs_platform]` | User-facing documentation platform | Future — commented template only |

The `agents.md` main spec documents the full `SddConfig` schema. This change extends it with:
- `repo_platform: Option<RepoPlatformConfig>` — git/PR operations config consumed by merge workflow
- `spec_platform: Option<SpecPlatformConfig>` — spec storage backend config
- Both fields are `#[serde(default)]` → `None` when section absent (backward compatible)

Cross-references: `platform-config-repo-spec` defines the structs and CLI. `change-merge-git-integration` defines how `repo_platform` is consumed by merge.


## Requirements

## Requirements
<!-- type: requirements lang: markdown -->

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | Document repo_platform in config schema spec | P0 | `agents.md` includes `## [sdd.repo_platform]` section documenting all fields: `type` (required string), `repo` (required string, no fallback), `default_branch` (string, default "main"), `auto_commit` (bool, default false), `auto_pr` (bool, default false). |
| R2 | Document spec_platform in config schema spec | P1 | `agents.md` includes `## [sdd.spec_platform]` section documenting fields: `type` (required string, currently only "local"), `path` (string, default "cclab/specs"). |
| R3 | Document docs_platform placeholder | P2 | `agents.md` mentions `[sdd.docs_platform]` as reserved/future section with no runtime behavior. |
| R4 | SddConfig struct fields documented | P0 | Config schema spec documents `repo_platform: Option<RepoPlatformConfig>` and `spec_platform: Option<SpecPlatformConfig>` fields on `SddConfig`, both `#[serde(default)]`. |
| R5 | Platform section cross-references | P1 | Config schema spec cross-references `platform-config-repo-spec` for struct definitions and `change-merge-git-integration` for merge consumption. |

### Constraints

- This spec modifies documentation only — no code changes (code is in `platform-config-repo-spec`)
- Existing sections of `agents.md` (workflow, providers, validation, etc.) unchanged
- `repo_platform.repo` is required with no fallback — per clarification Q3


## Scenarios

## Scenarios
<!-- type: scenarios lang: markdown -->

### S1: Config schema spec includes repo_platform section (R1, R4)

**GIVEN** `agents.md` is merged with this change spec
**WHEN** a reader looks up `[sdd.repo_platform]` in the config schema
**THEN** the spec contains a dedicated section with field table (`type`, `repo`, `default_branch`, `auto_commit`, `auto_pr`), types, defaults, and required markers.

### S2: Config schema spec includes spec_platform section (R2, R4)

**GIVEN** `agents.md` is merged with this change spec
**WHEN** a reader looks up `[sdd.spec_platform]` in the config schema
**THEN** the spec contains a dedicated section with field table (`type`, `path`), types, defaults, and the constraint that only `"local"` is currently supported.

### S3: docs_platform documented as future placeholder (R3)

**GIVEN** `agents.md` is merged with this change spec
**WHEN** a reader looks up `[sdd.docs_platform]`
**THEN** the spec notes it as reserved/future with no runtime code — template-only.

### S4: SddConfig struct documentation updated (R4)

**GIVEN** `agents.md` documents `SddConfig` struct location (`crates/cclab-sdd/src/models/change.rs`)
**WHEN** the spec is read after merge
**THEN** the frontmatter/metadata area reflects that `SddConfig` now includes `repo_platform` and `spec_platform` optional fields.

### S5: Cross-references are present (R5)

**GIVEN** `agents.md` is merged with this change spec
**WHEN** a reader needs full struct definitions or merge workflow details
**THEN** the spec references `platform-config-repo-spec` for RepoPlatformConfig/SpecPlatformConfig struct details, and `change-merge-git-integration` for auto-commit/auto-PR logic.


## Config

## Config: Platform Sections in SddConfig
<!-- type: config lang: json -->

New fields added to `SddConfig` (loaded from `cclab/config.toml`).

### `[sdd.repo_platform]`

```json
{
  "$id": "sdd-config-repo-platform",
  "title": "RepoPlatformConfig",
  "description": "Git repository and PR operations — [sdd.repo_platform] in cclab/config.toml",
  "type": "object",
  "properties": {
    "type": {
      "type": "string",
      "enum": ["github", "gitlab"],
      "description": "VCS platform type"
    },
    "repo": {
      "type": "string",
      "pattern": "^[\\w.-]+/[\\w.-]+$",
      "description": "Repository in owner/repo format. Required — no fallback to issue_platform.repo."
    },
    "default_branch": {
      "type": "string",
      "default": "main",
      "description": "Target branch for auto-PR creation"
    },
    "auto_commit": {
      "type": "boolean",
      "default": false,
      "description": "Auto git-commit cclab/ changes after merge archive"
    },
    "auto_pr": {
      "type": "boolean",
      "default": false,
      "description": "Auto-create PR after auto-commit. Requires auto_commit=true."
    }
  },
  "required": ["type", "repo"],
  "additionalProperties": false
}
```

### `[sdd.spec_platform]`

```json
{
  "$id": "sdd-config-spec-platform",
  "title": "SpecPlatformConfig",
  "description": "Spec storage backend — [sdd.spec_platform] in cclab/config.toml",
  "type": "object",
  "properties": {
    "type": {
      "type": "string",
      "enum": ["local"],
      "description": "Storage backend. Currently only 'local' supported."
    },
    "path": {
      "type": "string",
      "default": "cclab/specs",
      "description": "Relative path to spec storage directory from project root"
    }
  },
  "required": ["type"],
  "additionalProperties": false
}
```

### `[sdd.docs_platform]` (future)

Reserved section — not parsed at runtime. Documented in config template as commented-out block:

```toml
# [sdd.docs_platform]
# type = "github_pages"
```

### SddConfig Field Mapping

| Config Section | Rust Field | Type | Serde | Default |
|----------------|-----------|------|-------|---------|
| `[sdd.repo_platform]` | `repo_platform` | `Option<RepoPlatformConfig>` | `#[serde(default)]` | `None` |
| `[sdd.spec_platform]` | `spec_platform` | `Option<SpecPlatformConfig>` | `#[serde(default)]` | `None` |
| `[sdd.docs_platform]` | — | — | — | Not parsed |

### TOML Example

```toml
[sdd.repo_platform]
type = "github"
repo = "chrischeng-c4/cclab"
default_branch = "main"
auto_commit = true
auto_pr = false

[sdd.spec_platform]
type = "local"
path = "cclab/specs"

# [sdd.docs_platform]
# type = "github_pages"
```


## Changes

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: cclab/specs/crates/cclab-sdd/config/agents.md
    action: MODIFY
    desc: |
      Add platform config sections to the SDD config schema spec.

      After the "Envfile Resolution" section (end of current file), append:

      ## Platform Sections

      ### `[sdd.repo_platform]`

      Field table: type (string, required), repo (string, required — no fallback),
      default_branch (string, default "main"), auto_commit (bool, default false),
      auto_pr (bool, default false).

      Note: auto_pr requires auto_commit=true. repo is explicit, no fallback
      to issue_platform.repo.

      Loaded as: SddConfig.repo_platform: Option<RepoPlatformConfig>
      Serde: #[serde(default)] → None when section absent.

      ### `[sdd.spec_platform]`

      Field table: type (string, required — currently only "local"),
      path (string, default "cclab/specs").

      Loaded as: SddConfig.spec_platform: Option<SpecPlatformConfig>
      Serde: #[serde(default)] → None when section absent.

      ### `[sdd.docs_platform]` (future)

      Reserved section, not parsed at runtime. Template-only.

      TOML example showing all three sections.

      Cross-references to platform-config-repo-spec and
      change-merge-git-integration.
```

# Reviews

---
refs: [create-reference-context, review-reference-context, revise-reference-context, create-change-spec, review-change-spec, revise-change-spec, create-change-implementation, review-change-implementation, revise-change-implementation, create-change-merge]
id: agent-timeout
main_spec_ref: crates/cclab-sdd/config/agents.md
merge_strategy: extend
fill_sections: [overview, changes]
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


# Reviews

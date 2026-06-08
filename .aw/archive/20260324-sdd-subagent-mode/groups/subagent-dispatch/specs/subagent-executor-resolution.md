---
id: subagent-executor-resolution
main_spec_ref: "crates/cclab-sdd/logic/executor-resolution.md"
merge_strategy: extend
fill_sections: [overview, config, changes]
filled_sections: [overview, config, changes]
create_complete: true
---

# Sdd Execution Modes

## Overview

Add `subagent:{type}:{model}` executor string format for `claude_subagents` mode. Unlike `multi_agents` and `multi_claude_agents` where `build_workflow_response()` calls `run_agent()` (Rust-side CLI dispatch), `subagent:*` executors are returned directly to the mainthread skill for Agent tool invocation.

### Dispatch Branches in `build_workflow_response()`

| Executor Pattern | Branch | Dispatch By |
|---|---|---|
| `["mainthread"]` | Return `prompt_path` + `executor` | Mainthread reads prompt, executes inline |
| `subagent:*` | Return `prompt_path` + `executor` | Mainthread parses `subagent:{type}:{model}`, invokes Agent tool |
| Other (`gemini:*`, `codex:*`, `claude-agent:*`) | Call `run_agent()` internally | Rust CLI spawns external subprocess |

### Executor String Format

```
subagent:{subagent_type}:{model}
```

| Field | Values | Description |
|---|---|---|
| `subagent_type` | `Explore`, `general-purpose` | Maps to Claude Code Agent tool `subagent_type` param |
| `model` | `sonnet`, `opus`, `haiku` | Maps to Agent tool `model` param |

Examples: `subagent:Explore:sonnet`, `subagent:general-purpose:opus`

### Key Constraint

`subagent:*` executors bypass `run_agent()` entirely. Verification (STATE.yaml phase + artifact check) is performed by the mainthread skill after Agent tool returns, not by `sdd_delegate_agent`.
# "multi_agents" | "multi_claude_agents" | "claude_subagents" | "mainthread"
mode = "multi_claude_agents"
```

Executor resolution reads `workflow.mode` once, loads the corresponding preset table, and dispatches all phase actions from it. Agent failure triggers: retry once → fallback to mainthread.
## Requirements

<!-- type: requirements lang: markdown -->

### R1: Single Mode Config

Replace `[workflow.agents]` per-action arrays with `workflow.mode` in `cclab/config.toml`.

- **WHEN** config is loaded, **THEN** `workflow.mode` resolves to one of `"multi_agents"`, `"multi_claude_agents"`, `"claude_subagents"`, `"mainthread"`
- **WHEN** `workflow.mode` is absent, **THEN** default is `"mainthread"`
- **WHEN** `[workflow.agents]` per-action keys are present, **THEN** they are ignored (backward compat: log warning)

### R2: Fixed Preset Tables

Each mode embeds a complete phase-to-executor mapping — no per-action overrides.

- **WHEN** mode is `multi_agents`, **THEN** executor resolution uses the `multi_agents` preset (gemini/codex/mainthread by phase)
- **WHEN** mode is `multi_claude_agents`, **THEN** executor resolution uses the `multi_claude_agents` preset (`claude --agent sdd-X` by phase)
- **WHEN** mode is `claude_subagents`, **THEN** executor resolution uses the `claude_subagents` preset (Agent tool by phase)
- **WHEN** mode is `mainthread`, **THEN** all phase actions route to mainthread

### R3: Fallback Chain

- **WHEN** an external agent fails (non-zero exit or timeout), **THEN** retry once with the same executor
- **WHEN** retry also fails, **THEN** fall back to mainthread execution
- **WHEN** verification fails but phase has not changed, **THEN** treat as agent failure (fallback)
- **WHEN** verification fails but phase has advanced, **THEN** resume from `actual_phase` via `sdd_run_change`

### R4: `multi_claude_agents` Agent Definitions

New agent definition files in `.claude/agents/`:

| Agent | Tools | Disallowed | Model | maxTurns | Bash Hook |
|-------|-------|------------|-------|----------|-----------|
| `sdd-reference-context` | Read, Glob, Grep, Bash | Write, Edit, Agent | sonnet | 20 | readonly |
| `sdd-change-spec` | Read, Write, Edit, Glob, Grep | Bash, Agent | opus | 30 | — |
| `sdd-review` | Read, Glob, Grep, Bash | Write, Edit, Agent | sonnet | 15 | readonly+test |
| `sdd-change-implementation` | Read, Write, Edit, Glob, Grep, Bash | Agent | opus | 50 | safe |

### R5: Bash Hook Scripts

- **WHEN** `sdd-reference-context` or `sdd-review` runs Bash, **THEN** only `git log/diff/status/show`, `ls`, `cat`, `find`, `cargo test/check` are allowed
- **WHEN** `sdd-change-implementation` runs Bash, **THEN** `rm -rf`, `git push/reset`, `chmod 777` are blocked

### R6: Phase-Specific CLAUDE.md

- **WHEN** `run_agent()` spawns `claude --agent` for `multi_claude_agents` mode, **THEN** it generates a phase-specific CLAUDE.md before spawning
- **WHEN** phase is `reference_context`, **THEN** CLAUDE.md focuses on specs structure and explore strategy
- **WHEN** phase is `change_spec`, **THEN** CLAUDE.md focuses on spec format rules and JSON Schema/Mermaid conventions
- **WHEN** phase is `change_implementation`, **THEN** CLAUDE.md focuses on code style, test requirements, file size limits
- **WHEN** phase is `review`, **THEN** CLAUDE.md focuses on review checklist and severity criteria
## Scenarios

<!-- type: scenarios lang: markdown -->

### Scenario: multi_agents — spec phase routes to gemini

- **WHEN** `workflow.mode = "multi_agents"` and action is `create_change_spec`
- **THEN** executor resolves to `gemini:pro`
- **AND** `run_agent()` spawns `gemini` CLI subprocess with the spec prompt

### Scenario: multi_agents — review phase routes to codex

- **WHEN** `workflow.mode = "multi_agents"` and action is `review_change_spec`
- **THEN** executor resolves to `codex:max`
- **AND** `run_agent()` spawns `codex` CLI subprocess

### Scenario: multi_claude_agents — reference context routes to sdd-reference-context agent

- **WHEN** `workflow.mode = "multi_claude_agents"` and action is `create_reference_context`
- **THEN** executor resolves to `claude --agent sdd-reference-context` (sonnet)
- **AND** `run_agent()` generates a reference-context CLAUDE.md and spawns the subprocess
- **AND** only Read/Glob/Grep/Bash tools are available; Write/Edit/Agent are disallowed

### Scenario: multi_claude_agents — spec writing routes to sdd-change-spec agent

- **WHEN** `workflow.mode = "multi_claude_agents"` and action is `create_change_spec`
- **THEN** executor resolves to `claude --agent sdd-change-spec` (opus)
- **AND** Bash is disallowed; agent writes specs via Write/Edit tools only

### Scenario: claude_subagents — explore routes to Explore agent

- **WHEN** `workflow.mode = "claude_subagents"` and action is `create_reference_context`
- **THEN** mainthread invokes Agent tool with `subagent_type: "Explore"` and `model: "sonnet"`
- **AND** no subprocess is spawned; agent inherits mainthread session tools

### Scenario: mainthread — all phases execute inline

- **WHEN** `workflow.mode = "mainthread"`
- **THEN** every phase action executes in the current LLM context
- **AND** no external processes are spawned

### Scenario: agent failure with successful retry

- **WHEN** external agent exits with non-zero code on first attempt
- **THEN** `run_agent()` retries once with the same executor
- **AND** if retry succeeds, execution continues normally

### Scenario: agent failure with fallback to mainthread

- **WHEN** external agent fails on both initial attempt and retry
- **THEN** mainthread executes the phase prompt directly
- **AND** telemetry records the fallback event

### Scenario: verification failure with partial phase advance

- **WHEN** agent exits cleanly but expected artifact is missing and phase has advanced partially
- **THEN** `run_agent()` returns `status: ok, passed: false` with `actual_phase`
- **AND** caller resumes via `sdd_run_change` from `actual_phase`

### Scenario: config.toml missing mode field

- **WHEN** `cclab/config.toml` has no `workflow.mode` key
- **THEN** executor resolution defaults to `mainthread` for all phase actions
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

### Unit: Preset Table Lookup

- For each of the 4 modes, verify every phase action maps to the correct executor string
- Verify that `workflow.mode` missing from config defaults to `mainthread`
- Verify no per-action override is possible (preset is immutable)

### Unit: `multi_claude_agents` Agent Definitions

- Parse each `.claude/agents/sdd-*.md` file and verify frontmatter fields (tools, disallowedTools, model, maxTurns)
- Verify `sdd-reference-context` has Write/Edit/Agent in disallowedTools
- Verify `sdd-change-spec` has Bash/Agent in disallowedTools
- Verify `sdd-change-implementation` has Agent in disallowedTools but NOT Bash

### Unit: Bash Hook Scripts

- `sdd-readonly-bash.sh`: verify `cargo build` is blocked, `cargo test` is allowed
- `sdd-readonly-bash.sh`: verify `git push` is blocked, `git log` is allowed
- `sdd-safe-bash.sh`: verify `rm -rf /` is blocked, `cargo build` is allowed
- `sdd-safe-bash.sh`: verify `git reset --hard` is blocked

### Integration: Executor Resolution → run_agent() Dispatch

- `multi_agents` mode: `create_change_spec` → gemini subprocess spawned with correct args
- `multi_claude_agents` mode: `create_change_spec` → `claude --agent sdd-change-spec` spawned with phase CLAUDE.md
- `claude_subagents` mode: `create_reference_context` → Agent tool invoked with `subagent_type: "Explore"`
- `mainthread` mode: all actions execute inline (no subprocess)

### Integration: Fallback Chain

- Agent exits non-zero → retry once → if retry succeeds, no fallback
- Agent exits non-zero on both attempts → mainthread fallback executes
- Verification fails + phase unchanged → treated as failure → mainthread fallback
- Verification fails + phase advanced → `actual_phase` returned, no fallback

### Integration: Phase-Specific CLAUDE.md Generation

- Before `claude --agent sdd-change-spec` spawns, verify CLAUDE.md contains spec format rules
- Before `claude --agent sdd-reference-context` spawns, verify CLAUDE.md contains specs explore strategy
- Before `claude --agent sdd-review` spawns, verify CLAUDE.md contains review checklist

### Config: TOML Parsing

- Valid mode strings parse correctly
- Invalid mode string returns config validation error
- Missing `workflow.mode` falls back to `mainthread` without error
## Changes

```yaml
files:
  # Executor resolution — subagent:* routing in build_workflow_response()
  - path: crates/cclab-sdd/src/tools/workflow_common.rs
    action: MODIFY
    desc: |
      Add third dispatch branch in build_workflow_response(): when executor starts with
      "subagent:", return prompt_path + executor to caller (like mainthread) instead of
      calling run_agent(). The is_subagent check routes subagent:* executors back to
      the mainthread skill for Agent tool invocation.

  # Preset table — subagent:X:model executor strings
  - path: crates/cclab-sdd/src/models/change.rs
    action: MODIFY
    desc: |
      Add preset_claude_subagents() method to AgentsConfig returning subagent:X:model
      executor strings per phase action. Add ClaudeSubagents variant to ExecutionMode
      enum. Wire for_mode(ClaudeSubagents) to preset_claude_subagents().

  # Main spec update
  - path: cclab/specs/crates/cclab-sdd/logic/executor-resolution.md
    action: MODIFY
    desc: |
      Add subagent:X:model executor string format, claude_subagents preset table with
      explicit executor strings, and build_workflow_response() three-branch dispatch
      documentation. Add config JSON Schema for SubagentExecutorString.

  # Agents config spec — claude_subagents preset
  - path: cclab/specs/crates/cclab-sdd/config/agents.md
    action: MODIFY
    desc: |
      Add claude_subagents preset rows showing executor string format subagent:X:model.
      Document dispatch pattern: Rust returns executor string, mainthread parses and
      invokes Agent tool.
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


## Logic

<!-- type: logic lang: mermaid -->

```mermaid
flowchart TD
    Start([action]) --> ReadMode[read workflow.mode]
    ReadMode --> LoadPreset[load preset table for mode]
    LoadPreset --> Lookup[executor = preset_table[action]]
    Lookup --> IsMT{executor == mainthread?}
    IsMT -->|yes| MT([mainthread executes])
    IsMT -->|no| DispatchMode{mode?}
    DispatchMode -->|multi_agents| ExtCLI[run_agent: external CLI subprocess]
    DispatchMode -->|multi_claude_agents| ClaudeAgent[run_agent: claude --agent sdd-X]
    DispatchMode -->|claude_subagents| SubAgent[invoke Agent tool with type + model]
    ExtCLI --> CheckExit{exit_code == 0?}
    ClaudeAgent --> CheckExit
    SubAgent --> CheckExit
    CheckExit -->|yes| Verify[verify phase + artifact]
    Verify --> VPassed{passed?}
    VPassed -->|yes| Done([done])
    VPassed -->|no, phase unchanged| Fallback
    VPassed -->|no, phase advanced| Resume([resume from actual_phase])
    CheckExit -->|no| RetryCheck{retried?}
    RetryCheck -->|no| Retry[retry once]
    Retry --> CheckExit
    RetryCheck -->|yes| Fallback[fallback: mainthread executes]
    Fallback --> MT
```


## Config

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "SubagentExecutorString",
  "description": "Executor string format for claude_subagents mode. Returned by preset table, parsed by mainthread skill.",
  "type": "string",
  "pattern": "^subagent:(Explore|general-purpose|sdd-reference-context|sdd-change-spec|sdd-review|sdd-change-implementation):(sonnet|opus|haiku)$",
  "examples": ["subagent:Explore:sonnet", "subagent:general-purpose:opus", "subagent:general-purpose:haiku"]
}
```

**Preset: `claude_subagents`** — executor strings returned by `AgentsConfig::preset_claude_subagents()`

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

**Workflow Tool Response Schema (subagent branch)**

```json
{
  "type": "object",
  "properties": {
    "status": { "const": "ok" },
    "prompt_path": { "type": "string", "description": "Relative path to prompt file for mainthread to read" },
    "executor": {
      "type": "array",
      "items": { "type": "string", "pattern": "^subagent:.+:.+$" },
      "description": "Single-element array with subagent:type:model string"
    },
    "next_actions": { "type": "array", "items": {}, "const": [] }
  },
  "required": ["status", "prompt_path", "executor"]
}
```

**`build_workflow_response()` routing logic:**

```rust
let is_mainthread_only = executor.len() == 1 && executor[0] == "mainthread";
let is_subagent = executor.iter().any(|e| e.starts_with("subagent:"));

if is_mainthread_only || is_subagent {
    // Return prompt_path + executor to caller (mainthread skill dispatches)
} else {
    // Call run_agent() internally (Rust CLI dispatches external subprocess)
}
```
## State Machine

<!-- type: state-machine lang: mermaid -->

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Dispatching: action triggered
    Dispatching --> MainthreadExec: executor == mainthread
    Dispatching --> ExternalExec: executor != mainthread
    ExternalExec --> Verifying: exit_code == 0
    ExternalExec --> Retrying: exit_code != 0 && !retried
    Retrying --> ExternalExec: retry attempt
    Retrying --> MainthreadExec: retry_count > 0 (fallback)
    Verifying --> Done: verification passed
    Verifying --> MainthreadExec: passed == false && phase unchanged (fallback)
    Verifying --> PartialDone: passed == false && phase advanced
    MainthreadExec --> Done
    PartialDone --> [*]: resume via sdd_run_change(actual_phase)
    Done --> [*]
```


## Dependencies

<!-- type: dependency lang: mermaid -->

```mermaid
flowchart TD
    Config["cclab/config.toml\nworkflow.mode"] --> ER["executor-resolution\n(preset lookup)"]    ER --> MA["multi_agents\nexternal CLI subprocesses"]
    ER --> MCA["multi_claude_agents\nclaude --agent sdd-X"]
    ER --> CS["claude_subagents\nAgent tool"]
    ER --> MT["mainthread\ncurrent LLM"]
    MCA --> AgentDefs[".claude/agents/\nsdd-reference-context.md\nsdd-change-spec.md\nsdd-review.md\nsdd-change-implementation.md"]
    MCA --> BashHooks[".claude/hooks/\nsdd-readonly-bash.sh\nsdd-safe-bash.sh"]
    MCA --> PhaseCLAUDE["phase-specific CLAUDE.md\n(generated at runtime)"]
    MA --> ProviderModels["config/agents.md\nprovider model tables"]
    ER --> DA["delegate-agent\nrun_agent()"]
    DA --> Verify["STATE.yaml\nphase + artifact verification"]
```


## Interaction

<!-- type: interaction lang: mermaid -->

```mermaid
sequenceDiagram
    participant SM as State Machine
    participant ER as Executor Resolution
    participant RA as run_agent()
    participant EXT as External Agent
    participant MT as Mainthread

    SM->>ER: resolve(action)
    ER->>ER: load preset[mode][action] → executor
    alt executor == mainthread
        ER->>MT: execute(prompt)
        MT-->>SM: done
    else executor is external
        ER->>RA: run_agent(executor, prompt, change_id)
        RA->>EXT: spawn subprocess (CLI / Agent tool)
        EXT-->>RA: exit
        alt exit_code == 0
            RA->>RA: verify phase + artifact
            alt verification passed
                RA-->>SM: AgentResult{ok, passed=true}
            else phase unchanged (failure)
                RA->>MT: fallback: execute(prompt)
                MT-->>SM: done
            else phase advanced (partial)
                RA-->>SM: AgentResult{ok, passed=false, actual_phase}
            end
        else exit_code != 0 AND not retried
            RA->>EXT: retry once
            EXT-->>RA: exit
        else retried and still failing
            RA->>MT: fallback: execute(prompt)
            MT-->>SM: done
        end
    end
```


## CLI

<!-- type: cli lang: yaml -->

```yaml
# No new CLI subcommands. Execution mode is configured via cclab/config.toml.

config_field:
  path: cclab/config.toml
  section: workflow
  field: mode
  type: string
  values:
    - multi_agents
    - multi_claude_agents
    - claude_subagents
    - mainthread
  default: mainthread
  description: |
    Selects which preset table maps phase actions to executors.
    No per-action overrides allowed — preset is fixed per mode.

# Example config.toml:
# [workflow]
# mode = "multi_claude_agents"

agent_definitions:
  path: .claude/agents/
  used_by: multi_claude_agents
  files:
    - sdd-reference-context.md
    - sdd-change-spec.md
    - sdd-review.md
    - sdd-change-implementation.md

bash_hooks:
  path: .claude/hooks/
  used_by: multi_claude_agents
  files:
    - name: sdd-readonly-bash.sh
      used_by: [sdd-reference-context, sdd-review]
      allows: ["git log", "git diff", "git status", "git show", "ls", "cat", "find", "cargo test", "cargo check"]
    - name: sdd-safe-bash.sh
      used_by: [sdd-change-implementation]
      blocks: ["rm -rf", "git push", "git reset", "chmod 777"]
```

# Reviews

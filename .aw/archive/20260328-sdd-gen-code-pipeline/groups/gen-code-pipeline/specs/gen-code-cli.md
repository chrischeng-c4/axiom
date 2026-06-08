---
id: gen-code-cli
main_spec_ref: crates/cclab-sdd/interfaces/cli/commands.md
merge_strategy: extend
fill_sections: [overview, changes]
---

# Sdd Cli Context Command

## Overview

CLI interface additions to `cclab sdd` for agent context building (#946) and agent-optimized output (#949). Modifies the existing CLI commands spec to register two new subcommand/option entries.

| Attribute | Value |
|-----------|-------|
| Crate | cclab-sdd-cli |
| Spec target | `cclab/specs/crates/cclab-sdd/interfaces/cli/commands.md` |
| Issues | #946, #949 |
| New commands | `cclab sdd context <targets...> [--depth N]` |
| New options | `cclab sdd check --format agent` |
| Interface only | CLI-only — no MCP exposure (Q2 clarification) |
## Requirements

### Functional

| ID | Requirement | Source |
|----|-------------|--------|
| R1 | Add `context` subcommand to `cclab sdd` command tree: `cclab sdd context <targets...> [--depth N]` | #946 |
| R2 | `<targets...>` accepts one or more positional args in `file:symbol` format | #946 AC (agent-context-builder R1) |
| R3 | `--depth N` optional flag with default value 2 | #946 AC (agent-context-builder R2) |
| R4 | `context` subcommand outputs JSON to stdout (ContextResponse schema) | #946 (agent-context-builder R7, NF4) |
| R5 | Extend `check` subcommand `--format` accepted values to include `agent` | #949 (agent-output-format R1) |
| R6 | `check --format agent` outputs AgentOutput JSON to stdout | #949 (agent-output-format NF4) |
| R7 | Add `context` entry to CLI→Logic mapping table: routes to `ContextBuilder::build_context()` | #946 |
| R8 | Add `check --format agent` entry to CLI→Logic mapping: routes to `AgentOutputBuilder::build()` | #949 |

### Non-Functional

| ID | Requirement |
|----|-------------|
| NF1 | No MCP tool registration for `context` or `check --format agent` (Q2 clarification) |
| NF2 | Both new entries integrated into existing clap command tree in `cclab-sdd-cli/src/commands.rs` |
## Scenarios

### S1: Context command — basic usage

| Step | Action | Expected |
|------|--------|----------|
| 1 | `cclab sdd context src/services/user.py:get_user` | Parses target, uses default depth=2 |
| 2 | Output | JSON with `must_read`, `may_affect`, `type_context`, `stats` keys to stdout |
| 3 | Exit code | 0 on success |

### S2: Context command — custom depth

| Step | Action | Expected |
|------|--------|----------|
| 1 | `cclab sdd context src/a.py:foo --depth 1` | Parses depth=1 |
| 2 | Output | Traversal limited to 1 hop |

### S3: Context command — multiple targets

| Step | Action | Expected |
|------|--------|----------|
| 1 | `cclab sdd context src/a.py:foo src/b.py:bar` | Parses two targets |
| 2 | Output | Merged ContextResponse JSON |

### S4: Context command — invalid target format

| Step | Action | Expected |
|------|--------|----------|
| 1 | `cclab sdd context not_valid_format` | Missing `:symbol` separator |
| 2 | Exit | Non-zero exit code, error message to stderr |

### S5: Check with agent format

| Step | Action | Expected |
|------|--------|----------|
| 1 | `cclab sdd check src/ --format agent` | Accepted by `--format` enum |
| 2 | Output | AgentOutput JSON to stdout (symbols, imports, issues, impact, stats) |

### S6: Check format — existing formats unaffected

| Step | Action | Expected |
|------|--------|----------|
| 1 | `cclab sdd check src/ --format json` | Existing JSON format unchanged |
| 2 | `cclab sdd check src/ --format text` | Existing text format unchanged |
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

### Unit Tests (direct.rs)

| Test | Validates |
|------|-----------|
| `test_context_target_parse_valid` | Valid `file:symbol` format parses correctly (R2) |
| `test_context_target_parse_colon_in_path` | Handles colons in path via `rsplit_once` (R2) |
| `test_context_target_parse_no_colon` | Returns None for missing `:` separator (S4) |
| `test_context_target_parse_empty_file` | Returns None for `:symbol` format (S4) |
| `test_context_target_parse_empty_symbol` | Returns None for `file:` format (S4) |
| `test_context_target_parse_empty_string` | Returns None for empty string (S4) |
| `test_output_format_agent_accepted` | `OutputFormat::from_str("agent")` returns `Agent` (R5) |
| `test_output_format_json_unaffected` | Existing JSON format still works (S6) |
| `test_output_format_console_unaffected` | Existing console format still works (S6) |
| `test_output_format_markdown_unaffected` | Existing markdown format still works (S6) |

## Changes

```yaml
changes:
  - action: modify
    path: cclab/specs/crates/cclab-sdd/interfaces/cli/commands.md
    description: >
      Add `context` subcommand to Command Tree YAML block:
      ```
      context <targets...>:
        # targets: one or more file:symbol positional args
        --depth: integer (default 2)
      ```
      Add `agent` to check subcommand's --format accepted values.
      Add two rows to CLI → Logic Mapping table:
      - `sdd context <targets...>` → `context_builder::ContextBuilder::build_context()` → —
      - `sdd check --format agent` → `output::agent::AgentOutputBuilder::build()` → —
    requirements: [R1, R2, R3, R4, R5, R6, R7, R8]

  - action: modify
    path: crates/cclab-sdd-cli/src/commands.rs
    description: >
      Add `Context` variant to SDD CLI subcommand enum.
      Define context subcommand args: targets (Vec<String> positional), depth (Option<u32> default 2).
      Parse file:symbol format from each target string.
      Call ContextBuilder::build_context(request) and print JSON to stdout.
    requirements: [R1, R2, R3, R4, R7, NF2]

  - action: modify
    path: crates/cclab-sdd-cli/src/commands.rs
    description: >
      Extend existing check subcommand's --format clap value_parser to accept "agent" string.
      When format="agent", build SymbolTable + ImportGraph for checked files,
      pass to AgentOutputBuilder::build(), print AgentOutput JSON to stdout.
    requirements: [R5, R6, R8, NF2]
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

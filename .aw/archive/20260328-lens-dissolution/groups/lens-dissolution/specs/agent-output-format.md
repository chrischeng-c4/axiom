---
id: agent-output-format
main_spec_ref: crates/cclab-sdd/logic/agent-output-format.md
merge_strategy: new
fill_sections: [overview, requirements, schema, changes, test-plan]
filled_sections: [overview, requirements, scenarios, schema, changes, test-plan]
create_complete: true
---

# Agent Output Format

## Overview

Symbol-centric JSON output format optimized for LLM agent consumption. Adds `--format agent` to `cclab sdd check` (post lens dissolution), replacing file-centric diagnostic lists with a compact representation of symbols, dependency edges, issues, and impact scope.

| Attribute | Value |
|-----------|-------|
| Crate | cclab-sdd (post lens dissolution) |
| Module | `output` (extended — new `Agent` variant in `OutputFormat` enum) |
| Interface | CLI-only (`cclab sdd check --format agent`) — no MCP |
| Issue | #949 |
| Depends on | Reporter (output), SymbolTable (semantic/symbols), ImportGraph (graph/), FileResult/Diagnostic (lint pipeline), SearchIndex (search/) |
## Requirements

### Functional

| ID | Requirement | Source |
|----|-------------|--------|
| R1 | Add `Agent` variant to `OutputFormat` enum, selectable via `--format agent` | #949 AC |
| R2 | Output a top-level `symbols` map: symbol name → `{type, file, line, kind}` for every symbol defined in checked files | #949 format |
| R3 | Output an `imports` map: file path → list of imported symbol qualified names, derived from ImportGraph edges | #949 format |
| R4 | Output an `issues` array: each entry has `severity`, `symbol` (nearest enclosing symbol or file-level), `file`, `line`, `code`, `message` — no rule description, no SARIF boilerplate | #949 format, AC |
| R5 | Output an `impact` map: symbol qualified name → list of `file:line` locations that reference/call it, derived from SymbolTable references | #949 format |
| R6 | Attach the nearest enclosing symbol name to each diagnostic (symbol attribution) — if no enclosing symbol, use `"<file-level>"` | #949 design |
| R7 | Include type signatures inline in the `symbols` map via SymbolTable `type_info` or DeepTypeInferencer propagated types | #949 format |
| R8 | Output a `stats` summary: `files_checked`, `symbols_found`, `issues_count`, `impact_edges` | #949 compact |
| R9 | Compact output: omit empty maps/arrays, no pretty-printing by default, no redundant rule metadata | #949 AC |

### Non-Functional

| ID | Requirement |
|----|-------------|
| NF1 | CLI-only interface — no MCP tool exposure (clarification Q2) |
| NF2 | Module lives at `src/output/` after lens dissolution (not under `lens/`) |
| NF3 | Reuse existing SymbolTable, ImportGraph, SearchIndex — no new data structures for graph data |
| NF4 | Output must be valid JSON parseable by any JSON library |
| NF5 | Token-efficient: total output size for a typical 50-file project should be <50% of equivalent SARIF output |
## Scenarios

### S1: Python project with cross-file dependencies

| Step | Action | Expected |
|------|--------|----------|
| 1 | `cclab sdd check src/ --format agent` on a Python project with `db.py`, `handler.py`, `models.py` | Agent JSON output |
| 2 | `symbols` map | Contains `get_user: {type: "(int) -> User", file: "db.py", line: 42, kind: "function"}` |
| 3 | `imports` map | `handler.py` → `["db.get_user", "models.User"]` |
| 4 | `issues` array | `{severity: "error", symbol: "get_user", file: "db.py", line: 42, code: "PY101", message: "..."}` |
| 5 | `impact` map | `db.get_user` → `["handler.py:15", "api.py:33"]` |

### S2: No issues found — compact output

| Step | Action | Expected |
|------|--------|----------|
| 1 | `cclab sdd check clean_project/ --format agent` | Agent JSON output |
| 2 | `issues` key | Absent (omitted when empty, per R9) |
| 3 | `symbols` map | Present with all defined symbols |
| 4 | `impact` map | Present with reference edges even without issues |

### S3: File-level diagnostic without enclosing symbol

| Step | Action | Expected |
|------|--------|----------|
| 1 | Check a file with a top-level syntax issue (e.g. missing newline at EOF) | Agent JSON output |
| 2 | `issues` entry | `symbol` field is `"<file-level>"` (per R6) |

### S4: Mixed languages (Python + TypeScript)

| Step | Action | Expected |
|------|--------|----------|
| 1 | `cclab sdd check ./ --format agent` on a polyglot project | Single unified Agent JSON |
| 2 | `symbols` map | Contains symbols from both `.py` and `.ts` files |
| 3 | `imports` map | Separate entries for Python imports and TypeScript imports |

### S5: Large project — compactness validation

| Step | Action | Expected |
|------|--------|----------|
| 1 | `cclab sdd check large_project/ --format agent` vs `--format json` | Both complete successfully |
| 2 | Compare output sizes | Agent format output is <50% of JSON format output size (NF5) |
| 3 | Agent output | No `rule_description`, no `category` metadata, no `quick_fixes` — only actionable fields |
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

### Unit Tests

| Test | File | Validates |
|------|------|-----------|
| `test_output_format_agent_variant` | `output.rs` | `OutputFormat::from_str("agent")` returns `Some(OutputFormat::Agent)` (R1) |
| `test_build_symbols_from_symbol_table` | `output/agent.rs` | Builds symbols map from SymbolTable entries with correct type, file, line, kind fields (R2, R7) |
| `test_build_imports_from_graph` | `output/agent.rs` | Builds imports map from ImportGraph edges, maps file → imported symbol names (R3) |
| `test_build_issues_with_symbol_attribution` | `output/agent.rs` | Each diagnostic is attributed to nearest enclosing symbol via range binary search (R4, R6) |
| `test_build_issues_file_level_fallback` | `output/agent.rs` | Diagnostic outside any symbol scope gets `symbol: "<file-level>"` (R6) |
| `test_build_impact_from_references` | `output/agent.rs` | Builds impact map from SymbolReference entries grouped by target symbol (R5) |
| `test_compact_output_omits_empty` | `output/agent.rs` | When no issues exist, serialized JSON omits `issues` key entirely (R9) |
| `test_compact_output_omits_empty_imports` | `output/agent.rs` | When no import edges exist, serialized JSON omits `imports` key (R9) |
| `test_stats_computation` | `output/agent.rs` | Stats correctly counts files_checked, symbols_found, issues_count, impact_edges (R8) |
| `test_agent_output_valid_json` | `output/agent.rs` | Serialized output is parseable by `serde_json::from_str` round-trip (NF4) |

### Integration Tests

| Test | Validates |
|------|-----------|
| `test_cli_check_format_agent_python` | End-to-end: `cclab sdd check fixtures/python/ --format agent` produces valid AgentOutput JSON with symbols, imports, issues, impact (R1-R9, S1) |
| `test_cli_check_format_agent_clean` | Clean project: `--format agent` produces output with symbols but no issues key (S2, R9) |
| `test_cli_check_format_agent_polyglot` | Mixed Python+TypeScript: single unified output with symbols from both languages (S4) |
| `test_agent_output_smaller_than_json` | Size comparison: agent format output size < 50% of standard JSON output size on fixture project (NF5, S5) |
## Changes

```yaml
changes:
  - action: modify
    path: crates/cclab-sdd/src/output.rs
    description: >
      Add `Agent` variant to `OutputFormat` enum.
      Add `"agent"` match arm in `OutputFormat::from_str()`.
      Add `generate_agent()` method to `Reporter`.
      Wire `OutputFormat::Agent` in `Reporter::generate()` match.
    requirements: [R1]

  - action: create
    path: crates/cclab-sdd/src/output/agent.rs
    description: >
      Agent output builder. Struct `AgentOutputBuilder` with methods:
      - `build_symbols(results, symbol_tables) -> BTreeMap<String, SymbolDef>` — iterates SymbolTable for each file, emits qualified name → {type, file, line, kind}
      - `build_imports(import_graph) -> BTreeMap<String, Vec<String>>` — iterates ImportGraph edges, maps file → imported symbol qualified names
      - `build_issues(results, symbol_tables) -> Vec<AgentIssue>` — maps each Diagnostic to AgentIssue with nearest enclosing symbol attribution via binary search on SymbolTable ranges
      - `build_impact(symbol_tables) -> BTreeMap<String, Vec<String>>` — iterates SymbolReference entries, groups by target symbol → "file:line" locations
      - `build(results, symbol_tables, import_graph) -> AgentOutput` — orchestrates all builders, computes stats, omits empty fields
    requirements: [R2, R3, R4, R5, R6, R7, R8, R9]

  - action: create
    path: crates/cclab-sdd/src/output/agent_types.rs
    description: >
      Serde-serializable structs: AgentOutput, SymbolDef, AgentIssue, AgentStats.
      Derives Serialize. Uses `#[serde(skip_serializing_if = "...is_empty")]` for imports, issues, impact to enforce compactness (R9).
    requirements: [R2, R4, R5, R8, R9, NF4]

  - action: modify
    path: crates/cclab-sdd/src/lib.rs
    description: "Ensure `pub mod output;` exists (may already exist post dissolution)"
    requirements: [NF2]

  - action: modify
    path: crates/cclab-sdd-cli/src/commands.rs
    description: >
      Extend `check` subcommand's `--format` accepted values to include `agent`.
      When format=agent, additionally build SymbolTable and ImportGraph for checked files,
      then pass them to AgentOutputBuilder.
    requirements: [R1, NF1]
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


## Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "AgentOutput",
  "description": "Symbol-centric analysis output optimized for LLM agent consumption",
  "type": "object",
  "required": ["symbols", "stats"],
  "properties": {
    "symbols": {
      "type": "object",
      "description": "Map of symbol qualified name to definition info",
      "additionalProperties": {
        "$ref": "#/$defs/SymbolDef"
      }
    },
    "imports": {
      "type": "object",
      "description": "Map of file path to list of imported symbol qualified names",
      "additionalProperties": {
        "type": "array",
        "items": { "type": "string" }
      }
    },
    "issues": {
      "type": "array",
      "description": "Diagnostics attributed to nearest enclosing symbol",
      "items": { "$ref": "#/$defs/AgentIssue" }
    },
    "impact": {
      "type": "object",
      "description": "Map of symbol qualified name to list of file:line references",
      "additionalProperties": {
        "type": "array",
        "items": { "type": "string", "pattern": "^.+:\\d+$" }
      }
    },
    "stats": {
      "$ref": "#/$defs/AgentStats"
    }
  },
  "$defs": {
    "SymbolDef": {
      "type": "object",
      "required": ["file", "line", "kind"],
      "properties": {
        "type": { "type": "string", "description": "Type signature string, e.g. (int) -> User" },
        "file": { "type": "string", "description": "Relative file path from project root" },
        "line": { "type": "integer", "minimum": 1, "description": "1-based line number" },
        "kind": {
          "type": "string",
          "enum": ["function", "class", "method", "variable", "constant", "interface", "type_alias", "module"],
          "description": "Symbol kind"
        }
      }
    },
    "AgentIssue": {
      "type": "object",
      "required": ["severity", "symbol", "file", "line", "code", "message"],
      "properties": {
        "severity": { "type": "string", "enum": ["error", "warning", "info", "hint"] },
        "symbol": { "type": "string", "description": "Nearest enclosing symbol name or <file-level>" },
        "file": { "type": "string" },
        "line": { "type": "integer", "minimum": 1 },
        "code": { "type": "string", "description": "Diagnostic rule code (e.g. PY101)" },
        "message": { "type": "string" }
      }
    },
    "AgentStats": {
      "type": "object",
      "required": ["files_checked", "symbols_found", "issues_count", "impact_edges"],
      "properties": {
        "files_checked": { "type": "integer", "minimum": 0 },
        "symbols_found": { "type": "integer", "minimum": 0 },
        "issues_count": { "type": "integer", "minimum": 0 },
        "impact_edges": { "type": "integer", "minimum": 0 }
      }
    }
  }
}
```

# Reviews

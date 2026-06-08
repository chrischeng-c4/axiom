---
id: cli-commands-check-alignment
type: spec
title: "SDD CLI Commands"
version: 2
files:
  - cclab-sdd-cli/src/commands.rs
  - cclab-sdd-cli/src/lib.rs
main_spec_ref: "crates/cclab-sdd/interfaces/cli/commands.md"
merge_strategy: extend
fill_sections: [overview, requirements, scenarios, cli, changes]
filled_sections: [overview, requirements, scenarios, cli, changes]
create_complete: true
---

# CLI Commands

Registered as `cclab sdd` via `CliModule` trait (`cclab-sdd-cli` crate).

## Command Tree

```yaml
cclab sdd:
  status <change_id>:
    --json: bool
  list:
    --archived: bool
  view [change]:
    # Opens Plan Viewer in browser
  fillback:
    --path: string
    --module: string
    --force: bool
  platform:
    # subcommands: config, sync
  run-change:
    --change-id: string (required)
    --description: string
    --issue: string (repeatable)
    --git-workflow: enum [new_branch, in_place]
  workflow <action> <change_id> [extra_args_json]:
    # Routes to sdd_workflow_* tools
    # extra_args_json: optional JSON for params beyond change_id/project_path
  artifact <action> <change_id> <payload_path>:
    # Routes to sdd_artifact_* tools
    # payload_path: path to JSON file with payload content
```

## CLI → Logic Mapping

| CLI Command | Logic Function | Tool Name |
|-------------|---------------|-----------|
| `sdd status <id>` | `status::run()` | — |
| `sdd list` | `list::run()` | — |
| `sdd view` | `view::run_view()` | — |
| `sdd fillback` | `fillback::run()` | `sdd_analyze_code_for_spec` |
| `sdd run-change` | `workflow::execute()` | `sdd_run_change` |
| `sdd workflow <action> <id>` | `ToolRegistry::call_tool()` | `sdd_workflow_*` |
| `sdd artifact <action> <id> <payload>` | `ToolRegistry::call_tool()` | `sdd_artifact_*` |

## Registration

```yaml
# cclab-sdd-cli/src/lib.rs
CliModule:
  name: "sdd"
  struct: SddCli
  distributed_slice: CLI_MODULES

# cclab-cli/src/main.rs
force_link: cclab_sdd_cli
```


## Changes

### Add: check-alignment command

New command: `cclab sdd check-alignment [path]`

CLI entry point for `spec_alignment::check()`. Validates spec `.md` files for format compliance and logical consistency. Follows same pattern as `validate-spec-structure`.

```yaml
cclab sdd:
  check-alignment [path]:
    # Optional positional: file or directory path (defaults to cclab/specs/)
    # Delegates to spec_alignment::check()
    # --json flag for JSON output
    # Exit 0 = clean, exit 1 = violations
```

### Update: Command Tree

Extend the `cclab sdd` command tree block:

```yaml
cclab sdd:
  # ... existing commands unchanged ...
  check-alignment [path]:
    # Validates spec files for format compliance + logical consistency
    # --json: emit CheckResult as JSON
    # Exit 0 if clean, 1 if violations
```

### Update: CLI → Logic Mapping

Append one row to the mapping table:

| CLI Command | Logic Function | Tool Name |
|-------------|---------------|-----------|
| `sdd check-alignment [path]` | `check_alignment::run()` → `spec_alignment::check()` | — |

### File Changes

```yaml
changes:
  - path: crates/cclab-sdd-cli/src/commands.rs
    action: modify
    description: "Add CheckAlignment variant to SddCommand enum with optional path: Option<String> and --json: bool flag. Add match arm dispatching to check_alignment::run()."

  - path: crates/cclab-sdd-cli/src/check_alignment.rs
    action: create
    description: "CLI handler — resolve path (default cclab/specs/), call spec_alignment::check(), format text or JSON output, set exit code."

  - path: crates/cclab-sdd-cli/src/lib.rs
    action: modify
    description: "Add mod check_alignment declaration."
```
## Overview

Extends `cclab sdd` CLI command tree with `check-alignment [path]` subcommand.

**What**: Adds CLI entry point for `spec_alignment::check()` — validates spec `.md` files for format compliance and logical consistency.

**Why**: The library function (defined in `check-alignment` logic spec) needs a CLI surface. Follows the same pattern as `validate-spec-structure` — optional positional path, `--json` flag, hard error on violations.

| Aspect | Value |
|--------|-------|
| Command | `cclab sdd check-alignment [path]` |
| Handler crate | `cclab-sdd-cli` |
| Logic delegation | `spec_alignment::check()` from `cclab-sdd` |
| Output modes | text (default), JSON (`--json`) |
| Exit codes | 0 = clean, 1 = violations |


## Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| CR1 | Add `CheckAlignment` variant to `SddCommand` enum in `commands.rs` with optional positional `path: Option<String>` and `--json: bool` flag. | high |
| CR2 | Create `check_alignment.rs` CLI handler: resolve path (default `cclab/specs/`), call `spec_alignment::check(path)`, format output, set exit code. | high |
| CR3 | Text output format: `FAIL  {path}` with indented `  {kind}: {message}` lines per violation, `OK    {path}` for clean files. Matches `validate-spec-structure` output pattern. | high |
| CR4 | JSON output format: emit `CheckResult` struct as JSON when `--json` flag is set. | high |
| CR5 | Exit code: `std::process::exit(0)` if `CheckResult.passed == true`, `std::process::exit(1)` otherwise. | high |
| CR6 | Wire `CheckAlignment` variant in command dispatch match arm to `check_alignment::run()`. | high |


## Scenarios

### Scenario: Command with no path defaults to cclab/specs/
- **GIVEN** user runs `cclab sdd check-alignment` with no arguments
- **WHEN** the CLI handler resolves the path
- **THEN** it passes `{project_root}/cclab/specs/` to `spec_alignment::check()`

### Scenario: Command with file path checks single file
- **GIVEN** user runs `cclab sdd check-alignment cclab/specs/crates/cclab-sdd/logic/check-alignment.md`
- **WHEN** the CLI handler receives a file path
- **THEN** it passes that single file to `spec_alignment::check()`
- **AND** output shows one line (`OK` or `FAIL`)

### Scenario: Text output on violations
- **GIVEN** `spec_alignment::check()` returns `CheckResult` with 1 failing file and 2 violations
- **WHEN** `--json` is not set
- **THEN** output is:
  ```
  FAIL  crates/cclab-sdd/logic/foo.md
    missing_section_annotation: section "Commands" at line 32
    format_priority_violation: section "Config" expects json block
  ```
- **AND** exit code is 1

### Scenario: JSON output mode
- **GIVEN** `spec_alignment::check()` returns a `CheckResult`
- **WHEN** `--json` flag is set
- **THEN** output is `CheckResult` serialized as JSON
- **AND** exit code matches `passed` field

### Scenario: Clean run exits 0
- **GIVEN** all checked files pass validation
- **WHEN** `spec_alignment::check()` returns `passed: true`
- **THEN** exit code is 0
- **AND** text output shows `OK` for every file


## CLI

```yaml
cclab sdd:
  check-alignment [path]:
    args:
      path:
        type: string
        required: false
        description: "File or directory path. If directory, recursively checks all .md files. Defaults to cclab/specs/ if omitted."
    flags:
      --json:
        type: bool
        default: false
        description: "Emit results as JSON (CheckResult schema) instead of text"
    behavior:
      - Resolve path: if omitted, use {project_root}/cclab/specs/
      - Delegate to spec_alignment::check(resolved_path)
      - Text output (default): OK/FAIL per file, indented violations under FAIL
      - JSON output (--json): serialize CheckResult as JSON
      - Exit 0 if passed, exit 1 if violations
    output_format:
      text: |
        FAIL  {relative_path}
          {violation_kind}: {message}
        OK    {relative_path}
      json: |
        { "files": [...], "total_violations": N, "passed": bool }
    exit_codes:
      0: "All files passed"
      1: "One or more violations found"
```

# Reviews

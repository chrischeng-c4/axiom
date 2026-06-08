---
id: cli-commands-update
type: spec
title: "SDD CLI Commands"
version: 2
files:
  - cclab-sdd-cli/src/commands.rs
  - cclab-sdd-cli/src/lib.rs
main_spec_ref: crates/cclab-sdd/interfaces/cli/commands.md
merge_strategy: extend
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

### Add: scaffold-spec command

New command: `cclab sdd scaffold-spec <folder>`

Scaffolds the canonical inner-directory structure for a spec root. Creates subdirectories and a top-level `README.md`. No stub files or `.gitkeep` — directory skeleton only.

```yaml
cclab sdd:
  scaffold-spec <folder>:
    # Positional: spec root path (e.g., cclab/specs/crates/cclab-agent/)
    # Creates: interfaces/{mcp,cli,rest,...}/, logic/, config/ subdirs
    # Creates: README.md at spec root if absent
    # Rules sourced from logic/spec-structure.md
```

### Add: validate-spec-structure command

New command: `cclab sdd validate-spec-structure [path]`

Lints a spec root against canonical structure rules (defined in `logic/spec-structure.md`). Hard error — exits non-zero on any violation. No lenient/warning mode.

```yaml
cclab sdd:
  validate-spec-structure [path]:
    # Optional positional: spec root path (defaults to all crate roots in cclab/specs/)
    # Validates: no loose files at root (except README.md), interfaces/ present for API crates,
    #            one concept per file, subdirs match domain subsystems
    # Failure mode: hard error, exits non-zero, prints violation list
```

### Update: Command Tree

Extend the `cclab sdd` command tree block with the two new entries:

```yaml
cclab sdd:
  # ... existing commands unchanged ...
  scaffold-spec <folder>:
    # Creates canonical subdirectory skeleton for a spec root
    # Output: interfaces/{type}/ subdirs + README.md (no stub files)
  validate-spec-structure [path]:
    # Lints spec root against rules in logic/spec-structure.md
    # Hard error on violations — exits non-zero
```

### Update: CLI → Logic Mapping

Append two rows to the mapping table:

| CLI Command | Logic Function | Tool Name |
|-------------|---------------|-----------|
| `sdd scaffold-spec <folder>` | `scaffold_spec::run()` | — |
| `sdd validate-spec-structure [path]` | `validate_spec_structure::run()` | — |
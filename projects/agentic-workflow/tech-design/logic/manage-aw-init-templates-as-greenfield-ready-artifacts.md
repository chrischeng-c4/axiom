---
id: aw-greenfield-project-bootstrap
summary: Add greenfield project bootstrap over managed AW init templates.
fill_sections: [logic, cli, config, unit-test]
---

# AW Greenfield Project Bootstrap

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: aw-greenfield-project-bootstrap-flow
entry: start
nodes:
  start:
    kind: start
    label: "aw new receives project name plus optional --path"
  resolve_target:
    kind: process
    label: "Resolve target dir: --path when supplied, otherwise current directory joined with project name"
  target_exists:
    kind: decision
    label: "Target path already exists?"
  reject_non_empty:
    kind: terminal
    label: "Fail unless the existing target is an empty directory or --force is supplied"
  create_target:
    kind: process
    label: "Create target directory and minimal greenfield marker files"
  run_init:
    kind: process
    label: "Run init installer against target using managed templates and non-interactive platform defaults"
  report:
    kind: terminal
    label: "Print target path and next command for the user"
edges:
  - from: start
    to: resolve_target
    label: "parse args"
  - from: resolve_target
    to: target_exists
    label: "stat target"
  - from: target_exists
    to: reject_non_empty
    label: "exists and not safely reusable"
  - from: target_exists
    to: create_target
    label: "missing or reusable"
  - from: create_target
    to: run_init
    label: "directory ready"
  - from: run_init
    to: report
    label: ".aw installed"
---
flowchart TD
    start([aw new name --path dir]) --> resolve_target[Resolve target directory]
    resolve_target --> target_exists{Target exists?}
    target_exists -->|non-empty without force| reject_non_empty([error])
    target_exists -->|missing or reusable| create_target[Create target directory]
    create_target --> run_init[Install managed AW templates via init]
    run_init --> report([print next command])
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: aw new
    summary: Create a greenfield project directory and bootstrap Agentic Workflow.
    usage: "aw new <name> [--path <path>] [--force] [--no-init]"
    args:
      - name: name
        required: true
        description: Project directory name when --path is not supplied.
    flags:
      - name: --path
        value: path
        required: false
        description: Explicit target directory. When omitted, target is ./<name>.
      - name: --force
        required: false
        description: Allow reusing an existing empty directory and pass force refresh to init.
      - name: --no-init
        required: false
        description: Create the directory without running aw init.
    behavior:
      - Reject existing non-empty target directories unless --force is supplied.
      - Create the target directory before running init.
      - Run the same template installer used by aw init so greenfield and refresh paths share managed assets.
      - Print the resolved project path and next command.
  - name: aw init
    summary: Initialize or refresh Agentic Workflow in the current directory.
    usage: "aw init [--force]"
    relationship_to_new: "aw new is a greenfield wrapper; aw init remains the in-place bootstrap/refresh command."
```

## Config
<!-- type: config lang: yaml -->

```yaml
greenfield_bootstrap:
  command: aw new
  target_resolution:
    default_parent: "."
    path_flag: "--path"
    path_flag_semantics: "When supplied, use the exact path as the target directory."
  init_behavior:
    default: run_aw_init_after_directory_creation
    skip_flag: "--no-init"
    template_source: "projects/agentic-workflow/templates/cli/mainthread"
    issue_platform_selection: "non_interactive_defaults"
  safety:
    existing_non_empty_directory: reject
    existing_empty_directory: allow
    force_flag: "--force"
  generated_files:
    - ".aw/config.toml"
    - ".aw/tech-design/"
    - "CLAUDE.md"
    - ".claude/skills/"
    - ".claude/settings.json"
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: aw-greenfield-project-bootstrap-tests
title: AW greenfield bootstrap unit tests
requirements:
  UT1:
    text: aw new resolves its default target as current_dir/name.
    type: test
    priority: high
    verification: unit
  UT2:
    text: aw new uses --path as the exact target directory when supplied.
    type: test
    priority: high
    verification: unit
  UT3:
    text: aw new creates a missing target directory and allows an existing empty directory.
    type: test
    priority: high
    verification: unit
  UT4:
    text: aw new rejects file targets and existing non-empty directories before running init.
    type: test
    priority: high
    verification: unit
  UT5:
    text: aw new delegates bootstrap installation to the same in-place init installer used by aw init.
    type: test
    priority: medium
    verification: unit
---
requirementDiagram
  requirement UT1 {
    id: UT1
    text: "default target resolution"
    risk: low
    verifymethod: test
  }
  requirement UT2 {
    id: UT2
    text: "explicit path target resolution"
    risk: low
    verifymethod: test
  }
  requirement UT3 {
    id: UT3
    text: "safe reusable target directory"
    risk: medium
    verifymethod: test
  }
  requirement UT4 {
    id: UT4
    text: "unsafe target rejection"
    risk: medium
    verifymethod: test
  }
  requirement UT5 {
    id: UT5
    text: "shared init installer delegation"
    risk: medium
    verifymethod: test
  }
```

---
id: cli-hints-impl-prompt
type: spec
title: "Implementation — Logic"
version: 2
files:
  - tools/change_impl/common.rs
  - tools/change_impl/create.rs
  - tools/change_impl/review.rs
  - tools/change_impl/revise.rs
  - prompts/implement.md
  - prompts/code_review.md
  - prompts/resolve.md
main_spec_ref: "crates/cclab-sdd/logic/implement-task.md"
merge_strategy: extend
fill_sections: [overview, changes]
filled_sections: [overview, changes]
create_complete: true
---

# Implementation

## Phase Transition

```yaml
from: ChangeSpecReviewed (APPROVED) | ChangeImplementationReviewed | ChangeImplementationRevised
to: ChangeImplementationCreated | ChangeImplementationReviewed | ChangeImplementationRevised
terminal: all specs approved → ChangeMergeCreated
executor: [mainthread]
crr: true  # per-spec CRR cycle
max_revisions_per_spec: 2  # terminal failure on exceed
```

## Sub-State Machine

```mermaid
stateDiagram-v2
    [*] --> ImplementSpec: first spec in order
    ImplementSpec --> ImplementSpec: next spec (not last)
    ImplementSpec --> WriteDiff: last spec implemented
    WriteDiff --> ReviewSpec: diff captured
    ReviewSpec --> ReviseSpec: REVIEWED verdict
    ReviewSpec --> NextSpec: APPROVED
    ReviseSpec --> ReviewSpec: re-review
    NextSpec --> ImplementSpec: more specs
    NextSpec --> AdvanceToMerge: all approved
    ImplementSpec --> ImplementSpecWithCodegen: has_json_schema or has_api_spec
    ImplementSpecWithCodegen --> WriteDiff
    state "revision >= 2" as terminal
    ReviseSpec --> terminal: exceeded limit
```

### ImplSubState enum

```yaml
ImplSubState:
  NoSpecs: "No change specs found — error"
  ImplementSpec: "Implement code for a spec (is_first flag for begin prompt)"
  ImplementSpecWithCodegen: "Spec eligible for codegen path"
  WriteDiff: "All specs implemented — capture git diff"
  ReviewSpec: "Review implementation for a spec"
  ReviseSpec: "Fix review issues for a spec"
  TerminalFailure: "Spec exceeded revision limit (max 2)"
  AdvanceToMerge: "All specs implemented and approved"
```

## Spec Execution Order

Kahn's algorithm on `refs:` frontmatter — topological sort of spec DAG.

```yaml
# From change_impl/common.rs
function: build_spec_execution_order
input: groups/{group_id}/specs/ (Path)
output: Vec<String>  # ordered spec ids
```

## Codegen Routing

```mermaid
flowchart TD
    Start([resolve spec]) --> CheckFrontmatter{has_json_schema or has_api_spec?}
    CheckFrontmatter -->|yes| Codegen[ImplementSpecWithCodegen]
    CheckFrontmatter -->|no| Manual[ImplementSpec]
    Codegen --> PrismGen[prism_generate_from_spec]
    Manual --> AgentImpl[agent implements manually]
```

## Prompt Templates

### BeginImplementation (first spec)

```markdown
# Task: Implement change '{{change_id}}'

Read all approved specs and implement them in dependency order.

1. Read specs: `groups/{{group_id}}/specs/*.md`
2. Read requirements: `groups/{{group_id}}/requirements.md`
3. Implement ALL tasks following layer order: data → logic → integration → testing
4. Maintain code quality: tests, error handling, documentation
```

### ImplementSpec (per-spec)

```markdown
# Task: Implement spec '{{spec_id}}'

Read the spec and implement the code changes it describes.

{{#if revision_count > 0}}
## Previous Review Feedback
Read the review in implementation.md and address all issues.
{{/if}}

## Steps
1. Read spec: `changes/{{change_id}}/groups/{{group_id}}/specs/{{spec_id}}.md`
2. Implement code changes
3. Run tests to verify
```

### WriteDiff

```markdown
# Task: Capture implementation diff

Run `git diff` and write the result to implementation.md via
`sdd_artifact_create_change_implementation(diff=..., summary=...)`.
```

### ReviewSpec (code review)

```markdown
# Task: Review implementation of '{{spec_id}}'

## Checklist

1. Requirements: all spec requirements implemented
2. Tests: adequate coverage, passing
3. Security: no OWASP vulnerabilities
4. Best practices: idiomatic code, proper error handling
5. Performance: no obvious bottlenecks

## Verdict

- APPROVED: implementation matches spec
- REVIEWED: issues found (HIGH/MEDIUM/LOW severity)
- REJECTED: fundamentally wrong approach
```

### ReviseSpec

```markdown
# Task: Fix review issues for '{{spec_id}}'

1. Read review in implementation.md (`## Review: {{spec_id}}`)
2. Fix all HIGH severity issues
3. Fix MEDIUM issues if feasible
4. Run tests after fixes
```

### TerminalFailure

```markdown
# Spec '{{spec_id}}' exceeded revision limit ({{revisions}}/2)

This spec has failed review {{revisions}} times. Options:
1. Skip this spec and continue with remaining specs
2. Manually fix the issues
3. Reject the change
```

## Revision Tracking

Per-spec revision tracking via `STATE.yaml`:

```yaml
task_revisions:
  auth-api: 1
  db-migration: 0
current_task_id: auth-api
```

- `MAX_SPEC_REVISIONS = 2`
- Terminal failure triggers when `task_revisions[spec_id] >= 2`
- Terminal failure resets `task_revisions` and phase to allow retry

## Inline Review Format

Reviews written as `## Review: {spec_id}` sections in `implementation.md`:

```markdown
## Review: auth-api

**Verdict:** APPROVED
**Summary:** Implementation matches spec requirements.

### Checklist
- [x] Requirements coverage
- [x] Test coverage
- [x] Security review
```

## Side Effects

| Action | STATE.yaml change |
|--------|-------------------|
| Begin implementation | `phase → ChangeImplementationCreated`, `current_task_id` set |
| Write diff | `implementation.md` created with git diff |
| Review (APPROVED) | Mark spec done in inline reviews |
| Review (REVIEWED) | `phase → ChangeImplementationReviewed` |
| Revise | `phase → ChangeImplementationRevised`, `task_revisions.{spec_id} += 1` |
| Terminal failure | Log warning, skip spec or halt |
| All specs approved | `phase → ChangeMergeCreated` (via advance) |


## Overview

Add code intelligence CLI hints to the implementation prompt so the agent knows which `cclab sdd` commands are available for codebase exploration.

| Aspect | Detail |
|--------|--------|
| Target | `build_implement_code_prompt` in `tools/create_change_impl.rs` |
| Current | CLI Commands block lists only spec-read and workflow-advance commands |
| New | Append a Code Intelligence hints subsection listing daemon commands: `symbols`, `hover`, `references`, `impact`, `context` |
| Condition | Hints added only when executor is `[mainthread]` (agent has bash access) |
| Scope | `create_change_impl.rs` prompt builder only — no new functions or data types |

### Current CLI Commands block

The prompt currently ends with:

```markdown
## CLI Commands

```
# Read spec
Read file: {spec_path}

# Advance implementation workflow
cclab sdd workflow create-change-implementation {cid}
```
```

### New CLI Commands block

When executor is mainthread, the prompt appends a Code Intelligence subsection:

```markdown
## CLI Commands

```
# Read spec
Read file: {spec_path}

# Advance implementation workflow
cclab sdd workflow create-change-implementation {cid}

# Code intelligence — explore codebase before making changes
cclab sdd symbols <file>              # list symbols in a file
cclab sdd hover <file> <line> <col>   # type info for a symbol
cclab sdd references <file> <line> <col>  # find all references
cclab sdd impact <file> <line> <col>  # analyze change impact
cclab sdd context <file:symbol...> [--depth N]  # cross-ref context
```
```

### Constraints

- Hints are static text — no AST extraction or symbol resolution
- Gate on executor: only include when `executor == ["mainthread"]`
- Falls back to current behavior (no hints) for non-mainthread executors
- Does not affect other prompt builders (tests, codegen, review, revise)
## Logic

<!-- type: logic lang: mermaid -->

Prompt enrichment pipeline — runs inside `build_implement_code_prompt` when change spec has targets.

```mermaid
---
id: lens-impl-prompt-enrichment
refs:
  - $ref: "changes-section-schema#changes-section-v2"
---
flowchart TD
    Start([build_implement_code_prompt called]) --> ReadSpec[read change spec file]
    ReadSpec --> FindChanges{has changes section?}
    FindChanges -->|no| Fallback[generate standard prompt]
    FindChanges -->|yes| ParseYAML[parse changes YAML]
    ParseYAML --> FilterModify[filter entries where action=MODIFY and targets non-empty]
    FilterModify --> HasTargets{any MODIFY with targets?}
    HasTargets -->|no| Fallback
    HasTargets -->|yes| ForEachFile[for each file with targets]
    ForEachFile --> ParseAST[AstExtractor::parse_file path]
    ParseAST --> ASTOk{parse succeeded?}
    ASTOk -->|no| WarnUnresolved[emit warning: could not parse file]
    ASTOk -->|yes| ForEachTarget[for each target in entry]
    ForEachTarget --> MatchSymbol{symbol found in ModuleInfo?}
    MatchSymbol -->|no| WarnMissing[emit warning: symbol not found]
    MatchSymbol -->|yes| ExtractRange[read file lines start_line..end_line]
    ExtractRange --> BuildBlock[build target context block]
    BuildBlock --> ForEachTarget
    WarnMissing --> ForEachTarget
    ForEachTarget -->|done| BuildDNT[build DO NOT MODIFY list from do_not_touch]
    BuildDNT --> ForEachFile
    WarnUnresolved --> ForEachFile
    ForEachFile -->|done| AssemblePrompt[assemble enriched prompt]
    AssemblePrompt --> Done([return prompt string])
    Fallback --> Done
```

### Symbol matching rules

```yaml
match_rules:
  - target_type: function
    match: "Symbol where kind=Function and name matches target.name"
  - target_type: struct
    match: "Symbol where kind=Struct and name matches target.name"
  - target_type: enum
    match: "Symbol where kind=Enum and name matches target.name"
  - target_type: trait
    match: "Symbol where kind=Interface and name matches target.name"
  - target_type: impl
    match: "Symbol where kind=Class and name matches target.name"
  - target_type: method
    match: "Symbol where kind=Function and name matches ImplName::target.name"
```

### Enriched prompt template (per-target block)

```markdown
### Target: {{target.type}} `{{target.name}}` in `{{file.path}}`

**Change**: {{target.change}}
{{#if target.position}}**Position**: {{target.position}}{{/if}}
{{#if target.anchor}} relative to `{{target.anchor}}`{{/if}}

**Current code** (lines {{start_line}}-{{end_line}}):
~~~{{lang}}
{{extracted_source}}
~~~
```

### DO NOT MODIFY block template

```markdown
## DO NOT MODIFY

The following symbols must not be changed:
{{#each do_not_touch_entries}}
- `{{name}}` in `{{file_path}}` (lines {{start_line}}-{{end_line}})
{{/each}}
```

### Function signatures

```yaml
functions:
  - name: parse_changes_targets
    location: tools/create_change_impl.rs
    input: spec_content (String)
    output: "Result<Vec<FileChangeEntry>>"
    description: "Parse changes section YAML from spec markdown, extract FileChangeEntry list"

  - name: resolve_target_symbols
    location: tools/create_change_impl.rs
    input: "file_path (Path), targets (Vec<ChangeTarget>), project_root (Path)"
    output: "Vec<ResolvedTarget>"
    description: "Call AstExtractor for file, match targets against symbols, extract source ranges"

  - name: build_enriched_context
    location: tools/create_change_impl.rs
    input: "resolved_targets (Vec<ResolvedTarget>), do_not_touch (Vec<ResolvedSymbol>)"
    output: String
    description: "Format resolved targets and DNT list into markdown prompt sections"
```

### Data types

```yaml
FileChangeEntry:
  path: String
  action: String  # CREATE | MODIFY | DELETE
  description: Option<String>
  targets: Vec<ChangeTarget>
  do_not_touch: Vec<String>

ChangeTarget:
  type: String  # function | struct | enum | trait | impl | method
  name: String
  change: String
  anchor: Option<String>
  position: Option<String>  # before | after | replace | append

ResolvedTarget:
  target: ChangeTarget
  file_path: String
  start_line: usize
  end_line: usize
  source_code: String
  lang: String  # rs | py | ts | go — inferred from file extension

ResolvedSymbol:
  name: String
  file_path: String
  start_line: usize
  end_line: usize
```


## Changes

```yaml
_sdd:
  id: cli-hints-impl-prompt-changes
changes:
  - path: crates/cclab-sdd/src/tools/create_change_impl.rs
    action: MODIFY
    description: "Add code intelligence CLI hints to implementation prompt when executor is mainthread"
    targets:
      - type: function
        name: build_implement_code_prompt
        change: "Resolve executor before building prompt string. When executor contains only 'mainthread', append code intelligence hint lines to the CLI Commands block: cclab sdd symbols, hover, references, impact, context with usage examples. Move get_executor_chain call before prompt format! so executor is available for the gate check."
    do_not_touch:
      - build_implement_tests_prompt
      - build_codegen_prompt
      - build_write_diff_prompt
      - extract_change_targets
      - auto_populate_impl_baseline

  - path: cclab/specs/crates/cclab-sdd/logic/implement-task.md
    action: MODIFY
    description: "Document code intelligence CLI hints in the ImplementSpec prompt template"
    section: "Prompt Templates"
    targets:
      - type: function
        name: "ImplementSpec template"
        change: "Add optional Code Intelligence hints subsection showing cclab sdd daemon commands (symbols, hover, references, impact) and context command. Annotate as mainthread-only."
```
# Reviews

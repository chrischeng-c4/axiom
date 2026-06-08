---
id: score-agent-rename-hook-dispatch
main_spec_ref: projects/agentic-workflow/logic/dispatch-model.md
merge_strategy: new
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: cli-workflow-chain
    claim: cli-workflow-chain
    coverage: full
    rationale: "This dispatch logic TD supports workflow root command routing and executor resolution."
---

# Score Agent Rename Hook Dispatch

## Overview
<!-- type: overview lang: markdown -->

Renames all `sdd-*` Claude Code agent definitions to `score-*` and adds a `SubagentStop` hook that auto-injects next-step context after any `score-*` agent completes.

| Aspect | Before | After |
|--------|--------|-------|
| Agent prefix | `sdd-*` | `score-*` |
| Hook scripts | `sdd-*.sh` | `score-*.sh` |
| SubagentStop hook | none | `score-next-step.sh` (all `score-*` agents) |
| Mainthread diagnostics | manual `score run-change` calls | auto-injected via SubagentStop hook |

**Evolution note (three-role contract)**: The SubagentStop hook is superseded by `score-validate-advance.sh`, which runs `score workflow validate` and acts as the phase gatekeeper — see `projects/agentic-workflow/specs/three-role-contract.md`. `score-next-step.sh` was observation-only; the new hook gates phase advancement on artifact validation pass/fail.

The `score` brand is the CLI brand; all agent definitions, hook scripts, and routing strings in `projects/agentic-workflow/src/workflow/mod.rs` align to that brand.
## Requirements
<!-- type: doc lang: markdown -->

### R1: Rename agent definition files

Rename all `.claude/agents/sdd-*.md` files to `score-*.md`:
- `sdd-change-implementation.md` → `score-change-implementation.md`
- `sdd-change-spec.md` → `score-change-spec.md`
- `sdd-reference-context.md` → `score-reference-context.md`
- `sdd-review.md` → `score-review.md`
- `sdd-issue-author.md` → `score-issue-author.md`

Update `name:` frontmatter field in each file accordingly.

**Priority**: high

### R2: Rename hook scripts

Rename `.claude/hooks/sdd-safe-bash.sh` → `score-safe-bash.sh` and `.claude/hooks/sdd-readonly-bash.sh` → `score-readonly-bash.sh`. Update all internal comments and references.

**Priority**: high

### R3: Update agent definitions to reference renamed hook scripts

Update the `command:` field in each agent's `PreToolUse` hook from `sdd-safe-bash.sh` / `sdd-readonly-bash.sh` to `score-safe-bash.sh` / `score-readonly-bash.sh`.

**Priority**: high

### R4: Update workflow/mod.rs agent name strings

In `projects/agentic-workflow/src/workflow/mod.rs`, update the `default_agent` injection match block: replace all `"sdd-reference-context"`, `"sdd-change-spec"`, `"sdd-change-implementation"`, `"sdd-review"` strings to their `score-*` equivalents.

**Priority**: high

### R5: Update dispatch-model.md spec

In `projects/agentic-workflow/tech-design/core/logic/dispatch-model.md`, update:
- Overview table: change `4 phase agents (sdd-*.md)` to `score-*.md`
- R1 Phase → Agent Mapping table: rename all `sdd-*` agent names to `score-*`
- All scenario flows that reference `sdd-*` agent names

**Priority**: high

### R6: Update CLAUDE.md agent references

In `CLAUDE.md`, update all references to `sdd-*` agent names to `score-*`.

**Priority**: high

### R7: Add SubagentStop hook in settings.json

Add a `SubagentStop` hook entry in `.claude/settings.json` that matches all `score-*` agent names and runs `.claude/hooks/score-next-step.sh`.

**Priority**: high

### R8: Create score-next-step.sh hook script

Create `.claude/hooks/score-next-step.sh` that:
1. Reads the agent name from stdin (`jq -r '.agent_name'`)
2. Extracts the change-id from recent git commits or active changes
3. Runs `score run-change --change-id <id>` to get next phase info
4. Outputs `{"additionalContext": "..."}` to stdout with the next-step JSON
5. Always exits 0 (observation-only — never blocks)

**Priority**: high
## Scenarios
<!-- type: doc lang: markdown -->

### S1: Agent invoked after rename

- **GIVEN** `.claude/agents/score-change-implementation.md` exists (renamed from `sdd-change-implementation.md`)
- **WHEN** mainthread dispatches a subagent with type `score-change-implementation`
- **THEN** Claude Code resolves the correct agent definition
- **AND** the `PreToolUse` hook references `score-safe-bash.sh`

### S2: SubagentStop hook fires after score-* agent completes

- **GIVEN** `score-next-step.sh` is registered as SubagentStop hook matching `score-*`
- **WHEN** any `score-*` subagent finishes its work
- **THEN** the hook script runs `score run-change` and emits `additionalContext` with next phase info
- **AND** mainthread receives this context in its next turn without running diagnostics manually
- **AND** the hook exits 0 (non-blocking)

### S3: Hook script handles missing change-id gracefully

- **GIVEN** the SubagentStop hook fires but no active change-id can be determined
- **WHEN** `score-next-step.sh` runs
- **THEN** the script emits an empty or minimal `additionalContext` and exits 0
- **AND** mainthread flow is not interrupted
## Diagrams
<!-- type: doc lang: markdown -->

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- score-td-placeholder -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- score-td-placeholder -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- score-td-placeholder -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- score-td-placeholder -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- score-td-placeholder -->

## API Spec
<!-- type: doc lang: markdown -->

### REST API
<!-- type: rest-api lang: yaml -->
<!-- score-td-placeholder -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- score-td-placeholder -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- score-td-placeholder -->

### CLI
<!-- type: cli lang: yaml -->
<!-- score-td-placeholder -->

### Schema
<!-- type: schema lang: json -->
<!-- score-td-placeholder -->

### Config
<!-- type: config lang: json -->
<!-- score-td-placeholder -->

## Test Plan
<!-- type: doc lang: markdown -->

<!-- TODO -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  # Agent definition renames
  - path: .claude/agents/sdd-change-implementation.md
    action: delete
    section: logic
    impl_mode: codegen
  - path: .claude/agents/score-change-implementation.md
    action: create
    section: cli
    impl_mode: codegen
    desc: Renamed from sdd-change-implementation.md, name field updated
  - path: .claude/agents/sdd-change-spec.md
    action: delete
    section: schema
    impl_mode: codegen
  - path: .claude/agents/score-change-spec.md
    action: create
    section: rpc-api
    impl_mode: codegen
    desc: Renamed from sdd-change-spec.md, name field updated
  - path: .claude/agents/sdd-reference-context.md
    action: delete
    section: rest-api
    impl_mode: codegen
  - path: .claude/agents/score-reference-context.md
    action: create
    section: async-api
    impl_mode: codegen
    desc: Renamed from sdd-reference-context.md, name field updated
  - path: .claude/agents/sdd-review.md
    action: delete
    section: config
    impl_mode: codegen
  - path: .claude/agents/score-review.md
    action: create
    section: state-machine
    impl_mode: codegen
    desc: Renamed from sdd-review.md, name field updated
  - path: .claude/agents/sdd-issue-author.md
    action: delete
    section: dependency
    impl_mode: codegen
  - path: .claude/agents/score-issue-author.md
    action: create
    section: db-model
    impl_mode: codegen
    desc: Renamed from sdd-issue-author.md, name field updated

  # Hook script renames
  - path: .claude/hooks/sdd-safe-bash.sh
    action: delete
    section: interaction
    impl_mode: codegen
  - path: .claude/hooks/score-safe-bash.sh
    action: create
    section: component
    impl_mode: codegen
    desc: Renamed from sdd-safe-bash.sh
  - path: .claude/hooks/sdd-readonly-bash.sh
    action: delete
    section: wireframe
    impl_mode: codegen
  - path: .claude/hooks/score-readonly-bash.sh
    action: create
    section: design-token
    impl_mode: codegen
    desc: Renamed from sdd-readonly-bash.sh

  # New SubagentStop hook script
  - path: .claude/hooks/score-next-step.sh
    action: create
    section: doc
    impl_mode: codegen
    desc: SubagentStop hook — runs score run-change, injects additionalContext, exits 0

  # Settings update
  - path: .claude/settings.json
    action: modify
    section: overview
    impl_mode: codegen
    desc: Add SubagentStop hook for score-* agents; update PreToolUse hook paths to score-*.sh

  # Rust source update
  - path: projects/agentic-workflow/src/workflow/mod.rs
    section: source
    action: modify
    impl_mode: codegen
    desc: Update default_agent strings from sdd-* to score-*

  # CLAUDE.md update
  - path: CLAUDE.md
    action: modify
    section: logic
    impl_mode: codegen
    desc: Update agent name references from sdd-* to score-*

  # Spec update
  - path: projects/agentic-workflow/tech-design/core/logic/dispatch-model.md
    action: modify
    section: cli
    impl_mode: codegen
    desc: Update all sdd-* agent name references to score-* in overview, R1 table, scenarios
```
## Wireframe
<!-- type: wireframe lang: yaml -->

```yaml
wireframes: []
```

## Component
<!-- type: component lang: yaml -->

```yaml
components: []
```

## Design Token
<!-- type: design-token lang: yaml -->

```yaml
tokens: []
```

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

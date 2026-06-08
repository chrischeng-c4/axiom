---
id: score-agent-rename-hook-dispatch
main_spec_ref: crates/sdd/logic/dispatch-model.md
merge_strategy: new
fill_sections: [overview, requirements, scenarios, changes]
filled_sections: [overview, requirements, scenarios, changes]
create_complete: true
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

The `score` brand is the CLI brand; all agent definitions, hook scripts, and routing strings in `crates/sdd/src/workflow/mod.rs` align to that brand.
## Requirements

<!-- type: requirements lang: markdown -->

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

In `crates/sdd/src/workflow/mod.rs`, update the `default_agent` injection match block: replace all `"sdd-reference-context"`, `"sdd-change-spec"`, `"sdd-change-implementation"`, `"sdd-review"` strings to their `score-*` equivalents.

**Priority**: high

### R5: Update dispatch-model.md spec

In `.score/tech_design/crates/sdd/logic/dispatch-model.md`, update:
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

<!-- type: scenarios lang: markdown -->

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

<!-- TODO -->

## Changes

<!-- type: changes lang: yaml -->

```yaml
files:
  # Agent definition renames
  - path: .claude/agents/sdd-change-implementation.md
    action: DELETE
  - path: .claude/agents/score-change-implementation.md
    action: CREATE
    desc: Renamed from sdd-change-implementation.md, name field updated
  - path: .claude/agents/sdd-change-spec.md
    action: DELETE
  - path: .claude/agents/score-change-spec.md
    action: CREATE
    desc: Renamed from sdd-change-spec.md, name field updated
  - path: .claude/agents/sdd-reference-context.md
    action: DELETE
  - path: .claude/agents/score-reference-context.md
    action: CREATE
    desc: Renamed from sdd-reference-context.md, name field updated
  - path: .claude/agents/sdd-review.md
    action: DELETE
  - path: .claude/agents/score-review.md
    action: CREATE
    desc: Renamed from sdd-review.md, name field updated
  - path: .claude/agents/sdd-issue-author.md
    action: DELETE
  - path: .claude/agents/score-issue-author.md
    action: CREATE
    desc: Renamed from sdd-issue-author.md, name field updated

  # Hook script renames
  - path: .claude/hooks/sdd-safe-bash.sh
    action: DELETE
  - path: .claude/hooks/score-safe-bash.sh
    action: CREATE
    desc: Renamed from sdd-safe-bash.sh
  - path: .claude/hooks/sdd-readonly-bash.sh
    action: DELETE
  - path: .claude/hooks/score-readonly-bash.sh
    action: CREATE
    desc: Renamed from sdd-readonly-bash.sh

  # New SubagentStop hook script
  - path: .claude/hooks/score-next-step.sh
    action: CREATE
    desc: SubagentStop hook — runs score run-change, injects additionalContext, exits 0

  # Settings update
  - path: .claude/settings.json
    action: MODIFY
    desc: Add SubagentStop hook for score-* agents; update PreToolUse hook paths to score-*.sh

  # Rust source update
  - path: crates/sdd/src/workflow/mod.rs
    action: MODIFY
    desc: Update default_agent strings from sdd-* to score-*

  # CLAUDE.md update
  - path: CLAUDE.md
    action: MODIFY
    desc: Update agent name references from sdd-* to score-*

  # Spec update
  - path: .score/tech_design/crates/sdd/logic/dispatch-model.md
    action: MODIFY
    desc: Update all sdd-* agent name references to score-* in overview, R1 table, scenarios
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

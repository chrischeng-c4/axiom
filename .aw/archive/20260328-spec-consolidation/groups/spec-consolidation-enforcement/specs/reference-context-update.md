---
id: reference-context-update
type: spec
title: "Reference Context — Logic"
version: 4
files:
  - tools/common_reference_context.rs
  - tools/create_reference_context.rs
  - tools/review_reference_context.rs
  - tools/revise_reference_context.rs
main_spec_ref: crates/cclab-sdd/logic/reference-context.md
merge_strategy: extend
fill_sections: [changes]
filled_sections: [changes]
create_complete: true
---

# Reference Context

## Phase Transition

```yaml
from: PreClarificationsCreated
to: PostClarificationsCreated  # skips ReferenceContext* intermediate phases
executor: [mainthread]
crr: true  # per-group CRR cycle
progress_key: groups_progress.reference_context
max_revisions: 1  # auto-approve on exceed
```

## Central Router

`sdd_workflow_create_reference_context` is a **central router** that manages per-group CRR lifecycle. Each call:

1. Determines next unfinished group
2. Reads group's sub-state (Create/Review/Revise/AllDone)
3. Returns `next_actions` pointing to the appropriate tool

```mermaid
flowchart TD
    Start([workflow_create_reference_context]) --> ListGroups[list groups/ dirs]
    ListGroups --> ReadProgress[read groups_progress.reference_context]
    ReadProgress --> Remaining{remaining groups?}
    Remaining -->|none| AdvancePhase[advance to PostClarificationsCreated]
    Remaining -->|exists| GetSubState[resolve sub-state for remaining#91;0#93;]
    GetSubState --> SubSwitch{sub-state?}
    SubSwitch -->|Create| RouteCreate[→ workflow_create_reference_context prompt]
    SubSwitch -->|Review| RouteReview[→ workflow_review_reference_context]
    SubSwitch -->|Revise| RouteRevise[→ workflow_revise_reference_context]
```

## Sub-State Enum

```yaml
GroupSubState:
  Create: "No reference_context.md — needs creation"
  Review: "reference_context.md exists, no review section — needs review"
  Revise: "Reviewed with issues, revision_count < max — needs revision"
  AllDone: "APPROVED or revision_count >= max — mark group done"
```

## Artifact Writing Enforcement

Agents must write artifacts via CLI (`cclab sdd artifact create-reference-context`), not by directly writing files. The system enforces this with a two-layer approach:

### Layer 1: Prompt Constraint

All agent prompts include:

```
## CRITICAL: Artifact Writing Rule

DO NOT use Write or Edit tools to create/modify artifact files directly.
You MUST use the artifact CLI command. Direct file writes will be REJECTED.
```

### Layer 2: Retry Loop + Mainthread Fallback

Handled by `build_workflow_response()` in `workflow_common.rs`. See [Agent Retry Mechanism](#agent-retry-mechanism) for the full flow.

On each agent attempt, `verify_artifact_cli_called()` checks for the `written_by: artifact_cli` marker. If absent after max retries, `try_mainthread_fallback()` extracts spec references from the agent's prose and calls `execute_artifact()` internally.

## Prompt Templates

### Create

```markdown
# Task: Build Reference Context for group '{{group_id}}'

Explore the codebase and specs to identify relevant references for this group.

## Steps

1. Read: `groups/{{group_id}}/pre_clarifications.md`
2. Read: `user_input.md`
3. Explore:
   - Search `cclab/specs/` for related specs
   - Search `cclab/knowledge/` for relevant docs
4. For each relevant spec/doc, assess relevance (high/medium/low)
5. Write payload JSON, then run:
   `cclab sdd artifact create-reference-context {{change_id}} <payload_path>`

## Output: specs array

Each spec reference must include:
- spec_id: path relative to cclab/specs/
- spec_group: logical grouping (e.g. "mcp-tools", "state-machine")
- relevance: high | medium | low
- key_requirements: array of relevant requirement summaries

## Output: spec_plan array

For each change spec that will be created:
- spec_id: identifier for the new change spec
- action: "modify" (copy existing) or "create" (new skeleton)
- main_spec_ref: target path in cclab/specs/ (REQUIRED)
- source: path of existing spec to copy (only for "modify")
- sections: array of section types this spec needs (see change-spec.md § Section Selection)
```

### Review

```markdown
# Task: Review Reference Context for group '{{group_id}}'

## Checklist

- [ ] Coverage: all relevant specs identified (no major gaps)
- [ ] Relevance scores: high/medium/low correctly assigned
- [ ] Key requirements: accurately summarize what matters
- [ ] No false positives: irrelevant specs not included
- [ ] Completeness: knowledge docs and code references included
- [ ] spec_plan: every entry has main_spec_ref set (not null)
- [ ] spec_plan: sections are reasonable for the requirements
- [ ] spec_plan: modify entries have valid source paths

## Verdict

- APPROVED: checklist passes → mark group done
- REVIEWED: issues found, revision_count < 1 → route to revise
- Auto-approve: revision_count >= 1 → mark done regardless
```

### Revise

```markdown
# Task: Revise Reference Context for group '{{group_id}}'

Read review feedback and update reference context.

1. Read current: `groups/{{group_id}}/reference_context.md`
2. Address each review issue
3. Write corrected payload JSON, then run:
   `cclab sdd artifact revise-reference-context {{change_id}} <payload_path>`
```

## CRR Cycle

```mermaid
flowchart LR
    Create --> Review
    Review -->|APPROVED| Done[mark group done]
    Review -->|REVIEWED & rev<1| Revise
    Revise -->|rev>=1| Done
    Revise -->|rev<1| Review
```

Max 1 revision per group. Auto-approve on exceed.

## Artifact Schema

### specs array (create + revise input)

```json
{
  "type": "array",
  "minItems": 1,
  "items": {
    "type": "object",
    "required": ["spec_id", "spec_group", "relevance"],
    "properties": {
      "spec_id": { "type": "string" },
      "spec_group": { "type": "string" },
      "relevance": { "type": "string", "enum": ["high", "medium", "low"] },
      "key_requirements": { "type": "array", "items": { "type": "string" } }
    }
  }
}
```

### spec_plan array (create input)

Determines which change specs will be created, where they merge to, and which sections each spec needs.

```json
{
  "type": "array",
  "minItems": 1,
  "items": {
    "type": "object",
    "required": ["spec_id", "action", "main_spec_ref", "sections"],
    "properties": {
      "spec_id": { "type": "string", "description": "Change spec identifier" },
      "action": { "type": "string", "enum": ["modify", "create"] },
      "main_spec_ref": { "type": "string", "description": "Target path in cclab/specs/ (REQUIRED)" },
      "source": { "type": "string", "description": "Existing spec to copy (only for modify)" },
      "sections": {
        "type": "array",
        "items": { "type": "string", "enum": ["overview","rest-api","rpc-api","async-api","cli","schema","logic","interaction","state-machine","db-model","test-plan","dependency","wireframe","component","design-token","config","changes"] },
        "description": "Section types this spec needs. Determined by rule engine + agent input."
      }
    }
  }
}
```

**Section selection**: CLI rule engine matches requirements text against keyword patterns to suggest sections (see `change-spec.md` § Section Selection). Agent may adjust during reference_context creation. Review CRR catches gaps.

After reference_context is approved, the system uses `spec_plan` to **prepare spec files**:
- `action: modify` → copy `cclab/specs/{source}` to `groups/{group}/specs/{spec_id}.md`, set `main_spec_ref`
- `action: create` → write skeleton with `<!-- TODO -->` for each section in `sections`, set `main_spec_ref`

This ensures every spec has `main_spec_ref` and `sections` set before change_spec phase begins.

### review params

```json
{
  "verdict": { "type": "string", "enum": ["APPROVED", "REVIEWED"] },
  "summary": { "type": "string" },
  "checklist_results": {
    "type": "array",
    "items": {
      "type": "object",
      "required": ["item", "passed"],
      "properties": {
        "item": { "type": "string" },
        "passed": { "type": "boolean" },
        "note": { "type": "string" }
      }
    }
  },
  "issues": {
    "type": "array",
    "items": {
      "type": "object",
      "required": ["severity", "description"],
      "properties": {
        "severity": { "type": "string", "enum": ["HIGH", "MEDIUM", "LOW"] },
        "description": { "type": "string" },
        "recommendation": { "type": "string" }
      }
    }
  }
}
```

## Agent Retry Mechanism

When an agent completes but doesn't call the artifact CLI, the workflow layer retries before falling back to mainthread.

```yaml
MAX_AGENT_RETRIES: 2  # 3 total attempts (initial + 2 retries)
```

```mermaid
flowchart TD
    Start([build_workflow_response]) --> Loop[for agent in executor_chain]
    Loop --> Attempt[for attempt in 0..MAX_RETRIES]
    Attempt --> RunAgent[run_agent]
    RunAgent -->|error| NextAgent[try next agent]
    RunAgent -->|ok| Verify{verify_artifact_cli_called?}
    Verify -->|yes| RevCheck{revise action?}
    RevCheck -->|yes| IncRev[increment revision_count]
    RevCheck -->|no| Success[return ok]
    IncRev --> Success
    Verify -->|no, attempts left| Retry[log retry, continue]
    Retry --> Attempt
    Verify -->|no, max reached| Fallback{try_mainthread_fallback?}
    Fallback -->|ok| Success
    Fallback -->|fail| NextAgent
    NextAgent --> Loop
    Loop -->|all exhausted| MainThread[return executor=mainthread]
```

### Verification Function

`verify_artifact_cli_called(action, change_dir, extra_fields) -> bool`:

| Action pattern | Checks for |
|---------------|------------|
| `create_reference_context`, `revise_reference_context` | `written_by: artifact_cli` marker in group's `reference_context.md` |
| `review_reference_context` | `review_verdict:` presence in frontmatter |
| Other actions | Returns `true` (no verification, preserves existing behavior) |

### Mainthread Fallback

`try_mainthread_fallback(action, change_dir, extra_fields, project_root) -> bool`:

Only applies to `create_reference_context` and `revise_reference_context`. Reads agent's prose file, extracts spec references via `extract_specs_from_prose()`, and calls `execute_artifact()` internally.

## Side Effects

| Action | STATE.yaml change | Owner |
|--------|-------------------|-------|
| Create artifact | write `reference_context.md` only | artifact CLI |
| Review artifact (APPROVED) | Appends group_id to `groups_progress.reference_context` | workflow layer |
| Review artifact (auto-approve) | Same as APPROVED | workflow layer |
| Revise artifact | write `reference_context.md` only | artifact CLI |
| Revise workflow (post-agent) | `revision_counts.{key} += 1` | workflow layer |
| All groups done | `phase → PostClarificationsCreated` | workflow layer |


## Changes

### Update: Create Prompt Template

Replace the `### Create` block with an updated version containing two additions.

**Addition 1 — `## Existing Spec Structure` block.**

Insert immediately before `## Steps`:

```
## Existing Spec Structure

The following ASCII tree shows existing spec directories for the affected crate(s). Use this to plan spec_plan entries — prefer modifying existing files over creating new ones.

{{spec_dir_tree}}
```

`{{spec_dir_tree}}` is a template variable substituted by `build_create_prompt` with an ASCII tree (same format as the `tree` CLI tool) generated by `build_spec_dir_tree` in `workflow/scope.rs`:

```yaml
function: build_spec_dir_tree
module: workflow/scope.rs
signature: >
  pub fn build_spec_dir_tree(spec_groups: &[String], project_root: &Path, config: Option<&SddConfig>) -> String
behavior:
  - For each group, call resolve_spec_dir to get the spec root path
  - Walk directory recursively
  - Render ASCII tree: ├── / └── / │  prefix (tree CLI format)
  - Return empty string if no spec directories found
```

**Addition 2 — spec_plan guidance.**

In `## Output: spec_plan array`, replace the `main_spec_ref` bullet and add rules after the `sections` bullet:

```
- main_spec_ref: target path in cclab/specs/ (REQUIRED — must include a named subfolder,
  e.g. `crates/cclab-sdd/logic/foo.md`, not `crates/cclab-sdd/foo.md`)

**Action preference**: Use `action: modify` for any file visible in the spec directory tree
above. Reserve `action: create` for genuinely new subsystems with no existing spec file.
```

### Update: Review Prompt Checklist

Add one item after `spec_plan: sections are reasonable for the requirements`:

```
- [ ] spec_plan: main_spec_ref paths include a subfolder (not root-level under crate)
```

### Add: Spec_plan Path Validation

New section `## Spec_plan Path Validation` to be inserted after `## Artifact Schema`.

`prepare_specs_from_plan` validates each `spec_plan` entry before preparing files. Validation is a hard error — spec preparation is aborted if any entry fails.

| Check | Rule | Failure Mode |
|-------|------|--------------|
| Subfolder required | `main_spec_ref` must have ≥ 4 path components separated by `/` (e.g. `crates/cclab-sdd/logic/foo.md`). Root-level crate paths such as `crates/cclab-sdd/foo.md` are rejected. | Hard error — spec preparation aborted, no files written |

### Update: spec_plan Schema — `main_spec_ref` Description

In the `### spec_plan array` JSON Schema block, update the `main_spec_ref` property description:

| Before | After |
|--------|-------|
| `"description": "Target path in cclab/specs/ (REQUIRED)"` | `"description": "Target path in cclab/specs/ (REQUIRED — must reside in a named subfolder, min 4 path components: {category}/{crate}/{subdir}/{file}.md)"` |

# Reviews

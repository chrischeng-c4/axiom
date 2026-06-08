---
id: refactor-eliminate-state-yaml-user-input-md-groups-nesting-spec
main_spec_ref: crates/sdd/logic/state-machine.md
merge_strategy: append
create_complete: true
---

# Refactor Eliminate State Yaml User Input Md Groups Nesting Spec

## Overview
<!-- type: overview lang: markdown -->

Eliminate the vestigial `STATE.yaml` store and enforce the invariant that `change_id == issue_slug` across the SDD toolchain. Today `init_change` unconditionally treats `change_id` as the issue slug and, when that mismatches reality, silently falls back to writing `STATE.yaml` — producing changes with no title, no phase in issue frontmatter, and invisible workflow state. This change (a) rejects mismatched `change_id`/`issue_slug` pairs at the `init_change` boundary, (b) removes the `save_yaml_fallback` and `load_yaml` paths so sync-to-issue failures propagate as hard errors, and (c) deletes the dead `user_input.md` and `groups/{gid}/` nesting from the change scaffold. After the change, issue frontmatter is the single source of workflow truth; `meta.yaml` remains only for per-iteration operational data (checksums, telemetry, delegation guard).

## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: requirements
---
requirementDiagram

requirement R1 {
  id: R1
  text: "init_change rejects requests where resolved issue_slug differs from change_id with error 'change_id <x> does not match resolved issue slug <y>. change_id must equal the issue slug (one issue = one change).'"
  risk: high
  verifymethod: test
}

requirement R2 {
  id: R2
  text: "score run-change and score workflow <action> <change_id> refuse to operate when .score/issues/{open,closed}/<change_id>.md does not exist, returning the same change_id-must-equal-issue-slug error."
  risk: high
  verifymethod: test
}

requirement R3 {
  id: R3
  text: "CLI entry points that accept both --change-id and --issue are deprecated. change_id is auto-derived from issue slug; the --change-id override is removed."
  risk: medium
  verifymethod: inspection
}

requirement R4 {
  id: R4
  text: "StateManager::save() removes the STATE.yaml fallback. If sync_to_issue() returns Err, the error bubbles up unchanged. Non-issue code paths must migrate to an explicit alternate API."
  risk: high
  verifymethod: test
}

requirement R5 {
  id: R5
  text: "StateManager::load() removes the STATE.yaml read path. Only meta.yaml plus issue frontmatter are read. Legacy change directories containing STATE.yaml return Err with message 'STATE.yaml is deprecated. Migrate via <migration command>.'"
  risk: high
  verifymethod: test
}

requirement R6 {
  id: R6
  text: "The save_yaml_fallback branch in manager.rs:165-174 and the State::load_yaml path in legacy readers are deleted from source."
  risk: low
  verifymethod: inspection
}

requirement R7 {
  id: R7
  text: "user_input.md generator is removed and associated unused test fixtures are deleted."
  risk: low
  verifymethod: inspection
}

requirement R8 {
  id: R8
  text: "create_change_internal no longer creates groups/{gid}/ subtrees. Specs, prompts, and payloads live flat at .score/changes/{id}/{specs,prompts,payloads}/."
  risk: medium
  verifymethod: test
}

requirement R9 {
  id: R9
  text: "check_branch_uniqueness (init_change.rs:411-444) is rewritten to scan .score/worktrees/ plus issue frontmatter instead of STATE.yaml."
  risk: medium
  verifymethod: test
}

requirement R10 {
  id: R10
  text: "meta.yaml continues to hold per-iteration operational data — checksums, validations, telemetry, delegation_guard. These fields are NOT migrated to issue frontmatter."
  risk: low
  verifymethod: inspection
}

requirement R11 {
  id: R11
  text: "meta.yaml is written only when operational data is non-empty — current manager.rs:182-191 conditional write behavior is preserved."
  risk: low
  verifymethod: test
}

requirement R12 {
  id: R12
  text: "One-time migration CLI 'score changes migrate-legacy' walks .score/changes/*/STATE.yaml, copies workflow fields to the corresponding issue frontmatter, writes meta.yaml, then deletes STATE.yaml, user_input.md, and groups/. No-op when no legacy dirs exist. Out-of-scope fallback: document manual migration steps in issue-centric-workflow.md."
  risk: medium
  verifymethod: test
}

R2 - derive -> R1
R4 - derive -> R1
R5 - derive -> R4
R6 - derive -> R4
R9 - derive -> R1
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
- id: S1
  title: Mismatched change_id is rejected at init_change
  given: >-
    Issue 'enhancement-real-module-import-system' exists at
    .score/issues/open/enhancement-real-module-import-system.md with state: open.
    No worktree or change directory exists for either identifier.
  when: >-
    Caller invokes `score workflow init-change feat-mamba-import-system`
    with payload `{"description":"issue:enhancement-real-module-import-system"}`
    (change_id != resolved issue_slug).
  then: >-
    init_change returns Err with message
    "change_id 'feat-mamba-import-system' does not match resolved issue slug
    'enhancement-real-module-import-system'. change_id must equal the issue slug
    (one issue = one change). Re-run with change_id='enhancement-real-module-import-system'
    or fix the issue reference."
    No worktree is created under .score/worktrees/.
    No directory is created under .score/changes/.
    No mutation occurs on the issue frontmatter.
  verifies: [R1, R2]

- id: S2
  title: Matched change_id creates a worktree without STATE.yaml
  given: >-
    Issue 'refactor-eliminate-state-yaml-user-input-md-groups-nesting' exists at
    .score/issues/open/refactor-eliminate-state-yaml-user-input-md-groups-nesting.md
    with state: open, no phase/change_id/branch in frontmatter.
  when: >-
    Caller invokes `score workflow init-change refactor-eliminate-state-yaml-user-input-md-groups-nesting`
    with payload `{"description":"issue:refactor-eliminate-state-yaml-user-input-md-groups-nesting"}`.
  then: >-
    Worktree is created at
    .score/worktrees/refactor-eliminate-state-yaml-user-input-md-groups-nesting/
    on branch cclab/refactor-eliminate-state-yaml-user-input-md-groups-nesting.
    Issue frontmatter (inside worktree) is updated with
    phase: change_inited,
    change_id: refactor-eliminate-state-yaml-user-input-md-groups-nesting,
    branch: cclab/refactor-eliminate-state-yaml-user-input-md-groups-nesting.
    `find .score -name STATE.yaml` returns empty inside the worktree.
    No .score/changes/<id>/user_input.md and no .score/changes/<id>/groups/ subtree
    exists.
  verifies: [R1, R7, R8]

- id: S3
  title: sync_to_issue failure bubbles up with no STATE.yaml fallback
  given: >-
    A change directory .score/changes/<id>/ exists with a StateManager in memory.
    The issue file at .score/issues/open/<id>.md has been externally deleted or
    made unwritable so that backend.update(<id>, patch) returns Err(NotFound) or
    Err(PermissionDenied).
  when: >-
    Caller invokes StateManager::save() to persist a phase transition.
  then: >-
    save() returns Err propagated from sync_to_issue() unchanged.
    No STATE.yaml is written at .score/changes/<id>/STATE.yaml.
    meta.yaml is written only if operational data is non-empty (R11 preserved).
    The caller observes the underlying backend error, not a "wrote fallback" success.
  verifies: [R4, R5, R6, R11]
```

<!-- Diagrams, API Spec, Wireframe, Component, Design Token, Doc sections omitted — not applicable to this refactor (no UI, no new API, no public docs). -->

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: test-plan
---
requirementDiagram

element T1 {
  type: "Test"
}

element T2 {
  type: "Test"
}

element T3 {
  type: "Test"
}

element T4 {
  type: "Test"
}

element T5 {
  type: "Test"
}

element T6 {
  type: "Test"
}

T1 - verifies -> R1
T2 - verifies -> R2
T3 - verifies -> R4
T4 - verifies -> R5
T5 - verifies -> R8
T6 - verifies -> R9
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
- path: crates/sdd/src/tools/init_change.rs
  action: modify
  satisfies: [R1, R2, R3, R8, R9]
  summary: >-
    Resolve issue_slug from description or --issue flag, compare against
    change_id, return structured Err on mismatch. Remove --change-id override.
    Rewrite check_branch_uniqueness to scan .score/worktrees/ and issue
    frontmatter. Stop creating groups/{gid}/ subtree in create_change_internal.

- path: crates/sdd/src/state/manager.rs
  action: modify
  satisfies: [R4, R5, R6, R10, R11]
  summary: >-
    Delete save_yaml_fallback branch (lines ~165-174) and State::load_yaml
    legacy read path. save() bubbles sync_to_issue() Err unchanged. load()
    reads only meta.yaml plus issue frontmatter; emits deprecation Err when
    STATE.yaml detected. Preserve conditional meta.yaml write (empty → skip).

- path: crates/sdd/src/models/state.rs
  action: modify
  satisfies: [R4, R10]
  summary: >-
    Slim State struct — drop fields that now live in issue frontmatter
    (phase, change_id, branch, iteration). Retain only operational fields
    persisted in meta.yaml (checksums, validations, telemetry, delegation_guard).

- path: crates/sdd/src/tools/create_change_merge.rs
  action: modify
  satisfies: [R4, R8]
  summary: >-
    Align merge flow with new storage — write phase: change_archived and
    state: closed to issue frontmatter only. Remove any STATE.yaml teardown
    that assumes the file exists.

- path: projects/score/cli/src/list.rs
  action: verify
  satisfies: [R1]
  summary: >-
    No code change expected — confirm the scan path (per list-command spec
    line 407) correctly resolves `<change-id>.md` now that the invariant
    holds. Add/adjust regression test if behavior drift is observed.

- path: .score/tech_design/crates/sdd/logic/issue-centric-workflow.md
  action: modify
  satisfies: [R4, R5, R10, R12]
  summary: >-
    Rewrite Storage Model + Phase Storage sections (lines ~309-315): remove
    "dual-write" language, state issue frontmatter as single source for
    workflow fields, meta.yaml for per-iteration operational data only.
    Document manual migration steps if R12 CLI is deferred.

- path: .score/tech_design/crates/sdd/logic/state-machine.md
  action: modify
  satisfies: [R4, R10]
  summary: >-
    Update state-fields section — mark workflow fields (phase, change_id,
    branch, iteration) as stored in issue frontmatter. Append a Storage Model
    subsection describing the single-writer contract.
```

# Reviews

---
id: issue-merge-target
fill_sections: [schema, logic, cli, test-plan, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# Issue Merge Target

## Schema
<!-- type: schema lang: yaml -->

```yaml
"$schema": "https://json-schema.org/draft/2020-12/schema"
"$id": "issue-merge-target"
title: IssueMergeTargetFields
description: |
  Extension fields added to the Issue frontmatter schema to support
  per-issue merge target branch resolution. These fields are optional;
  absence preserves backward compatibility with all existing issues.
type: object
properties:
  target_branch:
    type: string
    minLength: 1
    description: |
      Target branch for git merge within the current checkout root. This is a
      branch-selection field only; it does not change the filesystem root chosen
      by find_project_root() and must not redirect linked worktrees to another
      checkout. When present, this value overrides any
      config-level default. When absent, resolution falls through to the
      current branch detection and then to [agentic_workflow.repo_platform].default_branch
      in .aw/config.toml (which itself defaults to "main").
    examples:
      - main
      - develop
      - release/2.0
```
## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: resolve-merge-target
entry: start
nodes:
  start: { kind: start, label: "resolve_merge_target(issue)" }
  check_frontmatter: { kind: decision, label: "issue.target_branch present?" }
  return_frontmatter: { kind: terminal, label: "use issue.target_branch" }
  check_override: { kind: decision, label: "CLI --target-branch flag set?" }
  return_override: { kind: terminal, label: "use CLI override" }
  detect_branch: { kind: process, label: "git rev-parse --abbrev-ref HEAD" }
  check_attached: { kind: decision, label: "branch != HEAD (not detached)?" }
  read_config: { kind: process, label: "read .aw/config.toml default_branch" }
  check_config: { kind: decision, label: "default_branch present?" }
  validate_exists: { kind: process, label: "git rev-parse branch (branch-exists check)" }
  check_valid: { kind: decision, label: "branch exists locally?" }
  return_resolved: { kind: terminal, label: "emit action:error — branch not found" }
  return_ok: { kind: terminal, label: "return resolved branch" }
  error_no_target: { kind: terminal, label: "emit action:error — cannot determine merge target" }
edges:
  - from: start
    to: check_override
  - from: check_override
    to: return_override
    label: "yes"
  - from: check_override
    to: check_frontmatter
    label: "no"
  - from: check_frontmatter
    to: return_frontmatter
    label: "yes"
  - from: check_frontmatter
    to: detect_branch
    label: "no"
  - from: detect_branch
    to: check_attached
  - from: check_attached
    to: read_config
    label: "no (detached HEAD)"
  - from: check_attached
    to: validate_exists
    label: "yes"
  - from: read_config
    to: check_config
  - from: check_config
    to: validate_exists
    label: "yes"
  - from: check_config
    to: error_no_target
    label: "no"
  - from: return_override
    to: return_ok
  - from: return_frontmatter
    to: validate_exists
  - from: validate_exists
    to: check_valid
  - from: check_valid
    to: return_ok
    label: "yes"
  - from: check_valid
    to: return_resolved
    label: "no"
---
flowchart TD
    start([resolve_merge_target]) --> check_override{CLI --target-branch set?}
    check_override -->|yes| return_override([use CLI override])
    check_override -->|no| check_frontmatter{issue.target_branch present?}
    check_frontmatter -->|yes| return_frontmatter([use issue.target_branch])
    check_frontmatter -->|no| detect_branch[git rev-parse --abbrev-ref HEAD]
    detect_branch --> check_attached{branch != HEAD?}
    check_attached -->|no| read_config[read .aw/config.toml default_branch]
    check_attached -->|yes| validate_exists
    read_config --> check_config{default_branch present?}
    check_config -->|yes| validate_exists
    check_config -->|no| error_no_target([action:error - cannot determine merge target])
    return_override --> return_ok([return resolved branch])
    return_frontmatter --> validate_exists[git rev-parse branch]
    validate_exists --> check_valid{branch exists locally?}
    check_valid -->|yes| return_ok([return resolved branch])
    check_valid -->|no| return_resolved([action:error - branch not found])
```
## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: aw wi create
    description: Create a new issue on a dedicated worktree.
    options:
      - name: --target-branch
        type: Option<String>
        description: |
          Target branch to merge this issue's worktree into when
          `aw wi merge` is invoked. When supplied, the value is
          written to the issue frontmatter as `target_branch`. When
          omitted, no field is written (absence preserves the default
          resolution path — current branch → config default_branch →
          error). Must refer to a branch that exists in the local
          repository at creation time; absent branches produce a
          structured error envelope.
        required: false
        examples:
          - --target-branch develop
          - --target-branch release/2.0

  - name: aw wi merge
    description: Merge the issue worktree branch into the target branch.
    options:
      - name: --target-branch
        type: Option<String>
        description: |
          Explicit target branch override. When supplied, takes
          precedence over all other resolution steps (issue frontmatter,
          current branch, config default). Intended for ad-hoc overrides
          at merge time; prefer setting `target_branch` in frontmatter
          for repeatable issue-level configuration.
        required: false
        examples:
          - --target-branch main
          - --target-branch release/1.9
```
## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: issue-merge-target-test-plan
requirements:
  r1_no_hardcoded_main:
    id: R1
    text: "resolve_merge_target reads issue.target_branch and uses it as the merge target when present, never hardcoding main"
    kind: functional
    risk: high
    verify: test
  r2_frontmatter_field:
    id: R2
    text: "Issue deserialization round-trips target_branch field: present value preserved, absent field absent in output"
    kind: functional
    risk: high
    verify: test
  r3_default_backward_compat:
    id: R3
    text: "Issues without target_branch in frontmatter produce the same merge target as before (current branch or config default_branch)"
    kind: functional
    risk: high
    verify: test
  r4_create_cli_flag:
    id: R4
    text: "aw wi create --target-branch <branch> writes target_branch to frontmatter; omitting the flag writes no field"
    kind: functional
    risk: medium
    verify: test
  r5_branch_exists_validation:
    id: R5
    text: "resolve_merge_target emits action:error when the resolved branch does not exist locally, not a silent git failure"
    kind: functional
    risk: high
    verify: test
  r6_frontmatter_precedence:
    id: R6
    text: "When both issue.target_branch and config.default_branch are set, issue.target_branch wins"
    kind: functional
    risk: medium
    verify: test
elements:
  t1_frontmatter_parse_present:
    kind: test
    type: "rs/#[test]"
  t2_frontmatter_parse_absent:
    kind: test
    type: "rs/#[test]"
  t3_resolve_uses_frontmatter:
    kind: test
    type: "rs/#[test]"
  t4_resolve_falls_back_no_frontmatter:
    kind: test
    type: "rs/#[test]"
  t5_branch_missing_error:
    kind: test
    type: "rs/#[test]"
  t6_create_flag_writes_field:
    kind: test
    type: "rs/#[test]"
  t7_create_no_flag_no_field:
    kind: test
    type: "rs/#[test]"
  t8_frontmatter_beats_config:
    kind: test
    type: "rs/#[test]"
relations:
  - from: t1_frontmatter_parse_present
    verifies: r2_frontmatter_field
  - from: t2_frontmatter_parse_absent
    verifies: r2_frontmatter_field
  - from: t3_resolve_uses_frontmatter
    verifies: r1_no_hardcoded_main
  - from: t3_resolve_uses_frontmatter
    verifies: r2_frontmatter_field
  - from: t4_resolve_falls_back_no_frontmatter
    verifies: r3_default_backward_compat
  - from: t5_branch_missing_error
    verifies: r5_branch_exists_validation
  - from: t6_create_flag_writes_field
    verifies: r4_create_cli_flag
  - from: t7_create_no_flag_no_field
    verifies: r4_create_cli_flag
  - from: t8_frontmatter_beats_config
    verifies: r6_frontmatter_precedence
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "resolve_merge_target uses issue.target_branch when present"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "Issue round-trips target_branch field correctly"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "Absent target_branch preserves backward-compat default"
      risk: high
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: "aw wi create --target-branch writes field to frontmatter"
      risk: medium
      verifymethod: test
    }
    requirement R5 {
      id: R5
      text: "Missing branch emits action:error envelope"
      risk: high
      verifymethod: test
    }
    requirement R6 {
      id: R6
      text: "issue.target_branch takes precedence over config.default_branch"
      risk: medium
      verifymethod: test
    }
    element t1_frontmatter_parse_present {
      type: "rs/#[test]"
    }
    element t2_frontmatter_parse_absent {
      type: "rs/#[test]"
    }
    element t3_resolve_uses_frontmatter {
      type: "rs/#[test]"
    }
    element t4_resolve_falls_back_no_frontmatter {
      type: "rs/#[test]"
    }
    element t5_branch_missing_error {
      type: "rs/#[test]"
    }
    element t6_create_flag_writes_field {
      type: "rs/#[test]"
    }
    element t7_create_no_flag_no_field {
      type: "rs/#[test]"
    }
    element t8_frontmatter_beats_config {
      type: "rs/#[test]"
    }
    t1_frontmatter_parse_present - verifies -> R2
    t2_frontmatter_parse_absent - verifies -> R2
    t3_resolve_uses_frontmatter - verifies -> R1
    t3_resolve_uses_frontmatter - verifies -> R2
    t4_resolve_falls_back_no_frontmatter - verifies -> R3
    t5_branch_missing_error - verifies -> R5
    t6_create_flag_writes_field - verifies -> R4
    t7_create_no_flag_no_field - verifies -> R4
    t8_frontmatter_beats_config - verifies -> R6
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/types.rs
    section: source
    action: modify
    impl_mode: codegen
    description: |
      Add optional `target_branch: Option<String>` field to the `Issue`
      struct with `#[serde(default, skip_serializing_if = "Option::is_none")]`.
      Add corresponding `target_branch: Option<String>` field to `IssuePatch`.
      No migration needed — existing issues without the field deserialize
      with `target_branch: None` (backward compatible via `serde(default)`).
      The field is folded into `projects/agentic-workflow/tech-design/core/interfaces/issues/types.md#schema`
      so the primary issue type CODEGEN block remains replayable.

  - path: projects/agentic-workflow/src/cli/issues.rs
    action: modify
    section: cli
    impl_mode: hand-written
    description: |
      Add `--target-branch <branch>` flag to `CreateArgs`. In `run_create`,
      when `args.target_branch` is `Some(branch)`, validate the branch exists
      locally via `git rev-parse <branch>` before writing frontmatter; emit
      `action:error` if it does not exist. Write `target_branch` to the `Issue`
      struct only when the flag is supplied; omit the field when the flag is
      absent (R4). Update `resolve_merge_target` call in `run_merge` to pass
      `issue.target_branch.clone()` as the frontmatter override before the
      existing `args.target_branch` CLI override (R6 precedence: CLI flag
      beats frontmatter beats current branch beats config).

  - path: projects/agentic-workflow/src/cli/merge_target.rs
    action: modify
    section: cli
    impl_mode: hand-written
    description: |
      Extend `resolve_merge_target` signature to accept an additional
      `frontmatter_branch: Option<String>` parameter positioned between the
      existing CLI override and the git-detect step. Resolution order becomes:
      1. `cli_override` if Some → return verbatim (no branch-exists check —
         user is explicit).
      2. `frontmatter_branch` if Some → validate branch exists locally via
         `git rev-parse`; emit structured error if missing (R5).
      3. `git rev-parse --abbrev-ref HEAD` → if not "HEAD" → validate exists
         (R5).
      4. `.aw/config.toml` `default_branch` → validate exists (R5).
      5. Return Err (no merge target determinable).

  - path: projects/agentic-workflow/tests/merge_target_branch.rs
    action: modify
    section: test-plan
    impl_mode: hand-written
    description: |
      Add unit tests for the new resolution steps:
      - t3_resolve_uses_frontmatter: frontmatter_branch present → used
        (beats config/current branch).
      - t4_resolve_falls_back_no_frontmatter: absent frontmatter_branch →
        falls through to current branch (backward compat).
      - t5_branch_missing_error: frontmatter_branch pointing at a
        non-existent local branch → structured error, not silent git failure.
      - t8_frontmatter_beats_config: both frontmatter and config set →
        frontmatter wins.

  - path: projects/agentic-workflow/src/issues/types.rs
    section: source
    action: modify
    impl_mode: hand-written
    description: |
      Add unit tests t1_frontmatter_parse_present and t2_frontmatter_parse_absent
      directly in the types module (existing test block) validating that Issue
      serde round-trips `target_branch` correctly when present and produces no
      YAML key when absent.

  - path: projects/agentic-workflow/tests/issue_create_worktree_test.rs
    action: modify
    section: test-plan
    impl_mode: hand-written
    description: |
      Add tests t6_create_flag_writes_field and t7_create_no_flag_no_field:
      spin up a temp git repo with a `develop` branch, invoke
      `aw wi create --target-branch develop` and verify the written
      frontmatter contains `target_branch: develop`; invoke without the flag
      and verify the field is absent.
  - action: annotate
    section: logic
    impl_mode: hand-written
    description: "Traceability metadata edge for the logic section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** needs-revision

- [logic] (item 3) The Logic flowchart has `return_override → validate_exists`, meaning the CLI `--target-branch` override is subjected to the branch-exists check. However, the `## Changes` description for `merge_target.rs` step 1 explicitly states CLI override returns "verbatim (no branch-exists check — user is explicit)". These two authoritative sections directly contradict each other. An implementer cannot determine whether the CLI override path skips or runs branch validation. Fix by making them consistent: either (a) remove the `return_override → validate_exists` edge from the Logic flowchart and route `return_override` directly to a `return_ok` terminal node (matching the Changes text), or (b) update the Changes description to state the CLI override is validated like all other resolved branches and drop the parenthetical exemption.

## Review 2
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [logic] Round-1 finding resolved: `return_override → validate_exists` edge removed; the flowchart now routes `return_override → return_ok` directly, consistent with the `## Changes` step-1 description ("return verbatim, no branch-exists check"). Both Mermaid Plus frontmatter and the rendered flowchart agree.
- [logic] All six R-ids (R1-R6) are reachable from the entry node. The CLI-override bypass, frontmatter path, current-branch fallback, config fallback, and error terminals each implement a distinct requirement.
- [schema] `target_branch` definition coheres with its usage in Logic (`check_frontmatter`/`return_frontmatter`) and in Test Plan serde round-trip tests. No unused definitions; no referenced-but-undefined types.
- [changes] The decomposition covers all required touch points (types, CLI, resolver, three test files). The duplicate entry for `projects/agentic-workflow/src/issues/types.rs` is intentional (struct fields vs. test block) and unambiguous.

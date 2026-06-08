---
id: sdd-issues-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# Issue Types

## Overview
<!-- type: overview lang: markdown -->

Type declarations in `projects/agentic-workflow/src/issues/types.rs`. Nine shapes across three groups:

- `Issue` + five enums (`ShipStatus`, `IssuePhase`, `IssueSection`, `IssueState`, `IssueType`) — the core artifact and its vocabulary types. `Issue.slug` and `Issue.body` carry `x-serde-skip: true` because they are derived from the filename and the file body respectively, not stored in frontmatter.
- `IssueFilter` + `IssuePatch` — query and update descriptors for `IssueBackend`. Both derive `Default`; all `Vec` and `bool` fields in `IssuePatch` are in `required:` so the generator emits plain types rather than `Option<T>`.
- `IssueErrorCode` — structured exit-code enum for agent-first JSON error output. Derives only `Debug, Clone, Copy, PartialEq, Eq` — no serde (custom `as_str` / `exit_code` impls are hand-written outside CODEGEN).

Codegen replaces all nine type declarations. Companion source templates own
runtime helper modules, impl blocks, free functions, regression tests,
module-level `use` declarations, and the module docstring.
The CODEGEN block owns its generated serde import; the remaining module-level import is `std::collections::HashMap`. Stable frontmatter extensions from sibling specs (`target_branch`, `ship_status`, `ship_commit`, `regen_verified_at`) are folded into this schema so the primary issue type block remains replayable.
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ShipStatus:
    type: string
    enum: [not_started, step1_shipped, loop_closed, rejected]
    description: Tracks the ship lifecycle of a merged issue.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize]
      serde_rename_all: snake_case
      variants:
        - { name: NotStarted,  doc: "No td_merged yet (default).", is_default: true }
        - { name: Step1Shipped, doc: "Validate advanced phase to td_merged and recorded the merge commit." }
        - { name: LoopClosed, doc: "Validate verified that gen-code output is byte-equivalent to current source." }
        - { name: Rejected, doc: "Issue was closed without merging." }

  IssuePhase:
    type: string
    enum: [created, fill_requirements, fill_scope, fill_reference_context, reviewed, revised, merged]
    description: Coarse phases of the CRRR loop, mirrored as Lifecycle-Stage git trailers.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      serde_rename_all: snake_case
      variants:
        - { name: Created,              doc: "Issue created." }
        - { name: FillRequirements,     doc: "Requirements section being filled." }
        - { name: FillScope,            doc: "Scope section being filled." }
        - { name: FillReferenceContext, doc: "Reference context section being filled." }
        - { name: Reviewed,             doc: "Review complete." }
        - { name: Revised,              doc: "Revision applied." }
        - { name: Merged,               doc: "Issue merged." }

  IssueSection:
    type: string
    enum: [problem, requirements, scope, reference_context]
    description: Issue body sections targetable by --section and flaggable by reviewer.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize]
      serde_rename_all: snake_case
      variants:
        - { name: Problem,          doc: "Problem statement section." }
        - { name: Requirements,     doc: "Requirements section." }
        - { name: Scope,            doc: "Scope section." }
        - { name: ReferenceContext, doc: "Reference context section." }

  IssueState:
    type: string
    enum: [open, closed, draft]
    description: Issue lifecycle state.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      serde_rename_all: lowercase
      variants:
        - { name: Open,   doc: "Open and actionable." }
        - { name: Closed, doc: "Closed (done, wontfix, duplicate, etc)." }
        - { name: Draft,  doc: "Local-only draft, not yet pushed to a tracker." }

  IssueType:
    type: string
    enum: [epic, bug, enhancement, refactor, test]
    description: Issue kind. Must match GitHub type:* labels 1:1.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      serde_rename_all: lowercase
      variants:
        - { name: Epic,        doc: "Large multi-issue initiative (may omit crate: label)." }
        - { name: Bug,         doc: "A defect in existing behavior." }
        - { name: Enhancement, doc: "New capability or improvement." }
        - { name: Refactor,    doc: "Code restructuring with no behavior change." }
        - { name: Test,        doc: "Test coverage work." }

  IssueErrorCode:
    type: string
    enum: [NotFound, Validation, Backend]
    description: Structured error codes for agent-first JSON error output.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq]
      variants:
        - { name: NotFound,   doc: "Resource not found." }
        - { name: Validation, doc: "Validation failure." }
        - { name: Backend,    doc: "Backend error." }

  Issue:
    type: object
    required: [issue_type, title, state, labels, slug, body, related, implements, validation_errors]
    description: An issue artifact — the same shape whether it comes from local files, GitHub, GitLab, Jira, or an agent-authored draft.
    properties:
      issue_type:
        type: object
        x-rust-type: "IssueType"
        x-serde-rename: "type"
        description: "Issue kind. Maps 1:1 to GitHub type:* labels."
      title:
        type: string
        description: "Full title (preserves prefixes like 'score(2.5):' for human context)."
      state:
        type: object
        x-rust-type: "IssueState"
        description: "Lifecycle state."
      id:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Local UUID — stable identifier across worktrees. Auto-generated on create."
      github_id:
        type: integer
        x-rust-type: "Option<u64>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "GitHub issue number. Backfilled after push/sync."
      gitlab_id:
        type: integer
        x-rust-type: "Option<u64>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "GitLab issue iid. Backfilled after push/sync."
      url:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Web URL where the issue lives on the tracker (if any)."
      author:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Author username."
      labels:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "All labels (including the type:* one — kept for fidelity to the backend)."
      created_at:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "ISO 8601 creation timestamp."
      updated_at:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "ISO 8601 last-update timestamp."
      slug:
        type: string
        x-serde-skip: true
        description: "Slug — the canonical local identifier (filename stem for local backend). Not serialized to frontmatter because it lives in the filename."
      body:
        type: string
        x-serde-skip: true
        description: "Markdown body (everything after frontmatter). Not stored in the frontmatter itself."
      related:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Soft see-also references to other issues, BRDs, PRDs (slugs or paths)."
      implements:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Hard references to changes or tech designs that realize this issue."
      phase:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "SDD workflow phase. None means no active SDD change for this issue."
      branch:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Git branch for this change."
      target_branch:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Target branch to merge this issue's worktree into. Optional override per issue-merge-target; when None, resolution falls through to current branch then config."
      git_workflow:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "How the branch was created: 'new_branch' or 'in_place'."
      change_id:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Change identifier — links this issue to .aw/changes/{change_id}/."
      iteration:
        type: integer
        x-rust-type: "Option<u32>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Re-proposal iteration counter."
      current_task_id:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Current spec being implemented (per-task workflow)."
      impl_spec_phase:
        type: object
        x-rust-type: "Option<HashMap<String, String>>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Per-spec implementation phase: spec_id to 'code' or 'tests'."
      task_revisions:
        type: object
        x-rust-type: "Option<HashMap<String, u32>>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Per-task revision counts."
      revision_counts:
        type: object
        x-rust-type: "Option<HashMap<String, u32>>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Per-phase revision counts."
      last_action:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Last workflow action performed."
      session_id:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Session ID for agent resume-by-index."
      validation_errors:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "CRR validation errors — set by aw wi validate, cleared on pass."
      review_count:
        type: integer
        x-rust-type: "Option<u8>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "CRRR review count — number of score-issue-reviewer runs on this issue."
      flagged_sections:
        type: array
        items: { type: string }
        x-rust-type: "Option<Vec<IssueSection>>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Sections flagged by the most recent needs-revision review."
      fill_retry_count:
        type: integer
        x-rust-type: "Option<u8>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Loop-fill retry counter. Reset to None on successful phase advance."
      ship_status:
        type: object
        x-rust-type: "Option<ShipStatus>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Ship lifecycle status for merged issues."
      ship_commit:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Git commit hash written by validate when phase first advances to td_merged."
      regen_verified_at:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Git commit hash recorded when validate confirmed regen byte-equivalence."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  IssueFilter:
    type: object
    description: Filter criteria for IssueBackend::list. All fields are conjunctive.
    properties:
      state:
        type: object
        x-rust-type: "Option<IssueState>"
        x-serde-default: true
        description: "Filter by lifecycle state."
      issue_type:
        type: object
        x-rust-type: "Option<IssueType>"
        x-serde-default: true
        description: "Filter by issue kind."
      label:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Match if the issue has this label (exact)."
      author:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Match if the issue's author equals this."
    x-rust-struct:
      derive: [Debug, Clone, Default]

  IssuePatch:
    type: object
    required: [add_labels, remove_labels, clear_phase, clear_transient]
    description: Partial update descriptor for IssueBackend::update. Only non-None or non-empty fields are applied.
    properties:
      title:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "New title to set."
      state:
        type: object
        x-rust-type: "Option<IssueState>"
        x-serde-default: true
        description: "New state to set."
      add_labels:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        description: "Labels to add."
      remove_labels:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        description: "Labels to remove."
      body:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "New body to set."
      phase:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Update SDD phase (issue-centric workflow)."
      clear_phase:
        type: boolean
        x-rust-type: "bool"
        description: "Clear SDD phase without writing a replacement."
      branch:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Update branch (issue-centric workflow)."
      target_branch:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Update target merge branch."
      git_workflow:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Update git_workflow (issue-centric workflow)."
      change_id:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Transient: change identifier."
      iteration:
        type: integer
        x-rust-type: "Option<u32>"
        x-serde-default: true
        description: "Transient: re-proposal iteration counter."
      current_task_id:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Transient: current task ID."
      impl_spec_phase:
        type: object
        x-rust-type: "Option<HashMap<String, String>>"
        x-serde-default: true
        description: "Transient: per-spec implementation phase."
      task_revisions:
        type: object
        x-rust-type: "Option<HashMap<String, u32>>"
        x-serde-default: true
        description: "Transient: per-task revision counts."
      revision_counts:
        type: object
        x-rust-type: "Option<HashMap<String, u32>>"
        x-serde-default: true
        description: "Transient: per-phase revision counts."
      last_action:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Transient: last workflow action performed."
      session_id:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Transient: session ID."
      validation_errors:
        type: array
        items: { type: string }
        x-rust-type: "Option<Vec<String>>"
        x-serde-default: true
        description: "Transient: validation errors to set."
      review_count:
        type: integer
        x-rust-type: "Option<u8>"
        x-serde-default: true
        description: "CRRR review counter — Some(n) writes, None leaves untouched."
      flagged_sections:
        type: array
        items: { type: string }
        x-rust-type: "Option<Vec<IssueSection>>"
        x-serde-default: true
        description: "CRRR flagged sections — Some(vec![]) to clear, Some(non-empty) to set, None untouched."
      fill_retry_count:
        type: integer
        x-rust-type: "Option<u8>"
        x-serde-default: true
        description: "Loop-fill retry counter — Some(n) writes, Some(0) resets, None untouched."
      clear_transient:
        type: boolean
        x-serde-default: true
        description: "Clear all transient SDD fields at once (used by merge)."
      ship_status:
        type: object
        x-rust-type: "Option<ShipStatus>"
        x-serde-default: true
        description: "Ship lifecycle status — Some(status) writes, None leaves untouched."
      ship_commit:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Git commit hash written at step1_shipped — Some(hash) writes, None leaves untouched."
      regen_verified_at:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Git commit hash written at loop_closed — Some(hash) writes, None leaves untouched."
    x-rust-struct:
      derive: [Debug, Clone, Default]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/types.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - Issue
      - ShipStatus
      - IssuePhase
      - IssueSection
      - IssueState
      - IssueType
      - IssueFilter
      - IssuePatch
      - IssueErrorCode
    description: |
      Codegen replaces all nine type declarations wrapped between
      // CODEGEN-BEGIN and // CODEGEN-END markers. All generated items
      carry @spec markers referencing this file's #schema anchor.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [schema] `Issue.slug` and `Issue.body` both carry `x-serde-skip: true` and are listed in `required:` — generator emits plain `String` with `#[serde(skip)]` rather than `Option<String>`, matching source lines 57-63. Primary verification passes.
- [schema] `Issue.required` includes all Vec fields (`labels`, `related`, `implements`, `validation_errors`) — no auto-Option wrapping risk for any container field.
- [schema] `IssuePatch.required` includes `add_labels`, `remove_labels` (Vec), `clear_phase` (bool), and `clear_transient` (bool) — consistent with source plain types at lines 393-420.
- [schema] `IssueSection` derive list includes `Hash, Ord, PartialOrd` — R7 satisfied; matches source line 209.
- [overview] Minor: overview and changes list 8 types while issue slug/title says "9 types". Spec is internally consistent at 8; discrepancy is in the issue title only and does not affect implementation.

---
id: score-td-revise-payload-merge-and-takeover
fill_sections: [schema, logic, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior."
---

# Score TD Revise — Payload Merge + Takeover Guard

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ReviseArgs:
    type: object
    description: >
      Arguments accepted by `aw td revise`. The `section` field mirrors
      `CreateArgs.section` so the reviser loop can apply per-section payloads
      with the same pre-validation merge step the author loop uses.
    properties:
      slug:
        type: string
        description: Issue slug (matches the worktree directory name).
      apply:
        type: boolean
        description: Apply mode — merge payload (when section set), validate, emit dispatch envelope.
      spec_path:
        type: string
        description: Spec file path relative to worktree root. Required with --apply.
      section:
        type: string
        description: >
          Per-section apply: read `.aw/payloads/<slug>/<section>.md` and merge it
          into the spec's `<section>` block BEFORE validation. Mirrors
          `aw td create --apply --section`. Required so the validator sees the
          post-merge result instead of the spec at HEAD (otherwise reviser cannot
          remove content the author committed — e.g. a deprecated section).
    required: [slug]

  TakeoverDetection:
    type: object
    description: >
      Signature `handle_revise_milestone` recognises as a valid mainthread takeover.
      When subagent fails, mainthread can edit the spec and commit with a `Td-Revise`
      Lifecycle-Stage trailer. Validate must accept that commit and advance phase
      without making a duplicate commit.
    properties:
      head_message_contains:
        type: string
        const: "Lifecycle-Stage: Td-Revise"
        description: Required substring in HEAD's commit message body.
      diff_head_to_parent_includes_spec:
        type: boolean
        const: true
        description: >
          The spec file MUST appear in `git diff --name-only HEAD~1 HEAD --` for the
          guard to accept the takeover. This proves the takeover commit actually
          modified the spec, not an unrelated commit happening to carry the trailer.
```

## Logic: revise-apply with section merge
<!-- type: logic lang: mermaid -->

```mermaid
---
id: revise-apply-section-merge
entry: start
nodes:
  start:           { kind: start,    label: "aw td revise --apply [--section X]" }
  has_section:     { kind: decision, label: "args.section set?" }
  read_payload:    { kind: process,  label: "read .aw/payloads/<slug>/<X>.md" }
  payload_exists:  { kind: decision, label: "payload exists?" }
  read_base_spec:  { kind: process,  label: "read worktree's spec_path" }
  merge_section:   { kind: process,  label: "merge_spec_section(base, X, payload)" }
  write_merged:    { kind: process,  label: "write merged spec back" }
  cleanup_payload: { kind: process,  label: "rm .aw/payloads/<slug>/<X>.md" }
  read_spec:       { kind: process,  label: "read spec file" }
  validate:        { kind: process,  label: "validate_spec(content)" }
  validate_ok:     { kind: decision, label: "errors.empty()?" }
  emit_dispatch:   { kind: terminal, label: "emit dispatch → aw td validate" }
  emit_error:      { kind: terminal, label: "emit error envelope" }
edges:
  - { from: start,            to: has_section }
  - { from: has_section,      to: read_payload,    kind: branch, label: yes }
  - { from: has_section,      to: read_spec,       kind: branch, label: no }
  - { from: read_payload,     to: payload_exists }
  - { from: payload_exists,   to: read_base_spec,  kind: branch, label: yes }
  - { from: payload_exists,   to: emit_error,      kind: branch, label: no }
  - { from: read_base_spec,   to: merge_section }
  - { from: merge_section,    to: write_merged }
  - { from: write_merged,     to: cleanup_payload }
  - { from: cleanup_payload,  to: read_spec }
  - { from: read_spec,        to: validate }
  - { from: validate,         to: validate_ok }
  - { from: validate_ok,      to: emit_dispatch,   kind: branch, label: yes }
  - { from: validate_ok,      to: emit_error,      kind: branch, label: no }
```

## Logic: revise-milestone takeover guard
<!-- type: logic lang: mermaid -->

```mermaid
---
id: revise-milestone-takeover-guard
entry: start
nodes:
  start:                 { kind: start,    label: "handle_revise_milestone(args)" }
  has_uncommitted:       { kind: decision, label: "git diff HEAD spec_path empty?" }
  is_takeover:           { kind: decision, label: "head message has Td-Revise trailer AND HEAD~1..HEAD touches spec?" }
  reject:                { kind: terminal, label: "emit error: no revision detected" }
  validate:              { kind: process,  label: "validate_spec(content)" }
  validate_ok:           { kind: decision, label: "errors.empty()?" }
  rollback_or_skip:      { kind: decision, label: "had uncommitted changes?" }
  rollback_file:         { kind: process,  label: "rollback_worktree_file(spec_path)" }
  emit_validate_error:   { kind: terminal, label: "emit error: revision validation failed" }
  update_phase:          { kind: process,  label: "patch issue: phase=td_revised, flagged_sections=[]" }
  has_uncommitted_again: { kind: decision, label: "had uncommitted changes?" }
  commit_lifecycle:      { kind: process,  label: "git add spec + issue; git commit -m 'td(slug) — revised' + Td-Revise trailer" }
  skip_commit:           { kind: process,  label: "(takeover already committed — no new commit)" }
  emit_review:           { kind: terminal, label: "emit dispatch → score-td-reviewer (review #2)" }
edges:
  - { from: start,                 to: has_uncommitted }
  - { from: has_uncommitted,       to: validate,              kind: branch, label: "no (uncommitted exists)" }
  - { from: has_uncommitted,       to: is_takeover,           kind: branch, label: "yes (working tree clean)" }
  - { from: is_takeover,           to: validate,              kind: branch, label: "yes (takeover commit)" }
  - { from: is_takeover,           to: reject,                kind: branch, label: "no (no signal)" }
  - { from: validate,              to: validate_ok }
  - { from: validate_ok,           to: rollback_or_skip,      kind: branch, label: no }
  - { from: validate_ok,           to: update_phase,          kind: branch, label: yes }
  - { from: rollback_or_skip,      to: rollback_file,         kind: branch, label: yes }
  - { from: rollback_or_skip,      to: emit_validate_error,   kind: branch, label: no }
  - { from: rollback_file,         to: emit_validate_error }
  - { from: update_phase,          to: has_uncommitted_again }
  - { from: has_uncommitted_again, to: commit_lifecycle,      kind: branch, label: "yes (subagent path)" }
  - { from: has_uncommitted_again, to: skip_commit,           kind: branch, label: "no (takeover path — already committed)" }
  - { from: commit_lifecycle,      to: emit_review }
  - { from: skip_commit,           to: emit_review }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/td.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      ReviseArgs gains optional `section` field (mirrors CreateArgs.section).
      run_revise_apply gains the same per-section merge-then-validate pipeline as
      run_create_apply: read .aw/payloads/<slug>/<section>.md, merge into spec
      via merge_spec_section, write back, then validate the merged content.
      Closes Bug 1 (revise validated HEAD spec instead of post-merge result).

      handle_revise_milestone guard loosened: accept either an uncommitted change
      OR a HEAD takeover commit. New helper head_is_takeover_revise(worktree, spec)
      returns true when (i) HEAD message body contains 'Lifecycle-Stage: Td-Revise'
      and (ii) git diff HEAD~1 HEAD includes the spec. When the takeover branch
      fires, validate skips the duplicate commit_lifecycle call (takeover already
      committed) but still advances phase + dispatches reviewer. Closes Bug 2.

  - path: projects/agentic-workflow/tech-design/surface/specs/score-td-revise-payload-merge-and-takeover.md
    action: create
    section: schema
    impl_mode: hand-written
    description: This spec file.
```

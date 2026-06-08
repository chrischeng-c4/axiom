---
id: three-role-contract
main_spec_ref: projects/agentic-workflow/specs/three-role-contract.md
merge_strategy: new
status: superseded
superseded_by: aw-mainthread-only-execution.md
superseded_on: 2026-05-03
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior."
---

# Three-Role Contract (Mainthread / Subagent / Hook)

> **SUPERSEDED.** The three-role separation described below was the
> dispatch model used until 2026-05-03. As of `aw-mainthread-only-execution.md`
> (merged 2026-05-03), the `subagent` role is being removed entirely:
> mainthread now owns both the orchestration AND the `--apply` step, while
> Claude Code hooks (PostToolUse Hook 1, PreToolUse Hook 2,
> UserPromptSubmit Hook 5) enforce the two-phase-commit invariant that
> `SubagentStop` previously gated. The current contract is a
> **two-role** model — mainthread + hooks — with the subagent column
> dropped.
>
> See `projects/agentic-workflow/tech-design/surface/specs/aw-mainthread-only-execution.md`
> for the new authoritative contract. The remainder of this document
> is preserved for historical reference only; do not implement against
> it.

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: happy_path_change_spec_subagent_writes_valid_artifact
    title: "Happy path - change-spec subagent writes valid artifact"
    given:
      - "score-change-spec subagent is dispatched at phase ChangeSpecCreated"
    when:
      - "subagent writes payload and runs score artifact create-change-spec <id> payload.json"
      - "subagent stops"
    then:
      - "SubagentStop hook runs score workflow validate <id> --agent-type score-change-spec"
      - "validate returns passed=true and phase_advanced_to=ChangeSpecReviewed"
      - "hook exits 0 with additionalContext indicating validation passed"
      - "mainthread's next score run-change returns the review-change-spec action"

  - id: bogus_artifact_subagent_writes_incomplete_spec
    title: "Bogus artifact - subagent writes incomplete spec"
    given:
      - "score-change-spec subagent writes a spec payload missing fill_sections"
    when:
      - "subagent stops"
    then:
      - "validate returns passed=false with fill_sections missing"
      - "hook returns decision=block with reason fill_sections missing and exits 2"
      - "Claude Code re-enters the subagent in the same invocation with the error as the new prompt"
      - "subagent fixes payload, re-runs score artifact, and stops again"
      - "validate passes and phase advances"

  - id: subagent_hits_maxturns_without_completing
    title: "Subagent hits maxTurns without completing"
    given:
      - "score-change-spec subagent dispatched"
    when:
      - "subagent exhausts maxTurns without running score artifact"
    then:
      - "SubagentStop hook runs validate"
      - "validate sees no artifact written and returns passed=false"
      - "hook emits decision=block, but maxTurns is already exhausted so the block cannot retry"
      - "task notification arrives at mainthread with no phase advance"
      - "mainthread decides whether to re-dispatch as a new iteration or escalate"
    notes:
      - "Cross-invocation retry counter is deferred and out of scope for this historical spec."

  - id: mainthread_attempts_to_write_artifact
    title: "Mainthread attempts to write artifact"
    given:
      - "mainthread has an in-flight change and attempts score artifact review-change-spec"
    when:
      - "the command runs"
    then:
      - "CLI succeeds because the CLI itself is neutral"
      - "no SubagentStop fires, so the validate gate does not run"
      - "artifact_writes.jsonl has no entry for this action because it logs only agent-originated writes"
    notes:
      - "The contract relies on mainthread discipline; env-var enforcement is a follow-up."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/workflow_validate.rs
    action: create
    section: changes
    impl_mode: hand-written
    purpose: "New validation subcommand"
  - path: projects/agentic-workflow/src/tools/mod.rs
    action: modify
    section: changes
    impl_mode: hand-written
    purpose: "Register sdd_workflow_validate"
  - path: projects/agentic-workflow/src/tools/create_change_spec.rs
    action: modify
    section: scenarios
    impl_mode: hand-written
    purpose: "Remove update_phase at 170 and 232"
  - path: projects/agentic-workflow/src/tools/review_change_spec.rs
    action: modify
    section: scenarios
    impl_mode: hand-written
    purpose: "Remove update_phase at 292"
  - path: projects/agentic-workflow/src/tools/review_change_impl.rs
    action: modify
    section: scenarios
    impl_mode: hand-written
    purpose: "Remove update_phase at 198"
  - path: projects/agentic-workflow/src/tools/create_change_docs.rs
    action: modify
    section: changes
    impl_mode: hand-written
    purpose: "Remove update_phase at 116, 149, and 219"
  - path: projects/agentic-workflow/src/tools/review_change_docs.rs
    action: modify
    section: scenarios
    impl_mode: hand-written
    purpose: "Remove update_phase at 171"
  - path: projects/agentic-workflow/src/cli/commands.rs
    action: modify
    impl_mode: hand-written
    section: source
    purpose: "Add WorkflowValidate CLI arg parsing and artifact_writes.jsonl append"
  - path: projects/agentic-workflow/src/cli/init.rs
    action: modify
    impl_mode: hand-written
    section: source
    purpose: "Install 3 new hooks and merge settings.json"
  - path: projects/agentic-workflow/templates/mainthread/hooks/score-subagent-start.sh
    action: create
    impl_mode: hand-written
    section: source
    purpose: "R2 additionalContext injector"
  - path: projects/agentic-workflow/templates/mainthread/hooks/score-artifact-guard.sh
    action: create
    impl_mode: hand-written
    section: source
    purpose: "R3 PreToolUse Edit/Write deny"
  - path: projects/agentic-workflow/templates/mainthread/hooks/score-validate-advance.sh
    action: create
    impl_mode: hand-written
    section: source
    purpose: "R5 SubagentStop gate; replaces score-next-step.sh"
  - path: projects/agentic-workflow/templates/mainthread/hooks/score-safe-bash.sh
    action: modify
    impl_mode: hand-written
    section: source
    purpose: "R10 git-mutation deny list"
  - path: projects/agentic-workflow/templates/mainthread/settings.json
    action: modify
    impl_mode: hand-written
    section: source
    purpose: "Register new hooks"
  - path: projects/agentic-workflow/tech-design/core/logic/dispatch-model.md
    action: modify
    section: changes
    impl_mode: hand-written
    purpose: "SubagentStop changes from observation to gatekeeper"
  - path: projects/agentic-workflow/tech-design/core/tools/utils/write-artifact.md
    action: modify
    section: changes
    impl_mode: hand-written
    purpose: "Note that phase writes are removed"
  - path: projects/agentic-workflow/tech-design/core/tools/utils/validate-change.md
    action: modify
    section: changes
    impl_mode: hand-written
    purpose: "Add --agent-type and per-agent rules"
  - path: CLAUDE.md
    action: modify
    section: changes
    impl_mode: hand-written
    purpose: "Replace stale files-affected line from parent issue R21"
```

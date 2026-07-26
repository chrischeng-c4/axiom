---
id: aw-wi-two-stage-project-plan
summary: "Drive deterministic normalization, reviewed reconciliation, and human-confirmed atomization as one chainable project-plan workflow root."
fill_sections: [logic, cli, e2e-test, changes]
capability_refs:
  - id: work-item-planning
    role: primary
    gap: epic-to-change-atomization
    claim: epic-to-change-atomization
    coverage: full
    rationale: "The planner jointly atomizes and prioritizes epics, then reconciles and atomizes their owned changes."
command_refs:
  - command: aw wi plan
  - command: aw wi epicize
  - command: aw wi atomize
  - command: aw wi prioritize
  - command: aw wi graph
---

# AW WI Two-Stage Project Plan

## Logic
<!-- type: logic lang: markdown -->

`aw.wi.project-plan.v2` is the stage-aware planning model for a configured
project's complete tracker inventory. `aw wi plan --project <project>` is the
workflow root: every bounded tick emits an `aw.cli.v1` envelope with a stable
root id, current stage, exact `invoke.command`, HITL question when required,
and `completion.workflow_complete`. Agents never choose the next stage and
never treat a child `action=done` as root completion.

The root runs `normalize -> reconcile -> atomize -> verify`:

1. `normalize` may update only mechanically provable canonical metadata:
   explicit type, project identity, explicit parent, explicit priority, and
   explicit dependency declarations. It never creates, closes, infers an
   owner, splits work, or requires independent semantic review. Its unreviewed
   apply path recompiles the exact plan and manifest from live configured
   inventory; editable artifact metadata cannot authorize writes.
2. `reconcile` evaluates owner, priority, duplicate, and supersession
   relationships among existing issues. Every non-explicit inference becomes
   a digest-bound HITL question before entering a mutation manifest. It never
   creates or closes.
3. `atomize` proposes new epics/changes, mixed-horizon siblings, and bounded
   replacements. Its exact candidates require policy-compliant semantic review
   (independent agent by default, native human review when explicitly
   configured) and then explicit human confirmation before create is admitted.
4. `verify` rebuilds the strict graph and all stages from the post-apply
   snapshot. Only a valid graph, no unresolved decisions, and zero pending
   mutations set `completion.workflow_complete=true`.

Every proposal and mutation records its stage, certainty
(`deterministic|inferred|decision`), evidence, decision source, HITL state, and
tracker snapshot digest. Stage mutation allowlists are checked both at manifest
creation and immediately before apply.

Unowned changes are never grouped into an invented `unclassified` bootstrap
epic. Explicit parent declarations may be normalized; otherwise reconcile asks
the human to select, defer, reject, or revise the owner decision.

`## Requirements` or a structured requirement id is the only atomization
source. Indented Markdown continuation lines remain part of their preceding
Requirement rather than being dropped. Scope and Acceptance Criteria explain
the contract but never become new backlog leaves. A requirement is deferred
only by a canonical label or an explicit scheduling prefix such as `Deferred:`
or `Later phase:`; merely
mentioning active/deferred policy does not split an epic. Closed/delivered
changes and transaction-marked local drafts count as coverage, and an explicit
`## Child Work Items` plan covers the aggregate epic contract once every listed
child is closed. A `Covers` column containing parent Requirement ids provides
partial-plan coverage only for the named, valid child WI; ordinary authoring
drafts and partially completed child plans without that mapping do not.
Oversized
replacements are emitted only for non-empty partitions of authoritative
Requirement ids. Missing authoritative requirements produce a diagnostic
rather than an inferred change.

An epic may declare `## Verification Inventory` as a Markdown table with
`Requirement | Gate | Oracle` columns. The planner binds each row to the named
Requirement id and carries its complete runnable gate and observable oracle
into every proposed child mutation for independent review.

All vectors and maps are sorted before serialization. The plan digest is
SHA-256 over the complete model with only its `digest` field cleared. The
model contains no timestamp, invocation alias, output path, or reviewer
identity, so identical tracker content produces byte-equivalent plan content
and the same digest.

Planning ticks never create, edit, label, or close tracker issues. They write
only previews, root state, decisions, and digest-bound review sidecars under
`/tmp/aw/workspaces/<workspace>/workitems/<project>/project-plan/`.
`aw wi plan-apply` is the sole tracker writer.

## CLI
<!-- type: cli lang: yaml -->

```yaml
authority:
  command: aw wi plan --project <project> [--stage normalize|reconcile|atomize|verify] [--json]
  artifact: /tmp/aw/workspaces/<workspace>/workitems/<project>/project-plan/project-plan.json
  schema: aw.wi.project-plan.v2
  review_kind: project_plan
compatibility:
  epicize: structured redirect to aw wi plan --stage atomize
  atomize: structured redirect to aw wi plan --stage atomize
  prioritize: structured redirect to aw wi plan --stage reconcile
envelope:
  schema_version: aw.cli.v1
  continuation: invoke.command
  terminal: completion.workflow_complete=true
write_set:
  - /tmp/aw/workspaces/<workspace>/workitems/<project>/project-plan/**
  - /tmp/aw/workspaces/<workspace>/payloads/planning-review/<project>/project_plan/**
tracker_write_set: []
invalid_graph:
  action: blocked
  exit: non-zero
  diagnostics: aw.wi.graph.v1 diagnostics
```

`--cap-path` on `aw wi plan` is decode-only compatibility input. Capability
sweep keeps its internal capability-claim planner, while public project
planning starts from the complete epic/change inventory. `--title` affects
only the CLI report and never the canonical model or digest. An explicit
`--output` must remain inside the canonical project-plan directory.

## E2E Test
<!-- type: e2e-test lang: markdown -->

- Starting with one `aw wi plan` and following every emitted
  `invoke.command` reaches `completion.workflow_complete=true`.
- Normalize emits no create/close and no inferred owner mutation.
- Reconcile emits bounded HITL questions for unresolved inference.
- Atomize cannot apply without policy-compliant semantic acceptance and human
  confirmation; explicit human-only review has an executable approve/revise
  recording command.
- Scope/Acceptance Criteria never produce candidates; Requirements do.
- Multi-line Requirements retain their complete normalized text.
- Requirement-specific Verification Inventory gates and oracles survive in the
  proposed child body without being shortened to its compact title.
- Closed/delivered coverage suppresses duplicate candidates.
- A complete explicit Child Work Items plan suppresses duplicate candidates,
  while a partially completed plan covers only Requirement ids explicitly
  mapped in its `Covers` column.
- Normative prose that mentions deferred scheduling does not create a deferred
  horizon without an explicit scheduling prefix or canonical label.
- Ordinary unmarked drafts never suppress missing Requirement candidates.
- Oversized replacement leaves always cover at least one authoritative
  Requirement.
- Unowned changes never create an `unclassified` bootstrap epic.
- Every non-terminal emitted command parses through the real clap tree.
- A locally published, transaction-marked draft change satisfies coverage on
  the next plan, so a post-publication rerun has no duplicate create mutation
  and identical unchanged reruns have the same digest.
- Strict graph success without zero-diff planning evidence is not terminal.

Gate: `cargo test -p agentic-workflow --test wi_two_stage_project_plan_cli_test -- --nocapture`

Review compatibility gate: `cargo test -p agentic-workflow --test inventory_plan_review_cli_test -- --nocapture`

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/issues/planner.rs
    action: create
    section: logic
    impl_mode: handwrite
    description: "DDD project-plan aggregate, stable schema/digest, staged certainty reconciliation, and shared boundedness classifier."
  - path: apps/agentic-workflow/src/issues/mod.rs
    action: modify
    section: logic
    impl_mode: codegen
    description: "Expose the project planner domain surface."
  - path: apps/agentic-workflow/src/cli/issues.rs
    action: modify
    section: cli
    impl_mode: codegen
    description: "Route every public planning verb to one canonical preview and review payload."
  - path: apps/agentic-workflow/tests/wi_two_stage_project_plan_cli_test.rs
    action: create
    section: e2e-test
    impl_mode: handwrite
    description: "Compiled staged planning, deterministic delegation, and graph fail-closed evidence."
  - path: apps/agentic-workflow/tests/inventory_plan_review_cli_test.rs
    action: modify
    section: e2e-test
    impl_mode: handwrite
    description: "Prove compatibility planning verbs share one project-plan review authority."
```

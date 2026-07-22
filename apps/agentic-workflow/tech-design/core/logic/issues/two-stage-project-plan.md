---
id: aw-wi-two-stage-project-plan
summary: "Replace competing epicize, atomize, and prioritize artifacts with one deterministic two-stage epic/change project plan."
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

`aw.wi.project-plan.v1` is the single authoritative read-only planning model
for a configured project's issue inventory. Its input is the complete tracker
inventory plus the canonical project label. Before planning, it builds
`aw.wi.graph.v1`. Every structural graph diagnostic except an unowned change
fails closed and is copied into a blocked project plan without guessing. An
unowned change is the deliberate migration exception: the planner groups it
by declared capability context and active/deferred horizon, proposes one
explicit bootstrap epic owner for that group, and leaves that full mapping for
independent review before publication.

The model runs two ordered stages:

1. `epic_inventory / atomize_and_prioritize` loads every open project epic,
   extracts active and deferred requirements, proposes active/deferred sibling
   epics for mixed-horizon roots, adds deterministic DDD/horizon bootstrap
   epics for legacy unowned changes, and orders the resulting epic work by
   explicit priority and horizon.
2. `change_inventory_by_epic / reconcile_atomize_and_prioritize` combines each
   epic contract with its existing owned changes, records valid coverage,
   identifies explicit or title-equivalent duplicates, applies the shared
   #2142 boundedness classifier, proposes sibling replacements for oversized
   leaves, and creates candidates only for uncovered requirements.

Mixed-epic proposals participate independently in the global priority order,
so a deferred `p3` sibling cannot jump ahead of an unrelated active `p2` epic.
Replacement proposals count as the planned coverage for the oversized source's
matched requirements; the planner does not also emit a redundant generic gap
proposal for the same requirement.

Every existing and proposed change has exactly one `owner_epic`. Existing
change priority is explicit or inherited from its epic. Proposed change
priority is inherited from its owner; deferred work is `p3`. Change lanes are
`ready_now`, `blocked_by_dependency`, `needs_triage`, `needs_atomize`,
`duplicate`, and `deferred`. Open dependencies block readiness; closed
dependencies remain graph evidence but do not block. A local backend may hold
a transaction-marked published plan record in `draft` until its ordinary WI
validation promotes it; that exact provenance-bearing draft remains active for
plan reconciliation, while unrelated authoring drafts do not enter the plan.
Once both transaction-marked active and deferred siblings name one mixed source
epic, those siblings are the source's published planning representation. The
retained source epic remains tracker history and is not split again on an
unchanged reread. Existing source changes are assigned to the sibling matching
their own active/deferred horizon during the same publication, so every change
remains represented exactly once after the source becomes historical. That
ownership update replaces every legacy body parent declaration with one
canonical `Parent Epic` value; it never leaves an old `Parent:` alongside the
new graph label.

For a transaction-marked published epic, `## Requirements` is the authoritative
planning contract; list items in its Scope or Acceptance Criteria explain that
contract but do not become fresh change requirements on a replan. Legacy epics
continue to accept their existing Scope and Acceptance Criteria compatibility
inputs.

All vectors and maps are sorted before serialization. The plan digest is
SHA-256 over the complete model with only its `digest` field cleared. The
model contains no timestamp, invocation alias, output path, or reviewer
identity, so identical tracker content produces byte-equivalent plan content
and the same digest.

The planner never creates, edits, labels, or closes tracker issues. It writes
only a canonical preview and digest-bound review sidecars under
`/tmp/aw/workspaces/<workspace>/workitems/<project>/project-plan/`.
Accepted review publication is a separate consumer specified by
`project-plan-transaction.md`: it binds the exact snapshot and ordered
mutations into the review digest, preflights before writing, and reconciles
idempotency markers on retry.

## CLI
<!-- type: cli lang: yaml -->

```yaml
authority:
  command: aw wi plan --project <project> [--json]
  artifact: /tmp/aw/workspaces/<workspace>/workitems/<project>/project-plan/project-plan.json
  schema: aw.wi.project-plan.v1
  review_kind: project_plan
compatibility:
  - aw wi epicize --project <project> [--json]
  - aw wi atomize --project <project> [--json]
  - aw wi prioritize --project <project> [--json]
delegation:
  model: identical
  artifact_path: identical
  digest: identical
  review_payload: identical
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

- A mixed-horizon epic produces active and deferred sibling epic proposals
  before ordering; the deferred proposal receives `p3`.
- Valid existing coverage suppresses a missing candidate, while a duplicate,
  an unstructured leaf, and an oversized leaf do not produce false coverage.
- An oversized existing change produces two bounded sibling replacements under
  the same owning epic through the shared #2142 classifier, without a redundant
  generic gap proposal for the same requirement.
- Open dependencies produce `blocked_by_dependency`; otherwise structured,
  bounded changes are `ready_now`.
- Every existing and proposed change exposes one owner.
- `plan`, `epicize`, `atomize`, and `prioritize` write the same path, bytes,
  model digest, review kind, and review digest.
- Legacy `Parent: #<epic>.` bodies normalize to their existing epic without a
  tracker write.
- Unowned changes produce reviewed DDD/horizon bootstrap epic proposals and
  one deterministic owner mapping; multiple owners and every other structural
  graph defect remain blocked with a non-zero CLI exit.
- A locally published, transaction-marked draft change satisfies coverage on
  the next plan, so a post-publication rerun has no duplicate create mutation
  and identical unchanged reruns have the same digest.
- After a mixed source epic publishes its active and deferred transaction-marked
  siblings, a replan uses those siblings, retains every former source change
  exactly once under its horizon sibling, and does not propose the same split a
  second time.

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
    description: "DDD project-plan aggregate, stable schema/digest, two-stage reconciliation, and shared boundedness classifier."
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
    description: "Compiled two-stage planning, deterministic delegation, and graph fail-closed evidence."
  - path: apps/agentic-workflow/tests/inventory_plan_review_cli_test.rs
    action: modify
    section: e2e-test
    impl_mode: handwrite
    description: "Prove compatibility planning verbs share one project-plan review authority."
```

---
id: aw-wi-project-plan-transaction
fill_sections: [overview, requirements, behavior, logic, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: work-item-planning
    role: primary
    gap: epic-to-change-atomization
    claim: digest-bound-project-planning-transaction
    coverage: full
    rationale: "One accepted project-plan digest must authorize one coherent issue-platform transaction."
command_refs:
  - command: aw wi plan
  - command: aw wi plan-review
  - command: aw wi graph
---

# Digest-Bound Project Planning Transaction

## Overview
<!-- type: overview lang: markdown -->

An accepted `aw.wi.project-plan.v1` is published through one
`aw.wi.project-plan-transaction.v1` manifest. The manifest binds the plan,
the exact project tracker snapshot, every ordered mutation, the executable
apply command, and the terminal graph command into the review digest.

The transaction never deletes work items. Duplicate and supersession decisions
are recorded as reviewed labels and body evidence so tracker history remains
available. Every create/update has a stable idempotency key stored on the issue;
a local checkpoint is audit evidence, while tracker markers are the recovery
authority after an ambiguous transport failure.

The transaction does not write a provenance-only update to an already clean
issue. A post-publication plan therefore converges to an empty mutation set;
re-running unchanged inventory preserves that plan digest.

Every proposed epic carries its reviewed planning horizon. The active and
deferred transaction-marked siblings of a mixed source epic are durable
publication evidence: subsequent planning retains the original for history but
uses its siblings instead of proposing the same split again. Existing changes
of the source are relabelled to the matching sibling only after that sibling
exists, preserving complete reconciliation across the transition. The same
update replaces every legacy body parent declaration with one canonical parent,
so strict graph verification never observes both old and new owners.

The transaction marker also makes the generated `## Requirements` section
authoritative on subsequent planning reads. Scope and acceptance list items are
evidence, not additional backlog leaves, which keeps an unchanged replan at
zero mutations.

## Requirements
<!-- type: requirements lang: yaml -->

```yaml
requirements:
  - id: R1
    text: One independent review covers the complete normalized project plan.
  - id: R2
    text: The review digest binds the exact tracker snapshot, mutation manifest, apply command, and terminal command.
  - id: R3
    text: Stale evidence, same-agent review, incomplete checklists, unsupported commands, and tracker drift fail before the first mutation.
  - id: R4
    text: needs_revision publishes nothing and explicit human-only review remains an opt-in policy.
  - id: R5
    text: Apply epic creates, change creates, epic updates, and change updates in a stable dependency-safe order.
  - id: R6
    text: Duplicate and superseded issues are retained; the transaction records recommendations without deleting or closing them.
  - id: R7
    text: Retry reconciles tracker idempotency markers and converges without duplicate issues or conflicting labels.
  - id: R8
    text: A clean post-publication inventory produces no provenance-only mutation, no duplicate proposal, and the same digest on unchanged reread.
```

## Behavior
<!-- type: behavior lang: gherkin -->

```gherkin
Feature: publish one reviewed project planning transaction

  Scenario: accepted review applies its exact manifest
    Given aw wi plan wrote one project plan and transaction manifest
    And an independent reviewer accepted the exact source digest
    When aw wi plan-review applies the evidence
    Then tracker snapshot preflight completes before the first write
    And proposed epics are created before proposed changes
    And canonical epic and change graph labels are updated afterward
    And the result names every reviewed mutation and one executable terminal command

  Scenario: tracker drift aborts before mutation
    Given a project issue changed after the review digest was authored
    When aw wi plan-review preflights the tracker
    Then it fails before the first mutation
    And the diagnostic names the changed issue

  Scenario: ambiguous transport failure resumes
    Given a create reached the tracker but its response failed before checkpointing
    When the same reviewed evidence is retried
    Then the transaction resolves the create through its tracker marker
    And it applies only remaining mutations
    And no duplicate work item is created

  Scenario: complete transaction reapplies as a no-op
    Given every reviewed mutation is already present on the tracker
    When the same accepted review is applied again
    Then the result is complete with no_op true and applied_count zero

  Scenario: local publication converges before draft promotion
    Given a local backend stores a created plan change as draft
    And the change carries its reviewed planning transaction marker
    When aw wi plan rereads the unchanged inventory
    Then the draft satisfies existing coverage
    And the next manifest has no mutation for it

  Scenario: mixed-horizon publication converges
    Given a mixed source epic produced reviewed active and deferred siblings
    And the transaction published both siblings with their planning horizons
    When aw wi plan rereads the unchanged inventory
    Then it uses the published siblings as the source representation
    And each existing source change belongs to exactly one horizon sibling
    And aw wi graph is valid with no stale body parent declaration
    And it does not propose another sibling pair or mutation
```

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: project-plan-transaction
entry: load_review
nodes:
  load_review: { kind: start, label: "load accepted digest-bound review" }
  validate_digest: { kind: decision, label: "plan + manifest + command digest valid?" }
  load_tracker: { kind: process, label: "load complete project tracker snapshot" }
  reconcile_markers: { kind: process, label: "resolve prior creates through idempotency markers" }
  preflight: { kind: decision, label: "snapshot equals reviewed or exact applied state?" }
  apply_ordered: { kind: process, label: "create epics, create changes, update epics, update changes" }
  checkpoint: { kind: process, label: "atomically checkpoint each resolved mutation" }
  complete: { kind: terminal, label: "complete result + aw wi graph next" }
  rejected: { kind: terminal, label: "reject before first mutation" }
edges:
  - { from: load_review, to: validate_digest, label: "decode" }
  - { from: validate_digest, to: rejected, label: "invalid" }
  - { from: validate_digest, to: load_tracker, label: "valid" }
  - { from: load_tracker, to: reconcile_markers, label: "inventory" }
  - { from: reconcile_markers, to: preflight, label: "resolved ids" }
  - { from: preflight, to: rejected, label: "drift" }
  - { from: preflight, to: apply_ordered, label: "clean" }
  - { from: apply_ordered, to: checkpoint, label: "one mutation" }
  - { from: checkpoint, to: apply_ordered, label: "remaining" }
  - { from: checkpoint, to: complete, label: "all reconciled" }
---
flowchart TD
    load_review([accepted review]) --> validate_digest{digest valid?}
    validate_digest -- no --> rejected([reject without writes])
    validate_digest -- yes --> load_tracker[load tracker]
    load_tracker --> reconcile_markers[resolve idempotency markers]
    reconcile_markers --> preflight{snapshot clean?}
    preflight -- no --> rejected
    preflight -- yes --> apply_ordered[ordered apply/reconcile]
    apply_ordered --> checkpoint[atomic checkpoint]
    checkpoint -- remaining --> apply_ordered
    checkpoint -- complete --> complete([terminal result + graph next])
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
producer:
  command: aw wi plan --project <project> --json
  manifest: /tmp/aw/workspaces/<workspace>/workitems/<project>/project-plan/project-plan.manifest.json
  evidence: /tmp/aw/workspaces/<workspace>/workitems/<project>/project-plan/project-plan.review-payload.json
apply:
  command: aw wi plan-review --evidence-file <evidence> --json
  checkpoint: /tmp/aw/workspaces/<workspace>/workitems/<project>/project-plan/project-plan.transaction.json
  terminal_next: aw wi graph --project <project> --json
mutation_order:
  - create_epic
  - create_change
  - update_epic
  - update_change
```

## Unit Test
<!-- type: unit-test lang: yaml -->

```yaml
gate: cargo test -p agentic-workflow --lib planning_transaction::tests -- --nocapture
proof:
  - tracker drift fails before checkpoint creation
  - a create that succeeds before transport failure is reconciled on retry
  - a third application is a clean no-op
  - post-publication lifecycle metadata may advance while graph-label drift is rejected
  - unchanged graph-clean issues receive no provenance-only update mutation
  - transaction-marked active and deferred siblings suppress a duplicate
    mixed-source split on replan
  - existing mixed-source changes are reparented only after their sibling epic
    create is resolved
  - a reparent update removes stale legacy body parents before strict graph
    verification
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
gate: cargo test -p agentic-workflow --test wi_project_plan_transaction_cli_test -- --nocapture
proof:
  - compiled aw binds snapshot, mutations, and apply command into one manifest
  - accepted review creates and updates local issue-platform artifacts
  - durable accepted evidence reapplies without duplicate issues
  - a post-publication local plan has zero mutations and preserves its digest on an unchanged rerun
  - published mixed-horizon siblings reparent existing source changes, leave a valid strict graph, and prevent duplicate split-epic creation on replan
  - post-review tracker drift names the issue and writes nothing
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/issues/planning_transaction.rs
    action: modify
    section: logic
    impl_mode: handwrite
    description: "DDD transaction aggregate, snapshot preflight, minimal ordered mutation manifest, idempotent apply, checkpoint reconciliation, and read-only published-graph admission verification."
  - path: apps/agentic-workflow/src/cli/issues.rs
    action: modify
    section: cli
    impl_mode: handwrite
    description: "Bind project-plan review to the transaction manifest and apply it after accepted evidence."
  - path: apps/agentic-workflow/tests/wi_project_plan_transaction_cli_test.rs
    action: create
    section: e2e-test
    impl_mode: handwrite
    description: "Compiled CLI coverage for apply, no-op retry, and pre-write drift rejection."
```

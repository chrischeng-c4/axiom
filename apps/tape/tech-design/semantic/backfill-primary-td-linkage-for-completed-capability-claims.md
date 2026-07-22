---
id: '2157'
capability_refs:
  - id: "long-running-stability"
    role: primary
    gap: "repeated-raft-restart-endurance"
    claim: "repeated-raft-restart-endurance"
    coverage: full
    rationale: "Bind the completed repeated Raft restart endurance claim to primary TD verification without changing its existing runtime oracle."
  - id: "long-running-stability"
    role: primary
    gap: "bounded-http-replay-soak"
    claim: "bounded-http-replay-soak"
    coverage: full
    rationale: "Bind the completed bounded HTTP replay soak claim to primary TD verification without changing its existing runtime oracle."
  - id: "security-hardening"
    role: primary
    gap: "topic-replay-security-boundary"
    claim: "topic-replay-security-boundary"
    coverage: full
    rationale: "Bind the existing topic replay security boundary evidence to primary TD verification."
  - id: "security-hardening"
    role: primary
    gap: "opt-in-server-ingress-network-policy"
    claim: "opt-in-server-ingress-network-policy"
    coverage: full
    rationale: "Bind the existing opt-in ingress policy evidence to primary TD verification."
  - id: "subscription-delivery-resources"
    role: primary
    gap: "pull-subscription-cursor-contract"
    claim: "pull-subscription-cursor-contract"
    coverage: full
    rationale: "Bind the completed pull subscription cursor contract to primary TD verification."
  - id: "retention-and-backfill"
    role: primary
    gap: "retention-window-and-backfill-contract"
    claim: "retention-window-and-backfill-contract"
    coverage: full
    rationale: "Bind the completed retention and backfill contract to primary TD verification."
  - id: "http2-api-list"
    role: primary
    gap: "service-http-shell-h2c-serve-standard-endpoints"
    claim: "service-http-shell-h2c-serve-standard-endpoints"
    coverage: full
    rationale: "Bind the existing h2c HTTP shell and standard endpoints evidence to the HTTP/2 API capability."
  - id: "http2-api-list"
    role: primary
    gap: "backup-service-tls-spec-gen-clients"
    claim: "backup-service-tls-spec-gen-clients"
    coverage: full
    rationale: "Bind the existing backup, service TLS, spec generation, and client evidence to the HTTP/2 API capability."
  - id: "standard-operational-endpoints"
    role: primary
    gap: "service-http-shell-h2c-serve-standard-endpoints"
    claim: "service-http-shell-h2c-serve-standard-endpoints"
    coverage: full
    rationale: "Bind the same implemented service shell to the standard operational endpoint capability."
  - id: "observability"
    role: primary
    gap: "prometheus-operator-scrape-alert-component"
    claim: "prometheus-operator-scrape-alert-component"
    coverage: full
    rationale: "Bind the existing Prometheus operator scrape and alert component evidence to primary TD verification."
  - id: "ec-gates-configured"
    role: primary
    gap: "crate-smoke-gate"
    claim: "crate-smoke-gate"
    coverage: full
    rationale: "Bind the configured Tape crate smoke gate to primary TD verification."
  - id: "ec-gates-configured"
    role: primary
    gap: "tape-vat-meter-guard-ec-gates-observability"
    claim: "tape-vat-meter-guard-ec-gates-observability"
    coverage: full
    rationale: "Bind the configured Tape, VAT, Meter, Guard, and observability EC evidence to primary TD verification."
  - id: "ec-gates-configured"
    role: primary
    gap: "shared-otlp-trace-export"
    claim: "shared-otlp-trace-export"
    coverage: full
    rationale: "Bind the shared OTLP trace export gate to primary TD verification."
  - id: "kubernetes-native-deployment"
    role: primary
    gap: "operator-kind-pvc-restart-replay"
    claim: "operator-kind-pvc-restart-replay"
    coverage: full
    rationale: "Bind the existing operator, Kind, PVC restart, and replay evidence to primary TD verification."
  - id: "backup-restore"
    role: primary
    gap: "exact-journal-snapshot-backup"
    claim: "exact-journal-snapshot-backup"
    coverage: full
    rationale: "Bind the exact journal snapshot backup evidence to primary TD verification."
  - id: "backup-restore"
    role: primary
    gap: "fresh-pvc-cold-recovery-seed"
    claim: "fresh-pvc-cold-recovery-seed"
    coverage: full
    rationale: "Bind the fresh PVC cold recovery seed evidence to primary TD verification."
  - id: "replica-sync-bootstrap"
    role: primary
    gap: "raft-log-existing-pvc-sync"
    claim: "raft-log-existing-pvc-sync"
    coverage: full
    rationale: "Bind the existing-PVC Raft log synchronization evidence to primary TD verification."
  - id: "replica-sync-bootstrap"
    role: primary
    gap: "empty-pvc-external-backup-seed"
    claim: "empty-pvc-external-backup-seed"
    coverage: full
    rationale: "Bind the empty-PVC external backup seed evidence to primary TD verification."
  - id: "primary-replicas"
    role: primary
    gap: "raft-backed-replay-journal"
    claim: "raft-backed-replay-journal"
    coverage: full
    rationale: "Bind the Raft-backed replay journal evidence to primary TD verification."
summary: >
  Backfill primary and full TD verification linkage for 19 completed Tape
  capability claims while retaining their existing runtime oracles and closed
  implementation work as historical provenance.
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-completed-claim-primary-linkage
entry: inventory_claims
nodes:
  inventory_claims:
    kind: start
    label: "Load the exact 19 completed Tape capability and claim pairs"
  preserve_history:
    kind: process
    label: "Keep closed implementation work items as historical provenance"
  bind_refs:
    kind: process
    label: "Bind every pair to this TD with primary and full coverage"
  structural_test:
    kind: process
    label: "Check the exact ids, role, coverage, and reference count"
  run_existing_gates:
    kind: process
    label: "Run existing Tape claim oracles and configured runtime gates"
  complete:
    kind: decision
    label: "Does the capability goal advance through runtime verification?"
  revise_metadata:
    kind: process
    label: "Revise only linkage metadata or structural coverage"
  done:
    kind: terminal
    label: "Completed Tape claims have primary verification linkage"
edges:
  - { from: inventory_claims, to: preserve_history }
  - { from: preserve_history, to: bind_refs }
  - { from: bind_refs, to: structural_test }
  - { from: structural_test, to: run_existing_gates }
  - { from: run_existing_gates, to: complete }
  - { from: complete, to: done, label: "yes" }
  - { from: complete, to: revise_metadata, label: "no" }
  - { from: revise_metadata, to: structural_test }
---
flowchart TD
  inventory_claims([Load exact 19 completed claim pairs]) --> preserve_history[Preserve closed WI provenance]
  preserve_history --> bind_refs[Bind primary and full TD refs]
  bind_refs --> structural_test[Verify exact linkage inventory]
  structural_test --> run_existing_gates[Run existing Tape claim oracles and runtime gates]
  run_existing_gates --> complete{Capability goal reaches runtime verification?}
  complete -->|yes| done([Primary verification linkage complete])
  complete -->|no| revise_metadata[Revise linkage metadata only]
  revise_metadata --> structural_test
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/tests/capability_primary_linkage.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Add a deterministic structural regression test for the exact 19 capability refs, including primary role and full coverage. generator gap: missing-generator:test:capability-td-linkage (#2157)."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-completed-claim-primary-linkage-verification
requirements:
  capability_goal_advances:
    id: R5
    text: "The Tape capability root advances past linkage reconciliation and evaluates the existing runtime gates."
    kind: functional
    risk: high
    verify: aw goal capability --project tape --non-interactive
  exact_primary_full_refs:
    id: R1
    text: "The TD binds all 19 listed capability and claim pairs with primary role and full coverage."
    kind: functional
    risk: high
    verify: capability_primary_linkage::exact_primary_full_linkage_inventory_is_preserved
  existing_oracles_remain_authoritative:
    id: R3
    text: "Existing Tape claim oracles and configured gates remain authoritative without claim weakening or duplicate runtime behavior."
    kind: regression
    risk: high
    verify: aw capability check --project tape --skip-issue-inventory
  linkage_regression_is_deterministic:
    id: R4
    text: "A deterministic structural test fails when an expected capability id, claim id, role, coverage value, or total reference count changes."
    kind: regression
    risk: high
    verify: capability_primary_linkage::exact_primary_full_linkage_inventory_is_preserved
  preserve_historical_work:
    id: R2
    text: "The reconciliation changes only TD linkage metadata, its structural test, and the producer-owned TD lock; completed implementation work remains historical provenance."
    kind: regression
    risk: medium
    verify: capability_primary_linkage::reconciliation_scope_is_metadata_only
---
flowchart TD
    r1[R1 exact primary full refs] --> capability_primary_linkage_exact_primary_full_linkage_inventory_is_preserved[capability_primary_linkage::exact_primary_full_linkage_inventory_is_preserved]
    r4[R4 linkage regression is deterministic] --> capability_primary_linkage_exact_primary_full_linkage_inventory_is_preserved
    r2[R2 preserve historical work] --> capability_primary_linkage_reconciliation_scope_is_metadata_only[capability_primary_linkage::reconciliation_scope_is_metadata_only]
    r3[R3 existing oracles remain authoritative] --> aw_capability_check_project_tape_skip_issue_inventory[aw capability check --project tape --skip-issue-inventory]
    r5[R5 capability goal advances] --> aw_goal_capability_project_tape_non_interactive[aw goal capability --project tape --non-interactive]
```

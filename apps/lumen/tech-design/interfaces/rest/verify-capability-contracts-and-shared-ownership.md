---
id: '2324'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-capability-shared-ownership-verification
entry: load_contract
nodes:
  load_contract:
    kind: start
    label: "Load Lumen capability source, README summary, traits, and declared verification gates"
  derive_baseline:
    kind: process
    label: "Derive mandatory HTTP, Kubernetes, replica, CLI, security, and observability baselines from aw.toml traits"
  map_owners:
    kind: process
    label: "Map platform mechanisms to cli-std, service-http, service-auth, service-observability, service-k8s, raft-runtime, peer-tls, and rig/vat owners"
  inspect_seams:
    kind: process
    label: "Inspect Lumen-local adapters and retain only search policy, CRD policy, and thin shared-library integration"
  run_evidence:
    kind: process
    label: "Run deterministic ownership tests, capability verification, and full health verification with current evidence"
  clean:
    kind: decision
    label: "Are capability linkage and non-domain ownership complete?"
  classify:
    kind: decision
    label: "Is each failure shared or Lumen-domain?"
  repair_shared:
    kind: process
    label: "Repair the canonical shared owner or thin integration and rerun every affected gate"
  track_domain:
    kind: process
    label: "Link one bounded Lumen issue and record tracked_skip without implementing domain scope"
  pass:
    kind: terminal
    label: "Record passed evidence with exact commands and no unresolved shared gap"
  skipped:
    kind: terminal
    label: "Record tracked_skip with validated app-domain issue"
edges:
  - { from: load_contract, to: derive_baseline }
  - { from: derive_baseline, to: map_owners }
  - { from: map_owners, to: inspect_seams }
  - { from: inspect_seams, to: run_evidence }
  - { from: run_evidence, to: clean }
  - { from: clean, to: pass, label: "yes" }
  - { from: clean, to: classify, label: "no" }
  - { from: classify, to: repair_shared, label: "shared/non-domain" }
  - { from: repair_shared, to: run_evidence }
  - { from: classify, to: track_domain, label: "Lumen-domain" }
  - { from: track_domain, to: skipped }
---
flowchart TD
  load_contract([Load capability source and traits]) --> derive_baseline[Derive mandatory baseline capabilities]
  derive_baseline --> map_owners[Map mechanisms to canonical shared libraries]
  map_owners --> inspect_seams[Inspect Lumen-local integration seams]
  inspect_seams --> run_evidence[Run ownership, capability, and health gates]
  run_evidence --> clean{Shared ownership and linkage complete?}
  clean -->|yes| pass([Record passed evidence])
  clean -->|no| classify{Failure owner?}
  classify -->|shared/non-domain| repair_shared[Repair shared owner or thin integration]
  repair_shared --> run_evidence
  classify -->|Lumen-domain| track_domain[Link bounded issue and record tracked_skip]
  track_domain --> skipped([Record tracked skip])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/tests/capability_shared_ownership.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Add a deterministic structural regression gate that derives Lumen's required service baselines from aw.toml, requires the capability contract to name shared stateful composition, verifies cli-std/service-http/service-auth/service-k8s/raft-runtime/peer-tls delegation at the actual integration seams, and rejects app-local copies of tracing, admission, auth registry, Kubernetes render, or Raft host mechanisms. Search policy, Lumen CRD policy, and thin adapters remain app-owned. generator gap: missing-generator:test:capability-shared-ownership (#2324)."
```

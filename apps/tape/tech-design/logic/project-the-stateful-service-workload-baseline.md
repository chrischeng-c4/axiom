---
id: "1554"
summary: >
  WI #1554 adds the required stateful-service-workload baseline root to the
  Tape capability map. The root composes existing journal, raft, backup,
  security-boundary, and StatefulSet evidence without asserting that any
  unfinished domain capability is complete or adding runtime behavior.
fill_sections: [logic, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-stateful-service-workload-contract
entry: root
nodes:
  root:
    kind: start
    label: "Add one README capability root, stateful-service-workload, with WI #1554; it is a shared baseline projection only"
  journal:
    kind: process
    label: "Link Topic Replay Journal: durable append log and protected replay state"
  replicas:
    kind: process
    label: "Link Primary Replicas: raft leader/follower, stable identity, PVC-backed recovery, and failover evidence"
  backup:
    kind: process
    label: "Link HTTP/2 API List: admin-gated snapshot and service-backup destination path"
  deploy:
    kind: process
    label: "Link Kubernetes-Native Deployment: StatefulSet, probes, PDB, operator, and dockerfile render evidence"
  security:
    kind: process
    label: "Link Security Hardening as its own planned boundary; do not assert its unfinished authz/rotation behavior is complete"
  check:
    kind: terminal
    label: "aw capability check --project tape --skip-issue-inventory recognizes the stateful_storage trait baseline"
edges:
  - { from: root, to: journal }
  - { from: journal, to: replicas }
  - { from: replicas, to: backup }
  - { from: backup, to: deploy }
  - { from: deploy, to: security }
  - { from: security, to: check }
---
flowchart TD
    root[Add one README capability root, stateful-service-workload, with WI #1554; it is a shared baseline projection only] --> journal[Link Topic Replay Journal: durable append log and protected replay state]
    journal --> replicas[Link Primary Replicas: raft leader/follower, stable identity, PVC-backed recovery, and failover evidence]
    replicas --> backup[Link HTTP/2 API List: admin-gated snapshot and service-backup destination path]
    backup --> deploy[Link Kubernetes-Native Deployment: StatefulSet, probes, PDB, operator, and dockerfile render evidence]
    deploy --> security[Link Security Hardening as its own planned boundary; do not assert its unfinished authz/rotation behavior is complete]
    security --> check([aw capability check --project tape --skip-issue-inventory recognizes the stateful_storage trait baseline])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the stateful-service-workload capability index row and canonical root for WI #1554. The root links the existing Topic Replay Journal, Primary Replicas, HTTP/2 API List, Kubernetes-Native Deployment, and Security Hardening evidence; it must not claim unfinished retention, kind dogfood, peer-mTLS termination, or security hardening as implemented. generator gap: missing-generator:capability:stateful-service-workload (#1554)."
```

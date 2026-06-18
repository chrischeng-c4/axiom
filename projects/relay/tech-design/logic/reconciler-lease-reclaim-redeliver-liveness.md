---
id: relay-reconciler
summary: relay-side work-queue liveness — a background per-shard sweep that reclaims expired leases so a dead worker's in-flight range is redelivered (epoch-bumped to fence the old worker), never a full log scan. Standalone.
fill_sections: [logic, config, unit-test, changes]
---

# relay reconciler — lease reclaim / redeliver / liveness

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-reconciler-flow
entry: tick
nodes:
  tick:
    kind: start
    label: "Reconciler wakes every reconcile_interval_ms"
  per_subject:
    kind: process
    label: "For each (subject, shard): sweep ONLY the in-flight leases (the frontier), never a full log scan"
  expired:
    kind: decision
    label: "Lease expires_at <= now?"
  reclaim:
    kind: process
    label: "Delete the expired lease -> its entry becomes redelivery-eligible"
  keep:
    kind: terminal
    label: "Still within lease -> leave it (worker alive / heartbeating)"
  redeliver:
    kind: process
    label: "On the next lease(), the entry is re-offered with a bumped epoch (prefers redeliver)"
  fence:
    kind: terminal
    label: "Old worker's late ack/heartbeat (old lease_id / epoch) is a no-op — fenced; no work lost"
edges:
  - { from: tick, to: per_subject }
  - { from: per_subject, to: expired }
  - { from: expired, to: reclaim, label: "yes" }
  - { from: expired, to: keep, label: "no" }
  - { from: reclaim, to: redeliver }
  - { from: redeliver, to: fence }
---
flowchart TD
    tick([every reconcile_interval_ms]) --> per_subject[sweep in-flight leases per shard]
    per_subject --> expired{expired?}
    expired -->|yes| reclaim[delete lease -> redelivery-eligible]
    expired -->|no| keep([keep, worker alive])
    reclaim --> redeliver[next lease re-offers, epoch bumped]
    redeliver --> fence([old worker fenced; no work lost])
```
## Config
<!-- type: config lang: yaml -->

```yaml
# Reconciler — relay-side work-queue liveness. Extends RelayServerConfig (#115).
reconcile_interval_ms: 1000   # how often the background sweep reclaims expired leases per shard
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-reconciler-test-plan
entry: start
nodes:
  start:
    kind: start
    label: "pending"
edges: []
---
flowchart TD
    start([pending])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
(fill)
```

---
id: relay-work-queue-throughput
summary: Work-queue throughput rework — per-(subject,shard) locking for real consumer concurrency, an O(1) next-eligible lease cursor (redeliver min-heap + next_offer + committed watermark) replacing the O(n) scan, and batch lease/ack endpoints to amortize HTTP round-trips. Same exactly-once / epoch-fencing semantics. Standalone.
fill_sections: [logic, schema, rest-api, unit-test, changes]
---

# relay work-queue throughput — per-shard lock + O(1) lease cursor + batch lease/ack

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-work-queue-throughput-flow
entry: req
nodes:
  req:
    kind: start
    label: "Worker calls lease / ack (single or batch) on a subject"
  resolve:
    kind: process
    label: "Resolve (subject, shard) and lock ONLY that shard's mutex (other shards proceed concurrently)"
  which:
    kind: decision
    label: "lease or ack?"
  pick:
    kind: process
    label: "O(1) pick: pop the redeliver min-heap if non-empty (prefer redeliver), else take next_offer and increment it"
  grant:
    kind: process
    label: "Grant Lease(epoch = ++attempt); for lease-batch, repeat up to `max`"
  ack:
    kind: process
    label: "ack(lease_id, epoch): epoch-checked delete; insert into acked; for ack-batch, repeat over the list"
  watermark:
    kind: process
    label: "Advance the committed watermark incrementally (amortized O(1)) while acked contains the next seq"
  reclaim:
    kind: process
    label: "Reconciler: an expired lease pushes its seq back onto the redeliver heap (bumped epoch fences the old worker)"
  done:
    kind: terminal
    label: "Return grants / committed offset; lock released"
edges:
  - { from: req, to: resolve }
  - { from: resolve, to: which }
  - { from: which, to: pick, label: "lease" }
  - { from: pick, to: grant }
  - { from: grant, to: done }
  - { from: which, to: ack, label: "ack" }
  - { from: ack, to: watermark }
  - { from: watermark, to: done }
  - { from: reclaim, to: pick, label: "redelivery-eligible seq re-offered O(1)" }
---
flowchart TD
    req([lease / ack, single or batch]) --> resolve[lock only this shard]
    resolve --> which{lease or ack?}
    which -->|lease| pick[O(1): redeliver-heap else next_offer++]
    pick --> grant[grant epoch lease, up to max]
    grant --> done([return; unlock])
    which -->|ack| ack[epoch-checked delete + acked.insert]
    ack --> watermark[advance committed watermark amortized O(1)]
    watermark --> done
    reclaim[reconciler: expired -> push seq to redeliver heap] --> pick
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
(fill)
```

## Rest Api
<!-- type: rest-api lang: yaml -->

```yaml
(fill)
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-wqt-test-plan
entry: start
nodes:
  start: { kind: start, label: "pending" }
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

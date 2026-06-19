---
id: relay-bounded-ram-log
summary: Bounded-RAM durable log — keep only the most recent ram_ring_entries resident, evict older entries to disk and read them back via a dense byte-offset index; bound the dedupe map to dedupe.window_entries. The log scales far beyond memory while broadcast replay and reads still work. Standalone.
fill_sections: [logic, unit-test, changes]
---

# relay bounded-RAM durable log — entry eviction + offset index + disk-backed reads + dedupe window

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-bounded-ram-log-flow
entry: op
nodes:
  op:
    kind: start
    label: "append or read on the (subject, shard) log"
  which:
    kind: decision
    label: "append or read?"
  append:
    kind: process
    label: "append: record the entry's byte offset in offsets[seq], write its NDJSON line (group-commit fsync)"
  ring_evict:
    kind: process
    label: "Push the entry into the RAM ring; if the ring exceeds ram_ring_entries, evict the oldest (it stays on disk)"
  dedupe_evict:
    kind: process
    label: "Insert the id into the dedupe map; if it exceeds dedupe.window_entries, evict the oldest id (FIFO window)"
  read:
    kind: decision
    label: "read entry(seq) / range: is seq still resident in the RAM ring?"
  ram:
    kind: terminal
    label: "Serve from the RAM ring (hot path)"
  disk:
    kind: terminal
    label: "Seek to offsets[seq] and read the line back from disk (cold path) — replay from an evicted range still works"
edges:
  - { from: op, to: which }
  - { from: which, to: append, label: "append" }
  - { from: append, to: ring_evict }
  - { from: ring_evict, to: dedupe_evict }
  - { from: which, to: read, label: "read" }
  - { from: read, to: ram, label: "resident" }
  - { from: read, to: disk, label: "evicted" }
---
flowchart TD
    op([append / read]) --> which{append or read?}
    which -->|append| append[record offset + write line + fsync]
    append --> ring_evict[ring push; evict oldest beyond ram_ring_entries]
    ring_evict --> dedupe_evict[dedupe insert; evict oldest beyond window]
    which -->|read| read{seq resident in ring?}
    read -->|resident| ram([serve from RAM])
    read -->|evicted| disk([seek offsets and read from disk])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-bounded-ram-log-test-plan
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

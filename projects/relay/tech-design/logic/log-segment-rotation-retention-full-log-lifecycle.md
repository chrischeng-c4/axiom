---
id: relay-log-segments-retention
summary: Full log lifecycle — roll the NDJSON log into size-bounded segment files, locate any seq by its segment (offset is per-segment), and prune the oldest whole segments by total bytes or age, advancing start_seq. Reads and broadcast replay clamp to the earliest surviving seq. Standalone.
fill_sections: [logic, unit-test, changes]
---

# relay log segment rotation + retention (full log lifecycle)

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-log-segments-flow
entry: op
nodes:
  op:
    kind: start
    label: "append or read on the (subject, shard) log"
  which:
    kind: decision
    label: "append or read?"
  rotate:
    kind: decision
    label: "append: active segment bytes >= segment_bytes (and non-empty)?"
  newseg:
    kind: process
    label: "Roll: close the active segment, open a new one with base_seq = len"
  write:
    kind: process
    label: "Write the line to the active segment; offsets[seq] = byte offset WITHIN that segment"
  prune:
    kind: process
    label: "Retention: while total bytes > max_bytes_per_shard, or the oldest segment's newest entry is older than max_age_secs, delete the oldest WHOLE segment (never the active one) and advance start_seq to the next segment's base_seq"
  read:
    kind: decision
    label: "read seq: seq >= start_seq (still available)?"
  locate:
    kind: terminal
    label: "Locate the segment by base_seq (largest base_seq <= seq), seek offsets[seq], read; range walks segment runs in order"
  gone:
    kind: terminal
    label: "seq < start_seq -> pruned; entry None / range clamps from_seq up to start_seq"
edges:
  - { from: op, to: which }
  - { from: which, to: rotate, label: "append" }
  - { from: rotate, to: newseg, label: "full" }
  - { from: rotate, to: write, label: "room" }
  - { from: newseg, to: write }
  - { from: write, to: prune }
  - { from: which, to: read, label: "read" }
  - { from: read, to: locate, label: "available" }
  - { from: read, to: gone, label: "pruned" }
---
flowchart TD
    op([append / read]) --> which{append or read?}
    which -->|append| rotate{active >= segment_bytes?}
    rotate -->|full| newseg[roll: new segment base_seq=len]
    rotate -->|room| write[write to active segment; offset within segment]
    newseg --> write
    write --> prune[prune oldest segments by bytes/age; advance start_seq]
    which -->|read| read{seq >= start_seq?}
    read -->|available| locate([locate segment, seek, read; range walks runs])
    read -->|pruned| gone([None / clamp to start_seq])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-log-segments-test-plan
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

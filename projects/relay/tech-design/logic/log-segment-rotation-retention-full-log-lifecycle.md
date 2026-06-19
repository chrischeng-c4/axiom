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
entry: suite
nodes:
  suite: { kind: start, label: "segment rotation + retention tests" }
  t_roll: { kind: process, label: "tiny segment_bytes; append many" }
  a_roll: { kind: terminal, label: "assert multiple segment files exist; range(0) reads all in order across segments" }
  t_bytes: { kind: process, label: "small max_bytes_per_shard; keep appending" }
  a_bytes: { kind: terminal, label: "assert oldest segments deleted, start_seq advances, total bytes bounded" }
  t_pruned: { kind: process, label: "read a pruned seq and range from before start_seq" }
  a_pruned: { kind: terminal, label: "assert pruned entry() = None; range clamps to start_seq and returns surviving entries in order" }
  t_recover: { kind: process, label: "roll into N segments, drop, reopen" }
  a_recover: { kind: terminal, label: "assert all surviving segments replayed; len + reads correct" }
  t_unchanged: { kind: process, label: "default segment_bytes (huge), no retention: append N" }
  a_unchanged: { kind: terminal, label: "assert single segment, no pruning, behavior identical (benchmark unaffected)" }
edges:
  - { from: suite, to: t_roll }
  - { from: t_roll, to: a_roll }
  - { from: suite, to: t_bytes }
  - { from: t_bytes, to: a_bytes }
  - { from: suite, to: t_pruned }
  - { from: t_pruned, to: a_pruned }
  - { from: suite, to: t_recover }
  - { from: t_recover, to: a_recover }
  - { from: suite, to: t_unchanged }
  - { from: t_unchanged, to: a_unchanged }
---
flowchart TD
    suite([segment suite]) --> t_roll[tiny segment_bytes]
    t_roll --> a_roll([many segments; range ordered])
    suite --> t_bytes[small max_bytes]
    t_bytes --> a_bytes([oldest deleted; start_seq up])
    suite --> t_pruned[read pruned]
    t_pruned --> a_pruned([None; range clamps])
    suite --> t_recover[reopen N segments]
    t_recover --> a_recover([replayed correctly])
    suite --> t_unchanged[default sizes]
    t_unchanged --> a_unchanged([single segment, identical])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/relay/src/log.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Segment the NDJSON store: an ordered Vec<Segment{base_seq,path,bytes,last_ts}>; the active segment rolls at segment_bytes (base_seq = len). offsets[seq] becomes the byte offset within the seq's segment (segment located by base_seq). Retention prunes the oldest whole segments by max_bytes_per_shard / max_age_secs and advances start_seq. entry/range clamp to start_seq and read across segment runs. Recovery replays all surviving segments in order."
  - path: projects/relay/tests/segments.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Tests: rotation into multiple segment files + ordered range across them, byte-based pruning advancing start_seq, reads of pruned seqs (None / clamp), multi-segment recovery on reopen, and single-segment parity at default sizes."
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Active segment rolls at segment_bytes (base_seq=len); per-segment offsets located by base_seq; retention deletes oldest whole segments by bytes/age advancing start_seq; reads clamp to start_seq and walk segment runs. Full log lifecycle. Applicable.
- [unit-test] Rotation + ordered cross-segment range, byte pruning + start_seq, pruned reads, multi-segment recovery, default-size parity. Applicable.
- [changes] log.rs (segments + retention) + a new test; reuses segment_bytes / retention config. Applicable.

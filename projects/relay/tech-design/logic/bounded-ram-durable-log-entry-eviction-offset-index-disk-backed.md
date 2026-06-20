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
entry: suite
nodes:
  suite: { kind: start, label: "bounded-RAM log tests" }
  t_evict: { kind: process, label: "tiny ram_ring (e.g. 4); publish 20 to a disk log" }
  a_evict: { kind: terminal, label: "assert entry(seq) returns the correct payload for evicted AND resident seqs (disk-backed read)" }
  t_range: { kind: process, label: "broadcast subscribe from_seq=0 over a log with most entries evicted" }
  a_range: { kind: terminal, label: "assert all entries delivered in order (range reads cold prefix from disk + hot tail from RAM)" }
  t_dedupe: { kind: process, label: "dedupe window = small; publish more distinct ids than the window, then re-publish a still-in-window id and an evicted id" }
  a_dedupe: { kind: terminal, label: "assert in-window id dedupes (same seq); evicted id re-appends (new seq) — bounded at-least-once window" }
  t_unchanged: { kind: process, label: "ram_ring larger than N: publish N, read back" }
  a_unchanged: { kind: terminal, label: "assert nothing evicted; behavior identical to before (hot path)" }
edges:
  - { from: suite, to: t_evict }
  - { from: t_evict, to: a_evict }
  - { from: suite, to: t_range }
  - { from: t_range, to: a_range }
  - { from: suite, to: t_dedupe }
  - { from: t_dedupe, to: a_dedupe }
  - { from: suite, to: t_unchanged }
  - { from: t_unchanged, to: a_unchanged }
---
flowchart TD
    suite([bounded-RAM log suite]) --> t_evict[tiny ring, publish 20]
    t_evict --> a_evict([entry reads evicted from disk])
    suite --> t_range[subscribe from 0, mostly evicted]
    t_range --> a_range([all in order, cold+hot])
    suite --> t_dedupe[small dedupe window]
    t_dedupe --> a_dedupe([in-window dedupes; evicted re-appends])
    suite --> t_unchanged[ring > N]
    t_unchanged --> a_unchanged([no eviction, identical])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/relay/src/log.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Bound RAM: keep a VecDeque ring of the most recent ram_ring_entries plus a dense Vec<u64> byte-offset index; append records the offset and evicts the oldest ring entry; entry/range read evicted seqs from disk via the offset index (sequential read for the cold prefix of a range). Bound dedupe to window_entries with FIFO eviction. Replay on open rebuilds offsets + ring(last cap) + dedupe(window) + len + write_pos."
  - path: projects/relay/tests/bounded_log.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Tests: disk-backed entry() for evicted seqs, broadcast range/replay across the evict boundary, bounded dedupe window (in-window dedupes, evicted re-appends), and no-eviction parity when the ring exceeds N."
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Sound: a dense offset index + a bounded ring give RAM ~8 bytes/entry instead of a full entry; cold reads seek the offset and read the line; range reads the cold prefix sequentially then the hot tail; dedupe is a FIFO window. Replay rebuilds all of it. Scales beyond RAM without changing the hot path.
- [unit-test] Covers disk-backed reads for evicted seqs, replay across the evict boundary, the bounded dedupe window (in-window dedupes, evicted re-appends), and no-eviction parity.
- [changes] Bounded to log.rs + a new test; reuses existing config knobs.

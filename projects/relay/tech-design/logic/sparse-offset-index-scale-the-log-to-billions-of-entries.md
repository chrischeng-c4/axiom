---
id: relay-sparse-index
summary: Sparse byte-offset index — keep one offset every INDEX_STRIDE entries (plus a per-segment anchor) instead of one per seq, so the in-RAM index is ~len/STRIDE points. Cold reads seek the nearest indexed point in the seq's segment and scan forward. Standalone.
fill_sections: [logic, unit-test, changes]
---

# relay sparse offset index (scale the log to billions of entries)

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-sparse-index-flow
entry: op
nodes:
  op:
    kind: start
    label: "append or cold-read on a (subject, shard) log"
  which:
    kind: decision
    label: "append or read?"
  append:
    kind: process
    label: "append: index this entry only if it is a segment anchor (seq == base_seq) or (seq - base_seq) % INDEX_STRIDE == 0 -> push (seq, offset-within-segment) to the sorted sparse index"
  locate:
    kind: process
    label: "cold read(seq): find the nearest indexed point with index_seq <= seq in seq's segment (binary search)"
  seek:
    kind: process
    label: "Seek to that offset, then read_line forward (seq - index_seq) times to reach seq; range reads the rest of the run sequentially"
  prune:
    kind: process
    label: "On prune, drop index points with seq < start_seq; recovery rebuilds the sparse index from surviving segments"
  done:
    kind: terminal
    label: "Index is ~len / INDEX_STRIDE points (<< len) -> log scales to billions"
edges:
  - { from: op, to: which }
  - { from: which, to: append, label: "append" }
  - { from: append, to: done }
  - { from: which, to: locate, label: "read" }
  - { from: locate, to: seek }
  - { from: seek, to: done }
  - { from: append, to: prune, label: "retention" }
  - { from: prune, to: done }
---
flowchart TD
    op([append / cold-read]) --> which{append or read?}
    which -->|append| append[index only anchors + every STRIDE-th -> sparse]
    append --> done([index ~ len/STRIDE points])
    which -->|read| locate[nearest indexed point <= seq in segment]
    locate --> seek[seek + scan forward to seq]
    seek --> done
    append --> prune[prune: drop points < start_seq]
    prune --> done
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-sparse-index-test-plan
entry: suite
nodes:
  suite: { kind: start, label: "sparse index tests" }
  t_reads: { kind: process, label: "small stride + tiny ram_ring; publish many; read seqs at, before, after stride boundaries" }
  a_reads: { kind: terminal, label: "assert every entry()/range() returns the correct payload (seek + scan-forward)" }
  t_sparse: { kind: process, label: "publish N" }
  a_sparse: { kind: terminal, label: "assert the index holds ~N/stride points (<< N)" }
  t_seg: { kind: process, label: "tiny segment_bytes so reads cross segments" }
  a_seg: { kind: terminal, label: "assert cross-segment cold range is correct in order" }
  t_prune: { kind: process, label: "prune by bytes, then reopen" }
  a_prune: { kind: terminal, label: "assert surviving seqs read correctly via the rebuilt sparse index" }
edges:
  - { from: suite, to: t_reads }
  - { from: t_reads, to: a_reads }
  - { from: suite, to: t_sparse }
  - { from: t_sparse, to: a_sparse }
  - { from: suite, to: t_seg }
  - { from: t_seg, to: a_seg }
  - { from: suite, to: t_prune }
  - { from: t_prune, to: a_prune }
---
flowchart TD
    suite([sparse index suite]) --> t_reads[read at/before/after stride]
    t_reads --> a_reads([correct payloads])
    suite --> t_sparse[publish N]
    t_sparse --> a_sparse([index ~ N/stride])
    suite --> t_seg[cross-segment reads]
    t_seg --> a_seg([ordered + correct])
    suite --> t_prune[prune + reopen]
    t_prune --> a_prune([rebuilt index reads ok])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/relay/src/log.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Replace the dense offsets Vec with a sparse sorted index Vec<(Seq, u64)> (offset within segment), sampled every INDEX_STRIDE entries plus an anchor at each segment base_seq. read_disk_range locates the nearest indexed point <= the target in the same segment, seeks, and scans forward to reach it (then reads the run sequentially). Prune drops index points below start_seq; recovery rebuilds the sparse index. Add index_entries() for tests."
  - path: projects/relay/tests/sparse_index.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Tests: correct reads at/before/after stride boundaries, sparse index size ~ N/stride, cross-segment cold range, and correct reads after prune + reopen."
```

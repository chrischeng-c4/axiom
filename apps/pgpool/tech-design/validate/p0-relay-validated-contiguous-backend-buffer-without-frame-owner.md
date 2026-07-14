---
id: '1697'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-contiguous-validated-relay-prefix
entry: read
nodes:
  read: { kind: start, label: "Read backend bytes into FrameReader buffer" }
  scan: { kind: process, label: "Non-consumingly validate contiguous complete relay frames" }
  first_invalid: { kind: terminal, label: "Malformed first frame: send nothing and end backend leg" }
  incomplete: { kind: process, label: "Keep incomplete suffix buffered; select validated prefix only" }
  ready: { kind: process, label: "Stop selected prefix at first ReadyForQuery" }
  write: { kind: process, label: "write_all the borrowed contiguous prefix" }
  consume: { kind: process, label: "Advance reader exactly after successful write" }
  close_suffix: { kind: terminal, label: "After valid prefix before malformed suffix, end backend leg" }
  await_more: { kind: terminal, label: "Await more backend bytes" }
edges:
  - { from: read, to: scan }
  - { from: scan, to: first_invalid, label: "first frame malformed" }
  - { from: scan, to: incomplete, label: "valid prefix then incomplete suffix" }
  - { from: scan, to: ready, label: "valid prefix reaches ReadyForQuery" }
  - { from: scan, to: write, label: "valid complete prefix" }
  - { from: incomplete, to: write }
  - { from: ready, to: write }
  - { from: write, to: consume, label: "write succeeds" }
  - { from: consume, to: close_suffix, label: "malformed suffix recorded" }
  - { from: consume, to: await_more, label: "no ReadyForQuery" }
---
flowchart LR
  read([backend read]) --> scan[validate contiguous frames\nwithout consuming]
  scan -->|malformed first| reject([send nothing; close])
  scan -->|valid prefix| write[write_all borrowed prefix]
  write -->|success| consume[advance exactly prefix length]
  consume -->|incomplete suffix| wait([await next backend bytes])
  consume -->|ReadyForQuery| boundary([apply lease boundary])
  consume -->|malformed suffix| close([close after valid prefix])
```

### Invariants

- The scan uses the same declared-length bounds and frame-specific structural validation as `FrameReader::next_relay_frame_with_raw`; no buffer bytes are exposed to the writer until every frame in the selected prefix has been accepted.
- The selected prefix begins at the current reader offset, ends at the first incomplete frame, first `ReadyForQuery`, or before malformed input, and is written by the existing single contiguous `write_all` path. It never performs an additional read to enlarge a batch.
- During the asynchronous write, the prefix is borrowed immutably from the reader and the reader cannot be read, mutated, or scanned again. A successful write is followed by exactly one advance of the selected byte count; a failed write consumes nothing and ends the leg.
- A malformed first frame has a zero-length validated prefix and fails before any client write. A malformed suffix after a nonempty valid prefix preserves existing ordering: write and consume that prefix once, then terminate without forwarding the invalid bytes.
- `ReadyForQuery` status is committed with the successful consume, before the transaction handler observes the batch result. Therefore lease return/reset decisions remain after the exact response bytes have reached the client.
- This design does not use scatter-gather `writev` (#1637) and does not alter `TcpStream` split/reunite ownership (#1663).

### Error handling

I/O and zero-write failures retain the existing relay outcome: the transaction backend leg ends and the pool closes rather than reuses the stream. Parser errors on an empty prefix produce no client output. Parser errors after a valid prefix are represented in the scan result so the caller forwards only that validated prefix, consumes it after success, then closes. Incomplete suffixes are neither consumed nor written and are completed by the next backend read.

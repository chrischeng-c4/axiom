---
id: '1695'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-reset-reader-reuse
entry: transaction_ready
nodes:
  transaction_ready: { kind: start, label: "Transaction reader observed ReadyForQuery Idle" }
  drained: { kind: decision, label: "Reader buffer is empty at the ownership boundary" }
  transfer: { kind: process, label: "Reunite backend stream and transfer the same reader to reset" }
  reset: { kind: process, label: "Send static DISCARD ALL and validate its response with the transferred reader" }
  fallback: { kind: process, label: "Generic pool release creates its existing reset reader" }
  park: { kind: terminal, label: "Park only reset-clean backend in idle pool" }
  close: { kind: terminal, label: "Close stream on residual bytes or reset failure" }
edges:
  - { from: transaction_ready, to: drained }
  - { from: drained, to: transfer, label: "yes" }
  - { from: drained, to: close, label: "no" }
  - { from: transfer, to: reset }
  - { from: reset, to: park, label: "ReadyForQuery Idle" }
  - { from: reset, to: close, label: "EOF malformed timeout" }
  - { from: fallback, to: reset }
---
flowchart LR
  ready([transaction ReadyForQuery Idle]) --> drained{reader buffer drained?}
  drained -->|yes| transfer[transfer stream and same reader to reset]
  drained -->|no| close([close backend])
  transfer --> reset[DISCARD ALL with transferred reader]
  reset -->|valid Idle| park([park reset-clean backend])
  reset -->|EOF malformed timeout| close
  fallback[generic release] --> new_reader[existing fresh reset reader]
  new_reader --> reset
```

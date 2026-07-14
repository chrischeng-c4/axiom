---
id: '1681'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-readiness-gated-idle-liveness
entry: acquire_idle
nodes:
  acquire_idle: { kind: start, label: "Acquire pops one reset-clean idle backend." }
  readiness_gate: { kind: process, label: "Call TcpStream::try_io with READABLE interest. If runtime has no readable readiness, its syscall closure is not invoked." }
  peek: { kind: process, label: "Only on reported readability, issue one socket-level MSG_PEEK inside try_io." }
  result: { kind: decision, label: "Classify no-readiness/WouldBlock, queued bytes, EOF, or I/O error." }
  live: { kind: terminal, label: "Lease the unchanged stream and existing permit." }
  discard: { kind: process, label: "Drop dead stream and permit, notify waiters, and retry existing acquisition." }
edges:
  - { from: acquire_idle, to: readiness_gate }
  - { from: readiness_gate, to: result, label: "not ready: WouldBlock, no closure" }
  - { from: readiness_gate, to: peek, label: "readable" }
  - { from: peek, to: result }
  - { from: result, to: live, label: "WouldBlock or bytes > 0" }
  - { from: result, to: discard, label: "EOF or other error" }
---
flowchart LR
  acquire_idle([pop idle backend]) --> readiness_gate{Tokio READABLE ready?}
  readiness_gate -->|no: no syscall| live([reuse unchanged])
  readiness_gate -->|yes| peek[one MSG_PEEK]
  peek --> result{result}
  result -->|bytes| live
  result -->|EOF/error| discard[drop, notify, retry]
  result -->|stale WouldBlock| live
```

### Contract invariants

- The non-ready fast path allocates no timer and invokes no socket syscall; `try_io` returns `WouldBlock` before executing its closure.
- `MSG_PEEK` runs only under Tokio READABLE interest and never consumes protocol bytes. A stale readiness result that yields `WouldBlock` clears Tokio's stale read bit and remains a live socket.
- Zero bytes is EOF; any error other than `WouldBlock` is unsafe for reuse and follows the existing drop-and-retry disposition.
- Stream/permit ownership, reset-before-idle, wakeups, fresh-connect fallback, and acquire deadline stay unchanged.

### Error handling

A `WouldBlock` result can represent either no registered readability or a stale readability bit after the peek syscall. Both mean the descriptor has no observable EOF/error and is returned unchanged. `Ok(0)` and non-`WouldBlock` errors drop the idle tuple before the acquire loop proceeds, preserving the physical cap.

---
id: '1680'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-nonblocking-idle-liveness-peek
entry: acquire_idle
nodes:
  acquire_idle: { kind: start, label: "Acquire pops one reset-clean idle backend." }
  socket_peek: { kind: process, label: "Perform one synchronous socket-level MSG_PEEK on the already nonblocking Tokio TCP socket; it never consumes a byte or creates a timer." }
  result: { kind: decision, label: "Did the peek return WouldBlock, EOF, another error, or queued bytes?" }
  live_pending: { kind: process, label: "WouldBlock means no pending bytes and the backend is live." }
  live_queued: { kind: process, label: "Queued bytes remain in the socket because MSG_PEEK is non-consuming." }
  lease: { kind: terminal, label: "Move the unchanged stream and its permit to outstanding and return a reused lease." }
  discard: { kind: process, label: "EOF or an I/O error drops the stream and permit, notifies capacity waiters, and retries normal acquisition." }
edges:
  - { from: acquire_idle, to: socket_peek }
  - { from: socket_peek, to: result }
  - { from: result, to: live_pending, label: "WouldBlock" }
  - { from: result, to: live_queued, label: "one or more bytes" }
  - { from: live_pending, to: lease }
  - { from: live_queued, to: lease }
  - { from: result, to: discard, label: "EOF or other error" }
---
flowchart LR
    acquire_idle([pop idle backend]) --> socket_peek[nonblocking MSG_PEEK]
    socket_peek --> result{result}
    result -->|WouldBlock| live_pending[alive, no queued bytes]
    result -->|bytes| live_queued[bytes remain queued]
    live_pending --> lease([reuse unchanged stream])
    live_queued --> lease
    result -->|EOF or error| discard[drop, notify, retry]
```

### Contract invariants

- The liveness probe runs only while the stream is exclusively owned by the idle tuple; normal relay never races this peek.
- `WouldBlock` is the normal idle state and returns a live lease without scheduling or awaiting Tokio I/O.
- A successful peek is `MSG_PEEK`: it reads no protocol byte, so the next relay read observes the same PostgreSQL frame boundary.
- EOF and every read error retain the existing dead-idle disposition: stream and permit are dropped before the next acquisition attempt.
- This changes neither `DISCARD ALL` before idle admission nor semaphore/permit ownership, capacity wakeups, or acquire deadlines.

### Error handling

A closed peer returns zero bytes and is discarded. A non-`WouldBlock` read error is likewise treated as unsafe for reuse. If no idle tuple remains after a discard, acquisition follows the existing fresh-connect or saturated-capacity path; the new probe creates no timer, wakeup, or alternate scheduling path.

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: apps/pgpool/Cargo.toml
    action: modify
    section: pgpool-nonblocking-idle-liveness-peek
    impl_mode: hand-written
    reason: Declare the direct socket abstraction used to issue a safe non-consuming nonblocking peek.
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-nonblocking-idle-liveness-peek
    impl_mode: hand-written
    reason: Replace zero-timeout async liveness probing with socket-level MSG_PEEK classification while retaining idle ownership and retry semantics.
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-nonblocking-idle-liveness-peek
    impl_mode: hand-written
    reason: Prove no-byte idle reuse, EOF discard-and-retry, and preservation of readable bytes across the liveness probe.
```

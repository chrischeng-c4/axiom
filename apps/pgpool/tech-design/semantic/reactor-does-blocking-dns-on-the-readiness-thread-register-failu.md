---
id: '1881'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-reactor-cached-backend-address
entry: reactor_start
nodes:
  start: { kind: start, label: "Construct TransactionReactor on the Tokio caller thread." }
  resolve: { kind: process, label: "Resolve configured backend hostname once before spawning the readiness owner." }
  cache: { kind: process, label: "Pass the resolved SocketAddr into ReactorRuntime as process-lifetime cached state." }
  connect: { kind: process, label: "Reactor opens Mio TCP streams directly to the cached address." }
  register: { kind: decision, label: "Did poll registry registration succeed?" }
  recycle: { kind: process, label: "Return failed registration token to free_tokens." }
  done: { kind: terminal, label: "No DNS call occurs on reactor hot path and token capacity remains reusable." }
edges:
  - { from: start, to: resolve }
  - { from: resolve, to: cache }
  - { from: cache, to: connect }
  - { from: connect, to: register }
  - { from: register, to: recycle, label: "no" }
  - { from: register, to: done, label: "yes" }
  - { from: recycle, to: done }
---
flowchart TD
    start([Start reactor]) --> resolve[Resolve hostname before reactor thread]
    resolve --> cache[Cache SocketAddr for process lifetime]
    cache --> connect[Reactor connects cached address]
    connect --> register{register succeeds?}
    register -->|no| recycle[Return token to free list]
    register -->|yes| done([Readiness loop remains nonblocking])
    recycle --> done
```

Backend DNS is resolved once by `TransactionReactor::start` before its dedicated readiness thread is spawned. The address is cached for that reactor lifetime; a new handler/reactor creation refreshes it, while routine DNS TTL refresh is intentionally deferred. `open_backend` only dials the cached `SocketAddr`. If Mio registration fails after token allocation, the token is immediately returned to `free_tokens` because no readiness event could retain it.

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: apps/pgpool/src/pool/reactor/runtime.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: start
    reason: Resolve the backend endpoint before spawning the readiness thread, cache its SocketAddr, and recycle tokens after failed Mio registration.
```

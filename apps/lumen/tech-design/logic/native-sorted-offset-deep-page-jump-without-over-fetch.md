---
id: '1782'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: native-search-offset
entry: request
nodes:
  request: { kind: start, label: "Receive SearchRequest with offset, cursor, sort and limit" }
  conflict: { kind: decision, label: "cursor present and offset non-zero?" }
  reject: { kind: terminal, label: "400 pagination conflict" }
  routed: { kind: decision, label: "sharded router?" }
  scatter: { kind: process, label: "fetch shard candidates and apply offset after global merge" }
  local: { kind: process, label: "filter and rank/sort under engine read lock" }
  slice: { kind: process, label: "skip offset then return at most limit hits" }
  done: { kind: terminal, label: "SearchResponse contains only requested page" }
edges:
  - { from: request, to: conflict }
  - { from: conflict, to: reject, label: "yes" }
  - { from: conflict, to: routed, label: "no" }
  - { from: routed, to: scatter, label: "yes" }
  - { from: routed, to: local, label: "no" }
  - { from: scatter, to: slice }
  - { from: local, to: slice }
  - { from: slice, to: done }
---
flowchart TD
    request([SearchRequest offset/cursor/sort/limit]) --> conflict{cursor and non-zero offset?}
    conflict -->|yes| reject([400 pagination conflict])
    conflict -->|no| routed{sharded router?}
    routed -->|yes| scatter[global shard merge]
    routed -->|no| local[local filter + rank/sort]
    scatter --> slice[skip offset; take limit]
    local --> slice
    slice --> done([only requested page returned])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/types.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the default-zero offset request field and document cursor exclusivity."
  - path: apps/lumen/src/storage.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Apply native offset after filtering and sort/ranking; reject cursor plus offset and cover number/keyword/composite pages."
  - path: apps/lumen/src/routing.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Apply offset after the global in-process shard merge."
  - path: apps/lumen/src/routing_remote.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Fetch sufficient shard candidates and preserve global offset semantics across pods."
  - path: apps/lumen/src/spec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Publish the native random-page contract in offline query shapes."
  - path: apps/lumen/tests/spec_cli.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Lock OpenAPI and offline-spec offset metadata."
```

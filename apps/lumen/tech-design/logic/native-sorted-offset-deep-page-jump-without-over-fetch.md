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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: native-search-offset-verification
requirements:
  cursor_exclusivity:
    id: R2
    text: "A non-zero offset cannot be combined with an opaque cursor and returns a clear client error."
    kind: functional
    risk: medium
    verify: cargo test -p lumen storage::sorted_offset_tests -- --nocapture
  global_shard_offset:
    id: R3
    text: "Shard routing applies offset only after global ordering and exposes the canonical OpenAPI field."
    kind: integration
    risk: high
    verify: cargo test -p lumen routing -- --nocapture && cargo test -p lumen --test spec_cli -- --nocapture
  sorted_offset_slice:
    id: R1
    text: "Native offset returns the exact number, keyword, composite and score-ranked slice without returning preceding hits."
    kind: regression
    risk: high
    verify: cargo test -p lumen storage::sorted_offset_tests -- --nocapture
---
flowchart TD
    r1[R1 sorted offset slice] --> cargo_test_p_lumen_storage_sorted_offset_tests_nocapture[cargo test -p lumen storage::sorted_offset_tests -- --nocapture]
    r2[R2 cursor exclusivity] --> cargo_test_p_lumen_storage_sorted_offset_tests_nocapture
    r3[R3 global shard offset] --> cargo_test_p_lumen_routing_nocapture_cargo_test_p_lumen_test_spec_cli_nocapture[cargo test -p lumen routing -- --nocapture && cargo test -p lumen --test spec_cli -- --nocapture]
```

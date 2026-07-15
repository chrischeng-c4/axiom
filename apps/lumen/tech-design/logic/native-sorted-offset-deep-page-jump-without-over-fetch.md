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

---
id: '1642'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: opaque-key-admission-control
entry: request
nodes:
  request: { kind: start, label: "Receive request with caller-selected endpoint class and opaque key" }
  enabled: { kind: decision, label: "Admission policy configured for endpoint class?" }
  bypass: { kind: process, label: "Allow without allocating limiter state" }
  fingerprint: { kind: process, label: "Hash opaque key; never retain or emit raw input" }
  bucket: { kind: process, label: "Refill matching token bucket from monotonic caller time" }
  available: { kind: decision, label: "At least one token available?" }
  allow: { kind: process, label: "Consume token and emit typed allow decision" }
  deny: { kind: process, label: "Emit typed deny with bounded Retry-After" }
  bound: { kind: process, label: "Evict least-recently-observed buckets until max_keys is respected" }
  done: { kind: terminal, label: "Return allow or standard 429 response without exposing key bytes" }
edges:
  - { from: request, to: enabled }
  - { from: enabled, to: bypass, label: "no" }
  - { from: enabled, to: fingerprint, label: "yes" }
  - { from: fingerprint, to: bucket }
  - { from: bucket, to: available }
  - { from: available, to: allow, label: "yes" }
  - { from: available, to: deny, label: "no" }
  - { from: allow, to: bound }
  - { from: deny, to: bound }
  - { from: bypass, to: done }
  - { from: bound, to: done }
---
flowchart TD
    request([Request]) --> enabled{Policy configured?}
    enabled -->|no| bypass[Allow without state]
    enabled -->|yes| fingerprint[Hash opaque key]
    fingerprint --> bucket[Refill bucket]
    bucket --> available{Token available?}
    available -->|yes| allow[Consume and allow]
    available -->|no| deny[429 plus Retry-After]
    allow --> bound[Enforce max_keys]
    deny --> bound
    bypass --> done([Credential-free result])
    bound --> done
```

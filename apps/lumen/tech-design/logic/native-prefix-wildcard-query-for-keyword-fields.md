---
id: '1792'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: keyword-prefix-query
entry: request
nodes:
  request: { kind: start, label: "Evaluate prefix field/value query node" }
  nonempty: { kind: decision, label: "prefix value non-empty?" }
  type: { kind: decision, label: "target field is keyword?" }
  reject: { kind: terminal, label: "400 invalid prefix query" }
  terms: { kind: process, label: "Enumerate segment plus live-tail keyword dictionary in lexical order" }
  union: { kind: process, label: "Union postings while term starts_with prefix; subtract tombstones through live_terms" }
  compose: { kind: process, label: "Compose bitmap under boolean and sort planners" }
  done: { kind: terminal, label: "Return exact prefix matches" }
edges:
  - { from: request, to: nonempty }
  - { from: nonempty, to: reject, label: "no" }
  - { from: nonempty, to: type, label: "yes" }
  - { from: type, to: reject, label: "no" }
  - { from: type, to: terms, label: "yes" }
  - { from: terms, to: union }
  - { from: union, to: compose }
  - { from: compose, to: done }
---
flowchart TD
    request([prefix field/value]) --> nonempty{value non-empty?}
    nonempty -->|no| reject([400 invalid prefix])
    nonempty -->|yes| type{keyword field?}
    type -->|no| reject
    type -->|yes| terms[lexical live term dictionary]
    terms --> union[union starts_with postings]
    union --> compose[boolean + sort composition]
    compose --> done([exact prefix matches])
```

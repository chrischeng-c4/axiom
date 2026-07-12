---
id: mamba-strict-type-wire-jit-local-producer-provenance-exhaustively
summary: Exhaustive JIT-local raw-or-boxed Int producer ownership transitions.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-strict-type-jit-local-producer-provenance
entry: emit_local_producer
nodes:
  emit_local_producer: { kind: start, label: "typed JIT local producer" }
  classify: { kind: process, label: "select declared producer contract" }
  evaluate: { kind: process, label: "evaluate data and owner before destination transition" }
  boundary: { kind: decision, label: "internal or dynamic call boundary" }
  defer_boundary: { kind: process, label: "publish explicit deferred owner action for 1452" }
  merge: { kind: process, label: "merge data and owner as a paired phi" }
  commit: { kind: process, label: "commit exactly one companion transition" }
  done: { kind: terminal, label: "destination has deterministic provenance" }
edges:
  - { from: emit_local_producer, to: classify }
  - { from: classify, to: evaluate }
  - { from: evaluate, to: boundary }
  - { from: boundary, to: defer_boundary, label: "internal or dynamic call" }
  - { from: boundary, to: merge, label: "checked arithmetic or shift split" }
  - { from: boundary, to: commit, label: "ordinary local producer" }
  - { from: defer_boundary, to: commit }
  - { from: merge, to: commit }
  - { from: commit, to: done }
---
flowchart TD
    producer([typed JIT local producer]) --> contract[select declared producer contract]
    contract --> evaluate[evaluate data and owner]
    evaluate --> boundary{call or split edge?}
    boundary -- internal or dynamic call --> defer[defer explicit owner boundary to 1452]
    boundary -- checked or lshift split --> merge[phi data and owner together]
    boundary -- ordinary local producer --> commit[commit one companion transition]
    defer --> commit
    merge --> commit
    commit --> done([deterministic local provenance])
```

Every typed JIT producer must derive its owner from the producer contract or the storage/runtime sidecar after evaluating data, then make one destination transition. The old generic pre-evaluation write is removed. Raw and immortal values publish `None`; fresh results transfer their declared owner; borrowed/pass-through results retain only their declared source. Checked arithmetic and left shift propagate `[data, owner]` through both predecessors in the same order. Internal and dynamic calls record an explicit deferred boundary for #1452 and must not infer payload provenance.

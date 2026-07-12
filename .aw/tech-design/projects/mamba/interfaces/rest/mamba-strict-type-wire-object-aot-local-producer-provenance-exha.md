---
id: mamba-strict-type-wire-object-aot-local-producer-provenance-exhaustively
summary: Exhaustive Object/AOT raw-or-boxed Int producer provenance.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-strict-type-object-local-producer-provenance
entry: emit_object_local_producer
nodes:
  emit_object_local_producer: { kind: start, label: "typed Object/AOT local producer" }
  action: { kind: process, label: "look up canonical producer owner action" }
  evaluate: { kind: process, label: "evaluate data and owner source before transition" }
  split: { kind: decision, label: "checked arithmetic or shift split" }
  merge: { kind: process, label: "merge data and owner as paired phi values" }
  boundary: { kind: process, label: "publish explicit 1451 or 1452 boundary action" }
  commit: { kind: process, label: "commit exactly one companion transaction" }
  done: { kind: terminal, label: "Object local has deterministic provenance" }
edges:
  - { from: emit_object_local_producer, to: action }
  - { from: action, to: evaluate }
  - { from: evaluate, to: split }
  - { from: split, to: merge, label: "checked or lshift" }
  - { from: split, to: boundary, label: "call boundary" }
  - { from: split, to: commit, label: "ordinary local producer" }
  - { from: merge, to: commit }
  - { from: boundary, to: commit }
  - { from: commit, to: done }
---
flowchart TD
    producer([typed Object/AOT local producer]) --> action[canonical owner action]
    action --> evaluate[evaluate data and owner source]
    evaluate --> split{split or boundary?}
    split -- checked or lshift --> merge[paired data and owner phi]
    split -- 1451 or 1452 --> boundary[explicit deferred boundary]
    split -- ordinary local --> commit[one post-evaluation transaction]
    merge --> commit
    boundary --> commit
    commit --> done([deterministic provenance])
```

`mod.rs` consumes the same MIR producer metadata as JIT lowering. It never derives ownership from the physical data register: raw values and compile-time immortals publish `None`; fresh runtime results transfer the owner returned by the runtime sidecar; Copy and pass-through operations retain their named source companion; and unknown call ingress/egress remains an explicit #1451/#1452 boundary. Checked arithmetic and left shift form paired `[data, owner]` merge values on every predecessor before the shared post-evaluation transaction releases the replaced companion.

---
id: mamba-strict-type-preserve-per-producer-owner-source-contracts
summary: Per-producer raw-or-boxed Int ownership metadata for later compiler and runtime consumers.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-strict-type-per-producer-owner-source-logic
entry: start
nodes:
  start: { kind: start, label: "MIR producer has a raw-or-boxed Int result" }
  physical: { kind: process, label: "retain existing physical ABI analysis" }
  owner_kind: { kind: decision, label: "select exact owner source independently" }
  ownerless: { kind: process, label: "record ownerless or immortal companion action" }
  fresh: { kind: process, label: "record fresh-result companion action" }
  passthrough: { kind: process, label: "record argument pass-through companion action" }
  explicit_out: { kind: process, label: "record explicit owner-out companion action" }
  boundary: { kind: process, label: "record named #1451 or #1452 boundary action" }
  extern: { kind: process, label: "attach runtime extern Int projection metadata" }
  inventory: { kind: process, label: "require one action for every producer family" }
  verify: { kind: process, label: "run metadata and runtime-symbol unit tests" }
  done: { kind: terminal, label: "MIR owner-source contract ready for later consumers" }
edges:
  - { from: start, to: physical }
  - { from: physical, to: owner_kind }
  - { from: owner_kind, to: ownerless, label: "ownerless or immortal" }
  - { from: owner_kind, to: fresh, label: "fresh runtime result" }
  - { from: owner_kind, to: passthrough, label: "argument pass-through" }
  - { from: owner_kind, to: explicit_out, label: "explicit owner-out" }
  - { from: owner_kind, to: boundary, label: "call boundary" }
  - { from: ownerless, to: extern }
  - { from: fresh, to: extern }
  - { from: passthrough, to: extern }
  - { from: explicit_out, to: extern }
  - { from: boundary, to: extern }
  - { from: extern, to: inventory }
  - { from: inventory, to: verify }
  - { from: verify, to: done }
---
flowchart TD
    start([raw-or-boxed Int producer]) --> physical[retain physical ABI analysis]
    physical --> owner_kind{exact owner source}
    owner_kind -- ownerless or immortal --> ownerless[record ownerless companion action]
    owner_kind -- fresh runtime result --> fresh[record fresh-result companion action]
    owner_kind -- argument pass-through --> passthrough[record pass-through companion action]
    owner_kind -- explicit owner-out --> explicit_out[record owner-out companion action]
    owner_kind -- call boundary --> boundary[record #1451 or #1452 boundary action]
    ownerless --> extern[attach runtime extern metadata]
    fresh --> extern
    passthrough --> extern
    explicit_out --> extern
    boundary --> extern
    extern --> inventory[require exhaustive producer action]
    inventory --> verify[run metadata and runtime-symbol tests]
    verify --> done([MIR contract ready])
```

---
id: mamba-strict-type-make-companion-writes-post-evaluation-transact
summary: Evaluation-safe companion-owner transactions for raw-or-boxed Int Cranelift VRegs.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-strict-type-post-evaluation-companion-transaction
entry: evaluate
nodes:
  evaluate: { kind: start, label: "evaluate producer data and obtain owner input" }
  classify: { kind: decision, label: "ownerless, fresh, borrowed, or source companion" }
  self_alias: { kind: decision, label: "destination aliases source" }
  retain: { kind: process, label: "retain borrowed or source-companion input" }
  release: { kind: process, label: "release old destination companion exactly once" }
  install: { kind: process, label: "install new companion exactly once" }
  done: { kind: terminal, label: "post-evaluation transaction complete" }
edges:
  - { from: evaluate, to: classify }
  - { from: classify, to: self_alias, label: "source-companion" }
  - { from: classify, to: retain, label: "borrowed" }
  - { from: classify, to: release, label: "ownerless or fresh" }
  - { from: self_alias, to: done, label: "same vreg: no-op" }
  - { from: self_alias, to: retain, label: "distinct vregs" }
  - { from: retain, to: release }
  - { from: release, to: install }
  - { from: install, to: done }
---
flowchart TD
    evaluate([evaluate data and owner input]) --> classify{owner input}
    classify -- ownerless or fresh --> release[release old destination companion]
    classify -- borrowed --> retain[retain incoming owner]
    classify -- source companion --> self_alias{dest equals source}
    self_alias -- yes --> done([no-op])
    self_alias -- no --> retain
    retain --> release
    release --> install[install incoming owner]
    install --> done
```

The shared Cranelift companion API is a two-step contract: the producer evaluates
data and obtains its owner input before it calls the transaction, then the
transaction performs retain-if-borrowed, release-old, and install-new in that
order. `Fresh(Value)` transfers ownership without a retain; `Borrowed(Value)`
and `SourceCompanion(VReg)` retain before the old destination is released; and
`Ownerless` installs the canonical `None` companion. A self source/destination
alias is a no-op. The transaction never derives ownership from data bits, tags,
or semantic type.

`ProducerWrite` is replaced by an explicit incoming-owner form so no caller can
release a destination before it has evaluated the replacement data and owner.
`MoveOut` and cleanup remain transfer/teardown operations, not producer writes.
This slice changes only the shared `VarAlloc` transaction region and its
colocated structural tests. JIT/Object producer match arms, runtime sidecars,
argument ingress, and return transport remain deferred to #1461, #1462, #1463,
#1451, and #1452 respectively.

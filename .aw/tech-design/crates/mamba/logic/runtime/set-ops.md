---
id: set-ops
title: Set and FrozenSet Operations
crate: mamba
files:
  - crates/mamba/src/runtime/set_ops.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: ad1e799c4
---

# Set and FrozenSet Operations

Mamba sets are `RwLock<Vec<MbValue>>` (mutable) and frozensets are
`Vec<MbValue>` (immutable). Membership uses linear scan with `eq_py`
for value-equality (so `1 in {1.0}` is True). Trade-off vs hash set:
linear scan keeps the storage simple and avoids the dunder-driven
`__hash__` complexity that DictKey has, at the cost of O(n) per
operation. Performance is not the load-bearing concern for the
conformance work today; correctness of value-equality is.

Three load-bearing invariants:

1. **Membership uses `eq_py`, not pointer identity** — `eq_py` walks
   the Python value-equality rules (numeric coercion, str/bytes
   compare, dunder `__eq__` on Instance). Otherwise `1 in {1.0}` would
   be False, breaking arithmetic-set fixtures.
2. **Set / FrozenSet share a unified read path** — `mb_set_contains`,
   `mb_set_len`, `mb_set_union`, etc. match on both `ObjData::Set`
   (RwLock-locked) and `ObjData::FrozenSet` (immutable Vec). User
   code shouldn't have to know which it has.
3. **Mutating ops on FrozenSet fail loudly** — `mb_set_add`,
   `mb_set_remove`, `mb_set_discard`, `mb_set_pop`, `mb_set_clear` all
   only match `ObjData::Set`; FrozenSet falls through to the no-op /
   default branch. Per Python semantics this should raise
   `AttributeError`; a future patch will surface that.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: set-types
types:
  ObjDataSet:       { kind: struct, label: "ObjData::Set(RwLock<Vec<MbValue>>)" }
  ObjDataFrozenSet: { kind: struct, label: "ObjData::FrozenSet(Vec<MbValue>)" }
  MbValue:          { kind: struct }
  EqPy:             { kind: struct, label: "eq_py — Python value equality" }
  ClassModule:      { kind: struct, label: "class.rs (__eq__ on Instance)" }
edges:
  - { from: ObjDataSet,       to: MbValue, kind: references }
  - { from: ObjDataFrozenSet, to: MbValue, kind: references }
  - { from: ObjDataSet,       to: EqPy,    kind: references, label: "membership" }
  - { from: ObjDataFrozenSet, to: EqPy,    kind: references, label: "membership" }
  - { from: EqPy,             to: ClassModule, kind: references, label: "__eq__ dispatch" }
---
classDiagram
    class ObjDataSet
    class ObjDataFrozenSet
    class MbValue
    class EqPy
    class ClassModule
    ObjDataSet --> MbValue : RwLock<Vec>
    ObjDataFrozenSet --> MbValue : Vec
    ObjDataSet --> EqPy : membership
    ObjDataFrozenSet --> EqPy : membership
    EqPy --> ClassModule : __eq__
```

## Set shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "set-types"
$defs:
  MbSet:
    type: object
    description: "ObjData::Set(RwLock<Vec<MbValue>>)"
    properties:
      lock:
        type: array
        items: { x-rust-type: MbValue }
        description: "linear membership; insertion-order preserved"
    required: [lock]
  MbFrozenSet:
    type: object
    description: "ObjData::FrozenSet(Vec<MbValue>) — immutable"
    properties:
      items:
        type: array
        items: { x-rust-type: MbValue }
    required: [items]
```

## Membership and set-algebra logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: set-algebra
entry: enter
nodes:
  enter:        { kind: start,    label: "mb_set_* entry (contains / union / intersect / diff / symdiff / sub/super/disjoint)" }
  classify:     { kind: decision, label: "operation?" }
  contains:     { kind: process,  label: "lock.read; iter.any(|v| eq_py(v, elem))" }
  extract_a:    { kind: process,  label: "extract_set_items(a) — clones from Set lock or FrozenSet vec" }
  extract_b:    { kind: process,  label: "extract_set_items(b)" }
  do_union:     { kind: process,  label: "result = a; for e in b: if not eq_py(any of result): push" }
  do_intersect: { kind: process,  label: "result = []; for e in a: if eq_py(any of b): push" }
  do_diff:      { kind: process,  label: "result = []; for e in a: if not eq_py(any of b): push" }
  do_symdiff:   { kind: process,  label: "diff(a,b) ++ diff(b,a)" }
  do_sub:       { kind: process,  label: "for e in a: if not eq_py(any of b): false; else true" }
  do_super:     { kind: process,  label: "swap a/b; reuse subset" }
  do_disjoint:  { kind: process,  label: "for e in a: if eq_py(any of b): false; else true" }
  alloc:        { kind: process,  label: "MbObject::new_set(result) — fresh allocation" }
  done:         { kind: terminal, label: "return MbValue (set / bool)" }
edges:
  - { from: enter,        to: classify }
  - { from: classify,     to: contains,     label: "in / __contains__" }
  - { from: classify,     to: extract_a,    label: "binary algebra" }
  - { from: extract_a,    to: extract_b }
  - { from: extract_b,    to: do_union,     label: "union" }
  - { from: extract_b,    to: do_intersect, label: "intersection" }
  - { from: extract_b,    to: do_diff,      label: "difference" }
  - { from: extract_b,    to: do_symdiff,   label: "symmetric_difference" }
  - { from: extract_b,    to: do_sub,       label: "issubset" }
  - { from: extract_b,    to: do_super,     label: "issuperset" }
  - { from: extract_b,    to: do_disjoint,  label: "isdisjoint" }
  - { from: do_union,     to: alloc }
  - { from: do_intersect, to: alloc }
  - { from: do_diff,      to: alloc }
  - { from: do_symdiff,   to: alloc }
  - { from: alloc,        to: done }
  - { from: contains,     to: done }
  - { from: do_sub,       to: done }
  - { from: do_super,     to: done }
  - { from: do_disjoint,  to: done }
---
flowchart TD
    enter([mb_set_*]) --> classify{op?}
    classify -->|in / contains| contains[lock.read; eq_py any]
    classify -->|binary algebra| extract_a[extract a]
    extract_a --> extract_b[extract b]
    extract_b -->|union| do_union[walk b; push if not eq_py any of result]
    extract_b -->|intersection| do_intersect[walk a; push if eq_py any of b]
    extract_b -->|difference| do_diff[walk a; push if not eq_py any of b]
    extract_b -->|sym diff| do_symdiff[diff a,b ++ diff b,a]
    extract_b -->|issubset| do_sub[bool: all a in b]
    extract_b -->|issuperset| do_super[swap; subset]
    extract_b -->|isdisjoint| do_disjoint[bool: no shared]
    do_union --> alloc[new_set]
    do_intersect --> alloc
    do_diff --> alloc
    do_symdiff --> alloc
    alloc --> done([result])
    contains --> done
    do_sub --> done
    do_super --> done
    do_disjoint --> done
```

## Membership interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: set-membership
actors:
  - { id: JIT,     kind: system }
  - { id: Set,     kind: system, label: "set_ops.rs" }
  - { id: Storage, kind: system, label: "RwLock<Vec> | Vec" }
  - { id: EqPy,    kind: system, label: "eq_py" }
messages:
  - { from: JIT,     to: Set,     name: "elem in set / mb_set_contains" }
  - { from: Set,     to: Storage, name: "lock.read (Set) | direct Vec (FrozenSet)" }
  - { from: Storage, to: Set,     name: items_iter }
  - { from: Set,     to: EqPy,    name: "for v in items: eq_py(v, elem)" }
  - { from: EqPy,    to: Set,     name: bool, returns: bool }
  - { from: Set,     to: JIT,     name: "true on first match | false after exhaust" }
---
sequenceDiagram
    participant JIT
    participant Set
    participant Storage
    participant EqPy
    JIT->>Set: elem in set
    Set->>Storage: lock.read or Vec
    Storage-->>Set: items_iter
    Set->>EqPy: per-item eq_py(v, elem)
    EqPy-->>Set: bool
    Set-->>JIT: any-match
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: set-algebra
    given: set_methods/operations_broad.py combines overlapping sets
    when: union, intersection, difference, and symmetric difference run
    then: extract and filter logic returns CPython-compatible fresh set values
  - id: value-equality-membership
    given: set_methods/value_eq_membership.py checks numeric-equivalent values
    when: membership scans compare elements
    then: eq_py makes 1 in {1.0} and True in {1} match Python value equality
  - id: frozenset-read-path
    given: set_methods/frozen_basics.py creates frozensets
    when: membership and set algebra run
    then: FrozenSet uses the shared immutable read path
  - id: instance-membership
    given: class_system/instance_set_membership.py stores a user-class instance
    when: a value-equal instance is checked for membership
    then: eq_py dispatches __eq__ on Instance and returns true
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: runtime-set-ops-test-plan
title: Set and FrozenSet Operations Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> Basics["set_methods/operations_broad.py"]
    Runner --> Predicates["set_methods/predicates.py"]
    Runner --> ValueEq["set_methods/value_eq_membership.py"]
    Runner --> Frozen["set_methods/frozen_basics.py"]
    Runner --> Instance["class_system/instance_set_membership.py"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/set_ops.rs
    action: modify
    impl_mode: hand-written
    description: "RwLock<Vec<MbValue>>-backed Set + immutable FrozenSet, linear-scan membership via eq_py, set algebra returns fresh allocations. Hand-written; storage is intentionally simple — perf is not load-bearing today."
```

---
id: tuple-ops
title: Tuple Operations
crate: mamba
files:
  - crates/mamba/src/runtime/tuple_ops.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: ad1e799c4
---

# Tuple Operations

Mamba tuples store `Vec<MbValue>` (no lock, no mutation primitives —
tuples are immutable). The runtime exposes the CPython-equivalent
read-only surface (`tup[i]` / slicing / `+` / `*` / `count` / `index`
/ equality), unpacking, plus some structural helpers used internally by
return-multiple-values lowering.

Three load-bearing invariants:

1. **Tuple equality is element-wise via `eq_py`** — same value-equality
   walk as set membership; `(1, 1.0) == (1.0, 1)` is True. Pointer
   equality alone would break arithmetic-tuple fixtures.
2. **`tup[i]` returns a *single* element; `tup[a:b:c]` returns a tuple**
   — `mb_tuple_getitem` dispatches by index type. Slice support
   matters for unpacking patterns like `*head, last = tup`.
3. **Tuple-return-unpack vs `set_current_exception`** — the JIT lowers
   `a, b = f()` as `tmp = f(); a = tmp[0]; b = tmp[1]`. A spurious
   `set_current_exception` from inside the tuple subscripting (commit
   `b34d575aa`) used to leak ValueError; the fix is now a precondition
   for any clean tuple-unpack fixture.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: tuple-types
types:
  ObjDataTuple:  { kind: struct, label: "ObjData::Tuple(Vec<MbValue>)" }
  MbValue:       { kind: struct }
  EqPy:          { kind: struct, label: "eq_py from runtime::set_ops (shared)" }
  ExceptionMod:  { kind: struct, label: "exception.rs (IndexError on subscript out-of-range)" }
  IterModule:    { kind: struct, label: "iter.rs (Tuple has dedicated IterKind variant)" }
  Slice:         { kind: struct, label: "Tuple(start, stop, step)" }
edges:
  - { from: ObjDataTuple, to: MbValue,      kind: references, label: "Vec elements (immutable)" }
  - { from: ObjDataTuple, to: EqPy,         kind: references, label: "element-wise compare" }
  - { from: ObjDataTuple, to: ExceptionMod, kind: references, label: "IndexError" }
  - { from: ObjDataTuple, to: IterModule,   kind: references, label: "iter wrap" }
  - { from: ObjDataTuple, to: Slice,        kind: references, label: "tup[a:b:c]" }
---
classDiagram
    class ObjDataTuple
    class MbValue
    class EqPy
    class ExceptionMod
    class IterModule
    class Slice
    ObjDataTuple --> MbValue : Vec
    ObjDataTuple --> EqPy : element-wise
    ObjDataTuple --> ExceptionMod : IndexError
    ObjDataTuple --> IterModule : iter wrap
    ObjDataTuple --> Slice : tup[a:b:c]
```

## Tuple shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "tuple-types"
$defs:
  MbTuple:
    type: object
    description: "ObjData::Tuple(Vec<MbValue>) — immutable"
    properties:
      items:
        type: array
        items: { x-rust-type: MbValue }
    required: [items]
  TupleSubscript:
    description: "Index value passed to mb_tuple_getitem"
    oneOf:
      - { title: Int,   x-rust-type: i64,    description: "single-element index; negative wraps from end" }
      - { title: Slice, properties: { start: { x-rust-type: MbValue }, stop: { x-rust-type: MbValue }, step: { x-rust-type: MbValue } } }
```

## Subscript / equality logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tuple-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "mb_tuple_getitem(tup, index)" }
  is_int:       { kind: decision, label: "index is INT?" }
  range_check:  { kind: decision, label: "0 <= i < len after negative-wrap?" }
  read_item:    { kind: process,  label: "items[i]; retain_if_ptr; return" }
  raise_idx:    { kind: process,  label: "raise IndexError" }
  is_slice:     { kind: decision, label: "index is Slice triple?" }
  build_slice:  { kind: process,  label: "compute (start, stop, step); collect items[start..stop step]" }
  alloc_tuple:  { kind: process,  label: "MbObject::new_tuple(slice_items) — fresh allocation, retains" }
  bad_index:    { kind: terminal, label: "TypeError: tuple indices must be integers or slices" }
  done:         { kind: terminal, label: "return value | tuple" }
edges:
  - { from: enter,        to: is_int }
  - { from: is_int,       to: range_check, label: "yes" }
  - { from: is_int,       to: is_slice,    label: "no" }
  - { from: range_check,  to: read_item,   label: "in range" }
  - { from: range_check,  to: raise_idx,   label: "out of range" }
  - { from: is_slice,     to: build_slice, label: "yes" }
  - { from: is_slice,     to: bad_index,   label: "neither" }
  - { from: build_slice,  to: alloc_tuple }
  - { from: read_item,    to: done }
  - { from: raise_idx,    to: done }
  - { from: alloc_tuple,  to: done }
---
flowchart TD
    enter([mb_tuple_getitem]) --> is_int{INT?}
    is_int -->|yes| range_check{in range?}
    is_int -->|no| is_slice{slice triple?}
    range_check -->|yes| read_item[items i; retain]
    range_check -->|no| raise_idx[IndexError]
    is_slice -->|yes| build_slice[start..stop step]
    is_slice -->|no| bad_index([TypeError])
    build_slice --> alloc_tuple[new_tuple]
    read_item --> done([value or tuple])
    raise_idx --> done
    alloc_tuple --> done
```

## Return-unpack interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: tuple-return-unpack
actors:
  - { id: JIT,    kind: system }
  - { id: Caller, kind: system, label: "user fn returning a tuple" }
  - { id: Tuple,  kind: system, label: "tuple_ops.rs" }
  - { id: Excs,   kind: system, label: "exception.rs (CURRENT_EXCEPTION slot)" }
messages:
  - { from: JIT,    to: Caller, name: "f()" }
  - { from: Caller, to: JIT,    name: "tuple value tmp", returns: MbValue }
  - { from: JIT,    to: Tuple,  name: "mb_tuple_getitem(tmp, 0)" }
  - { from: Tuple,  to: Excs,   name: "no spurious set_current_exception (fix b34d575aa)" }
  - { from: Tuple,  to: JIT,    name: "items[0]; retain", returns: MbValue }
  - { from: JIT,    to: JIT,    name: "a = items[0]" }
  - { from: JIT,    to: Tuple,  name: "mb_tuple_getitem(tmp, 1)" }
  - { from: Tuple,  to: JIT,    name: "items[1]; retain", returns: MbValue }
  - { from: JIT,    to: JIT,    name: "b = items[1]" }
---
sequenceDiagram
    participant JIT
    participant Caller
    participant Tuple
    participant Excs
    JIT->>Caller: f()
    Caller-->>JIT: tmp
    JIT->>Tuple: getitem 0
    Tuple->>Excs: no spurious set_current_exception
    Tuple-->>JIT: items[0]
    JIT->>Tuple: getitem 1
    Tuple-->>JIT: items[1]
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: tuple-access
    given: tuple_methods/access_broad.py exercises indexing, count, index, and negative wrap
    when: tuple operations read elements
    then: integer subscripts range-check and count/index use eq_py
  - id: tuple-from-iterable
    given: tuple_methods/tuple_from_iterable.py converts list, range, and generator inputs
    when: tuple(iterable) drains the source
    then: the runtime allocates a fresh immutable tuple with retained elements
  - id: tuple-return-unpack
    given: bugs/tuple_return_double_call_unpack.py unpacks a function-returned tuple
    when: subsequent calls execute after tuple subscripting
    then: no spurious ValueError remains in CURRENT_EXCEPTION
  - id: tuple-eq-concat-repeat
    given: tuple_methods/eq_concat_repeat.py uses tuple addition, repetition, and equality
    when: runtime operations combine or compare tuples
    then: concat and repeat allocate fresh tuples and equality is element-wise
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: runtime-tuple-ops-test-plan
title: Tuple Operations Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> Access["tuple_methods/access_broad.py"]
    Runner --> FromIterable["tuple_methods/tuple_from_iterable.py"]
    Runner --> UnpackBug["bugs/tuple_return_double_call_unpack.py"]
    Runner --> EqConcat["tuple_methods/eq_concat_repeat.py"]
    Runner --> StarUnpack["star_unpacking/star_unpack_basic.py"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/tuple_ops.rs
    action: modify
    impl_mode: hand-written
    description: "Vec<MbValue>-backed immutable tuple, subscript with int / slice, count / index via eq_py, concat / repeat allocate fresh tuples. Hand-written; subscript MUST not set CURRENT_EXCEPTION on success path (commit b34d575aa)."
```

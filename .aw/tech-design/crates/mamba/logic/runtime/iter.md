---
id: iter
title: Iteration Protocol
crate: mamba
files:
  - crates/mamba/src/runtime/iter.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: dbbaf7396
---

# Iteration Protocol

Mamba's iterator subsystem. Promotes any iterable (built-in container,
generator handle, or user-class instance with `__iter__`) to a uniform
iterator handle, then advances it with a single `mb_has_next` /
`mb_next` protocol that the JIT emits for every for-loop. Two
substrate-specific concerns — StopIteration's flag/exception duality and
the `peeked` cache — are this spec's load-bearing invariants.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: iter-types
types:
  MbIterator:        { kind: struct }
  IterKind:          { kind: enum }
  MbValue:           { kind: struct }
  ObjData:           { kind: enum }
  GeneratorRegistry: { kind: struct }
  IteratorRegistry:  { kind: struct }
edges:
  - { from: MbIterator,        to: IterKind, kind: owns,       label: "kind" }
  - { from: IterKind,          to: MbValue,  kind: references, label: "List/Tuple/UserDefined/Generator/Callable" }
  - { from: IteratorRegistry,  to: MbIterator, kind: owns,     label: "thread_local HashMap<u64, MbIterator>" }
  - { from: IterKind,          to: GeneratorRegistry, kind: references, label: "Generator variant only" }
  - { from: MbIterator,        to: MbValue,  kind: references, label: "peeked" }
---
classDiagram
    class MbIterator
    class IterKind
    class MbValue
    class ObjData
    class GeneratorRegistry
    class IteratorRegistry
    MbIterator --> IterKind : owns
    IterKind --> MbValue : references
    IteratorRegistry --> MbIterator : owns
    IterKind --> GeneratorRegistry : references
    MbIterator --> MbValue : peeked
```

## Iterator state shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "iter-types"
$defs:
  MbIterator:
    type: object
    x-rust-type: MbIterator
    properties:
      kind:      { $ref: "#/$defs/IterKind" }
      index:     { type: integer, minimum: 0, x-rust-type: usize, description: "monotonic index for index-based kinds" }
      exhausted: { type: boolean, description: "sticky once set" }
      peeked:
        oneOf:
          - { type: "null" }
          - { x-rust-type: MbValue }
        description: "pre-fetched value from has_next"
    required: [kind, index, exhausted, peeked]
  IterKind:
    oneOf:
      - { title: List,        type: object, properties: { value: { x-rust-type: MbValue } }, description: "retained ptr to ObjData::List" }
      - { title: Tuple,       type: object, properties: { value: { x-rust-type: MbValue } }, description: "retained ptr to ObjData::Tuple" }
      - { title: Str,         type: object, properties: { chars: { type: array, items: { x-rust-type: char } } }, description: "materialized chars" }
      - { title: DictKeys,    type: object, properties: { keys:  { type: array, items: { type: string } } }, description: "snapshotted keys" }
      - { title: Range,       type: object, properties: { current: { type: integer, x-rust-type: i64 }, stop: { type: integer, x-rust-type: i64 }, step: { type: integer, x-rust-type: i64 } } }
      - { title: Enumerate,   type: object, properties: { inner: { $ref: "#/$defs/MbIterator" }, count: { type: integer, x-rust-type: i64 } } }
      - { title: Zip,         type: object, properties: { iters: { type: array, items: { $ref: "#/$defs/MbIterator" } } } }
      - { title: Map,         type: object, properties: { func: { x-rust-type: MbValue }, inner: { $ref: "#/$defs/MbIterator" } } }
      - { title: Filter,      type: object, properties: { func: { x-rust-type: MbValue }, inner: { $ref: "#/$defs/MbIterator" } } }
      - { title: Reversed,    type: object, properties: { items: { type: array, items: { x-rust-type: MbValue } }, index: { type: integer, x-rust-type: usize } } }
      - { title: UserDefined, type: object, properties: { instance: { x-rust-type: MbValue } }, description: "instance with __next__ dunder" }
      - { title: Generator,   type: object, properties: { handle: { x-rust-type: MbValue } }, description: "wraps a generator handle" }
      - { title: Callable,    type: object, properties: { func: { x-rust-type: MbValue }, sentinel: { x-rust-type: MbValue } }, description: "iter(callable, sentinel) per PEP 234" }
```

## Iterator lifecycle
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: iter-lifecycle
initial: New
nodes:
  New:       { kind: initial,   label: "alloc_iter_id; insert into ITERATORS" }
  Active:    { kind: normal,    label: "Active (kind ready to advance)" }
  Peeked:    { kind: normal,    label: "Peeked (mb_has_next cached value)" }
  Exhausted: { kind: terminal,  label: "Exhausted (sticky)" }
  Released:  { kind: terminal,  label: "Released (entry dropped)" }
edges:
  - { from: New,       to: Active,    event: register }
  - { from: Active,    to: Peeked,    event: mb_has_next, guard: "advance succeeds" }
  - { from: Active,    to: Exhausted, event: mb_has_next, guard: "advance signals StopIteration; clear exception slot if user-defined" }
  - { from: Peeked,    to: Active,    event: mb_next, guard: "consume peeked value" }
  - { from: Active,    to: Exhausted, event: mb_next, guard: "advance signals StopIteration" }
  - { from: Active,    to: Released,  event: mb_iter_release }
  - { from: Peeked,    to: Released,  event: mb_iter_release }
  - { from: Exhausted, to: Released,  event: mb_iter_release }
---
stateDiagram-v2
    [*] --> New
    New --> Active: register
    Active --> Peeked: mb_has_next [advance ok]
    Active --> Exhausted: mb_has_next [StopIteration]
    Peeked --> Active: mb_next [consume]
    Active --> Exhausted: mb_next [StopIteration]
    Active --> Released: mb_iter_release
    Peeked --> Released: mb_iter_release
    Exhausted --> Released: mb_iter_release
    Released --> [*]
```

## Promote and advance dispatch
<!-- type: logic lang: mermaid -->

```mermaid
---
id: iter-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "mb_iter(obj) | mb_next(handle)" }
  is_iter:      { kind: decision, label: "obj is live iterator handle?" }
  return_obj:   { kind: terminal, label: "return obj (collapse iter(iter(x)))" }
  is_gen:       { kind: decision, label: "obj is generator handle?" }
  wrap_gen:     { kind: process,  label: "wrap as IterKind::Generator; register" }
  has_ptr:      { kind: decision, label: "obj.as_ptr matches container?" }
  promote_list: { kind: process,  label: "List/Tuple: retain; IterKind::List/Tuple" }
  promote_str:  { kind: process,  label: "Str/DictKeys/Set/FrozenSet/Bytes: snapshot" }
  is_instance:  { kind: decision, label: "ObjData::Instance?" }
  call_iter:    { kind: process,  label: "lookup + call __iter__(self)" }
  iter_result:  { kind: decision, label: "__iter__ returned what?" }
  use_known:    { kind: process,  label: "known iter handle: return as-is" }
  wrap_user:    { kind: process,  label: "other ptr: retain; IterKind::UserDefined" }
  type_error:   { kind: terminal, label: "TypeError; return none" }
  next_kind:    { kind: decision, label: "advance dispatch by kind" }
  adv_callable: { kind: process,  label: "advance_callable_if_applicable" }
  adv_gen:      { kind: process,  label: "advance_generator_if_applicable" }
  adv_user:     { kind: process,  label: "advance_userdefined_if_applicable; clear exception slot on stop" }
  adv_inline:   { kind: process,  label: "advance_iter (inline; List/Tuple/Str/DictKeys/Range/Enumerate/Zip/Map/Filter/Reversed)" }
  done:         { kind: terminal, label: "return value | none" }
edges:
  - { from: enter,        to: is_iter }
  - { from: is_iter,      to: return_obj,   label: "yes" }
  - { from: is_iter,      to: is_gen,       label: "no" }
  - { from: is_gen,       to: wrap_gen,     label: "yes" }
  - { from: is_gen,       to: has_ptr,      label: "no" }
  - { from: has_ptr,      to: promote_list, label: "List/Tuple" }
  - { from: has_ptr,      to: promote_str,  label: "Str/Dict/Set/FrozenSet/Bytes" }
  - { from: has_ptr,      to: is_instance,  label: "Instance" }
  - { from: has_ptr,      to: type_error,   label: "BigInt/Complex/CodeObject/primitive" }
  - { from: is_instance,  to: call_iter,    label: "yes" }
  - { from: call_iter,    to: iter_result }
  - { from: iter_result,  to: use_known,    label: "known iter handle | gen handle" }
  - { from: iter_result,  to: wrap_user,    label: "other" }
  - { from: iter_result,  to: type_error,   label: "no __iter__" }
  - { from: promote_list, to: next_kind }
  - { from: promote_str,  to: next_kind }
  - { from: wrap_gen,     to: next_kind }
  - { from: wrap_user,    to: next_kind }
  - { from: use_known,    to: next_kind }
  - { from: next_kind,    to: adv_callable, label: "Callable" }
  - { from: next_kind,    to: adv_gen,      label: "Generator" }
  - { from: next_kind,    to: adv_user,     label: "UserDefined" }
  - { from: next_kind,    to: adv_inline,   label: "built-in" }
  - { from: adv_callable, to: done }
  - { from: adv_gen,      to: done }
  - { from: adv_user,     to: done }
  - { from: adv_inline,   to: done }
---
flowchart TD
    enter([mb_iter / mb_next]) --> is_iter{iterator handle?}
    is_iter -->|yes| return_obj([return as-is])
    is_iter -->|no| is_gen{generator handle?}
    is_gen -->|yes| wrap_gen[wrap Generator]
    is_gen -->|no| has_ptr{ptr kind?}
    has_ptr -->|List/Tuple| promote_list[retain; promote]
    has_ptr -->|Str/Dict/Set/FrozenSet/Bytes| promote_str[snapshot]
    has_ptr -->|Instance| is_instance{has __iter__?}
    has_ptr -->|other| type_error([TypeError])
    is_instance -->|yes| call_iter[call __iter__ self]
    call_iter --> iter_result{result kind?}
    iter_result -->|known iter / gen| use_known[return / wrap]
    iter_result -->|other ptr| wrap_user[retain; UserDefined]
    iter_result -->|none| type_error
    promote_list --> next_kind{advance kind?}
    promote_str --> next_kind
    wrap_gen --> next_kind
    wrap_user --> next_kind
    use_known --> next_kind
    next_kind -->|Callable| adv_callable[advance_callable]
    next_kind -->|Generator| adv_gen[advance_generator]
    next_kind -->|UserDefined| adv_user[advance_userdefined; clear exc slot]
    next_kind -->|built-in| adv_inline[advance_iter inline]
    adv_callable --> done([return value])
    adv_gen --> done
    adv_user --> done
    adv_inline --> done
```

## For-loop interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: for-loop-emit
actors:
  - { id: JIT,    kind: system, label: "JIT-compiled for-loop" }
  - { id: Iter,   kind: system, label: "iter.rs runtime" }
  - { id: Source, kind: system, label: "underlying iterable (list / __iter__ obj / generator)" }
messages:
  - { from: JIT, to: Iter, name: mb_iter(obj) }
  - { from: Iter, to: Source, name: "__iter__ if Instance" }
  - { from: Source, to: Iter, name: iter_handle, returns: MbValue }
  - { from: Iter, to: JIT, name: handle }
  - { from: JIT, to: Iter, name: mb_has_next(handle) }
  - { from: Iter, to: Source, name: "__next__ / advance" }
  - { from: Source, to: Iter, name: value | StopIteration, returns: MbValue }
  - { from: Iter, to: JIT, name: "true (peeked cached)" }
  - { from: JIT, to: Iter, name: mb_next(handle) }
  - { from: Iter, to: JIT, name: "peeked value" }
  - { from: JIT, to: Iter, name: mb_has_next(handle) }
  - { from: Iter, to: Source, name: "__next__ / advance" }
  - { from: Source, to: Iter, name: StopIteration, returns: signal }
  - { from: Iter, to: JIT, name: "false (exhausted)" }
  - { from: JIT, to: Iter, name: mb_iter_release(handle) }
---
sequenceDiagram
    participant JIT
    participant Iter
    participant Source
    JIT->>Iter: mb_iter(obj)
    Iter->>Source: __iter__ (if Instance)
    Source-->>Iter: iter_handle
    Iter-->>JIT: handle
    JIT->>Iter: mb_has_next
    Iter->>Source: __next__ / advance
    Source-->>Iter: value
    Iter-->>JIT: true [peeked cached]
    JIT->>Iter: mb_next
    Iter-->>JIT: peeked value
    JIT->>Iter: mb_has_next
    Iter->>Source: __next__ / advance
    Source-->>Iter: StopIteration
    Iter-->>JIT: false [exhausted]
    JIT->>Iter: mb_iter_release
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: custom-iter-non-self
    given: iterators/custom_iter_non_self.py defines __iter__ returning iter([1,2,3])
    when: a for-loop iterates over the custom object
    then: the returned iterator handle is reused and stdout is 1, 2, 3
  - id: stop-iteration-clear
    given: a user-defined __next__ raises StopIteration after yielding values
    when: a for-loop exits through the StopIteration path
    then: the runtime clears the exception slot so StopIteration does not leak past the loop
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: runtime-iter-test-plan
title: Iteration Protocol Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> Basic["iterators/basic.py"]
    Runner --> Custom["iterators/custom_iter_non_self.py"]
    Runner --> Combo["iterators/enumerate_zip.py"]
    Runner --> Range["iterators/range_step.py"]
    Runner --> Stop["exceptions/keyerror_repr.py"]
    Runner --> Callable["iterators/callable_sentinel.py"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/iter.rs
    action: modify
    impl_mode: hand-written
    description: "Iterator state, allocation, advance protocol, combinators. Hand-written; spec is the design contract, not a codegen template."
```

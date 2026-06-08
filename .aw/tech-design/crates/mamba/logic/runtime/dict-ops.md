---
id: dict-ops
title: Dict Operations and DictKey
crate: mamba
files:
  - crates/mamba/src/runtime/dict_ops.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: ad1e799c4
---

# Dict Operations and DictKey

Mamba dicts use `RwLock<IndexMap<DictKey, MbValue>>` so insertion order
is preserved (matching Python 3.7+ semantics) and the lock allows
concurrent readers across threads. The dict-key normaliser
`to_dict_key` collapses every Mamba value into a hashable `DictKey`
variant: primitives map to dedicated variants; user-class instances
hash via `__hash__` and compare via `__eq__` on collision; non-hashable
heap objects fall back to `Other(bits.to_string())` so identity-keyed
lookups still work.

Three load-bearing invariants:

1. **`d[1]` and `d["1"]` are distinct entries** — `DictKey::Int(1)`
   and `DictKey::Str("1")` hash to different buckets even though
   Python `1 == "1"` would be False; this matches CPython.
2. **`DictKey::Instance` retains the instance pointer** — clone-and-
   drop walk the rc; the comparison path uses the cached `hash_val`
   for bucket selection then dispatches `__eq__` on collision.
3. **`mb_dict_getitem` raises `KeyError` with the raw key text** —
   `dict_key_raw_str` returns the unquoted form; the printer in
   `string-ops.md` `value_to_string` adds the repr-quoting at output
   time. Pre-quoting at the raise site would double-up (commit
   `dbbaf7396`).

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: dict-types
types:
  DictKey:        { kind: enum }
  ObjDataDict:    { kind: struct, label: "ObjData::Dict(RwLock<IndexMap<DictKey, MbValue>>)" }
  IndexMap:       { kind: struct, label: "indexmap::IndexMap" }
  MbValue:        { kind: struct }
  ToDictKey:      { kind: struct, label: "to_dict_key(val) — normaliser" }
  ClassModule:    { kind: struct, label: "class.rs (Instance __hash__/__eq__)" }
  ExceptionMod:   { kind: struct, label: "exception.rs (KeyError)" }
edges:
  - { from: ObjDataDict,  to: IndexMap, kind: owns }
  - { from: IndexMap,     to: DictKey,  kind: references, label: "key" }
  - { from: IndexMap,     to: MbValue,  kind: references, label: "value" }
  - { from: ToDictKey,    to: DictKey,  kind: owns,       label: "produces" }
  - { from: ToDictKey,    to: ClassModule, kind: references, label: "Instance: lookup __hash__" }
  - { from: ObjDataDict,  to: ExceptionMod, kind: references, label: "missing key → KeyError" }
---
classDiagram
    class DictKey
    class ObjDataDict
    class IndexMap
    class MbValue
    class ToDictKey
    class ClassModule
    class ExceptionMod
    ObjDataDict --> IndexMap : owns
    IndexMap --> DictKey : key
    IndexMap --> MbValue : value
    ToDictKey --> DictKey : produces
    ToDictKey --> ClassModule : __hash__
    ObjDataDict --> ExceptionMod : KeyError
```

## DictKey variants
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "dict-key-types"
$defs:
  DictKey:
    description: "Hashable + Eq via Hash impl; Clone/Drop walk rc on Instance variant"
    oneOf:
      - { title: Int,       properties: { value: { type: integer, x-rust-type: i64 } } }
      - { title: Str,       properties: { value: { type: string } } }
      - { title: Bool,      properties: { value: { type: boolean } } }
      - { title: None,      type: object }
      - title: Instance
        properties:
          hash_val:   { type: integer, x-rust-type: i64, description: "from __hash__ at insert time" }
          ptr:        { type: integer, x-rust-type: usize, description: "instance pointer (retained); release on Drop" }
          class_name: { type: string }
        required: [hash_val, ptr, class_name]
        description: "user class instance; __eq__ dispatched on bucket collision"
      - title: Other
        properties:
          bits: { type: string, description: "MbValue::to_bits().to_string() — non-hashable heap fallback" }
        required: [bits]
        description: "identity-keyed for non-hashable heap objects we can't dunder-route today"
  DictEntry:
    type: object
    description: "IndexMap<DictKey, MbValue> entry"
    properties:
      key:   { $ref: "#/$defs/DictKey" }
      value: { x-rust-type: MbValue }
    required: [key, value]
```

## Key-conversion / lookup logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: dict-key-and-getitem
entry: enter
nodes:
  enter:        { kind: start,    label: "to_dict_key(val) | mb_dict_getitem(d, key)" }
  is_int:       { kind: decision, label: "val is INT?" }
  k_int:        { kind: process,  label: "DictKey::Int(i)" }
  is_str:       { kind: decision, label: "val.as_ptr is Str?" }
  k_str:        { kind: process,  label: "DictKey::Str(s.clone())" }
  is_bool:      { kind: decision, label: "val is BOOL?" }
  k_bool:       { kind: process,  label: "DictKey::Bool(b)" }
  is_none:      { kind: decision, label: "val is NONE?" }
  k_none:       { kind: process,  label: "DictKey::None" }
  is_inst:      { kind: decision, label: "val.as_ptr is Instance with __hash__?" }
  k_inst:       { kind: process,  label: "DictKey::Instance{hash_val, ptr, class_name} — retain ptr" }
  k_other:      { kind: process,  label: "DictKey::Other(bits.to_string())" }
  read_lock:    { kind: process,  label: "lock.try_read or lock.read" }
  lookup:       { kind: decision, label: "guard.get(&dk)?" }
  hit:          { kind: process,  label: "retain_if_ptr; return value" }
  raise_kerr:   { kind: process,  label: "drop guard; mb_raise(KeyError, dict_key_raw_str(dk))" }
  done:         { kind: terminal, label: "return value | none" }
edges:
  - { from: enter,    to: is_int }
  - { from: is_int,   to: k_int,   label: "yes" }
  - { from: is_int,   to: is_str,  label: "no" }
  - { from: is_str,   to: k_str,   label: "yes" }
  - { from: is_str,   to: is_bool, label: "no" }
  - { from: is_bool,  to: k_bool,  label: "yes" }
  - { from: is_bool,  to: is_none, label: "no" }
  - { from: is_none,  to: k_none,  label: "yes" }
  - { from: is_none,  to: is_inst, label: "no" }
  - { from: is_inst,  to: k_inst,  label: "yes + has __hash__" }
  - { from: is_inst,  to: k_other, label: "no" }
  - { from: k_int,    to: read_lock }
  - { from: k_str,    to: read_lock }
  - { from: k_bool,   to: read_lock }
  - { from: k_none,   to: read_lock }
  - { from: k_inst,   to: read_lock }
  - { from: k_other,  to: read_lock }
  - { from: read_lock, to: lookup }
  - { from: lookup,   to: hit,         label: "found" }
  - { from: lookup,   to: raise_kerr,  label: "miss" }
  - { from: hit,      to: done }
  - { from: raise_kerr, to: done }
---
flowchart TD
    enter([to_dict_key + getitem]) --> is_int{INT?}
    is_int -->|yes| k_int[DictKey::Int]
    is_int -->|no| is_str{Str ptr?}
    is_str -->|yes| k_str[DictKey::Str clone]
    is_str -->|no| is_bool{BOOL?}
    is_bool -->|yes| k_bool[DictKey::Bool]
    is_bool -->|no| is_none{NONE?}
    is_none -->|yes| k_none[DictKey::None]
    is_none -->|no| is_inst{Instance has __hash__?}
    is_inst -->|yes| k_inst[DictKey::Instance retain ptr]
    is_inst -->|no| k_other[DictKey::Other bits]
    k_int --> read_lock[lock.read]
    k_str --> read_lock
    k_bool --> read_lock
    k_none --> read_lock
    k_inst --> read_lock
    k_other --> read_lock
    read_lock --> lookup{found?}
    lookup -->|yes| hit[retain; return]
    lookup -->|no| raise_kerr[KeyError raw_str]
    hit --> done([value])
    raise_kerr --> done
```

## Method dispatch interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: dict-method-dispatch
actors:
  - { id: JIT,        kind: system, label: "JIT-emitted lower-method-call" }
  - { id: Dispatcher, kind: system, label: "dispatch_dict_method" }
  - { id: Handler,    kind: system, label: "mb_dict_get / mb_dict_pop / ..." }
  - { id: Storage,    kind: system, label: "RwLock<IndexMap<DictKey, MbValue>>" }
messages:
  - { from: JIT,        to: Dispatcher, name: "dispatch_dict_method('get', d, [key, default])" }
  - { from: Dispatcher, to: Handler,    name: "mb_dict_get(d, key, default)" }
  - { from: Handler,    to: Handler,    name: "to_dict_key(key)" }
  - { from: Handler,    to: Storage,    name: "lock.read; guard.get(&dk)" }
  - { from: Storage,    to: Handler,    name: "Some(&v) | None" }
  - { from: Handler,    to: Handler,    name: "retain_if_ptr or return default" }
  - { from: Handler,    to: Dispatcher, name: result, returns: MbValue }
  - { from: Dispatcher, to: JIT,        name: result, returns: MbValue }
  - { from: JIT,        to: Handler,    name: "direct: mb_dict_getitem(d, key) — __getitem__ path" }
  - { from: Handler,    to: Storage,    name: "lock.read; guard.get" }
  - { from: Storage,    to: Handler,    name: "Some | None (raises KeyError on None)" }
  - { from: Handler,    to: JIT,        name: result }
---
sequenceDiagram
    participant JIT
    participant Dispatcher
    participant Handler
    participant Storage
    JIT->>Dispatcher: dispatch get / pop / keys / ...
    Dispatcher->>Handler: forward to mb_dict_*
    Handler->>Handler: to_dict_key(key)
    Handler->>Storage: lock.read; guard.get
    Storage-->>Handler: Some or None
    Handler-->>Dispatcher: result
    Dispatcher-->>JIT: result
    JIT->>Handler: direct mb_dict_getitem (subscript)
    Handler->>Storage: lock.read; guard.get
    Storage-->>Handler: Some or KeyError
    Handler-->>JIT: result
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: distinct-key-kinds
    given: dict_methods/deep_broad.py creates keys 1 and "1"
    when: both entries are inserted into the same dict
    then: DictKey::Int and DictKey::Str stay distinct and len is 2
  - id: keyerror-repr
    given: exceptions/keyerror_repr.py reads a missing string key
    when: mb_dict_getitem raises KeyError
    then: the raw key text is stored and output formatting quotes it exactly once
  - id: insertion-order
    given: dict_methods/insertion_order.py inserts keys in sequence
    when: keys, values, or items are observed
    then: IndexMap preserves Python 3.7+ insertion order
  - id: instance-dict-key
    given: class_system/instance_dict_key.py uses instances with __hash__ and __eq__
    when: a value-equal instance reads a stored key
    then: DictKey::Instance uses cached hash and __eq__ collision resolution
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: runtime-dict-ops-test-plan
title: Dict Operations and DictKey Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> Basics["dict_methods/deep_broad.py"]
    Runner --> KeyError["exceptions/keyerror_repr.py"]
    Runner --> GetDefault["dict_methods/get_setdefault.py"]
    Runner --> Pop["dict_methods/pop_popitem.py"]
    Runner --> Update["dict_methods/update_clear.py"]
    Runner --> InstanceKey["class_system/instance_dict_key.py"]
    Runner --> Unpack["dict_unpack/merge_broad.py"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/dict_ops.rs
    action: modify
    impl_mode: hand-written
    description: "DictKey enum (6 variants) + to_dict_key + dict_key_raw_str + dict_key_display + IndexMap-backed mb_dict_* surface + dispatch_dict_method router. Hand-written; key normalisation algorithm is load-bearing for cross-test compatibility."
```

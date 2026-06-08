---
id: value-and-rc
title: Value Representation and Reference Counting
crate: mamba
files:
  - crates/mamba/src/runtime/value.rs
  - crates/mamba/src/runtime/rc.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: 91213c22f
---

# Value Representation and Reference Counting

`MbValue` is a single 64-bit NaN-boxed word. Tagged integers, booleans,
None, NotImplemented, and JIT function pointers live entirely in the
NaN payload; floats live as plain non-tagged IEEE 754 doubles; heap
objects (str, list, dict, tuple, instance, set, frozenset, bytes,
bytearray, BigInt, Complex, CodeObject) live behind a 48-bit pointer
into a `MbObject` with an atomic refcount header.

Three load-bearing invariants:

1. **48-bit address bound** — `from_ptr` / `from_int` debug-assert that
   the payload fits 48 bits. ARM64 / x86-64 user space respects this
   today; if Mamba is ever ported to a target with wider pointers the
   NaN-boxing layout has to be revisited.
2. **NEW / BORROWED / VOID classification** — every public
   `mb_*` function that returns `MbValue` falls into one of three
   refcount classes (commit `#1129` ownership audit). NEW = caller
   owns rc=1; BORROWED = caller does NOT own, callee called
   `retain_if_ptr` so caller now does; VOID = no return value. Mismatch
   at any boundary leaks or double-frees.
3. **`IMMORTAL_REFCOUNT = u32::MAX`** — JIT-embedded constants
   (interned strings / bytes) have refcount sentinel `u32::MAX`;
   `mb_release` early-returns instead of decrementing so they are
   never freed.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: value-rc-types
types:
  MbValue:           { kind: struct, label: "u64 NaN-boxed" }
  MbObject:          { kind: struct, label: "header + data" }
  MbObjectHeader:    { kind: struct, label: "atomic rc + ObjKind tag" }
  ObjKind:           { kind: enum,   label: "u8 type tag (Str / List / Dict / ...)" }
  ObjData:           { kind: enum,   label: "12-variant data union" }
  DictKey:           { kind: enum,   label: "from runtime::dict_ops" }
  RetainRelease:     { kind: struct, label: "mb_retain / mb_release / retain_if_ptr / release_if_ptr" }
edges:
  - { from: MbValue,        to: MbObject,       kind: references, label: "ptr tag → 48-bit address" }
  - { from: MbObject,       to: MbObjectHeader, kind: owns }
  - { from: MbObject,       to: ObjData,        kind: owns }
  - { from: MbObjectHeader, to: ObjKind,        kind: owns,       label: "type tag separate from ObjData discriminator" }
  - { from: ObjData,        to: MbValue,        kind: references, label: "List/Tuple/Set/Dict/Instance/FrozenSet store MbValue" }
  - { from: ObjData,        to: DictKey,        kind: references, label: "Dict variant" }
  - { from: RetainRelease,  to: MbObjectHeader, kind: references, label: "atomic CAS on rc field" }
---
classDiagram
    class MbValue
    class MbObject
    class MbObjectHeader
    class ObjKind
    class ObjData
    class DictKey
    class RetainRelease
    MbValue --> MbObject : ptr tag
    MbObject --> MbObjectHeader : owns
    MbObject --> ObjData : owns
    MbObjectHeader --> ObjKind : kind
    ObjData --> MbValue : embedded
    ObjData --> DictKey : Dict variant
    RetainRelease --> MbObjectHeader : atomic CAS
```

## Value layout
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "value-rc-types"
$defs:
  MbValue:
    type: object
    x-rust-type: MbValue
    description: "NaN-boxed 64-bit; tag in bits 48..50 when NAN_PREFIX (0xFFF8_0000_0000_0000) set"
    properties:
      bits: { type: integer, x-rust-type: u64 }
    required: [bits]
    examples:
      - { bits: "0xFFF9_0000_0000_002A", description: "tag=INT(1), payload=42" }
      - { bits: "0xFFFA_0000_0000_0001", description: "tag=BOOL(2), payload=true" }
      - { bits: "0xFFFB_0000_0000_0000", description: "tag=NONE(3)" }
      - { bits: "0xFFFC_0000_0000_<addr>", description: "tag=FUNC(4), 48-bit code addr" }
      - { bits: "0xFFFD_0000_0000_0000", description: "tag=NOTIMPLEMENTED(5)" }
      - { bits: "0xFFF8_0000_0000_<addr>", description: "tag=PTR(0), 48-bit MbObject*" }
  Tag:
    type: string
    enum: [PTR, INT, BOOL, NONE, FUNC, NOTIMPLEMENTED]
    description: "tag values 0..5; remaining 3-bit values reserved"
  ObjKind:
    type: string
    enum:
      [Str, List, Dict, Tuple, Function, Class, Instance,
       Set, Bytes, ByteArray, FrozenSet, BigInt, Complex, CodeObject]
  MbObjectHeader:
    type: object
    x-rust-type: MbObjectHeader
    properties:
      rc:   { type: integer, x-rust-type: AtomicU32, description: "u32::MAX = IMMORTAL" }
      kind: { $ref: "#/$defs/ObjKind" }
    required: [rc, kind]
  ObjData:
    description: "12-variant data union, repr(C) follows MbObjectHeader"
    type: object
    oneOf:
      - { title: Str,        properties: { value: { type: string } } }
      - { title: List,       properties: { lock: { type: array, items: { x-rust-type: MbValue } } }, description: "RwLock<Vec<MbValue>>" }
      - { title: Dict,       properties: { lock: { type: object, additionalProperties: { x-rust-type: MbValue } } }, description: "RwLock<IndexMap<DictKey, MbValue>>" }
      - { title: Tuple,      properties: { items: { type: array, items: { x-rust-type: MbValue } } } }
      - { title: Instance,   properties: { class_name: { type: string }, fields: { type: object, additionalProperties: { x-rust-type: MbValue } } }, description: "fields wrapped in RwLock" }
      - { title: Set,        properties: { lock: { type: array, items: { x-rust-type: MbValue } } } }
      - { title: Bytes,      properties: { value: { type: array, items: { type: integer, minimum: 0, maximum: 255 } } } }
      - { title: ByteArray,  properties: { lock: { type: array, items: { type: integer, minimum: 0, maximum: 255 } } } }
      - { title: FrozenSet,  properties: { items: { type: array, items: { x-rust-type: MbValue } } } }
      - { title: BigInt,     properties: { value: { x-rust-type: "num_bigint::BigInt" } } }
      - { title: Complex,    properties: { real: { type: number }, imag: { type: number } } }
      - { title: CodeObject, properties: { source: { type: string }, filename: { type: string }, mode: { type: string }, ast: { x-rust-type: "Box<parser::ast::Module>" } } }
```

## Refcount lifecycle
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: rc-lifecycle
initial: Allocated
nodes:
  Allocated:    { kind: initial,  label: "MbObject::new_*; rc = 1" }
  Live:         { kind: normal,   label: "rc >= 1; mutable via RwLock if applicable" }
  Immortal:     { kind: normal,   label: "rc = IMMORTAL_REFCOUNT (u32::MAX)" }
  Released:     { kind: transient, label: "rc = 0; release children" }
  Freed:        { kind: terminal,  label: "Box::from_raw drop runs" }
edges:
  - { from: Allocated, to: Live,     event: "constructor returns" }
  - { from: Allocated, to: Immortal, event: "JIT embeds with rc=u32::MAX" }
  - { from: Live,      to: Live,     event: "mb_retain (rc += 1)" }
  - { from: Live,      to: Live,     event: "mb_release (rc > 1, rc -= 1)" }
  - { from: Live,      to: Released, event: "mb_release (rc == 1, rc → 0)" }
  - { from: Released,  to: Freed,    event: "drop_inner releases children + Box::from_raw" }
  - { from: Immortal,  to: Immortal, event: "mb_release no-op (rc unchanged at u32::MAX)" }
---
stateDiagram-v2
    [*] --> Allocated
    Allocated --> Live: constructor
    Allocated --> Immortal: JIT embed
    Live --> Live: mb_retain
    Live --> Live: mb_release [rc > 1]
    Live --> Released: mb_release [rc → 0]
    Released --> Freed: drop_inner
    Immortal --> Immortal: mb_release (no-op)
    Freed --> [*]
```

## Tag dispatch
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tag-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "MbValue.tag() / type query" }
  is_nan_pref:  { kind: decision, label: "(bits & NAN_PREFIX) == NAN_PREFIX AND bits != canonical NaN" }
  is_float:     { kind: terminal, label: "Float branch — f64::from_bits" }
  read_tag:     { kind: process,  label: "tag = (bits & TAG_MASK) >> 48" }
  match_tag:    { kind: decision, label: "tag value 0..5" }
  ptr_extract:  { kind: process,  label: "PTR(0): mask payload → MbObject*" }
  int_extract:  { kind: process,  label: "INT(1): sign-extend 48-bit payload" }
  bool_extract: { kind: process,  label: "BOOL(2): payload bit 0" }
  none_v:       { kind: process,  label: "NONE(3) singleton" }
  func_extract: { kind: process,  label: "FUNC(4): 48-bit code address" }
  ni_v:         { kind: process,  label: "NOTIMPLEMENTED(5) singleton" }
  retain_if:    { kind: decision, label: "retain_if_ptr / release_if_ptr called?" }
  retain_skip:  { kind: terminal, label: "no-op (non-ptr)" }
  retain_apply: { kind: process,  label: "mb_retain / mb_release on MbObject*" }
  done:         { kind: terminal, label: "value or refcount op done" }
edges:
  - { from: enter,        to: is_nan_pref }
  - { from: is_nan_pref,  to: is_float, label: "no" }
  - { from: is_nan_pref,  to: read_tag, label: "yes" }
  - { from: read_tag,     to: match_tag }
  - { from: match_tag,    to: ptr_extract,  label: "PTR(0)" }
  - { from: match_tag,    to: int_extract,  label: "INT(1)" }
  - { from: match_tag,    to: bool_extract, label: "BOOL(2)" }
  - { from: match_tag,    to: none_v,       label: "NONE(3)" }
  - { from: match_tag,    to: func_extract, label: "FUNC(4)" }
  - { from: match_tag,    to: ni_v,         label: "NOTIMPLEMENTED(5)" }
  - { from: ptr_extract,  to: retain_if }
  - { from: int_extract,  to: retain_if }
  - { from: bool_extract, to: retain_if }
  - { from: none_v,       to: retain_if }
  - { from: func_extract, to: retain_if }
  - { from: ni_v,         to: retain_if }
  - { from: is_float,     to: retain_if }
  - { from: retain_if,    to: retain_apply, label: "is_ptr" }
  - { from: retain_if,    to: retain_skip,  label: "non-ptr" }
  - { from: retain_apply, to: done }
  - { from: retain_skip,  to: done }
---
flowchart TD
    enter([MbValue tag query]) --> is_nan_pref{NAN_PREFIX set AND not canonical NaN?}
    is_nan_pref -->|no| is_float([Float])
    is_nan_pref -->|yes| read_tag[bits 48..50]
    read_tag --> match_tag{tag?}
    match_tag -->|PTR| ptr_extract[MbObject*]
    match_tag -->|INT| int_extract[sign-extend 48 bit]
    match_tag -->|BOOL| bool_extract[payload bit 0]
    match_tag -->|NONE| none_v[None singleton]
    match_tag -->|FUNC| func_extract[code address]
    match_tag -->|NI| ni_v[NotImplemented]
    ptr_extract --> retain_if{retain/release called?}
    int_extract --> retain_if
    bool_extract --> retain_if
    none_v --> retain_if
    func_extract --> retain_if
    ni_v --> retain_if
    is_float --> retain_if
    retain_if -->|is_ptr| retain_apply[mb_retain / mb_release]
    retain_if -->|non-ptr| retain_skip([no-op])
    retain_apply --> done([done])
    retain_skip --> done
```

## Ownership protocol interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: ownership-protocol
actors:
  - { id: Caller,    kind: system, label: "JIT-emitted code or runtime fn" }
  - { id: Constructor, kind: system, label: "NEW: mb_list_new / mb_dict_new / ..." }
  - { id: Borrowed,    kind: system, label: "BORROWED: mb_list_getitem / mb_getattr / ..." }
  - { id: Void,        kind: system, label: "VOID: mb_list_append / mb_setattr / ..." }
  - { id: Container,   kind: system, label: "owning collection / global slot" }
messages:
  - { from: Caller,      to: Constructor, name: mb_list_new }
  - { from: Constructor, to: Caller,      name: "rc=1 (NEW)", returns: MbValue }
  - { from: Caller,      to: Borrowed,    name: "mb_list_getitem(lst, i)" }
  - { from: Borrowed,    to: Container,   name: "read items[i] → ref" }
  - { from: Borrowed,    to: Borrowed,    name: "retain_if_ptr — promote borrow to NEW" }
  - { from: Borrowed,    to: Caller,      name: "rc+=1 owned ref", returns: MbValue }
  - { from: Caller,      to: Void,        name: "mb_list_append(lst, v)" }
  - { from: Void,        to: Container,   name: "items.push(v) — Vec retains" }
  - { from: Caller,      to: Caller,      name: "mb_release on returned values when drop scope ends" }
---
sequenceDiagram
    participant Caller
    participant Constructor
    participant Borrowed
    participant Void
    participant Container
    Caller->>Constructor: mb_list_new
    Constructor-->>Caller: rc=1 [NEW]
    Caller->>Borrowed: mb_list_getitem(lst, i)
    Borrowed->>Container: read items[i]
    Borrowed->>Borrowed: retain_if_ptr (NEW promote)
    Borrowed-->>Caller: rc+=1 owned
    Caller->>Void: mb_list_append(lst, v)
    Void->>Container: items.push(v)
    Note over Caller: mb_release at scope exit
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: int-overflow-promotes
    given: arithmetic/int_overflow.py creates a value outside the 48-bit inline integer range
    when: MbValue::from_int cannot encode the result inline
    then: the value promotes to a BigInt heap object and prints exactly
  - id: float-special-values
    given: float_methods/special_values.py manipulates NaN and infinity
    when: MbValue.tag sees a non-tagged IEEE 754 payload
    then: it stays on the float branch and preserves CPython NaN behavior
  - id: cycle-collection
    given: data_structures/cycle.py creates a self-referencing list
    when: normal rc cannot reclaim the cycle
    then: the cycle collector can break the cycle without violating retain/release ownership
  - id: immortal-constant
    given: arithmetic/jit_const_immortal.py reuses embedded constant heap objects
    when: mb_release sees IMMORTAL_REFCOUNT
    then: release is a no-op and the object is never freed
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: runtime-value-rc-test-plan
title: Value Representation and Reference Counting Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> IntOverflow["arithmetic/int_overflow.py"]
    Runner --> FloatSpecial["float_methods/special_values.py"]
    Runner --> Truthiness["bool_type/none_broad.py"]
    Runner --> IdBasic["language/id_basic.py"]
    Runner --> RcBalance["data_structures/append_pop.py"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/value.rs
    action: modify
    impl_mode: hand-written
    description: "NaN-boxed MbValue: 6 tag values (PTR/INT/BOOL/NONE/FUNC/NOTIMPLEMENTED), 48-bit payload, debug-asserted bounds. Hand-written; layout is a load-bearing ABI."
  - file: crates/mamba/src/runtime/rc.rs
    action: modify
    impl_mode: hand-written
    description: "MbObject + MbObjectHeader + ObjData (12 variants) + ObjKind tag. retain_if_ptr / release_if_ptr / mb_retain / mb_release; IMMORTAL_REFCOUNT sentinel. Hand-written; NEW/BORROWED/VOID classification doc-block in this file is the authoritative ownership audit."
```

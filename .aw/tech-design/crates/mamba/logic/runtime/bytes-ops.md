---
id: bytes-ops
title: Bytes and ByteArray Operations
crate: mamba
files:
  - crates/mamba/src/runtime/bytes_ops.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: fc12e3434
---

# Bytes and ByteArray Operations

Mamba bytes are immutable `Vec<u8>` (`ObjData::Bytes`); bytearrays are
`RwLock<Vec<u8>>` (`ObjData::ByteArray`). The runtime exposes the
CPython surface (`b'..'` literal construction, `+` / `*` /
indexing / slicing, `.decode` / `.hex` / `.fromhex`, predicate methods,
search/replace). `dispatch_bytes_method` routes per-method calls; most
read paths cover both Bytes and ByteArray uniformly.

Three load-bearing invariants:

1. **`bytes[i]` returns an `int`, not a single-byte bytes** — CPython
   compatibility. `mb_bytes_getitem` uses `MbValue::from_int(b as i64)`
   for single indices; slicing returns a fresh bytes object.
2. **`hex()` accepts no separator argument by default** — three-arg
   form `b.hex(sep, bytes_per_sep)` is a CPython 3.8+ feature; current
   `mb_bytes_hex` is single-arg only. Slicing-then-decode is the
   workaround. Open gap.
3. **`fromhex` is a classmethod, dispatched via the str type-name
   unbound-method path** — `bytes.fromhex(...)` lowers to
   `__unbound_method__(type='bytes', method='fromhex')` via
   `class.rs` getattr (see `class.md`); the dispatcher in
   `string_ops.rs` matches `"fromhex"` and routes to the bytes
   constructor. Same pattern for `bytearray.fromhex`.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: bytes-types
types:
  ObjDataBytes:     { kind: struct, label: "ObjData::Bytes(Vec<u8>) — immutable" }
  ObjDataByteArray: { kind: struct, label: "ObjData::ByteArray(RwLock<Vec<u8>>) — mutable" }
  MbValue:          { kind: struct }
  ExceptionMod:     { kind: struct, label: "exception.rs (IndexError on subscript)" }
  StringOps:        { kind: struct, label: "string_ops.rs (decode delegates to Str)" }
  IterModule:       { kind: struct, label: "iter.rs (Bytes/ByteArray materialize to int list)" }
  StrTypeUnbound:   { kind: struct, label: "class.rs (bytes.fromhex via __unbound_method__)" }
edges:
  - { from: ObjDataBytes,     to: MbValue,        kind: references, label: "subscript yields INT" }
  - { from: ObjDataByteArray, to: MbValue,        kind: references }
  - { from: ObjDataBytes,     to: ExceptionMod,   kind: references, label: "IndexError" }
  - { from: ObjDataBytes,     to: StringOps,      kind: references, label: ".decode" }
  - { from: ObjDataBytes,     to: IterModule,     kind: references, label: "iter materializes int list" }
  - { from: StrTypeUnbound,   to: ObjDataBytes,   kind: references, label: "fromhex constructor route" }
---
classDiagram
    class ObjDataBytes
    class ObjDataByteArray
    class MbValue
    class ExceptionMod
    class StringOps
    class IterModule
    class StrTypeUnbound
    ObjDataBytes --> MbValue : INT subscript
    ObjDataByteArray --> MbValue : INT subscript
    ObjDataBytes --> ExceptionMod : IndexError
    ObjDataBytes --> StringOps : decode
    ObjDataBytes --> IterModule : iter
    StrTypeUnbound --> ObjDataBytes : fromhex
```

## Bytes shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "bytes-types"
$defs:
  MbBytes:
    type: object
    description: "ObjData::Bytes(Vec<u8>) — immutable"
    properties:
      data: { type: array, items: { type: integer, minimum: 0, maximum: 255 } }
    required: [data]
  MbByteArray:
    type: object
    description: "ObjData::ByteArray(RwLock<Vec<u8>>) — mutable"
    properties:
      lock: { type: array, items: { type: integer, minimum: 0, maximum: 255 } }
    required: [lock]
  BytesSubscriptResult:
    description: "What mb_bytes_getitem returns"
    oneOf:
      - { title: SingleByte, x-rust-type: "MbValue (INT)", description: "from_int(byte as i64)" }
      - { title: SliceBytes, x-rust-type: "MbValue (PTR-Bytes)", description: "MbObject::new_bytes(slice)" }
```

## Subscript / dispatch logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: bytes-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "mb_bytes_getitem | dispatch_bytes_method" }
  classify:     { kind: decision, label: "operation?" }
  is_int_idx:   { kind: decision, label: "subscript: index is INT?" }
  range_check:  { kind: decision, label: "0 <= i < len after negative-wrap?" }
  read_byte:    { kind: process,  label: "MbValue::from_int(data[i] as i64)" }
  raise_idx:    { kind: process,  label: "raise IndexError" }
  is_slice:     { kind: decision, label: "subscript: index is Slice triple?" }
  slice_alloc:  { kind: process,  label: "new_bytes(slice)" }
  do_decode:    { kind: process,  label: "decode: utf8 → string_ops new_str" }
  do_hex:       { kind: process,  label: "hex: format!('{:02x}', b) join" }
  do_fromhex:   { kind: process,  label: "fromhex: parse string → Vec<u8>" }
  do_concat:    { kind: process,  label: "+: extend Vec; new_bytes" }
  do_repeat:    { kind: process,  label: "*: repeat Vec; new_bytes" }
  do_predicate: { kind: process,  label: "isdigit / isalpha / isspace / startswith / endswith — read-only" }
  done:         { kind: terminal, label: "return MbValue" }
edges:
  - { from: enter,        to: classify }
  - { from: classify,     to: is_int_idx,   label: "subscript" }
  - { from: classify,     to: do_decode,    label: "decode" }
  - { from: classify,     to: do_hex,       label: "hex" }
  - { from: classify,     to: do_fromhex,   label: "fromhex" }
  - { from: classify,     to: do_concat,    label: "+" }
  - { from: classify,     to: do_repeat,    label: "*" }
  - { from: classify,     to: do_predicate, label: "predicate / search" }
  - { from: is_int_idx,   to: range_check,  label: "yes" }
  - { from: is_int_idx,   to: is_slice,     label: "no" }
  - { from: range_check,  to: read_byte,    label: "in range" }
  - { from: range_check,  to: raise_idx,    label: "out" }
  - { from: is_slice,     to: slice_alloc,  label: "yes" }
  - { from: read_byte,    to: done }
  - { from: raise_idx,    to: done }
  - { from: slice_alloc,  to: done }
  - { from: do_decode,    to: done }
  - { from: do_hex,       to: done }
  - { from: do_fromhex,   to: done }
  - { from: do_concat,    to: done }
  - { from: do_repeat,    to: done }
  - { from: do_predicate, to: done }
---
flowchart TD
    enter([bytes op]) --> classify{op?}
    classify -->|subscript| is_int_idx{INT idx?}
    classify -->|decode| do_decode[utf8 → str]
    classify -->|hex| do_hex[lowercase hex]
    classify -->|fromhex| do_fromhex[parse string]
    classify -->|concat| do_concat[extend Vec]
    classify -->|repeat| do_repeat[repeat Vec]
    classify -->|predicate| do_predicate[read-only]
    is_int_idx -->|yes| range_check{in range?}
    is_int_idx -->|no| is_slice{slice?}
    range_check -->|yes| read_byte[from_int byte]
    range_check -->|no| raise_idx[IndexError]
    is_slice -->|yes| slice_alloc[new_bytes slice]
    read_byte --> done([result])
    raise_idx --> done
    slice_alloc --> done
    do_decode --> done
    do_hex --> done
    do_fromhex --> done
    do_concat --> done
    do_repeat --> done
    do_predicate --> done
```

## fromhex unbound-method interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: bytes-fromhex-flow
actors:
  - { id: User,    kind: actor }
  - { id: JIT,     kind: system }
  - { id: GetAttr, kind: system, label: "class.rs mb_getattr" }
  - { id: Dispatcher, kind: system, label: "string_ops.rs dispatch_str_method" }
  - { id: Bytes,   kind: system, label: "bytes_ops.rs" }
messages:
  - { from: User,        to: JIT,        name: "bytes.fromhex('48656c6c6f')" }
  - { from: JIT,         to: GetAttr,    name: "mb_getattr('bytes', 'fromhex')" }
  - { from: GetAttr,     to: GetAttr,    name: "Str-typed name → __unbound_method__ Instance with type='bytes', method='fromhex'" }
  - { from: GetAttr,     to: JIT,        name: unbound_handle, returns: MbValue }
  - { from: JIT,         to: Dispatcher, name: "call_spread(unbound_handle, ['48656c6c6f'])" }
  - { from: Dispatcher,  to: Dispatcher, name: "extract type='bytes', method='fromhex'; receiver = first arg" }
  - { from: Dispatcher,  to: Bytes,      name: "fromhex parse '48656c6c6f' → Vec<u8>" }
  - { from: Bytes,       to: Dispatcher, name: bytes_value, returns: MbValue }
  - { from: Dispatcher,  to: JIT,        name: bytes_value }
---
sequenceDiagram
    actor User
    participant JIT
    participant GetAttr
    participant Dispatcher
    participant Bytes
    User->>JIT: bytes.fromhex('48656c6c6f')
    JIT->>GetAttr: mb_getattr('bytes', 'fromhex')
    GetAttr-->>JIT: __unbound_method__ Instance
    JIT->>Dispatcher: call_spread(unbound, args)
    Dispatcher->>Bytes: fromhex parse
    Bytes-->>Dispatcher: bytes_value
    Dispatcher-->>JIT: bytes_value
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->
```yaml
scenarios:
  - id: bytes-basic
    given: bytes/deep_ops_broad.py performs subscript, slice, concat, and repeat
    when: bytes operations run
    then: single subscript returns int while slice, concat, and repeat return bytes
  - id: decode-hex
    given: bytes/decode_hex.py decodes utf-8 bytes and formats raw bytes as hex
    when: decode and hex execute
    then: decode routes to string_ops and hex returns lowercase hexadecimal
  - id: fromhex-classmethod
    given: bytes/fromhex_classmethod.py calls bytes.fromhex
    when: class getattr creates an unbound method
    then: dispatch_str_method routes to bytes_ops fromhex constructor
  - id: predicates-search
    given: bytes/predicates_search.py calls startswith and find
    when: read-only predicate and search paths execute
    then: results match CPython
```

## Tests
<!-- type: test-plan lang: mermaid -->
```mermaid
---
id: runtime-bytes-ops-test-plan
title: Bytes and ByteArray Operations Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> Basic["bytes/deep_ops_broad.py"]
    Runner --> DecodeHex["bytes/decode_hex.py"]
    Runner --> FromHex["bytes/fromhex_classmethod.py"]
    Runner --> ByteArray["bytes/bytearray_mutate.py"]
    Runner --> Predicates["bytes/predicates_search.py"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/bytes_ops.rs
    action: modify
    impl_mode: hand-written
    description: "Bytes (Vec<u8>, immutable) + ByteArray (RwLock<Vec<u8>>) surface; dispatch_bytes_method routing; subscript returns int for single index; .fromhex via class.rs unbound-method path. Hand-written; .hex(sep, bytes_per_sep) 3-arg form is an open gap."
```

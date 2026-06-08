---
id: stdlib-struct-and-binary
title: stdlib struct — Binary Pack / Unpack
crate: mamba
files:
  - crates/mamba/src/runtime/stdlib/struct_mod.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: c964bdaf9
---

# stdlib `struct`

Binary pack / unpack of Python values per a format string DSL
(`'>i'`, `'<2sI'`, `'@10s'`, etc.). Three entry points:
`struct.pack(fmt, *args)`, `struct.unpack(fmt, data)`,
`struct.calcsize(fmt)`.

Three load-bearing invariants:

1. **Endian-prefix binding**: `<` little-endian, `>` big-endian, `=`
   native, `!` network (big), `@` native + native align. The first
   non-letter character (or absent) determines mode; `@` enables
   native alignment padding (relevant for `'@i'` after `'@b'`).
2. **Format codes one-to-one with C types** — `b/B` (i8/u8), `h/H`
   (i16/u16), `i/I` (i32/u32), `q/Q` (i64/u64), `f/d` (f32/f64),
   `s` (n-byte string), `?` (bool), `x` (pad byte), `c` (char).
3. **`unpack` returns a tuple even when the format produces a single
   value** — `struct.unpack('i', data)` returns `(42,)` not `42`.
   Matches CPython.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: struct-types
types:
  StructMod:    { kind: struct, label: "struct_mod.rs" }
  FormatParser: { kind: struct, label: "tokenize fmt string" }
  Endian:       { kind: enum, label: "Little / Big / Native / NativeAlign" }
  ValueModule:  { kind: struct, label: "from runtime::value (NaN-box int / float)" }
  ExceptionMod: { kind: struct, label: "from runtime::exception (struct.error)" }
edges:
  - { from: StructMod, to: FormatParser, kind: owns }
  - { from: FormatParser, to: Endian, kind: owns }
  - { from: StructMod, to: ValueModule, kind: references }
  - { from: StructMod, to: ExceptionMod, kind: references, label: "struct.error" }
---
classDiagram
    class StructMod
    class FormatParser
    class Endian
    class ValueModule
    class ExceptionMod
    StructMod --> FormatParser : owns
    FormatParser --> Endian : owns
    StructMod --> ValueModule : refs
    StructMod --> ExceptionMod : struct.error
```

## Function catalog
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "struct-catalog"
$defs:
  StdlibFnEntry:
    type: object
    properties:
      python_name:    { type: string }
      mb_fn:          { type: string }
      arity:          { type: integer }
      cpython_parity: { type: string, enum: [full, partial, gap] }
      notes:          { type: string }
    required: [python_name, mb_fn, arity, cpython_parity]
  StructCatalog:
    type: array
    items: { $ref: "#/$defs/StdlibFnEntry" }
    examples:
      - - { python_name: "struct.pack",     mb_fn: "mb_struct_pack",     arity: -1, cpython_parity: full }
        - { python_name: "struct.unpack",   mb_fn: "mb_struct_unpack",   arity: 2,  cpython_parity: full }
        - { python_name: "struct.calcsize", mb_fn: "mb_struct_calcsize", arity: 1,  cpython_parity: full }
        - { python_name: "struct.iter_unpack", mb_fn: "(gap)",           arity: 2,  cpython_parity: gap, notes: "iterator over repeated unpacks" }
        - { python_name: "struct.error",    mb_fn: "(class)",            arity: -1, cpython_parity: full, notes: "subclass of Exception" }
  FormatCodes:
    type: array
    items:
      type: object
      properties:
        code:    { type: string }
        c_type:  { type: string }
        size:    { type: integer, description: "bytes" }
      required: [code, c_type, size]
    examples:
      - - { code: "b", c_type: "int8_t",   size: 1 }
        - { code: "B", c_type: "uint8_t",  size: 1 }
        - { code: "h", c_type: "int16_t",  size: 2 }
        - { code: "H", c_type: "uint16_t", size: 2 }
        - { code: "i", c_type: "int32_t",  size: 4 }
        - { code: "I", c_type: "uint32_t", size: 4 }
        - { code: "q", c_type: "int64_t",  size: 8 }
        - { code: "Q", c_type: "uint64_t", size: 8 }
        - { code: "f", c_type: "float",    size: 4 }
        - { code: "d", c_type: "double",   size: 8 }
        - { code: "s", c_type: "char[N]",  size: -1, description: "N from prefix; treated as bytes" }
        - { code: "?", c_type: "bool",     size: 1 }
        - { code: "x", c_type: "pad",      size: 1 }
        - { code: "c", c_type: "char",     size: 1 }
```

## Pack / unpack dispatch
<!-- type: logic lang: mermaid -->

```mermaid
---
id: struct-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "struct.pack(fmt, *args) | struct.unpack(fmt, data)" }
  parse_fmt:    { kind: process,  label: "tokenize fmt string into Endian + (count, code)+" }
  is_pack:      { kind: decision, label: "pack or unpack?" }
  zip_args:     { kind: process,  label: "pack: zip codes with args; per code → bytes (endian-aware)" }
  emit_bytes:   { kind: process,  label: "concat into Vec<u8>; wrap as ObjData::Bytes" }
  read_data:    { kind: process,  label: "unpack: walk codes against bytes; per code → MbValue (endian-aware)" }
  build_tuple:  { kind: process,  label: "wrap values in tuple (always tuple even single)" }
  size_walk:    { kind: process,  label: "calcsize: sum per-code sizes (multiplied by leading count)" }
  done:         { kind: terminal, label: "return Bytes / Tuple / Int" }
edges:
  - { from: enter,       to: parse_fmt }
  - { from: parse_fmt,   to: is_pack }
  - { from: is_pack,     to: zip_args, label: "pack" }
  - { from: is_pack,     to: read_data, label: "unpack" }
  - { from: zip_args,    to: emit_bytes }
  - { from: emit_bytes,  to: done }
  - { from: read_data,   to: build_tuple }
  - { from: build_tuple, to: done }
  - { from: parse_fmt,   to: size_walk, label: "calcsize" }
  - { from: size_walk,   to: done }
---
flowchart TD
    enter([struct op]) --> parse_fmt[tokenize fmt]
    parse_fmt --> is_pack{op?}
    is_pack -->|pack| zip_args[zip + per-code]
    is_pack -->|unpack| read_data[walk + per-code]
    is_pack -->|calcsize| size_walk[sum sizes]
    zip_args --> emit_bytes[Vec u8]
    emit_bytes --> done([Bytes])
    read_data --> build_tuple[tuple wrap]
    build_tuple --> done
    size_walk --> done
```

## Acceptance scenarios
<!-- type: overview lang: markdown -->

```mermaid
---
id: struct-acceptance
actors:
  - { id: User,    kind: actor }
  - { id: Mamba,   kind: system }
  - { id: Fixture, kind: system }
messages:
  - { from: User,    to: Mamba,   name: "run stdlib/struct_round_trip.py" }
  - { from: Mamba,   to: Fixture, name: "struct.unpack('!i', struct.pack('!i', 42))" }
  - { from: Fixture, to: Mamba,   name: "(42,)" }
  - { from: User,    to: Mamba,   name: "run stdlib/struct_endian.py" }
  - { from: Mamba,   to: Fixture, name: "struct.pack('<i', 1) vs struct.pack('>i', 1)" }
  - { from: Fixture, to: Mamba,   name: "different byte orders (LE vs BE)" }
  - { from: User,    to: Mamba,   name: "run stdlib/struct_calcsize.py" }
  - { from: Mamba,   to: Fixture, name: "struct.calcsize('!2sI')" }
  - { from: Fixture, to: Mamba,   name: "6 (2 + 4)" }
  - { from: User,    to: Mamba,   name: "run stdlib/struct_format_string.py" }
  - { from: Mamba,   to: Fixture, name: "struct.pack('!10s', b'hello')" }
  - { from: Fixture, to: Mamba,   name: "10-byte buffer; null-padded" }
---
sequenceDiagram
    actor User
    participant Mamba
    participant Fixture
    User->>Mamba: round_trip
    Mamba->>Fixture: pack then unpack
    Fixture-->>Mamba: tuple
    User->>Mamba: endian
    Mamba->>Fixture: LE vs BE
    Fixture-->>Mamba: byte order
    User->>Mamba: calcsize
    Mamba->>Fixture: 2sI
    Fixture-->>Mamba: 6
    User->>Mamba: 10s
    Mamba->>Fixture: short string
    Fixture-->>Mamba: padded
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
runner: "cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"
fixtures:
  - id: struct_round_trip
    name: "stdlib/struct_round_trip.py"
    paired: "stdlib/struct_round_trip.expected"
  - id: struct_endian
    name: "stdlib/struct_endian.py"
    paired: "stdlib/struct_endian.expected"
  - id: struct_calcsize
    name: "stdlib/struct_calcsize.py"
    paired: "stdlib/struct_calcsize.expected"
  - id: struct_string_field
    name: "stdlib/struct_string_field.py"
    paired: "stdlib/struct_string_field.expected"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/stdlib/struct_mod.rs
    action: modify
    impl_mode: hand-written
    description: "pack / unpack / calcsize over hand-written format-string parser. Hand-written; format DSL parsing is more algorithmic than mechanical → Phase 3 codegen (parser-rules section type) territory if codegen ever wants to handle it."
```

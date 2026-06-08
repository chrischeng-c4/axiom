---
id: c-parser-and-types
type: spec
title: "FFI: C Parser and Type Mapping"
version: 1
spec_type: algorithm
files:
  - ffi/c_parser.rs
  - ffi/c_types.rs
  - ffi/type_map.rs
---

# FFI: C Parser and Type Mapping

## Overview
<!-- type: overview lang: markdown -->

This specification defines the C header file parser, the C type representation model,
and the bidirectional C-to-Mamba type mapping used by the FFI subsystem.

- `ffi/c_parser.rs` (~587 LOC): Parses C function declarations, struct definitions,
  typedefs, and enums from header files. Produces a `CDeclaration` AST.
- `ffi/c_types.rs` (~271 LOC): Defines the `CType` enum representing all supported
  C types (Void, Int, Float, Double, Char, Pointer, Array, Struct, Enum,
  FunctionPointer, Typedef).
- `ffi/type_map.rs` (~393 LOC): Maps `CType` values to Mamba types and vice versa.
  Handles pointer indirection and array decay rules.

## Requirements
<!-- type: overview lang: markdown -->

### R1 - Parse C Function Declarations

```yaml
id: R1
priority: high
```

Parse C function declarations from `.h` header files, extracting return type,
function name, and parameter list (name + type for each parameter). Support
variadic functions (`...`).

### R2 - Parse C Struct and Enum Definitions

```yaml
id: R2
priority: high
```

Parse `struct` definitions with named fields and their types. Parse `enum`
definitions with named variants and optional integer values. Nested struct
references are supported via forward declarations.

### R3 - C Type Representation

```yaml
id: R3
priority: high
```

Represent C types as a `CType` enum with the following variants:

| Variant | Description |
|---------|-------------|
| `Void` | void type |
| `Int` | int (signed/unsigned, width variants) |
| `Float` | float |
| `Double` | double |
| `Char` | char |
| `Pointer(Box<CType>)` | pointer to inner type |
| `Array(Box<CType>, usize)` | fixed-size array |
| `Struct(String)` | named struct reference |
| `Enum(String)` | named enum reference |
| `FunctionPointer` | function pointer with signature |
| `Typedef(String)` | typedef alias (resolved before mapping) |

### R4 - Bidirectional C-to-Mamba Type Mapping

```yaml
id: R4
priority: high
```

Map C types to Mamba types according to these rules:

| C Type | Mamba Type |
|--------|------------|
| `int` | `i64` |
| `float` | `f64` |
| `double` | `f64` |
| `char` | `str` (single character) |
| `char*` | `str` |
| `void*` | `int` (opaque pointer as address) |
| `T*` (other) | `Optional[T]` |
| `struct S` | `dict` |
| `enum E` | `int` |
| `T[]` / `T[N]` | `list[T]` |

Array parameters decay to pointers per C semantics.

### R5 - Typedef Resolution

```yaml
id: R5
priority: medium
```

Resolve typedef aliases before type mapping. Maintain a typedef registry built
during parsing. Recursive typedefs are resolved to their base type.

## Acceptance Criteria
<!-- type: test_plan lang: markdown -->

### Scenario: Parse Simple Function

- **GIVEN** A header containing `int add(int a, int b);`
- **WHEN** The parser processes the header.
- **THEN** A `CDeclaration::Function` is produced with return type `Int`,
  name `add`, and two `Int` parameters.

### Scenario: Map struct pointer

- **GIVEN** A C type `struct Point*`
- **WHEN** The type mapper converts it.
- **THEN** The Mamba type is `Optional[dict]`.

### Scenario: Typedef chain resolution

- **GIVEN** `typedef int myint; typedef myint score;`
- **WHEN** `score` is resolved.
- **THEN** The base type is `CType::Int`.

## Diagrams
<!-- type: overview lang: markdown -->

### Parsing Pipeline

```mermaid
flowchart LR
    Header[".h file"] --> Lexer["Tokenize"]
    Lexer --> Parser["Parse declarations"]
    Parser --> AST["CDeclaration AST"]
    AST --> TypeMap["Type Mapper"]
    TypeMap --> MambaTypes["Mamba type signatures"]
```

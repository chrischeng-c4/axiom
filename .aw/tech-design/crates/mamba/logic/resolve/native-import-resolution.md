---
id: native-import-resolution
title: Native Import Resolution
crate: mamba
files:
  - crates/mamba/src/resolve/pass.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: a47e76722
---

# Native Import Resolution

The resolution pass collaborates with the runtime module registry
(per `runtime/module.md`) to special-case native modules at compile
time. When `import math` parses, the resolver recognises `math` as
native (registered via `mb_register_native_modules`) and emits direct
function-pointer references for `math.sin` / `math.cos` etc., bypassing
the slower `mb_module_getattr` runtime dispatch.

Three load-bearing invariants:

1. **Native module attribute access compiles to direct call** — for
   `math.sin(x)` where `math` is native, the MIR emits
   `Call(SymbolId(mb_math_sin), [x_reg])` rather than the generic
   `mb_module_getattr` + `mb_call_method` chain. This is a 5–10x
   speedup on hot math paths.
2. **Import-from binds names eagerly into the importer's scope** —
   `from math import sqrt as s` adds `s` as a local SymbolId at
   resolution time, NOT at runtime. The runtime sees the symbol
   already bound, just like a local variable.
3. **`from X import *` only fires at runtime** — the resolver can't
   statically know the target module's `__all__` (it might be
   user-defined in the module body). The resolver records a
   star-import marker; the lowerer emits `mb_import_star` which
   populates the local scope at runtime.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: native-import-types
types:
  ResolvePass:        { kind: struct, label: "from resolve/name-resolution" }
  NativeRegistry:     { kind: struct, label: "from runtime::module — native module list" }
  ImportClassifier:   { kind: struct, label: "decides native vs file vs star" }
  SymbolTable:        { kind: struct, label: "from resolve/scope" }
  HirImport:          { kind: struct, label: "from hir::HirModule (resolved imports)" }
edges:
  - { from: ResolvePass,      to: ImportClassifier, kind: owns }
  - { from: ImportClassifier, to: NativeRegistry,   kind: references, label: "is module native?" }
  - { from: ImportClassifier, to: SymbolTable,      kind: references, label: "bind imported names" }
  - { from: ImportClassifier, to: HirImport,        kind: owns,       label: "produces" }
---
classDiagram
    class ResolvePass
    class NativeRegistry
    class ImportClassifier
    class SymbolTable
    class HirImport
    ResolvePass --> ImportClassifier : owns
    ImportClassifier --> NativeRegistry : refs
    ImportClassifier --> SymbolTable : binds
    ImportClassifier --> HirImport : produces
```

## Import classification shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "native-import-types"
$defs:
  ImportKind:
    type: string
    enum: [native_direct, file_runtime, relative_runtime, star_runtime]
  ImportEntry:
    type: object
    properties:
      kind:        { $ref: "#/$defs/ImportKind" }
      module_path: { type: string }
      bindings:
        type: array
        items:
          type: object
          properties:
            local_name:    { type: string, description: "the name in the importer's scope" }
            module_attr:   { type: string, description: "name in the module" }
            symbol_id:     { x-rust-type: SymbolId }
            direct_target:
              oneOf:
                - { type: "null" }
                - { type: string, description: "for native_direct: mb_X function name" }
          required: [local_name, module_attr, symbol_id, direct_target]
    required: [kind, module_path, bindings]
  NativeModuleList:
    description: "Modules registered via mb_register_native_modules"
    type: array
    items: { type: string }
    examples:
      - [math, os, sys, json, re, datetime, hashlib, struct, csv, io,
         random, itertools, functools, collections, string, urllib,
         pathlib, asyncio]
```

## Classification logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: import-classify
entry: enter
nodes:
  enter:        { kind: start,    label: "ResolvePass sees Stmt::Import / ImportFrom" }
  is_relative:  { kind: decision, label: "level > 0 (dotted)?" }
  rel_runtime:  { kind: terminal, label: "ImportKind::relative_runtime — defer to mb_import_relative" }
  is_star:      { kind: decision, label: "from X import * ?" }
  star_runtime: { kind: terminal, label: "ImportKind::star_runtime — defer to mb_import_star" }
  is_native:    { kind: decision, label: "module name in NativeRegistry?" }
  native_bind:  { kind: process,  label: "for each binding: ImportKind::native_direct; direct_target = mb_X" }
  file_bind:    { kind: process,  label: "ImportKind::file_runtime — defer to mb_import / mb_import_from" }
  table_insert: { kind: process,  label: "SymbolTable::define each local_name → SymbolId" }
  done:         { kind: terminal, label: "HirImport built" }
edges:
  - { from: enter,       to: is_relative }
  - { from: is_relative, to: rel_runtime, label: "yes" }
  - { from: is_relative, to: is_star,     label: "no" }
  - { from: is_star,     to: star_runtime, label: "yes" }
  - { from: is_star,     to: is_native,   label: "no" }
  - { from: is_native,   to: native_bind, label: "yes" }
  - { from: is_native,   to: file_bind,   label: "no" }
  - { from: native_bind, to: table_insert }
  - { from: file_bind,   to: table_insert }
  - { from: table_insert, to: done }
  - { from: rel_runtime, to: table_insert }
  - { from: star_runtime, to: done }
---
flowchart TD
    enter([resolve Import]) --> is_relative{relative?}
    is_relative -->|yes| rel_runtime([relative_runtime])
    is_relative -->|no| is_star{star?}
    is_star -->|yes| star_runtime([star_runtime])
    is_star -->|no| is_native{native registry?}
    is_native -->|yes| native_bind[native_direct entries]
    is_native -->|no| file_bind[file_runtime defer]
    native_bind --> table_insert[SymbolTable::define]
    file_bind --> table_insert
    rel_runtime --> table_insert
    table_insert --> done([HirImport])
    star_runtime --> done
```

## Native binding interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: native-import-flow
actors:
  - { id: AST,           kind: system, label: "from math import sqrt" }
  - { id: Resolver,      kind: system }
  - { id: NativeReg,     kind: system, label: "runtime::module" }
  - { id: SymbolTable,   kind: system }
  - { id: HIR,           kind: system, label: "HirImport" }
messages:
  - { from: AST,           to: Resolver,    name: "import / import-from / import *" }
  - { from: Resolver,      to: NativeReg,   name: "is module 'math' native?" }
  - { from: NativeReg,     to: Resolver,    name: "yes; expose attrs as direct mb_math_*" }
  - { from: Resolver,      to: SymbolTable, name: "define 'sqrt' → SymbolId; mark direct_target = 'mb_math_sqrt'" }
  - { from: Resolver,      to: HIR,         name: "HirImport with native_direct entries" }
  - { from: HIR,           to: HIR,         name: "downstream lowerer emits direct Call to mb_math_sqrt" }
---
sequenceDiagram
    participant AST
    participant Resolver
    participant NativeReg
    participant SymbolTable
    participant HIR
    AST->>Resolver: from math import sqrt
    Resolver->>NativeReg: native?
    NativeReg-->>Resolver: yes
    Resolver->>SymbolTable: define sqrt
    Resolver->>HIR: HirImport native_direct
    HIR->>HIR: lowerer emits direct Call
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: native-module-direct-call
    given: imports/native_module.py imports math and calls math.sqrt(4)
    when: resolution sees math in the native registry
    then: it emits native_direct metadata so runtime calls mb_math_sqrt directly
  - id: from-native-alias
    given: imports/from_native_with_alias.py imports sqrt as s from math
    when: import-from resolution runs
    then: s is bound eagerly to the native direct target
  - id: relative-import-runtime
    given: imports/from_relative.py performs from . import sibling inside a package
    when: resolution classifies the import
    then: it records relative_runtime so lowering defers to mb_import_relative
  - id: star-import-runtime
    given: imports/import_star.py performs from mod import *
    when: resolution classifies the import
    then: it records star_runtime so runtime population handles __all__
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: native-import-resolution-test-plan
title: Native Import Resolution Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> NativeDirect["imports/native_module.py / .expected"]
    Runner --> FromNativeAlias["imports/from_native_with_alias.py / .expected"]
    Runner --> Relative["imports/from_relative.py / .expected"]
    Runner --> StarImport["imports/import_star.py / .expected"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/resolve/pass.rs
    action: modify
    impl_mode: hand-written
    description: "Import classification logic — native_direct vs file_runtime vs relative_runtime vs star_runtime; bindings written into SymbolTable + HirImport for downstream lowering. Hand-written; native fast-path is the contract."
```

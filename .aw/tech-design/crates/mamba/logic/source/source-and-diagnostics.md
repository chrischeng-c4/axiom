---
id: source-and-diagnostics
title: Source Spans and Diagnostics
crate: mamba
files:
  - crates/mamba/src/source/mod.rs
  - crates/mamba/src/source/span.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: bf867ab85
---

# Source Spans and Diagnostics

`source/span.rs` defines `FileId`, `Span`, and the `Spanned<T>` wrapper
used by every AST / HIR node to carry source-range info. `source/mod.rs`
defines `SourceMap` — the registry that maps `FileId` to file
contents + path so diagnostics can resolve byte offsets to line+column
+ file name.

Three load-bearing invariants:

1. **`Span { file, start, end }` is byte-offset-based** — line+column
   resolution is done lazily by `SourceMap::resolve_span` only when
   formatting a diagnostic. Storing line / column inline in every
   span would bloat the IR by ~50%.
2. **`Span::merge` requires same `FileId`** — debug-assert prevents
   accidentally merging spans from different files (which would
   produce a meaningless range).
3. **`FileId(0)` is reserved for the synthetic / dummy span** —
   compiler-generated nodes (e.g., desugar output) without a natural
   source location use `Span::dummy()`. Diagnostics format these as
   `<generated>` rather than pointing at file 0.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: source-types
types:
  FileId:        { kind: struct, label: "u32 — index into SourceMap" }
  Span:          { kind: struct, label: "FileId + start + end (byte offsets)" }
  Spanned:       { kind: struct, label: "Spanned<T> { node, span }" }
  SourceMap:     { kind: struct, label: "FileId → SourceFile { name, content }" }
  SourceFile:    { kind: struct, label: "name + content; line-offset cache" }
  Diagnostic:    { kind: struct, label: "level + message + span(s) + suggestions" }
  DiagFormatter: { kind: struct, label: "formats Diagnostic with source context" }
edges:
  - { from: Span,         to: FileId,    kind: references }
  - { from: Spanned,      to: Span,      kind: owns }
  - { from: SourceMap,    to: SourceFile, kind: owns }
  - { from: SourceFile,   to: FileId,    kind: references, label: "indexed by" }
  - { from: Diagnostic,   to: Span,      kind: references }
  - { from: DiagFormatter, to: SourceMap, kind: references, label: "resolve span → line+col" }
  - { from: DiagFormatter, to: Diagnostic, kind: references }
---
classDiagram
    class FileId
    class Span
    class Spanned
    class SourceMap
    class SourceFile
    class Diagnostic
    class DiagFormatter
    Span --> FileId : refs
    Spanned --> Span : owns
    SourceMap --> SourceFile : owns
    SourceFile --> FileId : indexed
    Diagnostic --> Span : refs
    DiagFormatter --> SourceMap : refs
    DiagFormatter --> Diagnostic : refs
```

## Span shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "source-types"
$defs:
  FileId:
    type: integer
    x-rust-type: u32
    description: "0 = dummy / synthetic"
  Span:
    type: object
    x-rust-type: Span
    properties:
      file:  { x-rust-type: FileId }
      start: { type: integer, x-rust-type: u32, description: "byte offset" }
      end:   { type: integer, x-rust-type: u32 }
    required: [file, start, end]
  Spanned:
    type: object
    x-rust-type: "Spanned<T>"
    properties:
      node: { description: "wrapped value" }
      span: { $ref: "#/$defs/Span" }
    required: [node, span]
  Diagnostic:
    type: object
    properties:
      level:    { type: string, enum: [error, warning, note, help] }
      message:  { type: string }
      spans:    { type: array, items: { $ref: "#/$defs/Span" } }
      suggestions:
        type: array
        items:
          type: object
          properties:
            span:        { $ref: "#/$defs/Span" }
            replacement: { type: string }
          required: [span, replacement]
    required: [level, message, spans]
```

## Diagnostic emission logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: diag-emit
entry: enter
nodes:
  enter:        { kind: start,    label: "produce Diagnostic(level, message, spans)" }
  is_dummy:     { kind: decision, label: "every span has FileId(0)?" }
  format_synth: { kind: process,  label: "format as <generated> position" }
  resolve:      { kind: process,  label: "for each span: SourceMap::resolve_span → line, col" }
  format_real:  { kind: process,  label: "format with file:line:col + source-line snippet + caret underline" }
  attach_sugg:  { kind: process,  label: "if suggestions: format with replacement preview" }
  emit:         { kind: terminal, label: "stderr / collected list" }
edges:
  - { from: enter,       to: is_dummy }
  - { from: is_dummy,    to: format_synth, label: "yes" }
  - { from: is_dummy,    to: resolve,      label: "no" }
  - { from: resolve,     to: format_real }
  - { from: format_real, to: attach_sugg }
  - { from: format_synth, to: attach_sugg }
  - { from: attach_sugg, to: emit }
---
flowchart TD
    enter([Diagnostic]) --> is_dummy{all dummy?}
    is_dummy -->|yes| format_synth[<generated>]
    is_dummy -->|no| resolve[SourceMap resolve]
    resolve --> format_real[file:line:col + snippet]
    format_real --> attach_sugg[suggestions]
    format_synth --> attach_sugg
    attach_sugg --> emit([stderr])
```

## Diagnostic flow interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: diag-flow
actors:
  - { id: Compiler,  kind: system, label: "parser / type-checker / resolver" }
  - { id: Span,      kind: system }
  - { id: SourceMap, kind: system }
  - { id: Formatter, kind: system, label: "DiagFormatter" }
  - { id: User,      kind: actor }
messages:
  - { from: Compiler, to: Span,      name: "produce Span on every AST/HIR/MIR node" }
  - { from: Compiler, to: SourceMap, name: "register file (path, content) → FileId" }
  - { from: Compiler, to: Compiler,  name: "type error / parse error / NameError" }
  - { from: Compiler, to: Formatter, name: "Diagnostic { spans }" }
  - { from: Formatter, to: SourceMap, name: "resolve_span → file path + line + col" }
  - { from: Formatter, to: User,     name: "stderr: file:line:col: error: msg + caret + suggestion" }
---
sequenceDiagram
    participant Compiler
    participant Span
    participant SourceMap
    participant Formatter
    actor User
    Compiler->>Span: spans on every node
    Compiler->>SourceMap: register file
    Compiler->>Formatter: emit Diagnostic
    Formatter->>SourceMap: resolve_span
    Formatter-->>User: file:line:col + caret
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->
```yaml
scenarios:
  - id: syntax-diagnostic
    given: syntax_error.py contains an unexpected plus on line 2 column 3
    when: mamba run syntax_error.py is executed
    then: the parser diagnostic renders syntax_error.py:2:3 with a caret span
  - id: type-diagnostic
    given: type_error.py assigns a string literal to an int annotation
    when: type checking emits a typed diagnostic
    then: the formatter resolves the span to line 1 column 9 and reports the type mismatch
  - id: name-diagnostic
    given: name_error.py calls print(undefined)
    when: name resolution emits a diagnostic
    then: the formatter reports the undefined name at the source span
```

## Tests
<!-- type: test-plan lang: mermaid -->
```mermaid
---
id: source-diagnostics-test-plan
title: Source Spans and Diagnostics Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test runtime_tests --release -- {name} --test-threads=1"]
    Runner --> SpanMerge["test_span_merge_same_file"]
    Runner --> Dummy["test_file_id_zero_is_synthetic"]
    Runner --> Caret["test_diagnostic_format_caret"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/source/span.rs
    action: modify
    impl_mode: hand-written
    description: "FileId / Span / Spanned<T> + merge / len / dummy. Hand-written; byte-offset shape is the contract."
  - file: crates/mamba/src/source/mod.rs
    action: modify
    impl_mode: hand-written
    description: "SourceMap (FileId → SourceFile); resolve_span → (file, line, col); line-offset cache. Hand-written."
```

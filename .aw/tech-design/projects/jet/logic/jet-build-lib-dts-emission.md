---
id: projects-jet-logic-jet-build-lib-dts-emission-md
fill_sections: [logic, changes]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: type-declaration-emission
    coverage: partial
    rationale: "Emitting .d.ts declarations + wiring the types field makes jet build --lib output consumable by TypeScript users — the type-declaration leg of library-build-publishing."
---

# jet build --lib: .d.ts Type Declaration Emission

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-build-lib-dts-flow
entry: start
nodes:
  start:       { kind: start,    label: "build_library emits per-entry output (A1)" }
  collect:     { kind: process,  label: "for each library entry source file" }
  parse:       { kind: process,  label: "tree-sitter parse entry + internal re-exports" }
  walk:        { kind: process,  label: "walk top-level exported declarations" }
  classify:    { kind: decision, label: "export is a type/interface/enum decl?" }
  keep_type:   { kind: process,  label: "emit type/interface/enum declaration verbatim" }
  is_typed:    { kind: decision, label: "exported value has explicit type annotation?" }
  emit_decl:   { kind: process,  label: "emit `export declare` signature, drop body" }
  err_untyped: { kind: terminal, label: "error: export lacks explicit type (isolatedDeclarations)" }
  assemble:    { kind: process,  label: "assemble entry .d.ts, preserve external type imports" }
  write:       { kind: process,  label: "write <entry>.d.ts next to JS output" }
  meta:        { kind: process,  label: "set types / exports.types in build metadata" }
  done:        { kind: terminal, label: "DtsResult { per-entry .d.ts paths }" }
edges:
  - { from: start,    to: collect }
  - { from: collect,  to: parse }
  - { from: parse,    to: walk }
  - { from: walk,     to: classify }
  - { from: classify, to: keep_type, label: "type" }
  - { from: classify, to: is_typed,  label: "value" }
  - { from: is_typed, to: emit_decl, label: "yes" }
  - { from: is_typed, to: err_untyped, label: "no" }
  - { from: keep_type, to: assemble }
  - { from: emit_decl, to: assemble }
  - { from: assemble, to: write }
  - { from: write,    to: meta }
  - { from: meta,     to: done }
---
flowchart TD
    start([build_library per-entry output]) --> collect[for each library entry]
    collect --> parse[tree-sitter parse entry + re-exports]
    parse --> walk[walk exported declarations]
    walk --> classify{type/interface/enum decl?}
    classify -->|type| keep_type[emit decl verbatim]
    classify -->|value| is_typed{explicit type annotation?}
    is_typed -->|yes| emit_decl[emit export declare signature]
    is_typed -->|no| err_untyped([error: isolatedDeclarations])
    keep_type --> assemble[assemble .d.ts, keep external type imports]
    emit_decl --> assemble
    assemble --> write[write entry.d.ts]
    write --> meta[set types / exports.types]
    meta --> done([DtsResult per-entry paths])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/bundler/dts.rs"
    action: create
    section: logic
    description: |
      New isolatedDeclarations-style declaration emitter: parse a library entry
      with tree-sitter-typescript, walk top-level exported declarations, emit
      type/interface/enum decls verbatim and `export declare` signatures for
      explicitly-typed exported values, error on untyped exports, and return the
      assembled `<entry>.d.ts` text (external type imports preserved).
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/types.rs"
    action: modify
    section: logic
    description: |
      Add a `declaration: bool` option (emit .d.ts) to the library build options,
      defaulting on for lib mode and off for app mode.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/lib_build.rs"
    action: modify
    section: logic
    description: |
      When declaration emission is enabled, call dts::emit_declarations for each
      entry, write `<entry>.d.ts` alongside the JS output, and record
      types/exports.types in LibBuildResult/metadata.
    impl_mode: hand-written
  - path: "projects/jet/src/cli.rs"
    action: modify
    section: cli
    description: |
      Wire a `--dts/--no-dts` flag (and `[lib].dts` config) into the library
      build options.
    impl_mode: hand-written
  - path: "projects/jet/tests/build/library_dts.rs"
    action: create
    section: unit-test
    description: |
      Tests: a typed fixture library emits `.d.ts` with the right exported
      signatures, the build sets `types`/`exports.types`, and a consumer
      type-checks clean against the emitted declarations.
    impl_mode: hand-written
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract logic (id jet-build-lib-dts-flow) is complete and deterministic: per library entry, parse with tree-sitter, walk exported declarations, branch type-vs-value, emit type/interface/enum verbatim and `export declare` signatures for explicitly-typed values, terminal error on untyped exports (isolatedDeclarations contract), assemble per-entry .d.ts preserving external type imports, write `<entry>.d.ts`, and set types/exports.types. All nodes reachable; both decisions (classify, is_typed) carry labeled branches; terminals (err_untyped, done) are real ends. Scope correct: builds on A1 output; publish/registry is A3.

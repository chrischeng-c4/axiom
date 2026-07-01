---
id: projects-jet-logic-jet-build-lib-dts-preserve-modules-dts-silently-emits-no-d-ts-fi-md
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: type-declaration-emission
    coverage: partial
    rationale: "Library preserve-modules output must honor declaration emission instead of silently dropping --dts."
---

# jet build --lib --dts --preserve-modules: Preserve Module Declarations

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-lib-dts-preserve-modules-flow
entry: start
nodes:
  start: { kind: start, label: "Start preserve_modules library build" }
  collect: { kind: process, label: "Collect reachable source modules" }
  declaration_on: { kind: decision, label: "declaration emission enabled?" }
  emit_dts: { kind: process, label: "Emit one .d.ts per source module" }
  record: { kind: process, label: "Record TypesOutput and EntryOutput::dts paths" }
  emit_js: { kind: process, label: "Emit ESM/CJS preserve-module JS files" }
  skip: { kind: process, label: "Skip declarations when disabled" }
  done: { kind: terminal, label: "Preserve modules build reports matching JS/type outputs" }
edges:
  - { from: start, to: collect }
  - { from: collect, to: declaration_on }
  - { from: declaration_on, to: emit_dts, label: "yes" }
  - { from: emit_dts, to: record }
  - { from: record, to: emit_js }
  - { from: declaration_on, to: skip, label: "no" }
  - { from: skip, to: emit_js }
  - { from: emit_js, to: done }
---
flowchart TD
    start([Start preserve_modules build]) --> collect[Collect reachable source modules]
    collect --> declaration_on{Declaration enabled?}
    declaration_on -->|yes| emit_dts[Emit one d.ts per source module]
    emit_dts --> record[Record TypesOutput and EntryOutput dts paths]
    record --> emit_js[Emit ESM/CJS JS files]
    declaration_on -->|no| skip[Skip declarations]
    skip --> emit_js
    emit_js --> done([JS and type outputs match])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/bundler/lib_build.rs"
    action: modify
    section: logic
    description: |
      Extend build_library_preserve_modules so declaration=true emits one
      .d.ts file per reachable source module, records those files in
      LibBuildResult::types, and attaches the matching .d.ts path to each
      ESM/CJS EntryOutput.
    impl_mode: hand-written
  - path: "projects/jet/tests/build/library_dts.rs"
    action: modify
    section: unit-test
    description: |
      Add a preserve_modules + dts regression covering dual ESM/CJS output,
      sibling .d.ts files for every source module, result.types reporting, and
      EntryOutput::dts links.
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-lib-dts-preserve-modules-tests
requirements:
  R1:
    text: "With preserve_modules and declaration enabled, Jet emits one .d.ts next to each source module's JS output."
    risk: high
    verify: unit
  R2:
    text: "The build result records preserve-module type outputs instead of returning an empty types list."
    risk: high
    verify: unit
  R3:
    text: "Every ESM/CJS preserve-module EntryOutput points at its matching .d.ts path."
    risk: high
    verify: unit
---
requirementDiagram
requirement R1 {
  id: R1
  text: "Preserve modules emits d.ts files"
  risk: High
  verifymethod: Test
}
requirement R2 {
  id: R2
  text: "TypesOutput records preserve d.ts"
  risk: High
  verifymethod: Test
}
requirement R3 {
  id: R3
  text: "EntryOutput links dts"
  risk: High
  verifymethod: Test
}
```

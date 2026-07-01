---
id: projects-jet-logic-jet-build-lib-dts-stops-at-first-isolateddeclarations-violation-md
fill_sections: [logic, changes]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: type-declaration-emission
    coverage: partial
    rationale: "Aggregating isolatedDeclarations diagnostics makes jet build --lib --dts usable for real library migration instead of revealing one declaration error per build."
---

# jet build --lib --dts: Aggregate isolatedDeclarations Diagnostics

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-build-lib-dts-aggregate-isolated-declarations-contract
entry: start
nodes:
  start: { kind: start, label: "Start lib dts emission" }
  collect_modules: { kind: process, label: "Collect entry and reachable local re-export modules" }
  emit_module: { kind: process, label: "Attempt declaration emit for each module" }
  module_ok: { kind: decision, label: "Module emitted cleanly?" }
  store_output: { kind: process, label: "Store pending .d.ts path and text" }
  store_diagnostic: { kind: process, label: "Store source path plus every isolatedDeclarations diagnostic" }
  more_modules: { kind: decision, label: "More modules?" }
  has_diagnostics: { kind: decision, label: "Diagnostics collected?" }
  fail: { kind: terminal, label: "Return one error containing all diagnostics" }
  write: { kind: terminal, label: "Write pending outputs and return entry .d.ts path" }
edges:
  - { from: start, to: collect_modules }
  - { from: collect_modules, to: emit_module }
  - { from: emit_module, to: module_ok }
  - { from: module_ok, to: store_output, label: "yes" }
  - { from: module_ok, to: store_diagnostic, label: "no" }
  - { from: store_output, to: more_modules }
  - { from: store_diagnostic, to: more_modules }
  - { from: more_modules, to: emit_module, label: "yes" }
  - { from: more_modules, to: has_diagnostics, label: "no" }
  - { from: has_diagnostics, to: fail, label: "yes" }
  - { from: has_diagnostics, to: write, label: "no" }
---
flowchart TD
    start([Start lib dts emission]) --> collect_modules[Collect entry and reachable local re-export modules]
    collect_modules --> emit_module[Attempt declaration emit for each module]
    emit_module --> module_ok{Module emitted cleanly?}
    module_ok -->|yes| store_output[Store pending .d.ts path and text]
    module_ok -->|no| store_diagnostic[Store source path plus every isolatedDeclarations diagnostic]
    store_output --> more_modules{More modules?}
    store_diagnostic --> more_modules
    more_modules -->|yes| emit_module
    more_modules -->|no| has_diagnostics{Diagnostics collected?}
    has_diagnostics -->|yes| fail([Return one error containing all diagnostics])
    has_diagnostics -->|no| write([Write pending outputs and return entry .d.ts path])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/bundler/dts.rs"
    action: modify
    section: logic
    description: |
      Change declaration emission from fail-fast to diagnostic aggregation for
      isolatedDeclarations contract errors within one source module, preserving
      successful declaration text only when no diagnostics were collected.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/lib_build.rs"
    action: modify
    section: logic
    description: |
      Aggregate declaration diagnostics across every module reached by one
      library entry's declaration tree, include each module path in the final
      error, and avoid writing partial .d.ts output when any module fails.
    impl_mode: hand-written
  - path: "projects/jet/tests/build/library_dts.rs"
    action: modify
    section: unit-test
    description: |
      Add regression coverage proving a lib build with several invalid exported
      declarations reports all isolatedDeclarations violations in one failure
      instead of stopping at the first invalid module or first invalid export.
    impl_mode: hand-written
```

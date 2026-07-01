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
      Introduce a declaration diagnostic aggregate for one module. Export-boundary
      validation records all isolatedDeclarations errors in source order instead
      of returning at the first missing type/return annotation.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/lib_build.rs"
    action: modify
    section: logic
    description: |
      Buffer declaration output for every module in the entry declaration tree,
      aggregate module-scoped diagnostics, and return one formatted error before
      writing any .d.ts files when diagnostics exist.
    impl_mode: hand-written
  - path: "projects/jet/tests/build/library_dts.rs"
    action: modify
    section: unit-test
    description: |
      Add a regression test with multiple invalid exports across the entry and a
      local re-exported module. The assertion must prove the final error includes
      all invalid symbols and both source files.
    impl_mode: hand-written
```

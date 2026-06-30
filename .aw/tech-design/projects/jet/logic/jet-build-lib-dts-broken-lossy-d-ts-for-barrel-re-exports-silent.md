---
id: projects-jet-logic-jet-build-lib-dts-broken-lossy-d-ts-for-barrel-re-exports-silent-md
fill_sections: [logic, changes]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: type-declaration-emission
    coverage: partial
    rationale: "Barrel re-export .d.ts siblings and explicit isolated-declaration errors keep jet build --lib declarations consumable instead of silently lossy."
---

# jet build --lib --dts: Barrel Re-Export Declaration Completeness

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-dts-barrel-reexport-flow
entry: start
nodes:
  start:         { kind: start,    label: "jet build --lib --dts starts for a library entry" }
  entry_dts:     { kind: process,  label: "emit public entry .d.ts from explicit export boundary" }
  scan:          { kind: process,  label: "scan local export-from specifiers in the entry declaration tree" }
  local_target:  { kind: decision, label: "specifier resolves to an internal source module?" }
  emit_sibling:  { kind: process,  label: "emit sibling .d.ts mirroring the source-relative module path" }
  validate:      { kind: decision, label: "public function/member/value has explicit type boundary?" }
  fail_loud:     { kind: terminal, label: "error: isolatedDeclarations requires explicit return/type" }
  preserve:      { kind: process,  label: "preserve barrel re-export lines in the entry declaration" }
  done:          { kind: terminal, label: "entry .d.ts plus re-export target .d.ts files are present" }
edges:
  - { from: start,        to: entry_dts }
  - { from: entry_dts,    to: scan }
  - { from: scan,         to: local_target }
  - { from: local_target, to: emit_sibling, label: "yes" }
  - { from: local_target, to: preserve,     label: "no/external" }
  - { from: emit_sibling, to: validate }
  - { from: validate,     to: fail_loud,    label: "missing" }
  - { from: validate,     to: preserve,     label: "explicit" }
  - { from: preserve,     to: done }
---
flowchart TD
    start([jet build --lib --dts]) --> entry_dts[emit public entry .d.ts]
    entry_dts --> scan[scan local export-from specifiers]
    scan --> local_target{internal source module?}
    local_target -->|yes| emit_sibling[emit sibling .d.ts]
    local_target -->|no or external| preserve[preserve barrel re-export line]
    emit_sibling --> validate{explicit public type boundary?}
    validate -->|missing| fail_loud([isolatedDeclarations error])
    validate -->|explicit| preserve
    preserve --> done([usable declarations])
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
      Emit a declaration tree for each public library entry: keep the entry
      `.d.ts` as the public `LibBuildResult::types` record, and write sibling
      `.d.ts` files for internal modules reached by local `export ... from`
      barrel re-exports so preserved `export * from "./x"` declarations do not
      dangle.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/dts.rs"
    action: modify
    section: logic
    description: |
      Tighten isolatedDeclarations behavior so exported functions, public class
      methods, and public class fields without explicit return/type annotations
      fail loudly instead of emitting implicit-any declarations.
    impl_mode: hand-written
  - path: "projects/jet/tests/build/library_dts.rs"
    action: modify
    section: unit-test
    description: |
      Add regression coverage for barrel re-export sibling declaration files and
      explicit failures for untyped exported function/class-member return types.
    impl_mode: hand-written
```

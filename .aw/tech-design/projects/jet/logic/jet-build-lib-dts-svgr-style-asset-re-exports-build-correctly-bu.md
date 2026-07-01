---
id: projects-jet-logic-jet-build-lib-dts-svgr-style-asset-re-exports-build-correctly-bu-md
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: type-declaration-emission
    coverage: partial
    rationale: "Library declaration emission must preserve SVGR asset re-export type surfaces instead of emitting transformed runtime aliases with no declaration."
---

# jet build --lib --dts: SVGR Asset Re-export Declarations

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-lib-dts-svgr-asset-reexport-flow
entry: start
nodes:
  start: { kind: start, label: "Start library declaration tree emission" }
  parse: { kind: process, label: "Parse entry top-level export statements" }
  reexport: { kind: decision, label: "Statement is export-from re-export?" }
  emit_verbatim: { kind: process, label: "Emit original re-export line verbatim into .d.ts" }
  external: { kind: decision, label: "Specifier is external package?" }
  local_source: { kind: decision, label: "Relative target is JS/TS source or extensionless source candidate?" }
  chase: { kind: process, label: "Resolve and emit sibling declaration module" }
  skip_asset: { kind: process, label: "Do not resolve/chase non-source asset target" }
  done: { kind: terminal, label: "Declaration output keeps ambient module type resolution" }
edges:
  - { from: start, to: parse }
  - { from: parse, to: reexport }
  - { from: reexport, to: done, label: "no" }
  - { from: reexport, to: emit_verbatim, label: "yes" }
  - { from: emit_verbatim, to: external }
  - { from: external, to: done, label: "yes" }
  - { from: external, to: local_source, label: "no" }
  - { from: local_source, to: chase, label: "source" }
  - { from: local_source, to: skip_asset, label: "asset" }
  - { from: chase, to: done }
  - { from: skip_asset, to: done }
---
flowchart TD
    start([Start declaration tree emission]) --> parse[Parse top-level exports]
    parse --> reexport{export-from re-export?}
    reexport -->|no| done([Declaration complete])
    reexport -->|yes| emit_verbatim[Emit original re-export line verbatim]
    emit_verbatim --> external{External package?}
    external -->|yes| done
    external -->|no| local_source{JS/TS source target?}
    local_source -->|yes| chase[Resolve and emit sibling declaration module]
    local_source -->|no| skip_asset[Skip non-source asset resolution]
    chase --> done
    skip_asset --> done
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
      Keep declaration-tree traversal for local JS/TS re-export targets, but
      skip non-source asset re-export targets such as .svg so the emitted
      .d.ts preserves the asset re-export for ambient module declarations.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/dts.rs"
    action: modify
    section: unit-test
    description: |
      Add emitter-level regression coverage proving SVGR-style
      `export { ReactComponent as Icon } from "./icon.svg"` statements are
      preserved verbatim and do not become runtime SVG aliases.
    impl_mode: hand-written
  - path: "projects/jet/tests/build/library_dts.rs"
    action: modify
    section: unit-test
    description: |
      Add library-build declaration coverage for an entry that re-exports an
      SVG ReactComponent with an ambient `*.svg` module declaration, asserting
      index.d.ts keeps the source-level re-export and no transformed Svg*
      alias leaks into the declaration output.
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-lib-dts-svgr-asset-reexport-tests
requirements:
  R1:
    text: "The declaration emitter preserves SVGR-style SVG ReactComponent re-export lines verbatim."
    risk: high
    verify: unit
  R2:
    text: "Library declaration traversal does not attempt to resolve non-source asset re-export targets as JS/TS modules."
    risk: high
    verify: unit
  R3:
    text: "The emitted entry .d.ts contains no undeclared transformed Svg* runtime aliases for SVG asset re-exports."
    risk: high
    verify: unit
---
requirementDiagram
requirement R1 {
  id: R1
  text: "Preserve SVG ReactComponent re-export"
  risk: High
  verifymethod: Test
}
requirement R2 {
  id: R2
  text: "Do not chase asset re-export targets"
  risk: High
  verifymethod: Test
}
requirement R3 {
  id: R3
  text: "No undeclared Svg aliases in .d.ts"
  risk: High
  verifymethod: Test
}
```

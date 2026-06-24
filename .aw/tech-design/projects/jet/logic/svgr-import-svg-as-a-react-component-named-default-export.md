---
id: projects-jet-logic-svgr-import-svg-as-a-react-component-named-default-export-md
fill_sections: [logic, changes]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: library-build-mode
    coverage: partial
    rationale: "jet SVGR: import .svg as a React component"
---

# jet SVGR: import .svg as a React component

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-svgr
entry: imp
nodes:
  imp: { kind: start,    label: "resolve an import specifier" }
  issvg: { kind: decision, label: "specifier ends with .svg?" }
  normal: { kind: process,  label: "non-svg: existing resolution" }
  ascomp: { kind: decision, label: "imported as component? (config exportType / query)" }
  comp: { kind: process,  label: "parse svg, emit React component (JSX svg + props spread)" }
  url: { kind: process,  label: "svg as asset URL (existing)" }
  done: { kind: terminal, label: "import resolves (component or url)" }
edges:
  - { from: imp,    to: issvg }
  - { from: issvg,  to: normal, label: "no" }
  - { from: issvg,  to: ascomp, label: "yes" }
  - { from: ascomp, to: comp,   label: "component" }
  - { from: ascomp, to: url,    label: "url" }
  - { from: normal, to: done }
  - { from: comp,   to: done }
  - { from: url,    to: done }
---
flowchart TD
    imp([import specifier]) --> issvg{.svg?}
    issvg -->|no| normal[existing resolution]
    issvg -->|yes| ascomp{as component?}
    ascomp -->|component| comp[emit React component from svg]
    ascomp -->|url| url[svg asset URL]
    normal --> done([resolved])
    comp --> done
    url --> done
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/asset/svgr.rs"
    action: create
    section: logic
    description: |
      New SVGR transform: parse an .svg file and emit a React component module (a component returning the SVG as JSX with props/ref forwarded); configurable named (ReactComponent) and/or default export, matching vite-plugin-svgr defaults.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/imports.rs"
    action: modify
    section: logic
    description: |
      Route .svg imports through SVGR when imported as a component; keep asset-URL import otherwise.
    impl_mode: hand-written
  - path: "projects/jet/src/asset/mod.rs"
    action: modify
    section: logic
    description: |
      Wire svgr module; expose config (exportType named/default).
    impl_mode: hand-written
  - path: "projects/jet/tests/build/svgr.rs"
    action: create
    section: unit-test
    description: |
      Tests: importing an .svg yields a component rendering <svg> with forwarded props; named vs default export; plain asset-url import still works.
    impl_mode: hand-written
```


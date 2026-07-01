---
id: projects-jet-logic-jet-stories-build-scss-is-never-compiled-scss-files-copied-verba-md
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: component-workbench
    role: primary
    gap: component-workbench-readiness
    claim: stories-static-export
    coverage: partial
    rationale: "Static stories builds must compile style side-effect imports to real CSS assets and link them from previews instead of emitting raw Sass as JavaScript."
---

# jet stories build: SCSS Static Style Asset Emission

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-stories-static-scss-flow
entry: start
nodes:
  start: { kind: start, label: "Start static story module emit" }
  import_kind: { kind: decision, label: "Import resolves to style file?" }
  js_module: { kind: process, label: "Emit JS module/dependency as before" }
  compile_css: { kind: process, label: "Process CSS/SCSS/Sass through CssPipeline" }
  strip_import: { kind: process, label: "Remove style side-effect import from emitted JS" }
  record_asset: { kind: process, label: "Record emitted CSS asset path" }
  preview_links: { kind: process, label: "Inject CSS links into static preview HTML" }
  done: { kind: terminal, label: "Static output has JS modules plus CSS assets" }
edges:
  - { from: start, to: import_kind }
  - { from: import_kind, to: js_module, label: "no" }
  - { from: js_module, to: done }
  - { from: import_kind, to: compile_css, label: "yes" }
  - { from: compile_css, to: strip_import }
  - { from: strip_import, to: record_asset }
  - { from: record_asset, to: preview_links }
  - { from: preview_links, to: done }
---
flowchart TD
    start([Static story module emit]) --> import_kind{Relative import resolves to .css/.scss/.sass?}
    import_kind -->|no| js_module[Emit JS module/dependency as before]
    js_module --> done([Static output complete])
    import_kind -->|yes| compile_css[Process style via CssPipeline]
    compile_css --> strip_import[Strip style side-effect import from JS]
    strip_import --> record_asset[Record emitted CSS asset]
    record_asset --> preview_links[Link CSS from preview HTML]
    preview_links --> done
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/stories/build.rs"
    action: modify
    section: logic
    description: |
      Teach the static stories emitter to classify relative .css/.scss/.sass
      imports as style assets, compile them through the existing CssPipeline,
      emit deterministic .css files under the static output, remove side-effect
      style imports from emitted JS modules, and link emitted CSS from previews.
    impl_mode: hand-written
  - path: "projects/jet/tests/stories/stories_build.rs"
    action: modify
    section: unit-test
    description: |
      Add a static stories regression where a component imports SCSS with
      nesting/variables; assert the build emits a real CSS asset, does not emit
      .scss.js, removes the JS style import, and links the CSS from previews.
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-stories-static-scss-tests
requirements:
  R1:
    text: "A static stories build compiles SCSS side-effect imports into real .css assets."
    risk: high
    verify: unit
  R2:
    text: "The emitted JS module graph does not contain raw .scss.js modules or style imports that the browser parses as JavaScript."
    risk: high
    verify: unit
  R3:
    text: "Static preview HTML links emitted CSS assets with relative URLs."
    risk: high
    verify: unit
---
requirementDiagram
requirement R1 {
  id: R1
  text: "SCSS emits CSS"
  risk: High
  verifymethod: Test
}
requirement R2 {
  id: R2
  text: "No .scss.js"
  risk: High
  verifymethod: Test
}
requirement R3 {
  id: R3
  text: "Preview links CSS"
  risk: High
  verifymethod: Test
}
```

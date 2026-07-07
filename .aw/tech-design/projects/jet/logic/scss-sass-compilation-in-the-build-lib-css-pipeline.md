---
id: projects-jet-logic-scss-sass-compilation-in-the-build-lib-css-pipeline-md
fill_sections: [logic, changes, e2e-test]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: library-build-mode
    coverage: partial
    rationale: "jet SCSS/Sass compilation in build + lib CSS pipeline"
  - id: bundler-production-build
    role: primary
    gap: bundler-production-readiness
    claim: scss-sass-compilation
    coverage: full
    rationale: "SCSS/Sass compilation is a production-build claim backed by the focused css::scss gate."
---

# jet SCSS/Sass compilation in build + lib CSS pipeline

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-scss
entry: imp
nodes:
  imp: { kind: start,    label: "resolve a style import" }
  isscss: { kind: decision, label: ".scss / .sass?" }
  css: { kind: process,  label: "plain .css: existing lightningcss path" }
  compile: { kind: process,  label: "compile SCSS->CSS (grass): nesting/vars/@use/@import" }
  pipe: { kind: process,  label: "feed compiled CSS into lightningcss pipeline" }
  done: { kind: terminal, label: "CSS in build output (lib: single style.css)" }
edges:
  - { from: imp,     to: isscss }
  - { from: isscss,  to: compile, label: "scss" }
  - { from: isscss,  to: css,     label: "css" }
  - { from: compile, to: pipe }
  - { from: css,     to: pipe }
  - { from: pipe,    to: done }
---
flowchart TD
    imp([style import]) --> isscss{.scss/.sass?}
    isscss -->|scss| compile[compile SCSS to CSS grass]
    isscss -->|css| css[existing lightningcss]
    compile --> pipe[lightningcss pipeline]
    css --> pipe
    pipe --> done([CSS in output])
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: scss_sass_compilation
    capability_id: bundler-production-build
    claim_id: scss-sass-compilation
    name: "SCSS and Sass compilation"
    command: "cargo test -p jet --lib css::scss"
    proves: "SCSS/Sass compilation is covered by the focused css::scss gate."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/Cargo.toml"
    action: modify
    section: logic
    description: |
      Add a Rust Sass implementation dependency (grass) for SCSS/Sass compilation.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/css_bundle/mod.rs"
    action: modify
    section: logic
    description: |
      Compile .scss/.sass to CSS (grass) before the lightningcss pipeline; support nesting, variables, @use/@import partials; flows into the single lib style.css.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/imports.rs"
    action: modify
    section: logic
    description: |
      Resolve .scss/.sass style imports through the new compile step.
    impl_mode: hand-written
  - path: "projects/jet/tests/build/scss.rs"
    action: create
    section: unit-test
    description: |
      Tests: a .scss with nesting + a variable compiles, resolved rules appear in build CSS; lib emits compiled SCSS into style.css; plain .css still works.
    impl_mode: hand-written
```

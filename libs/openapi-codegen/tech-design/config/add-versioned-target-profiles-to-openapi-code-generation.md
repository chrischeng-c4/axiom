---
id: "1569"
summary: Versioned Python, TypeScript, and Rust target profiles for deterministic OpenAPI client artifacts.
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: openapi-codegen-target-profile-dispatch
entry: start
nodes:
  start: { kind: start, label: "generate spec and options" }
  normalize: { kind: process, label: "normalize legacy target default" }
  valid: { kind: decision, label: "target matches language" }
  error: { kind: terminal, label: "return generation error" }
  parse: { kind: process, label: "parse OpenAPI and build semantic IR" }
  emitter: { kind: decision, label: "select language emitter" }
  python: { kind: process, label: "render Python typing and annotations" }
  typescript: { kind: process, label: "render TypeScript compiler and module compatible artifact" }
  rust: { kind: process, label: "render Rust edition compatible artifact" }
  requirements: { kind: process, label: "attach deterministic target requirements" }
  output: { kind: terminal, label: "return GeneratedOutput" }
edges:
  - { from: start, to: normalize }
  - { from: normalize, to: valid }
  - { from: valid, to: error, label: "no" }
  - { from: valid, to: parse, label: "yes" }
  - { from: parse, to: emitter }
  - { from: emitter, to: python, label: "Python" }
  - { from: emitter, to: typescript, label: "TypeScript" }
  - { from: emitter, to: rust, label: "Rust" }
  - { from: python, to: requirements }
  - { from: typescript, to: requirements }
  - { from: rust, to: requirements }
  - { from: requirements, to: output }
---
flowchart TD
  start([generate spec and options]) --> normalize[normalize legacy target default]
  normalize --> valid{target matches language}
  valid -->|no| error([return generation error])
  valid -->|yes| parse[parse OpenAPI and build semantic IR]
  parse --> emitter{select language emitter}
  emitter -->|Python| python[render Python typing and annotations]
  emitter -->|TypeScript| typescript[render TypeScript compiler and module compatible artifact]
  emitter -->|Rust| rust[render Rust edition compatible artifact]
  python --> requirements[attach deterministic target requirements]
  typescript --> requirements
  rust --> requirements
  requirements --> output([return GeneratedOutput])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: behavior
changes:
  - path: "libs/openapi-codegen/src/target.rs"
    action: create
    section: logic
    impl_mode: hand-written
    description: "Public language-valid target profiles and deterministic generated-artifact requirements."
  - path: "libs/openapi-codegen/src/lib.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Expose target profiles and validate them with generation options."
  - path: "libs/openapi-codegen/src/emit/py/pymap.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Render Python typing forms selected by the Python target profile."
  - path: "libs/openapi-codegen/src/emit/py/models_emit.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Render profile-aware Python model aliases and imports."
  - path: "libs/openapi-codegen/src/emit/py/client_emit.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Render profile-aware Python client annotations and imports."
  - path: "libs/openapi-codegen/src/emit/py/runtime_emit.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Keep generated Python runtime annotation behavior compatible with its profile."
  - path: "libs/openapi-codegen/src/emit/ts/client_emit.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Attach TypeScript compiler and module requirements without changing OpenAPI semantics."
  - path: "libs/openapi-codegen/src/emit/rust/models_emit.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Use Rust edition-aware generated identifiers, including Rust 2024 gen."
```

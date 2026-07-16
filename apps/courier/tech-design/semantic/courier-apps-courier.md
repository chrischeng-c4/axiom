---
id: semantic-courier-apps-courier
fill_sections: [schema, unit-test, changes]
---

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "apps/courier/build.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "main"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/courier"
      - path: "apps/courier/build.sh"
        language: "shell"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "apps/courier"
      - path: "apps/courier/install.sh"
        language: "shell"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "apps/courier"
      - path: "apps/courier/llms.txt"
        language: "llms"
        ownership_state: "codegen"
        generator_primitives: ["project_root_llms"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "llms"
          role: "source"
          section_type: "schema"
          domain: "apps/courier"
      - path: "apps/courier/src/llm.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "TOOL"
            kind: "constant"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/courier"
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: unit-test
---
requirementDiagram
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "apps/courier/build.rs"
    action: modify
    section: schema
    impl_mode: hand-written
    anchor: main
    description: |
      Lossless rust-source-unit ownership created from explicit file fillback.
  - path: "apps/courier/build.sh"
    action: modify
    section: schema
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:#4158>"
    description: |
      Project build entrypoint.
  - path: "apps/courier/install.sh"
    action: modify
    section: schema
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:#4158>"
    description: |
      Project installation entrypoint.
  - path: "apps/courier/llms.txt"
    action: modify
    section: schema
    impl_mode: codegen
    description: |
      Generated LLM documentation root artifact.
  - path: "apps/courier/src/llm.rs"
    action: modify
    section: schema
    impl_mode: hand-written
    anchor: TOOL
    description: |
      Exposes the offline agent-oriented documentation.
```

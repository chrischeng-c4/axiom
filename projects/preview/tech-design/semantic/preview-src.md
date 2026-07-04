---
id: semantic-preview-src
summary: Semantic coverage for "projects/preview/src"
capability_refs:
  - id: "gke-uat-preview-environment-rendering"
    role: primary
    claim: "mr-scoped-namespace-projection"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/preview/src`."
fill_sections: [schema, changes]
---

# Semantic TD: preview/src

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "preview/src"
  source_group: "projects/preview/src"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/preview/src/render.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "RenderInput"
            kind: "struct"
            public: true
          - name: "RenderFile"
            kind: "struct"
            public: true
          - name: "render_files"
            kind: "function"
            public: true
          - name: "preview_environment"
            kind: "function"
            public: true
          - name: "cleanup_plan"
            kind: "function"
            public: true
          - name: "mr_comment"
            kind: "function"
            public: true
          - name: "phase_name"
            kind: "function"
            public: false
          - name: "labels"
            kind: "function"
            public: false
          - name: "label_map"
            kind: "function"
            public: false
          - name: "selector"
            kind: "function"
            public: false
          - name: "namespace"
            kind: "function"
            public: false
          - name: "deployment"
            kind: "function"
            public: false
          - name: "service"
            kind: "function"
            public: false
          - name: "route_binding"
            kind: "function"
            public: false
          - name: "yaml"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/preview/src"
      - path: "projects/preview/src/lib.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "model"
            kind: "module"
            public: true
          - name: "render"
            kind: "module"
            public: true
          - name: "router"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/preview/src"
      - path: "projects/preview/src/router.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "RouteBinding"
            kind: "struct"
            public: true
          - name: "RouteRequest"
            kind: "struct"
            public: true
          - name: "ResolvedRoute"
            kind: "struct"
            public: true
          - name: "resolve_route"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/preview/src"
      - path: "projects/preview/src/main.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Cli"
            kind: "struct"
            public: false
          - name: "Command"
            kind: "enum"
            public: false
          - name: "IssueCommand"
            kind: "enum"
            public: false
          - name: "RenderArgs"
            kind: "struct"
            public: false
          - name: "CleanupArgs"
            kind: "struct"
            public: false
          - name: "main"
            kind: "function"
            public: false
          - name: "render"
            kind: "function"
            public: false
          - name: "print_llm"
            kind: "function"
            public: false
          - name: "into_input"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/preview/src"
      - path: "projects/preview/src/model.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "enum_model"]
        symbols:
          - name: "PreviewEnvironment"
            kind: "struct"
            public: true
          - name: "PreviewMetadata"
            kind: "struct"
            public: true
          - name: "Label"
            kind: "struct"
            public: true
          - name: "PreviewSpec"
            kind: "struct"
            public: true
          - name: "RouteSpec"
            kind: "struct"
            public: true
          - name: "GkeSpec"
            kind: "struct"
            public: true
          - name: "PreviewStatus"
            kind: "struct"
            public: true
          - name: "PreviewPhase"
            kind: "enum"
            public: true
          - name: "CleanupPlan"
            kind: "struct"
            public: true
          - name: "CleanupAction"
            kind: "enum"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/preview/src"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/preview/src/render.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-src-render-rs>"
  - path: "projects/preview/src/lib.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-src-lib-rs>"
  - path: "projects/preview/src/router.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-src-router-rs>"
  - path: "projects/preview/src/main.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-src-main-rs>"
  - path: "projects/preview/src/model.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-src-model-rs>"
```

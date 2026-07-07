---
id: semantic-rig-src
summary: Semantic coverage for "projects/rig/src"
capability_refs:
  - id: "scenario-engine"
    role: primary
    claim: "record-contract-check-and-json-report"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/rig/src`."
fill_sections: [schema, changes]
---

# Semantic TD: rig/src

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "rig/src"
  source_group: "projects/rig/src"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/rig/src/config.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "default_qps"
            kind: "function"
            public: false
          - name: "default_workers"
            kind: "function"
            public: false
          - name: "default_duration"
            kind: "function"
            public: false
          - name: "LoadConfig"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "Config"
            kind: "struct"
            public: true
          - name: "load_from"
            kind: "function"
            public: true
          - name: "case_dirs"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/rig/src"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/rig/src/config.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

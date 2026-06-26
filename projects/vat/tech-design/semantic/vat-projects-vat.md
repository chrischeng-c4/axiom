---
id: semantic-vat-projects-vat
summary: Semantic coverage for "projects/vat"
capability_refs:
  - id: "agent-native-gpu-native-dev-containers"
    role: primary
    claim: "host-process-execution-and-gpu-visibility"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/vat`."
fill_sections: [schema, changes]
---

# Semantic TD: vat/projects/vat

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "vat/projects/vat"
  source_group: "projects/vat"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/vat/build.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "main"
            kind: "function"
            public: false
          - name: "stamp_provenance"
            kind: "function"
            public: false
          - name: "short_sha"
            kind: "function"
            public: false
          - name: "compile_pubsub_proto"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/vat/build.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

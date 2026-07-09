---
id: semantic-guard-guard-cli-src-bin
summary: Semantic coverage for "apps/guard/guard-cli/src/bin"
capability_refs:
  - id: static-security-scan
    role: primary
    gap: json-report-envelope
    claim: json-report-envelope
    coverage: full
    rationale: "The standalone guard binary routes parsed commands to the JSON report envelope."
fill_sections: [schema, changes]
---

# Semantic TD: guard/guard-cli/src/bin

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "guard/guard-cli/src/bin"
  source_group: "apps/guard/guard-cli/src/bin"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "apps/guard/guard-cli/src/bin/guard.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "main"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/guard/guard-cli/src/bin"
```


## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "apps/guard/guard-cli/src/bin/guard.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: codegen
```

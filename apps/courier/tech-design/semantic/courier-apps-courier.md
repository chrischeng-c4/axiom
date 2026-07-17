---
id: semantic-courier-apps-courier
fill_sections: [schema, changes]
capability_refs:
  - id: github-issues-proxy
    role: primary
    claim: github-issues-proxy-service
    coverage: full
---

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  coverage_kind: semantic
  evidence:
    source_units:
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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "apps/courier/llms.txt"
    action: modify
    section: schema
    impl_mode: codegen
    description: |
      Generated LLM documentation root artifact.
```

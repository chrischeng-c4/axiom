---
id: semantic-guard-projects-guard
summary: Semantic coverage for "projects/guard"
capability_refs:
  - id: static-security-scan
    role: primary
    gap: json-report-envelope
    claim: json-report-envelope
    coverage: full
    rationale: "Project-root agent/build artifacts keep guard's static scan command discoverable and runnable."
fill_sections: [schema, changes]
---

# Semantic TD: guard/projects/guard

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "guard/projects/guard"
  source_group: "projects/guard"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/guard/build.sh"
        language: "shell"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "projects/guard"
      - path: "projects/guard/install.sh"
        language: "shell"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "projects/guard"
      - path: "projects/guard/llms.txt"
        language: "llms"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "llms"
          role: "source"
          section_type: "schema"
          domain: "projects/guard"
```


## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/guard/build.sh"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: codegen
  - path: "projects/guard/install.sh"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: codegen
  - path: "projects/guard/llms.txt"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: codegen
```

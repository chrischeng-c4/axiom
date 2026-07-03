---
id: semantic-preview-projects-preview
summary: Semantic coverage for "projects/preview"
capability_refs:
  - id: "gke-uat-preview-environment-rendering"
    role: primary
    claim: "mr-scoped-namespace-projection"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/preview`."
fill_sections: [schema, changes]
---

# Semantic TD: preview/projects/preview

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "preview/projects/preview"
  source_group: "projects/preview"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/preview/build.sh"
        language: "shell"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "projects/preview"
      - path: "projects/preview/install.sh"
        language: "shell"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "projects/preview"
      - path: "projects/preview/llms.txt"
        language: "llms"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "llms"
          role: "source"
          section_type: "schema"
          domain: "projects/preview"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/preview/build.sh"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-build-sh>"
  - path: "projects/preview/install.sh"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-install-sh>"
  - path: "projects/preview/llms.txt"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

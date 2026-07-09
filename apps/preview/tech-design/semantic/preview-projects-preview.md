---
id: semantic-preview-projects-preview
summary: Semantic coverage for "apps/preview"
capability_refs:
  - id: "gke-uat-preview-environment-rendering"
    role: primary
    gap: "mr-scoped-namespace-projection"
    claim: "mr-scoped-namespace-projection"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `apps/preview`."
  - id: "preview-external-contracts"
    role: primary
    gap: "ci-template-lifecycle"
    claim: "ci-template-lifecycle"
    coverage: partial
    rationale: "CI/CD lifecycle templates are documented under apps/preview/docs/ci-templates."
fill_sections: [schema, changes]
---

# Semantic TD: preview/apps/preview

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "preview/apps/preview"
  source_group: "apps/preview"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "apps/preview/build.sh"
        language: "shell"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "apps/preview"
      - path: "apps/preview/install.sh"
        language: "shell"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "apps/preview"
      - path: "apps/preview/llms.txt"
        language: "llms"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "llms"
          role: "source"
          section_type: "schema"
          domain: "apps/preview"
      - path: "apps/preview/docs/ci-templates/README.md"
        language: "markdown"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "docs"
          ecosystem: "markdown"
          role: "source"
          section_type: "schema"
          domain: "apps/preview"
      - path: "apps/preview/docs/ci-templates/github-actions-preview.yaml"
        language: "yaml"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "docs"
          ecosystem: "yaml"
          role: "source"
          section_type: "schema"
          domain: "apps/preview"
      - path: "apps/preview/docs/ci-templates/gitlab-ci-preview.yml"
        language: "yaml"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "docs"
          ecosystem: "yaml"
          role: "source"
          section_type: "schema"
          domain: "apps/preview"
      - path: "apps/preview/docs/ci-templates/local-kind-lifecycle.sh"
        language: "shell"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "docs"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "apps/preview"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "apps/preview/build.sh"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-build-sh>"
  - path: "apps/preview/install.sh"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-install-sh>"
  - path: "apps/preview/llms.txt"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/preview/docs/ci-templates/README.md"
    action: add
    section: schema
    description: |
      CI/CD lifecycle template documentation is covered by this semantic TD.
    impl_mode: hand-written
  - path: "apps/preview/docs/ci-templates/github-actions-preview.yaml"
    action: add
    section: schema
    description: |
      GitHub Actions preview lifecycle template is covered by this semantic TD.
    impl_mode: hand-written
  - path: "apps/preview/docs/ci-templates/gitlab-ci-preview.yml"
    action: add
    section: schema
    description: |
      GitLab CI preview lifecycle template is covered by this semantic TD.
    impl_mode: hand-written
  - path: "apps/preview/docs/ci-templates/local-kind-lifecycle.sh"
    action: add
    section: schema
    description: |
      Local kind lifecycle script is covered by this semantic TD.
    impl_mode: hand-written
```

---
id: semantic-vat-projects-vat
summary: Semantic coverage for "projects/vat"
fill_sections: [schema, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: full
    rationale: "Project-root build, install, and LLM context artifacts keep vat usable by agents that rely on the local runner protocol."
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
      - path: "projects/vat/build.sh"
        language: "shell"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "projects/vat"
      - path: "projects/vat/install.sh"
        language: "shell"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "projects/vat"
      - path: "projects/vat/llms.txt"
        language: "llms"
        ownership_state: "codegen"
        generator_primitives: ["project_root_llms"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "llms"
          role: "source"
          section_type: "schema"
          domain: "projects/vat"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/vat/build.sh"
    action: modify
    section: schema
    description: |
      Existing source behavior is delegated to the lossless build-script text source unit.
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:#4158>"
  - path: "projects/vat/install.sh"
    action: modify
    section: schema
    description: |
      Existing source behavior is delegated to the lossless install-script text source unit.
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:#4158>"
  - path: "projects/vat/llms.txt"
    action: modify
    section: schema
    description: |
      Generated TD-first agent context map from project config, README capability map,
      TD root, build/install scripts, and workspace test commands.
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:#4158>"
```

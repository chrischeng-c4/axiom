---
id: semantic-meter-projects-meter
summary: Semantic coverage for "projects/meter"
fill_sections: [schema, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: llm-usage-guide
    claim: llm-usage-guide
    coverage: full
    rationale: "Project-root build, install, and LLM context artifacts keep meter's agent-use-first CLI contract discoverable and runnable."
---

# Semantic TD: meter/projects/meter

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "meter/projects/meter"
  source_group: "projects/meter"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/meter/build.sh"
        language: "shell"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "projects/meter"
      - path: "projects/meter/install.sh"
        language: "shell"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "projects/meter"
      - path: "projects/meter/llms.txt"
        language: "llms"
        ownership_state: "codegen"
        generator_primitives: ["project_root_llms"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "llms"
          role: "source"
          section_type: "schema"
          domain: "projects/meter"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/meter/build.sh"
    action: modify
    section: schema
    description: |
      Existing source behavior is delegated to the lossless build-script text source unit.
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:#4158>"
  - path: "projects/meter/install.sh"
    action: modify
    section: schema
    description: |
      Existing source behavior is delegated to the lossless install-script text source unit.
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:#4158>"
  - path: "projects/meter/llms.txt"
    action: modify
    section: schema
    description: |
      Generated TD-first agent context map from project config, README capability map,
      TD root, build/install scripts, and workspace test commands.
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:#4158>"
```

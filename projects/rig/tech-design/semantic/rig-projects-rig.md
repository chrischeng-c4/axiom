---
id: semantic-rig-projects-rig
summary: Semantic coverage for "projects/rig"
fill_sections: [schema, changes]
capability_refs:
  - id: scenario-engine
    role: primary
    claim: record-contract-check-and-json-report
    coverage: partial
    rationale: "Project-root build and LLM context artifacts keep rig's scenario-engine CLI contract discoverable and runnable."
---

# Semantic TD: rig/projects/rig

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "rig/projects/rig"
  source_group: "projects/rig"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/rig/build.sh"
        language: "shell"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "projects/rig"
      - path: "projects/rig/llms.txt"
        language: "llms"
        ownership_state: "codegen"
        generator_primitives: ["project_root_llms"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "llms"
          role: "source"
          section_type: "schema"
          domain: "projects/rig"
      - path: "projects/rig/install.sh"
        language: "shell"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "projects/rig"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/rig/llms.txt"
    action: modify
    section: schema
    description: |
      Generated TD-first agent context map from project config, README capability map,
      TD root, build script, and workspace test command.
    impl_mode: codegen
  - path: "projects/rig/install.sh"
    action: modify
    section: schema
    description: |
      Project-local source installer dispatch for the rig CLI.
    impl_mode: codegen
```

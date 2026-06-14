---
id: semantic-arena-projects-arena
summary: Semantic coverage for "projects/arena"
fill_sections: [schema, changes]
---

# Semantic TD: arena/projects/arena

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "arena/projects/arena"
  source_group: "projects/arena"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/arena/build.sh"
        language: "shell"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "projects/arena"
      - path: "projects/arena/install.sh"
        language: "shell"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "shell"
          role: "source"
          section_type: "schema"
          domain: "projects/arena"
      - path: "projects/arena/llms.txt"
        language: "llms"
        ownership_state: "codegen"
        generator_primitives: ["project_root_llms"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "llms"
          role: "source"
          section_type: "schema"
          domain: "projects/arena"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/arena/llms.txt"
    action: modify
    section: schema
    description: |
      Generated TD-first agent context map from project config, README capability map,
      TD root, build/install scripts, and workspace test command.
    impl_mode: codegen
```

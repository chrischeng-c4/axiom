---
id: semantic-cap-projects-cap
summary: Semantic coverage for "projects/cap"
fill_sections: [schema, changes]
capability_refs:
  - id: daemon-lifecycle-and-status
    role: primary
    gap: cli-status-and-wait-surfaces
    claim: cli-status-and-wait-surfaces
    coverage: full
    rationale: "Project-root build, install, and LLM context artifacts keep cap operable and legible for CLI lifecycle work."
---

# Semantic TD: cap/projects/cap

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "cap/projects/cap"
  source_group: "projects/cap"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/cap/llms.txt"
        language: "llms"
        ownership_state: "codegen"
        generator_primitives: ["project_root_llms"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "llms"
          role: "source"
          section_type: "schema"
          domain: "projects/cap"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/cap/llms.txt"
    action: modify
    section: schema
    description: |
      Generated TD-first agent context map from project config, README capability map,
      TD root, build/install scripts, and workspace test commands.
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:#4158>"
```

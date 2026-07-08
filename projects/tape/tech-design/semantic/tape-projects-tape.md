---
id: tape-projects-tape
coverage_kind: semantic
capability_refs:
  - id: "cli-interface"
    role: primary
    claim: "tape-cli-convention-and-replay-verbs"
    gap: "tape-cli-convention-and-replay-verbs"
    coverage: partial
    rationale: "The root llms map tells agents how to build, test, and operate the Tape CLI slice."
fill_sections: [overview, schema, changes]
---

# Tape Agent Context

## Overview
<!-- type: overview lang: markdown -->

`projects/tape/llms.txt` is the project-root agent map for the first Tape
service slice. It points agents to README capability intent, tech design,
readiness commands, build/install scripts, tests, and local replay commands.

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  evidence:
    source_units:
      - path: projects/tape/llms.txt
        language: llms
        ownership: codegen
        generator_primitives: [project_root_llms]
        source_evidence_node:
          kind: project_root_context
          project: tape
          inputs:
            - projects/tape/README.md
            - projects/tape/tech-design
            - projects/tape/aw.toml
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/tape/llms.txt
    action: modify
    section: schema
    impl_mode: codegen
    generator_primitives: [project_root_llms]
    description: "Generated TD-first project-root agent context for Tape."
```

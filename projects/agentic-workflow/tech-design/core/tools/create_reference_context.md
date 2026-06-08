---
id: sdd-tools-create-reference-context
fill_sections: [changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create reference context

## Overview
<!-- type: overview lang: markdown -->

Create-reference-context workflow and artifact tool code ownership.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tech-design/core/tools/create_reference_context/definitions.md
    action: create
    impl_mode: codegen
    section: changes
    description: "Source-fragment spec for create-reference-context tool definitions."
  - path: projects/agentic-workflow/tech-design/core/tools/create_reference_context/artifact.md
    action: create
    impl_mode: codegen
    section: changes
    description: "Source-fragment spec for create-reference-context artifact flow."
```

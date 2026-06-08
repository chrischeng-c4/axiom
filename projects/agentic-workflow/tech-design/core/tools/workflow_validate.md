---
id: sdd-tools-workflow-validate
fill_sections: [changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools workflow validate

## Overview
<!-- type: overview lang: markdown -->

Workflow validation gate used by the three-role contract.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tech-design/core/tools/workflow_validate/definition.md
    action: create
    impl_mode: codegen
    section: changes
    description: "Source-fragment spec for the workflow validation tool definition."
```

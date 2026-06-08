---
id: projects-sdd-src-tools-implementation-rs
fill_sections: [overview, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Standardized projects/agentic-workflow/src/tools/implementation.rs

## Overview
<!-- type: overview lang: markdown -->

Implementation support MCP tools are managed by source fragments under
`projects/agentic-workflow/tech-design/core/tools/implementation/`. The split keeps each
fragment below the spec hard size limit while exercising module preamble,
module trailer, symbol, and test-module replacement.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/implementation.rs
    section: source
    action: modify
    impl_mode: codegen
    description: |
      Whole-file HANDWRITE wrapper is removed through source-fragment
      composition. See implementation/*.md for the concrete generated regions.
```

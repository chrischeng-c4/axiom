---
id: sdd-tools-phase-transition
fill_sections: [changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools phase transition

## Overview
<!-- type: overview lang: markdown -->

Phase parsing, canonical string conversion, ordering, and transition validation for the legacy SDD change lifecycle.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tech-design/core/tools/phase_transition/source.md
    action: create
    impl_mode: codegen
    section: changes
    description: "Source-fragment spec that owns the phase transition logic block."
  - path: projects/agentic-workflow/src/tools/phase_transition.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:missing-generator:sdd-phase-transition-logic>"
    description: "Replace the tracked HANDWRITE phase transition logic with a source-generated CODEGEN block."
```

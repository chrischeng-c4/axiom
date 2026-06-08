---
id: projects-sdd-src-tools-revise-change-docs-rs
fill_sections: [changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Standardized projects/agentic-workflow/src/tools/revise_change_docs.rs

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/revise_change_docs.rs
    action: modify
    section: changes
    impl_mode: hand-written
    description: |
      Existing source claimed by `aw standardize run`. The code is wrapped
      in a tracked HANDWRITE block until deterministic generator coverage can
      replace it with CODEGEN.
```

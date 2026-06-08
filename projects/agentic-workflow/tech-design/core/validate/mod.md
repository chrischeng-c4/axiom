---
id: projects-sdd-src-validate-mod-rs
fill_sections: [changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Standardized projects/agentic-workflow/src/validate/mod.rs

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/mod.rs
    action: modify
    section: changes
    impl_mode: hand-written
    description: |
      Existing source claimed by `aw standardize run`. The code is wrapped
      in a tracked HANDWRITE block until deterministic generator coverage can
      replace it with CODEGEN.
```

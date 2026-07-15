---
id: projects-sdd-src-tools-platform-sync-rs
fill_sections: [changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: agent-first-cli-product-model
    claim: agent-first-cli-product-model
    coverage: full
    rationale: "Issue and platform-sync tool TDs expose AW workflow state through configured issue platforms."
---

# Standardized apps/agentic-workflow/src/tools/platform_sync.rs

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/tools/platform_sync.rs
    action: modify
    section: changes
    impl_mode: hand-written
    description: |
      Existing source claimed by `aw standardize run`. The code is wrapped
      in a tracked HANDWRITE block until deterministic generator coverage can
      replace it with CODEGEN.
```

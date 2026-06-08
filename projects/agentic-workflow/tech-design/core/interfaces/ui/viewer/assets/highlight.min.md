---
id: projects-sdd-src-ui-viewer-assets-highlight-min-js
fill_sections: [changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Standardized projects/agentic-workflow/src/ui/viewer/assets/highlight.min.js

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/ui/viewer/assets/highlight.min.js
    action: modify
    section: changes
    impl_mode: hand-written
    description: |
      Vendored minified Highlight.js browser bundle. This file is owned as a
      tracked HANDWRITE vendor bundle; regenerate it from the upstream
      package, not from project TD templates.
```

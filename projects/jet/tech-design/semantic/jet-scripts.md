---
id: semantic-jet-scripts
summary: Semantic coverage for "projects/jet/scripts"
capability_refs:
  - id: "rust-native-frontend-toolchain"
    role: primary
    claim: "production-replacement-readiness"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/jet/scripts`."
fill_sections: [logic, changes]
---

# Semantic TD: jet/scripts

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: frontend-jet-scripts-logic
entry: preserve_frontend_behavior
nodes:
  preserve_frontend_behavior:
    kind: start
    label: "Preserve observed frontend source behavior"
  classify_frontend_ast:
    kind: process
    label: "Map FE ecosystem AST group \"projects/jet/scripts\" to logic emitter primitives"
  generator_gap:
    kind: terminal
    label: "Promote from semantic coverage to deterministic frontend codegen"
edges:
  - { from: preserve_frontend_behavior, to: classify_frontend_ast }
  - { from: classify_frontend_ast, to: generator_gap }
---
flowchart TD
  preserve_frontend_behavior --> classify_frontend_ast --> generator_gap
```

<!-- frontend_source_evidence
- projects/jet/scripts/compare-basic-builds.mjs
- projects/jet/scripts/compare-dom-build-corpus.mjs
- projects/jet/scripts/compare-prod-static-serve.mjs
- projects/jet/scripts/compare-pkg-management.mjs
- projects/jet/scripts/verify-browser-bridge-replacement.mjs
-->

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/scripts/compare-basic-builds.mjs"
    action: modify
    section: logic
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-jet-scripts-compare-basic-builds-mjs>"
  - path: "projects/jet/scripts/compare-dom-build-corpus.mjs"
    action: modify
    section: logic
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-jet-scripts-compare-dom-build-corpus-mjs>"
  - path: "projects/jet/scripts/compare-prod-static-serve.mjs"
    action: modify
    section: logic
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-jet-scripts-compare-prod-static-serve-mjs>"
  - path: "projects/jet/scripts/compare-pkg-management.mjs"
    action: modify
    section: logic
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-jet-scripts-compare-pkg-management-mjs>"
  - path: "projects/jet/scripts/verify-browser-bridge-replacement.mjs"
    action: modify
    section: logic
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-jet-scripts-verify-browser-bridge-replacement-mjs>"
```

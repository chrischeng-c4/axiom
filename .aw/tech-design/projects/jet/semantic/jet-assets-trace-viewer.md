---
id: semantic-jet-assets-trace-viewer
summary: Semantic coverage for "projects/jet/assets/trace-viewer"
fill_sections: [design-token, logic, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/assets/trace-viewer

## Design Token
<!-- type: design-token lang: yaml -->

```yaml
frontend_semantic:
  section_type: "design-token"
  key: "jet/assets/trace-viewer"
  source_group: "projects/jet/assets/trace-viewer"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/assets/trace-viewer/viewer.css"
        language: "stylesheet"
        ownership_state: "codegen"
        generator_primitives: ["frontend_style-surface", "td_section_design_token"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "style"
          role: "style"
          section_type: "design-token"
          domain: "projects/jet/assets/trace-viewer"
          workspace_root: "projects/jet/assets/trace-viewer"
        frontend_node:
          workspace_root: "projects/jet/assets/trace-viewer"
          role: "style"
          section_type: "design-token"
          artifact_kind: "style-surface"
  frontend_ast:
    nodes:
      - path: "projects/jet/assets/trace-viewer/viewer.css"
        workspace_root: "projects/jet/assets/trace-viewer"
        role: "style"
        artifact_kind: "style-surface"
        section_type: "design-token"
```

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: frontend-jet-assets-trace-viewer-logic
entry: preserve_frontend_behavior
nodes:
  preserve_frontend_behavior:
    kind: start
    label: "Preserve observed frontend source behavior"
  classify_frontend_ast:
    kind: process
    label: "Map FE ecosystem AST group \"projects/jet/assets/trace-viewer\" to logic emitter primitives"
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
- projects/jet/assets/trace-viewer/viewer.js
-->

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/assets/trace-viewer/viewer.js"
    action: modify
    section: logic
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/assets/trace-viewer/viewer.css"
    action: modify
    section: design-token
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

---
id: semantic-agentic-workflow-ui-viewer-assets
summary: Semantic coverage for "projects/agentic-workflow/src/ui/viewer/assets"
fill_sections: [design-token, logic, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "This semantic TD covers AW core/client model source behavior and shared workflow domain primitives."
---

# Semantic TD: agentic-workflow/ui/viewer/assets

## Design Token
<!-- type: design-token lang: yaml -->

```yaml
frontend_semantic:
  section_type: "design-token"
  key: "agentic-workflow/ui/viewer/assets"
  source_group: "projects/agentic-workflow/src/ui/viewer/assets"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/ui/viewer/assets/highlight.min.css"
        language: "stylesheet"
        ownership_state: "codegen"
        generator_primitives: ["frontend_style-surface", "td_section_design_token"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "style"
          role: "style"
          section_type: "design-token"
          domain: "projects/agentic-workflow/src/ui/viewer/assets"
          workspace_root: "projects/agentic-workflow/src/ui/viewer/assets"
        frontend_node:
          workspace_root: "projects/agentic-workflow/src/ui/viewer/assets"
          role: "style"
          section_type: "design-token"
          artifact_kind: "style-surface"
      - path: "projects/agentic-workflow/src/ui/viewer/assets/styles.css"
        language: "stylesheet"
        ownership_state: "codegen"
        generator_primitives: ["frontend_style-surface", "td_section_design_token"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "style"
          role: "style"
          section_type: "design-token"
          domain: "projects/agentic-workflow/src/ui/viewer/assets"
          workspace_root: "projects/agentic-workflow/src/ui/viewer/assets"
        frontend_node:
          workspace_root: "projects/agentic-workflow/src/ui/viewer/assets"
          role: "style"
          section_type: "design-token"
          artifact_kind: "style-surface"
  frontend_ast:
    nodes:
      - path: "projects/agentic-workflow/src/ui/viewer/assets/highlight.min.css"
        workspace_root: "projects/agentic-workflow/src/ui/viewer/assets"
        role: "style"
        artifact_kind: "style-surface"
        section_type: "design-token"
      - path: "projects/agentic-workflow/src/ui/viewer/assets/styles.css"
        workspace_root: "projects/agentic-workflow/src/ui/viewer/assets"
        role: "style"
        artifact_kind: "style-surface"
        section_type: "design-token"
```

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: frontend-agentic-workflow-ui-viewer-assets-logic
entry: preserve_frontend_behavior
nodes:
  preserve_frontend_behavior:
    kind: start
    label: "Preserve observed frontend source behavior"
  classify_frontend_ast:
    kind: process
    label: "Map FE ecosystem AST group \"projects/agentic-workflow/src/ui/viewer/assets\" to logic emitter primitives"
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
- projects/agentic-workflow/src/ui/viewer/assets/app.js
-->

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/agentic-workflow/src/ui/viewer/assets/highlight.min.css"
    action: modify
    section: design-token
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/ui/viewer/assets/styles.css"
    action: modify
    section: design-token
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/ui/viewer/assets/app.js"
    action: modify
    section: logic
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

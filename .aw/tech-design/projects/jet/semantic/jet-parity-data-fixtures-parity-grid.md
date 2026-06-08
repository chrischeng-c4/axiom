---
id: semantic-jet-parity-data-fixtures-parity-grid
summary: Semantic coverage for "projects/jet/parity/data/fixtures/parity-grid"
fill_sections: [component, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/parity/data/fixtures/parity-grid

## Component
<!-- type: component lang: yaml -->

```yaml
frontend_semantic:
  section_type: "component"
  key: "jet/parity/data/fixtures/parity-grid"
  source_group: "projects/jet/parity/data/fixtures/parity-grid"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/parity/data/fixtures/parity-grid/index.tsx"
        language: "typescript"
        ownership_state: "codegen"
        generator_primitives: ["frontend_component", "td_section_component", "ts_component"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "typescript-jsx"
          role: "component"
          section_type: "component"
          domain: "projects/jet/parity/data/fixtures/parity-grid"
          workspace_root: "projects/jet/parity/data/fixtures/parity-grid"
          component: "ParityGrid"
        frontend_node:
          workspace_root: "projects/jet/parity/data/fixtures/parity-grid"
          role: "component"
          section_type: "component"
          artifact_kind: "component"
          component: "ParityGrid"
  frontend_ast:
    nodes:
      - path: "projects/jet/parity/data/fixtures/parity-grid/index.tsx"
        workspace_root: "projects/jet/parity/data/fixtures/parity-grid"
        role: "component"
        artifact_kind: "component"
        section_type: "component"
        component: "ParityGrid"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/parity/data/fixtures/parity-grid/index.tsx"
    action: modify
    section: component
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

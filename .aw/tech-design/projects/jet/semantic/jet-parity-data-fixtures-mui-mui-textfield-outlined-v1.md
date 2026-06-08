---
id: semantic-jet-parity-data-fixtures-mui-mui-textfield-outlined-v1
summary: Semantic coverage for "projects/jet/parity/data/fixtures/mui/mui-textfield-outlined-v1"
fill_sections: [component, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/parity/data/fixtures/mui/mui-textfield-outlined-v1

## Component
<!-- type: component lang: yaml -->

```yaml
frontend_semantic:
  section_type: "component"
  key: "jet/parity/data/fixtures/mui/mui-textfield-outlined-v1"
  source_group: "projects/jet/parity/data/fixtures/mui/mui-textfield-outlined-v1"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/parity/data/fixtures/mui/mui-textfield-outlined-v1/index.tsx"
        language: "typescript"
        ownership_state: "codegen"
        generator_primitives: ["frontend_component", "service_method", "td_section_component", "ts_component"]
        symbols:
          - name: "MuiTextFieldOutlinedV1"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "frontend"
          ecosystem: "typescript-jsx"
          role: "component"
          section_type: "component"
          domain: "projects/jet/parity/data/fixtures/mui/mui-textfield-outlined-v1"
          workspace_root: "projects/jet/parity/data/fixtures/mui/mui-textfield-outlined-v1"
          component: "MuiTextfieldOutlinedV1"
        frontend_node:
          workspace_root: "projects/jet/parity/data/fixtures/mui/mui-textfield-outlined-v1"
          role: "component"
          section_type: "component"
          artifact_kind: "component"
          component: "MuiTextfieldOutlinedV1"
  frontend_ast:
    nodes:
      - path: "projects/jet/parity/data/fixtures/mui/mui-textfield-outlined-v1/index.tsx"
        workspace_root: "projects/jet/parity/data/fixtures/mui/mui-textfield-outlined-v1"
        role: "component"
        artifact_kind: "component"
        section_type: "component"
        component: "MuiTextfieldOutlinedV1"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/parity/data/fixtures/mui/mui-textfield-outlined-v1/index.tsx"
    action: modify
    section: component
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

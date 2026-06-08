---
id: semantic-jet-parity-oracle-fixtures
summary: Semantic coverage for "projects/jet/parity/oracle/fixtures"
fill_sections: [component, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/parity/oracle/fixtures

## Component
<!-- type: component lang: yaml -->

```yaml
frontend_semantic:
  section_type: "component"
  key: "jet/parity/oracle/fixtures"
  source_group: "projects/jet/parity/oracle/fixtures"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/parity/oracle/fixtures/mui-button.tsx"
        language: "typescript"
        ownership_state: "codegen"
        generator_primitives: ["frontend_component", "service_method", "td_section_component", "ts_component"]
        symbols:
          - name: "MuiButton"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "frontend"
          ecosystem: "typescript-jsx"
          role: "component"
          section_type: "component"
          domain: "projects/jet/parity/oracle/fixtures"
          workspace_root: "projects/jet/parity/oracle/fixtures"
          component: "MuiButton"
        frontend_node:
          workspace_root: "projects/jet/parity/oracle/fixtures"
          role: "component"
          section_type: "component"
          artifact_kind: "component"
          component: "MuiButton"
  frontend_ast:
    nodes:
      - path: "projects/jet/parity/oracle/fixtures/mui-button.tsx"
        workspace_root: "projects/jet/parity/oracle/fixtures"
        role: "component"
        artifact_kind: "component"
        section_type: "component"
        component: "MuiButton"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/parity/oracle/fixtures/mui-button.tsx"
    action: modify
    section: component
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

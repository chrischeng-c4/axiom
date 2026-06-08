---
id: semantic-jet-tests-fixtures
summary: Semantic coverage for "projects/jet/tests/fixtures"
fill_sections: [component, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/tests/fixtures

## Component
<!-- type: component lang: yaml -->

```yaml
frontend_semantic:
  section_type: "component"
  key: "jet/tests/fixtures"
  source_group: "projects/jet/tests/fixtures"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/tests/fixtures/production-build-regression/package.json"
        language: "json"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "test_case"]
        symbols:
          - name: "jet-production-build-regression"
            kind: "package"
            public: false
        source_evidence_node:
          layer: "frontend"
          ecosystem: "typescript-jsx"
          role: "fixture"
          section_type: "component"
          domain: "projects/jet/tests/fixtures"
          workspace_root: "projects/jet/tests/fixtures/production-build-regression"
          component: "ProductionBuildRegressionFixture"
      - path: "projects/jet/tests/fixtures/production-build-regression/src/main.tsx"
        language: "typescript"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "frontend_component", "test_case", "ts_component"]
        symbols:
          - name: "ProductionBuildFixture"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "frontend"
          ecosystem: "typescript-jsx"
          role: "component"
          section_type: "component"
          domain: "projects/jet/tests/fixtures"
          workspace_root: "projects/jet/tests/fixtures/production-build-regression"
          component: "ProductionBuildRegressionFixture"
        frontend_node:
          workspace_root: "projects/jet/tests/fixtures/production-build-regression"
          role: "component"
          section_type: "component"
          artifact_kind: "component"
          component: "ProductionBuildRegressionFixture"
      - path: "projects/jet/tests/fixtures/production-build-regression/src/message.cjs"
        language: "javascript"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        symbols:
          - name: "message"
            kind: "constant"
            public: false
        source_evidence_node:
          layer: "frontend"
          ecosystem: "javascript"
          role: "fixture"
          section_type: "component"
          domain: "projects/jet/tests/fixtures"
          workspace_root: "projects/jet/tests/fixtures/production-build-regression"
          component: "ProductionBuildRegressionFixture"
      - path: "projects/jet/tests/fixtures/production-build-regression/src/style.css"
        language: "css"
        ownership_state: "codegen"
        generator_primitives: ["test_case"]
        symbols:
          - name: "production-fixture"
            kind: "css_class"
            public: false
        source_evidence_node:
          layer: "frontend"
          ecosystem: "css"
          role: "fixture"
          section_type: "component"
          domain: "projects/jet/tests/fixtures"
          workspace_root: "projects/jet/tests/fixtures/production-build-regression"
          component: "ProductionBuildRegressionFixture"
      - path: "projects/jet/tests/fixtures/tsx_to_rust_counter.tsx"
        language: "typescript"
        ownership_state: "codegen"
        generator_primitives: ["frontend_component", "service_method", "td_section_component", "test_case", "ts_component", "ts_type_surface"]
        symbols:
          - name: "CounterProps"
            kind: "interface"
            public: true
          - name: "Counter"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "frontend"
          ecosystem: "typescript-jsx"
          role: "component"
          section_type: "component"
          domain: "projects/jet/tests/fixtures"
          workspace_root: "projects/jet/tests/fixtures"
          component: "TsxToRustCounter"
        frontend_node:
          workspace_root: "projects/jet/tests/fixtures"
          role: "component"
          section_type: "component"
          artifact_kind: "component"
          component: "TsxToRustCounter"
      - path: "projects/jet/tests/fixtures/tsx_to_rust_boolean_literal_state.tsx"
        language: "typescript"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "frontend_component", "td_section_component", "test_case", "ts_component", "ts_type_surface"]
        symbols:
          - name: "SandboxProps"
            kind: "interface"
            public: true
          - name: "[on, setOn]"
            kind: "constant"
            public: true
          - name: "[count, setCount]"
            kind: "constant"
            public: true
          - name: "[name, setName]"
            kind: "constant"
            public: true
        source_evidence_node:
          layer: "frontend"
          ecosystem: "typescript-jsx"
          role: "component"
          section_type: "component"
          domain: "projects/jet/tests/fixtures"
          workspace_root: "projects/jet/tests/fixtures"
          component: "TsxToRustBooleanLiteralState"
        frontend_node:
          workspace_root: "projects/jet/tests/fixtures"
          role: "component"
          section_type: "component"
          artifact_kind: "component"
          component: "TsxToRustBooleanLiteralState"
      - path: "projects/jet/tests/fixtures/tsx_to_rust_i18n_copy_constants.tsx"
        language: "typescript"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "frontend_component", "service_method", "td_section_component", "test_case", "ts_component", "ts_type_surface"]
        symbols:
          - name: "AppProps"
            kind: "interface"
            public: true
          - name: "COPY"
            kind: "constant"
            public: true
          - name: "GREETING"
            kind: "constant"
            public: true
          - name: "App"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "frontend"
          ecosystem: "typescript-jsx"
          role: "component"
          section_type: "component"
          domain: "projects/jet/tests/fixtures"
          workspace_root: "projects/jet/tests/fixtures"
          component: "TsxToRustI18nCopyConstants"
        frontend_node:
          workspace_root: "projects/jet/tests/fixtures"
          role: "component"
          section_type: "component"
          artifact_kind: "component"
          component: "TsxToRustI18nCopyConstants"
      - path: "projects/jet/tests/fixtures/tsx_to_rust_toggle.tsx"
        language: "typescript"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "frontend_component", "td_section_component", "test_case", "ts_component", "ts_type_surface"]
        symbols:
          - name: "ToggleProps"
            kind: "interface"
            public: true
          - name: "[on, setOn]"
            kind: "constant"
            public: true
        source_evidence_node:
          layer: "frontend"
          ecosystem: "typescript-jsx"
          role: "component"
          section_type: "component"
          domain: "projects/jet/tests/fixtures"
          workspace_root: "projects/jet/tests/fixtures"
          component: "TsxToRustToggle"
        frontend_node:
          workspace_root: "projects/jet/tests/fixtures"
          role: "component"
          section_type: "component"
          artifact_kind: "component"
          component: "TsxToRustToggle"
  frontend_ast:
    nodes:
      - path: "projects/jet/tests/fixtures/tsx_to_rust_counter.tsx"
        workspace_root: "projects/jet/tests/fixtures"
        role: "component"
        artifact_kind: "component"
        section_type: "component"
        component: "TsxToRustCounter"
      - path: "projects/jet/tests/fixtures/tsx_to_rust_boolean_literal_state.tsx"
        workspace_root: "projects/jet/tests/fixtures"
        role: "component"
        artifact_kind: "component"
        section_type: "component"
        component: "TsxToRustBooleanLiteralState"
      - path: "projects/jet/tests/fixtures/tsx_to_rust_i18n_copy_constants.tsx"
        workspace_root: "projects/jet/tests/fixtures"
        role: "component"
        artifact_kind: "component"
        section_type: "component"
        component: "TsxToRustI18nCopyConstants"
      - path: "projects/jet/tests/fixtures/tsx_to_rust_toggle.tsx"
        workspace_root: "projects/jet/tests/fixtures"
        role: "component"
        artifact_kind: "component"
        section_type: "component"
        component: "TsxToRustToggle"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/tests/fixtures/tsx_to_rust_counter.tsx"
    action: modify
    section: component
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/fixtures/tsx_to_rust_boolean_literal_state.tsx"
    action: modify
    section: component
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/fixtures/tsx_to_rust_i18n_copy_constants.tsx"
    action: modify
    section: component
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/fixtures/tsx_to_rust_toggle.tsx"
    action: modify
    section: component
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

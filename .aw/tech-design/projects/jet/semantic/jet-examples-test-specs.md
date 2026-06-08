---
id: semantic-jet-examples-test-specs
summary: Semantic coverage for "projects/jet/examples/test-specs"
fill_sections: [tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/examples/test-specs

## Tests
<!-- type: tests lang: yaml -->

```yaml
frontend_semantic:
  section_type: "tests"
  key: "jet/examples/test-specs"
  source_group: "projects/jet/examples/test-specs"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/examples/test-specs/page-object-pattern.spec.ts"
        language: "typescript"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "frontend_test", "td_section_tests", "ts_type_surface"]
        symbols:
          - name: "JetTest"
            kind: "type"
            public: true
          - name: "Page"
            kind: "type"
            public: true
          - name: "LoginPage"
            kind: "class"
            public: true
          - name: "testWithLogin"
            kind: "constant"
            public: true
        source_evidence_node:
          layer: "frontend"
          ecosystem: "typescript-jsx"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/examples/test-specs"
          workspace_root: "projects/jet/examples/test-specs"
        frontend_node:
          workspace_root: "projects/jet/examples/test-specs"
          role: "test"
          section_type: "tests"
          artifact_kind: "test"
  frontend_ast:
    nodes:
      - path: "projects/jet/examples/test-specs/page-object-pattern.spec.ts"
        workspace_root: "projects/jet/examples/test-specs"
        role: "test"
        artifact_kind: "test"
        section_type: "tests"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/examples/test-specs/page-object-pattern.spec.ts"
    action: modify
    section: tests
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

---
id: semantic-jet-tests-fixtures-jet-test-api-compat
summary: Semantic coverage for "projects/jet/tests/fixtures/jet-test-api-compat"
fill_sections: [e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/tests/fixtures/jet-test-api-compat

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
frontend_semantic:
  section_type: "tests"
  key: "jet/tests/fixtures/jet-test-api-compat"
  source_group: "projects/jet/tests/fixtures/jet-test-api-compat"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/tests/fixtures/jet-test-api-compat/lifecycle-and-snapshot.spec.js"
        language: "javascript"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "frontend_test", "td_section_tests", "test_case"]
        symbols:
          - name: "order"
            kind: "constant"
            public: true
        source_evidence_node:
          layer: "frontend"
          ecosystem: "javascript"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests/fixtures/jet-test-api-compat"
          workspace_root: "projects/jet/tests/fixtures/jet-test-api-compat"
        frontend_node:
          workspace_root: "projects/jet/tests/fixtures/jet-test-api-compat"
          role: "test"
          section_type: "tests"
          artifact_kind: "test"
  frontend_ast:
    nodes:
      - path: "projects/jet/tests/fixtures/jet-test-api-compat/lifecycle-and-snapshot.spec.js"
        workspace_root: "projects/jet/tests/fixtures/jet-test-api-compat"
        role: "test"
        artifact_kind: "test"
        section_type: "tests"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/tests/fixtures/jet-test-api-compat/lifecycle-and-snapshot.spec.js"
    action: modify
    section: e2e-test
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

---
id: semantic-jet-tests-fixtures-playwright-compat
summary: Semantic coverage for "projects/jet/tests/fixtures/playwright-compat"
fill_sections: [e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/tests/fixtures/playwright-compat

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
frontend_semantic:
  section_type: "tests"
  key: "jet/tests/fixtures/playwright-compat"
  source_group: "projects/jet/tests/fixtures/playwright-compat"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/tests/fixtures/playwright-compat/basic.spec.ts"
        language: "typescript"
        ownership_state: "codegen"
        generator_primitives: ["frontend_test", "td_section_tests", "test_case"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "typescript-jsx"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/tests/fixtures/playwright-compat"
          workspace_root: "projects/jet/tests/fixtures/playwright-compat"
        frontend_node:
          workspace_root: "projects/jet/tests/fixtures/playwright-compat"
          role: "test"
          section_type: "tests"
          artifact_kind: "test"
  frontend_ast:
    nodes:
      - path: "projects/jet/tests/fixtures/playwright-compat/basic.spec.ts"
        workspace_root: "projects/jet/tests/fixtures/playwright-compat"
        role: "test"
        artifact_kind: "test"
        section_type: "tests"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/tests/fixtures/playwright-compat/basic.spec.ts"
    action: modify
    section: e2e-test
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

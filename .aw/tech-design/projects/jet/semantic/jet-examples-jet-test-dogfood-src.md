---
id: semantic-jet-examples-jet-test-dogfood-src
summary: Semantic coverage for "projects/jet/examples/jet-test-dogfood/src"
fill_sections: [tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/examples/jet-test-dogfood/src

## Tests
<!-- type: tests lang: yaml -->

```yaml
frontend_semantic:
  section_type: "tests"
  key: "jet/examples/jet-test-dogfood/src"
  source_group: "projects/jet/examples/jet-test-dogfood/src"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/examples/jet-test-dogfood/src/unit.spec.ts"
        language: "typescript"
        ownership_state: "codegen"
        generator_primitives: ["frontend_test", "service_method", "td_section_tests"]
        symbols:
          - name: "add"
            kind: "function"
            public: true
          - name: "slugify"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "frontend"
          ecosystem: "typescript-jsx"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/examples/jet-test-dogfood/src"
          workspace_root: "projects/jet/examples/jet-test-dogfood/src"
        frontend_node:
          workspace_root: "projects/jet/examples/jet-test-dogfood/src"
          role: "test"
          section_type: "tests"
          artifact_kind: "test"
      - path: "projects/jet/examples/jet-test-dogfood/src/failure-fixture.spec.ts"
        language: "typescript"
        ownership_state: "codegen"
        generator_primitives: ["frontend_test", "td_section_tests"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "typescript-jsx"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/examples/jet-test-dogfood/src"
          workspace_root: "projects/jet/examples/jet-test-dogfood/src"
        frontend_node:
          workspace_root: "projects/jet/examples/jet-test-dogfood/src"
          role: "test"
          section_type: "tests"
          artifact_kind: "test"
      - path: "projects/jet/examples/jet-test-dogfood/src/frontend-integration.spec.ts"
        language: "typescript"
        ownership_state: "codegen"
        generator_primitives: ["frontend_test", "service_method", "td_section_tests", "ts_type_surface"]
        symbols:
          - name: "Todo"
            kind: "type"
            public: true
          - name: "State"
            kind: "type"
            public: true
          - name: "Action"
            kind: "type"
            public: true
          - name: "initial"
            kind: "function"
            public: true
          - name: "reduce"
            kind: "function"
            public: true
          - name: "selectOpenCount"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "frontend"
          ecosystem: "typescript-jsx"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/examples/jet-test-dogfood/src"
          workspace_root: "projects/jet/examples/jet-test-dogfood/src"
        frontend_node:
          workspace_root: "projects/jet/examples/jet-test-dogfood/src"
          role: "test"
          section_type: "tests"
          artifact_kind: "test"
  frontend_ast:
    nodes:
      - path: "projects/jet/examples/jet-test-dogfood/src/unit.spec.ts"
        workspace_root: "projects/jet/examples/jet-test-dogfood/src"
        role: "test"
        artifact_kind: "test"
        section_type: "tests"
      - path: "projects/jet/examples/jet-test-dogfood/src/failure-fixture.spec.ts"
        workspace_root: "projects/jet/examples/jet-test-dogfood/src"
        role: "test"
        artifact_kind: "test"
        section_type: "tests"
      - path: "projects/jet/examples/jet-test-dogfood/src/frontend-integration.spec.ts"
        workspace_root: "projects/jet/examples/jet-test-dogfood/src"
        role: "test"
        artifact_kind: "test"
        section_type: "tests"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/examples/jet-test-dogfood/src/unit.spec.ts"
    action: modify
    section: tests
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/examples/jet-test-dogfood/src/failure-fixture.spec.ts"
    action: modify
    section: tests
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/examples/jet-test-dogfood/src/frontend-integration.spec.ts"
    action: modify
    section: tests
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

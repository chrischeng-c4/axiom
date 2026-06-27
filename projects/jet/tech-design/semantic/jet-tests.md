---
id: semantic-jet-tests
summary: Semantic coverage for "projects/jet/tests"
capability_refs:
  - id: "rust-native-frontend-toolchain"
    role: primary
    claim: "production-replacement-readiness"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/jet/tests`."
fill_sections: [schema, unit-test, changes]
---

# Semantic TD: jet/tests

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/tests"
  source_group: "projects/jet/tests"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/tests/react_dom_oracle_conformance.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "react_dom_oracle_conformance"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/wasm_dom_parity_gate.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "wasm_dom_parity_gate"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/behavior_multi_fixture_dom_wasm_screenshot_pixel_parity.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "multi_fixture_dom_wasm_screenshot_pixel_parity"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/browser_cli_smoke.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "browser_cli_smoke"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/behavior_multi_fixture_dom_wasm_layout_parity.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "multi_fixture_dom_wasm_layout_parity"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/behavior_counter_dom_wasm_parity_after_click.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "counter_dom_wasm_parity_after_click"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/behavior_phase_3_build_gate.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "phase_3_build_gate"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/renderer_layout.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "renderer_layout"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/openapi_golden.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "codegen_openapi_golden"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/behavior_multi_fixture_dom_wasm_parity.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "multi_fixture_dom_wasm_parity"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/behavior_library_dom_wasm_parity.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "library_dom_wasm_parity"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/behavior_live_wasm_capture_after_click.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "live_wasm_capture_after_click"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/behavior_dom_renderer_controlled_textarea_parity.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "dom_renderer_controlled_textarea_parity"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/behavior_full_jet_health_after_unit_repair.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "full_jet_health_after_unit_repair"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/behavior_dom_renderer_controlled_input_parity.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "dom_renderer_controlled_input_parity"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/behavior_dom_renderer_controlled_input_regression.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "dom_renderer_controlled_input_regression"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/behavior_multi_fixture_dom_wasm_canvas_paint_parity.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "multi_fixture_dom_wasm_canvas_paint_parity"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
      - path: "projects/jet/tests/behavior_package_phase_gate.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "package_phase_gate"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/jet/tests"
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: unit-test
coverage_kind: semantic
strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
evidence:
  source_tests:
    - path: "projects/jet/tests/react_dom_oracle_conformance.rs"
    - path: "projects/jet/tests/wasm_dom_parity_gate.rs"
    - path: "projects/jet/tests/behavior_multi_fixture_dom_wasm_screenshot_pixel_parity.rs"
    - path: "projects/jet/tests/browser_cli_smoke.rs"
    - path: "projects/jet/tests/behavior_multi_fixture_dom_wasm_layout_parity.rs"
    - path: "projects/jet/tests/behavior_counter_dom_wasm_parity_after_click.rs"
    - path: "projects/jet/tests/behavior_phase_3_build_gate.rs"
    - path: "projects/jet/tests/renderer_layout.rs"
    - path: "projects/jet/tests/openapi_golden.rs"
    - path: "projects/jet/tests/behavior_multi_fixture_dom_wasm_parity.rs"
    - path: "projects/jet/tests/behavior_library_dom_wasm_parity.rs"
    - path: "projects/jet/tests/behavior_live_wasm_capture_after_click.rs"
    - path: "projects/jet/tests/behavior_dom_renderer_controlled_textarea_parity.rs"
    - path: "projects/jet/tests/behavior_full_jet_health_after_unit_repair.rs"
    - path: "projects/jet/tests/behavior_dom_renderer_controlled_input_parity.rs"
    - path: "projects/jet/tests/behavior_dom_renderer_controlled_input_regression.rs"
    - path: "projects/jet/tests/behavior_multi_fixture_dom_wasm_canvas_paint_parity.rs"
    - path: "projects/jet/tests/behavior_package_phase_gate.rs"
---
requirementDiagram

element UT_SOURCE_TESTS {
  type: "TestEvidence"
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/tests/react_dom_oracle_conformance.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-jet-tests-react-dom-oracle-conformance-rs>"
  - path: "projects/jet/tests/wasm_dom_parity_gate.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-jet-tests-wasm-dom-parity-gate-rs>"
  - path: "projects/jet/tests/behavior_multi_fixture_dom_wasm_screenshot_pixel_parity.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/browser_cli_smoke.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-jet-tests-browser-cli-smoke-rs>"
  - path: "projects/jet/tests/behavior_multi_fixture_dom_wasm_layout_parity.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/behavior_counter_dom_wasm_parity_after_click.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/behavior_phase_3_build_gate.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/renderer_layout.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-jet-tests-renderer-layout-rs>"
  - path: "projects/jet/tests/openapi_golden.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-jet-tests-openapi-golden-rs>"
  - path: "projects/jet/tests/behavior_multi_fixture_dom_wasm_parity.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/behavior_library_dom_wasm_parity.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/behavior_live_wasm_capture_after_click.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/behavior_dom_renderer_controlled_textarea_parity.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/behavior_full_jet_health_after_unit_repair.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/behavior_dom_renderer_controlled_input_parity.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/behavior_dom_renderer_controlled_input_regression.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/behavior_multi_fixture_dom_wasm_canvas_paint_parity.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/behavior_package_phase_gate.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/tests/renderer_layout.rs"
    action: verify
    section: unit-test
    description: |
      Preserve the observed Jet source-test evidence graph while semantic
      coverage is promoted toward deterministic generator primitives.
    impl_mode: hand-written
  - path: "projects/jet/tests/openapi_golden.rs"
    action: verify
    section: unit-test
    description: |
      Preserve the observed Jet codegen golden wrapper evidence graph while
      semantic coverage is promoted toward deterministic generator primitives.
    impl_mode: hand-written
```

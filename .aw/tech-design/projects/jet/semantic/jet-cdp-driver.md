---
id: semantic-jet-cdp-driver
summary: Semantic coverage for "projects/jet/src/cdp_driver"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/cdp_driver

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/cdp_driver"
  source_group: "projects/jet/src/cdp_driver"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/cdp_driver/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "page_binding"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/cdp_driver"
      - path: "projects/jet/src/cdp_driver/page_binding.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["enum_model", "service_method"]
        symbols:
          - name: "PageRequest"
            kind: "enum"
            public: true
          - name: "PageResponse"
            kind: "enum"
            public: true
          - name: "format_unknown_page_request_warn"
            kind: "function"
            public: true
          - name: "parse_page_request"
            kind: "function"
            public: true
          - name: "dispatch_page_request"
            kind: "function"
            public: true
          - name: "write_page_response"
            kind: "function"
            public: true
          - name: "do_click"
            kind: "function"
            public: false
          - name: "do_fill"
            kind: "function"
            public: false
          - name: "do_wait_load_state"
            kind: "function"
            public: false
          - name: "do_get_attribute"
            kind: "function"
            public: false
          - name: "do_set_viewport_size"
            kind: "function"
            public: false
          - name: "do_screenshot"
            kind: "function"
            public: false
          - name: "do_go_back"
            kind: "function"
            public: false
          - name: "do_go_forward"
            kind: "function"
            public: false
          - name: "format_history_field_warn"
            kind: "function"
            public: true
          - name: "navigate_history_delta"
            kind: "function"
            public: false
          - name: "do_reload"
            kind: "function"
            public: false
          - name: "do_keyboard_press"
            kind: "function"
            public: false
          - name: "do_keyboard_type"
            kind: "function"
            public: false
          - name: "do_mouse_event"
            kind: "function"
            public: false
          - name: "do_set_content"
            kind: "function"
            public: false
          - name: "do_bounding_box"
            kind: "function"
            public: false
          - name: "do_hover"
            kind: "function"
            public: false
          - name: "do_locator_press"
            kind: "function"
            public: false
          - name: "playwright_key_to_cdp"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3743_history_field_warn_tests"
            kind: "module"
            public: false
          - name: "gh3745_parse_page_request_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/cdp_driver"
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
tests:
  coverage_kind: semantic
  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
  evidence:
    source_tests: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/cdp_driver/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/cdp_driver/page_binding.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-cdp-driver.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```

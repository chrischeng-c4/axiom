---
id: semantic-jet-browser
summary: Semantic coverage for "projects/jet/src/browser"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/browser

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/browser"
  source_group: "projects/jet/src/browser"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/browser/cdp.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "CdpSession"
            kind: "struct"
            public: true
          - name: "CdpClient"
            kind: "struct"
            public: true
          - name: "CdpEvent"
            kind: "struct"
            public: true
          - name: "CdpRequest"
            kind: "struct"
            public: false
          - name: "CdpResponse"
            kind: "struct"
            public: false
          - name: "CdpError"
            kind: "struct"
            public: false
          - name: "OutgoingMessage"
            kind: "enum"
            public: false
          - name: "connect"
            kind: "function"
            public: true
          - name: "send"
            kind: "function"
            public: true
          - name: "create_target"
            kind: "function"
            public: true
          - name: "attach_to_target"
            kind: "function"
            public: true
          - name: "next_event"
            kind: "function"
            public: true
          - name: "root_session"
            kind: "function"
            public: true
          - name: "send"
            kind: "function"
            public: true
          - name: "child_session"
            kind: "function"
            public: true
          - name: "session_id"
            kind: "function"
            public: true
          - name: "try_parse_cdp_response"
            kind: "function"
            public: false
          - name: "format_cdp_send_closed_err"
            kind: "function"
            public: true
          - name: "format_cdp_response_dropped_err"
            kind: "function"
            public: true
          - name: "gh3333_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/browser"
      - path: "projects/jet/src/browser/page.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "Page"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "with_context"
            kind: "function"
            public: true
          - name: "context_id"
            kind: "function"
            public: true
          - name: "goto"
            kind: "function"
            public: true
          - name: "evaluate"
            kind: "function"
            public: true
          - name: "add_init_script"
            kind: "function"
            public: true
          - name: "query_selector"
            kind: "function"
            public: true
          - name: "screenshot"
            kind: "function"
            public: true
          - name: "title"
            kind: "function"
            public: true
          - name: "url"
            kind: "function"
            public: true
          - name: "wait_for_selector"
            kind: "function"
            public: true
          - name: "target_id"
            kind: "function"
            public: true
          - name: "session"
            kind: "function"
            public: true
          - name: "locator"
            kind: "function"
            public: true
          - name: "get_by_role"
            kind: "function"
            public: true
          - name: "get_by_text"
            kind: "function"
            public: true
          - name: "wait_for_load"
            kind: "function"
            public: false
          - name: "format_page_load_timeout_err"
            kind: "function"
            public: true
          - name: "ElementHandle"
            kind: "struct"
            public: true
          - name: "click"
            kind: "function"
            public: true
          - name: "type_text"
            kind: "function"
            public: true
          - name: "text_content"
            kind: "function"
            public: true
          - name: "resolve_object_id"
            kind: "function"
            public: false
          - name: "page_value_kind"
            kind: "function"
            public: false
          - name: "coerce_page_string_or_warn"
            kind: "function"
            public: true
          - name: "format_page_string_shape_warn"
            kind: "function"
            public: true
          - name: "gh3770_page_shape_warn_tests"
            kind: "module"
            public: false
          - name: "gh3556_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/browser"
      - path: "projects/jet/src/browser/locator.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method", "ts_type_surface"]
        symbols:
          - name: "LocatorError"
            kind: "enum"
            public: true
          - name: "from"
            kind: "function"
            public: false
          - name: "LocatorResult"
            kind: "type"
            public: false
          - name: "Actionability"
            kind: "enum"
            public: true
          - name: "LocatorOptions"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "SelectorExpr"
            kind: "enum"
            public: true
          - name: "parse"
            kind: "function"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "parse_role_with_name"
            kind: "function"
            public: false
          - name: "Index"
            kind: "enum"
            public: false
          - name: "Filter"
            kind: "enum"
            public: false
          - name: "Locator"
            kind: "struct"
            public: true
          - name: "new_root"
            kind: "function"
            public: true
          - name: "locator"
            kind: "function"
            public: true
          - name: "filter_has_text"
            kind: "function"
            public: true
          - name: "nth"
            kind: "function"
            public: true
          - name: "first"
            kind: "function"
            public: true
          - name: "last"
            kind: "function"
            public: true
          - name: "get_by_role"
            kind: "function"
            public: true
          - name: "get_by_text"
            kind: "function"
            public: true
          - name: "with_options"
            kind: "function"
            public: true
          - name: "describe"
            kind: "function"
            public: false
          - name: "count"
            kind: "function"
            public: true
          - name: "text_content"
            kind: "function"
            public: true
          - name: "inner_text"
            kind: "function"
            public: true
          - name: "get_attribute"
            kind: "function"
            public: true
          - name: "is_visible"
            kind: "function"
            public: true
          - name: "wait_for"
            kind: "function"
            public: true
          - name: "click"
            kind: "function"
            public: true
          - name: "fill"
            kind: "function"
            public: true
          - name: "check"
            kind: "function"
            public: true
          - name: "uncheck"
            kind: "function"
            public: true
          - name: "hover"
            kind: "function"
            public: true
          - name: "compile_collection_expr"
            kind: "function"
            public: false
          - name: "compile_count_js"
            kind: "function"
            public: false
          - name: "compile_single_expr"
            kind: "function"
            public: false
          - name: "compile_single_js"
            kind: "function"
            public: false
          - name: "wait_until"
            kind: "function"
            public: false
          - name: "probe_state"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/browser"
      - path: "projects/jet/src/browser/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "cdp"
            kind: "module"
            public: true
          - name: "context"
            kind: "module"
            public: true
          - name: "install"
            kind: "module"
            public: true
          - name: "launcher"
            kind: "module"
            public: true
          - name: "locator"
            kind: "module"
            public: true
          - name: "page"
            kind: "module"
            public: true
          - name: "Browser"
            kind: "struct"
            public: true
          - name: "launch"
            kind: "function"
            public: true
          - name: "connect"
            kind: "function"
            public: true
          - name: "new_page"
            kind: "function"
            public: true
          - name: "new_context"
            kind: "function"
            public: true
          - name: "default_context"
            kind: "function"
            public: true
          - name: "close"
            kind: "function"
            public: true
          - name: "ws_url"
            kind: "function"
            public: true
          - name: "create_browser_context"
            kind: "function"
            public: false
          - name: "try_close_browser_rpc"
            kind: "function"
            public: true
          - name: "try_kill_browser_process"
            kind: "function"
            public: true
          - name: "gh3488_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/browser"
      - path: "projects/jet/src/browser/install.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method"]
        symbols:
          - name: "DEFAULT_CHROMIUM_REVISION"
            kind: "constant"
            public: true
          - name: "resolve_platform"
            kind: "function"
            public: false
          - name: "resolve_native_platform"
            kind: "function"
            public: false
          - name: "install_chromium"
            kind: "function"
            public: true
          - name: "install_chromium_for"
            kind: "function"
            public: true
          - name: "install_chromium_inner"
            kind: "function"
            public: false
          - name: "download_to_tempfile"
            kind: "function"
            public: false
          - name: "extract_zip"
            kind: "function"
            public: false
          - name: "gh3479_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/browser"
      - path: "projects/jet/src/browser/launcher.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "LaunchOptions"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "BrowserLauncher"
            kind: "struct"
            public: true
          - name: "launch"
            kind: "function"
            public: true
          - name: "find_chrome"
            kind: "function"
            public: false
          - name: "find_chrome_in"
            kind: "function"
            public: true
          - name: "find_chrome_system"
            kind: "function"
            public: false
          - name: "find_free_port"
            kind: "function"
            public: false
          - name: "wait_for_ws_endpoint"
            kind: "function"
            public: false
          - name: "format_browser_ws_timeout_err"
            kind: "function"
            public: true
          - name: "format_chromium_entry_warn"
            kind: "function"
            public: true
          - name: "is_executable"
            kind: "function"
            public: false
          - name: "gh3328_tests"
            kind: "module"
            public: false
          - name: "gh3520_tests"
            kind: "module"
            public: false
          - name: "gh3727_browser_ws_timeout_err_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/browser"
      - path: "projects/jet/src/browser/context.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "BrowserContext"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "id"
            kind: "function"
            public: true
          - name: "is_default"
            kind: "function"
            public: true
          - name: "new_page"
            kind: "function"
            public: true
          - name: "pages"
            kind: "function"
            public: true
          - name: "cookies"
            kind: "function"
            public: true
          - name: "add_cookies"
            kind: "function"
            public: true
          - name: "clear_cookies"
            kind: "function"
            public: true
          - name: "storage_state"
            kind: "function"
            public: true
          - name: "set_storage_state"
            kind: "function"
            public: true
          - name: "close"
            kind: "function"
            public: true
          - name: "json_value_type_name"
            kind: "function"
            public: true
          - name: "format_storage_state_cookies_shape_warn"
            kind: "function"
            public: true
          - name: "format_cdp_get_cookies_shape_warn"
            kind: "function"
            public: true
          - name: "gh3739_set_storage_state_silent_no_op_tests"
            kind: "module"
            public: false
          - name: "gh3761_get_cookies_silent_shape_drop_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/browser"
```

## Tests
<!-- type: tests lang: yaml -->

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
  - path: "projects/jet/src/browser/cdp.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/browser/page.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/browser/locator.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/browser/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/browser/install.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/browser/launcher.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/browser/context.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-browser.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```

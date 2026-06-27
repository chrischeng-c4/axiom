---
id: semantic-jet-browser-cli
summary: Semantic coverage for "projects/jet/src/browser_cli"
capability_refs:
  - id: "rust-native-frontend-toolchain"
    role: primary
    claim: "production-replacement-readiness"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/jet/src/browser_cli`."
fill_sections: [schema, changes]
---

# Semantic TD: jet/browser_cli

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/browser_cli"
  source_group: "projects/jet/src/browser_cli"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/browser_cli/session.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "MODE_FOREGROUND"
            kind: "constant"
            public: true
          - name: "MODE_DETACHED"
            kind: "constant"
            public: true
          - name: "Session"
            kind: "struct"
            public: true
          - name: "default_mode"
            kind: "function"
            public: false
          - name: "is_detached"
            kind: "function"
            public: true
          - name: "session_path"
            kind: "function"
            public: true
          - name: "shutdown_request_path"
            kind: "function"
            public: true
          - name: "write"
            kind: "function"
            public: true
          - name: "request_shutdown"
            kind: "function"
            public: true
          - name: "shutdown_requested"
            kind: "function"
            public: true
          - name: "clear_shutdown_request"
            kind: "function"
            public: true
          - name: "read"
            kind: "function"
            public: true
          - name: "clear"
            kind: "function"
            public: true
          - name: "now_unix"
            kind: "function"
            public: true
          - name: "safe_session_now_unix"
            kind: "function"
            public: true
          - name: "format_safe_session_now_unix_warn"
            kind: "function"
            public: true
          - name: "read_live"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3677_safe_session_now_unix_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/browser_cli"
      - path: "projects/jet/src/browser_cli/interact.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["config_surface", "enum_model", "service_method"]
        symbols:
          - name: "BB_OBSERVE_INIT_JS"
            kind: "constant"
            public: true
          - name: "Target"
            kind: "enum"
            public: true
          - name: "parse_target"
            kind: "function"
            public: true
          - name: "is_ref_shaped"
            kind: "function"
            public: false
          - name: "json_str"
            kind: "function"
            public: false
          - name: "element_expr"
            kind: "function"
            public: false
          - name: "action_js"
            kind: "function"
            public: false
          - name: "SNAPSHOT_JS"
            kind: "constant"
            public: false
          - name: "snapshot"
            kind: "function"
            public: true
          - name: "click"
            kind: "function"
            public: true
          - name: "fill"
            kind: "function"
            public: true
          - name: "type_text"
            kind: "function"
            public: true
          - name: "hover"
            kind: "function"
            public: true
          - name: "select"
            kind: "function"
            public: true
          - name: "set_checked"
            kind: "function"
            public: true
          - name: "arm_observe_hooks"
            kind: "function"
            public: false
          - name: "goto"
            kind: "function"
            public: true
          - name: "history_step"
            kind: "function"
            public: true
          - name: "reload"
            kind: "function"
            public: true
          - name: "resize"
            kind: "function"
            public: true
          - name: "wait_for_ready"
            kind: "function"
            public: false
          - name: "wait"
            kind: "function"
            public: true
          - name: "read_observe_buffer"
            kind: "function"
            public: false
          - name: "HOOKS_HEALED_NOTE"
            kind: "constant"
            public: false
          - name: "observe_result"
            kind: "function"
            public: false
          - name: "console"
            kind: "function"
            public: true
          - name: "requests"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/browser_cli"
      - path: "projects/jet/src/browser_cli/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "interact"
            kind: "module"
            public: true
          - name: "mcp"
            kind: "module"
            public: true
          - name: "pretty"
            kind: "module"
            public: true
          - name: "session"
            kind: "module"
            public: true
          - name: "safe_chrome_path_override"
            kind: "function"
            public: true
          - name: "format_safe_chrome_path_warn"
            kind: "function"
            public: true
          - name: "attach"
            kind: "function"
            public: true
          - name: "assert_debug_bridge"
            kind: "function"
            public: false
          - name: "expr"
            kind: "function"
            public: false
          - name: "prepare_session"
            kind: "function"
            public: true
          - name: "prepare_session_with_init_scripts"
            kind: "function"
            public: true
          - name: "prepare_session_with_mode"
            kind: "function"
            public: false
          - name: "launch_detached"
            kind: "function"
            public: true
          - name: "launch_foreground"
            kind: "function"
            public: true
          - name: "wait_for_shutdown_request"
            kind: "function"
            public: false
          - name: "shutdown"
            kind: "function"
            public: true
          - name: "close_remote_browser"
            kind: "function"
            public: false
          - name: "format_browser_cli_ctrl_c_warn"
            kind: "function"
            public: true
          - name: "tree"
            kind: "function"
            public: true
          - name: "hooks"
            kind: "function"
            public: true
          - name: "pick"
            kind: "function"
            public: true
          - name: "highlight"
            kind: "function"
            public: true
          - name: "frame"
            kind: "function"
            public: true
          - name: "perf"
            kind: "function"
            public: true
          - name: "dispatch_mouse_event"
            kind: "function"
            public: false
          - name: "mouse"
            kind: "function"
            public: true
          - name: "wheel"
            kind: "function"
            public: true
          - name: "drag"
            kind: "function"
            public: true
          - name: "key_code_for"
            kind: "function"
            public: false
          - name: "windows_virtual_key_code_for"
            kind: "function"
            public: false
          - name: "dispatch_key_event"
            kind: "function"
            public: false
          - name: "key"
            kind: "function"
            public: true
          - name: "observation_bundle"
            kind: "function"
            public: true
          - name: "read_target_manifest_bundle"
            kind: "function"
            public: false
          - name: "screenshot_visual_probe_from_png"
            kind: "function"
            public: false
          - name: "pixel_differs_from_background"
            kind: "function"
            public: false
          - name: "screenshot_visual_probe_tests"
            kind: "module"
            public: false
          - name: "dom_observation_bundle_from_page"
            kind: "function"
            public: true
          - name: "dom_observation_bundle"
            kind: "function"
            public: true
          - name: "dom_tree_expr"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/browser_cli"
      - path: "projects/jet/src/browser_cli/pretty.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "element_tree"
            kind: "function"
            public: true
          - name: "render_element"
            kind: "function"
            public: false
          - name: "layout_tree"
            kind: "function"
            public: true
          - name: "fiber_tree"
            kind: "function"
            public: true
          - name: "hook_values"
            kind: "function"
            public: true
          - name: "paint_ops"
            kind: "function"
            public: true
          - name: "render_rect"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/browser_cli"
      - path: "projects/jet/src/browser_cli/mcp.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["config_surface", "service_method"]
        symbols:
          - name: "PROTOCOL_VERSION"
            kind: "constant"
            public: false
          - name: "SERVER_NAME"
            kind: "constant"
            public: false
          - name: "serve"
            kind: "function"
            public: true
          - name: "write_frame"
            kind: "function"
            public: false
          - name: "handle_message"
            kind: "function"
            public: false
          - name: "initialize_result"
            kind: "function"
            public: false
          - name: "tool_definitions"
            kind: "function"
            public: false
          - name: "arg_target"
            kind: "function"
            public: false
          - name: "text_content"
            kind: "function"
            public: false
          - name: "arg_f64"
            kind: "function"
            public: false
          - name: "call_tool"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/browser_cli"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/browser_cli/session.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/browser_cli/interact.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:jet-bb-semantic-surface>"
  - path: "projects/jet/src/browser_cli/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/browser_cli/pretty.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/browser_cli/mcp.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:jet-bb-mcp-server>"
```

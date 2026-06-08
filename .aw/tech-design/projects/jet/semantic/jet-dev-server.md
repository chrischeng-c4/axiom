---
id: semantic-jet-dev-server
summary: Semantic coverage for "projects/jet/src/dev_server"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/dev_server

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/dev_server"
  source_group: "projects/jet/src/dev_server"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/dev_server/polyfills_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "t34_detect_builtin_import_via_require"
            kind: "function"
            public: false
          - name: "t35_detect_node_prefixed_builtin"
            kind: "function"
            public: false
          - name: "t36_crypto_polyfill_exports_web_crypto"
            kind: "function"
            public: false
          - name: "t37_buffer_polyfill_uint8array"
            kind: "function"
            public: false
          - name: "t38_process_polyfill_node_env"
            kind: "function"
            public: false
          - name: "t39_path_polyfill_posix_functions"
            kind: "function"
            public: false
          - name: "t40_events_polyfill_eventemitter"
            kind: "function"
            public: false
          - name: "t41_url_polyfill_browser_native"
            kind: "function"
            public: false
          - name: "t42_stub_builtin_exports_empty_object"
            kind: "function"
            public: false
          - name: "t43_stub_builtin_emits_warning"
            kind: "function"
            public: false
          - name: "t44_unused_builtin_not_detected"
            kind: "function"
            public: false
          - name: "test_has_polyfill_true_for_known"
            kind: "function"
            public: false
          - name: "test_has_polyfill_false_for_stubs"
            kind: "function"
            public: false
          - name: "test_find_require_imports"
            kind: "function"
            public: false
          - name: "test_find_from_imports"
            kind: "function"
            public: false
          - name: "test_detect_mixed_import_styles"
            kind: "function"
            public: false
          - name: "test_generate_polyfill_unknown"
            kind: "function"
            public: false
          - name: "test_stream_polyfill_ensures_events"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/src/dev_server"
      - path: "projects/jet/src/dev_server/module_graph.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "ModuleGraphNode"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: false
          - name: "HmrBoundaryResult"
            kind: "enum"
            public: true
          - name: "ModuleGraph"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "add_module"
            kind: "function"
            public: true
          - name: "update_module"
            kind: "function"
            public: true
          - name: "set_self_accepting"
            kind: "function"
            public: true
          - name: "set_accepted_deps"
            kind: "function"
            public: true
          - name: "set_has_react_refresh"
            kind: "function"
            public: true
          - name: "set_timestamp"
            kind: "function"
            public: true
          - name: "remove_module"
            kind: "function"
            public: true
          - name: "find_hmr_boundary"
            kind: "function"
            public: true
          - name: "dependents_of"
            kind: "function"
            public: true
          - name: "get"
            kind: "function"
            public: true
          - name: "urls"
            kind: "function"
            public: true
          - name: "default"
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
          domain: "projects/jet/src/dev_server"
      - path: "projects/jet/src/dev_server/proxy.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "HOP_BY_HOP"
            kind: "constant"
            public: false
          - name: "is_hop_by_hop"
            kind: "function"
            public: false
          - name: "ProxyHandler"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "is_empty"
            kind: "function"
            public: true
          - name: "match_target"
            kind: "function"
            public: true
          - name: "forward_http"
            kind: "function"
            public: true
          - name: "forward_websocket"
            kind: "function"
            public: true
          - name: "format_proxy_body_build_warn"
            kind: "function"
            public: true
          - name: "convert_response"
            kind: "function"
            public: false
          - name: "segment_prefix_match"
            kind: "function"
            public: false
          - name: "http_to_ws_url"
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
          domain: "projects/jet/src/dev_server"
      - path: "projects/jet/src/dev_server/importmap.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "build_importmap"
            kind: "function"
            public: true
          - name: "build_importmap_full"
            kind: "function"
            public: true
          - name: "inject_importmap_html"
            kind: "function"
            public: true
          - name: "mui_emotion_patches"
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
          domain: "projects/jet/src/dev_server"
      - path: "projects/jet/src/dev_server/polyfills.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method"]
        symbols:
          - name: "POLYFILL_BUILTINS"
            kind: "constant"
            public: false
          - name: "STUB_BUILTINS"
            kind: "constant"
            public: false
          - name: "all_builtins"
            kind: "function"
            public: false
          - name: "detect_builtin_imports"
            kind: "function"
            public: true
          - name: "find_require_imports"
            kind: "function"
            public: false
          - name: "find_from_imports"
            kind: "function"
            public: false
          - name: "generate_polyfill"
            kind: "function"
            public: true
          - name: "generate_stub"
            kind: "function"
            public: true
          - name: "has_polyfill"
            kind: "function"
            public: true
          - name: "write_polyfills"
            kind: "function"
            public: true
          - name: "generate_crypto_polyfill"
            kind: "function"
            public: false
          - name: "generate_url_polyfill"
            kind: "function"
            public: false
          - name: "generate_buffer_polyfill"
            kind: "function"
            public: false
          - name: "generate_path_polyfill"
            kind: "function"
            public: false
          - name: "generate_events_polyfill"
            kind: "function"
            public: false
          - name: "generate_util_polyfill"
            kind: "function"
            public: false
          - name: "generate_querystring_polyfill"
            kind: "function"
            public: false
          - name: "generate_process_polyfill"
            kind: "function"
            public: false
          - name: "generate_stream_polyfill"
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
          domain: "projects/jet/src/dev_server"
      - path: "projects/jet/src/dev_server/hmr.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "ClientMessage"
            kind: "enum"
            public: true
          - name: "ConsoleLevel"
            kind: "enum"
            public: true
          - name: "HmrMessage"
            kind: "enum"
            public: true
          - name: "HmrUpdateResult"
            kind: "enum"
            public: true
          - name: "determine"
            kind: "function"
            public: true
          - name: "HmrManager"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "broadcast"
            kind: "function"
            public: true
          - name: "subscribe"
            kind: "function"
            public: true
          - name: "subscriber_count"
            kind: "function"
            public: true
          - name: "default"
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
          domain: "projects/jet/src/dev_server"
      - path: "projects/jet/src/dev_server/incremental_rebuilder.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "RebuildOutcome"
            kind: "struct"
            public: true
          - name: "IncrementalRebuilder"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "rebuild"
            kind: "function"
            public: true
          - name: "metrics_snapshot"
            kind: "function"
            public: true
          - name: "is_supported"
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
          domain: "projects/jet/src/dev_server"
      - path: "projects/jet/src/dev_server/prebundle_tests.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "t02_esm_package_skipped_module_field"
            kind: "function"
            public: false
          - name: "t03_esm_package_skipped_exports_import"
            kind: "function"
            public: false
          - name: "test_cjs_package_detected"
            kind: "function"
            public: false
          - name: "test_type_module_detected_as_esm"
            kind: "function"
            public: false
          - name: "t05_scoped_package_filename"
            kind: "function"
            public: false
          - name: "test_dep_filename_regular"
            kind: "function"
            public: false
          - name: "t04_subpath_export_filename"
            kind: "function"
            public: false
          - name: "t06_prebundle_cache_hit"
            kind: "function"
            public: false
          - name: "t07_cache_invalidation_package_json"
            kind: "function"
            public: false
          - name: "t08_cache_invalidation_lockfile"
            kind: "function"
            public: false
          - name: "test_no_cache_marker"
            kind: "function"
            public: false
          - name: "test_stale_cache_marker_version_invalid"
            kind: "function"
            public: false
          - name: "test_resolve_exports_prefers_browser_module_nested_default"
            kind: "function"
            public: false
          - name: "test_resolve_exports_import_object_default"
            kind: "function"
            public: false
          - name: "t12_virtual_esm_entry"
            kind: "function"
            public: false
          - name: "test_virtual_esm_entry_scoped"
            kind: "function"
            public: false
          - name: "t13_process_env_node_env_resolved"
            kind: "function"
            public: false
          - name: "t16_exports_map_condition_resolution"
            kind: "function"
            public: false
          - name: "test_exports_require_fallback"
            kind: "function"
            public: false
          - name: "test_exports_string"
            kind: "function"
            public: false
          - name: "test_has_import_condition"
            kind: "function"
            public: false
          - name: "test_no_import_condition"
            kind: "function"
            public: false
          - name: "test_esm_passthrough"
            kind: "function"
            public: false
          - name: "t01_cjs_produces_valid_esm"
            kind: "function"
            public: false
          - name: "t14_circular_require_detected"
            kind: "function"
            public: false
          - name: "test_no_circular_when_linear"
            kind: "function"
            public: false
          - name: "t15_transitive_dep_discovered"
            kind: "function"
            public: false
          - name: "test_esm_dependency_transitive_cjs_root_prebundled"
            kind: "function"
            public: false
          - name: "cjs_nested_dependencies_resolve_from_parent_package"
            kind: "function"
            public: false
          - name: "mui_emotion_resolver_version_invalidates_cache"
            kind: "function"
            public: false
          - name: "manifest_mtime_within_marker_invalidates_when_mtime_unreadable"
            kind: "function"
            public: false
          - name: "cache_invalidated_when_pkg_json_newer_than_marker"
            kind: "function"
            public: false
          - name: "mui_emotion_resolver_version_matches_keeps_cache"
            kind: "function"
            public: false
          - name: "esm_detection_ignores_import_text_inside_cjs_strings"
            kind: "function"
            public: false
          - name: "esm_detection_accepts_static_import_export_syntax"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/src/dev_server"
      - path: "projects/jet/src/dev_server/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "hmr"
            kind: "module"
            public: true
          - name: "hmr_client"
            kind: "module"
            public: true
          - name: "importmap"
            kind: "module"
            public: true
          - name: "incremental_rebuilder"
            kind: "module"
            public: true
          - name: "module_graph"
            kind: "module"
            public: true
          - name: "polyfills"
            kind: "module"
            public: true
          - name: "prebundle"
            kind: "module"
            public: true
          - name: "proxy"
            kind: "module"
            public: true
          - name: "react_refresh"
            kind: "module"
            public: true
          - name: "source_analysis"
            kind: "module"
            public: true
          - name: "watcher"
            kind: "module"
            public: true
          - name: "DEV_SERVER_SERVE_FILE_NO_EXTENSION_FALLBACK"
            kind: "constant"
            public: true
          - name: "DEV_SERVER_HMR_REBUILD_NO_EXTENSION_FALLBACK"
            kind: "constant"
            public: true
          - name: "format_dev_server_hmr_rebuild_no_extension_warn"
            kind: "function"
            public: true
          - name: "format_dev_server_hmr_rebuild_non_utf8_extension_warn"
            kind: "function"
            public: true
          - name: "coerce_dev_server_hmr_rebuild_extension_or_warn"
            kind: "function"
            public: true
          - name: "format_dev_server_serve_file_no_extension_warn"
            kind: "function"
            public: true
          - name: "format_dev_server_serve_file_non_utf8_extension_warn"
            kind: "function"
            public: true
          - name: "coerce_dev_server_serve_file_extension_or_warn"
            kind: "function"
            public: true
          - name: "DevServer"
            kind: "struct"
            public: true
          - name: "ServerConfig"
            kind: "struct"
            public: true
          - name: "ServerState"
            kind: "struct"
            public: false
          - name: "new"
            kind: "function"
            public: true
          - name: "register_css_entry"
            kind: "function"
            public: true
          - name: "start"
            kind: "function"
            public: true
          - name: "create_router"
            kind: "function"
            public: false
          - name: "start_file_watcher"
            kind: "function"
            public: false
          - name: "root_dispatch_handler"
            kind: "function"
            public: false
          - name: "path_dispatch_handler"
            kind: "function"
            public: false
          - name: "dispatch_request"
            kind: "function"
            public: false
          - name: "hmr_websocket_handler"
            kind: "function"
            public: false
          - name: "hmr_websocket"
            kind: "function"
            public: false
          - name: "serve_bundle"
            kind: "function"
            public: false
          - name: "serve_index_html"
            kind: "function"
            public: false
          - name: "load_index_html_or_default"
            kind: "function"
            public: false
          - name: "default_index_html"
            kind: "function"
            public: false
          - name: "has_parent_dir_component"
            kind: "function"
            public: false
          - name: "serve_static_file"
            kind: "function"
            public: false
          - name: "serve_root_file"
            kind: "function"
            public: false
          - name: "pre_bundle_cjs_deps"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/dev_server"
      - path: "projects/jet/src/dev_server/hmr_client.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "generate_hmr_runtime"
            kind: "function"
            public: true
          - name: "generate_hot_preamble"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/dev_server"
      - path: "projects/jet/src/dev_server/watcher.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "DEBOUNCE_MS"
            kind: "constant"
            public: false
          - name: "FileWatcher"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "subscribe"
            kind: "function"
            public: true
          - name: "format_watch_error"
            kind: "function"
            public: false
          - name: "should_ignore"
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
          domain: "projects/jet/src/dev_server"
      - path: "projects/jet/src/dev_server/prebundle.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "CACHE_MARKER_VERSION"
            kind: "constant"
            public: true
          - name: "PreBundler"
            kind: "struct"
            public: true
          - name: "PreBundleResult"
            kind: "struct"
            public: true
          - name: "read_cached_importmap_or_warn"
            kind: "function"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "prebundle_deps"
            kind: "function"
            public: true
          - name: "merge_workspace_transitive_deps"
            kind: "function"
            public: true
          - name: "collect_dependencies"
            kind: "function"
            public: false
          - name: "is_cjs_package"
            kind: "function"
            public: true
          - name: "bundle_cjs_dep"
            kind: "function"
            public: false
          - name: "bundle_nested_cjs_deps"
            kind: "function"
            public: false
          - name: "bundle_cjs_dep_without_nested"
            kind: "function"
            public: false
          - name: "discover_subpath_exports"
            kind: "function"
            public: false
          - name: "collect_source_subpath_imports"
            kind: "function"
            public: false
          - name: "add_known_patch_subpath_imports"
            kind: "function"
            public: false
          - name: "prebundle_known_cjs_patch_roots"
            kind: "function"
            public: false
          - name: "collect_source_subpath_imports_from_dir"
            kind: "function"
            public: false
          - name: "resolve_subpath_export"
            kind: "function"
            public: true
          - name: "resolve_package_main"
            kind: "function"
            public: false
          - name: "check_cache_valid"
            kind: "function"
            public: true
          - name: "scan_esm_deps"
            kind: "function"
            public: false
          - name: "symlink_into_node_modules"
            kind: "function"
            public: false
          - name: "resolve_package_dir"
            kind: "function"
            public: false
          - name: "write_cache_marker"
            kind: "function"
            public: false
          - name: "discover_pnpm_deps"
            kind: "function"
            public: false
          - name: "split_bare_subpath_import"
            kind: "function"
            public: false
          - name: "is_workspace_symlink"
            kind: "function"
            public: false
          - name: "is_local_source_package"
            kind: "function"
            public: false
          - name: "create_virtual_entry"
            kind: "function"
            public: true
          - name: "dep_filename"
            kind: "function"
            public: false
          - name: "has_import_condition"
            kind: "function"
            public: false
          - name: "resolve_exports_entry"
            kind: "function"
            public: true
          - name: "resolve_exports_condition"
            kind: "function"
            public: true
          - name: "detect_circular_deps"
            kind: "function"
            public: true
          - name: "inline_requires"
            kind: "function"
            public: false
          - name: "inline_requires_depth"
            kind: "function"
            public: false
          - name: "convert_cjs_to_esm"
            kind: "function"
            public: false
          - name: "collect_external_requires"
            kind: "function"
            public: false
          - name: "resolve_nested_package_dir"
            kind: "function"
            public: false
          - name: "strip_js_comments_for_require_scan"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/dev_server"
      - path: "projects/jet/src/dev_server/source_analysis.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "extract_imports_from_source"
            kind: "function"
            public: true
          - name: "trim_buf_to_tail_chars"
            kind: "function"
            public: false
          - name: "extract_import_from_statement"
            kind: "function"
            public: false
          - name: "extract_string_literal"
            kind: "function"
            public: true
          - name: "source_has_react_components"
            kind: "function"
            public: true
          - name: "build_error_frame"
            kind: "function"
            public: true
          - name: "detect_hmr_accept_calls"
            kind: "function"
            public: true
          - name: "extract_error_location"
            kind: "function"
            public: true
          - name: "parse_leading_number"
            kind: "function"
            public: false
          - name: "file_path_to_url"
            kind: "function"
            public: true
          - name: "format_file_path_to_url_warn"
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
          domain: "projects/jet/src/dev_server"
      - path: "projects/jet/src/dev_server/react_refresh.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "react_refresh_runtime_source"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/dev_server"
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
  - path: "projects/jet/src/dev_server/polyfills_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/dev_server/module_graph.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/dev_server/proxy.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/dev_server/importmap.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/dev_server/polyfills.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/dev_server/hmr.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/dev_server/incremental_rebuilder.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/dev_server/prebundle_tests.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/dev_server/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/dev_server/hmr_client.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/dev_server/watcher.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/dev_server/prebundle.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/dev_server/source_analysis.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/dev_server/react_refresh.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-dev-server.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```

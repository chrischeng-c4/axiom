---
id: semantic-jet-stories
summary: Semantic coverage for "projects/jet/src/stories"
capability_refs:
  - id: "rust-native-frontend-toolchain"
    role: primary
    claim: "production-replacement-readiness"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/jet/src/stories`."
fill_sections: [schema, changes]
---

# Semantic TD: jet/stories

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/stories"
  source_group: "projects/jet/src/stories"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/stories/prop_extractor.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "PropDef"
            kind: "struct"
            public: true
          - name: "extract_props"
            kind: "function"
            public: true
          - name: "extract_props_at"
            kind: "function"
            public: true
          - name: "parse_tsx"
            kind: "function"
            public: false
          - name: "PropsType"
            kind: "enum"
            public: false
          - name: "find_component_props_type"
            kind: "function"
            public: false
          - name: "props_type_from_declarators"
            kind: "function"
            public: false
          - name: "fc_type_argument"
            kind: "function"
            public: false
          - name: "props_type_from_params"
            kind: "function"
            public: false
          - name: "props_type_from_type_node"
            kind: "function"
            public: false
          - name: "generic_props_type"
            kind: "function"
            public: false
          - name: "Resolver"
            kind: "struct"
            public: false
          - name: "new"
            kind: "function"
            public: false
          - name: "resolve_named"
            kind: "function"
            public: false
          - name: "find_local_decl"
            kind: "function"
            public: false
          - name: "props_from_decl"
            kind: "function"
            public: false
          - name: "props_from_type_alias"
            kind: "function"
            public: false
          - name: "props_from_intersection"
            kind: "function"
            public: false
          - name: "resolve_imported"
            kind: "function"
            public: false
          - name: "find_type_decl"
            kind: "function"
            public: false
          - name: "type_alias_rhs"
            kind: "function"
            public: false
          - name: "interface_extends_names"
            kind: "function"
            public: false
          - name: "type_param_substitution"
            kind: "function"
            public: false
          - name: "merge_props"
            kind: "function"
            public: false
          - name: "resolve_relative_type_file"
            kind: "function"
            public: false
          - name: "import_specifier_for"
            kind: "function"
            public: false
          - name: "import_binds_type"
            kind: "function"
            public: false
          - name: "read_object_type_members"
            kind: "function"
            public: false
          - name: "read_object_type_members_subst"
            kind: "function"
            public: false
          - name: "apply_type_param_subst"
            kind: "function"
            public: false
          - name: "member_is_optional"
            kind: "function"
            public: false
          - name: "named_children"
            kind: "function"
            public: false
          - name: "first_child_of_kind"
            kind: "function"
            public: false
          - name: "identifier_name"
            kind: "function"
            public: false
          - name: "node_text"
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
          domain: "projects/jet/src/stories"
      - path: "projects/jet/src/stories/csf.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "CsfValue"
            kind: "enum"
            public: true
          - name: "CsfMeta"
            kind: "struct"
            public: true
          - name: "CsfStory"
            kind: "struct"
            public: true
          - name: "CsfReExport"
            kind: "struct"
            public: true
          - name: "ParsedStoryFile"
            kind: "struct"
            public: true
          - name: "parse_csf"
            kind: "function"
            public: true
          - name: "parse_meta_object"
            kind: "function"
            public: false
          - name: "parse_story_object"
            kind: "function"
            public: false
          - name: "SpreadScope"
            kind: "struct"
            public: false
          - name: "StoryMutation"
            kind: "struct"
            public: false
          - name: "collect_story_mutations"
            kind: "function"
            public: false
          - name: "object_member_nodes"
            kind: "function"
            public: false
          - name: "resolve_object_args"
            kind: "function"
            public: false
          - name: "resolve_args"
            kind: "function"
            public: false
          - name: "resolve_args_guarded"
            kind: "function"
            public: false
          - name: "spread_base_members"
            kind: "function"
            public: false
          - name: "pair_kv"
            kind: "function"
            public: false
          - name: "is_bind_call"
            kind: "function"
            public: false
          - name: "unwrap_to_call"
            kind: "function"
            public: false
          - name: "re_export_source"
            kind: "function"
            public: false
          - name: "collect_re_exports"
            kind: "function"
            public: false
          - name: "declarators"
            kind: "function"
            public: false
          - name: "declarator_value"
            kind: "function"
            public: false
          - name: "unwrap_to_object"
            kind: "function"
            public: false
          - name: "is_default_export"
            kind: "function"
            public: false
          - name: "default_export_object"
            kind: "function"
            public: false
          - name: "object_pairs"
            kind: "function"
            public: false
          - name: "pair_value"
            kind: "function"
            public: false
          - name: "value_of"
            kind: "function"
            public: false
          - name: "named_children"
            kind: "function"
            public: false
          - name: "first_child_of_kind"
            kind: "function"
            public: false
          - name: "node_text"
            kind: "function"
            public: false
          - name: "strip_quotes"
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
          domain: "projects/jet/src/stories"
      - path: "projects/jet/src/stories/build.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "BuildStaticResult"
            kind: "struct"
            public: true
          - name: "build_stories_static"
            kind: "function"
            public: true
          - name: "manager_relative_html"
            kind: "function"
            public: true
          - name: "preview_relative_html"
            kind: "function"
            public: true
          - name: "preview_module_import_url"
            kind: "function"
            public: false
          - name: "EmitItem"
            kind: "enum"
            public: false
          - name: "emitted_path"
            kind: "function"
            public: false
          - name: "emit_module_graph"
            kind: "function"
            public: false
          - name: "EmittedItem"
            kind: "struct"
            public: false
          - name: "emit_item"
            kind: "function"
            public: false
          - name: "rewrite_imports"
            kind: "function"
            public: false
          - name: "resolve_relative_file"
            kind: "function"
            public: false
          - name: "lexically_normalize"
            kind: "function"
            public: false
          - name: "path_has_node_modules"
            kind: "function"
            public: false
          - name: "relative_emitted_specifier"
            kind: "function"
            public: false
          - name: "transform_source"
            kind: "function"
            public: false
          - name: "resolve_url_to_file"
            kind: "function"
            public: false
          - name: "file_to_root_url"
            kind: "function"
            public: false
          - name: "story_module_root_url"
            kind: "function"
            public: false
          - name: "to_js_path"
            kind: "function"
            public: false
          - name: "write_emitted"
            kind: "function"
            public: false
          - name: "clean_out_dir"
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
          domain: "projects/jet/src/stories"
      - path: "projects/jet/src/stories/hmr.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "STORIES_HMR_ROUTE"
            kind: "constant"
            public: true
          - name: "UpdateKind"
            kind: "enum"
            public: true
          - name: "is_patch"
            kind: "function"
            public: true
          - name: "StoriesHmrMessage"
            kind: "enum"
            public: true
          - name: "to_json"
            kind: "function"
            public: true
          - name: "StoriesHmrManager"
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
          - name: "affected_modules"
            kind: "function"
            public: true
          - name: "classify_update"
            kind: "function"
            public: true
          - name: "is_react_component_module"
            kind: "function"
            public: false
          - name: "message_for_change"
            kind: "function"
            public: true
          - name: "module_url_for"
            kind: "function"
            public: true
          - name: "is_watchable_module"
            kind: "function"
            public: false
          - name: "now_ms"
            kind: "function"
            public: false
          - name: "spawn_watcher"
            kind: "function"
            public: true
          - name: "register_served_module"
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
          domain: "projects/jet/src/stories"
      - path: "projects/jet/src/stories/server.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "MANAGER_PREFIX"
            kind: "constant"
            public: true
          - name: "DEP_PREFIX"
            kind: "constant"
            public: true
          - name: "REACT_REFRESH_ROUTE"
            kind: "constant"
            public: true
          - name: "WorkbenchState"
            kind: "struct"
            public: false
          - name: "start_stories_workbench"
            kind: "function"
            public: true
          - name: "build_router"
            kind: "function"
            public: true
          - name: "build_router_with"
            kind: "function"
            public: false
          - name: "manager_handler"
            kind: "function"
            public: false
          - name: "controls_for_story"
            kind: "function"
            public: false
          - name: "read_component_source"
            kind: "function"
            public: false
          - name: "component_import_specifier"
            kind: "function"
            public: false
          - name: "import_binds"
            kind: "function"
            public: false
          - name: "resolve_module_file"
            kind: "function"
            public: false
          - name: "strip_quotes"
            kind: "function"
            public: false
          - name: "preview_handler"
            kind: "function"
            public: false
          - name: "react_refresh_handler"
            kind: "function"
            public: false
          - name: "stories_hmr_handler"
            kind: "function"
            public: false
          - name: "stories_hmr_socket"
            kind: "function"
            public: false
          - name: "module_handler"
            kind: "function"
            public: false
          - name: "register_module_imports"
            kind: "function"
            public: false
          - name: "resolve_relative_import"
            kind: "function"
            public: false
          - name: "serve_module"
            kind: "function"
            public: false
          - name: "dep_handler"
            kind: "function"
            public: false
          - name: "transform_to_js"
            kind: "function"
            public: false
          - name: "rewrite_bare_imports_to_dep_routes"
            kind: "function"
            public: false
          - name: "module_url_for"
            kind: "function"
            public: false
          - name: "html_response"
            kind: "function"
            public: false
          - name: "js_response"
            kind: "function"
            public: false
          - name: "story_module_url"
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
          domain: "projects/jet/src/stories"
      - path: "projects/jet/src/stories/deps.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["service_method"]
        symbols:
          - name: "resolve_bare_specifier"
            kind: "function"
            public: true
          - name: "dep_key"
            kind: "function"
            public: true
          - name: "path_has_node_modules"
            kind: "function"
            public: false
          - name: "extract_all_import_specifiers"
            kind: "function"
            public: true
          - name: "specifier_from_statement"
            kind: "function"
            public: false
          - name: "extract_first_string_literal"
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
          domain: "projects/jet/src/stories"
      - path: "projects/jet/src/stories/controls.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "ControlKind"
            kind: "enum"
            public: true
          - name: "Control"
            kind: "struct"
            public: true
          - name: "infer_control"
            kind: "function"
            public: true
          - name: "string_literal_union"
            kind: "function"
            public: false
          - name: "strip_string_literal"
            kind: "function"
            public: false
          - name: "resolve_controls"
            kind: "function"
            public: true
          - name: "arg_type_is_disabled"
            kind: "function"
            public: false
          - name: "control_kind_from_arg_type"
            kind: "function"
            public: false
          - name: "control_from_type_name"
            kind: "function"
            public: false
          - name: "csf_string_list"
            kind: "function"
            public: false
          - name: "parse_array_literal"
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
          domain: "projects/jet/src/stories"
      - path: "projects/jet/src/stories/manager.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["config_surface", "enum_model", "service_method"]
        symbols:
          - name: "PREVIEW_PREFIX"
            kind: "constant"
            public: true
          - name: "UrlMode"
            kind: "enum"
            public: true
          - name: "preview_url"
            kind: "function"
            public: false
          - name: "empty_preview_url"
            kind: "function"
            public: false
          - name: "render_manager_html"
            kind: "function"
            public: true
          - name: "render_manager_html_with_mode"
            kind: "function"
            public: true
          - name: "render_controls_panel"
            kind: "function"
            public: false
          - name: "render_control_widget"
            kind: "function"
            public: false
          - name: "current_value_string"
            kind: "function"
            public: false
          - name: "controls_to_args_json"
            kind: "function"
            public: false
          - name: "render_sidebar"
            kind: "function"
            public: false
          - name: "render_diagnostics"
            kind: "function"
            public: false
          - name: "story_display_title"
            kind: "function"
            public: false
          - name: "render_preview_html"
            kind: "function"
            public: true
          - name: "render_preview_html_with_mode"
            kind: "function"
            public: true
          - name: "render_preview_hmr_client"
            kind: "function"
            public: false
          - name: "render_empty_preview_html"
            kind: "function"
            public: true
          - name: "args_to_json"
            kind: "function"
            public: false
          - name: "json_string"
            kind: "function"
            public: false
          - name: "escape_html"
            kind: "function"
            public: false
          - name: "escape_js"
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
          domain: "projects/jet/src/stories"
      - path: "projects/jet/src/stories/mod.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "build"
            kind: "module"
            public: true
          - name: "controls"
            kind: "module"
            public: true
          - name: "csf"
            kind: "module"
            public: true
          - name: "deps"
            kind: "module"
            public: true
          - name: "hmr"
            kind: "module"
            public: true
          - name: "manager"
            kind: "module"
            public: true
          - name: "prop_extractor"
            kind: "module"
            public: true
          - name: "server"
            kind: "module"
            public: true
          - name: "STORY_GLOBS"
            kind: "constant"
            public: false
          - name: "StoryMeta"
            kind: "struct"
            public: true
          - name: "StoryEntry"
            kind: "struct"
            public: true
          - name: "StoryIndex"
            kind: "struct"
            public: true
          - name: "title_hierarchy"
            kind: "function"
            public: true
          - name: "discover"
            kind: "function"
            public: true
          - name: "assemble_file"
            kind: "function"
            public: false
          - name: "push_story"
            kind: "function"
            public: false
          - name: "resolve_re_export"
            kind: "function"
            public: false
          - name: "rel_display"
            kind: "function"
            public: false
          - name: "resolve_title_path"
            kind: "function"
            public: false
          - name: "strip_story_suffix"
            kind: "function"
            public: false
          - name: "slug"
            kind: "function"
            public: false
          - name: "discover_files"
            kind: "function"
            public: false
          - name: "build_globset"
            kind: "function"
            public: false
          - name: "is_hidden"
            kind: "function"
            public: false
          - name: "is_node_modules"
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
          domain: "projects/jet/src/stories"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/stories/prop_extractor.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:standardize-gap-projects-jet-src-stories-prop-extractor-rs>"
  - path: "projects/jet/src/stories/csf.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:standardize-gap-projects-jet-src-stories-csf-rs>"
  - path: "projects/jet/src/stories/build.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:standardize-gap-projects-jet-src-stories-build-rs>"
  - path: "projects/jet/src/stories/hmr.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:standardize-gap-projects-jet-src-stories-hmr-rs>"
  - path: "projects/jet/src/stories/server.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:standardize-gap-projects-jet-src-stories-server-rs>"
  - path: "projects/jet/src/stories/deps.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:standardize-gap-projects-jet-src-stories-deps-rs>"
  - path: "projects/jet/src/stories/controls.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:standardize-gap-projects-jet-src-stories-controls-rs>"
  - path: "projects/jet/src/stories/manager.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:standardize-gap-projects-jet-src-stories-manager-rs>"
  - path: "projects/jet/src/stories/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:standardize-gap-projects-jet-src-stories-mod-rs>"
```

---
id: semantic-jet-bundler
summary: Semantic coverage for "projects/jet/src/bundler"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/bundler

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/bundler"
  source_group: "projects/jet/src/bundler"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/bundler/imports.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "ModuleImports"
            kind: "struct"
            public: true
          - name: "ImportDeclaration"
            kind: "struct"
            public: true
          - name: "ImportKind"
            kind: "enum"
            public: true
          - name: "ExportDeclaration"
            kind: "struct"
            public: true
          - name: "ExportKind"
            kind: "enum"
            public: true
          - name: "extract_imports"
            kind: "function"
            public: true
          - name: "extract_from_node"
            kind: "function"
            public: false
          - name: "parse_import_statement"
            kind: "function"
            public: false
          - name: "determine_import_kind"
            kind: "function"
            public: false
          - name: "is_dynamic_import"
            kind: "function"
            public: false
          - name: "extract_dynamic_import"
            kind: "function"
            public: false
          - name: "is_require_call"
            kind: "function"
            public: false
          - name: "extract_require_specifier"
            kind: "function"
            public: false
          - name: "parse_export_statement"
            kind: "function"
            public: false
          - name: "find_child_by_kind"
            kind: "function"
            public: false
          - name: "node_text"
            kind: "function"
            public: false
          - name: "strip_quotes"
            kind: "function"
            public: false
          - name: "apply_alias"
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
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/tree_shake.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "format_tree_shake_non_utf8_module_path_warn"
            kind: "function"
            public: true
          - name: "tree_shake_module_path_matches_any_glob"
            kind: "function"
            public: true
          - name: "TreeShakeResult"
            kind: "struct"
            public: true
          - name: "analyze_used_exports"
            kind: "function"
            public: true
          - name: "shake_module"
            kind: "function"
            public: true
          - name: "extract_export_names"
            kind: "function"
            public: false
          - name: "extract_single_export_name"
            kind: "function"
            public: false
          - name: "extract_import_bindings"
            kind: "function"
            public: false
          - name: "ReexportKind"
            kind: "enum"
            public: false
          - name: "extract_reexport_bindings"
            kind: "function"
            public: false
          - name: "extract_dynamic_import_targets"
            kind: "function"
            public: true
          - name: "extract_specifier"
            kind: "function"
            public: true
          - name: "extract_imported_names"
            kind: "function"
            public: true
          - name: "extract_cjs_export_names"
            kind: "function"
            public: false
          - name: "extract_cjs_require_bindings"
            kind: "function"
            public: false
          - name: "extract_require_specifier"
            kind: "function"
            public: false
          - name: "extract_destructured_names"
            kind: "function"
            public: false
          - name: "extract_require_property_access"
            kind: "function"
            public: false
          - name: "extract_string_value"
            kind: "function"
            public: false
          - name: "find_module_by_specifier"
            kind: "function"
            public: true
          - name: "has_side_effects"
            kind: "function"
            public: true
          - name: "is_ts"
            kind: "function"
            public: false
          - name: "SideEffectsDecl"
            kind: "enum"
            public: true
          - name: "read_package_side_effects"
            kind: "function"
            public: true
          - name: "module_has_side_effects"
            kind: "function"
            public: true
          - name: "glob_matches"
            kind: "function"
            public: false
          - name: "glob_match_recursive"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3815_non_utf8_sideeffects_glob_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/types.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "BundleOptions"
            kind: "struct"
            public: true
          - name: "BuildConfig"
            kind: "struct"
            public: true
          - name: "PreloadHint"
            kind: "struct"
            public: true
          - name: "SourceMapOption"
            kind: "enum"
            public: true
          - name: "OutputFormat"
            kind: "enum"
            public: true
          - name: "BuildResult"
            kind: "struct"
            public: true
          - name: "OutputChunk"
            kind: "struct"
            public: true
          - name: "OutputChunkType"
            kind: "enum"
            public: true
          - name: "BundleOutput"
            kind: "struct"
            public: true
          - name: "Asset"
            kind: "struct"
            public: true
          - name: "AssetType"
            kind: "enum"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "default"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/mangle.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "mangle_variables"
            kind: "function"
            public: true
          - name: "mangle_variables_with_root"
            kind: "function"
            public: true
          - name: "mangle_variables_inner"
            kind: "function"
            public: false
          - name: "TK"
            kind: "enum"
            public: false
          - name: "Tok"
            kind: "struct"
            public: false
          - name: "txt"
            kind: "function"
            public: false
          - name: "tokenize"
            kind: "function"
            public: false
          - name: "is_id_start"
            kind: "function"
            public: false
          - name: "is_id_cont"
            kind: "function"
            public: false
          - name: "Scope"
            kind: "struct"
            public: false
          - name: "ScopeInfo"
            kind: "struct"
            public: false
          - name: "build_scopes"
            kind: "function"
            public: false
          - name: "compute_renames"
            kind: "function"
            public: false
          - name: "apply_renames"
            kind: "function"
            public: false
          - name: "NameGen"
            kind: "struct"
            public: false
          - name: "new"
            kind: "function"
            public: false
          - name: "next_name"
            kind: "function"
            public: false
          - name: "gen_name"
            kind: "function"
            public: false
          - name: "is_reserved"
            kind: "function"
            public: false
          - name: "RESERVED"
            kind: "constant"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/graph.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method", "ts_type_surface"]
        symbols:
          - name: "ModuleId"
            kind: "type"
            public: true
          - name: "ModuleGraph"
            kind: "struct"
            public: true
          - name: "ModuleNode"
            kind: "struct"
            public: true
          - name: "ModuleKind"
            kind: "enum"
            public: true
          - name: "EdgeKind"
            kind: "enum"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "add_module"
            kind: "function"
            public: true
          - name: "add_dependency"
            kind: "function"
            public: true
          - name: "get_module"
            kind: "function"
            public: true
          - name: "get_node"
            kind: "function"
            public: true
          - name: "topological_sort"
            kind: "function"
            public: true
          - name: "find_cycle_from"
            kind: "function"
            public: true
          - name: "has_cycle"
            kind: "function"
            public: true
          - name: "dependencies"
            kind: "function"
            public: true
          - name: "dependents_of"
            kind: "function"
            public: true
          - name: "module_count"
            kind: "function"
            public: true
          - name: "all_node_ids"
            kind: "function"
            public: true
          - name: "clear"
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
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/splitting.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Chunk"
            kind: "struct"
            public: true
          - name: "ChunkType"
            kind: "enum"
            public: true
          - name: "SplitEdge"
            kind: "struct"
            public: true
          - name: "SplitResult"
            kind: "struct"
            public: true
          - name: "ManualChunkConfig"
            kind: "struct"
            public: true
          - name: "split_chunks"
            kind: "function"
            public: true
          - name: "split_chunks_with_config"
            kind: "function"
            public: true
          - name: "generate_preload_hints"
            kind: "function"
            public: false
          - name: "bfs_static"
            kind: "function"
            public: false
          - name: "chunk_name"
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
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/css_bundle.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "CSS_BUNDLE_ASSET_STEM_FALLBACK"
            kind: "constant"
            public: true
          - name: "CSS_BUNDLE_ASSET_EXT_FALLBACK"
            kind: "constant"
            public: true
          - name: "format_css_bundle_asset_non_utf8_stem_warn"
            kind: "function"
            public: true
          - name: "format_css_bundle_asset_non_utf8_ext_warn"
            kind: "function"
            public: true
          - name: "coerce_css_bundle_asset_stem_or_warn"
            kind: "function"
            public: true
          - name: "coerce_css_bundle_asset_ext_or_warn"
            kind: "function"
            public: true
          - name: "bundle_css"
            kind: "function"
            public: true
          - name: "bundle_css_from_source"
            kind: "function"
            public: true
          - name: "resolve_css_imports"
            kind: "function"
            public: false
          - name: "process_css_source"
            kind: "function"
            public: false
          - name: "extract_css_import"
            kind: "function"
            public: false
          - name: "strip_quotes"
            kind: "function"
            public: false
          - name: "is_remote"
            kind: "function"
            public: false
          - name: "CssRewriteResult"
            kind: "struct"
            public: true
          - name: "RewrittenAsset"
            kind: "struct"
            public: true
          - name: "rewrite_css_urls"
            kind: "function"
            public: true
          - name: "compute_asset_hash"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3817_css_bundle_asset_non_utf8_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/minify.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["enum_model", "service_method"]
        symbols:
          - name: "DropStatement"
            kind: "enum"
            public: true
          - name: "minify_js"
            kind: "function"
            public: true
          - name: "minify_css"
            kind: "function"
            public: true
          - name: "strip_comments"
            kind: "function"
            public: false
          - name: "strip_css_comments"
            kind: "function"
            public: false
          - name: "drop_statements"
            kind: "function"
            public: false
          - name: "is_escaped"
            kind: "function"
            public: false
          - name: "is_regex_start"
            kind: "function"
            public: false
          - name: "needs_space_after"
            kind: "function"
            public: false
          - name: "should_insert_asi_semicolon"
            kind: "function"
            public: false
          - name: "can_end_statement"
            kind: "function"
            public: false
          - name: "can_start_statement"
            kind: "function"
            public: false
          - name: "starts_with_keyword"
            kind: "function"
            public: false
          - name: "is_identifier_char"
            kind: "function"
            public: false
          - name: "is_no_space_before"
            kind: "function"
            public: false
          - name: "replace_bool_literals"
            kind: "function"
            public: true
          - name: "is_id_char"
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
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/define.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "safe_import_meta_env_define_value"
            kind: "function"
            public: true
          - name: "replace_defines"
            kind: "function"
            public: true
          - name: "production_defines"
            kind: "function"
            public: true
          - name: "build_import_meta_env_defines"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3641_safe_import_meta_env_define_value_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/json_shake.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["enum_model", "service_method"]
        symbols:
          - name: "is_json"
            kind: "function"
            public: false
          - name: "analyze_json_imports"
            kind: "function"
            public: true
          - name: "JsonImportUsage"
            kind: "enum"
            public: true
          - name: "shake_json"
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
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/scope_hoist_opt.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "inline_cross_module_constants"
            kind: "function"
            public: true
          - name: "count_identifier_refs"
            kind: "function"
            public: false
          - name: "replace_identifier"
            kind: "function"
            public: false
          - name: "eliminate_unused_exports"
            kind: "function"
            public: true
          - name: "count_export_reads"
            kind: "function"
            public: false
          - name: "remove_export_assignment"
            kind: "function"
            public: false
          - name: "remove_var_declaration"
            kind: "function"
            public: false
          - name: "is_side_effect_free"
            kind: "function"
            public: true
          - name: "find_package_info"
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
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/sourcemap.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "SourceMap"
            kind: "struct"
            public: true
          - name: "SourceMapMode"
            kind: "enum"
            public: true
          - name: "generate_source_map"
            kind: "function"
            public: true
          - name: "append_source_map_url"
            kind: "function"
            public: true
          - name: "inline_source_map"
            kind: "function"
            public: true
          - name: "write_external_map"
            kind: "function"
            public: true
          - name: "build_mappings"
            kind: "function"
            public: false
          - name: "vlq_encode"
            kind: "function"
            public: false
          - name: "vlq_char"
            kind: "function"
            public: false
          - name: "MappingEntry"
            kind: "struct"
            public: true
          - name: "decode_mappings"
            kind: "function"
            public: true
          - name: "encode_mappings"
            kind: "function"
            public: false
          - name: "vlq_decode"
            kind: "function"
            public: false
          - name: "vlq_decode_char"
            kind: "function"
            public: false
          - name: "format_sourcemap_parse_warn"
            kind: "function"
            public: true
          - name: "SOURCEMAP_FIELD_ABSENT_FALLBACK"
            kind: "constant"
            public: true
          - name: "format_sourcemap_field_wrong_shape_warn"
            kind: "function"
            public: true
          - name: "coerce_sourcemap_string_field_or_warn"
            kind: "function"
            public: true
          - name: "compose_source_maps"
            kind: "function"
            public: true
          - name: "find_mapping_for"
            kind: "function"
            public: false
          - name: "escape_json"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3813_sourcemap_field_wrong_shape_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "css_bundle"
            kind: "module"
            public: true
          - name: "dce"
            kind: "module"
            public: true
          - name: "define"
            kind: "module"
            public: true
          - name: "fold"
            kind: "module"
            public: true
          - name: "graph"
            kind: "module"
            public: true
          - name: "html_minify"
            kind: "module"
            public: true
          - name: "imports"
            kind: "module"
            public: true
          - name: "json_shake"
            kind: "module"
            public: true
          - name: "mangle"
            kind: "module"
            public: true
          - name: "minify"
            kind: "module"
            public: true
          - name: "scope_hoist"
            kind: "module"
            public: true
          - name: "scope_hoist_opt"
            kind: "module"
            public: true
          - name: "sourcemap"
            kind: "module"
            public: true
          - name: "splitting"
            kind: "module"
            public: true
          - name: "tree_shake"
            kind: "module"
            public: true
          - name: "types"
            kind: "module"
            public: true
          - name: "BUNDLER_EDGE_KIND_NO_EXTENSION_FALLBACK"
            kind: "constant"
            public: true
          - name: "format_bundler_edge_kind_no_extension_warn"
            kind: "function"
            public: true
          - name: "format_bundler_edge_kind_non_utf8_extension_warn"
            kind: "function"
            public: true
          - name: "coerce_bundler_edge_kind_extension_or_warn"
            kind: "function"
            public: true
          - name: "determine_module_kind"
            kind: "function"
            public: false
          - name: "calculate_hash"
            kind: "function"
            public: false
          - name: "generate_wasm_glue"
            kind: "function"
            public: false
          - name: "generate_runtime"
            kind: "function"
            public: false
          - name: "generate_preload_tags"
            kind: "function"
            public: true
          - name: "inject_preload_hints"
            kind: "function"
            public: true
          - name: "UnresolvedDependency"
            kind: "struct"
            public: false
          - name: "Bundler"
            kind: "struct"
            public: true
          - name: "CompilationCache"
            kind: "struct"
            public: true
          - name: "CompiledModule"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "with_defines"
            kind: "function"
            public: true
          - name: "bundle"
            kind: "function"
            public: true
          - name: "try_process_css_entry"
            kind: "function"
            public: false
          - name: "build_graph"
            kind: "function"
            public: false
          - name: "record_unresolved"
            kind: "function"
            public: false
          - name: "check_unresolved_deps"
            kind: "function"
            public: false
          - name: "resolve_dependency"
            kind: "function"
            public: false
          - name: "transform_modules"
            kind: "function"
            public: false
          - name: "apply_tree_shaking"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/fold.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "fold_constants"
            kind: "function"
            public: true
          - name: "eliminate_dead_after_return"
            kind: "function"
            public: true
          - name: "is_id"
            kind: "function"
            public: false
          - name: "push_string"
            kind: "function"
            public: false
          - name: "is_regex_ctx"
            kind: "function"
            public: false
          - name: "push_regex"
            kind: "function"
            public: false
          - name: "skip_string"
            kind: "function"
            public: false
          - name: "fold_typeof"
            kind: "function"
            public: false
          - name: "fold_string_concat"
            kind: "function"
            public: false
          - name: "fold_numeric_bitwise"
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
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/scope_hoist.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "generate_scope_hoisted_bundle"
            kind: "function"
            public: true
          - name: "estimate_output_size"
            kind: "function"
            public: false
          - name: "is_scope_hoist_safe"
            kind: "function"
            public: true
          - name: "is_flatten_safe"
            kind: "function"
            public: true
          - name: "generate_flattened_bundle"
            kind: "function"
            public: true
          - name: "inline_module_body"
            kind: "function"
            public: false
          - name: "is_id_start_byte"
            kind: "function"
            public: true
          - name: "is_id_cont_byte"
            kind: "function"
            public: true
          - name: "is_js_decl_keyword"
            kind: "function"
            public: false
          - name: "collect_decl_names_from"
            kind: "function"
            public: false
          - name: "DeclKind"
            kind: "enum"
            public: true
          - name: "DeclInfo"
            kind: "struct"
            public: true
          - name: "collect_top_level_decls_with_kind"
            kind: "function"
            public: false
          - name: "collect_top_level_decls"
            kind: "function"
            public: false
          - name: "apply_renames_in_module_body"
            kind: "function"
            public: false
          - name: "inline_module_body_v2"
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
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/html_minify.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method"]
        symbols:
          - name: "minify_html"
            kind: "function"
            public: true
          - name: "strip_html_comments"
            kind: "function"
            public: false
          - name: "PRESERVE_WS_TAGS"
            kind: "constant"
            public: false
          - name: "collapse_html_whitespace"
            kind: "function"
            public: false
          - name: "extract_tag_name"
            kind: "function"
            public: false
          - name: "collect_tag"
            kind: "function"
            public: false
          - name: "minify_tag_attributes"
            kind: "function"
            public: false
          - name: "is_simple_attr_value"
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
          domain: "projects/jet/src/bundler"
      - path: "projects/jet/src/bundler/dce.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "build_byte_offsets"
            kind: "function"
            public: false
          - name: "slice_source"
            kind: "function"
            public: false
          - name: "eliminate_dead_code"
            kind: "function"
            public: true
          - name: "eval_condition"
            kind: "function"
            public: false
          - name: "extract_string_literal"
            kind: "function"
            public: false
          - name: "parse_bool"
            kind: "function"
            public: false
          - name: "find_matching_brace"
            kind: "function"
            public: false
          - name: "eliminate_if_blocks"
            kind: "function"
            public: false
          - name: "find_matching_paren"
            kind: "function"
            public: false
          - name: "eliminate_ternaries"
            kind: "function"
            public: false
          - name: "find_ternary_colon"
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
          domain: "projects/jet/src/bundler"
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
  - path: "projects/jet/src/bundler/imports.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/tree_shake.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/types.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/mangle.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/graph.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/splitting.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/css_bundle.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/minify.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/define.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/json_shake.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/scope_hoist_opt.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/sourcemap.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/fold.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/scope_hoist.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/html_minify.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/dce.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-bundler.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```

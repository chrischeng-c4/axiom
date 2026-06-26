---
id: semantic-jet-pkg-manager
summary: Semantic coverage for "projects/jet/src/pkg_manager"
fill_sections: [schema, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/pkg_manager

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/pkg_manager"
  source_group: "projects/jet/src/pkg_manager"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/pkg_manager/registry.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "DISK_CACHE_TTL"
            kind: "constant"
            public: false
          - name: "RegistryClient"
            kind: "struct"
            public: true
          - name: "PackageMetadata"
            kind: "struct"
            public: true
          - name: "VersionMetadata"
            kind: "struct"
            public: true
          - name: "BinField"
            kind: "enum"
            public: true
          - name: "DistInfo"
            kind: "struct"
            public: true
          - name: "format_proxy_parse_warn"
            kind: "function"
            public: true
          - name: "format_client_build_warn"
            kind: "function"
            public: true
          - name: "xdg_metadata_cache_dir"
            kind: "function"
            public: false
          - name: "safe_xdg_metadata_cache_dir"
            kind: "function"
            public: true
          - name: "format_safe_xdg_cache_warn"
            kind: "function"
            public: true
          - name: "maybe_migrate_old_cache"
            kind: "function"
            public: false
          - name: "new"
            kind: "function"
            public: true
          - name: "new_with_options"
            kind: "function"
            public: true
          - name: "disk_cache_path"
            kind: "function"
            public: false
          - name: "load_disk_cache"
            kind: "function"
            public: false
          - name: "write_disk_cache"
            kind: "function"
            public: false
          - name: "get_package_metadata"
            kind: "function"
            public: true
          - name: "get_latest_version"
            kind: "function"
            public: true
          - name: "download_package"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3610_safe_xdg_metadata_cache_dir_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/pkg_manager"
      - path: "projects/jet/src/pkg_manager/audit.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Severity"
            kind: "enum"
            public: true
          - name: "Vulnerability"
            kind: "struct"
            public: true
          - name: "AuditSummary"
            kind: "struct"
            public: true
          - name: "AuditReport"
            kind: "struct"
            public: true
          - name: "has_critical_or_high"
            kind: "function"
            public: true
          - name: "AuditClient"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "audit"
            kind: "function"
            public: true
          - name: "parse_response"
            kind: "function"
            public: false
          - name: "require_audit_string_field"
            kind: "function"
            public: true
          - name: "optional_audit_string_field"
            kind: "function"
            public: true
          - name: "format_audit_field_err"
            kind: "function"
            public: true
          - name: "describe_audit_field_kind"
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
          domain: "projects/jet/src/pkg_manager"
      - path: "projects/jet/src/pkg_manager/lockfile.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Lockfile"
            kind: "struct"
            public: true
          - name: "LockfileEntry"
            kind: "struct"
            public: true
          - name: "is_false"
            kind: "function"
            public: false
          - name: "HydrationDefect"
            kind: "enum"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "Resolution"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "compute_deps_hash"
            kind: "function"
            public: true
          - name: "from_resolved"
            kind: "function"
            public: true
          - name: "is_valid"
            kind: "function"
            public: true
          - name: "verify_hydrated"
            kind: "function"
            public: true
          - name: "to_resolved"
            kind: "function"
            public: true
          - name: "read"
            kind: "function"
            public: true
          - name: "write"
            kind: "function"
            public: true
          - name: "lockfile_read_io_ctx"
            kind: "function"
            public: true
          - name: "lockfile_parse_ctx"
            kind: "function"
            public: true
          - name: "lockfile_serialize_ctx"
            kind: "function"
            public: true
          - name: "lockfile_write_io_ctx"
            kind: "function"
            public: true
          - name: "LinkTargetClass"
            kind: "enum"
            public: true
          - name: "safe_classify_link_target"
            kind: "function"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "parse_name_from_key"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3648_safe_classify_link_target_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/pkg_manager"
      - path: "projects/jet/src/pkg_manager/gc.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "StoreGc"
            kind: "struct"
            public: true
          - name: "GcResult"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "prune"
            kind: "function"
            public: true
          - name: "collect_references"
            kind: "function"
            public: false
          - name: "list_store_entries"
            kind: "function"
            public: false
          - name: "format_gc_walkdir_warn"
            kind: "function"
            public: true
          - name: "format_gc_metadata_warn"
            kind: "function"
            public: true
          - name: "dir_size"
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
          domain: "projects/jet/src/pkg_manager"
      - path: "projects/jet/src/pkg_manager/store.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "BUILD_SCRIPT_TIMEOUT"
            kind: "constant"
            public: false
          - name: "StoreManager"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "has_package"
            kind: "function"
            public: true
          - name: "install_package"
            kind: "function"
            public: true
          - name: "link_package"
            kind: "function"
            public: true
          - name: "link_bins"
            kind: "function"
            public: true
          - name: "run_lifecycle_script"
            kind: "function"
            public: true
          - name: "get_package_path"
            kind: "function"
            public: true
          - name: "create_nested_node_modules"
            kind: "function"
            public: true
          - name: "matches_current_platform"
            kind: "function"
            public: false
          - name: "extract_platform_field"
            kind: "function"
            public: false
          - name: "describe_platform_field_kind"
            kind: "function"
            public: false
          - name: "format_platform_field_shape_warn"
            kind: "function"
            public: true
          - name: "format_platform_field_element_warn"
            kind: "function"
            public: true
          - name: "format_integrity_write_err"
            kind: "function"
            public: true
          - name: "verify_shasum"
            kind: "function"
            public: false
          - name: "safe_tarball_entry_path"
            kind: "function"
            public: true
          - name: "format_safe_tarball_entry_path_err"
            kind: "function"
            public: true
          - name: "extract_tarball"
            kind: "function"
            public: false
          - name: "hardlink_dir"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3749_platform_field_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/pkg_manager"
      - path: "projects/jet/src/pkg_manager/patch.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "PatchManager"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "prepare_patch"
            kind: "function"
            public: true
          - name: "commit_patch"
            kind: "function"
            public: true
          - name: "read_package_version"
            kind: "function"
            public: false
          - name: "describe_version_kind"
            kind: "function"
            public: false
          - name: "format_patch_missing_version_err"
            kind: "function"
            public: false
          - name: "copy_dir_recursive"
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
          domain: "projects/jet/src/pkg_manager"
      - path: "projects/jet/src/pkg_manager/nx.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ProjectJson"
            kind: "struct"
            public: true
          - name: "NxConfig"
            kind: "struct"
            public: true
          - name: "NxProjectGraph"
            kind: "struct"
            public: true
          - name: "NxGraphData"
            kind: "struct"
            public: true
          - name: "NxProject"
            kind: "struct"
            public: true
          - name: "NxProjectData"
            kind: "struct"
            public: true
          - name: "NxDependency"
            kind: "struct"
            public: true
          - name: "topological_sort"
            kind: "function"
            public: true
          - name: "project_names"
            kind: "function"
            public: true
          - name: "project_root"
            kind: "function"
            public: true
          - name: "NxWorkspaceManager"
            kind: "struct"
            public: true
          - name: "discover"
            kind: "function"
            public: true
          - name: "get_project_graph"
            kind: "function"
            public: true
          - name: "build_project_graph_from_files"
            kind: "function"
            public: true
          - name: "walk_for_project_json"
            kind: "function"
            public: false
          - name: "describe_nx_dep_kind"
            kind: "function"
            public: false
          - name: "format_nx_dep_version_shape_warn"
            kind: "function"
            public: true
          - name: "derive_rel_root"
            kind: "function"
            public: true
          - name: "format_pkg_manager_nx_rel_root_no_parent_warn"
            kind: "function"
            public: true
          - name: "format_pkg_manager_nx_rel_root_outside_workspace_warn"
            kind: "function"
            public: true
          - name: "format_pkg_manager_nx_rel_root_non_utf8_warn"
            kind: "function"
            public: true
          - name: "derive_project_name_from_dir"
            kind: "function"
            public: true
          - name: "format_pkg_manager_nx_project_name_non_utf8_warn"
            kind: "function"
            public: true
          - name: "nx_test"
            kind: "module"
            public: false
          - name: "gh3751_nx_dep_version_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/pkg_manager"
      - path: "projects/jet/src/pkg_manager/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "audit"
            kind: "module"
            public: true
          - name: "gc"
            kind: "module"
            public: true
          - name: "lockfile"
            kind: "module"
            public: true
          - name: "npmrc"
            kind: "module"
            public: true
          - name: "nx"
            kind: "module"
            public: true
          - name: "patch"
            kind: "module"
            public: true
          - name: "platform"
            kind: "module"
            public: true
          - name: "publish"
            kind: "module"
            public: true
          - name: "registry"
            kind: "module"
            public: true
          - name: "resolver"
            kind: "module"
            public: true
          - name: "store"
            kind: "module"
            public: true
          - name: "workspace"
            kind: "module"
            public: true
          - name: "MAX_CONCURRENT_DOWNLOADS"
            kind: "constant"
            public: false
          - name: "PackageManager"
            kind: "struct"
            public: true
          - name: "PackageJson"
            kind: "struct"
            public: true
          - name: "parse_package_spec"
            kind: "function"
            public: false
          - name: "format_pkg_manager_home_warn"
            kind: "function"
            public: true
          - name: "pkg_manager_home_or_fallback"
            kind: "function"
            public: true
          - name: "pkg_manager_home_from_result"
            kind: "function"
            public: true
          - name: "CI_ENV_VARS"
            kind: "constant"
            public: true
          - name: "safe_is_ci_env"
            kind: "function"
            public: true
          - name: "format_safe_is_ci_env_warn"
            kind: "function"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "new_with_flags"
            kind: "function"
            public: true
          - name: "install"
            kind: "function"
            public: true
          - name: "install_with_options"
            kind: "function"
            public: true
          - name: "install_lockfile_only"
            kind: "function"
            public: true
          - name: "update"
            kind: "function"
            public: true
          - name: "audit"
            kind: "function"
            public: true
          - name: "is_ci_env"
            kind: "function"
            public: false
          - name: "add"
            kind: "function"
            public: true
          - name: "add_many"
            kind: "function"
            public: true
          - name: "resolve_add_spec"
            kind: "function"
            public: false
          - name: "remove"
            kind: "function"
            public: true
          - name: "install_resolved"
            kind: "function"
            public: false
          - name: "read_package_json"
            kind: "function"
            public: false
          - name: "write_package_json"
            kind: "function"
            public: false
          - name: "read_package_json_raw"
            kind: "function"
            public: false
          - name: "write_package_json_raw"
            kind: "function"
            public: false
          - name: "write_lockfile"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/pkg_manager"
      - path: "projects/jet/src/pkg_manager/publish.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "format_workspace_discover_warn"
            kind: "function"
            public: true
          - name: "require_publish_identity"
            kind: "function"
            public: true
          - name: "require_publish_string_field"
            kind: "function"
            public: false
          - name: "format_publish_identity_err"
            kind: "function"
            public: true
          - name: "describe_publish_field_kind"
            kind: "function"
            public: false
          - name: "Publisher"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "pack"
            kind: "function"
            public: true
          - name: "publish"
            kind: "function"
            public: true
          - name: "read_and_transform_package_json"
            kind: "function"
            public: false
          - name: "transform_workspace_deps"
            kind: "function"
            public: false
          - name: "create_tarball"
            kind: "function"
            public: false
          - name: "create_tarball_bytes"
            kind: "function"
            public: false
          - name: "collect_publish_files"
            kind: "function"
            public: false
          - name: "base64_encode"
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
          domain: "projects/jet/src/pkg_manager"
      - path: "projects/jet/src/pkg_manager/workspace.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "PnpmWorkspaceYaml"
            kind: "struct"
            public: false
          - name: "WorkspaceConfig"
            kind: "struct"
            public: true
          - name: "HoistingConfig"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "default_hoist_patterns"
            kind: "function"
            public: false
          - name: "WorkspacePackage"
            kind: "struct"
            public: true
          - name: "WorkspaceManager"
            kind: "struct"
            public: true
          - name: "discover"
            kind: "function"
            public: true
          - name: "load_config"
            kind: "function"
            public: false
          - name: "expand_packages"
            kind: "function"
            public: false
          - name: "read_workspace_package"
            kind: "function"
            public: false
          - name: "extract_deps"
            kind: "function"
            public: false
          - name: "topological_order"
            kind: "function"
            public: true
          - name: "resolve_workspace_protocol"
            kind: "function"
            public: true
          - name: "get_package"
            kind: "function"
            public: true
          - name: "catalog_version"
            kind: "function"
            public: true
          - name: "is_workspace_protocol"
            kind: "function"
            public: true
          - name: "WorkspaceMode"
            kind: "enum"
            public: true
          - name: "detect"
            kind: "function"
            public: true
          - name: "format_workspace_pkg_warn"
            kind: "function"
            public: true
          - name: "format_workspace_deps_shape_warn"
            kind: "function"
            public: true
          - name: "parse_workspaces_field"
            kind: "function"
            public: true
          - name: "describe_json_kind"
            kind: "function"
            public: false
          - name: "workspace_config_io_ctx"
            kind: "function"
            public: true
          - name: "workspace_config_parse_ctx"
            kind: "function"
            public: true
          - name: "require_workspace_string_field"
            kind: "function"
            public: true
          - name: "format_workspace_identity_err"
            kind: "function"
            public: true
          - name: "safe_workspace_relative_path"
            kind: "function"
            public: true
          - name: "format_safe_workspace_relative_path_warn"
            kind: "function"
            public: true
          - name: "describe_workspace_field_kind"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3747_extract_deps_warn_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/pkg_manager"
      - path: "projects/jet/src/pkg_manager/npmrc.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "safe_user_npmrc_path"
            kind: "function"
            public: true
          - name: "format_safe_user_npmrc_warn"
            kind: "function"
            public: true
          - name: "format_npmrc_read_warn"
            kind: "function"
            public: true
          - name: "NpmrcConfig"
            kind: "struct"
            public: true
          - name: "load"
            kind: "function"
            public: true
          - name: "config_paths"
            kind: "function"
            public: false
          - name: "parse_file"
            kind: "function"
            public: false
          - name: "merge"
            kind: "function"
            public: false
          - name: "registry_for"
            kind: "function"
            public: true
          - name: "auth_token_for"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3608_safe_user_npmrc_path_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/pkg_manager"
      - path: "projects/jet/src/pkg_manager/nx_test.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "make_graph"
            kind: "function"
            public: false
          - name: "test_topological_sort_no_deps"
            kind: "function"
            public: false
          - name: "test_topological_sort_linear_chain"
            kind: "function"
            public: false
          - name: "test_topological_sort_diamond"
            kind: "function"
            public: false
          - name: "test_topological_sort_independent_projects"
            kind: "function"
            public: false
          - name: "test_project_names_sorted"
            kind: "function"
            public: false
          - name: "test_project_root"
            kind: "function"
            public: false
          - name: "test_nx_graph_json_parse_minimal"
            kind: "function"
            public: false
          - name: "test_nx_graph_json_parse_with_deps"
            kind: "function"
            public: false
          - name: "test_discover_no_nx_json"
            kind: "function"
            public: false
          - name: "test_discover_with_minimal_nx_json"
            kind: "function"
            public: false
          - name: "test_discover_with_full_nx_json"
            kind: "function"
            public: false
          - name: "test_discover_malformed_nx_json_returns_error"
            kind: "function"
            public: false
          - name: "write_project_json"
            kind: "function"
            public: false
          - name: "write_pkg_json"
            kind: "function"
            public: false
          - name: "test_build_graph_single_project"
            kind: "function"
            public: false
          - name: "test_build_graph_implicit_dependency_edge"
            kind: "function"
            public: false
          - name: "test_build_graph_three_project_chain"
            kind: "function"
            public: false
          - name: "test_build_graph_workspace_protocol_edge"
            kind: "function"
            public: false
          - name: "test_build_graph_project_name_fallback_to_dirname"
            kind: "function"
            public: false
          - name: "test_build_graph_skips_node_modules"
            kind: "function"
            public: false
          - name: "test_build_graph_malformed_project_json_returns_error"
            kind: "function"
            public: false
          - name: "test_build_graph_project_names_sorted"
            kind: "function"
            public: false
          - name: "test_build_graph_workspace_edge_missing_pkg_json_is_silent"
            kind: "function"
            public: false
          - name: "test_build_graph_workspace_edge_malformed_pkg_json_surfaces_warn"
            kind: "function"
            public: false
          - name: "test_build_graph_workspace_edge_via_dev_and_peer_deps"
            kind: "function"
            public: false
          - name: "test_walk_for_project_json_discovers_nested_projects"
            kind: "function"
            public: false
          - name: "test_walk_for_project_json_unreadable_subdir_keeps_siblings"
            kind: "function"
            public: false
          - name: "gh3765_rel_root_at_workspace_root_returns_empty"
            kind: "function"
            public: false
          - name: "gh3765_rel_root_nested_project_returns_relative_path"
            kind: "function"
            public: false
          - name: "gh3765_rel_root_outside_workspace_returns_empty"
            kind: "function"
            public: false
          - name: "gh3765_rel_root_non_utf8_recovers_via_lossy"
            kind: "function"
            public: false
          - name: "gh3765_helpers_include_issue_tag"
            kind: "function"
            public: false
          - name: "gh3765_helpers_distinct_from_prior_nx_warns"
            kind: "function"
            public: false
          - name: "gh3765_no_parent_message_records_path"
            kind: "function"
            public: false
          - name: "gh3765_outside_workspace_message_records_both_paths"
            kind: "function"
            public: false
          - name: "gh3765_non_utf8_message_records_lossy"
            kind: "function"
            public: false
          - name: "gh3765_helper_naming_convention_discoverable"
            kind: "function"
            public: false
          - name: "gh3772_utf8_dir_name_returns_dir_name"
            kind: "function"
            public: false
          - name: "gh3772_no_parent_returns_none"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/src/pkg_manager"
      - path: "projects/jet/src/pkg_manager/resolver.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "prewarm_speculative_dep"
            kind: "function"
            public: true
          - name: "format_speculative_prewarm_warn"
            kind: "function"
            public: true
          - name: "SPECULATIVE_DEPS"
            kind: "constant"
            public: false
          - name: "ResolvedPackage"
            kind: "struct"
            public: true
          - name: "DependencyResolver"
            kind: "struct"
            public: true
          - name: "ResolverState"
            kind: "struct"
            public: false
          - name: "new"
            kind: "function"
            public: true
          - name: "resolve"
            kind: "function"
            public: true
          - name: "resolve_with_prefetch"
            kind: "function"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "decrement_pending"
            kind: "function"
            public: false
          - name: "try_claim_package"
            kind: "function"
            public: false
          - name: "stream_resolve_package"
            kind: "function"
            public: false
          - name: "resolve_alias"
            kind: "function"
            public: false
          - name: "is_bare_package_name"
            kind: "function"
            public: false
          - name: "should_skip_optional"
            kind: "function"
            public: true
          - name: "resolve_bin_field"
            kind: "function"
            public: false
          - name: "parse_version_range"
            kind: "function"
            public: false
          - name: "parse_all_version_ranges"
            kind: "function"
            public: true
          - name: "normalize_npm_range"
            kind: "function"
            public: false
          - name: "expand_hyphen_range"
            kind: "function"
            public: false
          - name: "is_version_token"
            kind: "function"
            public: false
          - name: "find_best_version"
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
          domain: "projects/jet/src/pkg_manager"
      - path: "projects/jet/src/pkg_manager/platform.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "current_platform"
            kind: "function"
            public: true
          - name: "matches_platform"
            kind: "function"
            public: true
          - name: "matches_field"
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
          domain: "projects/jet/src/pkg_manager"
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
  - path: "projects/jet/src/pkg_manager/registry.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/audit.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/lockfile.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/gc.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/store.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/patch.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/nx.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/publish.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/workspace.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/npmrc.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/nx_test.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/resolver.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/platform.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-pkg-manager.md"
    action: verify
    section: e2e-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```

---
id: semantic-agentic-workflow-services-platform-sync
summary: Semantic coverage for "projects/agentic-workflow/src/services/platform_sync"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "This semantic TD covers AW core/client model source behavior and shared workflow domain primitives."
---

# Semantic TD: agentic-workflow/services/platform_sync

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/services/platform_sync"
  source_group: "projects/agentic-workflow/src/services/platform_sync"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/services/platform_sync/types.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model"]
        symbols:
          - name: "SpecPayload"
            kind: "struct"
            public: true
          - name: "SpecSyncResult"
            kind: "struct"
            public: true
          - name: "SyncPayload"
            kind: "struct"
            public: true
          - name: "SyncResult"
            kind: "struct"
            public: true
          - name: "SyncStatus"
            kind: "enum"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/services/platform_sync"
      - path: "projects/agentic-workflow/src/services/platform_sync/config.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "AuthConfig"
            kind: "struct"
            public: true
          - name: "LabelConfig"
            kind: "struct"
            public: true
          - name: "PlatformConfig"
            kind: "struct"
            public: true
          - name: "ScopeAutoDetect"
            kind: "struct"
            public: true
          - name: "ScopeConfig"
            kind: "struct"
            public: true
          - name: "StatusLabels"
            kind: "struct"
            public: true
          - name: "TitleConfig"
            kind: "struct"
            public: true
          - name: "load"
            kind: "function"
            public: true
          - name: "get_envfile"
            kind: "function"
            public: false
          - name: "get_envfield"
            kind: "function"
            public: false
          - name: "get_token"
            kind: "function"
            public: true
          - name: "proposal_label"
            kind: "function"
            public: true
          - name: "spec_label"
            kind: "function"
            public: true
          - name: "format_proposal_title"
            kind: "function"
            public: true
          - name: "format_spec_title"
            kind: "function"
            public: true
          - name: "extract_scope_labels"
            kind: "function"
            public: true
          - name: "ConfigFile"
            kind: "struct"
            public: false
          - name: "SddSection"
            kind: "struct"
            public: false
          - name: "resolve_path"
            kind: "function"
            public: false
          - name: "parse_env_file"
            kind: "function"
            public: false
          - name: "parse_quoted_value"
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
          domain: "projects/agentic-workflow/src/services/platform_sync"
      - path: "projects/agentic-workflow/src/services/platform_sync/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "config"
            kind: "module"
            public: false
          - name: "github"
            kind: "module"
            public: false
          - name: "payload"
            kind: "module"
            public: true
          - name: "types"
            kind: "module"
            public: false
          - name: "PlatformSyncService"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "load_config"
            kind: "function"
            public: true
          - name: "sync"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/services/platform_sync"
      - path: "projects/agentic-workflow/src/services/platform_sync/github.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "GitHubProvider"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "with_token"
            kind: "function"
            public: true
          - name: "can_sync"
            kind: "function"
            public: true
          - name: "get_api_hostname"
            kind: "function"
            public: false
          - name: "sync"
            kind: "function"
            public: true
          - name: "upsert_issue_api"
            kind: "function"
            public: false
          - name: "upsert_issue_cli"
            kind: "function"
            public: false
          - name: "run_gh"
            kind: "function"
            public: false
          - name: "sanitize_error_output"
            kind: "function"
            public: false
          - name: "parse_issue_url"
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
          domain: "projects/agentic-workflow/src/services/platform_sync"
      - path: "projects/agentic-workflow/src/services/platform_sync/payload.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "extract_frontmatter"
            kind: "function"
            public: false
          - name: "detect_line_ending"
            kind: "function"
            public: false
          - name: "extract_github_issue"
            kind: "function"
            public: false
          - name: "write_issue_to_frontmatter"
            kind: "function"
            public: true
          - name: "build_payload"
            kind: "function"
            public: true
          - name: "build_payload_with_config"
            kind: "function"
            public: true
          - name: "build_spec_payloads"
            kind: "function"
            public: false
          - name: "build_spec_body"
            kind: "function"
            public: false
          - name: "build_parent_body"
            kind: "function"
            public: false
          - name: "update_body_with_spec_links"
            kind: "function"
            public: true
          - name: "extract_summary"
            kind: "function"
            public: false
          - name: "extract_affected_code"
            kind: "function"
            public: false
          - name: "read_specs"
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
          domain: "projects/agentic-workflow/src/services/platform_sync"
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
  - path: "projects/agentic-workflow/src/services/platform_sync/types.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/services/platform_sync/config.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/services/platform_sync/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/services/platform_sync/github.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/services/platform_sync/payload.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```

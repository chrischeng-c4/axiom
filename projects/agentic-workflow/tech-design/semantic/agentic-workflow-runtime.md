---
id: semantic-agentic-workflow-runtime
summary: Semantic coverage for "projects/agentic-workflow/src/runtime"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "This semantic TD covers AW core/client model source behavior and shared workflow domain primitives."
---

# Semantic TD: agentic-workflow/runtime

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/runtime"
  source_group: "projects/agentic-workflow/src/runtime"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/runtime/mainthread.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["enum_model", "service_method"]
        symbols:
          - name: "MainthreadDecision"
            kind: "enum"
            public: true
          - name: "parse_decision"
            kind: "function"
            public: true
          - name: "strip_fence"
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
          domain: "projects/agentic-workflow/src/runtime"
      - path: "projects/agentic-workflow/src/runtime/session.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "Phase"
            kind: "enum"
            public: true
          - name: "IssueBinding"
            kind: "struct"
            public: true
          - name: "SESSION_CHANNEL_BUFFER"
            kind: "constant"
            public: false
          - name: "REQUIREMENTS_SYSTEM_PROMPT"
            kind: "constant"
            public: false
          - name: "REVIEW_SYSTEM_PROMPT"
            kind: "constant"
            public: false
          - name: "REVISE_SYSTEM_PROMPT"
            kind: "constant"
            public: false
          - name: "MAINTHREAD_SYSTEM_PROMPT"
            kind: "constant"
            public: false
          - name: "Session"
            kind: "struct"
            public: true
          - name: "builder"
            kind: "function"
            public: true
          - name: "binding"
            kind: "function"
            public: true
          - name: "next_turn_id"
            kind: "function"
            public: false
          - name: "create_issue"
            kind: "function"
            public: true
          - name: "decide"
            kind: "function"
            public: true
          - name: "turn"
            kind: "function"
            public: true
          - name: "list_issues"
            kind: "function"
            public: true
          - name: "read_issue"
            kind: "function"
            public: true
          - name: "close_issue"
            kind: "function"
            public: true
          - name: "set_binding"
            kind: "function"
            public: true
          - name: "run_create_issue"
            kind: "function"
            public: false
          - name: "run_mainthread_decide"
            kind: "function"
            public: false
          - name: "drive_lifecycle_loop"
            kind: "function"
            public: false
          - name: "DispatchPlan"
            kind: "struct"
            public: false
          - name: "ApplyVerb"
            kind: "enum"
            public: false
          - name: "dispatch_plan"
            kind: "function"
            public: false
          - name: "build_prompt"
            kind: "function"
            public: false
          - name: "extract_sections"
            kind: "function"
            public: false
          - name: "run_llm_turn"
            kind: "function"
            public: false
          - name: "SessionBuilder"
            kind: "struct"
            public: true
          - name: "provider"
            kind: "function"
            public: true
          - name: "score_process"
            kind: "function"
            public: true
          - name: "issue_backend"
            kind: "function"
            public: true
          - name: "router"
            kind: "function"
            public: true
          - name: "binding"
            kind: "function"
            public: true
          - name: "build"
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
          domain: "projects/agentic-workflow/src/runtime"
      - path: "projects/agentic-workflow/src/runtime/score_process.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "ScoreProcessError"
            kind: "enum"
            public: true
          - name: "RealScoreProcess"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "new"
            kind: "function"
            public: true
          - name: "run"
            kind: "function"
            public: false
          - name: "create"
            kind: "function"
            public: false
          - name: "fill_section_apply"
            kind: "function"
            public: false
          - name: "validate"
            kind: "function"
            public: false
          - name: "review_apply"
            kind: "function"
            public: false
          - name: "merge"
            kind: "function"
            public: false
          - name: "ScoreCall"
            kind: "enum"
            public: true
          - name: "verb"
            kind: "function"
            public: true
          - name: "MockScoreProcess"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "enqueue_create"
            kind: "function"
            public: true
          - name: "enqueue_create_err"
            kind: "function"
            public: true
          - name: "enqueue_fill_section"
            kind: "function"
            public: true
          - name: "enqueue_review"
            kind: "function"
            public: true
          - name: "enqueue_validate"
            kind: "function"
            public: true
          - name: "enqueue_merge"
            kind: "function"
            public: true
          - name: "calls"
            kind: "function"
            public: true
          - name: "create"
            kind: "function"
            public: false
          - name: "fill_section_apply"
            kind: "function"
            public: false
          - name: "validate"
            kind: "function"
            public: false
          - name: "review_apply"
            kind: "function"
            public: false
          - name: "merge"
            kind: "function"
            public: false
          - name: "LocalIssueBackend"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "with_project_root"
            kind: "function"
            public: true
          - name: "with_issues_dir"
            kind: "function"
            public: true
          - name: "local_store"
            kind: "function"
            public: false
          - name: "backend_kind"
            kind: "function"
            public: false
          - name: "create"
            kind: "function"
            public: false
          - name: "list"
            kind: "function"
            public: false
          - name: "read"
            kind: "function"
            public: false
          - name: "update"
            kind: "function"
            public: false
          - name: "close"
            kind: "function"
            public: false
          - name: "runtime_filter_matches"
            kind: "function"
            public: false
          - name: "stored_state_to_runtime"
            kind: "function"
            public: false
          - name: "issue_ref_from_stored"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/runtime"
      - path: "projects/agentic-workflow/src/runtime/github_backend.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "TOKEN_ENV_VAR"
            kind: "constant"
            public: false
          - name: "GitHubIssueBackend"
            kind: "struct"
            public: true
          - name: "from_env"
            kind: "function"
            public: true
          - name: "with_binary"
            kind: "function"
            public: true
          - name: "GhIssueJson"
            kind: "struct"
            public: false
          - name: "GhLabel"
            kind: "struct"
            public: false
          - name: "parse_state"
            kind: "function"
            public: false
          - name: "issue_ref_from_json"
            kind: "function"
            public: false
          - name: "parse_issue_number_from_url"
            kind: "function"
            public: false
          - name: "run_gh"
            kind: "function"
            public: false
          - name: "backend_kind"
            kind: "function"
            public: false
          - name: "create"
            kind: "function"
            public: false
          - name: "list"
            kind: "function"
            public: false
          - name: "read"
            kind: "function"
            public: false
          - name: "update"
            kind: "function"
            public: false
          - name: "close"
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
          domain: "projects/agentic-workflow/src/runtime"
      - path: "projects/agentic-workflow/src/runtime/gitlab_backend.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "TOKEN_ENV_VAR"
            kind: "constant"
            public: false
          - name: "GitLabIssueBackend"
            kind: "struct"
            public: true
          - name: "from_env"
            kind: "function"
            public: true
          - name: "with_binary"
            kind: "function"
            public: true
          - name: "GlabIssueJson"
            kind: "struct"
            public: false
          - name: "parse_state"
            kind: "function"
            public: false
          - name: "issue_ref_from_json"
            kind: "function"
            public: false
          - name: "parse_iid_from_url"
            kind: "function"
            public: false
          - name: "run_glab"
            kind: "function"
            public: false
          - name: "backend_kind"
            kind: "function"
            public: false
          - name: "create"
            kind: "function"
            public: false
          - name: "list"
            kind: "function"
            public: false
          - name: "read"
            kind: "function"
            public: false
          - name: "update"
            kind: "function"
            public: false
          - name: "close"
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
          domain: "projects/agentic-workflow/src/runtime"
      - path: "projects/agentic-workflow/src/runtime/event.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model"]
        symbols:
          - name: "TurnId"
            kind: "struct"
            public: true
          - name: "SessionEvent"
            kind: "enum"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/runtime"
      - path: "projects/agentic-workflow/src/runtime/issue_backend.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "BackendKind"
            kind: "enum"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "IssueId"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "IssueState"
            kind: "enum"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "ListFilter"
            kind: "struct"
            public: true
          - name: "IssueRef"
            kind: "struct"
            public: true
          - name: "IssueBody"
            kind: "struct"
            public: true
          - name: "BackendError"
            kind: "enum"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/runtime"
      - path: "projects/agentic-workflow/src/runtime/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "envelope"
            kind: "module"
            public: true
          - name: "event"
            kind: "module"
            public: true
          - name: "github_backend"
            kind: "module"
            public: true
          - name: "gitlab_backend"
            kind: "module"
            public: true
          - name: "issue_backend"
            kind: "module"
            public: true
          - name: "jira_backend"
            kind: "module"
            public: true
          - name: "mainthread"
            kind: "module"
            public: true
          - name: "router"
            kind: "module"
            public: true
          - name: "score_process"
            kind: "module"
            public: true
          - name: "session"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/runtime"
      - path: "projects/agentic-workflow/src/runtime/envelope.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Envelope"
            kind: "enum"
            public: true
          - name: "Invoke"
            kind: "struct"
            public: true
          - name: "parse"
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
          domain: "projects/agentic-workflow/src/runtime"
      - path: "projects/agentic-workflow/src/runtime/router.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "ModelChoice"
            kind: "struct"
            public: true
          - name: "Task"
            kind: "enum"
            public: true
          - name: "as_str"
            kind: "function"
            public: true
          - name: "StaticRouter"
            kind: "struct"
            public: true
          - name: "defaults"
            kind: "function"
            public: true
          - name: "empty"
            kind: "function"
            public: true
          - name: "with_route"
            kind: "function"
            public: true
          - name: "route"
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
          domain: "projects/agentic-workflow/src/runtime"
      - path: "projects/agentic-workflow/src/runtime/jira_backend.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "BASE_URL_ENV"
            kind: "constant"
            public: false
          - name: "EMAIL_ENV"
            kind: "constant"
            public: false
          - name: "TOKEN_ENV"
            kind: "constant"
            public: false
          - name: "PROJECT_ENV"
            kind: "constant"
            public: false
          - name: "JiraIssueBackend"
            kind: "struct"
            public: true
          - name: "from_env"
            kind: "function"
            public: true
          - name: "with_base_url"
            kind: "function"
            public: true
          - name: "auth_header"
            kind: "function"
            public: false
          - name: "adf_paragraph"
            kind: "function"
            public: false
          - name: "flatten_adf_to_text"
            kind: "function"
            public: false
          - name: "flatten_adf_node"
            kind: "function"
            public: false
          - name: "JiraCreateResponse"
            kind: "struct"
            public: false
          - name: "JiraSearchResponse"
            kind: "struct"
            public: false
          - name: "JiraIssueResponse"
            kind: "struct"
            public: false
          - name: "JiraFields"
            kind: "struct"
            public: false
          - name: "JiraStatus"
            kind: "struct"
            public: false
          - name: "JiraStatusCategory"
            kind: "struct"
            public: false
          - name: "map_status"
            kind: "function"
            public: false
          - name: "issue_ref_from_response"
            kind: "function"
            public: false
          - name: "build_jql"
            kind: "function"
            public: false
          - name: "map_response_status"
            kind: "function"
            public: false
          - name: "backend_kind"
            kind: "function"
            public: false
          - name: "create"
            kind: "function"
            public: false
          - name: "list"
            kind: "function"
            public: false
          - name: "read"
            kind: "function"
            public: false
          - name: "update"
            kind: "function"
            public: false
          - name: "close"
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
          domain: "projects/agentic-workflow/src/runtime"
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
  - path: "projects/agentic-workflow/src/runtime/mainthread.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/runtime/session.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/runtime/score_process.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/runtime/github_backend.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/runtime/gitlab_backend.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/runtime/event.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/runtime/issue_backend.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/runtime/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/runtime/envelope.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/runtime/router.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/runtime/jira_backend.rs"
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

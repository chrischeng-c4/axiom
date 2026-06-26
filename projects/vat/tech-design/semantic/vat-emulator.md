---
id: semantic-vat-emulator
summary: Semantic coverage for "projects/vat/src/emulator"
capability_refs:
  - id: "agent-native-gpu-native-dev-containers"
    role: primary
    claim: "host-process-execution-and-gpu-visibility"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/vat/src/emulator`."
fill_sections: [schema, changes]
---

# Semantic TD: vat/emulator

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "vat/emulator"
  source_group: "projects/vat/src/emulator"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/vat/src/emulator/dispatch.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "SECRET"
            kind: "constant"
            public: false
          - name: "Target"
            kind: "struct"
            public: true
          - name: "Oidc"
            kind: "struct"
            public: true
          - name: "OidcClaims"
            kind: "struct"
            public: false
          - name: "mint_oidc"
            kind: "function"
            public: false
          - name: "DispatchResult"
            kind: "struct"
            public: true
          - name: "dispatch_collect"
            kind: "function"
            public: true
          - name: "dispatch_http"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/emulator"
      - path: "projects/vat/src/emulator/openapi.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "MockResponse"
            kind: "struct"
            public: true
          - name: "OpenApiSpec"
            kind: "struct"
            public: true
          - name: "from_str"
            kind: "function"
            public: true
          - name: "load"
            kind: "function"
            public: true
          - name: "respond"
            kind: "function"
            public: true
          - name: "match_operation"
            kind: "function"
            public: false
          - name: "response_status"
            kind: "function"
            public: false
          - name: "select_response"
            kind: "function"
            public: false
          - name: "response_body"
            kind: "function"
            public: false
          - name: "example_from_schema"
            kind: "function"
            public: false
          - name: "resolve_ref"
            kind: "function"
            public: false
          - name: "serialize"
            kind: "function"
            public: false
          - name: "path_matches"
            kind: "function"
            public: false
          - name: "SpecRegistry"
            kind: "struct"
            public: true
          - name: "add"
            kind: "function"
            public: true
          - name: "clear"
            kind: "function"
            public: true
          - name: "respond"
            kind: "function"
            public: true
          - name: "Registration"
            kind: "struct"
            public: true
          - name: "AppState"
            kind: "struct"
            public: false
          - name: "serve"
            kind: "function"
            public: true
          - name: "handle_any"
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
          domain: "projects/vat/src/emulator"
      - path: "projects/vat/src/emulator/tasks.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method", "test_case"]
        symbols:
          - name: "AppState"
            kind: "struct"
            public: false
          - name: "Store"
            kind: "struct"
            public: false
          - name: "serve"
            kind: "function"
            public: true
          - name: "parent"
            kind: "function"
            public: false
          - name: "create_queue"
            kind: "function"
            public: false
          - name: "list_queues"
            kind: "function"
            public: false
          - name: "get_queue"
            kind: "function"
            public: false
          - name: "delete_queue"
            kind: "function"
            public: false
          - name: "create_task"
            kind: "function"
            public: false
          - name: "schedule_delay"
            kind: "function"
            public: false
          - name: "task_target"
            kind: "function"
            public: false
          - name: "deliver"
            kind: "function"
            public: false
          - name: "list_tasks"
            kind: "function"
            public: false
          - name: "strip_run"
            kind: "function"
            public: false
          - name: "get_task"
            kind: "function"
            public: false
          - name: "delete_task"
            kind: "function"
            public: false
          - name: "run_task"
            kind: "function"
            public: false
          - name: "TasksGrpc"
            kind: "struct"
            public: false
          - name: "method_to_str"
            kind: "function"
            public: false
          - name: "method_from_str"
            kind: "function"
            public: false
          - name: "ts_to_rfc3339"
            kind: "function"
            public: false
          - name: "rfc3339_to_ts"
            kind: "function"
            public: false
          - name: "task_proto_to_json"
            kind: "function"
            public: false
          - name: "task_json_to_proto"
            kind: "function"
            public: false
          - name: "queue_json_to_proto"
            kind: "function"
            public: false
          - name: "create_queue"
            kind: "function"
            public: false
          - name: "get_queue"
            kind: "function"
            public: false
          - name: "list_queues"
            kind: "function"
            public: false
          - name: "delete_queue"
            kind: "function"
            public: false
          - name: "create_task"
            kind: "function"
            public: false
          - name: "get_task"
            kind: "function"
            public: false
          - name: "list_tasks"
            kind: "function"
            public: false
          - name: "delete_task"
            kind: "function"
            public: false
          - name: "run_task"
            kind: "function"
            public: false
          - name: "update_queue"
            kind: "function"
            public: false
          - name: "purge_queue"
            kind: "function"
            public: false
          - name: "pause_queue"
            kind: "function"
            public: false
          - name: "resume_queue"
            kind: "function"
            public: false
          - name: "get_iam_policy"
            kind: "function"
            public: false
          - name: "set_iam_policy"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/vat/src/emulator"
      - path: "projects/vat/src/emulator/auth.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "PROJECT"
            kind: "constant"
            public: false
          - name: "SECRET"
            kind: "constant"
            public: false
          - name: "AppState"
            kind: "struct"
            public: false
          - name: "Store"
            kind: "struct"
            public: false
          - name: "User"
            kind: "struct"
            public: false
          - name: "Claims"
            kind: "struct"
            public: false
          - name: "serve"
            kind: "function"
            public: true
          - name: "mint"
            kind: "function"
            public: false
          - name: "local_id_of"
            kind: "function"
            public: false
          - name: "PasswordRequest"
            kind: "struct"
            public: false
          - name: "token_response"
            kind: "function"
            public: false
          - name: "sign_up"
            kind: "function"
            public: false
          - name: "sign_in"
            kind: "function"
            public: false
          - name: "lookup"
            kind: "function"
            public: false
          - name: "user_json"
            kind: "function"
            public: false
          - name: "delete_account"
            kind: "function"
            public: false
          - name: "refresh_token"
            kind: "function"
            public: false
          - name: "config"
            kind: "function"
            public: false
          - name: "list_accounts"
            kind: "function"
            public: false
          - name: "clear_accounts"
            kind: "function"
            public: false
          - name: "banner"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/emulator"
      - path: "projects/vat/src/emulator/grpc_mux.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["service_method"]
        symbols:
          - name: "serve"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/emulator"
      - path: "projects/vat/src/emulator/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["enum_model", "service_method"]
        symbols:
          - name: "auth"
            kind: "module"
            public: true
          - name: "dispatch"
            kind: "module"
            public: true
          - name: "grpc_mux"
            kind: "module"
            public: true
          - name: "httpmock"
            kind: "module"
            public: true
          - name: "openapi"
            kind: "module"
            public: true
          - name: "pubsub"
            kind: "module"
            public: true
          - name: "scheduler"
            kind: "module"
            public: true
          - name: "storage"
            kind: "module"
            public: true
          - name: "tasks"
            kind: "module"
            public: true
          - name: "workflows"
            kind: "module"
            public: true
          - name: "googleapis"
            kind: "module"
            public: true
          - name: "Kind"
            kind: "enum"
            public: true
          - name: "serve"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/emulator"
      - path: "projects/vat/src/emulator/scheduler.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "Job"
            kind: "struct"
            public: false
          - name: "AppState"
            kind: "struct"
            public: false
          - name: "serve"
            kind: "function"
            public: true
          - name: "parent"
            kind: "function"
            public: false
          - name: "create_job"
            kind: "function"
            public: false
          - name: "list_jobs"
            kind: "function"
            public: false
          - name: "get_job"
            kind: "function"
            public: false
          - name: "delete_job"
            kind: "function"
            public: false
          - name: "job_action"
            kind: "function"
            public: false
          - name: "job_target"
            kind: "function"
            public: false
          - name: "fire"
            kind: "function"
            public: false
          - name: "tick"
            kind: "function"
            public: false
          - name: "SchedulerGrpc"
            kind: "struct"
            public: false
          - name: "method_to_str"
            kind: "function"
            public: false
          - name: "method_from_str"
            kind: "function"
            public: false
          - name: "job_proto_to_json"
            kind: "function"
            public: false
          - name: "job_json_to_proto"
            kind: "function"
            public: false
          - name: "create_job"
            kind: "function"
            public: false
          - name: "get_job"
            kind: "function"
            public: false
          - name: "list_jobs"
            kind: "function"
            public: false
          - name: "delete_job"
            kind: "function"
            public: false
          - name: "run_job"
            kind: "function"
            public: false
          - name: "pause_job"
            kind: "function"
            public: false
          - name: "resume_job"
            kind: "function"
            public: false
          - name: "update_job"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/emulator"
      - path: "projects/vat/src/emulator/storage.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "Object"
            kind: "struct"
            public: false
          - name: "Store"
            kind: "struct"
            public: false
          - name: "AppState"
            kind: "struct"
            public: false
          - name: "serve"
            kind: "function"
            public: true
          - name: "decode"
            kind: "function"
            public: false
          - name: "md5_base64"
            kind: "function"
            public: false
          - name: "now"
            kind: "function"
            public: false
          - name: "object_resource"
            kind: "function"
            public: false
          - name: "store_object"
            kind: "function"
            public: false
          - name: "create_bucket"
            kind: "function"
            public: false
          - name: "list_buckets"
            kind: "function"
            public: false
          - name: "get_bucket"
            kind: "function"
            public: false
          - name: "delete_bucket"
            kind: "function"
            public: false
          - name: "list_objects"
            kind: "function"
            public: false
          - name: "get_object"
            kind: "function"
            public: false
          - name: "delete_object"
            kind: "function"
            public: false
          - name: "upload_object"
            kind: "function"
            public: false
          - name: "resumable_put"
            kind: "function"
            public: false
          - name: "parse_multipart"
            kind: "function"
            public: false
          - name: "split_on"
            kind: "function"
            public: false
          - name: "find"
            kind: "function"
            public: false
          - name: "trim_crlf"
            kind: "function"
            public: false
          - name: "not_found"
            kind: "function"
            public: false
          - name: "bad_request"
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
          domain: "projects/vat/src/emulator"
      - path: "projects/vat/src/emulator/pubsub.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "pb"
            kind: "module"
            public: true
          - name: "Sub"
            kind: "struct"
            public: false
          - name: "State"
            kind: "struct"
            public: false
          - name: "PubsubEmulator"
            kind: "struct"
            public: false
          - name: "now_ts"
            kind: "function"
            public: false
          - name: "pull_into"
            kind: "function"
            public: false
          - name: "ack"
            kind: "function"
            public: false
          - name: "create_topic"
            kind: "function"
            public: false
          - name: "get_topic"
            kind: "function"
            public: false
          - name: "list_topics"
            kind: "function"
            public: false
          - name: "list_topic_subscriptions"
            kind: "function"
            public: false
          - name: "delete_topic"
            kind: "function"
            public: false
          - name: "publish"
            kind: "function"
            public: false
          - name: "create_subscription"
            kind: "function"
            public: false
          - name: "get_subscription"
            kind: "function"
            public: false
          - name: "list_subscriptions"
            kind: "function"
            public: false
          - name: "delete_subscription"
            kind: "function"
            public: false
          - name: "modify_ack_deadline"
            kind: "function"
            public: false
          - name: "acknowledge"
            kind: "function"
            public: false
          - name: "pull"
            kind: "function"
            public: false
          - name: "streaming_pull"
            kind: "function"
            public: false
          - name: "serve"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/emulator"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/vat/src/emulator/dispatch.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/emulator/openapi.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/emulator/tasks.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/emulator/auth.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/emulator/grpc_mux.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/emulator/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/emulator/scheduler.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/emulator/storage.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/emulator/pubsub.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

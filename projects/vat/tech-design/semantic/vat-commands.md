---
id: semantic-vat-commands
summary: Semantic coverage for "projects/vat/src/commands"
capability_refs:
  - id: "agent-native-gpu-native-dev-containers"
    role: primary
    claim: "host-process-execution-and-gpu-visibility"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/vat/src/commands`."
fill_sections: [schema, changes]
---

# Semantic TD: vat/commands

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "vat/commands"
  source_group: "projects/vat/src/commands"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/vat/src/commands/llm.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method"]
        symbols:
          - name: "GUIDE"
            kind: "constant"
            public: false
          - name: "exec"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/commands"
      - path: "projects/vat/src/commands/rm.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "exec"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/commands"
      - path: "projects/vat/src/commands/ls.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "exec"
            kind: "function"
            public: true
          - name: "status_label"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/commands"
      - path: "projects/vat/src/commands/emulator.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "exec"
            kind: "function"
            public: true
          - name: "parse_routes"
            kind: "function"
            public: false
          - name: "exec"
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
          domain: "projects/vat/src/commands"
      - path: "projects/vat/src/commands/run.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Args"
            kind: "struct"
            public: true
          - name: "Target"
            kind: "enum"
            public: true
          - name: "exec"
            kind: "function"
            public: true
          - name: "RunnerArgs"
            kind: "struct"
            public: false
          - name: "DirectArgs"
            kind: "struct"
            public: false
          - name: "exec_direct"
            kind: "function"
            public: false
          - name: "exec_runner"
            kind: "function"
            public: false
          - name: "process_exit_code"
            kind: "function"
            public: false
          - name: "run_configured"
            kind: "function"
            public: false
          - name: "kill_runner_processes"
            kind: "function"
            public: false
          - name: "ordered_required_services"
            kind: "function"
            public: false
          - name: "visit_required_service"
            kind: "function"
            public: false
          - name: "RunnerProc"
            kind: "struct"
            public: false
          - name: "sandbox_wrap"
            kind: "function"
            public: true
          - name: "spawn_runner_process"
            kind: "function"
            public: false
          - name: "wait_runner_processes"
            kind: "function"
            public: false
          - name: "run_setup_step"
            kind: "function"
            public: false
          - name: "ServicePlan"
            kind: "struct"
            public: false
          - name: "ReadyProbe"
            kind: "enum"
            public: false
          - name: "ServiceHandle"
            kind: "struct"
            public: false
          - name: "prepare_service"
            kind: "function"
            public: false
          - name: "prepare_cluster_service"
            kind: "function"
            public: false
          - name: "start_service"
            kind: "function"
            public: false
          - name: "prepare_preset_service"
            kind: "function"
            public: false
          - name: "ResolvedRuntime"
            kind: "enum"
            public: false
          - name: "resolve_preset_runtime"
            kind: "function"
            public: false
          - name: "prepare_preset_docker_service"
            kind: "function"
            public: false
          - name: "prepare_firebase_service"
            kind: "function"
            public: false
          - name: "firebase_emulator_host_var"
            kind: "function"
            public: false
          - name: "builtin_emulator_info"
            kind: "function"
            public: false
          - name: "explicit_network_routes"
            kind: "function"
            public: false
          - name: "preset_auto_routes"
            kind: "function"
            public: false
          - name: "seed_preset_routes_into_proxy"
            kind: "function"
            public: false
          - name: "prepare_builtin_service"
            kind: "function"
            public: false
          - name: "http_mock_env"
            kind: "function"
            public: false
          - name: "prepare_image_service"
            kind: "function"
            public: false
          - name: "docker_run_command"
            kind: "function"
            public: false
          - name: "docker_ready_probe"
            kind: "function"
            public: false
          - name: "container_name"
            kind: "function"
            public: false
          - name: "preset_image"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/commands"
      - path: "projects/vat/src/commands/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "cluster"
            kind: "module"
            public: true
          - name: "diff"
            kind: "module"
            public: true
          - name: "emulator"
            kind: "module"
            public: true
          - name: "gpu"
            kind: "module"
            public: true
          - name: "llm"
            kind: "module"
            public: true
          - name: "logs"
            kind: "module"
            public: true
          - name: "ls"
            kind: "module"
            public: true
          - name: "rm"
            kind: "module"
            public: true
          - name: "run"
            kind: "module"
            public: true
          - name: "snapshot"
            kind: "module"
            public: true
          - name: "state"
            kind: "module"
            public: true
          - name: "print_json"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/commands"
      - path: "projects/vat/src/commands/state.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "exec"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/commands"
      - path: "projects/vat/src/commands/snapshot.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "branch"
            kind: "function"
            public: false
          - name: "snapshot"
            kind: "function"
            public: true
          - name: "fork"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/commands"
      - path: "projects/vat/src/commands/gpu.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "exec"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/commands"
      - path: "projects/vat/src/commands/diff.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "exec"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/commands"
      - path: "projects/vat/src/commands/cluster.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "CREATE_TIMEOUT"
            kind: "constant"
            public: false
          - name: "ClusterRecord"
            kind: "struct"
            public: true
          - name: "create"
            kind: "function"
            public: true
          - name: "ls"
            kind: "function"
            public: true
          - name: "kubeconfig"
            kind: "function"
            public: true
          - name: "delete"
            kind: "function"
            public: true
          - name: "default_cluster_name"
            kind: "function"
            public: false
          - name: "read_registry"
            kind: "function"
            public: false
          - name: "load_record"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/commands"
      - path: "projects/vat/src/commands/logs.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "exec"
            kind: "function"
            public: true
          - name: "print_pair"
            kind: "function"
            public: false
          - name: "print_file"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/commands"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/vat/src/commands/llm.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/commands/rm.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/commands/ls.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/commands/emulator.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/commands/run.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/commands/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/commands/state.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/commands/snapshot.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/commands/gpu.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/commands/diff.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/commands/cluster.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/vat/src/commands/logs.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

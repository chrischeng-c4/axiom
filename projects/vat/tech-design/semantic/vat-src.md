---
id: semantic-vat-src
summary: Semantic coverage for "projects/vat/src"
fill_sections: [schema, changes]
---

# Semantic TD: vat/src

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "vat/src"
  source_group: "projects/vat/src"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/vat/src/spec.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "EnvSpec"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "default_workdir"
            kind: "function"
            public: false
          - name: "Base"
            kind: "enum"
            public: true
          - name: "Isolation"
            kind: "enum"
            public: true
          - name: "GpuRequest"
            kind: "enum"
            public: true
          - name: "Limits"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src"
      - path: "projects/vat/src/id.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "fresh"
            kind: "function"
            public: true
          - name: "base36"
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
          domain: "projects/vat/src"
      - path: "projects/vat/src/config.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "FILE_NAME"
            kind: "constant"
            public: true
          - name: "VatConfig"
            kind: "struct"
            public: true
          - name: "WorkspaceConfig"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "default_dot"
            kind: "function"
            public: false
          - name: "RetentionPolicy"
            kind: "enum"
            public: true
          - name: "ClusterBackend"
            kind: "enum"
            public: true
          - name: "validate_cluster_service"
            kind: "function"
            public: false
          - name: "validate_firebase_service"
            kind: "function"
            public: false
          - name: "SetupStep"
            kind: "struct"
            public: true
          - name: "ServiceConfig"
            kind: "struct"
            public: true
          - name: "default_service_timeout"
            kind: "function"
            public: false
          - name: "RunnerConfig"
            kind: "struct"
            public: true
          - name: "load_nearest"
            kind: "function"
            public: true
          - name: "load_file"
            kind: "function"
            public: true
          - name: "validate"
            kind: "function"
            public: true
          - name: "validate_id"
            kind: "function"
            public: false
          - name: "validate_cmd"
            kind: "function"
            public: false
          - name: "runner"
            kind: "function"
            public: true
          - name: "service"
            kind: "function"
            public: true
          - name: "base_dir"
            kind: "function"
            public: true
          - name: "resolve_relative"
            kind: "function"
            public: true
          - name: "should_run_setup"
            kind: "function"
            public: true
          - name: "digest_bytes"
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
          domain: "projects/vat/src"
      - path: "projects/vat/src/lib.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface"]
        symbols:
          - name: "cluster"
            kind: "module"
            public: true
          - name: "commands"
            kind: "module"
            public: true
          - name: "config"
            kind: "module"
            public: true
          - name: "emulator"
            kind: "module"
            public: true
          - name: "event"
            kind: "module"
            public: true
          - name: "gpu"
            kind: "module"
            public: true
          - name: "id"
            kind: "module"
            public: true
          - name: "overlay"
            kind: "module"
            public: true
          - name: "paths"
            kind: "module"
            public: true
          - name: "sandbox"
            kind: "module"
            public: true
          - name: "spec"
            kind: "module"
            public: true
          - name: "state"
            kind: "module"
            public: true
          - name: "store"
            kind: "module"
            public: true
          - name: "cli"
            kind: "module"
            public: true
          - name: "VERSION"
            kind: "constant"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src"
      - path: "projects/vat/src/paths.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "root"
            kind: "function"
            public: true
          - name: "vats_dir"
            kind: "function"
            public: true
          - name: "vat_dir"
            kind: "function"
            public: true
          - name: "clusters_dir"
            kind: "function"
            public: true
          - name: "cluster_dir"
            kind: "function"
            public: true
          - name: "file"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src"
      - path: "projects/vat/src/overlay.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method", "ts_type_surface"]
        symbols:
          - name: "FileStat"
            kind: "struct"
            public: true
          - name: "Manifest"
            kind: "type"
            public: true
          - name: "clone_tree"
            kind: "function"
            public: true
          - name: "clonefile_macos"
            kind: "function"
            public: false
          - name: "clone_tree_portable"
            kind: "function"
            public: false
          - name: "copy_recursive"
            kind: "function"
            public: false
          - name: "manifest_of"
            kind: "function"
            public: true
          - name: "diff"
            kind: "function"
            public: true
          - name: "save_manifest"
            kind: "function"
            public: true
          - name: "load_manifest"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src"
      - path: "projects/vat/src/store.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "CHANGE_SAMPLE"
            kind: "constant"
            public: false
          - name: "EVENTS_TAIL"
            kind: "constant"
            public: false
          - name: "Vat"
            kind: "struct"
            public: true
          - name: "rootfs"
            kind: "function"
            public: true
          - name: "meta_path"
            kind: "function"
            public: true
          - name: "events_path"
            kind: "function"
            public: true
          - name: "base_manifest_path"
            kind: "function"
            public: true
          - name: "save"
            kind: "function"
            public: true
          - name: "log"
            kind: "function"
            public: true
          - name: "base_manifest"
            kind: "function"
            public: true
          - name: "changes"
            kind: "function"
            public: true
          - name: "project"
            kind: "function"
            public: true
          - name: "create"
            kind: "function"
            public: true
          - name: "load"
            kind: "function"
            public: true
          - name: "list"
            kind: "function"
            public: true
          - name: "remove"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src"
      - path: "projects/vat/src/event.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Event"
            kind: "struct"
            public: true
          - name: "EventKind"
            kind: "enum"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "with_data"
            kind: "function"
            public: true
          - name: "append"
            kind: "function"
            public: true
          - name: "tail"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src"
      - path: "projects/vat/src/state.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Status"
            kind: "enum"
            public: true
          - name: "RunRecord"
            kind: "struct"
            public: true
          - name: "VatMeta"
            kind: "struct"
            public: true
          - name: "ConfigRef"
            kind: "struct"
            public: true
          - name: "ClusterRunRecord"
            kind: "struct"
            public: true
          - name: "ServiceRunRecord"
            kind: "struct"
            public: true
          - name: "RunnerRunRecord"
            kind: "struct"
            public: true
          - name: "ProcessStatus"
            kind: "enum"
            public: true
          - name: "ArtifactRecord"
            kind: "struct"
            public: true
          - name: "TestRunEvidence"
            kind: "struct"
            public: true
          - name: "ChangeSet"
            kind: "struct"
            public: true
          - name: "total"
            kind: "function"
            public: true
          - name: "is_empty"
            kind: "function"
            public: true
          - name: "oneline"
            kind: "function"
            public: true
          - name: "summary"
            kind: "function"
            public: true
          - name: "ChangeSummary"
            kind: "struct"
            public: true
          - name: "WorkspaceInfo"
            kind: "struct"
            public: true
          - name: "VatState"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src"
      - path: "projects/vat/src/gpu.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "GpuInfo"
            kind: "struct"
            public: true
          - name: "detect"
            kind: "function"
            public: true
          - name: "detect_macos"
            kind: "function"
            public: false
          - name: "detect_other"
            kind: "function"
            public: false
          - name: "sysctl"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src"
      - path: "projects/vat/src/main.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "main"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src"
      - path: "projects/vat/src/cli.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Cli"
            kind: "struct"
            public: false
          - name: "Cmd"
            kind: "enum"
            public: false
          - name: "ClusterCmd"
            kind: "enum"
            public: false
          - name: "EmulatorKind"
            kind: "enum"
            public: true
          - name: "run"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src"
      - path: "projects/vat/src/cluster.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "ResolvedBackend"
            kind: "enum"
            public: true
          - name: "ClusterSpec"
            kind: "struct"
            public: true
          - name: "ClusterInfo"
            kind: "struct"
            public: true
          - name: "BackendUnavailable"
            kind: "struct"
            public: true
          - name: "backend_token"
            kind: "function"
            public: true
          - name: "resolve_backend"
            kind: "function"
            public: true
          - name: "pick_backend"
            kind: "function"
            public: false
          - name: "cluster_name"
            kind: "function"
            public: true
          - name: "kind_multinode_config"
            kind: "function"
            public: false
          - name: "run_capture"
            kind: "function"
            public: false
          - name: "docker_daemon_up"
            kind: "function"
            public: false
          - name: "which"
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
          domain: "projects/vat/src"
      - path: "projects/vat/build.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "main"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat"
      - path: "projects/vat/src/emulator/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "enum_model"]
        symbols:
          - name: "auth"
            kind: "module"
            public: true
          - name: "pubsub"
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
      - path: "projects/vat/src/emulator/auth.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "data_model"]
        symbols:
          - name: "serve"
            kind: "function"
            public: true
          - name: "sign_up"
            kind: "function"
            public: false
          - name: "sign_in"
            kind: "function"
            public: false
          - name: "lookup"
            kind: "function"
            public: false
          - name: "mint"
            kind: "function"
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
        generator_primitives: ["service_method", "data_model"]
        symbols:
          - name: "pb"
            kind: "module"
            public: true
          - name: "PubsubEmulator"
            kind: "struct"
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
      - path: "projects/vat/src/emulator/dispatch.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "data_model"]
        symbols:
          - name: "Target"
            kind: "struct"
            public: true
          - name: "Oidc"
            kind: "struct"
            public: true
          - name: "dispatch_http"
            kind: "function"
            public: true
          - name: "mint_oidc"
            kind: "function"
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
        generator_primitives: ["service_method", "data_model"]
        symbols:
          - name: "serve"
            kind: "function"
            public: true
          - name: "create_task"
            kind: "function"
            public: false
          - name: "deliver"
            kind: "function"
            public: false
          - name: "task_target"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/emulator"
      - path: "projects/vat/src/emulator/scheduler.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "data_model"]
        symbols:
          - name: "serve"
            kind: "function"
            public: true
          - name: "create_job"
            kind: "function"
            public: false
          - name: "job_action"
            kind: "function"
            public: false
          - name: "fire"
            kind: "function"
            public: false
          - name: "tick"
            kind: "function"
            public: false
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
  - path: "projects/vat/src/spec.rs"
    action: modify
    section: schema
    description: |
      Generate this vat Rust source unit from the aggregate TD AST source group.
    impl_mode: codegen
    replaces:
      - "<whole-file>"
    rust_source: |
      //! vat.toml project contract for ephemeral local agent test runs.
      //!
      //! `vat.toml` is the explicit protocol between an agent and vat: the agent
      //! declares setup, run-scoped services, and named runners; vat prepares the
      //! workspace, executes the runner, and returns structured evidence.
      
      use std::collections::{BTreeMap, BTreeSet};
      use std::path::{Path, PathBuf};
      
      use anyhow::{bail, Context, Result};
      use serde::{Deserialize, Serialize};
      
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      pub const FILE_NAME: &str = "vat.toml";
      
      /// Parsed project-level vat contract.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct VatConfig {
          pub version: u32,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub name: Option<String>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub default_runner: Option<String>,
          #[serde(default)]
          pub workspace: WorkspaceConfig,
          #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
          pub env: BTreeMap<String, String>,
          #[serde(default, skip_serializing_if = "Vec::is_empty")]
          pub setup: Vec<SetupStep>,
          #[serde(default, skip_serializing_if = "Vec::is_empty")]
          pub services: Vec<ServiceConfig>,
          #[serde(default)]
          pub runners: Vec<RunnerConfig>,
      
          #[serde(skip)]
          pub path: PathBuf,
          #[serde(skip)]
          pub root: PathBuf,
          #[serde(skip)]
          pub digest: String,
      }
      
      /// Workspace defaults for one test run.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct WorkspaceConfig {
          #[serde(default = "default_dot")]
          pub base: PathBuf,
          #[serde(default = "default_dot")]
          pub workdir: PathBuf,
          #[serde(default)]
          pub keep: RetentionPolicy,
      }
      
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-config-rs.md#source
      impl Default for WorkspaceConfig {
          fn default() -> Self {
              WorkspaceConfig {
                  base: default_dot(),
                  workdir: default_dot(),
                  keep: RetentionPolicy::default(),
              }
          }
      }
      
      fn default_dot() -> PathBuf {
          PathBuf::from(".")
      }
      
      /// Evidence retention policy after runner completion.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
      #[serde(rename_all = "snake_case")]
      pub enum RetentionPolicy {
          #[default]
          Failed,
          Always,
          Never,
      }
      
      /// Setup command executed before services start.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct SetupStep {
          pub id: String,
          pub cmd: Vec<String>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub when: Option<String>,
      }
      
      /// Run-scoped service required by a runner.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct ServiceConfig {
          pub id: String,
          #[serde(default, skip_serializing_if = "Vec::is_empty")]
          pub cmd: Vec<String>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub preset: Option<ServicePreset>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub version: Option<String>,
          #[serde(default)]
          pub port: PortSpec,
          #[serde(default, skip_serializing_if = "Vec::is_empty")]
          pub seed: Vec<PathBuf>,
          #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
          pub export: BTreeMap<String, String>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub ready_http: Option<String>,
          #[serde(default = "default_service_timeout")]
          pub timeout_s: u64,
      }
      
      /// Built-in local service presets.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
      #[serde(rename_all = "kebab-case")]
      pub enum ServicePreset {
          Postgres,
          Redis,
          Nats,
          Rabbitmq,
          Mysql,
          Mongo,
      }
      
      /// Port policy for a service. Presets default to `auto` to avoid conflicts.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
      #[serde(untagged)]
      pub enum PortSpec {
          Auto(String),
          Fixed(u16),
      }
      
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      impl Default for PortSpec {
          fn default() -> Self {
              PortSpec::Auto("auto".to_string())
          }
      }
      
      /// Why `vat run` selected a runner.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      #[derive(Debug, Clone, Copy, PartialEq, Eq)]
      pub enum RunnerSelectionReason {
          Explicit,
          DefaultRunner,
          SingleRunner,
      }
      
      fn default_service_timeout() -> u64 {
          60
      }
      
      /// Named runner an agent can invoke via `vat run <id>`.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct RunnerConfig {
          pub id: String,
          #[serde(default, skip_serializing_if = "Vec::is_empty")]
          pub requires: Vec<String>,
          pub cmd: Vec<String>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub timeout_s: Option<u64>,
          #[serde(default, skip_serializing_if = "Vec::is_empty")]
          pub artifacts: Vec<String>,
      }
      
      /// Load the nearest `vat.toml` from `start` or one of its ancestors.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
      pub fn load_nearest(start: &Path) -> Result<VatConfig> {
          let mut dir = std::fs::canonicalize(start)
              .with_context(|| format!("resolve config search dir {}", start.display()))?;
          loop {
              let candidate = dir.join(FILE_NAME);
              if candidate.exists() {
                  return load_file(&candidate);
              }
              if !dir.pop() {
                  bail!("no {FILE_NAME} found from {}", start.display());
              }
          }
      }
      
      /// Load and validate one `vat.toml` file.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      pub fn load_file(path: &Path) -> Result<VatConfig> {
          let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
          let text = std::str::from_utf8(&bytes).context("vat.toml is not valid UTF-8")?;
          let mut cfg: VatConfig = toml::from_str(text).context("parse vat.toml")?;
          if cfg.version != 1 {
              bail!("unsupported vat.toml version {}; expected 1", cfg.version);
          }
          let root = path
              .parent()
              .context("vat.toml must have a parent directory")?
              .to_path_buf();
          cfg.path = path.to_path_buf();
          cfg.root = root;
          cfg.digest = digest_bytes(&bytes);
          validate(&cfg)?;
          Ok(cfg)
      }
      
      /// Validate ids, command arrays, and runner service references.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      pub fn validate(cfg: &VatConfig) -> Result<()> {
          let mut setup_ids = BTreeSet::new();
          for step in &cfg.setup {
              validate_id("setup", &step.id)?;
              validate_cmd("setup", &step.id, &step.cmd)?;
              if !setup_ids.insert(step.id.as_str()) {
                  bail!("duplicate setup id `{}`", step.id);
              }
              if let Some(when) = &step.when {
                  if !when.starts_with("missing:") {
                      bail!("setup `{}` has unsupported when `{}`", step.id, when);
                  }
              }
          }
      
          let mut service_ids = BTreeSet::new();
          for service in &cfg.services {
              validate_id("service", &service.id)?;
              match (&service.preset, service.cmd.is_empty()) {
                  (None, true) => bail!("service `{}` must define cmd or preset", service.id),
                  (Some(_), false) => bail!(
                      "service `{}` must not define both cmd and preset",
                      service.id
                  ),
                  (None, false) => validate_cmd("service", &service.id, &service.cmd)?,
                  (Some(_), true) => {}
              }
              if let PortSpec::Auto(value) = &service.port {
                  if value != "auto" {
                      bail!("service `{}` port string must be \"auto\"", service.id);
                  }
              }
              if !service_ids.insert(service.id.as_str()) {
                  bail!("duplicate service id `{}`", service.id);
              }
          }
      
          let mut runner_ids = BTreeSet::new();
          for runner in &cfg.runners {
              validate_id("runner", &runner.id)?;
              validate_cmd("runner", &runner.id, &runner.cmd)?;
              if !runner_ids.insert(runner.id.as_str()) {
                  bail!("duplicate runner id `{}`", runner.id);
              }
              for required in &runner.requires {
                  if !service_ids.contains(required.as_str()) {
                      bail!(
                          "runner `{}` requires unknown service `{}`",
                          runner.id,
                          required
                      );
                  }
              }
          }
          if cfg.runners.is_empty() {
              bail!("vat.toml must define at least one [[runners]] entry");
          }
          if let Some(default_runner) = &cfg.default_runner {
              if !runner_ids.contains(default_runner.as_str()) {
                  bail!("default_runner `{default_runner}` does not match any runner id");
              }
          }
          Ok(())
      }
      
      fn validate_id(kind: &str, id: &str) -> Result<()> {
          if id.trim().is_empty() {
              bail!("{kind} id must not be empty");
          }
          Ok(())
      }
      
      fn validate_cmd(kind: &str, id: &str, cmd: &[String]) -> Result<()> {
          if cmd.is_empty() || cmd[0].trim().is_empty() {
              bail!("{kind} `{id}` cmd must contain a program");
          }
          Ok(())
      }
      
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-config-rs.md#source
      impl VatConfig {
          pub fn select_runner(
              &self,
              requested: Option<&str>,
          ) -> Result<(&RunnerConfig, RunnerSelectionReason)> {
              if let Some(id) = requested {
                  return Ok((self.runner(id)?, RunnerSelectionReason::Explicit));
              }
              if let Some(id) = &self.default_runner {
                  return Ok((self.runner(id)?, RunnerSelectionReason::DefaultRunner));
              }
              if self.runners.len() == 1 {
                  return Ok((&self.runners[0], RunnerSelectionReason::SingleRunner));
              }
              let ids = self
                  .runners
                  .iter()
                  .map(|runner| runner.id.as_str())
                  .collect::<Vec<_>>()
                  .join(", ");
              bail!("multiple runners; set default_runner or run `vat run <runner>` ({ids})");
          }
      
          pub fn runner(&self, id: &str) -> Result<&RunnerConfig> {
              self.runners
                  .iter()
                  .find(|r| r.id == id)
                  .with_context(|| format!("runner `{id}` not found in {}", self.path.display()))
          }
      
          pub fn service(&self, id: &str) -> Result<&ServiceConfig> {
              self.services
                  .iter()
                  .find(|s| s.id == id)
                  .with_context(|| format!("service `{id}` not found in {}", self.path.display()))
          }
      
          pub fn base_dir(&self) -> PathBuf {
              resolve_relative(&self.root, &self.workspace.base)
          }
      }
      
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-config-rs.md#source
      pub fn resolve_relative(root: &Path, path: &Path) -> PathBuf {
          if path.is_absolute() {
              path.to_path_buf()
          } else {
              root.join(path)
          }
      }
      
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-config-rs.md#source
      pub fn should_run_setup(rootfs: &Path, step: &SetupStep) -> bool {
          match step.when.as_deref() {
              Some(when) if when.starts_with("missing:") => {
                  let rel = when.trim_start_matches("missing:");
                  !rootfs.join(rel).exists()
              }
              Some(_) => true,
              None => true,
          }
      }
      
      fn digest_bytes(bytes: &[u8]) -> String {
          let mut hash = 0xcbf29ce484222325u64;
          for b in bytes {
              hash ^= u64::from(*b);
              hash = hash.wrapping_mul(0x100000001b3);
          }
          format!("fnv1a64:{hash:016x}")
      }
      
      #[cfg(test)]
      mod tests {
          use super::*;
      
          #[test]
          fn parses_valid_config() {
              let dir = tempfile::tempdir().unwrap();
              let path = dir.path().join(FILE_NAME);
              std::fs::write(
                  &path,
                  r#"
      version = 1
      name = "demo"
      
      [workspace]
      base = "."
      workdir = "."
      keep = "failed"
      
      [env]
      MODE = "test"
      
      [[setup]]
      id = "install"
      cmd = ["sh", "-c", "true"]
      when = "missing:node_modules"
      
      [[services]]
      id = "web"
      cmd = ["sh", "-c", "sleep 1"]
      ready_http = "http://127.0.0.1:1/"
      
      [[runners]]
      id = "e2e"
      requires = ["web"]
      cmd = ["sh", "-c", "true"]
      artifacts = ["out.txt"]
      "#,
              )
              .unwrap();
      
              let cfg = load_file(&path).unwrap();
              assert_eq!(cfg.version, 1);
              assert_eq!(cfg.runner("e2e").unwrap().requires, vec!["web"]);
              assert!(cfg.digest.starts_with("fnv1a64:"));
          }
      
          #[test]
          fn rejects_unknown_required_service() {
              let cfg = VatConfig {
                  version: 1,
                  name: None,
                  default_runner: None,
                  workspace: WorkspaceConfig::default(),
                  env: BTreeMap::new(),
                  setup: Vec::new(),
                  services: Vec::new(),
                  runners: vec![RunnerConfig {
                      id: "e2e".into(),
                      requires: vec!["web".into()],
                      cmd: vec!["true".into()],
                      timeout_s: None,
                      artifacts: Vec::new(),
                  }],
                  path: PathBuf::from("vat.toml"),
                  root: PathBuf::from("."),
                  digest: String::new(),
              };
              assert!(validate(&cfg).is_err());
          }
      }
  - path: "projects/vat/src/id.rs"
    action: modify
    section: schema
    description: |
      Generate this vat Rust source unit from the aggregate TD AST source group.
    impl_mode: codegen
    replaces:
      - "<whole-file>"
    rust_source: |
      //! Vat identifiers.
      //!
      //! An id is short, lowercase, and greppable: `vat-` + a base36 stamp derived
      //! from the wall clock and pid. Collisions are astronomically unlikely for a
      //! local, single-user tool; if two vats ever land on the same id, [`store`]
      //! refuses to clobber an existing directory.
      //!
      //! [`store`]: crate::store
      
      use std::process;
      use std::time::{SystemTime, UNIX_EPOCH};
      
      /// Generate a fresh vat id, e.g. `vat-7f3k1q9`.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-id-rs.md#source
      pub fn fresh() -> String {
          let nanos = SystemTime::now()
              .duration_since(UNIX_EPOCH)
              .map(|d| d.as_nanos())
              .unwrap_or(0);
          // Mix in the pid so two vats created within the same nanosecond tick
          // (e.g. a fork burst) still diverge.
          let mixed = nanos ^ ((process::id() as u128) << 80);
          format!("vat-{}", base36(mixed as u64 & 0xff_ffff_ffff))
      }
      
      /// Lowercase base36 of a u64 (no leading-zero padding; ids are opaque).
      fn base36(mut n: u64) -> String {
          const ALPHABET: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
          if n == 0 {
              return "0".to_string();
          }
          let mut buf = Vec::new();
          while n > 0 {
              buf.push(ALPHABET[(n % 36) as usize]);
              n /= 36;
          }
          buf.reverse();
          String::from_utf8(buf).expect("base36 alphabet is ascii")
      }
      
      #[cfg(test)]
      mod tests {
          use super::*;
      
          #[test]
          fn fresh_ids_have_prefix_and_differ() {
              let a = fresh();
              assert!(a.starts_with("vat-"), "got {a}");
              // The clock advances between calls, so ids differ in practice.
              let b = fresh();
              assert_ne!(a, b);
          }
      
          #[test]
          fn base36_is_stable() {
              assert_eq!(base36(0), "0");
              assert_eq!(base36(35), "z");
              assert_eq!(base36(36), "10");
          }
      }
  - path: "projects/vat/src/config.rs"
    action: modify
    section: schema
    description: |
      Generate this vat Rust source unit from the aggregate TD AST source group.
    impl_mode: codegen
    replaces:
      - "<whole-file>"
    rust_source: |
      //! vat.toml project contract for ephemeral local agent test runs.
      //!
      //! `vat.toml` is the explicit protocol between an agent and vat: the agent
      //! declares setup, run-scoped services, and named runners; vat prepares the
      //! workspace, executes the runner, and returns structured evidence.
      
      use std::collections::{BTreeMap, BTreeSet};
      use std::path::{Path, PathBuf};
      
      use anyhow::{bail, Context, Result};
      use serde::{Deserialize, Serialize};
      
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      pub const FILE_NAME: &str = "vat.toml";
      
      /// Parsed project-level vat contract.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct VatConfig {
          pub version: u32,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub name: Option<String>,
          #[serde(default)]
          pub workspace: WorkspaceConfig,
          #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
          pub env: BTreeMap<String, String>,
          #[serde(default, skip_serializing_if = "Vec::is_empty")]
          pub setup: Vec<SetupStep>,
          #[serde(default, skip_serializing_if = "Vec::is_empty")]
          pub services: Vec<ServiceConfig>,
          #[serde(default)]
          pub runners: Vec<RunnerConfig>,
      
          #[serde(skip)]
          pub path: PathBuf,
          #[serde(skip)]
          pub root: PathBuf,
          #[serde(skip)]
          pub digest: String,
      }
      
      /// Workspace defaults for one test run.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct WorkspaceConfig {
          #[serde(default = "default_dot")]
          pub base: PathBuf,
          #[serde(default = "default_dot")]
          pub workdir: PathBuf,
          #[serde(default)]
          pub keep: RetentionPolicy,
      }
      
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-config-rs.md#source
      impl Default for WorkspaceConfig {
          fn default() -> Self {
              WorkspaceConfig {
                  base: default_dot(),
                  workdir: default_dot(),
                  keep: RetentionPolicy::default(),
              }
          }
      }
      
      fn default_dot() -> PathBuf {
          PathBuf::from(".")
      }
      
      /// Evidence retention policy after runner completion.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
      #[serde(rename_all = "snake_case")]
      pub enum RetentionPolicy {
          #[default]
          Failed,
          Always,
          Never,
      }
      
      /// Setup command executed before services start.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct SetupStep {
          pub id: String,
          pub cmd: Vec<String>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub when: Option<String>,
      }
      
      /// Run-scoped service required by a runner.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct ServiceConfig {
          pub id: String,
          pub cmd: Vec<String>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub ready_http: Option<String>,
          #[serde(default = "default_service_timeout")]
          pub timeout_s: u64,
      }
      
      fn default_service_timeout() -> u64 {
          60
      }
      
      /// Named runner an agent can invoke via `vat run <id>`.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct RunnerConfig {
          pub id: String,
          #[serde(default, skip_serializing_if = "Vec::is_empty")]
          pub requires: Vec<String>,
          pub cmd: Vec<String>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub timeout_s: Option<u64>,
          #[serde(default, skip_serializing_if = "Vec::is_empty")]
          pub artifacts: Vec<String>,
      }
      
      /// Load the nearest `vat.toml` from `start` or one of its ancestors.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
      pub fn load_nearest(start: &Path) -> Result<VatConfig> {
          let mut dir = std::fs::canonicalize(start)
              .with_context(|| format!("resolve config search dir {}", start.display()))?;
          loop {
              let candidate = dir.join(FILE_NAME);
              if candidate.exists() {
                  return load_file(&candidate);
              }
              if !dir.pop() {
                  bail!("no {FILE_NAME} found from {}", start.display());
              }
          }
      }
      
      /// Load and validate one `vat.toml` file.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      pub fn load_file(path: &Path) -> Result<VatConfig> {
          let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
          let text = std::str::from_utf8(&bytes).context("vat.toml is not valid UTF-8")?;
          let mut cfg: VatConfig = toml::from_str(text).context("parse vat.toml")?;
          if cfg.version != 1 {
              bail!("unsupported vat.toml version {}; expected 1", cfg.version);
          }
          let root = path
              .parent()
              .context("vat.toml must have a parent directory")?
              .to_path_buf();
          cfg.path = path.to_path_buf();
          cfg.root = root;
          cfg.digest = digest_bytes(&bytes);
          validate(&cfg)?;
          Ok(cfg)
      }
      
      /// Validate ids, command arrays, and runner service references.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
      pub fn validate(cfg: &VatConfig) -> Result<()> {
          let mut setup_ids = BTreeSet::new();
          for step in &cfg.setup {
              validate_id("setup", &step.id)?;
              validate_cmd("setup", &step.id, &step.cmd)?;
              if !setup_ids.insert(step.id.as_str()) {
                  bail!("duplicate setup id `{}`", step.id);
              }
              if let Some(when) = &step.when {
                  if !when.starts_with("missing:") {
                      bail!("setup `{}` has unsupported when `{}`", step.id, when);
                  }
              }
          }
      
          let mut service_ids = BTreeSet::new();
          for service in &cfg.services {
              validate_id("service", &service.id)?;
              validate_cmd("service", &service.id, &service.cmd)?;
              if !service_ids.insert(service.id.as_str()) {
                  bail!("duplicate service id `{}`", service.id);
              }
          }
      
          let mut runner_ids = BTreeSet::new();
          for runner in &cfg.runners {
              validate_id("runner", &runner.id)?;
              validate_cmd("runner", &runner.id, &runner.cmd)?;
              if !runner_ids.insert(runner.id.as_str()) {
                  bail!("duplicate runner id `{}`", runner.id);
              }
              for required in &runner.requires {
                  if !service_ids.contains(required.as_str()) {
                      bail!(
                          "runner `{}` requires unknown service `{}`",
                          runner.id,
                          required
                      );
                  }
              }
          }
          if cfg.runners.is_empty() {
              bail!("vat.toml must define at least one [[runners]] entry");
          }
          Ok(())
      }
      
      fn validate_id(kind: &str, id: &str) -> Result<()> {
          if id.trim().is_empty() {
              bail!("{kind} id must not be empty");
          }
          Ok(())
      }
      
      fn validate_cmd(kind: &str, id: &str, cmd: &[String]) -> Result<()> {
          if cmd.is_empty() || cmd[0].trim().is_empty() {
              bail!("{kind} `{id}` cmd must contain a program");
          }
          Ok(())
      }
      
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-config-rs.md#source
      impl VatConfig {
          pub fn runner(&self, id: &str) -> Result<&RunnerConfig> {
              self.runners
                  .iter()
                  .find(|r| r.id == id)
                  .with_context(|| format!("runner `{id}` not found in {}", self.path.display()))
          }
      
          pub fn service(&self, id: &str) -> Result<&ServiceConfig> {
              self.services
                  .iter()
                  .find(|s| s.id == id)
                  .with_context(|| format!("service `{id}` not found in {}", self.path.display()))
          }
      
          pub fn base_dir(&self) -> PathBuf {
              resolve_relative(&self.root, &self.workspace.base)
          }
      }
      
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-config-rs.md#source
      pub fn resolve_relative(root: &Path, path: &Path) -> PathBuf {
          if path.is_absolute() {
              path.to_path_buf()
          } else {
              root.join(path)
          }
      }
      
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-config-rs.md#source
      pub fn should_run_setup(rootfs: &Path, step: &SetupStep) -> bool {
          match step.when.as_deref() {
              Some(when) if when.starts_with("missing:") => {
                  let rel = when.trim_start_matches("missing:");
                  !rootfs.join(rel).exists()
              }
              Some(_) => true,
              None => true,
          }
      }
      
      fn digest_bytes(bytes: &[u8]) -> String {
          let mut hash = 0xcbf29ce484222325u64;
          for b in bytes {
              hash ^= u64::from(*b);
              hash = hash.wrapping_mul(0x100000001b3);
          }
          format!("fnv1a64:{hash:016x}")
      }
      
      #[cfg(test)]
      mod tests {
          use super::*;
      
          #[test]
          fn parses_valid_config() {
              let dir = tempfile::tempdir().unwrap();
              let path = dir.path().join(FILE_NAME);
              std::fs::write(
                  &path,
                  r#"
      version = 1
      name = "demo"
      
      [workspace]
      base = "."
      workdir = "."
      keep = "failed"
      
      [env]
      MODE = "test"
      
      [[setup]]
      id = "install"
      cmd = ["sh", "-c", "true"]
      when = "missing:node_modules"
      
      [[services]]
      id = "web"
      cmd = ["sh", "-c", "sleep 1"]
      ready_http = "http://127.0.0.1:1/"
      
      [[runners]]
      id = "e2e"
      requires = ["web"]
      cmd = ["sh", "-c", "true"]
      artifacts = ["out.txt"]
      "#,
              )
              .unwrap();
      
              let cfg = load_file(&path).unwrap();
              assert_eq!(cfg.version, 1);
              assert_eq!(cfg.runner("e2e").unwrap().requires, vec!["web"]);
              assert!(cfg.digest.starts_with("fnv1a64:"));
          }
      
          #[test]
          fn rejects_unknown_required_service() {
              let cfg = VatConfig {
                  version: 1,
                  name: None,
                  workspace: WorkspaceConfig::default(),
                  env: BTreeMap::new(),
                  setup: Vec::new(),
                  services: Vec::new(),
                  runners: vec![RunnerConfig {
                      id: "e2e".into(),
                      requires: vec!["web".into()],
                      cmd: vec!["true".into()],
                      timeout_s: None,
                      artifacts: Vec::new(),
                  }],
                  path: PathBuf::from("vat.toml"),
                  root: PathBuf::from("."),
                  digest: String::new(),
              };
              assert!(validate(&cfg).is_err());
          }
      }
  - path: "projects/vat/src/lib.rs"
    action: modify
    section: schema
    description: |
      Generate this vat Rust source unit from the aggregate TD AST source group.
    impl_mode: codegen
    replaces:
      - "<whole-file>"
    rust_source: |
      //! vat — agent-native, GPU-native dev containers.
      //!
      //! ## What vat is
      //!
      //! A container runtime for the one user who never gets a say in Docker's
      //! design: a coding/ML **agent**. Two things make it different from "Docker
      //! minus the GUI":
      //!
      //! 1. **Agent-legible state.** Every vat projects its full current state as
      //!    one compact, structured [`state::VatState`] JSON value — what's
      //!    installed, what changed on disk vs. its base, the last run, recent
      //!    events, the GPU it can see, its fork lineage. An agent reads *one*
      //!    document to understand "what is this environment right now" instead of
      //!    parsing the scrollback of `docker ps/inspect/diff/logs`.
      //!
      //! 2. **GPU-native because there is no VM.** On Apple Silicon, Docker runs
      //!    Linux containers inside a Linux VM, and Metal has no compute passthrough
      //!    into that guest — so the M-series GPU is invisible to the container.
      //!    vat does not use a VM. A vat is a **sandboxed host process** over a
      //!    copy-on-write workspace, so the workload runs natively on macOS and the
      //!    Apple GPU (Metal / MPS / MLX) is simply present. See [`gpu`].
      //!
      //! ## The model
      //!
      //! A *vat* = a copy-on-write workspace ([`overlay`]) + a declarative
      //! [`spec::EnvSpec`] + an append-only [`event`] log + projected
      //! [`state::VatState`]. Vats are cheap to [`snapshot`](commands::snapshot) and
      //! to **fork** (try two approaches from one starting point), like git for a
      //! running environment. Isolation is a pluggable [`sandbox::Sandbox`] backend;
      //! v1 ships a host-process backend with an opt-in macOS seatbelt profile.
      
      pub mod commands;
      pub mod config;
      pub mod event;
      pub mod gpu;
      pub mod id;
      pub mod overlay;
      pub mod paths;
      pub mod sandbox;
      pub mod spec;
      pub mod state;
      pub mod store;
      
      pub mod cli;
      
      /// Crate version, surfaced by `vat --version`.
      pub const VERSION: &str = env!("CARGO_PKG_VERSION");
  - path: "projects/vat/src/paths.rs"
    action: modify
    section: schema
    description: |
      Generate this vat Rust source unit from the aggregate TD AST source group.
    impl_mode: codegen
    replaces:
      - "<whole-file>"
    rust_source: |
      //! On-disk layout for vat state.
      //!
      //! Everything lives under a single root (default `~/.vat`, override with
      //! `$VAT_HOME`). One directory per vat keeps the store trivially inspectable
      //! by a human *or* an agent with nothing but `ls`:
      //!
      //! ```text
      //! ~/.vat/
      //!   vats/
      //!     vat-7f3k1q9/
      //!       meta.json          persisted VatMeta (id, status, spec, lineage, last_run)
      //!       events.jsonl       append-only structured event log
      //!       base_manifest.json file stats captured at clone time (diff baseline)
      //!       rootfs/            the copy-on-write workspace the command runs in
      //!       logs/              per-run stdout/stderr (future)
      //! ```
      
      use std::path::PathBuf;
      
      use anyhow::{Context, Result};
      
      /// Root of all vat state. Honors `$VAT_HOME`, else `~/.vat`.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-paths-rs.md#source
      pub fn root() -> Result<PathBuf> {
          if let Some(custom) = std::env::var_os("VAT_HOME") {
              return Ok(PathBuf::from(custom));
          }
          let home = dirs::home_dir().context("could not determine home directory (set $VAT_HOME)")?;
          Ok(home.join(".vat"))
      }
      
      /// Directory holding every vat (`<root>/vats`).
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-paths-rs.md#source
      pub fn vats_dir() -> Result<PathBuf> {
          Ok(root()?.join("vats"))
      }
      
      /// Directory for a single vat (`<root>/vats/<id>`).
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-paths-rs.md#source
      pub fn vat_dir(id: &str) -> Result<PathBuf> {
          Ok(vats_dir()?.join(id))
      }
      
      /// Filenames within a vat directory. Centralized so the layout has one source
      /// of truth.
      pub mod file {
          pub const META: &str = "meta.json";
          pub const EVENTS: &str = "events.jsonl";
          pub const BASE_MANIFEST: &str = "base_manifest.json";
          pub const ROOTFS: &str = "rootfs";
          pub const LOGS: &str = "logs";
      }
  - path: "projects/vat/src/overlay.rs"
    action: modify
    section: schema
    description: |
      Generate this vat Rust source unit from the aggregate TD AST source group.
    impl_mode: codegen
    replaces:
      - "<whole-file>"
    rust_source: |
      //! Copy-on-write workspace + filesystem diffing.
      //!
      //! A vat's `rootfs` is a copy-on-write clone of its base. On APFS (macOS) this
      //! is `clonefile(2)`: cloning a whole directory tree is a near-instant
      //! metadata operation that shares blocks until written. On Linux we try a
      //! reflink copy (`cp --reflink=auto`) and fall back to a plain recursive copy.
      //!
      //! Diffing is how an agent learns "what did my run change". At clone time we
      //! capture a [`Manifest`] (path → size + mtime) as the baseline; after a run
      //! we re-walk the rootfs and compare. v1 uses size+mtime (cheap, good enough
      //! to spot changes); content hashing is a tracked refinement.
      
      use std::collections::BTreeMap;
      use std::path::Path;
      use std::time::UNIX_EPOCH;
      
      use anyhow::{bail, Context, Result};
      use serde::{Deserialize, Serialize};
      use walkdir::WalkDir;
      
      use crate::state::ChangeSet;
      
      /// Per-file stat used for cheap change detection.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-overlay-rs.md#source
      #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
      pub struct FileStat {
          pub size: u64,
          /// Modification time, ms since the Unix epoch.
          pub mtime_ms: i64,
      }
      
      /// Map of rootfs-relative path → stat. Sorted for stable diffs and output.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-overlay-rs.md#source
      pub type Manifest = BTreeMap<String, FileStat>;
      
      /// Copy-on-write clone of `src` into `dst`. `dst` must not already exist.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-overlay-rs.md#source
      pub fn clone_tree(src: &Path, dst: &Path) -> Result<()> {
          if dst.exists() {
              bail!("clone target already exists: {}", dst.display());
          }
          if !src.exists() {
              bail!("clone source does not exist: {}", src.display());
          }
          if let Some(parent) = dst.parent() {
              std::fs::create_dir_all(parent)
                  .with_context(|| format!("create parent of {}", dst.display()))?;
          }
      
          #[cfg(target_os = "macos")]
          {
              clonefile_macos(src, dst)
          }
          #[cfg(not(target_os = "macos"))]
          {
              clone_tree_portable(src, dst)
          }
      }
      
      /// macOS: one `clonefile(2)` clones the entire tree, copy-on-write.
      #[cfg(target_os = "macos")]
      fn clonefile_macos(src: &Path, dst: &Path) -> Result<()> {
          use std::ffi::CString;
          use std::os::unix::ffi::OsStrExt;
      
          let c_src = CString::new(src.as_os_str().as_bytes()).context("src path has NUL byte")?;
          let c_dst = CString::new(dst.as_os_str().as_bytes()).context("dst path has NUL byte")?;
          // clonefile(const char *src, const char *dst, int flags)
          let rc = unsafe { libc::clonefile(c_src.as_ptr(), c_dst.as_ptr(), 0) };
          if rc != 0 {
              let err = std::io::Error::last_os_error();
              // Fall back to a portable copy if the volume isn't APFS or clonefile
              // is otherwise unhappy — correctness over speed.
              eprintln!(
                  "vat: clonefile failed ({err}); falling back to recursive copy. \
                   (COW disabled — is the workspace on a non-APFS volume?)"
              );
              return copy_recursive(src, dst);
          }
          Ok(())
      }
      
      /// Portable clone: reflink via `cp` if available, else a plain recursive copy.
      #[cfg(not(target_os = "macos"))]
      fn clone_tree_portable(src: &Path, dst: &Path) -> Result<()> {
          // Try reflink first (instant COW on btrfs/xfs); `--reflink=auto` degrades
          // to a normal copy when unsupported, so this single call is enough on
          // Linux. We still keep a manual fallback for hosts without GNU cp.
          let reflink = std::process::Command::new("cp")
              .args(["-a", "--reflink=auto"])
              .arg(src)
              .arg(dst)
              .status();
          match reflink {
              Ok(s) if s.success() => return Ok(()),
              _ => {}
          }
          copy_recursive(src, dst)
      }
      
      /// Last-resort recursive copy (used as the universal fallback on every host).
      fn copy_recursive(src: &Path, dst: &Path) -> Result<()> {
          std::fs::create_dir_all(dst)?;
          for entry in WalkDir::new(src).min_depth(1) {
              let entry = entry?;
              let rel = entry.path().strip_prefix(src)?;
              let target = dst.join(rel);
              if entry.file_type().is_dir() {
                  std::fs::create_dir_all(&target)?;
              } else if entry.file_type().is_file() {
                  if let Some(parent) = target.parent() {
                      std::fs::create_dir_all(parent)?;
                  }
                  std::fs::copy(entry.path(), &target)?;
              }
          }
          Ok(())
      }
      
      /// Walk `root` and record a stat manifest of every regular file. Symlinks are
      /// not followed (we record the link's own stat); directories are implied by
      /// their files.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-overlay-rs.md#source
      pub fn manifest_of(root: &Path) -> Result<Manifest> {
          let mut m = Manifest::new();
          for entry in WalkDir::new(root).min_depth(1).follow_links(false) {
              let entry = entry.with_context(|| format!("walk {}", root.display()))?;
              if !entry.file_type().is_file() {
                  continue;
              }
              let rel = entry
                  .path()
                  .strip_prefix(root)
                  .context("strip rootfs prefix")?
                  .to_string_lossy()
                  .into_owned();
              let meta = entry.metadata().context("stat file")?;
              let mtime_ms = meta
                  .modified()
                  .ok()
                  .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                  .map(|d| d.as_millis() as i64)
                  .unwrap_or(0);
              m.insert(
                  rel,
                  FileStat {
                      size: meta.len(),
                      mtime_ms,
                  },
              );
          }
          Ok(m)
      }
      
      /// Diff a current manifest against the captured baseline.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-overlay-rs.md#source
      pub fn diff(base: &Manifest, now: &Manifest) -> ChangeSet {
          let mut cs = ChangeSet::default();
          for (path, stat) in now {
              match base.get(path) {
                  None => cs.added.push(path.clone()),
                  Some(old) if old != stat => cs.modified.push(path.clone()),
                  Some(_) => {}
              }
          }
          for path in base.keys() {
              if !now.contains_key(path) {
                  cs.deleted.push(path.clone());
              }
          }
          cs
      }
      
      /// Persist a manifest as pretty JSON.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-overlay-rs.md#source
      pub fn save_manifest(path: &Path, m: &Manifest) -> Result<()> {
          let json = serde_json::to_vec_pretty(m).context("serialize manifest")?;
          std::fs::write(path, json).with_context(|| format!("write {}", path.display()))?;
          Ok(())
      }
      
      /// Load a previously saved manifest.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-overlay-rs.md#source
      pub fn load_manifest(path: &Path) -> Result<Manifest> {
          let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
          serde_json::from_slice(&bytes).context("parse manifest")
      }
  - path: "projects/vat/src/store.rs"
    action: modify
    section: schema
    description: |
      Generate this vat Rust source unit from the aggregate TD AST source group.
    impl_mode: codegen
    replaces:
      - "<whole-file>"
    rust_source: |
      //! The vat store: create, load, list, and remove vats on disk, and project a
      //! [`VatState`] from persisted [`VatMeta`] plus live computation.
      
      use std::path::PathBuf;
      
      use anyhow::{bail, Context, Result};
      use chrono::Utc;
      
      use crate::event::{self, Event, EventKind};
      use crate::gpu;
      use crate::overlay::{self, Manifest};
      use crate::paths::{self, file};
      use crate::spec::EnvSpec;
      use crate::state::{ChangeSet, Status, VatMeta, VatState, WorkspaceInfo};
      
      /// Bounded sample size per change category in projected state.
      const CHANGE_SAMPLE: usize = 20;
      
      /// Number of trailing events surfaced in projected state.
      const EVENTS_TAIL: usize = 12;
      
      /// A handle to one vat directory plus its loaded metadata.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-store-rs.md#source
      pub struct Vat {
          pub dir: PathBuf,
          pub meta: VatMeta,
      }
      
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-store-rs.md#source
      impl Vat {
          // --- paths -----------------------------------------------------------
      
          pub fn rootfs(&self) -> PathBuf {
              self.dir.join(file::ROOTFS)
          }
          pub fn meta_path(&self) -> PathBuf {
              self.dir.join(file::META)
          }
          pub fn events_path(&self) -> PathBuf {
              self.dir.join(file::EVENTS)
          }
          pub fn base_manifest_path(&self) -> PathBuf {
              self.dir.join(file::BASE_MANIFEST)
          }
      
          // --- persistence -----------------------------------------------------
      
          /// Write `meta.json` (touches `updated_at`).
          pub fn save(&mut self) -> Result<()> {
              self.meta.updated_at = Utc::now();
              let json = serde_json::to_vec_pretty(&self.meta).context("serialize meta")?;
              std::fs::write(self.meta_path(), json)
                  .with_context(|| format!("write {}", self.meta_path().display()))?;
              Ok(())
          }
      
          /// Append an event to this vat's log.
          pub fn log(&self, ev: Event) -> Result<()> {
              event::append(&self.events_path(), &ev)
          }
      
          pub fn base_manifest(&self) -> Result<Manifest> {
              overlay::load_manifest(&self.base_manifest_path())
          }
      
          // --- projection ------------------------------------------------------
      
          /// Live filesystem changes vs. the captured baseline.
          pub fn changes(&self) -> Result<ChangeSet> {
              let base = self.base_manifest()?;
              let now = overlay::manifest_of(&self.rootfs())?;
              Ok(overlay::diff(&base, &now))
          }
      
          /// Build the full agent-legible [`VatState`].
          pub fn project(&self) -> Result<VatState> {
              let changes = self.changes().unwrap_or_default();
              let now = overlay::manifest_of(&self.rootfs()).unwrap_or_default();
              let size_bytes = now.values().map(|s| s.size).sum();
              let events_tail = event::tail(&self.events_path(), EVENTS_TAIL)?;
      
              Ok(VatState {
                  id: self.meta.id.clone(),
                  name: self.meta.name.clone(),
                  status: self.meta.status.clone(),
                  created_at: self.meta.created_at,
                  updated_at: self.meta.updated_at,
                  spec: self.meta.spec.clone(),
                  lineage: self.meta.lineage.clone(),
                  last_run: self.meta.last_run.clone(),
                  test_run: self.meta.test_run.clone(),
                  workspace: WorkspaceInfo {
                      rootfs: self.rootfs().to_string_lossy().into_owned(),
                      file_count: now.len(),
                      size_bytes,
                  },
                  changes: changes.summary(CHANGE_SAMPLE),
                  gpu: gpu::detect(),
                  events_tail,
              })
          }
      }
      
      // --- store-level operations ----------------------------------------------
      
      /// Create a new vat directory with the given spec and a fresh rootfs.
      ///
      /// `rootfs_source` is the directory to copy-on-write clone into the vat's
      /// rootfs; `None` creates an empty rootfs. `lineage` carries ancestor ids when
      /// forking. The base manifest is captured immediately so later diffs are
      /// relative to creation time.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-store-rs.md#source
      pub fn create(
          id: &str,
          name: Option<String>,
          spec: EnvSpec,
          rootfs_source: Option<&std::path::Path>,
          lineage: Vec<String>,
      ) -> Result<Vat> {
          let dir = paths::vat_dir(id)?;
          if dir.exists() {
              bail!("vat already exists: {id}");
          }
          std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
      
          let rootfs = dir.join(file::ROOTFS);
          match rootfs_source {
              Some(src) => overlay::clone_tree(src, &rootfs)
                  .with_context(|| format!("clone {} into rootfs", src.display()))?,
              None => std::fs::create_dir_all(&rootfs).context("create empty rootfs")?,
          }
      
          // Capture the diff baseline up front.
          let manifest = overlay::manifest_of(&rootfs)?;
          overlay::save_manifest(&dir.join(file::BASE_MANIFEST), &manifest)?;
      
          let now = Utc::now();
          let mut vat = Vat {
              dir,
              meta: VatMeta {
                  id: id.to_string(),
                  name,
                  status: Status::Created,
                  created_at: now,
                  updated_at: now,
                  spec,
                  lineage,
                  last_run: None,
                  test_run: None,
              },
          };
          vat.save()?;
          vat.log(Event::new(EventKind::Created, format!("created vat {id}")))?;
          Ok(vat)
      }
      
      /// Load a vat by id.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-store-rs.md#source
      pub fn load(id: &str) -> Result<Vat> {
          let dir = paths::vat_dir(id)?;
          let meta_path = dir.join(file::META);
          if !meta_path.exists() {
              bail!("no such vat: {id}");
          }
          let bytes =
              std::fs::read(&meta_path).with_context(|| format!("read {}", meta_path.display()))?;
          let meta: VatMeta = serde_json::from_slice(&bytes).context("parse meta.json")?;
          Ok(Vat { dir, meta })
      }
      
      /// List all vats (unsorted directory order; callers sort as needed).
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-store-rs.md#source
      pub fn list() -> Result<Vec<Vat>> {
          let dir = paths::vats_dir()?;
          if !dir.exists() {
              return Ok(Vec::new());
          }
          let mut out = Vec::new();
          for entry in std::fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
              let entry = entry?;
              if !entry.file_type()?.is_dir() {
                  continue;
              }
              let id = entry.file_name().to_string_lossy().into_owned();
              match load(&id) {
                  Ok(v) => out.push(v),
                  // A half-written vat dir shouldn't break `vat ls`.
                  Err(_) => continue,
              }
          }
          Ok(out)
      }
      
      /// Remove a vat directory entirely.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-store-rs.md#source
      pub fn remove(id: &str) -> Result<()> {
          let dir = paths::vat_dir(id)?;
          if !dir.exists() {
              bail!("no such vat: {id}");
          }
          std::fs::remove_dir_all(&dir).with_context(|| format!("remove {}", dir.display()))?;
          Ok(())
      }
  - path: "projects/vat/src/event.rs"
    action: modify
    section: schema
    description: |
      Generate this vat Rust source unit from the aggregate TD AST source group.
    impl_mode: codegen
    replaces:
      - "<whole-file>"
    rust_source: |
      //! Append-only structured event log.
      //!
      //! Every state transition writes one JSON line to `events.jsonl`. This is the
      //! "what happened" half of agent legibility: instead of scraping a console,
      //! an agent reads typed events with timestamps and structured payloads. The
      //! most recent few are surfaced in [`crate::state::VatState::events_tail`].
      
      use std::fs::OpenOptions;
      use std::io::{BufRead, BufReader, Write};
      use std::path::Path;
      
      use anyhow::{Context, Result};
      use chrono::{DateTime, Utc};
      use serde::{Deserialize, Serialize};
      
      /// One logged event.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-event-rs.md#source
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct Event {
          pub ts: DateTime<Utc>,
          pub kind: EventKind,
          /// Human/agent-readable summary.
          pub message: String,
          /// Optional structured payload (exit codes, paths, counts, …).
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub data: Option<serde_json::Value>,
      }
      
      /// Closed set of event kinds. Keep it small and meaningful.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-event-rs.md#source
      #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
      #[serde(rename_all = "snake_case")]
      pub enum EventKind {
          Created,
          Setup,
          RunStarted,
          RunFinished,
          Snapshot,
          Fork,
          Removed,
      }
      
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-event-rs.md#source
      impl Event {
          pub fn new(kind: EventKind, message: impl Into<String>) -> Self {
              Event {
                  ts: Utc::now(),
                  kind,
                  message: message.into(),
                  data: None,
              }
          }
      
          pub fn with_data(mut self, data: serde_json::Value) -> Self {
              self.data = Some(data);
              self
          }
      }
      
      /// Append one event to a vat's `events.jsonl`, creating it if needed.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-event-rs.md#source
      pub fn append(events_path: &Path, event: &Event) -> Result<()> {
          let line = serde_json::to_string(event).context("serialize event")?;
          let mut f = OpenOptions::new()
              .create(true)
              .append(true)
              .open(events_path)
              .with_context(|| format!("open {} for append", events_path.display()))?;
          writeln!(f, "{line}").context("write event line")?;
          Ok(())
      }
      
      /// Read up to the last `n` events (chronological order). Malformed lines are
      /// skipped rather than failing the whole read — the log must stay legible
      /// even if a write was once torn.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-event-rs.md#source
      pub fn tail(events_path: &Path, n: usize) -> Result<Vec<Event>> {
          if !events_path.exists() {
              return Ok(Vec::new());
          }
          let f = std::fs::File::open(events_path)
              .with_context(|| format!("open {}", events_path.display()))?;
          let mut all: Vec<Event> = Vec::new();
          for line in BufReader::new(f).lines() {
              let line = line.context("read event line")?;
              if line.trim().is_empty() {
                  continue;
              }
              if let Ok(ev) = serde_json::from_str::<Event>(&line) {
                  all.push(ev);
              }
          }
          let start = all.len().saturating_sub(n);
          Ok(all.split_off(start))
      }
  - path: "projects/vat/src/state.rs"
    action: modify
    section: schema
    description: |
      Generate this vat Rust source unit from the aggregate TD AST source group.
    impl_mode: codegen
    replaces:
      - "<whole-file>"
    rust_source: |
      //! The state model — vat's reason to exist.
      //!
      //! Two shapes live here:
      //!
      //! - [`VatMeta`] is what's **persisted** to `meta.json`: identity, status,
      //!   spec, lineage, and the last run. It's small and changes on transitions.
      //! - [`VatState`] is the **projection** an agent reads: meta plus things
      //!   computed on demand — the live filesystem [`ChangeSet`] vs. base, recent
      //!   [`events`](crate::event), workspace size, and the [`gpu`](crate::gpu) the
      //!   vat can see. One `vat state <id>` returns the whole document.
      //!
      //! The contract is: *an agent should never have to parse logs to understand a
      //! vat.* If understanding the environment needs a fact, it belongs in
      //! [`VatState`].
      
      use chrono::{DateTime, Utc};
      use serde::{Deserialize, Serialize};
      
      use crate::config::RetentionPolicy;
      use crate::event::Event;
      use crate::gpu::GpuInfo;
      use crate::spec::EnvSpec;
      
      /// Lifecycle status of a vat.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
      #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
      #[serde(rename_all = "snake_case", tag = "state")]
      pub enum Status {
          /// Created, never run.
          Created,
          /// A command is currently executing.
          Running,
          /// Last command finished with this exit code.
          Exited { code: i32 },
          /// A frozen, read-only label (produced by `vat snapshot`).
          Snapshot,
      }
      
      /// Persisted record of the most recent run.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct RunRecord {
          /// The program and its arguments, as invoked.
          pub command: Vec<String>,
          pub started_at: DateTime<Utc>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub finished_at: Option<DateTime<Utc>>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub exit_code: Option<i32>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub duration_ms: Option<u64>,
      }
      
      /// Persisted, on-disk record of a vat. Stored as `meta.json`.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct VatMeta {
          pub id: String,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub name: Option<String>,
          pub status: Status,
          pub created_at: DateTime<Utc>,
          pub updated_at: DateTime<Utc>,
          pub spec: EnvSpec,
          /// Ancestor vat ids, oldest first — the fork tree this vat sits in.
          #[serde(default, skip_serializing_if = "Vec::is_empty")]
          pub lineage: Vec<String>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub last_run: Option<RunRecord>,
          /// Evidence for a vat.toml runner invocation.
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub test_run: Option<TestRunEvidence>,
      }
      
      /// vat.toml config reference captured for one runner invocation.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct ConfigRef {
          pub path: String,
          pub digest: String,
      }
      
      /// Captured service state for one run-scoped dependency process.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct ServiceRunRecord {
          pub id: String,
          pub command: Vec<String>,
          pub status: ProcessStatus,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub pid: Option<u32>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub exit_code: Option<i32>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub ready_http: Option<String>,
          pub stdout_log: String,
          pub stderr_log: String,
      }
      
      /// Captured runner process state.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct RunnerRunRecord {
          pub id: String,
          pub command: Vec<String>,
          pub status: ProcessStatus,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub exit_code: Option<i32>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub duration_ms: Option<u64>,
          pub stdout_log: String,
          pub stderr_log: String,
      }
      
      /// Process status used inside test-run evidence.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
      #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
      #[serde(rename_all = "snake_case")]
      pub enum ProcessStatus {
          Created,
          Running,
          Ready,
          Exited,
          Failed,
          Timeout,
      }
      
      /// Artifact captured from a runner workspace.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct ArtifactRecord {
          pub path: String,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub size_bytes: Option<u64>,
      }
      
      /// Complete evidence bundle for one vat.toml runner invocation.
      /// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct TestRunEvidence {
          pub config: ConfigRef,
          pub runner_id: String,
          pub retention: RetentionPolicy,
          pub services: Vec<ServiceRunRecord>,
          #[serde(default, skip_serializing_if = "Option::is_none")]
          pub runner: Option<RunnerRunRecord>,
          #[serde(default, skip_serializing_if = "Vec::is_empty")]
          pub artifacts: Vec<ArtifactRecord>,
      }
      
      /// Filesystem changes vs. the base manifest. Full lists; the projection
      /// samples them for compactness.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
      #[derive(Debug, Clone, Default, Serialize, Deserialize)]
      pub struct ChangeSet {
          pub added: Vec<String>,
          pub modified: Vec<String>,
          pub deleted: Vec<String>,
      }
      
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
      impl ChangeSet {
          pub fn total(&self) -> usize {
              self.added.len() + self.modified.len() + self.deleted.len()
          }
      
          pub fn is_empty(&self) -> bool {
              self.total() == 0
          }
      
          /// One-line summary, e.g. `+3 ~1 -0`.
          pub fn oneline(&self) -> String {
              format!(
                  "+{} ~{} -{}",
                  self.added.len(),
                  self.modified.len(),
                  self.deleted.len()
              )
          }
      
          /// Compact summary for [`VatState`]: counts plus a bounded sample so the
          /// JSON stays token-cheap even when thousands of files changed.
          pub fn summary(&self, sample: usize) -> ChangeSummary {
              let take = |v: &[String]| v.iter().take(sample).cloned().collect::<Vec<_>>();
              ChangeSummary {
                  added: self.added.len(),
                  modified: self.modified.len(),
                  deleted: self.deleted.len(),
                  total: self.total(),
                  truncated: self.total() > sample * 3,
                  sample_added: take(&self.added),
                  sample_modified: take(&self.modified),
                  sample_deleted: take(&self.deleted),
              }
          }
      }
      
      /// Bounded change view embedded in [`VatState`].
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct ChangeSummary {
          pub added: usize,
          pub modified: usize,
          pub deleted: usize,
          pub total: usize,
          /// True when sample lists omit entries (full lists via `vat diff`).
          pub truncated: bool,
          pub sample_added: Vec<String>,
          pub sample_modified: Vec<String>,
          pub sample_deleted: Vec<String>,
      }
      
      /// Workspace footprint.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct WorkspaceInfo {
          pub rootfs: String,
          pub file_count: usize,
          pub size_bytes: u64,
      }
      
      /// The full, agent-legible projection of a vat. This is what `vat state`
      /// prints and what an agent should read to understand the environment.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct VatState {
          pub id: String,
          #[serde(skip_serializing_if = "Option::is_none")]
          pub name: Option<String>,
          pub status: Status,
          pub created_at: DateTime<Utc>,
          pub updated_at: DateTime<Utc>,
          pub spec: EnvSpec,
          #[serde(skip_serializing_if = "Vec::is_empty")]
          pub lineage: Vec<String>,
          #[serde(skip_serializing_if = "Option::is_none")]
          pub last_run: Option<RunRecord>,
          #[serde(skip_serializing_if = "Option::is_none")]
          pub test_run: Option<TestRunEvidence>,
          pub workspace: WorkspaceInfo,
          pub changes: ChangeSummary,
          /// The GPU this vat can reach — the headline contrast with Docker-in-VM.
          pub gpu: GpuInfo,
          pub events_tail: Vec<Event>,
      }
  - path: "projects/vat/src/gpu.rs"
    action: modify
    section: schema
    description: |
      Generate this vat Rust source unit from the aggregate TD AST source group.
    impl_mode: codegen
    replaces:
      - "<whole-file>"
    rust_source: |
      //! GPU visibility — the reason vat exists for ML agents.
      //!
      //! ## The problem vat solves
      //!
      //! On Apple Silicon, Docker runs Linux containers inside a Linux VM
      //! (Virtualization.framework / QEMU). Apple's GPU is only reachable through
      //! **Metal**, and Metal has no compute passthrough into a Linux guest — so
      //! `torch.backends.mps`, MLX, and `tensorflow-metal` all report "no GPU"
      //! inside a Docker container. There is no `--gpus all` equivalent that works.
      //!
      //! ## Why vat doesn't have the problem
      //!
      //! A vat is **not a VM**. The workload runs as a sandboxed *host* process over
      //! a copy-on-write workspace (see [`crate::overlay`] and
      //! [`crate::sandbox`]). Because the process never leaves macOS, the Metal
      //! device is simply present — the GPU was never taken away, so there is
      //! nothing to "bridge".
      //!
      //! This module reports what the host (and therefore every vat) can see, so an
      //! agent can answer "do I have a GPU, and can my vat use it?" from
      //! [`crate::state::VatState`] without guessing.
      //!
      //! v1 detection is deliberately light: chip identity via `sysctl`, presence of
      //! the Metal stack via a well-known framework path. Enumerating GPU core count
      //! and unified-memory size via the `metal` crate (a real `MTLDevice` query) is
      //! a tracked follow-up.
      
      use serde::{Deserialize, Serialize};
      
      /// What GPU acceleration a vat can reach. This is host truth: on macOS every
      /// vat shares it because every vat is a host process.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-gpu-rs.md#source
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct GpuInfo {
          /// `"apple"`, `"none"`, or another vendor on non-macOS hosts.
          pub vendor: String,
          /// Human chip string, e.g. `"Apple M3 Max"`. `None` if undetected.
          pub chip: Option<String>,
          /// Acceleration backends a workload can use right now.
          pub backends: Vec<String>,
          /// True when the GPU is reachable by host processes (always true for a
          /// real Apple Silicon host; the headline contrast with Docker-in-VM).
          pub accessible: bool,
          /// One-line explanation aimed at an agent reading state.
          pub note: String,
      }
      
      /// Detect host GPU visibility. Cheap and side-effect free; safe to call per
      /// `vat state`.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-gpu-rs.md#source
      pub fn detect() -> GpuInfo {
          #[cfg(target_os = "macos")]
          {
              detect_macos()
          }
          #[cfg(not(target_os = "macos"))]
          {
              detect_other()
          }
      }
      
      #[cfg(target_os = "macos")]
      fn detect_macos() -> GpuInfo {
          let chip = sysctl("machdep.cpu.brand_string");
          let is_apple_silicon = chip
              .as_deref()
              .map(|c| c.starts_with("Apple"))
              .unwrap_or(false);
      
          if is_apple_silicon {
              // Metal ships with the OS; MPS/MLX ride on it. We report the backends
              // a host process *can* use — whether the user installed torch/mlx is
              // their business, not the sandbox's.
              GpuInfo {
                  vendor: "apple".into(),
                  chip,
                  backends: vec!["metal".into(), "mps".into(), "mlx".into()],
                  accessible: true,
                  note: "Apple GPU is reachable: a vat is a host process, not a Linux \
                         VM, so Metal/MPS/MLX work where Docker shows no GPU."
                      .into(),
              }
          } else {
              // Intel Mac: integrated/discrete GPU via Metal, no unified-memory ML
              // story worth advertising.
              GpuInfo {
                  vendor: "apple-intel".into(),
                  chip,
                  backends: vec!["metal".into()],
                  accessible: true,
                  note: "Intel Mac: Metal available to host processes; no Apple \
                         Silicon unified-memory acceleration."
                      .into(),
              }
          }
      }
      
      #[cfg(not(target_os = "macos"))]
      fn detect_other() -> GpuInfo {
          // The Linux/other backend will grow CUDA/ROCm detection alongside its
          // namespace-based sandbox. For now report honestly that we don't probe it.
          GpuInfo {
              vendor: "unknown".into(),
              chip: None,
              backends: vec![],
              accessible: false,
              note: "Non-macOS host: GPU probing not implemented in v1 (the \
                     GPU-native story targets Apple Silicon)."
                  .into(),
          }
      }
      
      /// Read a single `sysctl` string value, or `None` if unavailable.
      #[cfg(target_os = "macos")]
      fn sysctl(key: &str) -> Option<String> {
          let out = std::process::Command::new("sysctl")
              .args(["-n", key])
              .output()
              .ok()?;
          if !out.status.success() {
              return None;
          }
          let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
          if s.is_empty() {
              None
          } else {
              Some(s)
          }
      }
  - path: "projects/vat/src/main.rs"
    action: modify
    section: schema
    description: |
      Generate this vat Rust source unit from the aggregate TD AST source group.
    impl_mode: codegen
    replaces:
      - "<whole-file>"
    rust_source: |
      use std::process::ExitCode;
      
      fn main() -> ExitCode {
          match vat::cli::run() {
              Ok(code) => code,
              Err(err) => {
                  // Print the full anyhow chain so an agent reading stderr gets the
                  // root cause, not just the top-level message.
                  eprintln!("vat: error: {err:#}");
                  ExitCode::FAILURE
              }
          }
      }
  - path: "projects/vat/src/cli.rs"
    action: modify
    section: schema
    description: |
      Generate this vat Rust source unit from the aggregate TD AST source group.
    impl_mode: codegen
    replaces:
      - "<whole-file>"
    rust_source: |
      //! CLI surface.
      //!
      //! Verbs are deliberately few and composable, because the operator is an
      //! agent, not a human juggling a dashboard. The defaults that matter for an
      //! agent — JSON state, forwarded exit codes, copy-on-write disposability — are
      //! the *unflagged* path. The README carries the tradeoff rationale for where
      //! vat departs from Docker's human-dev ergonomics.
      
      use std::path::PathBuf;
      use std::process::ExitCode;
      
      use anyhow::Result;
      use clap::{Parser, Subcommand};
      
      use crate::commands;
      use crate::spec::{GpuRequest, Isolation};
      
      #[derive(Parser)]
      #[command(
          name = "vat",
          version = crate::VERSION,
          about = "agent-native, GPU-native dev containers (no VM: the Apple GPU just works)",
          long_about = "agent-native, GPU-native dev containers (no VM: the Apple GPU just works)\n\nRun `vat llm` for the compact agent-facing usage contract, including when to use vat.toml, how to inspect evidence, and what Docker-like assumptions do not apply."
      )]
      struct Cli {
          #[command(subcommand)]
          cmd: Cmd,
      }
      
      #[derive(Subcommand)]
      enum Cmd {
          /// Create a fresh vat and run a command inside it.
          Run {
              /// Named runner from vat.toml. Omit when using `vat run -- <command>`.
              runner: Option<String>,
              /// Clone from this host directory (default: current directory).
              #[arg(long)]
              base: Option<PathBuf>,
              /// Fork from an existing vat instead of a host directory.
              #[arg(long)]
              from: Option<String>,
              /// Optional human label for the vat.
              #[arg(long)]
              name: Option<String>,
              /// Isolation backend.
              #[arg(long, value_enum, default_value = "none")]
              isolation: Isolation,
              /// GPU expectation.
              #[arg(long, value_enum, default_value = "auto")]
              gpu: GpuRequest,
              /// Print full VatState JSON instead of a human summary.
              #[arg(long)]
              json: bool,
              /// Direct command mode, e.g. `vat run -- python train.py`.
              #[arg(last = true, allow_hyphen_values = true, value_name = "COMMAND")]
              cmd: Vec<String>,
          },
          /// List all vats.
          Ls {
              #[arg(long)]
              json: bool,
          },
          /// Print the full agent-legible state of a vat as JSON.
          State {
              id: String,
              /// Single-line JSON instead of pretty.
              #[arg(long)]
              compact: bool,
          },
          /// Show every filesystem change vs. the vat's base.
          Diff {
              id: String,
              #[arg(long)]
              json: bool,
          },
          /// Fork a vat into a new runnable working copy.
          Fork {
              id: String,
              #[arg(long)]
              name: Option<String>,
          },
          /// Freeze a vat into an immutable snapshot.
          Snapshot {
              id: String,
              #[arg(long)]
              name: Option<String>,
          },
          /// Delete a vat and its workspace.
          Rm { id: String },
          /// Print captured logs from a vat.toml runner invocation.
          Logs { id: String, source: Option<String> },
          /// Print the compact LLM/agent usage guide.
          Llm,
          /// Report the GPU every vat on this host can reach.
          Gpu {
              #[arg(long)]
              json: bool,
          },
      }
      
      /// Parse argv and dispatch. Returns the process exit code (notably, `run`
      /// forwards the child command's code).
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-cli-rs.md#source
      pub fn run() -> Result<ExitCode> {
          let cli = Cli::parse();
          match cli.cmd {
              Cmd::Run {
                  runner,
                  base,
                  from,
                  name,
                  isolation,
                  gpu,
                  json,
                  mut cmd,
              } => {
                  let target = if !cmd.is_empty() {
                      let program = cmd.remove(0);
                      commands::run::Target::Direct {
                          program,
                          program_args: cmd,
                      }
                  } else if let Some(runner_id) = runner {
                      commands::run::Target::Runner { runner_id }
                  } else {
                      anyhow::bail!("expected `vat run <runner-id>` or `vat run -- <command>`");
                  };
                  commands::run::exec(commands::run::Args {
                      target,
                      base,
                      from,
                      name,
                      isolation,
                      gpu,
                      json,
                  })
              }
              Cmd::Ls { json } => commands::ls::exec(json),
              Cmd::State { id, compact } => commands::state::exec(id, compact),
              Cmd::Diff { id, json } => commands::diff::exec(id, json),
              Cmd::Fork { id, name } => commands::snapshot::fork(id, name),
              Cmd::Snapshot { id, name } => commands::snapshot::snapshot(id, name),
              Cmd::Rm { id } => commands::rm::exec(id),
              Cmd::Logs { id, source } => commands::logs::exec(id, source),
              Cmd::Llm => commands::llm::exec(),
              Cmd::Gpu { json } => commands::gpu::exec(json),
          }
      }
```

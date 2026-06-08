---
id: vat-source-projects-vat-src-config-rs
summary: Source replay payload for projects/vat/src/config.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: agent-legible-state-and-diff-surface
    claim: agent-legible-state-and-diff-surface
    coverage: full
    rationale: "This source replay TD preserves vat.toml runner evidence, local service orchestration, and agent-legible run state."
---

# Source TD: projects/vat/src/config.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/config.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FILE_NAME` | projects/vat/src/config.rs | constant | pub | 16 |  |
| `RetentionPolicy` | projects/vat/src/config.rs | enum | pub | 75 |  |
| `RunnerConfig` | projects/vat/src/config.rs | struct | pub | 111 |  |
| `ServiceConfig` | projects/vat/src/config.rs | struct | pub | 95 |  |
| `SetupStep` | projects/vat/src/config.rs | struct | pub | 85 |  |
| `VatConfig` | projects/vat/src/config.rs | struct | pub | 21 |  |
| `WorkspaceConfig` | projects/vat/src/config.rs | struct | pub | 47 |  |
| `base_dir` | projects/vat/src/config.rs | function | pub | 237 | base_dir(&self) -> PathBuf |
| `load_file` | projects/vat/src/config.rs | function | pub | 140 | load_file(path: &Path) -> Result<VatConfig> |
| `load_nearest` | projects/vat/src/config.rs | function | pub | 124 | load_nearest(start: &Path) -> Result<VatConfig> |
| `resolve_relative` | projects/vat/src/config.rs | function | pub | 243 | resolve_relative(root: &Path, path: &Path) -> PathBuf |
| `runner` | projects/vat/src/config.rs | function | pub | 223 | runner(&self, id: &str) -> Result<&RunnerConfig> |
| `service` | projects/vat/src/config.rs | function | pub | 230 | service(&self, id: &str) -> Result<&ServiceConfig> |
| `should_run_setup` | projects/vat/src/config.rs | function | pub | 252 | should_run_setup(rootfs: &Path, step: &SetupStep) -> bool |
| `validate` | projects/vat/src/config.rs | function | pub | 160 | validate(cfg: &VatConfig) -> Result<()> |
## Source
<!-- type: source lang: rust -->

`````rust
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

`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/config.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-src.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-config-rs-source-replay-superseded>"
```

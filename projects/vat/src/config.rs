// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-src.md#schema
// CODEGEN-BEGIN
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
    pub requires: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cmd: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<ServicePreset>,
    /// Docker image backing this service. Mutually exclusive with `cmd` and
    /// `preset`. vat starts it via `docker run` as a managed foreground child;
    /// the runner itself is never containerized, so the host GPU story holds.
    /// vat is not an image builder/registry — it pulls and runs, nothing more.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// Port the service listens on *inside* the image. Mapped to the
    /// auto-allocated (or fixed `port`) host port. Required for image services.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_port: Option<u16>,
    /// Environment variables passed *into* the container (e.g.
    /// `POSTGRES_PASSWORD`). Image services only.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub image_env: BTreeMap<String, String>,
    /// How a `preset` service is provided. `auto` (default) prefers the native
    /// host binary (Homebrew) and falls back to the preset's official Docker
    /// image when the binary is missing; `native` forces the binary; `docker`
    /// forces the image. Only meaningful with `preset` — `image` services are
    /// always Docker and `cmd` services are always native.
    #[serde(default)]
    pub runtime: ServiceRuntime,
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
    /// Corpus-aware readiness command. "Ready" means this command exits 0
    /// (e.g. a SQL row-count `>= N` check), not merely that the server
    /// process accepts connections. Overrides a preset's default probe.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ready_cmd: Vec<String>,
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
    Opensearch,
}

/// How a `preset` service is provided. The default prefers the native binary
/// (Homebrew) so the host GPU and zero-friction model hold, and only reaches
/// for Docker when the binary is absent — or when the preset has no native
/// equivalent on this host.
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ServiceRuntime {
    /// Prefer the native binary; fall back to the preset's Docker image when it
    /// is missing. The sensible default.
    #[default]
    Auto,
    /// Require the native host binary; fail if it is not installed.
    Native,
    /// Always run the preset's official Docker image.
    Docker,
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
        let has_cmd = !service.cmd.is_empty();
        let has_preset = service.preset.is_some();
        let has_image = service.image.is_some();
        match (has_cmd, has_preset, has_image) {
            (false, false, false) => bail!(
                "service `{}` must define exactly one of cmd, preset, or image",
                service.id
            ),
            (true, false, false) => validate_cmd("service", &service.id, &service.cmd)?,
            (false, true, false) => {}
            (false, false, true) => validate_image_service(service)?,
            _ => bail!(
                "service `{}` must define only one of cmd, preset, or image",
                service.id
            ),
        }
        if service.runtime != ServiceRuntime::Auto && !has_preset {
            bail!(
                "service `{}` sets `runtime` but only preset services accept it; \
                 image services are always Docker and cmd services are always native",
                service.id
            );
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
    for service in &cfg.services {
        for required in &service.requires {
            if !service_ids.contains(required.as_str()) {
                bail!(
                    "service `{}` requires unknown service `{}`",
                    service.id,
                    required
                );
            }
        }
    }
    for service in &cfg.services {
        let mut visiting = BTreeSet::new();
        let mut visited = BTreeSet::new();
        validate_service_dependency_cycle(cfg, &service.id, &mut visiting, &mut visited)?;
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

fn validate_service_dependency_cycle(
    cfg: &VatConfig,
    service_id: &str,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
) -> Result<()> {
    if visited.contains(service_id) {
        return Ok(());
    }
    if !visiting.insert(service_id.to_string()) {
        bail!("service dependency cycle includes `{service_id}`");
    }
    let service = cfg.service(service_id)?;
    for required in &service.requires {
        validate_service_dependency_cycle(cfg, required, visiting, visited)?;
    }
    visiting.remove(service_id);
    visited.insert(service_id.to_string());
    Ok(())
}

fn validate_id(kind: &str, id: &str) -> Result<()> {
    if id.trim().is_empty() {
        bail!("{kind} id must not be empty");
    }
    Ok(())
}

/// An `image`-backed service runs a Docker container, so it needs a non-empty
/// image reference and a container port to map onto the host.
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
fn validate_image_service(service: &ServiceConfig) -> Result<()> {
    if service
        .image
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        bail!("service `{}` image must not be empty", service.id);
    }
    if service.container_port.is_none() {
        bail!(
            "service `{}` image requires `container_port` (the port the service listens on inside the image)",
            service.id
        );
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
    fn parses_preset_service_with_seed_and_ready_cmd() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE_NAME);
        std::fs::write(
            &path,
            r#"
version = 1

[[services]]
id = "pg"
preset = "postgres"
seed = ["schema.sql", "data.sql"]
ready_cmd = ["sh", "-c", "psql -tAc 'select count(*) from docs' | grep -qE '^[0-9]+$'"]

[[services]]
id = "search"
preset = "opensearch"

[[runners]]
id = "ec"
requires = ["pg", "search"]
cmd = ["true"]
"#,
        )
        .unwrap();

        let cfg = load_file(&path).unwrap();
        let pg = cfg.service("pg").unwrap();
        assert_eq!(pg.preset, Some(ServicePreset::Postgres));
        assert_eq!(
            pg.seed,
            vec![PathBuf::from("schema.sql"), PathBuf::from("data.sql")]
        );
        assert_eq!(pg.ready_cmd.len(), 3);
        assert_eq!(
            cfg.service("search").unwrap().preset,
            Some(ServicePreset::Opensearch)
        );
    }

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
requires = ["db"]
cmd = ["sh", "-c", "sleep 1"]
ready_http = "http://127.0.0.1:1/"

[[services]]
id = "db"
cmd = ["sh", "-c", "sleep 1"]

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
        assert_eq!(cfg.service("web").unwrap().requires, vec!["db"]);
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

    #[test]
    fn rejects_unknown_required_service_dependency() {
        let cfg = VatConfig {
            version: 1,
            name: None,
            default_runner: None,
            workspace: WorkspaceConfig::default(),
            env: BTreeMap::new(),
            setup: Vec::new(),
            services: vec![ServiceConfig {
                id: "web".into(),
                requires: vec!["db".into()],
                cmd: vec!["true".into()],
                preset: None,
                image: None,
                container_port: None,
                image_env: BTreeMap::new(),
                runtime: ServiceRuntime::default(),
                version: None,
                port: PortSpec::default(),
                seed: Vec::new(),
                export: BTreeMap::new(),
                ready_http: None,
                ready_cmd: Vec::new(),
                timeout_s: default_service_timeout(),
            }],
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

    #[test]
    fn rejects_service_dependency_cycle() {
        let cfg = VatConfig {
            version: 1,
            name: None,
            default_runner: None,
            workspace: WorkspaceConfig::default(),
            env: BTreeMap::new(),
            setup: Vec::new(),
            services: vec![
                ServiceConfig {
                    id: "web".into(),
                    requires: vec!["api".into()],
                    cmd: vec!["true".into()],
                    preset: None,
                    image: None,
                    container_port: None,
                    image_env: BTreeMap::new(),
                    runtime: ServiceRuntime::default(),
                    version: None,
                    port: PortSpec::default(),
                    seed: Vec::new(),
                    export: BTreeMap::new(),
                    ready_http: None,
                    ready_cmd: Vec::new(),
                    timeout_s: default_service_timeout(),
                },
                ServiceConfig {
                    id: "api".into(),
                    requires: vec!["web".into()],
                    cmd: vec!["true".into()],
                    preset: None,
                    image: None,
                    container_port: None,
                    image_env: BTreeMap::new(),
                    runtime: ServiceRuntime::default(),
                    version: None,
                    port: PortSpec::default(),
                    seed: Vec::new(),
                    export: BTreeMap::new(),
                    ready_http: None,
                    ready_cmd: Vec::new(),
                    timeout_s: default_service_timeout(),
                },
            ],
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

    fn cfg_with_service(service: ServiceConfig) -> VatConfig {
        VatConfig {
            version: 1,
            name: None,
            default_runner: None,
            workspace: WorkspaceConfig::default(),
            env: BTreeMap::new(),
            setup: Vec::new(),
            services: vec![service],
            runners: vec![RunnerConfig {
                id: "e2e".into(),
                requires: vec!["svc".into()],
                cmd: vec!["true".into()],
                timeout_s: None,
                artifacts: Vec::new(),
            }],
            path: PathBuf::from("vat.toml"),
            root: PathBuf::from("."),
            digest: String::new(),
        }
    }

    fn bare_service(id: &str) -> ServiceConfig {
        ServiceConfig {
            id: id.into(),
            requires: Vec::new(),
            cmd: Vec::new(),
            preset: None,
            image: None,
            container_port: None,
            image_env: BTreeMap::new(),
            runtime: ServiceRuntime::default(),
            version: None,
            port: PortSpec::default(),
            seed: Vec::new(),
            export: BTreeMap::new(),
            ready_http: None,
            timeout_s: default_service_timeout(),
        }
    }

    #[test]
    fn accepts_image_backed_service() {
        let mut svc = bare_service("svc");
        svc.image = Some("postgres:16".into());
        svc.container_port = Some(5432);
        assert!(validate(&cfg_with_service(svc)).is_ok());
    }

    #[test]
    fn rejects_service_with_no_backing() {
        // Neither cmd, preset, nor image.
        assert!(validate(&cfg_with_service(bare_service("svc"))).is_err());
    }

    #[test]
    fn rejects_image_and_cmd_together() {
        let mut svc = bare_service("svc");
        svc.cmd = vec!["true".into()];
        svc.image = Some("redis:7".into());
        svc.container_port = Some(6379);
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn rejects_image_and_preset_together() {
        let mut svc = bare_service("svc");
        svc.preset = Some(ServicePreset::Postgres);
        svc.image = Some("postgres:16".into());
        svc.container_port = Some(5432);
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn rejects_image_without_container_port() {
        let mut svc = bare_service("svc");
        svc.image = Some("postgres:16".into());
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn rejects_runtime_on_non_preset_service() {
        // `runtime` only applies to preset services.
        let mut svc = bare_service("svc");
        svc.image = Some("postgres:16".into());
        svc.container_port = Some(5432);
        svc.runtime = ServiceRuntime::Docker;
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn accepts_preset_with_runtime() {
        let mut svc = bare_service("svc");
        svc.preset = Some(ServicePreset::Postgres);
        svc.runtime = ServiceRuntime::Docker;
        assert!(validate(&cfg_with_service(svc)).is_ok());
    }
}
// CODEGEN-END

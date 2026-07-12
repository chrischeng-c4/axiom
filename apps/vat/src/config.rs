// SPEC-MANAGED: apps/vat/tech-design/semantic/source/projects-vat-src-config-rs.md#rust-source-unit
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

use crate::spec::EgressPolicy;

/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
pub const FILE_NAME: &str = "vat.toml";

/// Parsed project-level vat contract.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scenarios: Vec<ScenarioConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkConfig>,

    #[serde(skip)]
    pub path: PathBuf,
    #[serde(skip)]
    pub root: PathBuf,
    #[serde(skip)]
    pub digest: String,
}

/// Transparent service routing for a run: known hosts the proxy should send to a
/// local emulator/mock instead of the real upstream.
/// @spec apps/vat/tech-design/logic/vat-network-sandbox-v1-transparent-http-service-routing-to-local.md#config
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routes: Vec<RouteConfig>,
    /// Outbound egress policy for the run (seatbelt-enforced). Default `open`.
    #[serde(default)]
    pub egress: EgressPolicy,
}

/// One host-routing rule: requests to `host` are served by `target` (a local base
/// URL) instead of being forwarded upstream.
/// @spec apps/vat/tech-design/logic/vat-network-sandbox-v1-transparent-http-service-routing-to-local.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub host: String,
    pub target: String,
}

/// Workspace defaults for one test run.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    #[serde(default = "default_dot")]
    pub base: PathBuf,
    #[serde(default = "default_dot")]
    pub workdir: PathBuf,
    #[serde(default)]
    pub keep: RetentionPolicy,
}

/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-config-rs.md#source
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
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum RetentionPolicy {
    #[default]
    Failed,
    Always,
    Never,
}

/// Setup command executed before services start.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStep {
    pub id: String,
    pub cmd: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
}

/// Run-scoped service required by a runner.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
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
    /// Declares this service as an ephemeral local Kubernetes cluster (kind /
    /// k3d / minikube). Mutually exclusive with cmd/preset/image. vat creates
    /// the cluster before the runner, exports KUBECONFIG into the runner, and
    /// deletes it at teardown subject to the workspace `keep` policy. `auto`
    /// resolves to the first installed backend whose Docker daemon is reachable.
    /// @spec apps/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#config
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster: Option<ClusterBackend>,
    /// Attach to a service already provisioned by the surrounding environment,
    /// such as a GitLab CI `services:` sidecar. vat waits for readiness,
    /// exports env, and records evidence, but never starts or stops it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external: Option<ExternalServiceConfig>,
    /// Optional Kubernetes version for the cluster node image (e.g. "1.30").
    /// Only meaningful with `cluster`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub k8s_version: Option<String>,
    /// Cluster node count (default 1). Only meaningful with `cluster`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nodes: Option<u32>,
    /// Path (relative to vat.toml) to an OpenAPI document. Required for the
    /// `openapi` preset, which serves spec-derived mock responses; rejected for
    /// every other backing.
    /// @spec apps/vat/tech-design/interfaces/rest/openapi-driven-mock-http-service.md#config
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec: Option<String>,
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

/// Endpoint for an externally managed service.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServiceConfig {
    pub host: String,
    pub port: u16,
}

/// Built-in local service presets.
///
/// The datastore/broker presets (postgres … mongo) prefer a native Homebrew
/// binary with a Docker image fallback. The emulator presets
/// (firestore … spanner) wrap the GCP `gcloud beta emulators` family — native
/// when the gcloud component is installed, Docker otherwise — and `firebase` is
/// the Firebase Emulator Suite bundle driven by a workspace `firebase.json`.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
/// @spec apps/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#config
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
    Firestore,
    Pubsub,
    Datastore,
    Bigtable,
    Spanner,
    Firebase,
    FirebaseAuth,
    CloudTasks,
    CloudScheduler,
    CloudWorkflows,
    CloudStorage,
    HttpMock,
    Openapi,
}

/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-config-rs.md#source
impl ServicePreset {
    /// Whether this preset is a GCP/Firebase emulator (vs a datastore/broker).
    /// @spec apps/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#config
    pub fn is_emulator(self) -> bool {
        matches!(
            self,
            ServicePreset::Firestore
                | ServicePreset::Pubsub
                | ServicePreset::Datastore
                | ServicePreset::Bigtable
                | ServicePreset::Spanner
                | ServicePreset::Firebase
                | ServicePreset::FirebaseAuth
                | ServicePreset::CloudTasks
                | ServicePreset::CloudScheduler
                | ServicePreset::CloudWorkflows
                | ServicePreset::CloudStorage
                | ServicePreset::HttpMock
                | ServicePreset::Openapi
        )
    }

    /// Whether vat ships a built-in Rust emulator for this preset. Built-in
    /// presets run vat's own in-process server under `runtime = auto`.
    /// @spec apps/vat/tech-design/logic/built-in-rust-emulators-pub-sub-firebase-auth.md#config
    pub fn is_builtin(self) -> bool {
        matches!(
            self,
            ServicePreset::Pubsub
                | ServicePreset::FirebaseAuth
                | ServicePreset::CloudTasks
                | ServicePreset::CloudScheduler
                | ServicePreset::CloudWorkflows
                | ServicePreset::CloudStorage
                | ServicePreset::HttpMock
                | ServicePreset::Openapi
        )
    }

    /// Built-in presets that have *only* the built-in path (no gcloud/Docker
    /// equivalent), so `runtime` must stay `auto`.
    /// @spec apps/vat/tech-design/logic/built-in-rust-emulators-pub-sub-firebase-auth.md#config
    pub fn is_builtin_only(self) -> bool {
        matches!(
            self,
            ServicePreset::FirebaseAuth
                | ServicePreset::CloudTasks
                | ServicePreset::CloudScheduler
                | ServicePreset::CloudWorkflows
                | ServicePreset::CloudStorage
                | ServicePreset::HttpMock
                | ServicePreset::Openapi
        )
    }

    /// The real GCP hostname this emulator preset stands in for, used to
    /// auto-derive a transparent host route (`real host -> local emulator`).
    /// `None` for presets that aren't a GCP service with a stable public host.
    /// @spec apps/vat/tech-design/logic/vat-network-sandbox-v1-transparent-http-service-routing-to-local.md#config
    pub fn preset_gcp_host(self) -> Option<&'static str> {
        match self {
            ServicePreset::CloudTasks => Some("cloudtasks.googleapis.com"),
            ServicePreset::CloudScheduler => Some("cloudscheduler.googleapis.com"),
            ServicePreset::Pubsub => Some("pubsub.googleapis.com"),
            ServicePreset::Firestore => Some("firestore.googleapis.com"),
            ServicePreset::CloudStorage => Some("storage.googleapis.com"),
            _ => None,
        }
    }
}

/// How a `preset` service is provided. The default prefers the native binary
/// (Homebrew) so the host GPU and zero-friction model hold, and only reaches
/// for Docker when the binary is absent — or when the preset has no native
/// equivalent on this host.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
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

/// Local Kubernetes cluster backend for a `cluster` service. `auto` (the
/// default when the field is present) prefers the first installed of kind,
/// then k3d, then minikube whose Docker daemon is reachable. All require Docker
/// on Apple Silicon.
/// @spec apps/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#config
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ClusterBackend {
    /// Prefer the first installed backend whose Docker daemon is reachable.
    #[default]
    Auto,
    /// kind — Kubernetes in Docker.
    Kind,
    /// k3d — k3s in Docker.
    K3d,
    /// minikube with the docker driver.
    Minikube,
}

/// Port policy for a service. Presets default to `auto` to avoid conflicts.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PortSpec {
    Auto(String),
    Fixed(u16),
}

/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
impl Default for PortSpec {
    fn default() -> Self {
        PortSpec::Auto("auto".to_string())
    }
}

/// Why `vat run` selected a runner.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerSelectionReason {
    Explicit,
    DefaultRunner,
    SingleRunner,
}

fn default_service_timeout() -> u64 {
    60
}

/// Named production-like integration scenario. A scenario promotes an app
/// service plus its dependency set to a first-class runner target while reusing
/// the existing service lifecycle.
/// @spec apps/vat/tech-design/logic/production-like-integration-scenarios.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioConfig {
    pub id: String,
    pub app: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    pub runner: String,
    #[serde(default)]
    pub network: ScenarioNetworkMode,
}

/// Scenario-scoped network safety mode.
/// @spec apps/vat/tech-design/logic/production-like-integration-scenarios.md#schema
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScenarioNetworkMode {
    /// Use the project/run network policy as configured.
    #[default]
    Open,
    /// Require http-mock participation and no-forward proxy behavior.
    Hermetic,
}

/// Named runner an agent can invoke via `vat run <id>`.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
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
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
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
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
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
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
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
        let has_cluster = service.cluster.is_some();
        let has_external = service.external.is_some();
        let backing = [has_cmd, has_preset, has_image, has_cluster, has_external]
            .into_iter()
            .filter(|b| *b)
            .count();
        match backing {
            0 => bail!(
                "service `{}` must define exactly one of cmd, preset, image, cluster, or external",
                service.id
            ),
            1 => {
                if has_cmd {
                    validate_cmd("service", &service.id, &service.cmd)?;
                } else if has_image {
                    validate_image_service(service)?;
                } else if has_cluster {
                    validate_cluster_service(service)?;
                } else if has_external {
                    validate_external_service(service)?;
                } else if service.preset == Some(ServicePreset::Firebase) {
                    validate_firebase_service(cfg, service)?;
                } else if service.preset == Some(ServicePreset::Openapi) {
                    validate_openapi_service(service)?;
                }
                // other presets: no extra checks here.
            }
            _ => bail!(
                "service `{}` must define only one of cmd, preset, image, cluster, or external",
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
        if let Some(preset) = service.preset {
            if preset.is_builtin_only() && service.runtime != ServiceRuntime::Auto {
                bail!(
                    "service `{}` preset `{preset:?}` only has vat's built-in emulator; \
                     leave `runtime` at the default `auto`",
                    service.id
                );
            }
        }
        if service.spec.is_some() && service.preset != Some(ServicePreset::Openapi) {
            bail!(
                "service `{}` sets `spec` but only the `openapi` preset accepts it",
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
    let mut scenario_ids = BTreeSet::new();
    for scenario in &cfg.scenarios {
        validate_id("scenario", &scenario.id)?;
        if !scenario_ids.insert(scenario.id.as_str()) {
            bail!("duplicate scenario id `{}`", scenario.id);
        }
        if !service_ids.contains(scenario.app.as_str()) {
            bail!(
                "scenario `{}` app references unknown service `{}`",
                scenario.id,
                scenario.app
            );
        }
        if !runner_ids.contains(scenario.runner.as_str()) {
            bail!(
                "scenario `{}` runner references unknown runner `{}`",
                scenario.id,
                scenario.runner
            );
        }
        for required in &scenario.requires {
            if !service_ids.contains(required.as_str()) {
                bail!(
                    "scenario `{}` requires unknown service `{}`",
                    scenario.id,
                    required
                );
            }
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

/// An `external` service is owned by CI/local infrastructure. vat only attaches
/// to the endpoint, so Docker/cluster/service-start knobs do not apply.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
fn validate_external_service(service: &ServiceConfig) -> Result<()> {
    let endpoint = service
        .external
        .as_ref()
        .context("external service missing endpoint")?;
    if endpoint.host.trim().is_empty() {
        bail!("service `{}` external host must not be empty", service.id);
    }
    if endpoint.port == 0 {
        bail!(
            "service `{}` external port must be greater than 0",
            service.id
        );
    }
    if service.container_port.is_some()
        || !service.image_env.is_empty()
        || !service.seed.is_empty()
        || matches!(service.port, PortSpec::Fixed(_))
    {
        bail!(
            "service `{}` external does not accept port, container_port, image_env, or seed",
            service.id
        );
    }
    Ok(())
}

/// An `image`-backed service runs a Docker container, so it needs a non-empty
/// image reference and a container port to map onto the host.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#config
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

/// A `cluster` service spins up an ephemeral local Kubernetes cluster, so it
/// rejects the container/preset-only knobs and bounds the node count.
/// @spec apps/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#config
fn validate_cluster_service(service: &ServiceConfig) -> Result<()> {
    if service.container_port.is_some() || !service.image_env.is_empty() || !service.seed.is_empty()
    {
        bail!(
            "service `{}` cluster does not accept container_port, image_env, or seed",
            service.id
        );
    }
    if let Some(nodes) = service.nodes {
        if !(1..=9).contains(&nodes) {
            bail!(
                "service `{}` cluster nodes must be between 1 and 9",
                service.id
            );
        }
    }
    Ok(())
}

/// The `openapi` preset serves spec-derived mock responses, so it requires a
/// `spec` pointing at an OpenAPI document.
/// @spec apps/vat/tech-design/interfaces/rest/openapi-driven-mock-http-service.md#config
fn validate_openapi_service(service: &ServiceConfig) -> Result<()> {
    if service
        .spec
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        bail!(
            "service `{}` preset `openapi` must set `spec` to an OpenAPI document path",
            service.id
        );
    }
    Ok(())
}

/// The `firebase` preset is a bundle driven by the Firebase Emulator Suite, so
/// it requires a `firebase.json` in the workspace to know which emulators and
/// ports to start.
/// @spec apps/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#config
fn validate_firebase_service(cfg: &VatConfig, service: &ServiceConfig) -> Result<()> {
    if !cfg.root.join("firebase.json").exists() {
        bail!(
            "service `{}` preset `firebase` requires a firebase.json in the workspace",
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

/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-config-rs.md#source
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

    pub fn scenario(&self, id: &str) -> Result<&ScenarioConfig> {
        self.scenarios
            .iter()
            .find(|s| s.id == id)
            .with_context(|| format!("scenario `{id}` not found in {}", self.path.display()))
    }

    pub fn base_dir(&self) -> PathBuf {
        resolve_relative(&self.root, &self.workspace.base)
    }
}

/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-config-rs.md#source
pub fn resolve_relative(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-config-rs.md#source
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
    fn parses_scenario_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE_NAME);
        std::fs::write(
            &path,
            r#"
version = 1

[[services]]
id = "api"
cmd = ["true"]

[[services]]
id = "pg"
cmd = ["true"]

[[runners]]
id = "e2e"
requires = ["pg"]
cmd = ["true"]

[[scenarios]]
id = "prod-like"
app = "api"
requires = ["pg"]
runner = "e2e"
network = "hermetic"
"#,
        )
        .unwrap();

        let cfg = load_file(&path).unwrap();
        let scenario = cfg.scenario("prod-like").unwrap();
        assert_eq!(scenario.app, "api");
        assert_eq!(scenario.requires, vec!["pg"]);
        assert_eq!(scenario.runner, "e2e");
        assert_eq!(scenario.network, ScenarioNetworkMode::Hermetic);
    }

    #[test]
    fn rejects_scenario_unknown_references() {
        let cfg = VatConfig {
            version: 1,
            network: None,
            name: None,
            default_runner: None,
            workspace: WorkspaceConfig::default(),
            env: BTreeMap::new(),
            setup: Vec::new(),
            services: {
                let mut service = bare_service("api");
                service.cmd = vec!["true".into()];
                vec![service]
            },
            runners: vec![RunnerConfig {
                id: "e2e".into(),
                requires: Vec::new(),
                cmd: vec!["true".into()],
                timeout_s: None,
                artifacts: Vec::new(),
            }],
            scenarios: vec![ScenarioConfig {
                id: "bad".into(),
                app: "missing".into(),
                requires: Vec::new(),
                runner: "e2e".into(),
                network: ScenarioNetworkMode::Open,
            }],
            path: PathBuf::from("vat.toml"),
            root: PathBuf::from("."),
            digest: String::new(),
        };
        assert!(validate(&cfg).is_err());
    }

    #[test]
    fn rejects_unknown_required_service() {
        let cfg = VatConfig {
            version: 1,
            network: None,
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
            scenarios: Vec::new(),
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
            network: None,
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
                cluster: None,
                external: None,
                k8s_version: None,
                nodes: None,
                spec: None,
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
            scenarios: Vec::new(),
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
            network: None,
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
                    cluster: None,
                    external: None,
                    k8s_version: None,
                    nodes: None,
                    spec: None,
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
                    cluster: None,
                    external: None,
                    k8s_version: None,
                    nodes: None,
                    spec: None,
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
            scenarios: Vec::new(),
            path: PathBuf::from("vat.toml"),
            root: PathBuf::from("."),
            digest: String::new(),
        };
        assert!(validate(&cfg).is_err());
    }

    fn cfg_with_service(service: ServiceConfig) -> VatConfig {
        VatConfig {
            version: 1,
            network: None,
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
            scenarios: Vec::new(),
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
            cluster: None,
            external: None,
            k8s_version: None,
            nodes: None,
            spec: None,
            version: None,
            port: PortSpec::default(),
            seed: Vec::new(),
            export: BTreeMap::new(),
            ready_http: None,
            ready_cmd: Vec::new(),
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

    #[test]
    fn accepts_cluster_service() {
        let mut svc = bare_service("svc");
        svc.cluster = Some(ClusterBackend::Auto);
        assert!(validate(&cfg_with_service(svc)).is_ok());
    }

    #[test]
    fn accepts_external_service() {
        let mut svc = bare_service("svc");
        svc.external = Some(ExternalServiceConfig {
            host: "postgres".into(),
            port: 5432,
        });
        assert!(validate(&cfg_with_service(svc)).is_ok());
    }

    #[test]
    fn rejects_external_and_cmd_together() {
        let mut svc = bare_service("svc");
        svc.external = Some(ExternalServiceConfig {
            host: "postgres".into(),
            port: 5432,
        });
        svc.cmd = vec!["true".into()];
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn rejects_external_empty_host() {
        let mut svc = bare_service("svc");
        svc.external = Some(ExternalServiceConfig {
            host: " ".into(),
            port: 5432,
        });
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn rejects_external_zero_port() {
        let mut svc = bare_service("svc");
        svc.external = Some(ExternalServiceConfig {
            host: "postgres".into(),
            port: 0,
        });
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn rejects_external_with_service_start_knobs() {
        let mut svc = bare_service("svc");
        svc.external = Some(ExternalServiceConfig {
            host: "postgres".into(),
            port: 5432,
        });
        svc.port = PortSpec::Fixed(15432);
        assert!(validate(&cfg_with_service(svc.clone())).is_err());

        svc.port = PortSpec::default();
        svc.container_port = Some(5432);
        assert!(validate(&cfg_with_service(svc.clone())).is_err());

        svc.container_port = None;
        svc.image_env
            .insert("POSTGRES_PASSWORD".into(), "pw".into());
        assert!(validate(&cfg_with_service(svc.clone())).is_err());

        svc.image_env.clear();
        svc.seed = vec![PathBuf::from("schema.sql")];
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn rejects_cluster_and_cmd_together() {
        let mut svc = bare_service("svc");
        svc.cluster = Some(ClusterBackend::Kind);
        svc.cmd = vec!["true".into()];
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn rejects_cluster_and_preset_together() {
        let mut svc = bare_service("svc");
        svc.cluster = Some(ClusterBackend::Kind);
        svc.preset = Some(ServicePreset::Postgres);
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn rejects_cluster_and_image_together() {
        let mut svc = bare_service("svc");
        svc.cluster = Some(ClusterBackend::Kind);
        svc.image = Some("postgres:16".into());
        svc.container_port = Some(5432);
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn rejects_cluster_with_container_port() {
        let mut svc = bare_service("svc");
        svc.cluster = Some(ClusterBackend::Auto);
        svc.container_port = Some(6443);
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn rejects_cluster_with_seed() {
        let mut svc = bare_service("svc");
        svc.cluster = Some(ClusterBackend::Auto);
        svc.seed = vec![PathBuf::from("schema.sql")];
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn rejects_cluster_nodes_zero() {
        let mut svc = bare_service("svc");
        svc.cluster = Some(ClusterBackend::Auto);
        svc.nodes = Some(0);
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn rejects_cluster_nodes_too_many() {
        let mut svc = bare_service("svc");
        svc.cluster = Some(ClusterBackend::Auto);
        svc.nodes = Some(10);
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn cluster_backend_parses_k3d_kebab() {
        // serde round-trips the backend tokens used in vat.toml / --backend.
        for (token, backend) in [
            ("auto", ClusterBackend::Auto),
            ("kind", ClusterBackend::Kind),
            ("k3d", ClusterBackend::K3d),
            ("minikube", ClusterBackend::Minikube),
        ] {
            let parsed: ClusterBackend =
                serde_json::from_value(serde_json::Value::String(token.into())).unwrap();
            assert_eq!(parsed, backend);
            let dumped = serde_json::to_value(backend).unwrap();
            assert_eq!(dumped, serde_json::Value::String(token.into()));
        }
    }

    #[test]
    fn accepts_gcp_emulator_presets() {
        for preset in [
            ServicePreset::Firestore,
            ServicePreset::Pubsub,
            ServicePreset::Datastore,
            ServicePreset::Bigtable,
            ServicePreset::Spanner,
        ] {
            let mut svc = bare_service("svc");
            svc.preset = Some(preset);
            assert!(validate(&cfg_with_service(svc)).is_ok(), "{preset:?}");
        }
    }

    #[test]
    fn rejects_firebase_without_firebase_json() {
        // cfg_with_service roots at "." (the crate dir), which has no firebase.json.
        let mut svc = bare_service("svc");
        svc.preset = Some(ServicePreset::Firebase);
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn emulator_presets_round_trip() {
        for (token, preset) in [
            ("firestore", ServicePreset::Firestore),
            ("pubsub", ServicePreset::Pubsub),
            ("datastore", ServicePreset::Datastore),
            ("bigtable", ServicePreset::Bigtable),
            ("spanner", ServicePreset::Spanner),
            ("firebase", ServicePreset::Firebase),
        ] {
            let parsed: ServicePreset =
                serde_json::from_value(serde_json::Value::String(token.into())).unwrap();
            assert_eq!(parsed, preset);
            assert_eq!(
                serde_json::to_value(preset).unwrap(),
                serde_json::Value::String(token.into())
            );
            assert!(preset.is_emulator());
        }
    }

    #[test]
    fn accepts_firebase_auth_preset_and_classifies_builtin() {
        let parsed: ServicePreset =
            serde_json::from_value(serde_json::Value::String("firebase-auth".into())).unwrap();
        assert_eq!(parsed, ServicePreset::FirebaseAuth);
        assert!(ServicePreset::FirebaseAuth.is_builtin());
        assert!(ServicePreset::FirebaseAuth.is_builtin_only());
        assert!(ServicePreset::Pubsub.is_builtin());
        assert!(!ServicePreset::Pubsub.is_builtin_only());
        assert!(!ServicePreset::Firestore.is_builtin());

        let mut svc = bare_service("svc");
        svc.preset = Some(ServicePreset::FirebaseAuth);
        assert!(validate(&cfg_with_service(svc)).is_ok());
    }

    #[test]
    fn rejects_firebase_auth_with_explicit_runtime() {
        let mut svc = bare_service("svc");
        svc.preset = Some(ServicePreset::FirebaseAuth);
        svc.runtime = ServiceRuntime::Docker;
        assert!(validate(&cfg_with_service(svc)).is_err());
    }

    #[test]
    fn accepts_cloud_tasks_and_scheduler_builtin_presets() {
        for (token, preset) in [
            ("cloud-tasks", ServicePreset::CloudTasks),
            ("cloud-scheduler", ServicePreset::CloudScheduler),
            ("cloud-workflows", ServicePreset::CloudWorkflows),
            ("cloud-storage", ServicePreset::CloudStorage),
            ("http-mock", ServicePreset::HttpMock),
        ] {
            let parsed: ServicePreset =
                serde_json::from_value(serde_json::Value::String(token.into())).unwrap();
            assert_eq!(parsed, preset);
            assert!(preset.is_builtin());
            assert!(preset.is_builtin_only());
            let mut svc = bare_service("svc");
            svc.preset = Some(preset);
            assert!(validate(&cfg_with_service(svc)).is_ok());
        }
    }

    #[test]
    fn rejects_cloud_preset_with_explicit_runtime() {
        for preset in [
            ServicePreset::CloudTasks,
            ServicePreset::CloudScheduler,
            ServicePreset::CloudWorkflows,
            ServicePreset::CloudStorage,
            ServicePreset::HttpMock,
        ] {
            let mut svc = bare_service("svc");
            svc.preset = Some(preset);
            svc.runtime = ServiceRuntime::Docker;
            assert!(validate(&cfg_with_service(svc)).is_err());
        }
    }

    #[test]
    fn openapi_preset_classifies_builtin_and_requires_spec() {
        let parsed: ServicePreset =
            serde_json::from_value(serde_json::Value::String("openapi".into())).unwrap();
        assert_eq!(parsed, ServicePreset::Openapi);
        assert!(ServicePreset::Openapi.is_emulator());
        assert!(ServicePreset::Openapi.is_builtin());
        assert!(ServicePreset::Openapi.is_builtin_only());

        // openapi without a `spec` → rejected.
        let mut svc = bare_service("svc");
        svc.preset = Some(ServicePreset::Openapi);
        assert!(validate(&cfg_with_service(svc.clone())).is_err());

        // openapi with a `spec` → ok.
        svc.spec = Some("api.yaml".into());
        assert!(validate(&cfg_with_service(svc.clone())).is_ok());

        // a built-in-only preset must keep runtime = auto, even with a spec.
        let mut docker = svc.clone();
        docker.runtime = ServiceRuntime::Docker;
        assert!(validate(&cfg_with_service(docker)).is_err());

        // `spec` on a non-openapi preset → rejected.
        let mut other = bare_service("svc");
        other.preset = Some(ServicePreset::Pubsub);
        other.spec = Some("api.yaml".into());
        assert!(validate(&cfg_with_service(other)).is_err());
    }

    #[test]
    fn preset_gcp_host_map() {
        assert_eq!(
            ServicePreset::CloudTasks.preset_gcp_host(),
            Some("cloudtasks.googleapis.com")
        );
        assert_eq!(
            ServicePreset::CloudScheduler.preset_gcp_host(),
            Some("cloudscheduler.googleapis.com")
        );
        assert_eq!(
            ServicePreset::Pubsub.preset_gcp_host(),
            Some("pubsub.googleapis.com")
        );
        // Non-GCP / no stable public host → None.
        assert_eq!(ServicePreset::Postgres.preset_gcp_host(), None);
        assert_eq!(ServicePreset::HttpMock.preset_gcp_host(), None);
    }
}
// CODEGEN-END
// SPEC-MANAGED: apps/vat/tech-design/logic/vat-microvm-phase-3-vat-compose-limited-compose-subset-up-down-p.md#schema
// CODEGEN-BEGIN

/// @spec apps/vat/tech-design/logic/vat-microvm-phase-3-vat-compose-limited-compose-subset-up-down-p.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VatMicroVmPhase3VatComposeDataModelAdditions {
    /// New apps/vat/src/compose.rs struct ComposeFile (Deserialize via serde_yaml):
    /// services: BTreeMap<String, ComposeService>, volumes: BTreeMap<String, ComposeVolume>
    /// (named-volume declarations only -- driver: local or unspecified; any other
    /// driver/driver_opts form is a hard reject per R3), version: serde(default) String
    /// (parsed but ignored, matching Docker Compose's own deprecation of the version key).
    #[serde(default)]
    pub compose_file_struct: Option<serde_json::Value>,
    /// New apps/vat/src/compose.rs struct ComposeService (Deserialize): image: Option<String>,
    /// build: Option<ComposeBuild> (enum Short(String) or Full{context: String,
    /// dockerfile: Option<String>, args: Option<ComposeEnv>}), ports: Vec<String> (raw
    /// H:C or bare C strings, parsed by expand()), environment: Option<ComposeEnv>
    /// (enum List(Vec<String>) or Map(BTreeMap<String, Option<String>>) -- a Map entry
    /// whose value is None, or a List entry with no `=`, is R3's bare-key-no-value hard
    /// reject), depends_on: Option<ComposeDependsOn> (enum List(Vec<String>) or
    /// Map(BTreeMap<String, ComposeDependsOnEntry{condition: Option<String>}>) --
    /// condition equal to service_healthy is R3's hard reject, every other form or
    /// absence is pure start-ordering), volumes: Vec<String> (named-volume:path entries
    /// only; any entry whose host segment before the colon looks like a filesystem path
    /// is bind-mount form and a hard reject per R3), plus #[serde(flatten)]
    /// extra: BTreeMap<String, serde_yaml::Value> capturing every other key so parse()
    /// can detect x- prefixed keys (ignored) vs. hard-reject keys (deploy, secrets,
    /// configs, extends, networks, profiles, healthcheck, command, entrypoint) by
    /// iterating extra's keys.
    #[serde(default)]
    pub compose_service_struct: Option<serde_json::Value>,
    /// New apps/vat/src/compose.rs fn parse(path: &Path) -> Result<ComposeFile>. Reads
    /// the file, deserializes via serde_yaml (already an unconditional dependency as of
    /// Phase 2), then walks every top-level key (services, volumes, version, x- prefixed
    /// keys allowed, everything else hard-reject) and every per-service extra key (x-
    /// prefixed ignored, R3's list hard-reject) -- producing this exact error text:
    /// compose file {file} service {id} uses unsupported key {key} -- {reason}; remove
    /// it or edit the generated vat.toml directly after vat compose import. Top-level
    /// (non-service) hard-reject keys use {file} with no service segment.
    #[serde(default)]
    pub compose_parse_fn: Option<serde_json::Value>,
    /// New apps/vat/src/compose.rs fn expand(file: &ComposeFile, project: &str) ->
    /// Result<Vec<ServiceConfig>> (R1). For each ComposeService: resolves build: by
    /// calling commands::build::build_image() in-process (R6), writing the returned
    /// BuildReport.tag into ServiceConfig.image (so image: and build: entries converge
    /// to one uniform image-shaped ServiceConfig); parses ports (H:C becomes
    /// PortSpec::Fixed(H) plus container_port C, bare C becomes PortSpec::Auto plus
    /// container_port C); flattens environment into ServiceConfig.image_env; maps
    /// depends_on 1:1 onto ServiceConfig.requires (service_healthy already rejected by
    /// parse()); maps named volumes entries onto the new ServiceConfig.volumes:
    /// Vec<VolumeMount>; and sets every ServiceConfig.runtime from the project-wide
    /// --runtime selection (R4/R8). Prints one non-fatal warning per service with a
    /// non-empty depends_on, naming the no-bridge-network-DNS caveat (R3).
    #[serde(default)]
    pub compose_expand_fn: Option<serde_json::Value>,
    /// New apps/vat/src/compose.rs fn materialize(services: &[ServiceConfig],
    /// out: &Path) -> Result<()> (R1). Builds a VatConfig with cfg.services set to the
    /// given slice and exactly one synthesized RunnerConfig (id: project.up, cmd: sleep
    /// infinity, requires: every service's id, in the same order expand() produced
    /// them), serializes via toml, and writes it to out (the project directory's
    /// vat.toml) -- satisfying VatConfig::validate()'s existing at-least-one-runner
    /// requirement with no change to that function's control flow.
    #[serde(default)]
    pub compose_materialize_fn: Option<serde_json::Value>,
    /// apps/vat/src/config.rs: ServiceRuntime gains a MicroVm variant (Auto, Native,
    /// Docker, MicroVm), still deriving clap::ValueEnum -- the same enum vat.toml's
    /// [[services]].runtime key already parses is reused verbatim for the new
    /// CLI-facing vat compose --runtime auto|docker|microvm flag (R4/R8), so no new
    /// parsing surface is introduced.
    #[serde(default)]
    pub service_runtime_microvm_variant: Option<serde_json::Value>,
    /// apps/vat/src/config.rs: ServiceConfig gains a
    /// #[serde(default, skip_serializing_if = "Vec::is_empty")] pub volumes:
    /// Vec<VolumeMount> field and a new struct VolumeMount { pub name: String,
    /// pub path: String } (named-volume-to-container-path pairs from a compose
    /// service's volumes: list, R2/R4). Applied as -v name:path argv entries by both
    /// docker_run_command and the new container_run_command when non-empty; empty on
    /// every non-compose vat.toml, so this is additive with zero effect on existing
    /// services.
    #[serde(default)]
    pub service_config_volumes_field: Option<serde_json::Value>,
    /// apps/vat/src/config.rs validate(): the existing gate that bails when
    /// service.runtime is not Auto and there is no preset becomes a gate that also
    /// allows an image-backed service (R4) -- an image-backed ServiceConfig may now
    /// declare runtime: docker or runtime: microvm explicitly (cmd services remain
    /// always-native and still bail); the error text is updated to name image services
    /// as an accepted case alongside preset services.
    #[serde(default)]
    pub validate_runtime_gate_relaxation: Option<serde_json::Value>,
    /// apps/vat/src/commands/run.rs: prepare_service's image branch (the
    /// else-if-let-Some(image) arm), which today calls prepare_image_service
    /// unconditionally, gains a match on service.runtime: ServiceRuntime::MicroVm
    /// calls the new prepare_microvm_service(vat, service, image); every other value
    /// (Auto, Docker, Native) keeps calling prepare_image_service(vat, service, image)
    /// unchanged -- so the default (auto, and today's implicit Docker-only behavior) is
    /// bit-for-bit identical to pre-Phase-3 behavior (R4/R5).
    #[serde(default)]
    pub prepare_service_dispatch_update: Option<serde_json::Value>,
    /// New apps/vat/src/commands/run.rs fn prepare_microvm_service(vat: &store::Vat,
    /// service: &ServiceConfig, image: &str) -> Result<ServicePlan>, structurally
    /// mirroring prepare_image_service line for line: calls the new private
    /// ensure_microvm_available() (R5) instead of ensure_docker_available(), builds its
    /// argv via the new container_run_command() instead of docker_run_command(), and
    /// returns a ServicePlan with the new microvm_name: Some(name) field set
    /// (docker_name stays None) so teardown removes the right container kind.
    #[serde(default)]
    pub prepare_microvm_service_fn: Option<serde_json::Value>,
    /// New private apps/vat/src/commands/run.rs fn ensure_microvm_available() ->
    /// Result<()>, structurally mirroring the existing ensure_docker_available and
    /// commands/build.rs's private fn of the same name (not reusable across files
    /// since it is not pub there): requires sandbox::microvm::available() (container
    /// binary on PATH) and, if sandbox::microvm::system_up() is not immediately true,
    /// waits via sandbox::microvm::ensure_system_started(bounded timeout) before
    /// failing with a structured container_unavailable emit_jsonl error, mirroring
    /// ensure_docker_available's docker_unavailable shape.
    #[serde(default)]
    pub run_ensure_microvm_available_fn: Option<serde_json::Value>,
    /// New apps/vat/src/commands/run.rs fn container_run_command(name: &str,
    /// image: &str, host_port: u16, container_port: u16,
    /// container_env: &BTreeMap<String, String>, volumes: &[VolumeMount]) ->
    /// Vec<String>, structurally mirroring docker_run_command: container, run, --rm,
    /// --name, name, -p, 127.0.0.1:host_port:container_port, then -v name:path per
    /// volumes entry, then -e key=value per sorted container_env entry, then image.
    /// Env and volumes both iterate in deterministic (sorted-key / input-slice) order,
    /// matching docker_run_command's existing determinism guarantee (the
    /// container_run_command_shape unit test asserts this exact argv).
    #[serde(default)]
    pub container_run_command_fn: Option<serde_json::Value>,
    /// apps/vat/src/commands/run.rs: ServicePlan and ServiceHandle both gain
    /// microvm_name: Option<String> (R5), set only by prepare_microvm_service --
    /// parallel to the existing docker_name: Option<String> field both structs already
    /// carry for the Docker path; start_service copies plan.microvm_name into the
    /// handle exactly the way it already copies plan.docker_name.
    #[serde(default)]
    pub service_plan_handle_microvm_name: Option<serde_json::Value>,
    /// apps/vat/src/commands/run.rs: stop_services() gains a microvm_name branch
    /// alongside the existing docker_name branch that shells out to docker rm -f --
    /// the new branch shells out to container rm -f name, force-removing the
    /// container regardless of how the container run child fared, identical semantics
    /// to the Docker branch (R5).
    #[serde(default)]
    pub stop_services_microvm_teardown: Option<serde_json::Value>,
    /// apps/vat/src/state.rs: RunnerRunRecord gains a
    /// #[serde(default, skip_serializing_if = "Option::is_none")] pub pid: Option<u32>
    /// field (R7) -- the same optional, backward-compatible shape ServiceRunRecord.pid
    /// already uses; legacy metadata without this field deserializes with pid: None.
    #[serde(default)]
    pub runner_run_record_pid_field: Option<serde_json::Value>,
    /// apps/vat/src/commands/run.rs run_configured(): immediately after the
    /// runner-spawn loop and strictly before the blocking wait_runner_processes(procs)
    /// call, an interim Vec<RunnerRunRecord> is built from procs (status:
    /// ProcessStatus::Running, pid: Some(proc.child.id()), exit_code: None,
    /// duration_ms: None, command/stdout_log/stderr_log copied from each RunnerProc)
    /// and written into vat.meta.test_run.runner and test_run.runners, followed by
    /// vat.save() -- mirroring persist_services()'s existing early-write pattern for
    /// services. Required by R9: without this, test_run.runner.pid is only ever
    /// populated after wait_runner_processes returns (i.e. after the runner has
    /// already exited), which would make vat compose down's SIGTERM-while-running
    /// path impossible.
    #[serde(default)]
    pub runner_early_persist_behavior: Option<serde_json::Value>,
    /// New apps/vat/src/commands/compose.rs struct ComposeRecord
    /// (Serialize/Deserialize, mirrors commands/cluster.rs's ClusterRecord):
    /// project: String, vat_id: Option<String>, service_ids: Vec<String>,
    /// status: String (starting, started, or running), created_at: String (RFC3339).
    /// Persisted at root/compose/project/project.json, where root is
    /// paths::root()? (R10) -- computed inline in commands/compose.rs rather than
    /// adding a helper to paths.rs, since paths.rs is not in this WI's file scope.
    #[serde(default)]
    pub compose_record_struct: Option<serde_json::Value>,
    /// apps/vat/src/cli.rs gains Cmd::Compose { cmd: ComposeCmd } and
    /// enum ComposeCmd { Import { file: PathBuf, project: Option<String>,
    /// runtime: ServiceRuntime }, Up { project: Option<String>, detach: bool },
    /// Down { project: String }, Ps { project: String }, Logs { project: String,
    /// service: String } } (R8). Import/Up's project defaults to the compose file's
    /// parent directory basename (sanitized the same way container_name() sanitizes)
    /// when omitted; Down/Ps/Logs require an explicit project naming an
    /// already-imported or running project (no other file to default from). runtime
    /// defaults to auto via ServiceRuntime's existing #[default], applied
    /// project-wide at import/up time, never per-service.
    #[serde(default)]
    pub compose_cmd_cli_variant: Option<serde_json::Value>,
}
// CODEGEN-END

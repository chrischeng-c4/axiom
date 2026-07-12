// SPEC-MANAGED: apps/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#rust-source-unit
// CODEGEN-BEGIN
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
/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
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
/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
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
/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
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
    /// Opaque upstream execution plan attached with `vat run --plan`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan: Option<PlanEvidence>,
}

/// vat.toml config reference captured for one runner invocation.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRef {
    pub path: String,
    pub digest: String,
}

/// Captured state of a local Kubernetes cluster backing a `cluster` service.
/// @spec apps/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterRunRecord {
    /// Backend that provisioned the cluster: "kind", "k3d", or "minikube".
    pub backend: String,
    /// Cluster name as known to the backend.
    pub name: String,
    /// Path to the isolated kubeconfig exported to the runner.
    pub kubeconfig: String,
    /// Number of nodes requested for the cluster.
    pub node_count: u32,
    /// Time from create to first readiness, when measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready_ms: Option<u64>,
}

/// Captured service state for one run-scoped dependency process.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRunRecord {
    pub id: String,
    pub command: Vec<String>,
    pub status: ProcessStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owned_by_vat: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepare_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepare_duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready_duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exported_env: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready_http: Option<String>,
    /// Present when this service is a local Kubernetes cluster.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster: Option<ClusterRunRecord>,
    pub stdout_log: String,
    pub stderr_log: String,
}

/// Captured runner process state.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
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

/// Route visible in a scenario topology report.
/// @spec apps/vat/tech-design/logic/production-like-integration-scenarios.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRecord {
    pub host: String,
    pub target: String,
    pub source: String,
}

/// Captured scenario topology for a production-like integration run.
/// @spec apps/vat/tech-design/logic/production-like-integration-scenarios.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioRunRecord {
    pub id: String,
    pub app: String,
    pub runner: String,
    pub network: String,
    pub services: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routes: Vec<RouteRecord>,
    pub hermetic: bool,
}

/// Process status used inside test-run evidence.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
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
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

/// Opaque upstream plan file attached to a run.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanEvidence {
    pub source_path: String,
    pub rootfs_path: String,
    pub digest: String,
}

/// Topology selected for one configured run.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyEvidence {
    pub runners: Vec<String>,
    pub services: Vec<String>,
    pub network: String,
    pub hermetic: bool,
}

/// Complete evidence bundle for one vat.toml runner invocation.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunEvidence {
    pub config: ConfigRef,
    pub runner_id: String,
    pub retention: RetentionPolicy,
    pub services: Vec<ServiceRunRecord>,
    /// Scenario topology for `vat run --scenario`; absent for existing runner
    /// modes and old metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario: Option<ScenarioRunRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner: Option<RunnerRunRecord>,
    /// Every runner of a concurrent `vat run a b ...` set; `runner` keeps the
    /// first record for backward compatibility. Empty on legacy metadata.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runners: Vec<RunnerRunRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<ArtifactRecord>,
    /// Opaque upstream execution plan attached with `vat run --plan`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan: Option<PlanEvidence>,
    /// Runner/scenario topology selected before execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topology: Option<TopologyEvidence>,
}

/// Filesystem changes vs. the base manifest. Full lists; the projection
/// samples them for compactness.
/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChangeSet {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
}

/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
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
/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
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
/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub rootfs: String,
    pub file_count: usize,
    pub size_bytes: u64,
}

/// The full, agent-legible projection of a vat. This is what `vat state`
/// prints and what an agent should read to understand the environment.
/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<PlanEvidence>,
    pub workspace: WorkspaceInfo,
    pub changes: ChangeSummary,
    /// The GPU this vat can reach — the headline contrast with Docker-in-VM.
    pub gpu: GpuInfo,
    pub events_tail: Vec<Event>,
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

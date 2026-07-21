// HANDWRITE-BEGIN gap="missing-generator:cli:compose-lifecycle-orchestration" tracker="#1484" reason="R8-R10 plus #1526/#1529: Cmd dispatch for import/up/down/ps/logs, the locked ComposeRecord registry at <root>/compose/<project>/project.json, atomic import publication/rollback with a fail-closed imported-record service-id gate, up's foreground poll-thread-plus-in-process-run vs. --detach re-exec-plus-poll divergence, and down's VAT-owned stop-request acknowledgement. This process-orchestration shape (in-process call vs. self-re-exec vs. child-owned teardown acknowledgement) is genuinely new -- no existing vat command proxies a long-running run in three different ways -- so the whole file is hand-authored this WI (missing-generator:cli:compose-lifecycle-orchestration, trackers #1484, #1526, and #1529), the same class of gap Phase 2's commands/build.rs recorded for its own dual-mode divergence (missing-generator:cli:streamed-subprocess-dual-mode, tracker #1479)."

//! Compose lifecycle orchestration: import/up/down/ps/logs for docker-compose projects.
//!
//! Manages a registry at `root/compose/<project>/project.json` to track
//! running compose projects, their vat_id, and service list. Up runs in two
//! modes: foreground (poll in-process, then run), or --detach (re-exec self).

use crate::cli::ComposeCmd;
use crate::config::ServiceRuntime;
use crate::spec::{GpuRequest, Isolation};
use crate::state::{ProcessStatus, Status, TestRunEvidence};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
#[cfg(unix)]
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Compose project registry entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComposeRecord {
    project: String,
    vat_id: Option<String>,
    /// Exact Docker-shaped compatibility profile that created this record.
    /// Ordinary `vat compose import` deliberately clears this value so its
    /// generic lifecycle owns the newly imported project. Missing provenance
    /// is legacy/generic state and is never enough for a Docker-shaped post
    /// verb to operate it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    docker_shim_profile: Option<String>,
    /// Monotonically advances for every detached launch of this durable
    /// project record. It is intentionally not inferred from `vat_id`: a
    /// detached `down`/re-import/relaunch can reuse names while a waiting
    /// Docker-shaped caller must never attach to that replacement lifecycle.
    #[serde(default, skip_serializing_if = "is_zero_generation")]
    launch_generation: u64,
    /// Fresh, persisted nonce paired with `launch_generation`. A generation
    /// alone is not globally unique across a removed/re-imported registry, so
    /// waiters pin both values before observing runner readiness.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    launch_ticket: Option<String>,
    /// Durable provenance for the token-owned compose launch protocol.  It
    /// remains after publication deliberately: the transient token and PID
    /// are cleared once their handoff is complete, but a later missing VAT
    /// metadata file must not make a current binding look like a historical
    /// uncorrelated record that is safe to reclaim.
    #[serde(default, skip_serializing_if = "is_legacy_handoff_protocol")]
    handoff_protocol: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    startup_pid: Option<u32>,
    /// Correlates a re-exec'd detached `vat run` with this exact startup so
    /// the child can durably publish its vat id if the parent exits early.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    startup_token: Option<String>,
    /// Time the token-backed detached handoff began. A record with neither a
    /// VAT id nor a launcher PID is only considered abandoned after a small
    /// grace window; this prevents a parent crash before spawn from wedging
    /// the project in `starting` forever.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    startup_started_at: Option<String>,
    service_ids: Vec<String>,
    status: String, // imported, starting, ready
    created_at: String,
}

fn is_zero_generation(generation: &u64) -> bool {
    *generation == 0
}

/// The lifecycle surface that owns a registry operation. Docker-shaped access
/// is deliberately private to the shim wrappers below: public `vat compose`
/// must not accidentally launch or operate a project imported through a
/// narrower Docker compatibility contract.
#[derive(Clone, Copy)]
enum ComposeAccess {
    VatCli,
    /// A deliberately narrow recovery route for a future Docker shim profile
    /// this VAT build does not understand. It may remove only an inactive
    /// registry, never launch, inspect, adopt, or tear down a runtime.
    VatCliDown,
    /// The validated profile captured from this exact `docker compose up`
    /// input must still be the one persisted by the shim import.
    DockerShimUp(crate::compose::DockerComposeProfile),
    /// Post verbs have no source file to parse, so they accept only one of the
    /// exact profiles VAT itself persisted during a prior shim import.
    DockerShimPost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ComposeAccessDecision {
    Full,
    UnknownShimRegistryCleanup,
}

/// A unique, one-shot ownership proof for a compose startup. Only the holder
/// of this token may publish the VAT id that backs a compose project; a VAT
/// name is deliberately not a correlation key because ordinary `vat run`
/// invocations may use the same name concurrently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ComposeHandoff {
    project: String,
    token: String,
}

impl ComposeHandoff {
    fn new(project: impl AsRef<str>, token: impl Into<String>) -> Result<Self> {
        let project = sanitize_project_name(project.as_ref());
        if project.is_empty() {
            bail!("compose startup handoff supplied an empty project name");
        }
        let token = token.into();
        if token.is_empty() {
            bail!("compose startup handoff supplied an empty token");
        }
        Ok(Self { project, token })
    }
}

/// The only truthful detached startup states. A discovered vat id alone is
/// not a startup success: its persisted service evidence must say every
/// compose service is Ready.
#[derive(Debug, Clone, PartialEq, Eq)]
enum DetachedStartup {
    Starting,
    Ready,
    /// The VAT parent has begun terminal teardown. Its runner may already be
    /// terminal, but compose must retain the binding until VAT status is
    /// Exited or Interrupted and every tracked service has a confirmed
    /// terminal state.
    Stopping,
    /// Persisted VAT evidence could not be read. This is never terminal: a
    /// concurrent atomic replacement or transient filesystem error cannot
    /// prove services were torn down, so the compose binding stays retained.
    EvidenceUnavailable(String),
    Terminal(String),
    /// The VAT reached a terminal state, but a VAT-owned MicroVM cleanup was
    /// not confirmed. The compose binding must stay retained until a retry
    /// proves the resource is gone.
    CleanupUnconfirmed(String),
}

/// Cross-process claim around every compose registry read-modify-write
/// transition. The persistent lock inode avoids the stale-file race: advisory
/// ownership is released by the OS when its owner crashes or is SIGKILLed.
struct StartupClaim {
    // An advisory lock is released by the OS if `vat compose up` crashes or is
    // SIGKILLed. The lock file intentionally remains as a stable lock inode;
    // deleting it after unlock would let a concurrent opener lock a new inode.
    #[cfg(unix)]
    _file: File,
    #[cfg(not(unix))]
    path: PathBuf,
}

impl StartupClaim {
    fn acquire(registry_dir: &Path, project_name: &str) -> Result<Self> {
        Self::acquire_with_deadline(registry_dir, project_name, None)
    }

    /// Detached children use a blocking claim during the parent-to-child
    /// handoff. The parent holds the claim through spawn and PID persistence,
    /// so a child cannot publish a VAT id that the parent subsequently
    /// overwrites with stale state.
    fn acquire_blocking(registry_dir: &Path, project_name: &str) -> Result<Self> {
        Self::acquire_with_deadline(
            registry_dir,
            project_name,
            Some(Instant::now() + STARTUP_CLAIM_WAIT),
        )
    }

    /// Wait only until a caller-owned readiness deadline. `docker compose up
    /// --wait` must not spend the historic fixed handoff/claim interval after
    /// its user supplied observation budget has expired.
    fn acquire_until(registry_dir: &Path, project_name: &str, deadline: Instant) -> Result<Self> {
        Self::acquire_with_deadline(registry_dir, project_name, Some(deadline))
    }

    #[cfg(unix)]
    fn acquire_with_deadline(
        registry_dir: &Path,
        project_name: &str,
        deadline: Option<Instant>,
    ) -> Result<Self> {
        let path = registry_dir.join("startup.lock");
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .with_context(|| format!("open compose startup lock {}", path.display()))?;
        loop {
            if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
                break;
            }
            let err = std::io::Error::last_os_error();
            let busy = matches!(
                err.raw_os_error(),
                Some(code) if code == libc::EWOULDBLOCK || code == libc::EAGAIN
            );
            if busy && deadline.is_some_and(|deadline| Instant::now() < deadline) {
                let remaining = deadline
                    .expect("deadline checked above")
                    .saturating_duration_since(Instant::now());
                std::thread::sleep(remaining.min(Duration::from_millis(10)));
                continue;
            }
            bail!(
                "compose project `{project_name}` has a lifecycle operation in progress; retry `vat compose ps {project_name}` ({err})"
            );
        }
        file.set_len(0)
            .with_context(|| format!("reset compose startup lock {}", path.display()))?;
        writeln!(file, "{}", std::process::id())
            .with_context(|| format!("write {}", path.display()))?;
        Ok(Self { _file: file })
    }

    #[cfg(not(unix))]
    fn acquire_with_deadline(
        registry_dir: &Path,
        project_name: &str,
        deadline: Option<Instant>,
    ) -> Result<Self> {
        let path = registry_dir.join("startup.lock");
        loop {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    if let Err(err) = writeln!(file, "{}", std::process::id()) {
                        let _ = fs::remove_file(&path);
                        return Err(err).with_context(|| format!("write {}", path.display()));
                    }
                    return Ok(Self { path });
                }
                Err(err)
                    if deadline.is_some() && err.kind() == std::io::ErrorKind::AlreadyExists =>
                {
                    if Instant::now() >= deadline.expect("checked above") {
                        bail!(
                            "compose project `{project_name}` has a lifecycle operation in progress; retry `vat compose ps {project_name}`"
                        );
                    }
                    let remaining = deadline
                        .expect("deadline checked above")
                        .saturating_duration_since(Instant::now());
                    std::thread::sleep(remaining.min(Duration::from_millis(10)));
                }
                Err(err) => {
                    return Err(err).with_context(|| {
                        format!(
                            "compose project `{project_name}` has a lifecycle operation in progress; retry `vat compose ps {project_name}`"
                        )
                    });
                }
            }
        }
    }
}

impl Drop for StartupClaim {
    fn drop(&mut self) {
        #[cfg(not(unix))]
        let _ = fs::remove_file(&self.path);
    }
}

/// Id of the synthesized runner every imported project gets (see `compose::materialize`).
const RUNNER_ID: &str = "project.up";
/// Version of the durable token-owned compose launch protocol.  Records that
/// predate it deserialize as zero and retain the narrowly scoped legacy
/// recovery behavior.
const HANDOFF_PROTOCOL: u8 = 1;

fn is_legacy_handoff_protocol(protocol: &u8) -> bool {
    *protocol == 0
}

/// An internal parent/child handoff may briefly contend on the same registry
/// immediately after the child publishes its VAT id.  Wait only for that
/// bounded transition; external lifecycle commands remain non-blocking.
const STARTUP_CLAIM_WAIT: Duration = Duration::from_secs(10);

/// A detached MicroVM service can take several seconds to acknowledge a
/// force-delete after its runner consumes the stop request. This outer wait
/// must exceed the bounded runtime teardown rather than racing it and making a
/// healthy Apple Container lifecycle look unacknowledged.
const COMPOSE_SHUTDOWN_WAIT: Duration = Duration::from_secs(60);

/// A real re-exec child records its PID immediately on entry. This small
/// window lets a parent crash before spawn be reclaimed without confusing the
/// normal spawn-to-exec handoff for a failed launch.
const DETACHED_HANDOFF_GRACE: Duration = Duration::from_secs(2);

/// Historic non-wait `compose up -d` behavior waits briefly for a
/// token-owned child to publish its VAT id. `--wait` supplies its own bounded
/// readiness deadline instead of adding this interval after it.
const DEFAULT_DETACHED_HANDOFF_WAIT: Duration = Duration::from_secs(10);

/// Main dispatch for compose subcommands.
pub fn exec(cmd: ComposeCmd) -> Result<ExitCode> {
    match cmd {
        ComposeCmd::Import {
            file,
            project,
            runtime,
        } => import_cmd(file, project, runtime),
        ComposeCmd::Up { project, detach } => {
            up_cmd(project, detach, ComposeAccess::VatCli, None, true).map(|outcome| outcome.code)
        }
        ComposeCmd::Down { project } => down_cmd(project, ComposeAccess::VatCliDown),
        ComposeCmd::Ps { project } => ps_cmd(project, ComposeAccess::VatCli),
        ComposeCmd::Logs { project, service } => logs_cmd(project, service, ComposeAccess::VatCli),
    }
}

/// Import an already-validated Docker-shaped Compose profile. Keeping this
/// surface private to the shim prevents ordinary VAT imports from minting
/// Docker provenance by accident.
pub(crate) fn import_docker_shim_profile(
    compose_file: crate::compose::ComposeFile,
    project: String,
    profile: crate::compose::DockerComposeProfile,
) -> Result<ImportedCompose> {
    import_parsed(
        compose_file,
        Some(project),
        ServiceRuntime::MicroVm,
        Some(profile),
    )
}

/// Start a record that was just imported by the Docker-shaped shim. The
/// expected profile is captured from the same parsed file used for import and
/// checked again under the lifecycle claim before any runtime process starts.
pub(crate) fn docker_shim_up(
    project: String,
    profile: crate::compose::DockerComposeProfile,
    readiness_deadline: Option<Instant>,
    emit_lifecycle_json: bool,
) -> Result<DockerShimLaunch> {
    let outcome = up_cmd(
        Some(project),
        true,
        ComposeAccess::DockerShimUp(profile),
        readiness_deadline,
        emit_lifecycle_json,
    )?;
    if outcome.code != ExitCode::SUCCESS {
        bail!("Docker-shaped compose launch did not reach a successful detached handoff");
    }
    if outcome.wait_deadline_elapsed && outcome.wait_target.is_none() {
        return Ok(DockerShimLaunch::DeadlineElapsedBeforeLaunch);
    }
    Ok(DockerShimLaunch::Launched {
        target: outcome.wait_target.context(
            "Docker-shaped compose launch completed without a persisted generation/ticket wait target",
        )?,
        deadline_elapsed: outcome.wait_deadline_elapsed,
    })
}

/// Outcome of initiating one Docker-shaped detached launch. A deadline that
/// expires after spawn deliberately retains the registry/runtime and its
/// target so the shim can emit one structured timeout rather than guessing
/// whether a child might still become ready.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DockerShimLaunch {
    DeadlineElapsedBeforeLaunch,
    Launched {
        target: DockerShimWaitTarget,
        deadline_elapsed: bool,
    },
}

/// Immutable handle for one detached Docker-shaped launch. This is not a VAT
/// id because names and ids are not a safe lifecycle correlation boundary;
/// callers must compare all three values while holding the registry claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DockerShimWaitTarget {
    profile: crate::compose::DockerComposeProfile,
    generation: u64,
    ticket: String,
}

/// One claimed readiness observation for a target launched by this shim call.
/// The caller must release the claim (this API does so before returning) and
/// sleep/retry only for `Starting` or transient evidence unavailability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DockerShimWaitObservation {
    /// No registry claim was acquired before the deadline. This is not an
    /// observation of the target and must not be paired with a stale endpoint
    /// or a Docker-shaped `ps` handoff.
    DeadlineElapsedBeforeClaim,
    Starting(DockerShimPsSnapshot),
    Ready(DockerShimPsSnapshot),
    Degraded(DockerShimPsSnapshot),
    Inactive(DockerShimPsSnapshot),
    Stopping(DockerShimPsSnapshot),
    LifecycleReplaced(String),
    EvidenceUnavailable(String),
    Terminal(String),
    CleanupUnconfirmed(String),
}

/// Docker-shaped post verbs intentionally cannot use the public dispatch:
/// a registry lacking valid shim provenance fails closed before it can inspect
/// or alter a generic compose project.
pub(crate) fn docker_shim_down(project: String) -> Result<ExitCode> {
    down_cmd(project, ComposeAccess::DockerShimPost)
}

/// A Docker-shaped `compose ps` observation assembled while VAT still holds
/// the compose registry claim. The shim serializes this value directly rather
/// than reopening the registry after the text projection has completed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DockerShimPsSnapshot {
    pub profile: crate::compose::DockerComposeProfile,
    pub topology: DockerShimTopologySnapshot,
}

/// Agent-facing topology for one narrow Docker Compose compatibility project.
/// `phase` is the lifecycle-wide state; a service can retain its last observed
/// state while the project is stopping, but endpoints are withheld unless the
/// whole project is currently ready.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DockerShimTopologySnapshot {
    pub phase: DockerShimTopologyPhase,
    pub ready: bool,
    pub services: Vec<DockerShimTopologyService>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DockerShimTopologyPhase {
    Inactive,
    Starting,
    Ready,
    /// Runner lifecycle evidence said Ready, but the agent-facing endpoint
    /// proof was incomplete or unsafe. This never leaks a partial endpoint.
    Degraded,
    Stopping,
}

impl DockerShimTopologyPhase {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Degraded => "degraded",
            Self::Stopping => "stopping",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DockerShimTopologyService {
    pub name: String,
    pub state: DockerShimTopologyServiceState,
    /// A canonical loopback address in `127.0.0.1:<port>` form. It is absent
    /// unless VAT can prove the exact running MicroVM and endpoint ownership.
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DockerShimTopologyServiceState {
    Inactive,
    Starting,
    Stopping,
    Created,
    Running,
    Ready,
    Interrupted,
    Exited,
    Failed,
    Timeout,
}

impl DockerShimTopologyServiceState {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::Starting => "starting",
            Self::Stopping => "stopping",
            Self::Created => "created",
            Self::Running => "running",
            Self::Ready => "ready",
            Self::Interrupted => "interrupted",
            Self::Exited => "exited",
            Self::Failed => "failed",
            Self::Timeout => "timeout",
        }
    }
}

/// Default and maximum line counts for a VAT-native Docker Compose log
/// snapshot. The JSON surface is intentionally bounded; text `logs SERVICE`
/// keeps its historic full-stream behavior.
pub(crate) const DEFAULT_DOCKER_SHIM_LOG_TAIL_LINES: usize = 200;
pub(crate) const MAX_DOCKER_SHIM_LOG_TAIL_LINES: usize = 1000;

/// Each Docker-shaped agent capture is bounded before decoding, and each
/// emitted JSON stream value is bounded again after lossy UTF-8 decoding plus
/// JSON escaping. The second cap prevents invalid bytes or control characters
/// from expanding an agent response beyond its advertised per-stream limit.
const MAX_DOCKER_SHIM_STREAM_CAPTURE_BYTES: u64 = 64 * 1024;
const MAX_DOCKER_SHIM_JSON_STREAM_VALUE_BYTES: usize = 64 * 1024;

/// One UTF-8-safe, bounded stream from a VAT-captured Compose service log.
/// `truncated` is true when either the byte window or requested line tail
/// omitted earlier content. `utf8_lossy` discloses replacement decoding for a
/// non-UTF-8 captured byte sequence rather than silently emitting invalid JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DockerShimLogStreamSnapshot {
    pub text: String,
    pub truncated: bool,
    pub utf8_lossy: bool,
}

/// Claimed, provenance-validated bounded log snapshot for one Docker-shaped
/// service. It is capture-only: callers never invoke Apple Container or write
/// the Compose project record while collecting it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DockerShimLogSnapshot {
    pub profile: crate::compose::DockerComposeProfile,
    pub stdout: DockerShimLogStreamSnapshot,
    pub stderr: DockerShimLogStreamSnapshot,
}

/// Result of one agent-native `docker compose exec --format json` invocation.
/// The same exact registry claim/provenance/ready proof used by text exec was
/// held through child spawn, but is deliberately released before waiting or
/// draining arbitrary command output.
#[derive(Debug)]
pub(crate) struct DockerShimExecSnapshot {
    pub profile: crate::compose::DockerComposeProfile,
    pub status: ExitStatus,
    pub stdout: DockerShimLogStreamSnapshot,
    pub stderr: DockerShimLogStreamSnapshot,
}

/// Docker-shaped post verbs intentionally cannot use the public dispatch:
/// a registry lacking valid shim provenance fails closed before it can inspect
/// or alter a generic compose project. This one returns the already-locked
/// observation so the caller cannot accidentally reopen a different record to
/// manufacture its public topology JSON.
pub(crate) fn docker_shim_ps(project: String) -> Result<DockerShimPsSnapshot> {
    docker_shim_ps_with_text(project, true)
}

/// The agent-facing JSON `docker compose ps --format json` surface must use
/// the same claim-held provenance observation as text mode, but cannot include
/// the historic human-readable table before its one JSON document.
pub(crate) fn docker_shim_ps_json(project: String) -> Result<DockerShimPsSnapshot> {
    docker_shim_ps_with_text(project, false)
}

fn docker_shim_ps_with_text(
    project: String,
    emit_human_text: bool,
) -> Result<DockerShimPsSnapshot> {
    let snapshot = collect_compose_ps_snapshot(project, ComposeAccess::DockerShimPost)?;
    if emit_human_text {
        print_compose_ps_snapshot(&snapshot, ComposeAccess::DockerShimPost);
    }
    snapshot.into_docker_shim_snapshot()
}

/// Observe exactly the detached launch identified by `target`. This acquires
/// one bounded claim, verifies immutable shim provenance plus the persisted
/// generation/ticket, reconciles the durable VAT runner evidence into typed
/// topology, then drops the claim before returning. It never probes TCP/HTTP:
/// the runner's own stored readiness evidence is the only readiness source.
pub(crate) fn docker_shim_wait_observe(
    project: String,
    target: &DockerShimWaitTarget,
    deadline: Instant,
) -> Result<DockerShimWaitObservation> {
    let project_name = sanitize_project_name(&project);
    let registry_dir = registry_dir_for_project(&project_name)?;
    if Instant::now() >= deadline {
        return Ok(DockerShimWaitObservation::DeadlineElapsedBeforeClaim);
    }
    let _claim = StartupClaim::acquire_until(&registry_dir, &project_name, deadline)?;
    let mut record = read_registry(&registry_dir)
        .with_context(|| format!("no compose project `{project_name}` in registry"))?;
    // Compare the wait target *before* generic Docker-shim provenance access.
    // A concurrent `down` + ordinary re-import deliberately clears that
    // provenance; it is a typed lifecycle replacement for this waiter, not a
    // generic error path that would skip the required structured result.
    if !docker_shim_wait_target_matches(&project_name, &record, target) {
        return Ok(DockerShimWaitObservation::LifecycleReplaced(format!(
            "expected profile `{}`, generation {}, ticket `{}`; registry now has profile `{}`, generation {}, ticket `{}`",
            target.profile.as_str(),
            target.generation,
            target.ticket,
            record.docker_shim_profile.as_deref().unwrap_or("none"),
            record.launch_generation,
            record.launch_ticket.as_deref().unwrap_or("none"),
        )));
    }
    require_compose_access(&record, &project_name, ComposeAccess::DockerShimPost)?;

    if record.vat_id.is_none() && record.status == "imported" {
        let snapshot =
            ComposePsSnapshot::from_record(&record, DockerShimTopologyPhase::Inactive, None)
                .into_docker_shim_snapshot()?;
        return Ok(DockerShimWaitObservation::Inactive(snapshot));
    }

    let ReconciledStartupEvidence { state, test_run } =
        reconcile_detached_startup_with_evidence(&record)?;
    let observation = match state {
        DetachedStartup::Starting => {
            record.status = "starting".to_string();
            write_registry(&registry_dir, &record)?;
            DockerShimWaitObservation::Starting(
                ComposePsSnapshot::from_record(
                    &record,
                    DockerShimTopologyPhase::Starting,
                    test_run,
                )
                .into_docker_shim_snapshot()?,
            )
        }
        DetachedStartup::Ready => {
            record.status = "ready".to_string();
            write_registry(&registry_dir, &record)?;
            let snapshot =
                ComposePsSnapshot::from_record(&record, DockerShimTopologyPhase::Ready, test_run)
                    .into_docker_shim_snapshot()?;
            if snapshot.topology.ready {
                DockerShimWaitObservation::Ready(snapshot)
            } else {
                DockerShimWaitObservation::Degraded(snapshot)
            }
        }
        DetachedStartup::Stopping => {
            record.status = "stopping".to_string();
            write_registry(&registry_dir, &record)?;
            DockerShimWaitObservation::Stopping(
                ComposePsSnapshot::from_record(
                    &record,
                    DockerShimTopologyPhase::Stopping,
                    test_run,
                )
                .into_docker_shim_snapshot()?,
            )
        }
        DetachedStartup::EvidenceUnavailable(message) => {
            DockerShimWaitObservation::EvidenceUnavailable(message)
        }
        DetachedStartup::Terminal(message) => {
            reset_active_run(&registry_dir, &mut record)?;
            DockerShimWaitObservation::Terminal(message)
        }
        DetachedStartup::CleanupUnconfirmed(message) => {
            DockerShimWaitObservation::CleanupUnconfirmed(message)
        }
    };
    Ok(observation)
}

pub(crate) fn docker_shim_logs(project: String, service: String) -> Result<ExitCode> {
    logs_cmd(project, service, ComposeAccess::DockerShimPost)
}

/// Read one bounded, VAT-captured service-log snapshot while holding the same
/// claim/provenance boundary as the text Docker-shaped `logs` verb. This is a
/// pure observation: it never invokes Apple Container or changes project.json.
pub(crate) fn docker_shim_logs_json(
    project: String,
    service: String,
    tail_lines: usize,
) -> Result<DockerShimLogSnapshot> {
    if !(1..=MAX_DOCKER_SHIM_LOG_TAIL_LINES).contains(&tail_lines) {
        bail!(
            "VAT's Docker-shaped JSON log snapshot tail must be between 1 and {MAX_DOCKER_SHIM_LOG_TAIL_LINES} lines"
        );
    }
    with_compose_log_source(
        &project,
        &service,
        ComposeAccess::DockerShimPost,
        |source| {
            let profile = source.profile.context(
                "Docker-shaped log snapshot lost recognized shim provenance while the compose claim was held",
            )?;
            Ok(DockerShimLogSnapshot {
                profile,
                stdout: bounded_log_stream(&source.stdout_log, tail_lines)?,
                stderr: bounded_log_stream(&source.stderr_log, tail_lines)?,
            })
        },
    )
}

/// Internal import result retained by callers that already captured a parsed
/// Compose document and need exact runtime-local image references without
/// reopening VAT's private materialized config.
#[derive(Debug)]
pub(crate) struct ImportedCompose {
    pub built_images: Vec<String>,
}

/// Import a compose file as a vat.toml project from the ordinary CLI surface.
fn import_cmd(file: PathBuf, project: Option<String>, runtime: ServiceRuntime) -> Result<ExitCode> {
    let compose_file = crate::compose::parse(&file)?;
    // A normal re-import is an explicit transfer back to the generic VAT
    // lifecycle, including when it reuses a project name that was previously
    // created through the Docker shim.
    import_parsed(compose_file, project, runtime, None).map(|_| ExitCode::SUCCESS)
}

/// Import an already captured Compose document. The strict Docker-shaped shim
/// uses this rather than reopening the path it validated, so a changed file or
/// symlink cannot smuggle a wider Compose shape into materialization.
fn import_parsed(
    compose_file: crate::compose::ComposeFile,
    project: Option<String>,
    runtime: ServiceRuntime,
    docker_shim_profile: Option<crate::compose::DockerComposeProfile>,
) -> Result<ImportedCompose> {
    let project_name = if let Some(p) = project {
        sanitize_project_name(&p)
    } else {
        compose_file
            .source_path()
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(sanitize_project_name)
            .ok_or_else(|| anyhow::anyhow!("cannot infer project name from compose file path"))?
    };
    if project_name.is_empty() {
        bail!("compose project name must contain at least one letter, number, dash, or underscore");
    }

    let registry_dir = registry_dir_for_project(&project_name)?;
    fs::create_dir_all(&registry_dir)
        .with_context(|| format!("create registry dir {}", registry_dir.display()))?;
    let _claim = StartupClaim::acquire(&registry_dir, &project_name)?;
    let prior_launch_generation = if registry_dir.join("project.json").exists() {
        let existing = read_registry(&registry_dir)?;
        if existing.vat_id.is_some() || matches!(existing.status.as_str(), "starting" | "ready") {
            bail!(
                "compose project `{project_name}` has an active lifecycle; run `vat compose down {project_name}` before re-importing"
            );
        }
        if docker_shim_profile.is_none()
            && existing
                .docker_shim_profile
                .as_deref()
                .is_some_and(|profile| !crate::compose::DockerComposeProfile::is_known(profile))
        {
            bail!(
                "compose project `{project_name}` has unknown Docker shim provenance; generic import will not adopt it. Run `vat compose down {project_name}` for inactive registry-only cleanup, then import again"
            );
        }
        existing.launch_generation
    } else {
        0
    };
    // Expand (and therefore preflight/build) only while this project claim is
    // held. A failing runtime leaves the previous materialized import intact,
    // and a concurrent import cannot replace its registry in the meantime.
    let services = crate::compose::expand(&compose_file, &project_name, runtime)?;
    let built_images = services
        .iter()
        .filter(|service| compose_file.service_uses_build(&service.id))
        .filter_map(|service| service.image.clone())
        .collect();
    let vat_toml = registry_dir.join("vat.toml");
    let previous_vat_toml = match fs::read(&vat_toml) {
        Ok(contents) => Some(contents),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(error).with_context(|| {
                format!("read existing materialized config {}", vat_toml.display())
            });
        }
    };
    crate::compose::materialize(&services, &vat_toml)?;
    let service_ids = match compose_service_ids(&vat_toml) {
        Ok(service_ids) => service_ids,
        Err(error) => {
            return Err(rollback_failed_import(
                &vat_toml,
                previous_vat_toml.as_deref(),
                "validate newly materialized vat.toml",
                error,
            ));
        }
    };

    let record = ComposeRecord {
        project: project_name.clone(),
        vat_id: None,
        docker_shim_profile: docker_shim_profile.map(|profile| profile.as_str().to_string()),
        // A re-import is a new lifecycle boundary, but preserve the monotonic
        // counter so the next detached launch advances rather than accidentally
        // reusing a prior generation. The ticket is deliberately cleared: no
        // wait target survives import without a fresh launch.
        launch_generation: prior_launch_generation,
        launch_ticket: None,
        handoff_protocol: HANDOFF_PROTOCOL,
        startup_pid: None,
        startup_token: None,
        startup_started_at: None,
        service_ids,
        status: "imported".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    if let Err(error) = write_registry(&registry_dir, &record) {
        return Err(rollback_failed_import(
            &vat_toml,
            previous_vat_toml.as_deref(),
            "publish compose registry",
            error,
        ));
    }

    println!(
        "Imported compose project `{project_name}` -> {}",
        vat_toml.display()
    );
    Ok(ImportedCompose { built_images })
}

/// Prove the caller owns this exact lifecycle contract while holding the
/// registry claim. This is intentionally checked before any reconciliation,
/// state transition, log read, or runtime process spawn so a generic VAT
/// command cannot accidentally take over a Docker-shaped project (and vice
/// versa).
fn require_compose_access(
    record: &ComposeRecord,
    project_name: &str,
    access: ComposeAccess,
) -> Result<ComposeAccessDecision> {
    if record.project != project_name {
        bail!(
            "compose project `{project_name}` registry belongs to `{}`; refuse lifecycle access without an exact project binding",
            record.project
        );
    }
    match access {
        ComposeAccess::VatCli => {
            if let Some(profile) = record.docker_shim_profile.as_deref() {
                bail!(
                    "compose project `{project_name}` was imported by VAT's Docker shim profile `{profile}`; use Docker-shaped `docker compose` lifecycle commands, or explicitly re-import it with `vat compose import` to clear shim provenance"
                );
            }
        }
        ComposeAccess::VatCliDown => match record.docker_shim_profile.as_deref() {
            None => {}
            Some(profile) if crate::compose::DockerComposeProfile::is_known(profile) => {
                bail!(
                    "compose project `{project_name}` was imported by VAT's Docker shim profile `{profile}`; only Docker-shaped `docker compose -p {project_name} down` may clean up a known shim project"
                );
            }
            Some(_) => return Ok(ComposeAccessDecision::UnknownShimRegistryCleanup),
        },
        ComposeAccess::DockerShimUp(expected) => {
            let Some(actual) = record.docker_shim_profile.as_deref() else {
                bail!(
                    "compose project `{project_name}` has no Docker shim profile provenance; refusing Docker-shaped launch. Run `docker compose -f <file> -p {project_name} up -d` so VAT can import a validated profile"
                );
            };
            if actual != expected.as_str() {
                bail!(
                    "compose project `{project_name}` has Docker shim profile `{actual}`, not the validated `{}` profile for this launch; refusing to start a mismatched registry",
                    expected.as_str()
                );
            }
        }
        ComposeAccess::DockerShimPost => {
            let Some(profile) = record.docker_shim_profile.as_deref() else {
                bail!(
                    "compose project `{project_name}` has no Docker shim profile provenance; refusing Docker-shaped lifecycle access. Use `vat compose` for a generic project, or start it through `docker compose -f <file> -p {project_name} up -d`"
                );
            };
            if !crate::compose::DockerComposeProfile::is_known(profile) {
                bail!(
                    "compose project `{project_name}` has unsupported Docker shim provenance `{profile}`; refusing Docker-shaped lifecycle access"
                );
            }
        }
    }
    Ok(ComposeAccessDecision::Full)
}

/// Capture a wait handle while the launch's final `StartupClaim` is still
/// held. The profile comparison is repeated here even though the caller
/// checked access at the start: the durable record is the only authority a
/// later wait poll may use.
fn docker_shim_wait_target_from_record(
    record: &ComposeRecord,
    profile: crate::compose::DockerComposeProfile,
) -> Result<DockerShimWaitTarget> {
    if record.docker_shim_profile.as_deref() != Some(profile.as_str()) {
        bail!(
            "Docker-shaped compose launch lost profile provenance before its wait target could be persisted"
        );
    }
    if record.launch_generation == 0 {
        bail!("Docker-shaped compose launch did not persist a nonzero launch generation");
    }
    let ticket = record
        .launch_ticket
        .clone()
        .context("Docker-shaped compose launch did not persist a launch ticket")?;
    Ok(DockerShimWaitTarget {
        profile,
        generation: record.launch_generation,
        ticket,
    })
}

fn docker_shim_wait_target_matches(
    project_name: &str,
    record: &ComposeRecord,
    target: &DockerShimWaitTarget,
) -> bool {
    record.project == project_name
        && record.docker_shim_profile.as_deref() == Some(target.profile.as_str())
        && record.launch_generation == target.generation
        && record.launch_ticket.as_deref() == Some(target.ticket.as_str())
}

// <HANDWRITE gap="vat-compose-detached-readiness-reconciliation" tracker="#1526" reason="Reconcile persisted VAT service records for detached compose so starting, ready, and terminal startup failure are truthful and diagnosable.">
#[derive(Debug)]
struct ComposeUpOutcome {
    code: ExitCode,
    /// Present only for an opt-in Docker-shaped detached launch. It is
    /// captured under the same final claim that validates the handoff, before
    /// a `down`/re-import/relaunch can replace the registry.
    wait_target: Option<DockerShimWaitTarget>,
    /// True only for a Docker-shaped `--wait` launch whose supplied readiness
    /// deadline elapsed before spawn or after spawn while handoff ownership
    /// was still settling. It is deliberately not an error: the caller must
    /// emit structured timeout JSON and leave durable state untouched.
    wait_deadline_elapsed: bool,
}

/// Start a compose project (foreground or detached).
fn up_cmd(
    project: Option<String>,
    detach: bool,
    access: ComposeAccess,
    readiness_deadline: Option<Instant>,
    emit_lifecycle_json: bool,
) -> Result<ComposeUpOutcome> {
    let project_name = sanitize_project_name(
        &project.ok_or_else(|| anyhow::anyhow!("--project required for up"))?,
    );
    let registry_dir = registry_dir_for_project(&project_name)?;
    let vat_toml = registry_dir.join("vat.toml");
    if !vat_toml.exists() {
        bail!("no imported compose project `{project_name}` -- run `vat compose import` first");
    }
    if detach && readiness_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
        return Ok(ComposeUpOutcome {
            code: ExitCode::SUCCESS,
            wait_target: None,
            wait_deadline_elapsed: true,
        });
    }

    let claim = match readiness_deadline {
        Some(deadline) => {
            if Instant::now() >= deadline {
                return Ok(ComposeUpOutcome {
                    code: ExitCode::SUCCESS,
                    wait_target: None,
                    wait_deadline_elapsed: true,
                });
            }
            match StartupClaim::acquire_until(&registry_dir, &project_name, deadline) {
                Ok(claim) => claim,
                Err(_error) if Instant::now() >= deadline => {
                    return Ok(ComposeUpOutcome {
                        code: ExitCode::SUCCESS,
                        wait_target: None,
                        wait_deadline_elapsed: true,
                    });
                }
                Err(error) => return Err(error),
            }
        }
        None => StartupClaim::acquire(&registry_dir, &project_name)?,
    };
    let mut record = load_and_validate_registry(&registry_dir, &project_name, &vat_toml)?;
    require_compose_access(&record, &project_name, access)?;
    if detach && readiness_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
        drop(claim);
        return Ok(ComposeUpOutcome {
            code: ExitCode::SUCCESS,
            wait_target: None,
            wait_deadline_elapsed: true,
        });
    }
    if matches!(record.status.as_str(), "started" | "running") {
        // Records written by older VAT versions used these intermediate labels.
        // Treat them as active instead of clearing their vat id and spawning a
        // second run.
        record.status = "starting".to_string();
        write_registry(&registry_dir, &record)?;
    }
    if record.vat_id.is_some()
        || matches!(record.status.as_str(), "starting" | "ready" | "stopping")
    {
        match reconcile_detached_startup(&record)? {
            DetachedStartup::Terminal(_) => reset_active_run(&registry_dir, &mut record)?,
            DetachedStartup::CleanupUnconfirmed(message) => {
                bail!(compose_cleanup_unconfirmed_error(
                    &project_name,
                    record.vat_id.as_deref(),
                    &message
                ));
            }
            DetachedStartup::Starting | DetachedStartup::Ready | DetachedStartup::Stopping => {
                bail!(
                    "compose project `{project_name}` is already {}; use `vat compose ps {project_name}` or `vat compose down {project_name}`",
                    record.status
                );
            }
            DetachedStartup::EvidenceUnavailable(message) => {
                bail!(
                    "compose project `{project_name}` VAT evidence is temporarily unavailable: {message}; registry retained to avoid overlapping services; retry `vat compose ps {project_name}`"
                );
            }
        }
    }
    record.status = "starting".to_string();
    record.vat_id = None;
    // A re-launch through this binary is now token-owned even when the
    // imported registry originated before the protocol existed.  Keep this
    // marker after publish/reset so missing evidence never frees a current
    // service binding.
    record.handoff_protocol = HANDOFF_PROTOCOL;
    record.startup_pid = None;
    record.startup_token = None;
    record.startup_started_at = None;
    if detach {
        record.launch_generation = record
            .launch_generation
            .checked_add(1)
            .context("compose launch generation exhausted")?;
        record.launch_ticket = Some(crate::id::fresh());
    } else {
        record.launch_ticket = None;
    }
    write_registry(&registry_dir, &record)?;

    if detach {
        // Persist this immutable target before a child exists. A timeout after
        // spawn must return it to the shim without trying to rediscover a VAT
        // id (which could already belong to a replacement lifecycle).
        let launch_wait_target = match access {
            ComposeAccess::DockerShimUp(profile) => {
                Some(docker_shim_wait_target_from_record(&record, profile)?)
            }
            ComposeAccess::VatCli | ComposeAccess::VatCliDown | ComposeAccess::DockerShimPost => {
                None
            }
        };
        // Persist this correlation token before spawn. The re-exec'd child uses
        // it to write its VAT id itself, so a slow clone or a parent crash
        // cannot strand the project in `starting`. The token, not the VAT
        // name or creation time, is the only authority allowed to bind this
        // compose project to a VAT.
        let handoff = ComposeHandoff::new(&project_name, crate::id::fresh())?;
        record.startup_token = Some(handoff.token.clone());
        record.startup_started_at = Some(Utc::now().to_rfc3339());
        write_registry(&registry_dir, &record)?;
        if readiness_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            // There is no child yet, so do not retain a false active ticket.
            reset_active_run(&registry_dir, &mut record)?;
            return Ok(ComposeUpOutcome {
                code: ExitCode::SUCCESS,
                wait_target: None,
                wait_deadline_elapsed: true,
            });
        }
        // `vat docker install-shim` intentionally invokes this same binary
        // through a `docker` symlink. Re-exec the canonical VAT target, not
        // that argv0 path: otherwise the detached child interprets its own
        // internal `vat run` as a Docker command and never publishes compose
        // ownership evidence.
        let runner_executable = std::env::current_exe()
            .context("resolve current VAT executable for detached compose run")?
            .canonicalize()
            .context("canonicalize VAT executable for detached compose run")?;
        let mut child = match Command::new(&runner_executable)
            .arg("run")
            .arg(RUNNER_ID)
            .arg("--name")
            .arg(&project_name)
            // Compose must preserve failed startup evidence even when a
            // user-edited imported vat.toml changes its normal retention.
            .args(["--keep", "always"])
            .current_dir(&registry_dir)
            .env("VAT_COMPOSE_PROJECT", &handoff.project)
            .env("VAT_COMPOSE_STARTUP_TOKEN", &handoff.token)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(err) => {
                reset_active_run(&registry_dir, &mut record)?;
                return Err(err).context("spawn detached `vat run`");
            }
        };
        record.startup_pid = Some(child.id());
        write_registry(&registry_dir, &record)?;
        drop(claim);

        let handoff_deadline =
            readiness_deadline.unwrap_or_else(|| Instant::now() + DEFAULT_DETACHED_HANDOFF_WAIT);
        let observed_vat_id =
            poll_for_detached_handoff(&registry_dir, &handoff, handoff_deadline, &mut child);
        // The detached child takes this same claim before it records its PID
        // or publishes the VAT id. Reacquiring before every post-poll update
        // prevents parent/child/ps lost updates and keeps project.json
        // serialized across processes.
        let _final_claim = match readiness_deadline {
            Some(deadline) => {
                if Instant::now() >= deadline {
                    return Ok(ComposeUpOutcome {
                        code: ExitCode::SUCCESS,
                        wait_target: launch_wait_target,
                        wait_deadline_elapsed: true,
                    });
                }
                match StartupClaim::acquire_until(&registry_dir, &project_name, deadline) {
                    Ok(claim) => claim,
                    Err(_error) if Instant::now() >= deadline => {
                        return Ok(ComposeUpOutcome {
                            code: ExitCode::SUCCESS,
                            wait_target: launch_wait_target,
                            wait_deadline_elapsed: true,
                        });
                    }
                    Err(error) => return Err(error),
                }
            }
            None => StartupClaim::acquire_blocking(&registry_dir, &project_name)?,
        };
        let mut record = read_registry(&registry_dir)?;
        let observed_vat_id = match observed_vat_id {
            Ok(vat_id) => vat_id,
            Err(err) => {
                // Reset only the startup this parent still owns. A child that
                // published before exiting (or a newer lifecycle) must never
                // be clobbered by this parent's stale in-memory record.
                if record.vat_id.is_none()
                    && record.startup_token.as_deref() == Some(handoff.token.as_str())
                {
                    reset_active_run(&registry_dir, &mut record)?;
                }
                return Err(err);
            }
        };
        // The child writes the registry itself while proving this exact token.
        // A parent must never infer a VAT id from the global store: a normal
        // `vat run --name <project>` can otherwise win the same-name race and
        // redirect compose cleanup to an unrelated service set.
        let token_matches = record.startup_token.as_deref() == Some(handoff.token.as_str());
        let child_already_published = match (record.vat_id.as_deref(), observed_vat_id.as_deref()) {
            (Some(actual), Some(observed)) => actual == observed && record.startup_token.is_none(),
            (None, None) => token_matches,
            _ => false,
        };
        if !child_already_published {
            bail!(
                "detached compose startup for `{project_name}` lost registry ownership; inspect `vat compose ps {project_name}`"
            );
        }
        match reconcile_detached_startup(&record)? {
            DetachedStartup::Starting => record.status = "starting".to_string(),
            DetachedStartup::Ready => record.status = "ready".to_string(),
            DetachedStartup::Stopping => record.status = "stopping".to_string(),
            DetachedStartup::EvidenceUnavailable(message) => {
                return Err(anyhow::anyhow!(
                    "detached compose startup for `{project_name}` VAT evidence is temporarily unavailable: {message}; registry retained to avoid overlapping services"
                ));
            }
            DetachedStartup::Terminal(message) => {
                let vat_id = record.vat_id.clone();
                reset_active_run(&registry_dir, &mut record)?;
                return Err(compose_terminal_startup_error(
                    &project_name,
                    vat_id.as_deref(),
                    &message,
                ));
            }
            DetachedStartup::CleanupUnconfirmed(message) => {
                return Err(compose_cleanup_unconfirmed_error(
                    &project_name,
                    record.vat_id.as_deref(),
                    &message,
                ));
            }
        }
        write_registry(&registry_dir, &record)?;

        let wait_target = match (launch_wait_target, access) {
            (Some(expected), ComposeAccess::DockerShimUp(profile)) => {
                let actual = docker_shim_wait_target_from_record(&record, profile)?;
                if actual != expected {
                    bail!(
                        "detached Docker-shaped compose launch changed generation/ticket during its initial handoff"
                    );
                }
                Some(actual)
            }
            (
                None,
                ComposeAccess::VatCli | ComposeAccess::VatCliDown | ComposeAccess::DockerShimPost,
            ) => None,
            (None, ComposeAccess::DockerShimUp(_)) => bail!(
                "Docker-shaped compose launch lost its persisted wait target before final handoff"
            ),
            (
                Some(_),
                ComposeAccess::VatCli | ComposeAccess::VatCliDown | ComposeAccess::DockerShimPost,
            ) => {
                bail!("generic compose launch unexpectedly captured Docker shim wait provenance")
            }
        };
        if emit_lifecycle_json {
            crate::commands::print_json(
                &serde_json::json!({
                    "project": project_name,
                    "vat_id": record.vat_id,
                    "status": record.status,
                }),
                true,
            )?;
        }
        return Ok(ComposeUpOutcome {
            code: ExitCode::SUCCESS,
            wait_target,
            wait_deadline_elapsed: false,
        });
    }

    // Foreground uses the same token-owned handoff as a detached child. This
    // eliminates the former background name/time store poll, which could bind
    // an unrelated ordinary `vat run --name <project>` to this compose run.
    let handoff = ComposeHandoff::new(&project_name, crate::id::fresh())?;
    record.startup_pid = Some(std::process::id());
    record.startup_token = Some(handoff.token.clone());
    record.startup_started_at = Some(Utc::now().to_rfc3339());
    write_registry(&registry_dir, &record)?;
    drop(claim);

    std::env::set_current_dir(&registry_dir)
        .with_context(|| format!("cd into {}", registry_dir.display()))?;
    crate::commands::run::exec(crate::commands::run::Args {
        target: crate::commands::run::Target::Runner {
            runner_ids: vec![RUNNER_ID.to_string()],
        },
        base: None,
        from: None,
        name: Some(project_name),
        isolation: Isolation::default(),
        gpu: GpuRequest::default(),
        microvm_image: None,
        json: false,
        plan: None,
        keep: None,
        compose_handoff: Some(handoff),
    })
    .map(|code| ComposeUpOutcome {
        code,
        wait_target: None,
        wait_deadline_elapsed: false,
    })
}
// </HANDWRITE>

/// Stop a running compose project.
fn down_cmd(project: String, access: ComposeAccess) -> Result<ExitCode> {
    let project_name = sanitize_project_name(&project);
    let registry_dir = registry_dir_for_project(&project_name)?;
    // Hold the registry claim through acknowledgement and final reset. A
    // concurrent `up` must not bind a second service set while this request
    // is still waiting for the old VAT parent to finish cleanup.
    let _claim = StartupClaim::acquire(&registry_dir, &project_name)?;
    let mut record = read_registry(&registry_dir)
        .with_context(|| format!("no compose project `{project_name}` in registry"))?;
    if require_compose_access(&record, &project_name, access)?
        == ComposeAccessDecision::UnknownShimRegistryCleanup
    {
        return remove_unknown_shim_registry(&registry_dir, &project_name, &record);
    }

    if record.vat_id.is_none() && record.status == "imported" {
        bail!("compose project `{project_name}` is imported but has no active vat run");
    }
    match reconcile_detached_startup(&record)? {
        DetachedStartup::Starting => bail!(
            "compose project `{project_name}` is still starting; retry `vat compose down {project_name}` once the runner PID is persisted"
        ),
        DetachedStartup::EvidenceUnavailable(message) => {
            bail!(
                "compose project `{project_name}` VAT evidence is temporarily unavailable: {message}; registry retained to avoid overlapping services; retry `vat compose down {project_name}`"
            );
        }
        DetachedStartup::Stopping => {
            let vat_id = record
                .vat_id
                .as_deref()
                .context("stopping compose project is missing its VAT id")?;
            wait_for_compose_shutdown(vat_id, &record.service_ids, COMPOSE_SHUTDOWN_WAIT)?;
            reset_active_run(&registry_dir, &mut record)?;
            println!("Stopped compose project `{project_name}` after VAT cleanup");
            return Ok(ExitCode::SUCCESS);
        }
        DetachedStartup::Terminal(message) => {
            reset_active_run(&registry_dir, &mut record)?;
            println!("compose project `{project_name}` already terminated: {message}");
            return Ok(ExitCode::SUCCESS);
        }
        DetachedStartup::CleanupUnconfirmed(message) => {
            let vat_id = record
                .vat_id
                .as_deref()
                .context("cleanup-unconfirmed compose project is missing its VAT id")?;
            let mut vat = crate::store::load(vat_id).with_context(|| {
                format!("load VAT {vat_id} for compose project `{project_name}` cleanup retry")
            })?;
            if let Err(error) = crate::commands::run::retry_unconfirmed_service_cleanup(&mut vat) {
                return Err(compose_cleanup_unconfirmed_error(
                    &project_name,
                    Some(vat_id),
                    &format!("{message}; retry failed: {error}"),
                ));
            }
            match reconcile_detached_startup(&record)? {
                DetachedStartup::Terminal(recovered) => {
                    reset_active_run(&registry_dir, &mut record)?;
                    println!(
                        "Stopped compose project `{project_name}` after confirming prior MicroVM cleanup: {recovered}"
                    );
                    return Ok(ExitCode::SUCCESS);
                }
                DetachedStartup::CleanupUnconfirmed(retry_message) => {
                    return Err(compose_cleanup_unconfirmed_error(
                        &project_name,
                        Some(vat_id),
                        &retry_message,
                    ));
                }
                DetachedStartup::Starting | DetachedStartup::Ready | DetachedStartup::Stopping => {
                    bail!(
                        "compose project `{project_name}` changed while retrying cleanup; inspect `vat state {vat_id}` before retrying `vat compose down {project_name}`"
                    );
                }
                DetachedStartup::EvidenceUnavailable(message) => {
                    bail!(
                        "compose project `{project_name}` VAT evidence is temporarily unavailable after cleanup retry: {message}; registry retained to avoid overlapping services"
                    );
                }
            }
        }
        DetachedStartup::Ready => {}
    }

    let vat_id = record
        .vat_id
        .as_deref()
        .context("ready compose project is missing its vat id")?;

    let vat = crate::store::load(vat_id)
        .with_context(|| format!("load vat {vat_id} for compose project `{project_name}`"))?;

    // Do not signal a persisted OS PID directly: it can be stale or reused,
    // and resetting the registry before the VAT parent owns teardown creates
    // a port-collision window. The live VAT process consumes this request,
    // stops its own runner/services, and persists Status::Exited first.
    crate::commands::run::request_detached_compose_stop(&vat)?;
    wait_for_compose_shutdown(vat_id, &record.service_ids, COMPOSE_SHUTDOWN_WAIT)?;
    reset_active_run(&registry_dir, &mut record)?;
    println!("Stopped compose project `{project_name}` after VAT cleanup");
    Ok(ExitCode::SUCCESS)
}

/// Forward-compatible escape hatch for an inactive record created by a newer
/// Docker shim profile. This build cannot truthfully inspect or stop that
/// profile, so it only removes the durable registry binding while holding the
/// claim. It never starts a runtime and preserves `vat.toml` for diagnosis;
/// any later use must perform a fresh import rather than silently adopting an
/// unknown contract.
fn remove_unknown_shim_registry(
    registry_dir: &Path,
    project_name: &str,
    record: &ComposeRecord,
) -> Result<ExitCode> {
    if record.vat_id.is_some()
        || record.status != "imported"
        || record.startup_pid.is_some()
        || record.startup_token.is_some()
        || record.startup_started_at.is_some()
    {
        bail!(
            "compose project `{project_name}` has unknown Docker shim provenance and may still own runtime resources or an in-flight startup handoff; refusing registry-only cleanup. Use a VAT version that recognizes `{}` or restore the matching Docker shim before retrying",
            record.docker_shim_profile.as_deref().unwrap_or("unknown")
        );
    }
    let path = registry_dir.join("project.json");
    fs::remove_file(&path)
        .with_context(|| format!("remove unknown Docker shim registry {}", path.display()))?;
    println!(
        "Removed inactive unknown Docker shim registry for `{project_name}` without touching a runtime; re-import before launching this project"
    );
    Ok(ExitCode::SUCCESS)
}

// <HANDWRITE gap="vat-compose-detached-readiness-projection" tracker="#1526" reason="Project reconciled detached compose state instead of treating a discovered VAT id as a successful startup.">
/// A complete `ps` observation assembled under one compose registry claim.
/// The runner evidence belongs to the same reconciliation read that selected
/// `phase`, so the Docker-shaped projection cannot pair a fresh registry with
/// a different generation of VAT metadata.
#[derive(Debug)]
struct ComposePsSnapshot {
    project: String,
    docker_shim_profile: Option<crate::compose::DockerComposeProfile>,
    phase: DockerShimTopologyPhase,
    service_ids: Vec<String>,
    vat_id: Option<String>,
    test_run: Option<TestRunEvidence>,
}

impl ComposePsSnapshot {
    fn from_record(
        record: &ComposeRecord,
        phase: DockerShimTopologyPhase,
        test_run: Option<TestRunEvidence>,
    ) -> Self {
        Self {
            project: record.project.clone(),
            docker_shim_profile: docker_shim_profile_from_record(record),
            phase,
            service_ids: record.service_ids.clone(),
            vat_id: record.vat_id.clone(),
            test_run,
        }
    }

    fn into_docker_shim_snapshot(self) -> Result<DockerShimPsSnapshot> {
        let profile = self.docker_shim_profile.context(
            "Docker-shaped compose ps lost its validated profile provenance while assembling the locked topology snapshot",
        )?;
        let observed_services = self
            .service_ids
            .iter()
            .map(|service_id| {
                (
                    service_id,
                    self.test_run
                        .as_ref()
                        .and_then(|test_run| unique_service_evidence(test_run, service_id)),
                )
            })
            .collect::<Vec<_>>();
        // `DetachedStartup::Ready` means the runner considers each service
        // ready. An agent-facing endpoint needs a stricter proof: one exact
        // VAT-owned Apple Container record for every service. Do not expose a
        // partially proven service set as ready just because its runner lives.
        let ready = self.phase == DockerShimTopologyPhase::Ready
            && observed_services.iter().all(|(service_id, evidence)| {
                docker_shim_loopback_endpoint(self.vat_id.as_deref(), service_id, *evidence)
                    .is_some()
            });
        let phase = if self.phase == DockerShimTopologyPhase::Ready && !ready {
            DockerShimTopologyPhase::Degraded
        } else {
            self.phase
        };
        let services = observed_services
            .into_iter()
            .map(|(service_id, evidence)| {
                let state = docker_shim_service_state(phase, evidence);
                let endpoint = (phase == DockerShimTopologyPhase::Ready)
                    .then(|| {
                        docker_shim_loopback_endpoint(self.vat_id.as_deref(), service_id, evidence)
                    })
                    .flatten();
                DockerShimTopologyService {
                    name: service_id.clone(),
                    state,
                    endpoint,
                }
            })
            .collect();
        Ok(DockerShimPsSnapshot {
            profile,
            topology: DockerShimTopologySnapshot {
                phase,
                ready,
                services,
            },
        })
    }
}

/// Parse only a durable profile that this build recognizes. The caller has
/// already run `require_compose_access`; returning `None` for unknown/legacy
/// records is still deliberate so a new call site cannot accidentally turn an
/// arbitrary string into Docker-shaped authority.
fn docker_shim_profile_from_record(
    record: &ComposeRecord,
) -> Option<crate::compose::DockerComposeProfile> {
    match record.docker_shim_profile.as_deref() {
        Some("strict-single-image-v1") => {
            Some(crate::compose::DockerComposeProfile::StrictSingleImageV1)
        }
        Some("strict-single-build-v1") => {
            Some(crate::compose::DockerComposeProfile::StrictSingleBuildV1)
        }
        Some("host-facing-independent-v1") => {
            Some(crate::compose::DockerComposeProfile::HostFacingIndependentV1)
        }
        Some(_) | None => None,
    }
}

/// Return exactly one service evidence record. Duplicate IDs in persisted
/// runner metadata are not a safe ownership proof, so the Docker projection
/// withholds both their endpoint and their evidence-derived status.
fn unique_service_evidence<'a>(
    test_run: &'a TestRunEvidence,
    service_id: &str,
) -> Option<&'a crate::state::ServiceRunRecord> {
    let mut matching = test_run
        .services
        .iter()
        .filter(|service| service.id == service_id);
    let service = matching.next()?;
    matching.next().is_none().then_some(service)
}

fn docker_shim_service_state(
    phase: DockerShimTopologyPhase,
    evidence: Option<&crate::state::ServiceRunRecord>,
) -> DockerShimTopologyServiceState {
    match evidence.map(|service| service.status) {
        Some(ProcessStatus::Created) => DockerShimTopologyServiceState::Created,
        Some(ProcessStatus::Running) => DockerShimTopologyServiceState::Running,
        Some(ProcessStatus::Ready) => DockerShimTopologyServiceState::Ready,
        Some(ProcessStatus::Interrupted) => DockerShimTopologyServiceState::Interrupted,
        Some(ProcessStatus::Exited) => DockerShimTopologyServiceState::Exited,
        Some(ProcessStatus::Failed) => DockerShimTopologyServiceState::Failed,
        Some(ProcessStatus::Timeout) => DockerShimTopologyServiceState::Timeout,
        None => match phase {
            DockerShimTopologyPhase::Inactive => DockerShimTopologyServiceState::Inactive,
            DockerShimTopologyPhase::Starting
            | DockerShimTopologyPhase::Ready
            | DockerShimTopologyPhase::Degraded => DockerShimTopologyServiceState::Starting,
            DockerShimTopologyPhase::Stopping => DockerShimTopologyServiceState::Stopping,
        },
    }
}

/// The public endpoint is intentionally stricter than ordinary `ps` text:
/// it proves the exact VAT-owned Apple Container resource as well as its
/// current Ready record. A stale/non-loopback/non-container record is useful
/// diagnostic evidence but never enough to hand an agent a routable endpoint.
fn docker_shim_loopback_endpoint(
    vat_id: Option<&str>,
    service_id: &str,
    evidence: Option<&crate::state::ServiceRunRecord>,
) -> Option<String> {
    let vat_id = vat_id?;
    let service = evidence?;
    let port = service.port?;
    let expected_microvm_name = compose_microvm_name(vat_id, service_id);
    if service.status != ProcessStatus::Ready
        || service.owned_by_vat != Some(true)
        || service.prepare_mode.as_deref() != Some("container_run")
        || service.cleanup_error.is_some()
        || service.host.as_deref() != Some("127.0.0.1")
        || port == 0
        || service.microvm_name.as_deref() != Some(expected_microvm_name.as_str())
    {
        return None;
    }
    Some(format!("127.0.0.1:{port}"))
}

/// Gather one provenance-validated `ps` observation while holding the compose
/// claim. This is the only path the Docker shim uses, so its JSON cannot race
/// a later registry import/replacement after the text projection finishes.
fn collect_compose_ps_snapshot(
    project: String,
    access: ComposeAccess,
) -> Result<ComposePsSnapshot> {
    let project_name = sanitize_project_name(&project);
    let registry_dir = registry_dir_for_project(&project_name)?;
    let _claim = StartupClaim::acquire(&registry_dir, &project_name)?;
    let mut record = read_registry(&registry_dir)
        .with_context(|| format!("no compose project `{project_name}` in registry"))?;
    require_compose_access(&record, &project_name, access)?;

    if record.vat_id.is_none() && record.status == "imported" {
        return Ok(ComposePsSnapshot::from_record(
            &record,
            DockerShimTopologyPhase::Inactive,
            None,
        ));
    }

    let ReconciledStartupEvidence { state, test_run } =
        reconcile_detached_startup_with_evidence(&record)?;
    match state {
        DetachedStartup::Starting => {
            record.status = "starting".to_string();
            write_registry(&registry_dir, &record)?;
            Ok(ComposePsSnapshot::from_record(
                &record,
                DockerShimTopologyPhase::Starting,
                test_run,
            ))
        }
        DetachedStartup::Ready => {
            record.status = "ready".to_string();
            write_registry(&registry_dir, &record)?;
            Ok(ComposePsSnapshot::from_record(
                &record,
                DockerShimTopologyPhase::Ready,
                test_run,
            ))
        }
        DetachedStartup::Stopping => {
            record.status = "stopping".to_string();
            write_registry(&registry_dir, &record)?;
            Ok(ComposePsSnapshot::from_record(
                &record,
                DockerShimTopologyPhase::Stopping,
                test_run,
            ))
        }
        DetachedStartup::EvidenceUnavailable(message) => Err(anyhow::anyhow!(
            "compose project `{project_name}` VAT evidence is temporarily unavailable: {message}; registry retained to avoid overlapping services; retry `vat compose ps {project_name}`"
        )),
        DetachedStartup::Terminal(message) => {
            let vat_id = record.vat_id.clone();
            reset_active_run(&registry_dir, &mut record)?;
            Err(compose_terminal_startup_error(
                &project_name,
                vat_id.as_deref(),
                &message,
            ))
        }
        DetachedStartup::CleanupUnconfirmed(message) => Err(compose_cleanup_unconfirmed_error(
            &project_name,
            record.vat_id.as_deref(),
            &message,
        )),
    }
}

/// Preserve the existing human-readable `vat compose ps` text surface while
/// keeping the Docker shim's final JSON derived from the same returned
/// snapshot. Text intentionally retains historic evidence ordering; only the
/// new machine topology has the record-service deterministic order contract.
fn print_compose_ps_snapshot(snapshot: &ComposePsSnapshot, access: ComposeAccess) {
    match snapshot.phase {
        DockerShimTopologyPhase::Inactive => match access {
            ComposeAccess::DockerShimPost => println!(
                "compose project `{}` is imported; the Docker shim does not retain its source Compose file. Restart only by rerunning Docker-shaped `compose up -d` with the same validated `-f <compose-file>` and `-p {}` arguments",
                snapshot.project, snapshot.project
            ),
            ComposeAccess::VatCli | ComposeAccess::VatCliDown | ComposeAccess::DockerShimUp(_) => {
                println!(
                    "compose project `{}` is imported; run `vat compose up --project {}`",
                    snapshot.project, snapshot.project
                );
            }
        },
        DockerShimTopologyPhase::Starting => {
            println!("compose project `{}` is starting", snapshot.project);
        }
        DockerShimTopologyPhase::Stopping => println!(
            "compose project `{}` is stopping; registry remains bound until VAT cleanup is confirmed",
            snapshot.project
        ),
        DockerShimTopologyPhase::Ready | DockerShimTopologyPhase::Degraded => {
            println!("compose project `{}` is ready", snapshot.project);
            if let Some(test_run) = snapshot.test_run.as_ref() {
                for service in &test_run.services {
                    if snapshot.service_ids.contains(&service.id) {
                        println!(
                            "{}\t{:?}\t{}",
                            service.id,
                            service.status,
                            service
                                .port
                                .map(|port| port.to_string())
                                .unwrap_or_else(|| "-".to_string())
                        );
                    }
                }
            }
        }
    }
}

/// List services in a compose project.
fn ps_cmd(project: String, access: ComposeAccess) -> Result<ExitCode> {
    let snapshot = collect_compose_ps_snapshot(project, access)?;
    print_compose_ps_snapshot(&snapshot, access);
    Ok(ExitCode::SUCCESS)
}
// </HANDWRITE>

/// Print logs from a service in a compose project.
fn logs_cmd(project: String, service: String, access: ComposeAccess) -> Result<ExitCode> {
    with_compose_log_source(&project, &service, access, |source| {
        print_file(&source.stdout_log)?;
        print_file(&source.stderr_log)?;
        Ok(ExitCode::SUCCESS)
    })
}

/// One service's durable VAT log paths exposed only to an observation closure
/// while its compose registry claim and access provenance remain current.
#[derive(Debug)]
struct ComposeLogSource {
    profile: Option<crate::compose::DockerComposeProfile>,
    stdout_log: String,
    stderr_log: String,
}

fn with_compose_log_source<T>(
    project: &str,
    service: &str,
    access: ComposeAccess,
    observe: impl FnOnce(&ComposeLogSource) -> Result<T>,
) -> Result<T> {
    let project_name = sanitize_project_name(project);
    let registry_dir = registry_dir_for_project(&project_name)?;
    let claim = StartupClaim::acquire(&registry_dir, &project_name)?;
    let record = read_registry(&registry_dir)
        .with_context(|| format!("no compose project `{project_name}` in registry"))?;
    require_compose_access(&record, &project_name, access)?;

    let Some(vat_id) = record.vat_id.clone() else {
        bail!("compose project `{project_name}` is still starting (no vat_id yet)");
    };

    if !record.service_ids.iter().any(|id| id == service) {
        bail!("service `{service}` is not part of compose project `{project_name}`");
    }

    let vat = crate::store::load(&vat_id)
        .with_context(|| format!("load vat {vat_id} for compose project `{project_name}`"))?;

    let Some(test_run) = vat.meta.test_run.as_ref() else {
        bail!("compose project `{project_name}` has no runner evidence yet");
    };

    let Some(svc) = test_run.services.iter().find(|s| s.id == service) else {
        bail!("no log source `{service}` in compose project `{project_name}`");
    };

    let source = ComposeLogSource {
        profile: docker_shim_profile_from_record(&record),
        stdout_log: svc.stdout_log.clone(),
        stderr_log: svc.stderr_log.clone(),
    };
    let result = observe(&source);
    // Keep the registry lock through the log read/print. A caller must never
    // pair provenance from one lifecycle with log paths from a later
    // re-import or teardown that reused the project name.
    drop(claim);
    result
}

/// Read only the trailing bounded portion of one captured service stream.
/// The file can grow while a service is running, so the metadata length is a
/// best-effort snapshot boundary; `Read::take` independently guarantees this
/// call never consumes more than the public byte cap.
fn bounded_log_stream(path: &str, tail_lines: usize) -> Result<DockerShimLogStreamSnapshot> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DockerShimLogStreamSnapshot {
                text: String::new(),
                truncated: false,
                utf8_lossy: false,
            });
        }
        Err(error) => return Err(error).with_context(|| format!("read log {path}")),
    };
    let length = file
        .metadata()
        .with_context(|| format!("inspect log {path}"))?
        .len();
    let byte_start = length.saturating_sub(MAX_DOCKER_SHIM_STREAM_CAPTURE_BYTES);
    file.seek(SeekFrom::Start(byte_start))
        .with_context(|| format!("seek log {path}"))?;
    let mut bytes = Vec::with_capacity(
        length
            .saturating_sub(byte_start)
            .min(MAX_DOCKER_SHIM_STREAM_CAPTURE_BYTES) as usize,
    );
    file.take(MAX_DOCKER_SHIM_STREAM_CAPTURE_BYTES)
        .read_to_end(&mut bytes)
        .with_context(|| format!("read bounded log {path}"))?;
    let (decoded, utf8_lossy) = match String::from_utf8_lossy(&bytes) {
        std::borrow::Cow::Borrowed(text) => (text.to_string(), false),
        std::borrow::Cow::Owned(text) => (text, true),
    };
    let lines = decoded.lines().collect::<Vec<_>>();
    let line_start = lines.len().saturating_sub(tail_lines);
    let (text, json_truncated) = cap_log_text_to_json_value(lines[line_start..].join("\n"))?;
    Ok(DockerShimLogStreamSnapshot {
        text,
        truncated: byte_start > 0 || line_start > 0 || json_truncated,
        utf8_lossy,
    })
}

/// Retain the newest valid UTF-8 suffix whose JSON string encoding stays
/// within the public stream budget. Measuring serde_json itself avoids making
/// assumptions about escaping rules for control or non-ASCII characters.
fn cap_log_text_to_json_value(text: String) -> Result<(String, bool)> {
    if serialized_json_string_len(&text)? <= MAX_DOCKER_SHIM_JSON_STREAM_VALUE_BYTES {
        return Ok((text, false));
    }

    let mut boundaries = text
        .char_indices()
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    boundaries.push(text.len());
    let mut lower = 0;
    let mut upper = boundaries.len() - 1;
    while lower < upper {
        let middle = lower + (upper - lower) / 2;
        let suffix = &text[boundaries[middle]..];
        if serialized_json_string_len(suffix)? <= MAX_DOCKER_SHIM_JSON_STREAM_VALUE_BYTES {
            upper = middle;
        } else {
            lower = middle + 1;
        }
    }
    let suffix = text[boundaries[lower]..].to_string();
    debug_assert!(serialized_json_string_len(&suffix)
        .is_ok_and(|length| length <= MAX_DOCKER_SHIM_JSON_STREAM_VALUE_BYTES));
    Ok((suffix, true))
}

fn serialized_json_string_len(text: &str) -> Result<usize> {
    serde_json::to_vec(text)
        .map(|encoded| encoded.len())
        .context("serialize bounded Docker-shaped log stream")
}

/// Spawn a non-interactive Docker-shaped exec only after proving the project,
/// profile, service readiness, and exact VAT-owned MicroVM name while holding
/// one registry claim. The claim is released immediately after spawn rather
/// than while waiting for an arbitrary foreground child, so lifecycle
/// operations cannot be stalled by a long shell command.
pub(crate) fn docker_shim_exec(
    project: &str,
    service: &str,
    command: &[String],
) -> Result<ExitStatus> {
    let mut spawned = spawn_docker_shim_exec(
        project,
        service,
        command,
        Stdio::inherit(),
        Stdio::inherit(),
    )?;
    spawned.child.wait().with_context(|| {
        format!(
            "wait for strict Compose service `{service}` command in Apple Container `{}`",
            spawned.microvm_name
        )
    })
}

/// Spawn and drain one agent-native non-interactive Docker-shaped exec. The
/// two pipe readers run concurrently, so a child that fills both streams
/// cannot deadlock while its output remains bounded in memory. The claim is
/// already released by `spawn_docker_shim_exec` before the arbitrary child
/// duration and stream drain begin.
pub(crate) fn docker_shim_exec_json(
    project: &str,
    service: &str,
    command: &[String],
) -> Result<DockerShimExecSnapshot> {
    let mut spawned =
        spawn_docker_shim_exec(project, service, command, Stdio::piped(), Stdio::piped())?;
    let stdout = match spawned.child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            terminate_docker_shim_exec_child(&mut spawned.child);
            bail!("Docker-shaped JSON compose exec child did not expose stdout capture");
        }
    };
    let stderr = match spawned.child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            terminate_docker_shim_exec_child(&mut spawned.child);
            bail!("Docker-shaped JSON compose exec child did not expose stderr capture");
        }
    };
    let stdout_reader = match thread::Builder::new()
        .name("vat-compose-exec-stdout".to_string())
        .spawn(move || bounded_exec_stream(stdout))
    {
        Ok(reader) => reader,
        Err(error) => {
            terminate_docker_shim_exec_child(&mut spawned.child);
            return Err(error).context("start bounded Docker-shaped compose exec stdout reader");
        }
    };
    let stderr_reader = match thread::Builder::new()
        .name("vat-compose-exec-stderr".to_string())
        .spawn(move || bounded_exec_stream(stderr))
    {
        Ok(reader) => reader,
        Err(error) => {
            terminate_docker_shim_exec_child(&mut spawned.child);
            let _ = stdout_reader.join();
            return Err(error).context("start bounded Docker-shaped compose exec stderr reader");
        }
    };

    let status = spawned.child.wait().with_context(|| {
        format!(
            "wait for strict Compose service `{service}` command in Apple Container `{}`",
            spawned.microvm_name
        )
    });
    let stdout = join_docker_shim_exec_reader(stdout_reader, "stdout");
    let stderr = join_docker_shim_exec_reader(stderr_reader, "stderr");
    Ok(DockerShimExecSnapshot {
        profile: spawned.profile,
        status: status?,
        stdout: stdout?,
        stderr: stderr?,
    })
}

struct DockerShimExecSpawn {
    profile: crate::compose::DockerComposeProfile,
    microvm_name: String,
    child: Child,
}

/// The spawn boundary retains all existing Docker-shim ownership checks. It
/// returns only after the child is launched and the registry claim is dropped,
/// so neither text nor JSON exec can hold lifecycle operations behind a long
/// foreground command.
fn spawn_docker_shim_exec(
    project: &str,
    service: &str,
    command: &[String],
    stdout: Stdio,
    stderr: Stdio,
) -> Result<DockerShimExecSpawn> {
    validate_docker_shim_exec_args(project, service, command)?;
    let project_name = sanitize_project_name(project);
    let registry_dir = registry_dir_for_project(&project_name)?;
    let claim = StartupClaim::acquire(&registry_dir, &project_name)?;
    let record = read_registry(&registry_dir)
        .with_context(|| format!("no compose project `{project_name}` in registry"))?;
    require_compose_access(&record, &project_name, ComposeAccess::DockerShimPost)?;
    let profile = docker_shim_profile_from_record(&record).context(
        "Docker-shaped compose exec lost recognized shim provenance while the compose claim was held",
    )?;
    let microvm_name = ready_microvm_name_for_docker_shim(&record, &project_name, service)?;

    // `spawn` is the operation authority boundary. It runs while the claim
    // still protects the exact profile and ready ownership proof. Waiting is
    // intentionally outside the claim so a user command cannot indefinitely
    // serialize `ps`/`down` or an explicit generic re-import.
    let child = Command::new("container")
        .arg("exec")
        .arg(&microvm_name)
        // Apple Container's exec grammar is `container exec CONTAINER
        // COMMAND [ARG...]`; unlike Docker's user-facing grammar, it has no
        // command separator. The validated Docker-facing delimiter is never
        // forwarded, and an option-looking first command remains raw argv.
        .args(command)
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .with_context(|| {
            format!("exec strict Compose service `{service}` in Apple Container `{microvm_name}`")
        })?;
    drop(claim);
    Ok(DockerShimExecSpawn {
        profile,
        microvm_name,
        child,
    })
}

/// A capture-setup error occurs after an authorized child spawn. Kill and
/// reap that child before surfacing the setup failure so a failed reader
/// allocation cannot orphan a command that still owns the copied stdio pipes.
fn terminate_docker_shim_exec_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn join_docker_shim_exec_reader(
    reader: thread::JoinHandle<Result<DockerShimLogStreamSnapshot>>,
    stream: &str,
) -> Result<DockerShimLogStreamSnapshot> {
    reader
        .join()
        .map_err(|_| {
            anyhow::anyhow!("bounded Docker-shaped compose exec {stream} reader panicked")
        })?
        .with_context(|| format!("capture bounded Docker-shaped compose exec {stream}"))
}

/// Drain an arbitrary child pipe without allowing output volume to grow this
/// agent result. Retaining only the latest bytes preserves the final command
/// diagnostics while continuously reading prevents a full OS pipe from
/// blocking the child. The JSON-serialized string cap is applied after lossy
/// decoding, exactly as it is for captured service logs.
fn bounded_exec_stream(mut reader: impl Read) -> Result<DockerShimLogStreamSnapshot> {
    let mut retained = Vec::with_capacity(MAX_DOCKER_SHIM_STREAM_CAPTURE_BYTES as usize);
    let mut chunk = [0_u8; 8 * 1024];
    let mut truncated = false;
    loop {
        let read = reader
            .read(&mut chunk)
            .context("read Docker-shaped compose exec child stream")?;
        if read == 0 {
            break;
        }
        let bytes = &chunk[..read];
        if bytes.len() >= MAX_DOCKER_SHIM_STREAM_CAPTURE_BYTES as usize {
            retained.clear();
            retained.extend_from_slice(
                &bytes[bytes.len() - MAX_DOCKER_SHIM_STREAM_CAPTURE_BYTES as usize..],
            );
            truncated = true;
            continue;
        }
        let overflow = retained
            .len()
            .saturating_add(bytes.len())
            .saturating_sub(MAX_DOCKER_SHIM_STREAM_CAPTURE_BYTES as usize);
        if overflow > 0 {
            retained.drain(..overflow);
            truncated = true;
        }
        retained.extend_from_slice(bytes);
    }
    let (decoded, utf8_lossy) = match String::from_utf8_lossy(&retained) {
        std::borrow::Cow::Borrowed(text) => (text.to_string(), false),
        std::borrow::Cow::Owned(text) => (text, true),
    };
    let (text, json_truncated) = cap_log_text_to_json_value(decoded)?;
    Ok(DockerShimLogStreamSnapshot {
        text,
        truncated: truncated || json_truncated,
        utf8_lossy,
    })
}

fn validate_docker_shim_exec_args(project: &str, service: &str, command: &[String]) -> Result<()> {
    let project_name = sanitize_project_name(project);
    let mut project_bytes = project.bytes();
    let valid_project_prefix = project_bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
    let valid_project_tail = project_bytes.all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
    });
    if project_name.is_empty()
        || project_name != project
        || !valid_project_prefix
        || !valid_project_tail
    {
        bail!(
            "compose exec requires a project name already valid without VAT sanitization: [a-z0-9][a-z0-9_-]*"
        );
    }
    if service.is_empty()
        || service.starts_with('-')
        || service
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        bail!("compose exec requires a non-empty service name without whitespace or leading '-'");
    }
    if command.is_empty() {
        bail!("compose exec requires a command after `--`");
    }
    Ok(())
}

/// The narrow Docker shim uses this proof to support `docker compose exec -T`
/// without exposing VAT's generated container name to an agent. It deliberately
/// refuses Docker-backed, external, unready, terminal, or cleanup-uncertain
/// services. Its caller must hold the registry claim.
fn ready_microvm_name_for_docker_shim(
    record: &ComposeRecord,
    project_name: &str,
    service: &str,
) -> Result<String> {
    if record.project != project_name {
        bail!(
            "compose project `{project_name}` registry belongs to `{}`; refuse to exec an unowned service",
            record.project
        );
    }
    if !record.service_ids.iter().any(|id| id == service) {
        bail!("service `{service}` is not part of compose project `{project_name}`");
    }

    // Keep the lifecycle classification and selected service evidence from
    // one metadata read. A second store load here could pair `Ready` from an
    // older revision with a different MicroVM record from a newer revision.
    let reconciled = reconcile_detached_startup_with_evidence(record)?;
    let vat_id = record
        .vat_id
        .as_deref()
        .context("ready compose project is missing its VAT id")?;
    ready_microvm_name_from_reconciled_evidence(vat_id, project_name, service, &reconciled)
}

/// Apply a single reconciled metadata revision to exec's readiness and exact
/// MicroVM proof. This deliberately accepts the reconciliation object rather
/// than reopening the store: `Ready` and the selected service record must be
/// inseparable parts of one snapshot.
fn ready_microvm_name_from_reconciled_evidence(
    vat_id: &str,
    project_name: &str,
    service: &str,
    reconciled: &ReconciledStartupEvidence,
) -> Result<String> {
    match &reconciled.state {
        DetachedStartup::Ready => {}
        DetachedStartup::Starting => bail!(
            "compose project `{project_name}` is still starting; retry `docker compose -p {project_name} exec -T {service} -- <command>` after `docker compose -p {project_name} ps` reports ready"
        ),
        DetachedStartup::Stopping => bail!(
            "compose project `{project_name}` is stopping; VAT will not exec into a service whose teardown has begun"
        ),
        DetachedStartup::EvidenceUnavailable(message) => bail!(
            "compose project `{project_name}` VAT evidence is temporarily unavailable: {message}; VAT will not exec without exact readiness proof"
        ),
        DetachedStartup::Terminal(message) => bail!(
            "compose project `{project_name}` is terminal: {message}; run `docker compose -p {project_name} down` or start a new strict profile before exec"
        ),
        DetachedStartup::CleanupUnconfirmed(message) => bail!(
            "compose project `{project_name}` cleanup is unconfirmed: {message}; VAT will not exec into an uncertain runtime resource"
        ),
    }
    let test_run = reconciled
        .test_run
        .as_ref()
        .context("ready compose project has no runner evidence")?;
    ready_microvm_name_from_evidence(vat_id, project_name, service, test_run)
}

/// Verify one service's evidence after reconciliation has already proved the
/// registry and runner state. Keeping this as a pure evidence proof makes the
/// duplicate-id rejection explicit and testable before the caller reaches its
/// Apple Container `spawn` authority boundary.
fn ready_microvm_name_from_evidence(
    vat_id: &str,
    project_name: &str,
    service: &str,
    test_run: &TestRunEvidence,
) -> Result<String> {
    let service_record = unique_service_evidence(test_run, service).with_context(|| {
        format!(
            "ready compose project `{project_name}` requires exactly one evidence record for service `{service}`; duplicate or missing evidence is not an ownership proof"
        )
    })?;
    if service_record.status != ProcessStatus::Ready {
        bail!(
            "compose service `{service}` is {:?}, not Ready; VAT will not exec into it",
            service_record.status
        );
    }
    if service_record.owned_by_vat != Some(true) {
        bail!(
            "compose service `{service}` is not a VAT-owned MicroVM runtime resource; VAT's Docker shim will not exec into it"
        );
    }
    if service_record.cleanup_error.is_some() {
        bail!(
            "compose service `{service}` has unconfirmed runtime cleanup; VAT will not exec into it"
        );
    }
    if service_record.prepare_mode.as_deref() != Some("container_run") {
        bail!(
            "compose service `{service}` is not backed by Apple Container; VAT's Docker shim supports compose exec only for its strict MicroVM profile"
        );
    }
    let microvm_name = service_record
        .microvm_name
        .as_deref()
        .context("ready MicroVM compose service is missing its exact Apple Container name")?;
    let expected = compose_microvm_name(vat_id, service);
    if microvm_name != expected {
        bail!(
            "compose service `{service}` reports an unexpected Apple Container name; VAT refuses to exec without exact ownership proof"
        );
    }
    Ok(microvm_name.to_string())
}

fn compose_microvm_name(vat_id: &str, service: &str) -> String {
    format!("{vat_id}-{service}")
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '-') {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn print_file(path: &str) -> Result<()> {
    match fs::read_to_string(path) {
        Ok(content) => {
            print!("{content}");
            Ok(())
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err).with_context(|| format!("read log {path}")),
    }
}

/// Wait only for a token-owned child to publish into its compose registry.
/// Global VAT-store name/time discovery is intentionally forbidden here: an
/// unrelated `vat run --name <project>` can be created in the same interval.
fn poll_for_detached_handoff(
    registry_dir: &Path,
    handoff: &ComposeHandoff,
    deadline: Instant,
    child: &mut Child,
) -> Result<Option<String>> {
    loop {
        if Instant::now() >= deadline {
            return Ok(None);
        }
        let record = read_registry(registry_dir).with_context(|| {
            format!(
                "read compose registry for `{}` while waiting for token-owned VAT publication",
                handoff.project
            )
        })?;
        if record.project != handoff.project {
            bail!(
                "detached compose startup for `{}` lost its registry project binding",
                handoff.project
            );
        }
        if let Some(vat_id) = record.vat_id {
            if record.startup_token.is_none() {
                return Ok(Some(vat_id));
            }
            bail!(
                "detached compose startup for `{}` published a VAT id without completing its token handoff",
                handoff.project
            );
        }
        if record.status != "starting"
            || record.startup_token.as_deref() != Some(handoff.token.as_str())
        {
            bail!(
                "detached compose startup for `{}` lost token ownership before VAT publication",
                handoff.project
            );
        }
        if let Some(status) = child.try_wait()? {
            bail!(
                "detached vat run for compose project `{}` exited {:?} before creating VAT evidence",
                handoff.project,
                status.code()
            );
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(None);
        }
        std::thread::sleep(remaining.min(Duration::from_millis(200)));
    }
}

/// Reconciliation plus the exact evidence revision used to make that
/// lifecycle decision. `docker compose ps` retains this value under its
/// registry claim instead of reconciling one VAT metadata revision and
/// projecting endpoints from a later reread.
#[derive(Debug)]
struct ReconciledStartupEvidence {
    state: DetachedStartup,
    test_run: Option<TestRunEvidence>,
}

fn reconcile_detached_startup(record: &ComposeRecord) -> Result<DetachedStartup> {
    Ok(reconcile_detached_startup_with_evidence(record)?.state)
}

fn reconcile_detached_startup_with_evidence(
    record: &ComposeRecord,
) -> Result<ReconciledStartupEvidence> {
    let Some(vat_id) = record.vat_id.as_deref() else {
        // Only new detached records carry a token. Older `started`/`running`
        // records without one retain their conservative legacy behavior, but
        // a token-backed launch can be classified once its re-exec child dies
        // rather than remaining in `starting` forever.
        if record.startup_token.is_some() {
            let state = match record.startup_pid {
                Some(pid) if detached_child_is_alive(pid) => DetachedStartup::Starting,
                Some(pid) => DetachedStartup::Terminal(format!(
                    "detached vat launcher pid {pid} exited before publishing VAT evidence"
                )),
                // The child records its own PID at the top of `vat run`; a
                // small spawn-to-exec window must remain recoverable, but a
                // token with no launcher forever is an abandoned parent crash
                // and must not wedge later up/down operations.
                None if detached_handoff_expired(record) => DetachedStartup::Terminal(format!(
                    "detached startup token never published a launcher pid within {}s",
                    DETACHED_HANDOFF_GRACE.as_secs()
                )),
                None => DetachedStartup::Starting,
            };
            return Ok(ReconciledStartupEvidence {
                state,
                test_run: None,
            });
        }
        if let Some(pid) = record.startup_pid {
            let state = if detached_child_is_alive(pid) {
                DetachedStartup::Starting
            } else {
                DetachedStartup::Terminal(format!(
                    "compose foreground launcher pid {pid} exited before publishing VAT evidence"
                ))
            };
            return Ok(ReconciledStartupEvidence {
                state,
                test_run: None,
            });
        }
        return Ok(ReconciledStartupEvidence {
            state: DetachedStartup::Starting,
            test_run: None,
        });
    };
    let vat = match crate::store::load(vat_id) {
        Ok(vat) => vat,
        Err(err) => {
            // Current compose launches retain their durable protocol marker
            // after the transient token and PID are cleared.  Their missing
            // metadata is not enough to establish that a service stopped:
            // retain the binding so a temporary read/delete race cannot
            // permit another run to reuse its published port.  Only
            // pre-protocol legacy records retain the historical recovery
            // path, and only when the metadata path is definitively absent
            // (not malformed or merely unreadable).
            if record.handoff_protocol == 0 && legacy_vat_metadata_is_definitively_absent(vat_id) {
                return Ok(ReconciledStartupEvidence {
                    state: DetachedStartup::Terminal(format!(
                        "legacy VAT evidence `{vat_id}` is absent"
                    )),
                    test_run: None,
                });
            }
            return Ok(ReconciledStartupEvidence {
                state: DetachedStartup::EvidenceUnavailable(format!(
                    "VAT evidence `{vat_id}` could not be read: {err}"
                )),
                test_run: None,
            });
        }
    };
    let crate::store::Vat { meta, .. } = vat;
    let status = meta.status;
    let test_run = meta.test_run;
    if !matches!(&status, Status::Exited { .. } | Status::Interrupted { .. }) {
        let state = detached_startup_while_active(&record.service_ids, test_run.as_ref());
        return Ok(ReconciledStartupEvidence { state, test_run });
    }
    let terminal_state = vat_terminal_state_label(&status);

    let Some(test_run_ref) = test_run.as_ref() else {
        return Ok(ReconciledStartupEvidence {
            state: DetachedStartup::Terminal(format!(
                "VAT reached {terminal_state} without compose run evidence"
            )),
            test_run: None,
        });
    };
    if let Some(message) = compose_cleanup_error(Some(test_run_ref), &record.service_ids) {
        return Ok(ReconciledStartupEvidence {
            state: DetachedStartup::CleanupUnconfirmed(message),
            test_run,
        });
    }
    if !compose_services_are_terminal(Some(test_run_ref), &record.service_ids) {
        return Ok(ReconciledStartupEvidence {
            state: DetachedStartup::Stopping,
            test_run,
        });
    }
    let outcome = detached_startup_from_evidence(&record.service_ids, Some(test_run_ref));
    let state = match outcome {
        // A terminal VAT cannot truthfully be starting or ready. Preserve the
        // binding until this point, then make malformed/incomplete evidence a
        // resettable terminal failure instead of wedging the project forever.
        DetachedStartup::Starting | DetachedStartup::Ready | DetachedStartup::Stopping => {
            DetachedStartup::Terminal(format!(
                "VAT reached {terminal_state} without terminal compose runner evidence"
            ))
        }
        terminal => terminal,
    };
    Ok(ReconciledStartupEvidence { state, test_run })
}

fn vat_terminal_state_label(status: &Status) -> &'static str {
    match status {
        Status::Interrupted { .. } => "Interrupted",
        Status::Exited { .. } => "Exited",
        Status::Created | Status::Running | Status::Snapshot => "nonterminal",
    }
}

/// A compatibility-only absence proof for records from before compose
/// handoffs existed.  `metadata` distinguishes a missing path from malformed
/// JSON, permission errors, and other I/O failures; only the former permits
/// legacy recovery.  Modern records never use this escape hatch.
fn legacy_vat_metadata_is_definitively_absent(vat_id: &str) -> bool {
    let Ok(vat_dir) = crate::paths::vat_dir(vat_id) else {
        return false;
    };
    matches!(
        fs::metadata(vat_dir.join(crate::paths::file::META)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound
    )
}

fn detached_handoff_expired(record: &ComposeRecord) -> bool {
    let Some(started_at) = record.startup_started_at.as_deref() else {
        // Token-bearing records from before this field existed cannot prove a
        // live launcher. Treat them as reclaimable; a delayed current child
        // checks token ownership before it creates VAT state.
        return true;
    };
    let Ok(started_at) = DateTime::parse_from_rfc3339(started_at) else {
        return true;
    };
    match Utc::now()
        .signed_duration_since(started_at.with_timezone(&Utc))
        .to_std()
    {
        Ok(age) => age >= DETACHED_HANDOFF_GRACE,
        // A clock moving backwards should prefer the safe, still-starting
        // interpretation rather than reclaiming a potentially live child.
        Err(_) => false,
    }
}

#[cfg(unix)]
fn detached_child_is_alive(pid: u32) -> bool {
    let result = unsafe { libc::kill(pid as i32, 0) };
    result == 0
        || std::io::Error::last_os_error()
            .raw_os_error()
            .is_some_and(|code| code == libc::EPERM)
}

#[cfg(not(unix))]
fn detached_child_is_alive(_pid: u32) -> bool {
    // The detached launcher handoff is still safe because the token can be
    // published by the child; a conservative fallback avoids declaring a
    // live process terminal on platforms without POSIX liveness probing.
    true
}

/// Wait for the VAT parent, not an arbitrary persisted PID, to acknowledge a
/// compose stop request and finish service teardown. The registry stays bound
/// on timeout so a subsequent command cannot start a second service set while
/// the first may still own published ports.
fn wait_for_compose_shutdown(
    vat_id: &str,
    service_ids: &[String],
    timeout: Duration,
) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        let vat = crate::store::load(vat_id)
            .with_context(|| format!("load VAT `{vat_id}` while waiting for compose shutdown"))?;
        if matches!(
            &vat.meta.status,
            Status::Exited { .. } | Status::Interrupted { .. }
        ) {
            if let Some(message) = compose_cleanup_error(vat.meta.test_run.as_ref(), service_ids) {
                bail!(
                    "compose stop request for VAT `{vat_id}` reached a terminal state but cleanup is unconfirmed: {message}; registry retained to avoid overlapping services; inspect with `vat state {vat_id}` and retry `vat compose down`"
                );
            }
            if compose_services_are_terminal(vat.meta.test_run.as_ref(), service_ids) {
                return Ok(());
            }
        }
        if Instant::now() >= deadline {
            bail!(
                "compose stop request for VAT `{vat_id}` was not acknowledged within {}s; registry retained to avoid overlapping services; inspect with `vat state {vat_id}` and retry `vat compose down`",
                timeout.as_secs()
            );
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn compose_services_are_terminal(
    test_run: Option<&TestRunEvidence>,
    service_ids: &[String],
) -> bool {
    let Some(test_run) = test_run else {
        return true;
    };
    service_ids.iter().all(|service_id| {
        test_run
            .services
            .iter()
            .find(|service| service.id == *service_id)
            .map(|service| {
                service.owned_by_vat == Some(false)
                    || (matches!(
                        service.status,
                        ProcessStatus::Interrupted
                            | ProcessStatus::Exited
                            | ProcessStatus::Failed
                            | ProcessStatus::Timeout
                    ) && service.cleanup_error.is_none())
            })
            // An exited VAT that never recorded this service did not have a
            // live process for it in the first place.
            .unwrap_or(true)
    })
}

fn detached_startup_from_evidence(
    service_ids: &[String],
    test_run: Option<&TestRunEvidence>,
) -> DetachedStartup {
    let Some(test_run) = test_run else {
        return DetachedStartup::Starting;
    };

    if let Some(message) = compose_cleanup_error(Some(test_run), service_ids) {
        return DetachedStartup::CleanupUnconfirmed(message);
    }

    for service_id in service_ids {
        if let Some(service) = test_run
            .services
            .iter()
            .find(|service| service.id == *service_id)
        {
            match service.status {
                ProcessStatus::Interrupted => {
                    return DetachedStartup::Terminal(format!(
                        "service `{service_id}` was interrupted before compose startup completed"
                    ));
                }
                ProcessStatus::Failed | ProcessStatus::Timeout => {
                    let detail = service
                        .readiness_error
                        .as_deref()
                        .unwrap_or("service failed before becoming ready");
                    return DetachedStartup::Terminal(format!(
                        "service `{service_id}` is {:?}: {detail}",
                        service.status
                    ));
                }
                ProcessStatus::Exited => {
                    return DetachedStartup::Terminal(format!(
                        "service `{service_id}` exited before compose startup completed"
                    ));
                }
                ProcessStatus::Created | ProcessStatus::Running | ProcessStatus::Ready => {}
            }
        }
    }

    if let Some(runner) = test_run
        .runner
        .iter()
        .chain(test_run.runners.iter())
        .find(|runner| {
            matches!(
                runner.status,
                ProcessStatus::Interrupted
                    | ProcessStatus::Exited
                    | ProcessStatus::Failed
                    | ProcessStatus::Timeout
            )
        })
    {
        return DetachedStartup::Terminal(format!(
            "runner `{}` is {:?} before compose startup completed",
            runner.id, runner.status
        ));
    }

    let runner_is_live = test_run
        .runner
        .iter()
        .chain(test_run.runners.iter())
        .any(|runner| {
            runner.id == RUNNER_ID
                && runner.status == ProcessStatus::Running
                && runner.pid.is_some()
        });

    if runner_is_live && all_registered_services_are_uniquely_ready(test_run, service_ids) {
        return DetachedStartup::Ready;
    }

    DetachedStartup::Starting
}

/// Reconcile a VAT that has not yet reached its durable terminal status. An
/// exited runner is not enough to release compose ownership: run_configured
/// persists that runner record before it tears down services. While teardown
/// is in flight, return `Stopping` so ps/down/up retain the binding and cannot
/// start a second host-port owner.
fn detached_startup_while_active(
    service_ids: &[String],
    test_run: Option<&TestRunEvidence>,
) -> DetachedStartup {
    let Some(test_run) = test_run else {
        return DetachedStartup::Starting;
    };

    let runner_is_live = test_run
        .runner
        .iter()
        .chain(test_run.runners.iter())
        .any(|runner| {
            runner.id == RUNNER_ID
                && runner.status == ProcessStatus::Running
                && runner.pid.is_some()
        });
    if runner_is_live && all_registered_services_are_uniquely_ready(test_run, service_ids) {
        return DetachedStartup::Ready;
    }

    let runner_is_terminal = test_run
        .runner
        .iter()
        .chain(test_run.runners.iter())
        .any(|runner| {
            matches!(
                runner.status,
                ProcessStatus::Interrupted
                    | ProcessStatus::Exited
                    | ProcessStatus::Failed
                    | ProcessStatus::Timeout
            )
        });
    let service_is_terminal = service_ids.iter().any(|service_id| {
        test_run
            .services
            .iter()
            .find(|service| service.id == *service_id)
            .is_some_and(|service| {
                matches!(
                    service.status,
                    ProcessStatus::Interrupted
                        | ProcessStatus::Exited
                        | ProcessStatus::Failed
                        | ProcessStatus::Timeout
                )
            })
    });
    if runner_is_terminal || service_is_terminal {
        DetachedStartup::Stopping
    } else {
        DetachedStartup::Starting
    }
}

/// `Ready` is an ownership claim, not a best-effort summary. A duplicated
/// service id can contain contradictory lifecycle records, so both compose
/// reconciliation paths must use the same uniqueness rule as the public ps
/// projection before any exec caller can reach a MicroVM spawn.
fn all_registered_services_are_uniquely_ready(
    test_run: &TestRunEvidence,
    service_ids: &[String],
) -> bool {
    !service_ids.is_empty()
        && service_ids.iter().all(|service_id| {
            unique_service_evidence(test_run, service_id)
                .is_some_and(|service| service.status == ProcessStatus::Ready)
        })
}

fn compose_cleanup_error(
    test_run: Option<&TestRunEvidence>,
    service_ids: &[String],
) -> Option<String> {
    let test_run = test_run?;
    service_ids.iter().find_map(|service_id| {
        let service = test_run
            .services
            .iter()
            .find(|service| service.id == *service_id)?;
        let error = service.cleanup_error.as_deref()?;
        Some(format!("service `{service_id}`: {error}"))
    })
}

fn compose_terminal_startup_error(
    project_name: &str,
    vat_id: Option<&str>,
    message: &str,
) -> anyhow::Error {
    let state = vat_id
        .map(|id| format!("vat state {id}"))
        .unwrap_or_else(|| "the detached vat could not be identified".to_string());
    anyhow::anyhow!(
        "compose project `{project_name}` startup failed: {message}; registry reset to imported; diagnose with `{state}`"
    )
}

fn compose_cleanup_unconfirmed_error(
    project_name: &str,
    vat_id: Option<&str>,
    message: &str,
) -> anyhow::Error {
    let state = vat_id
        .map(|id| format!("vat state {id}"))
        .unwrap_or_else(|| "the retained VAT state".to_string());
    anyhow::anyhow!(
        "compose project `{project_name}` cleanup is unconfirmed: {message}; registry retained to prevent published-port reuse; inspect with `{state}`, repair the runtime resource, then retry `vat compose down {project_name}`"
    )
}

/// Get or create the registry directory for a project.
fn registry_dir_for_project(project: &str) -> Result<PathBuf> {
    let root = crate::paths::root()?;
    let dir = root.join("compose").join(project);
    Ok(dir)
}

/// Read the compose registry entry for a project.
fn read_registry(registry_dir: &Path) -> Result<ComposeRecord> {
    let path = registry_dir.join("project.json");
    let content = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let record =
        serde_json::from_str(&content).with_context(|| format!("parse {}", path.display()))?;
    Ok(record)
}

/// Load the registry that commits an import, then prove it tracks the current
/// generated config's service ownership before any detached handoff can run.
/// A prior process may have crashed after atomically replacing vat.toml but
/// before it atomically replaced project.json; treating that split state as a
/// successful import could leave a newly added service untracked at teardown.
fn load_and_validate_registry(
    registry_dir: &Path,
    project_name: &str,
    vat_toml: &Path,
) -> Result<ComposeRecord> {
    let record = read_registry(registry_dir).with_context(|| {
        format!(
            "compose project `{project_name}` has vat.toml but no committed registry; re-import with `vat compose import <compose-file> --project {project_name}`"
        )
    })?;
    if record.project != project_name {
        bail!(
            "compose project `{project_name}` registry belongs to `{}`; re-import with `vat compose import <compose-file> --project {project_name}`",
            record.project
        );
    }
    // A bound record describes a VAT that was already launched from an earlier
    // config. Its current vat.toml may legitimately have been edited since
    // launch; reconciliation must use the durable registry and VAT evidence,
    // never block cleanup on a later config edit. Validate only immediately
    // before a new imported project could create a fresh runtime service set.
    if record.vat_id.is_some() || record.status != "imported" {
        return Ok(record);
    }
    // An invalid or unreadable config cannot start a service: `vat run` will
    // reject it before preparation. Preserve that established lifecycle error
    // path instead of masking it with ownership bookkeeping. A parseable
    // replacement config, however, must agree on its service identity set.
    if let Ok(actual_service_ids) = compose_service_ids(vat_toml) {
        if !compose_service_ids_match(&record.service_ids, &actual_service_ids) {
            bail!(
                "compose project `{project_name}` registry/config mismatch: project.json tracks {:?}, but vat.toml declares {:?}; refuse to launch because cleanup ownership would be incomplete. Re-import with `vat compose import <compose-file> --project {project_name}`",
                record.service_ids,
                actual_service_ids,
            );
        }
    }
    Ok(record)
}

fn compose_service_ids(vat_toml: &Path) -> Result<Vec<String>> {
    // This gate proves registry ownership, not full runner readiness. Keep
    // compose import's established behavior: it may materialize a bounded
    // image service before the user fills in a required runtime detail such as
    // container_port. `vat run` performs the complete config validation before
    // it can launch anything.
    let content = fs::read_to_string(vat_toml)
        .with_context(|| format!("read materialized config {}", vat_toml.display()))?;
    let config: crate::config::VatConfig = toml::from_str(&content)
        .with_context(|| format!("parse materialized config {}", vat_toml.display()))?;
    Ok(config
        .services
        .into_iter()
        .map(|service| service.id)
        .collect())
}

/// Service declaration order is not lifecycle ownership. Users may edit the
/// generated vat.toml, including reordering its service tables, so validate
/// the exact identity set rather than a serialization-order artifact.
fn compose_service_ids_match(recorded: &[String], actual: &[String]) -> bool {
    let mut recorded = recorded.to_vec();
    let mut actual = actual.to_vec();
    recorded.sort_unstable();
    actual.sort_unstable();
    recorded == actual
}

fn rollback_failed_import(
    vat_toml: &Path,
    previous_vat_toml: Option<&[u8]>,
    stage: &str,
    error: anyhow::Error,
) -> anyhow::Error {
    match crate::compose::restore_materialized_config(vat_toml, previous_vat_toml) {
        Ok(()) => error.context(format!(
            "{stage} failed; restored the previous materialized vat.toml"
        )),
        Err(rollback_error) => anyhow::anyhow!(
            "{stage} failed: {error}; also failed to restore the previous materialized vat.toml: {rollback_error}. The registry/config gate will refuse a later compose up; re-import before retrying."
        ),
    }
}

/// Write the compose registry entry for a project.
fn write_registry(registry_dir: &Path, record: &ComposeRecord) -> Result<()> {
    fs::create_dir_all(registry_dir)?;
    let path = registry_dir.join("project.json");
    let json = serde_json::to_string_pretty(record)?;
    // Registry readers run in other compose processes. Write to a unique
    // sibling and atomically rename it into place so readers observe either a
    // complete old JSON record or a complete new one, never a truncation.
    let temporary = registry_dir.join(format!(".project.{}.json.tmp", crate::id::fresh()));
    let write_result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .with_context(|| format!("create compose registry temp {}", temporary.display()))?;
        file.write_all(json.as_bytes())
            .with_context(|| format!("write compose registry temp {}", temporary.display()))?;
        file.sync_all()
            .with_context(|| format!("sync compose registry temp {}", temporary.display()))?;
        drop(file);
        fs::rename(&temporary, &path).with_context(|| {
            format!(
                "replace compose registry {} from {}",
                path.display(),
                temporary.display()
            )
        })?;
        Ok(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    write_result?;
    Ok(())
}

/// Decode an optional detached-child handoff. Both variables must be present:
/// accepting only one would turn malformed inherited environment into an
/// uncorrelated VAT run.
pub(crate) fn compose_handoff_from_env() -> Result<Option<ComposeHandoff>> {
    match (
        std::env::var("VAT_COMPOSE_PROJECT").ok(),
        std::env::var("VAT_COMPOSE_STARTUP_TOKEN").ok(),
    ) {
        (None, None) => Ok(None),
        (Some(project), Some(token)) => ComposeHandoff::new(project, token).map(Some),
        _ => bail!(
            "detached compose launcher must provide both VAT_COMPOSE_PROJECT and VAT_COMPOSE_STARTUP_TOKEN"
        ),
    }
}

/// Record the token owner's PID before it loads configuration or clones a
/// workspace. The explicit foreground path and a detached re-exec child use
/// this same operation; both fail before VAT creation if a newer lifecycle
/// reclaimed the registry.
pub(crate) fn register_compose_handoff(handoff: &ComposeHandoff) -> Result<bool> {
    let registry_dir = registry_dir_for_project(&handoff.project)?;
    // The detached parent intentionally holds this claim across spawn and its
    // initial PID write. Blocking here serializes the child handoff instead of
    // letting either side overwrite the other's JSON transition.
    let _claim = StartupClaim::acquire_blocking(&registry_dir, &handoff.project)?;
    let mut record = read_registry(&registry_dir)?;
    if record.project == handoff.project
        && record.status == "starting"
        && record.vat_id.is_none()
        && record.startup_token.as_deref() == Some(handoff.token.as_str())
    {
        record.startup_pid = Some(std::process::id());
        write_registry(&registry_dir, &record)?;
        return Ok(true);
    }
    Ok(false)
}

/// Let the token owner publish its VAT id immediately after durable VAT
/// creation. This is the sole path that can set `ComposeRecord.vat_id` during
/// startup; neither foreground nor detached parents infer it from VAT names.
pub(crate) fn publish_compose_handoff(handoff: &ComposeHandoff, vat_id: &str) -> Result<()> {
    let registry_dir = registry_dir_for_project(&handoff.project)?;
    let _claim = StartupClaim::acquire_blocking(&registry_dir, &handoff.project)?;
    let mut record = read_registry(&registry_dir).with_context(|| {
        format!(
            "read compose registry for project `{}` while publishing VAT `{vat_id}`",
            handoff.project
        )
    })?;
    // Publishing the same ID is idempotent. Any other mismatch is an ownership
    // loss, not a harmless no-op: continuing would create an untracked live
    // service set after a newer lifecycle reclaimed the project.
    if record.project == handoff.project
        && record.vat_id.as_deref() == Some(vat_id)
        && record.startup_token.is_none()
    {
        return Ok(());
    }
    if record.project != handoff.project
        || record.status != "starting"
        || record.vat_id.is_some()
        || record.startup_token.as_deref() != Some(handoff.token.as_str())
    {
        bail!(
            "compose startup for `{}` lost token ownership before publishing VAT `{vat_id}`",
            handoff.project
        );
    }
    record.vat_id = Some(vat_id.to_string());
    record.startup_pid = None;
    record.startup_token = None;
    record.startup_started_at = None;
    write_registry(&registry_dir, &record)
}

/// Clear only the active run binding. Keeping imported service metadata makes
/// a project immediately reusable after `down` or a terminal startup failure.
fn reset_active_run(registry_dir: &Path, record: &mut ComposeRecord) -> Result<()> {
    record.vat_id = None;
    record.startup_pid = None;
    record.startup_token = None;
    record.startup_started_at = None;
    // Preserve the counter for the next detached launch, but invalidate every
    // captured wait target as soon as this lifecycle is terminal/down.
    record.launch_ticket = None;
    record.status = "imported".to_string();
    write_registry(registry_dir, record)
}

/// Sanitize a project name (simple alphanumeric + dash/underscore).
fn sanitize_project_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::RetentionPolicy;
    use crate::state::{ConfigRef, RunnerRunRecord, ServiceRunRecord};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    fn service(id: &str, status: ProcessStatus, readiness_error: Option<&str>) -> ServiceRunRecord {
        ServiceRunRecord {
            id: id.to_string(),
            command: Vec::new(),
            status,
            preset: None,
            host: Some("127.0.0.1".to_string()),
            port: Some(8080),
            owned_by_vat: Some(true),
            prepare_mode: Some("container_run".to_string()),
            cache_key: None,
            prepare_duration_ms: Some(0),
            ready_duration_ms: None,
            exported_env: Vec::new(),
            pid: None,
            exit_code: None,
            ready_http: None,
            docker_name: None,
            microvm_name: None,
            readiness_error: readiness_error.map(str::to_string),
            cleanup_error: None,
            cluster: None,
            stdout_log: String::new(),
            stderr_log: String::new(),
        }
    }

    fn evidence(services: Vec<ServiceRunRecord>, runner: Option<ProcessStatus>) -> TestRunEvidence {
        TestRunEvidence {
            config: ConfigRef {
                path: "vat.toml".to_string(),
                digest: "test".to_string(),
            },
            runner_id: RUNNER_ID.to_string(),
            retention: RetentionPolicy::Always,
            services,
            scenario: None,
            runner: runner.map(|status| RunnerRunRecord {
                id: RUNNER_ID.to_string(),
                command: Vec::new(),
                status,
                exit_code: None,
                duration_ms: None,
                pid: (status == ProcessStatus::Running).then_some(42),
                cleanup_error: None,
                stdout_log: String::new(),
                stderr_log: String::new(),
            }),
            runners: Vec::new(),
            artifacts: Vec::new(),
            plan: None,
            topology: None,
        }
    }

    #[test]
    fn bounded_log_stream_keeps_agent_snapshots_line_and_serialized_json_bounded() {
        let temp = tempfile::tempdir().expect("log tempdir");
        let line_tail = temp.path().join("line-tail.log");
        fs::write(&line_tail, "one\ntwo\nthree\n").expect("write line tail");
        let snapshot = bounded_log_stream(line_tail.to_str().expect("UTF-8 log path"), 2)
            .expect("read line tail");
        assert_eq!(snapshot.text, "two\nthree");
        assert!(snapshot.truncated);
        assert!(!snapshot.utf8_lossy);

        let byte_tail = temp.path().join("byte-tail.log");
        let mut bytes = vec![b'x'; MAX_DOCKER_SHIM_STREAM_CAPTURE_BYTES as usize + 32];
        bytes.extend_from_slice(b"\nlast-line\n");
        fs::write(&byte_tail, bytes).expect("write byte-bounded log");
        let snapshot = bounded_log_stream(byte_tail.to_str().expect("UTF-8 log path"), 2)
            .expect("read byte tail");
        assert!(snapshot.truncated);
        assert!(snapshot.text.ends_with("last-line"));
        assert!(
            serialized_json_string_len(&snapshot.text).expect("serialize bounded text")
                <= MAX_DOCKER_SHIM_JSON_STREAM_VALUE_BYTES,
            "snapshot JSON stream value must remain within its public byte budget"
        );

        let non_utf8 = temp.path().join("non-utf8.log");
        fs::write(
            &non_utf8,
            vec![0xff; MAX_DOCKER_SHIM_STREAM_CAPTURE_BYTES as usize + 16],
        )
        .expect("write non-UTF-8 log");
        let snapshot = bounded_log_stream(
            non_utf8.to_str().expect("UTF-8 log path"),
            MAX_DOCKER_SHIM_LOG_TAIL_LINES,
        )
        .expect("read non-UTF-8 log");
        assert!(snapshot.utf8_lossy);
        assert!(snapshot.text.contains('\u{fffd}'));
        assert!(snapshot.truncated);
        assert!(
            serialized_json_string_len(&snapshot.text).expect("serialize lossy text")
                <= MAX_DOCKER_SHIM_JSON_STREAM_VALUE_BYTES,
            "lossy replacement expansion must still fit the JSON stream budget"
        );

        let controls = temp.path().join("controls.log");
        fs::write(
            &controls,
            vec![0; MAX_DOCKER_SHIM_STREAM_CAPTURE_BYTES as usize + 16],
        )
        .expect("write control-character log");
        let snapshot = bounded_log_stream(
            controls.to_str().expect("UTF-8 log path"),
            MAX_DOCKER_SHIM_LOG_TAIL_LINES,
        )
        .expect("read control-character log");
        assert!(!snapshot.utf8_lossy);
        assert!(snapshot.truncated);
        assert!(
            serialized_json_string_len(&snapshot.text).expect("serialize control text")
                <= MAX_DOCKER_SHIM_JSON_STREAM_VALUE_BYTES,
            "JSON control escaping must still fit the stream budget"
        );

        let mut exec_bytes = vec![0xff; MAX_DOCKER_SHIM_STREAM_CAPTURE_BYTES as usize + 16];
        exec_bytes.extend_from_slice(&[0, b'\n', b'e', b'n', b'd']);
        let snapshot = bounded_exec_stream(std::io::Cursor::new(exec_bytes))
            .expect("drain bounded exec stream");
        assert!(snapshot.truncated);
        assert!(snapshot.utf8_lossy);
        assert!(snapshot.text.ends_with("\nend"));
        assert!(
            serialized_json_string_len(&snapshot.text).expect("serialize bounded exec text")
                <= MAX_DOCKER_SHIM_JSON_STREAM_VALUE_BYTES,
            "exec lossy/control expansion must still fit the JSON stream budget"
        );
    }

    #[test]
    fn detached_startup_waits_for_all_services_and_surfaces_terminal_evidence() {
        let ids = vec!["web".to_string(), "db".to_string()];
        assert_eq!(
            detached_startup_from_evidence(
                &ids,
                Some(&evidence(
                    vec![
                        service("web", ProcessStatus::Ready, None),
                        service("db", ProcessStatus::Running, None),
                    ],
                    Some(ProcessStatus::Running),
                )),
            ),
            DetachedStartup::Starting
        );

        assert_eq!(
            detached_startup_from_evidence(
                &ids,
                Some(&evidence(
                    vec![
                        service("web", ProcessStatus::Ready, None),
                        service("db", ProcessStatus::Ready, None),
                    ],
                    Some(ProcessStatus::Running),
                )),
            ),
            DetachedStartup::Ready
        );

        // Contradictory duplicate evidence must never turn into a readiness
        // proof for `compose exec`: ps already treats duplicate IDs as
        // unproven, and both active/exited reconciliation paths must agree.
        let duplicate_same_id = evidence(
            vec![
                service("web", ProcessStatus::Ready, None),
                service(
                    "web",
                    ProcessStatus::Failed,
                    Some("conflicting duplicate evidence"),
                ),
                service("db", ProcessStatus::Ready, None),
            ],
            Some(ProcessStatus::Running),
        );
        assert_eq!(
            detached_startup_from_evidence(&ids, Some(&duplicate_same_id)),
            DetachedStartup::Starting,
            "duplicate service evidence must reject the exited-run readiness proof"
        );
        assert_eq!(
            detached_startup_while_active(&ids, Some(&duplicate_same_id)),
            DetachedStartup::Starting,
            "duplicate service evidence must reject the active-run readiness proof before exec can spawn"
        );

        assert_eq!(
            detached_startup_from_evidence(
                &ids,
                Some(&evidence(
                    vec![
                        service("web", ProcessStatus::Ready, None),
                        service("db", ProcessStatus::Ready, None),
                    ],
                    None,
                )),
            ),
            DetachedStartup::Starting
        );

        let state = detached_startup_from_evidence(
            &ids,
            Some(&evidence(
                vec![
                    service("web", ProcessStatus::Failed, Some("host endpoint reset")),
                    service("db", ProcessStatus::Ready, None),
                ],
                Some(ProcessStatus::Failed),
            )),
        );
        assert!(matches!(
            state,
            DetachedStartup::Terminal(message)
                if message.contains("web") && message.contains("host endpoint reset")
        ));
    }

    #[test]
    fn detached_startup_treats_terminal_runner_before_readiness_as_failure() {
        let ids = vec!["web".to_string()];
        let state = detached_startup_from_evidence(
            &ids,
            Some(&evidence(
                vec![service("web", ProcessStatus::Running, None)],
                Some(ProcessStatus::Failed),
            )),
        );
        assert!(matches!(
            state,
            DetachedStartup::Terminal(message) if message.contains("project.up")
        ));

        assert_eq!(
            vat_terminal_state_label(&Status::Interrupted {
                signal: libc::SIGTERM,
                reason: "received SIGTERM (15)".to_string(),
            }),
            "Interrupted"
        );
        assert_eq!(
            vat_terminal_state_label(&Status::Exited { code: 0 }),
            "Exited"
        );
    }

    #[test]
    fn cleanup_unconfirmed_blocks_compose_reuse_until_retry_succeeds() {
        let ids = vec!["web".to_string()];
        let mut run = evidence(
            vec![service(
                "web",
                ProcessStatus::Exited,
                Some("endpoint reset"),
            )],
            Some(ProcessStatus::Exited),
        );
        run.services[0].cleanup_error = Some("container rm -f web timed out".to_string());

        assert!(matches!(
            detached_startup_from_evidence(&ids, Some(&run)),
            DetachedStartup::CleanupUnconfirmed(message) if message.contains("container rm")
        ));
        assert!(!compose_services_are_terminal(Some(&run), &ids));

        run.services[0].cleanup_error = None;
        assert!(compose_services_are_terminal(Some(&run), &ids));
    }

    fn docker_shim_record(vat_id: &str, service_ids: &[&str]) -> ComposeRecord {
        ComposeRecord {
            project: "agent-tools".to_string(),
            vat_id: Some(vat_id.to_string()),
            docker_shim_profile: Some("host-facing-independent-v1".to_string()),
            launch_generation: 1,
            launch_ticket: Some("test-launch-ticket".to_string()),
            handoff_protocol: HANDOFF_PROTOCOL,
            startup_pid: None,
            startup_token: None,
            startup_started_at: None,
            service_ids: service_ids.iter().map(|id| (*id).to_string()).collect(),
            status: "ready".to_string(),
            created_at: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn docker_shim_wait_target_rejects_down_reimport_and_relaunch_replacements() {
        let record = docker_shim_record("vat-generation", &["docs"]);
        let target = docker_shim_wait_target_from_record(
            &record,
            crate::compose::DockerComposeProfile::HostFacingIndependentV1,
        )
        .expect("active shim record has a target");
        assert!(docker_shim_wait_target_matches(
            "agent-tools",
            &record,
            &target
        ));

        // `down` resets active state and clears the durable ticket. A waiter
        // must not observe the imported record as its original launch.
        let mut down = record.clone();
        down.launch_ticket = None;
        down.vat_id = None;
        down.status = "imported".to_string();
        assert!(!docker_shim_wait_target_matches(
            "agent-tools",
            &down,
            &target
        ));

        // Ordinary re-import clears shim provenance even if it retains the
        // counter; it is necessarily a replacement, never an adoption.
        let mut generic_reimport = down.clone();
        generic_reimport.docker_shim_profile = None;
        assert!(!docker_shim_wait_target_matches(
            "agent-tools",
            &generic_reimport,
            &target
        ));

        // A fresh Docker-shaped launch advances both the generation boundary
        // and its ticket, so even a same-profile relaunch cannot satisfy an
        // old wait target.
        let mut relaunch = record.clone();
        relaunch.launch_generation += 1;
        relaunch.launch_ticket = Some("fresh-launch-ticket".to_string());
        assert!(!docker_shim_wait_target_matches(
            "agent-tools",
            &relaunch,
            &target
        ));
    }

    fn ready_microvm_service(vat_id: &str, id: &str, port: u16) -> ServiceRunRecord {
        let mut record = service(id, ProcessStatus::Ready, None);
        record.port = Some(port);
        record.microvm_name = Some(compose_microvm_name(vat_id, id));
        record
    }

    #[test]
    fn duplicate_service_evidence_cannot_select_a_compose_exec_microvm() {
        let vat_id = "vat-duplicate-evidence";
        let run = evidence(
            vec![
                ready_microvm_service(vat_id, "docs", 18080),
                service(
                    "docs",
                    ProcessStatus::Failed,
                    Some("conflicting duplicate evidence"),
                ),
            ],
            Some(ProcessStatus::Running),
        );
        let error = ready_microvm_name_from_evidence(vat_id, "agent-tools", "docs", &run)
            .expect_err("duplicate service ids must reject exec ownership before spawn");
        assert!(
            error
                .to_string()
                .contains("requires exactly one evidence record"),
            "duplicate exec evidence error: {error:#}"
        );
    }

    #[test]
    fn reconciled_exec_proof_uses_one_ready_evidence_snapshot() {
        let vat_id = "vat-one-snapshot";
        let ready_snapshot = ReconciledStartupEvidence {
            state: DetachedStartup::Ready,
            test_run: Some(evidence(
                vec![ready_microvm_service(vat_id, "docs", 18080)],
                Some(ProcessStatus::Running),
            )),
        };
        assert_eq!(
            ready_microvm_name_from_reconciled_evidence(
                vat_id,
                "agent-tools",
                "docs",
                &ready_snapshot,
            )
            .expect("same ready snapshot must select its exact MicroVM"),
            compose_microvm_name(vat_id, "docs")
        );

        let missing_evidence = ReconciledStartupEvidence {
            state: DetachedStartup::Ready,
            test_run: None,
        };
        let error = ready_microvm_name_from_reconciled_evidence(
            vat_id,
            "agent-tools",
            "docs",
            &missing_evidence,
        )
        .expect_err("ready state without that same snapshot's runner evidence must fail closed");
        assert!(
            error.to_string().contains("has no runner evidence"),
            "missing evidence error: {error:#}"
        );

        let mut later_revision_service = ready_microvm_service(vat_id, "docs", 18080);
        later_revision_service.microvm_name = Some("later-revision-name".to_string());
        let later_revision = ReconciledStartupEvidence {
            state: DetachedStartup::Ready,
            test_run: Some(evidence(
                vec![later_revision_service],
                Some(ProcessStatus::Running),
            )),
        };
        assert!(
            ready_microvm_name_from_reconciled_evidence(
                vat_id,
                "agent-tools",
                "docs",
                &later_revision,
            )
            .is_err(),
            "a different evidence revision cannot be paired with the earlier Ready proof"
        );
    }

    fn topology_snapshot(
        record: &ComposeRecord,
        phase: DockerShimTopologyPhase,
        services: Vec<ServiceRunRecord>,
    ) -> DockerShimPsSnapshot {
        ComposePsSnapshot::from_record(
            record,
            phase,
            Some(evidence(services, Some(ProcessStatus::Running))),
        )
        .into_docker_shim_snapshot()
        .expect("known Docker shim record must project topology")
    }

    #[test]
    fn docker_shim_topology_orders_recorded_services_and_proves_exact_loopback_endpoints() {
        let vat_id = "vat-topology";
        let record = docker_shim_record(vat_id, &["docs", "inspector"]);
        // VAT evidence order is not a public ordering contract. The Docker
        // topology must instead follow the imported registry's service IDs.
        let snapshot = topology_snapshot(
            &record,
            DockerShimTopologyPhase::Ready,
            vec![
                ready_microvm_service(vat_id, "inspector", 18081),
                ready_microvm_service(vat_id, "docs", 18080),
            ],
        );

        assert_eq!(snapshot.topology.phase, DockerShimTopologyPhase::Ready);
        assert!(snapshot.topology.ready);
        assert_eq!(
            snapshot.topology.services,
            vec![
                DockerShimTopologyService {
                    name: "docs".to_string(),
                    state: DockerShimTopologyServiceState::Ready,
                    endpoint: Some("127.0.0.1:18080".to_string()),
                },
                DockerShimTopologyService {
                    name: "inspector".to_string(),
                    state: DockerShimTopologyServiceState::Ready,
                    endpoint: Some("127.0.0.1:18081".to_string()),
                },
            ]
        );
    }

    #[test]
    fn docker_shim_topology_fails_closed_without_complete_endpoint_proof() {
        let vat_id = "vat-topology";
        let record = docker_shim_record(vat_id, &["docs"]);
        let valid = ready_microvm_service(vat_id, "docs", 18080);

        // A lifecycle that is not globally ready must never disclose a host
        // endpoint even if its last service evidence happens to be Ready.
        for phase in [
            DockerShimTopologyPhase::Inactive,
            DockerShimTopologyPhase::Starting,
            DockerShimTopologyPhase::Stopping,
        ] {
            let snapshot = topology_snapshot(&record, phase, vec![valid.clone()]);
            assert_eq!(snapshot.topology.phase, phase);
            assert!(!snapshot.topology.ready);
            assert!(snapshot
                .topology
                .services
                .iter()
                .all(|service| service.endpoint.is_none()));
        }

        let cases: &[(&str, fn(&mut ServiceRunRecord))] = &[
            ("not VAT-owned", |service| {
                service.owned_by_vat = Some(false)
            }),
            ("wrong prepare mode", |service| {
                service.prepare_mode = Some("docker_run".to_string())
            }),
            ("non-loopback host", |service| {
                service.host = Some("0.0.0.0".to_string())
            }),
            ("zero port", |service| service.port = Some(0)),
            ("cleanup uncertainty", |service| {
                service.cleanup_error = Some("container cleanup timed out".to_string())
            }),
            ("wrong exact MicroVM name", |service| {
                service.microvm_name = Some("not-vat-owned".to_string())
            }),
            ("not ready", |service| {
                service.status = ProcessStatus::Running
            }),
        ];
        for (label, mutate) in cases {
            let mut invalid = valid.clone();
            mutate(&mut invalid);
            let snapshot =
                topology_snapshot(&record, DockerShimTopologyPhase::Ready, vec![invalid]);
            assert_eq!(
                snapshot.topology.phase,
                DockerShimTopologyPhase::Degraded,
                "{label} endpoint evidence must degrade topology"
            );
            assert!(
                !snapshot.topology.ready,
                "{label} endpoint evidence must not claim agent readiness"
            );
            assert!(snapshot
                .topology
                .services
                .iter()
                .all(|service| service.endpoint.is_none()));
        }

        let duplicate = topology_snapshot(
            &record,
            DockerShimTopologyPhase::Ready,
            vec![valid.clone(), valid],
        );
        assert_eq!(
            duplicate.topology.phase,
            DockerShimTopologyPhase::Degraded,
            "duplicate service evidence is not a unique ownership proof"
        );
        assert!(!duplicate.topology.ready);
        assert!(duplicate
            .topology
            .services
            .iter()
            .all(|service| service.endpoint.is_none()));
    }

    #[test]
    fn token_without_pid_is_reclaimed_only_after_handoff_grace() {
        let old = ComposeRecord {
            project: "example".to_string(),
            vat_id: None,
            docker_shim_profile: None,
            launch_generation: 0,
            launch_ticket: None,
            handoff_protocol: HANDOFF_PROTOCOL,
            startup_pid: None,
            startup_token: Some("old-token".to_string()),
            startup_started_at: Some((Utc::now() - chrono::Duration::seconds(10)).to_rfc3339()),
            service_ids: Vec::new(),
            status: "starting".to_string(),
            created_at: Utc::now().to_rfc3339(),
        };
        assert!(matches!(
            reconcile_detached_startup(&old).expect("reconcile old token"),
            DetachedStartup::Terminal(message) if message.contains("never published")
        ));

        let fresh = ComposeRecord {
            startup_token: Some("fresh-token".to_string()),
            startup_started_at: Some(Utc::now().to_rfc3339()),
            ..old
        };
        assert_eq!(
            reconcile_detached_startup(&fresh).expect("reconcile fresh token"),
            DetachedStartup::Starting
        );
    }

    #[test]
    fn atomic_registry_replacement_never_exposes_torn_json() {
        let temp = tempfile::tempdir().expect("registry tempdir");
        let mut record = ComposeRecord {
            project: "atomic".to_string(),
            vat_id: None,
            docker_shim_profile: None,
            launch_generation: 0,
            launch_ticket: None,
            handoff_protocol: HANDOFF_PROTOCOL,
            startup_pid: None,
            startup_token: None,
            startup_started_at: None,
            service_ids: vec!["web".to_string()],
            status: "imported".to_string(),
            created_at: Utc::now().to_rfc3339(),
        };
        write_registry(temp.path(), &record).expect("seed registry");

        let done = Arc::new(AtomicBool::new(false));
        let reader_done = Arc::clone(&done);
        let reader_dir = temp.path().to_path_buf();
        let reader = std::thread::spawn(move || {
            while !reader_done.load(Ordering::Acquire) {
                read_registry(&reader_dir)
                    .expect("reader must never observe partial registry JSON");
            }
        });

        for index in 0..128 {
            record.status = if index % 2 == 0 {
                "starting".to_string()
            } else {
                "imported".to_string()
            };
            write_registry(temp.path(), &record).expect("atomic registry replacement");
        }
        done.store(true, Ordering::Release);
        reader.join().expect("registry reader");
    }

    #[test]
    fn compose_access_requires_exact_registry_project_binding() {
        let record = ComposeRecord {
            project: "other-project".to_string(),
            vat_id: None,
            docker_shim_profile: Some("strict-single-image-v1".to_string()),
            launch_generation: 0,
            launch_ticket: None,
            handoff_protocol: HANDOFF_PROTOCOL,
            startup_pid: None,
            startup_token: None,
            startup_started_at: None,
            service_ids: vec!["web".to_string()],
            status: "imported".to_string(),
            created_at: Utc::now().to_rfc3339(),
        };
        let error =
            require_compose_access(&record, "expected-project", ComposeAccess::DockerShimPost)
                .expect_err("mismatched registry project must not be accessible");
        assert!(error
            .to_string()
            .contains("registry belongs to `other-project`"));
    }
}
// HANDWRITE-END

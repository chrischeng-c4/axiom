// SPEC-MANAGED: apps/vat/tech-design/semantic/source/projects-vat-src-commands-run-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `vat run` — direct command mode plus vat.toml runner mode.
//!
//! Direct mode (`vat run -- <cmd>`) preserves the original foreground behavior.
//! Runner mode (`vat run [runner-id]`) treats `vat.toml` as the project-local
//! agent test protocol: prepare a COW workspace, run setup, start run-scoped
//! services, wait for readiness, execute the runner, capture evidence, and
//! clean up services.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, ExitStatus, Stdio};
#[cfg(unix)]
use std::sync::atomic::{AtomicI32, Ordering};
#[cfg(unix)]
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use chrono::Utc;
#[cfg(unix)]
use signal_hook::{
    consts::signal::{SIGINT, SIGTERM},
    low_level::{register, unregister},
    SigId,
};
use walkdir::WalkDir;

use crate::cluster::{self, ClusterSpec, ResolvedBackend};
use crate::config::{
    self, ClusterBackend, PortSpec, RetentionPolicy, RunnerConfig, ScenarioConfig,
    ScenarioNetworkMode, ServiceConfig, ServicePreset, ServiceRuntime, VatConfig, VolumeMount,
};
use crate::event::{Event, EventKind};
use crate::gpu;
use crate::lumen_release;
use crate::overlay;
use crate::sandbox;
use crate::spec::{Base, EnvSpec, GpuRequest, Isolation};
use crate::state::{
    ArtifactRecord, ClusterRunRecord, ConfigRef, PlanEvidence, ProcessStatus, RouteRecord,
    RunRecord, RunnerRunRecord, ScenarioRunRecord, ServiceRunRecord, Status, TestRunEvidence,
    TopologyEvidence,
};
use crate::{id, store};

/// Inputs for `vat run`, already parsed by the CLI layer.
/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-commands-run-rs.md#source
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#cli
pub struct Args {
    pub target: Target,
    /// Clone from this host directory (default: current directory).
    pub base: Option<PathBuf>,
    /// Fork from an existing vat instead of a host directory.
    pub from: Option<String>,
    pub name: Option<String>,
    pub isolation: Isolation,
    pub gpu: GpuRequest,
    pub microvm_image: Option<String>,
    /// Direct mode prints full VatState JSON instead of a human summary.
    pub json: bool,
    /// Opaque upstream execution plan to copy into the vat and expose to the runner.
    pub plan: Option<PathBuf>,
    /// Per-invocation retention override for configured vat.toml runs.
    pub keep: Option<RetentionPolicy>,
    /// Internal compose-startup ownership proof. This is deliberately not a
    /// CLI flag: compose creates it before launching a runner, and only a
    /// matching token may publish the newly-created VAT id into its registry.
    pub(crate) compose_handoff: Option<crate::commands::compose::ComposeHandoff>,
}

/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#cli
pub enum Target {
    Direct {
        program: String,
        program_args: Vec<String>,
    },
    Runner {
        /// Empty = default selection; several = run CONCURRENTLY in one vat.
        runner_ids: Vec<String>,
    },
    Scenario {
        scenario_id: String,
    },
}

/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-commands-run-rs.md#source
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
fn gpu_satisfied(_gpu: GpuRequest, isolation: Isolation, info: &gpu::GpuInfo) -> bool {
    // GPU is only accessible in None and Seatbelt isolation modes.
    // MicroVm categorically cannot reach the host GPU.
    // _gpu is present for documentation (caller has already checked GpuRequest::Required).
    info.accessible && isolation != Isolation::MicroVm
}

#[derive(Debug, Clone)]
struct RunInterrupted {
    signal: i32,
    reason: String,
}

impl RunInterrupted {
    fn new(signal: i32) -> Self {
        Self {
            signal,
            reason: format!("received {} ({signal})", signal_name(signal)),
        }
    }

    fn exit_code(&self) -> i32 {
        128 + self.signal
    }
}

impl std::fmt::Display for RunInterrupted {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl std::error::Error for RunInterrupted {}

#[derive(Debug, Clone)]
struct RunCleanupFailed {
    interruption: RunInterrupted,
    cleanup_error: String,
}

impl std::fmt::Display for RunCleanupFailed {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}; owned cleanup remains unconfirmed: {}",
            self.interruption, self.cleanup_error
        )
    }
}

impl std::error::Error for RunCleanupFailed {}

#[derive(Debug)]
struct RunOwnedCleanupFailed {
    cleanup_error: String,
}

impl std::fmt::Display for RunOwnedCleanupFailed {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "VAT-owned helper cleanup remains unconfirmed: {}",
            self.cleanup_error
        )
    }
}

impl std::error::Error for RunOwnedCleanupFailed {}

fn signal_name(signal: i32) -> &'static str {
    match signal {
        libc::SIGINT => "SIGINT",
        libc::SIGTERM => "SIGTERM",
        _ => "signal",
    }
}

fn run_interruption(error: &anyhow::Error) -> Option<RunInterrupted> {
    error.downcast_ref::<RunInterrupted>().cloned()
}

fn run_cleanup_failure(error: &anyhow::Error) -> Option<RunCleanupFailed> {
    error.downcast_ref::<RunCleanupFailed>().cloned()
}

fn run_owned_cleanup_failure(error: &anyhow::Error) -> Option<&RunOwnedCleanupFailed> {
    error.downcast_ref::<RunOwnedCleanupFailed>()
}

fn configured_terminal_status(
    interruption: Option<&RunInterrupted>,
    cleanup_failed: bool,
    code: i32,
) -> Status {
    match interruption.filter(|_| !cleanup_failed) {
        Some(interruption) => Status::Interrupted {
            signal: interruption.signal,
            reason: interruption.reason.clone(),
        },
        None => Status::Exited { code },
    }
}

/// Scoped signal observer for one `vat run` invocation. The signal handler
/// records only the first SIGINT/SIGTERM in an atomic; the ordinary run thread
/// remains the sole owner of process cleanup and metadata persistence.
#[cfg(unix)]
struct RunCancellation {
    first_signal: Arc<AtomicI32>,
    registrations: Vec<SigId>,
}

#[cfg(unix)]
impl RunCancellation {
    fn new() -> Result<Self> {
        let first_signal = Arc::new(AtomicI32::new(0));
        let mut registrations = Vec::new();
        for signal in [SIGINT, SIGTERM] {
            let handler_signal = Arc::clone(&first_signal);
            let registration = unsafe {
                register(signal, move || {
                    let _ = handler_signal.compare_exchange(
                        0,
                        signal,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    );
                })
            };
            match registration {
                Ok(registration) => registrations.push(registration),
                Err(error) => {
                    for registration in registrations {
                        unregister(registration);
                    }
                    return Err(error).context("install scoped vat run cancellation handlers");
                }
            }
        }
        Ok(Self {
            first_signal,
            registrations,
        })
    }

    fn received(&self) -> Option<i32> {
        let signal = self.first_signal.load(Ordering::Acquire);
        (signal != 0).then_some(signal)
    }

    fn check(&self) -> Result<()> {
        match self.received() {
            Some(signal) => Err(RunInterrupted::new(signal).into()),
            None => Ok(()),
        }
    }

    #[cfg(test)]
    fn with_observed_signal(signal: i32) -> Self {
        Self {
            first_signal: Arc::new(AtomicI32::new(signal)),
            registrations: Vec::new(),
        }
    }
}

#[cfg(unix)]
impl Drop for RunCancellation {
    fn drop(&mut self) {
        for registration in self.registrations.drain(..) {
            unregister(registration);
        }
    }
}

#[cfg(not(unix))]
struct RunCancellation;

#[cfg(not(unix))]
impl RunCancellation {
    fn new() -> Result<Self> {
        Ok(Self)
    }

    fn received(&self) -> Option<i32> {
        None
    }

    fn check(&self) -> Result<()> {
        Ok(())
    }

    #[cfg(test)]
    fn with_observed_signal(_signal: i32) -> Self {
        Self
    }
}

/// Close the durable `Running` state when a fallible pre-child step fails.
/// Once `Running` has been published, sandbox selection, plan attachment,
/// compose handoff, and event logging are part of the run lifecycle even when
/// no owned child was spawned. They must never leave a ghost active VAT.
fn finish_pre_child_step<T>(
    vat: &mut store::Vat,
    cancellation: &RunCancellation,
    started: Instant,
    step: Result<T>,
    label: &str,
) -> Result<T> {
    match step {
        Ok(value) => Ok(value),
        Err(error) => {
            let interruption = cancellation.received().map(RunInterrupted::new);
            let code = interruption
                .as_ref()
                .map(RunInterrupted::exit_code)
                .unwrap_or(-1);
            vat.meta.status = configured_terminal_status(interruption.as_ref(), false, code);
            if let Some(run) = vat.meta.last_run.as_mut() {
                run.finished_at = Some(Utc::now());
                run.exit_code = Some(code);
                run.duration_ms = Some(started.elapsed().as_millis() as u64);
                run.signal = interruption.as_ref().map(|value| value.signal);
                run.owned_pgid = None;
                run.cleanup_error = None;
            }
            if let Err(persist_error) = vat.save() {
                return Err(error).context(format!(
                    "{label}; additionally failed to persist terminal pre-child state: {persist_error:#}"
                ));
            }
            match interruption {
                Some(interruption) => Err(anyhow::Error::new(interruption)).context(format!(
                    "{label}; the pre-child step also failed: {error:#}"
                )),
                None => Err(error).with_context(|| label.to_string()),
            }
        }
    }
}

/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-commands-run-rs.md#source
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
pub fn exec(args: Args) -> Result<ExitCode> {
    let cancellation = RunCancellation::new()?;
    let Args {
        target,
        base,
        from,
        name,
        isolation,
        gpu,
        microvm_image,
        json,
        plan,
        keep,
        compose_handoff,
    } = args;
    let compose_handoff = compose_handoff.or(crate::commands::compose::compose_handoff_from_env()?);
    // Register token ownership before parsing configuration or cloning a
    // workspace. A stale foreground handoff and a detached re-exec child share
    // this fail-closed path, so neither can create an untracked VAT.
    if let Some(handoff) = compose_handoff.as_ref() {
        if !crate::commands::compose::register_compose_handoff(handoff)
            .context("register compose launcher handoff")?
        {
            bail!(
                "compose launcher no longer owns its project registry; refusing to create an untracked VAT"
            );
        }
    }
    let result = match target {
        Target::Direct {
            program,
            program_args,
        } => {
            if compose_handoff.is_some() {
                bail!("compose handoff may only launch a configured runner");
            }
            exec_direct(
                DirectArgs {
                    program,
                    program_args,
                    base,
                    from,
                    name,
                    isolation,
                    gpu,
                    microvm_image,
                    json,
                    plan,
                },
                &cancellation,
            )
        }
        Target::Runner { runner_ids } => exec_runner(
            RunnerArgs {
                base,
                from,
                name,
                isolation,
                gpu,
                microvm_image,
                runner_ids,
                plan,
                keep,
                compose_handoff,
            },
            &cancellation,
        ),
        Target::Scenario { scenario_id } => {
            if compose_handoff.is_some() {
                bail!("compose handoff may only launch a configured runner");
            }
            exec_scenario(
                ScenarioArgs {
                    base,
                    from,
                    name,
                    isolation,
                    gpu,
                    microvm_image,
                    scenario_id,
                    plan,
                    keep,
                },
                &cancellation,
            )
        }
    };
    finish_exec_result(result)
}

fn finish_exec_result(result: Result<ExitCode>) -> Result<ExitCode> {
    match result {
        Err(error) if run_interruption(&error).is_some() => {
            let interruption = run_interruption(&error).expect("checked interruption");
            Ok(process_exit_code(interruption.exit_code()))
        }
        result => result,
    }
}

struct RunnerArgs {
    base: Option<PathBuf>,
    from: Option<String>,
    name: Option<String>,
    isolation: Isolation,
    gpu: GpuRequest,
    microvm_image: Option<String>,
    runner_ids: Vec<String>,
    plan: Option<PathBuf>,
    keep: Option<RetentionPolicy>,
    compose_handoff: Option<crate::commands::compose::ComposeHandoff>,
}

struct DirectArgs {
    program: String,
    program_args: Vec<String>,
    base: Option<PathBuf>,
    from: Option<String>,
    name: Option<String>,
    isolation: Isolation,
    gpu: GpuRequest,
    microvm_image: Option<String>,
    json: bool,
    plan: Option<PathBuf>,
}

struct ScenarioArgs {
    base: Option<PathBuf>,
    from: Option<String>,
    name: Option<String>,
    isolation: Isolation,
    gpu: GpuRequest,
    microvm_image: Option<String>,
    scenario_id: String,
    plan: Option<PathBuf>,
    keep: Option<RetentionPolicy>,
}

fn exec_direct(args: DirectArgs, cancellation: &RunCancellation) -> Result<ExitCode> {
    let gpu_info = gpu::detect();
    if args.gpu == GpuRequest::Required && !gpu_satisfied(args.gpu, args.isolation, &gpu_info) {
        bail!(
            "spec requires a GPU but {}",
            if args.isolation == Isolation::MicroVm {
                "GPU is categorically unreachable in an Apple Silicon microVM (Virtualization.framework constraint)".to_string()
            } else {
                format!("none is accessible on this host ({})", gpu_info.note)
            }
        );
    }

    let (source, spec_base, lineage): (PathBuf, Base, Vec<String>) = match &args.from {
        Some(parent_id) => {
            let parent = store::load(parent_id)
                .with_context(|| format!("--from refers to unknown vat {parent_id}"))?;
            let mut lineage = parent.meta.lineage.clone();
            lineage.push(parent.meta.id.clone());
            (parent.rootfs(), Base::Vat(parent.meta.id.clone()), lineage)
        }
        None => {
            let dir = match &args.base {
                Some(p) => p.clone(),
                None => std::env::current_dir().context("get current directory")?,
            };
            let canon = std::fs::canonicalize(&dir)
                .with_context(|| format!("resolve base dir {}", dir.display()))?;
            (canon.clone(), Base::Dir(canon), Vec::new())
        }
    };

    // Best-effort: a nearby vat.toml's `[network].egress` still applies to a
    // direct `vat run -- cmd` (the path that actually sandbox-wraps the command).
    let egress = std::env::current_dir()
        .ok()
        .and_then(|cwd| config::load_nearest(&cwd).ok())
        .and_then(|c| c.network)
        .map(|n| n.egress)
        .unwrap_or_default();
    let spec = EnvSpec {
        base: Some(spec_base),
        isolation: args.isolation,
        egress,
        gpu: args.gpu,
        microvm_image: args.microvm_image.clone(),
        ..EnvSpec::default()
    };

    let new_id = id::fresh();
    let mut vat = store::create(
        &new_id,
        args.name.clone(),
        spec.clone(),
        Some(&source),
        lineage,
    )
    .context("create vat")?;
    attach_plan_file(&mut vat, args.plan.as_deref())?;
    let spec = vat.meta.spec.clone();

    let command: Vec<String> = std::iter::once(args.program.clone())
        .chain(args.program_args.iter().cloned())
        .collect();
    let started = Instant::now();
    vat.meta.status = Status::Running;
    vat.meta.last_run = Some(RunRecord {
        command: command.clone(),
        started_at: Utc::now(),
        finished_at: None,
        exit_code: None,
        duration_ms: None,
        signal: None,
        owned_pgid: None,
        cleanup_error: None,
    });
    vat.save()?;
    let backend_step = sandbox::pick(&spec).map_err(anyhow::Error::msg);
    let backend = finish_pre_child_step(
        &mut vat,
        cancellation,
        started,
        backend_step,
        "select direct-run sandbox backend",
    )?;
    let log_step = vat.log(
        Event::new(EventKind::RunStarted, format!("run: {}", command.join(" ")))
            .with_data(serde_json::json!({ "backend": backend.name() })),
    );
    finish_pre_child_step(
        &mut vat,
        cancellation,
        started,
        log_step,
        "record direct-run start event",
    )?;

    let rootfs = vat.rootfs();
    let (prog, argv) = backend.resolve(&rootfs, &args.program, &args.program_args);
    let cwd = rootfs.join(&spec.workdir);
    let mut cmd = Command::new(&prog);
    cmd.args(&argv).current_dir(&cwd);
    for (key, value) in &spec.env {
        cmd.env(key, value);
    }
    set_process_group(&mut cmd);
    let mut owned_child = None;
    let outcome = (|| -> Result<ExitStatus> {
        cancellation.check()?;
        let child = cmd
            .spawn()
            .with_context(|| format!("spawn `{prog}` inside vat rootfs"))?;
        owned_child = Some(OwnedProcessGroup::new(child));
        wait_owned_process(
            owned_child.as_mut().expect("stored direct child"),
            "direct vat run",
            cancellation,
        )
    })();
    let duration_ms = started.elapsed().as_millis() as u64;
    let direct_cleanup_failure = match (&outcome, owned_child.as_mut()) {
        (Err(error), Some(child)) => preserve_direct_cleanup_failure(child, error),
        _ => None,
    };
    let (code, interruption) = match outcome {
        Ok(status) => match cancellation.received() {
            Some(signal) => {
                let interruption = RunInterrupted::new(signal);
                (interruption.exit_code(), Some(interruption))
            }
            None => (status.code().unwrap_or(-1), None),
        },
        Err(error) => match run_interruption(&error) {
            Some(interruption) => (interruption.exit_code(), Some(interruption)),
            None => {
                vat.meta.status = Status::Exited { code: -1 };
                if let Some(run) = vat.meta.last_run.as_mut() {
                    run.finished_at = Some(Utc::now());
                    run.exit_code = Some(-1);
                    run.duration_ms = Some(duration_ms);
                    run.signal = cancellation.received();
                    run.owned_pgid = direct_cleanup_failure
                        .as_ref()
                        .and_then(|(owned_pgid, _)| *owned_pgid);
                    run.cleanup_error = direct_cleanup_failure
                        .as_ref()
                        .map(|(_, message)| message.clone());
                }
                vat.save()?;
                return Err(error);
            }
        },
    };

    vat.meta.status = match interruption.as_ref() {
        Some(interruption) => Status::Interrupted {
            signal: interruption.signal,
            reason: interruption.reason.clone(),
        },
        None => Status::Exited { code },
    };
    if let Some(run) = vat.meta.last_run.as_mut() {
        run.finished_at = Some(Utc::now());
        run.exit_code = Some(code);
        run.duration_ms = Some(duration_ms);
        run.signal = interruption.as_ref().map(|value| value.signal);
        run.owned_pgid = None;
        run.cleanup_error = None;
    }
    vat.save()?;
    let changes = vat.changes().unwrap_or_default();
    vat.log(
        Event::new(
            EventKind::RunFinished,
            match interruption.as_ref() {
                Some(interruption) => format!(
                    "interrupted by {} in {duration_ms}ms · {}",
                    signal_name(interruption.signal),
                    changes.oneline()
                ),
                None => format!("exit {code} in {duration_ms}ms · {}", changes.oneline()),
            },
        )
        .with_data(serde_json::json!({
            "exit_code": code,
            "interrupted": interruption.is_some(),
            "signal": interruption.as_ref().map(|value| value.signal),
            "reason": interruption.as_ref().map(|value| value.reason.as_str()),
            "duration_ms": duration_ms,
            "changes": { "added": changes.added.len(), "modified": changes.modified.len(), "deleted": changes.deleted.len() },
        })),
    )?;

    if args.json {
        crate::commands::print_json(&vat.project()?, false)?;
    } else {
        print_summary(&vat, code, duration_ms, &changes, backend.name(), &gpu_info);
    }

    Ok(process_exit_code(code))
}

fn exec_runner(args: RunnerArgs, cancellation: &RunCancellation) -> Result<ExitCode> {
    let cwd = std::env::current_dir().context("get current directory")?;
    let cfg = config::load_nearest(&cwd)?;
    let retention = args.keep.unwrap_or(cfg.workspace.keep);
    if args.base.is_some() || args.from.is_some() {
        bail!(
            "vat run [runner-id] uses vat.toml workspace.base; --base/--from are direct mode only"
        );
    }
    let runners: Vec<RunnerConfig> = if args.runner_ids.len() > 1 {
        // Explicit concurrent set: every id must resolve; duplicates rejected.
        let mut seen = std::collections::BTreeSet::new();
        let mut selected = Vec::new();
        for id in &args.runner_ids {
            if !seen.insert(id.clone()) {
                bail!("runner `{id}` listed twice");
            }
            selected.push(cfg.runner(id)?.clone());
        }
        selected
    } else {
        match cfg.select_runner(args.runner_ids.first().map(String::as_str)) {
            Ok((runner_ref, _reason)) => vec![runner_ref.clone()],
            Err(err) => {
                emit_jsonl(serde_json::json!({
                    "type": "error",
                    "code": "runner_required",
                    "message": err.to_string(),
                    "runners": cfg.runners.iter().map(|runner| runner.id.as_str()).collect::<Vec<_>>(),
                }))?;
                return Err(err);
            }
        }
    };
    let selection_reason = if args.runner_ids.len() > 1 {
        "explicit_concurrent"
    } else if args.runner_ids.len() == 1 {
        "explicit"
    } else if cfg.default_runner.is_some() {
        "default_runner"
    } else {
        "single_runner"
    };
    let joined_ids = runners
        .iter()
        .map(|r| r.id.as_str())
        .collect::<Vec<_>>()
        .join("+");
    emit_jsonl(serde_json::json!({
        "type": "select",
        "runner": joined_ids.as_str(),
        "runners": runners.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
        "reason": selection_reason,
    }))?;
    let gpu_info = gpu::detect();
    if args.gpu == GpuRequest::Required && !gpu_satisfied(args.gpu, args.isolation, &gpu_info) {
        let error_msg = if args.isolation == Isolation::MicroVm {
            "GPU is categorically unreachable in an Apple Silicon microVM".to_string()
        } else {
            gpu_info.note.clone()
        };
        emit_jsonl(serde_json::json!({
            "type": "error",
            "code": "gpu_required",
            "message": error_msg.as_str(),
        }))?;
        bail!(
            "spec requires a GPU but {}",
            if args.isolation == Isolation::MicroVm {
                "GPU is categorically unreachable in an Apple Silicon microVM".to_string()
            } else {
                format!("none is accessible on this host ({})", gpu_info.note)
            }
        );
    }

    let source = std::fs::canonicalize(cfg.base_dir())
        .with_context(|| format!("resolve workspace base {}", cfg.base_dir().display()))?;
    let mut env = cfg.env.clone();
    env.entry("VAT_CONFIG_ROOT".to_string())
        .or_insert_with(|| cfg.root.to_string_lossy().into_owned());
    env.entry("VAT_WORKSPACE_BASE".to_string())
        .or_insert_with(|| source.to_string_lossy().into_owned());

    let spec = EnvSpec {
        base: Some(Base::Dir(source.clone())),
        workdir: cfg.workspace.workdir.clone(),
        env,
        isolation: args.isolation,
        egress: cfg.network.as_ref().map(|n| n.egress).unwrap_or_default(),
        gpu: args.gpu,
        microvm_image: args.microvm_image.clone(),
        ..EnvSpec::default()
    };

    let new_id = id::fresh();
    let name = args
        .name
        .or_else(|| cfg.name.clone())
        .or(Some(joined_ids.clone()));
    let mut vat = store::create(&new_id, name, spec.clone(), Some(&source), Vec::new())
        .context("create vat")?;
    let logs_dir = vat.dir.join(crate::paths::file::LOGS);
    std::fs::create_dir_all(&logs_dir).with_context(|| format!("create {}", logs_dir.display()))?;
    let topology_services = configured_service_ids(&cfg, &runners, &[])?;

    let started = Instant::now();
    vat.meta.status = Status::Running;
    vat.meta.test_run = Some(TestRunEvidence {
        config: ConfigRef {
            path: cfg.path.to_string_lossy().into_owned(),
            digest: cfg.digest.clone(),
        },
        runner_id: joined_ids.clone(),
        retention,
        services: Vec::new(),
        scenario: None,
        runner: None,
        runners: Vec::new(),
        artifacts: Vec::new(),
        cleanup_error: None,
        plan: None,
        topology: Some(TopologyEvidence {
            runners: runners.iter().map(|runner| runner.id.clone()).collect(),
            services: topology_services,
            network: "open".to_string(),
            hermetic: false,
        }),
    });
    vat.save()?;
    // Compose binding is published synchronously and token-authoritatively
    // before any service can start. A failed lock/I/O/ownership check must not
    // be logged and ignored: doing so would leave a live service set with no
    // registry owner able to stop it later.
    if let Some(handoff) = args.compose_handoff.as_ref() {
        let handoff_step = crate::commands::compose::publish_compose_handoff(handoff, &vat.meta.id);
        finish_pre_child_step(
            &mut vat,
            cancellation,
            started,
            handoff_step,
            "publish compose VAT handoff before starting services",
        )?;
    }
    let plan_step = attach_plan_file(&mut vat, args.plan.as_deref());
    finish_pre_child_step(
        &mut vat,
        cancellation,
        started,
        plan_step,
        "attach configured-run plan before starting children",
    )?;
    let log_step = vat.log(Event::new(
        EventKind::RunStarted,
        format!("runner: {joined_ids}"),
    ));
    finish_pre_child_step(
        &mut vat,
        cancellation,
        started,
        log_step,
        "record configured-run start event",
    )?;

    let result = run_configured(
        &mut vat,
        &cfg,
        &runners,
        &logs_dir,
        &[],
        false,
        retention,
        cancellation,
    );
    let mut interruption = None;
    let mut cleanup_failed_after_interruption = false;
    let mut run_error = None;
    let mut code = match result {
        Ok(code) => match cancellation.received() {
            Some(signal) => {
                let observed = RunInterrupted::new(signal);
                let code = observed.exit_code();
                interruption = Some(observed);
                code
            }
            None => code,
        },
        Err(err) if run_interruption(&err).is_some() => {
            let observed = run_interruption(&err).expect("checked interruption");
            let code = observed.exit_code();
            interruption = Some(observed);
            code
        }
        Err(err) if run_cleanup_failure(&err).is_some() => {
            let failure = run_cleanup_failure(&err).expect("checked cleanup failure");
            interruption = Some(failure.interruption);
            cleanup_failed_after_interruption = true;
            let message = err.to_string();
            append_test_run_cleanup_error(&mut vat, &failure.cleanup_error);
            run_error = Some(record_runner_failure_fail_closed(
                &mut vat,
                &runners[0],
                &logs_dir,
                &message,
            ));
            -1
        }
        Err(err) => {
            let message = err.to_string();
            if let Some(failure) = run_owned_cleanup_failure(&err) {
                append_test_run_cleanup_error(&mut vat, &failure.cleanup_error);
            }
            run_error = Some(record_runner_failure_fail_closed(
                &mut vat,
                &runners[0],
                &logs_dir,
                &message,
            ));
            -1
        }
    };
    let cleanup_unconfirmed = unconfirmed_runtime_cleanup_message(&vat);
    if cleanup_unconfirmed.is_some() {
        // A runner that passed or was interrupted cannot make an owned process
        // group/runtime safe to forget. Cleanup proof is a prerequisite for
        // publishing the successful Interrupted/130/143 terminal contract.
        code = -1;
        cleanup_failed_after_interruption |= interruption.is_some();
    }

    if let Some(interruption) = interruption
        .as_ref()
        .filter(|_| !cleanup_failed_after_interruption)
    {
        record_runner_interruption(&mut vat, &runners, &logs_dir, interruption)?;
    }
    vat.meta.status = configured_terminal_status(
        interruption.as_ref(),
        cleanup_failed_after_interruption,
        code,
    );
    vat.save()?;
    if let Some(interruption) = interruption
        .as_ref()
        .filter(|_| !cleanup_failed_after_interruption)
    {
        emit_run_interrupted(&vat, interruption)?;
    }
    if let Some(message) = run_error.as_deref() {
        emit_jsonl(serde_json::json!({
            "type": "error",
            "code": "run_failed",
            "message": message,
        }))?;
    }
    if let Some(message) = cleanup_unconfirmed.as_deref() {
        emit_owned_cleanup_unconfirmed(&vat, message)?;
    }
    let state = vat.project()?;
    let should_remove = should_remove_vat(
        &retention,
        code,
        cleanup_unconfirmed.is_some(),
        interruption.is_some(),
    );

    if should_remove {
        let _ = store::remove(&state.id);
    }

    let kept = !should_remove;
    let runner_results: Vec<serde_json::Value> = vat
        .meta
        .test_run
        .as_ref()
        .map(|t| {
            t.runners
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "ok": r.exit_code == Some(0),
                        "exit_code": r.exit_code,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    emit_jsonl(serde_json::json!({
        "type": "result",
        "id": state.id.as_str(),
        "runner": joined_ids.as_str(),
        "runners": runner_results,
        "ok": code == 0,
        "exit_code": code,
        "lifecycle": if cleanup_failed_after_interruption {
            "cleanup_failed_after_interrupt"
        } else if interruption.is_some() {
            "interrupted"
        } else {
            "exited"
        },
        "signal": interruption.as_ref().map(|value| value.signal),
        "reason": interruption.as_ref().map(|value| value.reason.as_str()),
        "state": if kept { "kept" } else { "removed" },
        "inspect": if kept {
            serde_json::json!({
                "state": format!("vat state {}", state.id),
                "logs": format!("vat logs {} runner", state.id),
                "diff": format!("vat diff {} --json", state.id),
            })
        } else {
            serde_json::Value::Null
        },
    }))?;

    Ok(process_exit_code(code))
}

fn exec_scenario(args: ScenarioArgs, cancellation: &RunCancellation) -> Result<ExitCode> {
    let cwd = std::env::current_dir().context("get current directory")?;
    let cfg = config::load_nearest(&cwd)?;
    let retention = args.keep.unwrap_or(cfg.workspace.keep);
    if args.base.is_some() || args.from.is_some() {
        bail!(
            "vat run --scenario uses vat.toml workspace.base; --base/--from are direct mode only"
        );
    }
    let scenario = match cfg.scenario(&args.scenario_id) {
        Ok(scenario) => scenario.clone(),
        Err(err) => {
            emit_jsonl(serde_json::json!({
                "type": "error",
                "code": "scenario_required",
                "message": err.to_string(),
                "scenarios": cfg.scenarios.iter().map(|scenario| scenario.id.as_str()).collect::<Vec<_>>(),
            }))?;
            return Err(err);
        }
    };
    let runner = cfg.runner(&scenario.runner)?.clone();
    let extra_service_ids = scenario_service_ids(&cfg, &scenario, &runner)?;
    if scenario.network == ScenarioNetworkMode::Hermetic
        && !service_set_has_http_mock(&cfg, &extra_service_ids)
    {
        emit_jsonl(serde_json::json!({
            "type": "error",
            "code": "scenario_hermetic_proxy_required",
            "scenario": scenario.id.as_str(),
            "message": "scenario network = hermetic requires a participating `preset = \"http-mock\"` service",
            "services": extra_service_ids,
        }))?;
        bail!(
            "scenario `{}` network = hermetic requires a participating `preset = \"http-mock\"` service",
            scenario.id
        );
    }
    emit_jsonl(serde_json::json!({
        "type": "select",
        "scenario": scenario.id.as_str(),
        "app": scenario.app.as_str(),
        "runner": runner.id.as_str(),
        "services": extra_service_ids,
        "reason": "scenario",
    }))?;

    let gpu_info = gpu::detect();
    // Note: The isolation override below happens AFTER this check, so we need to use args.isolation here.
    if args.gpu == GpuRequest::Required && !gpu_satisfied(args.gpu, args.isolation, &gpu_info) {
        let error_msg = if args.isolation == Isolation::MicroVm {
            "GPU is categorically unreachable in an Apple Silicon microVM".to_string()
        } else {
            gpu_info.note.clone()
        };
        emit_jsonl(serde_json::json!({
            "type": "error",
            "code": "gpu_required",
            "message": error_msg.as_str(),
        }))?;
        bail!(
            "spec requires a GPU but {}",
            if args.isolation == Isolation::MicroVm {
                "GPU is categorically unreachable in an Apple Silicon microVM".to_string()
            } else {
                format!("none is accessible on this host ({})", gpu_info.note)
            }
        );
    }

    let source = std::fs::canonicalize(cfg.base_dir())
        .with_context(|| format!("resolve workspace base {}", cfg.base_dir().display()))?;
    let mut env = cfg.env.clone();
    env.entry("VAT_CONFIG_ROOT".to_string())
        .or_insert_with(|| cfg.root.to_string_lossy().into_owned());
    env.entry("VAT_WORKSPACE_BASE".to_string())
        .or_insert_with(|| source.to_string_lossy().into_owned());

    let egress = if scenario.network == ScenarioNetworkMode::Hermetic {
        crate::spec::EgressPolicy::LocalhostOnly
    } else {
        cfg.network.as_ref().map(|n| n.egress).unwrap_or_default()
    };
    let isolation =
        if scenario.network == ScenarioNetworkMode::Hermetic && args.isolation == Isolation::None {
            Isolation::Seatbelt
        } else {
            args.isolation
        };
    let spec = EnvSpec {
        base: Some(Base::Dir(source.clone())),
        workdir: cfg.workspace.workdir.clone(),
        env,
        isolation,
        egress,
        gpu: args.gpu,
        microvm_image: args.microvm_image.clone(),
        ..EnvSpec::default()
    };

    let new_id = id::fresh();
    let name = args
        .name
        .or_else(|| cfg.name.clone())
        .or(Some(scenario.id.clone()));
    let mut vat = store::create(&new_id, name, spec.clone(), Some(&source), Vec::new())
        .context("create vat")?;
    let logs_dir = vat.dir.join(crate::paths::file::LOGS);
    std::fs::create_dir_all(&logs_dir).with_context(|| format!("create {}", logs_dir.display()))?;

    let started = Instant::now();
    vat.meta.status = Status::Running;
    vat.meta.test_run = Some(TestRunEvidence {
        config: ConfigRef {
            path: cfg.path.to_string_lossy().into_owned(),
            digest: cfg.digest.clone(),
        },
        runner_id: runner.id.clone(),
        retention,
        services: Vec::new(),
        scenario: Some(ScenarioRunRecord {
            id: scenario.id.clone(),
            app: scenario.app.clone(),
            runner: runner.id.clone(),
            network: scenario_network_name(scenario.network).to_string(),
            services: extra_service_ids.clone(),
            routes: Vec::new(),
            hermetic: scenario.network == ScenarioNetworkMode::Hermetic,
        }),
        runner: None,
        runners: Vec::new(),
        artifacts: Vec::new(),
        cleanup_error: None,
        plan: None,
        topology: Some(TopologyEvidence {
            runners: vec![runner.id.clone()],
            services: extra_service_ids.clone(),
            network: scenario_network_name(scenario.network).to_string(),
            hermetic: scenario.network == ScenarioNetworkMode::Hermetic,
        }),
    });
    vat.save()?;
    let plan_step = attach_plan_file(&mut vat, args.plan.as_deref());
    finish_pre_child_step(
        &mut vat,
        cancellation,
        started,
        plan_step,
        "attach scenario plan before starting children",
    )?;
    let log_step = vat.log(Event::new(
        EventKind::RunStarted,
        format!("scenario: {}", scenario.id),
    ));
    finish_pre_child_step(
        &mut vat,
        cancellation,
        started,
        log_step,
        "record scenario start event",
    )?;

    let runners = vec![runner.clone()];
    let result = run_configured(
        &mut vat,
        &cfg,
        &runners,
        &logs_dir,
        &extra_service_ids,
        scenario.network == ScenarioNetworkMode::Hermetic,
        retention,
        cancellation,
    );
    let mut interruption = None;
    let mut cleanup_failed_after_interruption = false;
    let mut run_error = None;
    let mut code = match result {
        Ok(code) => match cancellation.received() {
            Some(signal) => {
                let observed = RunInterrupted::new(signal);
                let code = observed.exit_code();
                interruption = Some(observed);
                code
            }
            None => code,
        },
        Err(err) if run_interruption(&err).is_some() => {
            let observed = run_interruption(&err).expect("checked interruption");
            let code = observed.exit_code();
            interruption = Some(observed);
            code
        }
        Err(err) if run_cleanup_failure(&err).is_some() => {
            let failure = run_cleanup_failure(&err).expect("checked cleanup failure");
            interruption = Some(failure.interruption);
            cleanup_failed_after_interruption = true;
            let message = err.to_string();
            append_test_run_cleanup_error(&mut vat, &failure.cleanup_error);
            run_error = Some(record_runner_failure_fail_closed(
                &mut vat, &runner, &logs_dir, &message,
            ));
            -1
        }
        Err(err) => {
            let message = err.to_string();
            if let Some(failure) = run_owned_cleanup_failure(&err) {
                append_test_run_cleanup_error(&mut vat, &failure.cleanup_error);
            }
            run_error = Some(record_runner_failure_fail_closed(
                &mut vat, &runner, &logs_dir, &message,
            ));
            -1
        }
    };
    let cleanup_unconfirmed = unconfirmed_runtime_cleanup_message(&vat);
    if cleanup_unconfirmed.is_some() {
        code = -1;
        cleanup_failed_after_interruption |= interruption.is_some();
    }

    if let Some(interruption) = interruption
        .as_ref()
        .filter(|_| !cleanup_failed_after_interruption)
    {
        record_runner_interruption(&mut vat, &runners, &logs_dir, interruption)?;
    }
    vat.meta.status = configured_terminal_status(
        interruption.as_ref(),
        cleanup_failed_after_interruption,
        code,
    );
    vat.save()?;
    if let Some(interruption) = interruption
        .as_ref()
        .filter(|_| !cleanup_failed_after_interruption)
    {
        emit_run_interrupted(&vat, interruption)?;
    }
    if let Some(message) = run_error.as_deref() {
        emit_jsonl(serde_json::json!({
            "type": "error",
            "code": "run_failed",
            "message": message,
        }))?;
    }
    if let Some(message) = cleanup_unconfirmed.as_deref() {
        emit_owned_cleanup_unconfirmed(&vat, message)?;
    }
    let state = vat.project()?;
    let should_remove = should_remove_vat(
        &retention,
        code,
        cleanup_unconfirmed.is_some(),
        interruption.is_some(),
    );
    if should_remove {
        let _ = store::remove(&state.id);
    }
    let kept = !should_remove;
    emit_jsonl(serde_json::json!({
        "type": "result",
        "id": state.id.as_str(),
        "scenario": scenario.id.as_str(),
        "app": scenario.app.as_str(),
        "runner": runner.id.as_str(),
        "ok": code == 0,
        "exit_code": code,
        "lifecycle": if cleanup_failed_after_interruption {
            "cleanup_failed_after_interrupt"
        } else if interruption.is_some() {
            "interrupted"
        } else {
            "exited"
        },
        "signal": interruption.as_ref().map(|value| value.signal),
        "reason": interruption.as_ref().map(|value| value.reason.as_str()),
        "state": if kept { "kept" } else { "removed" },
        "inspect": if kept {
            serde_json::json!({
                "state": format!("vat state {}", state.id),
                "logs": format!("vat logs {} runner", state.id),
                "diff": format!("vat diff {} --json", state.id),
            })
        } else {
            serde_json::Value::Null
        },
    }))?;

    Ok(process_exit_code(code))
}

fn process_exit_code(code: i32) -> ExitCode {
    if code < 0 {
        ExitCode::from(255)
    } else {
        ExitCode::from(code.clamp(0, 255) as u8)
    }
}

fn attach_plan_file(vat: &mut store::Vat, plan_path: Option<&Path>) -> Result<()> {
    let Some(plan_path) = plan_path else {
        return Ok(());
    };
    let bytes = std::fs::read(plan_path)
        .with_context(|| format!("read plan file {}", plan_path.display()))?;
    let source_path = std::fs::canonicalize(plan_path)
        .unwrap_or_else(|_| plan_path.to_path_buf())
        .to_string_lossy()
        .into_owned();
    let dest_dir = vat.rootfs().join(".vat-plan");
    std::fs::create_dir_all(&dest_dir).with_context(|| format!("create {}", dest_dir.display()))?;
    let file_name = plan_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("plan.json");
    let dest = dest_dir.join(file_name);
    std::fs::write(&dest, &bytes).with_context(|| format!("write {}", dest.display()))?;
    let evidence = PlanEvidence {
        source_path,
        rootfs_path: dest.to_string_lossy().into_owned(),
        digest: digest_bytes(&bytes),
    };
    vat.meta
        .spec
        .env
        .insert("VAT_PLAN_PATH".to_string(), evidence.rootfs_path.clone());
    vat.meta
        .spec
        .env
        .insert("VAT_PLAN_DIGEST".to_string(), evidence.digest.clone());
    vat.meta.plan = Some(evidence.clone());
    if let Some(test_run) = vat.meta.test_run.as_mut() {
        test_run.plan = Some(evidence);
    }
    vat.save()
}

const DETACHED_COMPOSE_STOP_REQUEST: &str = ".compose-stop-request";

fn detached_compose_stop_request_path(vat: &store::Vat) -> PathBuf {
    vat.dir.join(DETACHED_COMPOSE_STOP_REQUEST)
}

/// Request that a live detached compose run stop its own runner and service
/// tree. This avoids treating a persisted child PID as an authority boundary:
/// the VAT parent owns its children, cleanup, and terminal state persistence.
pub(crate) fn request_detached_compose_stop(vat: &store::Vat) -> Result<()> {
    let path = detached_compose_stop_request_path(vat);
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .with_context(|| format!("write detached compose stop request {}", path.display()))?;
    writeln!(file, "requested_at={}", Utc::now().to_rfc3339())
        .with_context(|| format!("write detached compose stop request {}", path.display()))?;
    file.flush()
        .with_context(|| format!("flush detached compose stop request {}", path.display()))
}

fn take_detached_compose_stop_request(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    let _ = std::fs::remove_file(path);
    true
}

fn consume_detached_compose_stop_request(path: &Path, phase: &str) -> Result<()> {
    if take_detached_compose_stop_request(path) {
        bail!("detached compose stop requested during {phase}");
    }
    Ok(())
}

// <HANDWRITE gap="missing-generator:logic" tracker="1872" reason="Expose validated active-run service capture paths to trusted same-run runners.">
fn prepare_service_log_handoff(
    vat_dir: &Path,
    logs_dir: &Path,
    service_ids: &[&str],
) -> Result<BTreeMap<String, String>> {
    let vat_root = std::fs::canonicalize(vat_dir)
        .with_context(|| format!("resolve active vat directory {}", vat_dir.display()))?;
    let logs_root = std::fs::canonicalize(logs_dir)
        .with_context(|| format!("resolve active vat logs directory {}", logs_dir.display()))?;
    if !logs_root.starts_with(&vat_root) {
        bail!(
            "active VAT logs directory {} escapes vat directory {}",
            logs_root.display(),
            vat_root.display()
        );
    }

    let mut tokens = BTreeMap::<String, String>::new();
    let mut paths = Vec::with_capacity(service_ids.len());
    for service_id in service_ids {
        let token = normalize_service_log_env_token(service_id)?;
        if let Some(existing) = tokens.insert(token.clone(), (*service_id).to_string()) {
            bail!(
                "service log environment token collision: `{existing}` and `{service_id}` both map to VAT_SERVICE_{token}"
            );
        }
        let stdout = logs_root.join(format!("{service_id}.stdout.log"));
        let stderr = logs_root.join(format!("{service_id}.stderr.log"));
        if stdout.parent() != Some(logs_root.as_path())
            || stderr.parent() != Some(logs_root.as_path())
        {
            bail!("service `{service_id}` log path escapes active VAT logs directory");
        }
        paths.push((token, stdout, stderr));
    }

    let mut env = BTreeMap::from([(
        "VAT_LOGS_DIR".to_string(),
        logs_root.to_string_lossy().into_owned(),
    )]);
    for (token, stdout, stderr) in paths {
        File::create(&stdout)
            .with_context(|| format!("create service stdout capture {}", stdout.display()))?;
        File::create(&stderr)
            .with_context(|| format!("create service stderr capture {}", stderr.display()))?;
        env.insert(
            format!("VAT_SERVICE_{token}_STDOUT_LOG"),
            stdout.to_string_lossy().into_owned(),
        );
        env.insert(
            format!("VAT_SERVICE_{token}_STDERR_LOG"),
            stderr.to_string_lossy().into_owned(),
        );
    }
    Ok(env)
}

fn normalize_service_log_env_token(service_id: &str) -> Result<String> {
    if service_id.is_empty() || service_id.len() > 64 {
        bail!("unsafe service id `{service_id}` for active log-path handoff");
    }
    let mut chars = service_id.chars();
    if !chars.next().is_some_and(|ch| ch.is_ascii_alphanumeric())
        || !service_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        bail!(
            "unsafe service id `{service_id}` for active log-path handoff; use an ASCII alphanumeric prefix followed by ASCII alphanumeric, dash, underscore, or dot"
        );
    }
    Ok(service_id
        .chars()
        .map(|ch| match ch {
            '-' | '.' => '_',
            ch => ch.to_ascii_uppercase(),
        })
        .collect())
}

fn run_configured(
    vat: &mut store::Vat,
    cfg: &VatConfig,
    runners: &[RunnerConfig],
    logs_dir: &Path,
    extra_service_ids: &[String],
    force_hermetic_proxy: bool,
    retention: RetentionPolicy,
    cancellation: &RunCancellation,
) -> Result<i32> {
    cancellation.check()?;
    let rootfs = vat.rootfs();
    let cwd = rootfs.join(&vat.meta.spec.workdir);
    std::fs::create_dir_all(&cwd).with_context(|| format!("create {}", cwd.display()))?;

    // Runner + setup-step commands run under the run's isolation backend
    // (seatbelt wraps them in sandbox-exec with the [network].egress policy; the
    // process backend is a passthrough). Picked once so any isolation/egress
    // error surfaces once, before any runner/service work starts. Services
    // below are spawned RAW — they keep network.
    let sandbox_spec = vat.meta.spec.clone();
    let sandbox_backend = sandbox::pick(&sandbox_spec).map_err(anyhow::Error::msg)?;

    // Services: the UNION of every runner's requires, order-preserving and
    // deduplicated — one shared instance set serves all concurrent runners.
    let mut service_ids: Vec<&str> = Vec::new();
    for service_id in extra_service_ids {
        if !service_ids.contains(&service_id.as_str()) {
            service_ids.push(service_id);
        }
    }
    for runner in runners {
        for service_id in &runner.requires {
            if !service_ids.contains(&service_id.as_str()) {
                service_ids.push(service_id);
            }
        }
    }
    // This is a trusted same-run handoff, not a cross-user security boundary:
    // paths remain under vat.dir, child bytes stay in their capture files, and
    // VAT neither parses nor replays them to its own JSONL stdout.
    let service_log_env = prepare_service_log_handoff(&vat.dir, logs_dir, &service_ids)?;
    let mut service_plans = Vec::new();
    let mut services = Vec::new();
    let mut procs = Vec::new();
    let execution = (|| -> Result<i32> {
        let mut run_env = vat.meta.spec.env.clone();
        let compose_stop_request = detached_compose_stop_request_path(vat);
        for service in ordered_required_services(cfg, &service_ids)? {
            cancellation.check()?;
            consume_detached_compose_stop_request(&compose_stop_request, "service preparation")?;
            let plan = prepare_service(vat, cfg, service, force_hermetic_proxy, cancellation)?;
            for (key, value) in &plan.env {
                run_env.insert(key.clone(), value.clone());
            }
            service_plans.push(plan);
            consume_detached_compose_stop_request(&compose_stop_request, "service preparation")?;
        }
        // VAT owns these reserved values. Apply them after user/service exports so
        // a config cannot redirect a collector to an arbitrary host path.
        run_env.extend(service_log_env);

        // Transparent service routing (network sandbox): every declared GCP emulator
        // preset's real googleapis host -> its resolved local endpoint, seeded onto
        // the http-mock proxy so the runner reaches the local emulator with NO
        // hand-written [network.routes]. Ports are resolved during the prepare loop
        // above, so the proxy is spawned with the full (explicit + preset-derived)
        // route set.
        seed_preset_routes_into_proxy(&mut service_plans, cfg);
        persist_scenario_topology(vat, cfg, &service_plans, force_hermetic_proxy)?;

        for step in &cfg.setup {
            cancellation.check()?;
            consume_detached_compose_stop_request(&compose_stop_request, "setup")?;
            if !config::should_run_setup(&rootfs, step) {
                continue;
            }
            run_setup_step(
                vat,
                step,
                &cwd,
                logs_dir,
                &run_env,
                sandbox_backend.as_ref(),
                &rootfs,
                cancellation,
            )?;
            consume_detached_compose_stop_request(&compose_stop_request, "setup")?;
        }

        for plan in &mut service_plans {
            consume_detached_compose_stop_request(&compose_stop_request, "service startup")?;
            if let Err(error) = cancellation.check() {
                let interruption = run_interruption(&error);
                finalize_services_and_persist(
                    vat,
                    &mut services,
                    should_delete_clusters(&retention, -1),
                    interruption.as_ref(),
                )?;
                return Err(error);
            }
            let handle = match start_service(
                vat,
                plan,
                &cwd,
                logs_dir,
                &run_env,
                service_sandbox_backend(force_hermetic_proxy, sandbox_backend.as_ref()),
                &rootfs,
                cancellation,
            ) {
                Ok(handle) => handle,
                Err(err) => {
                    finalize_services_and_persist(
                        vat,
                        &mut services,
                        should_delete_clusters(&retention, -1),
                        None,
                    )?;
                    return Err(err);
                }
            };
            services.push(handle);
            // Transfer cluster ownership from the prepared plan to the live
            // ServiceHandle. Any cluster still present on a plan at scope exit was
            // prepared but never started and is deleted by the outer finalizer.
            plan.cluster = None;
            // Persist ownership before the blocking readiness loop. If VAT itself
            // is interrupted while a MicroVM endpoint is still being verified,
            // `vat state` must retain the generated container name for diagnosis
            // and cleanup rather than leaving an untracked runtime resource.
            if let Err(err) = persist_services(vat, &services) {
                // A failed metadata write is itself terminal, but the service has
                // already launched. Tear it down before returning so this early
                // persistence checkpoint cannot create an untracked MicroVM.
                finalize_services_and_persist(
                    vat,
                    &mut services,
                    should_delete_clusters(&retention, -1),
                    None,
                )?;
                return Err(err);
            }
            let last = services.len() - 1;
            if let Err(err) = wait_for_services(vat, &mut services[last..], cancellation) {
                let interruption = run_interruption(&err);
                finalize_services_and_persist(
                    vat,
                    &mut services,
                    should_delete_clusters(&retention, -1),
                    interruption.as_ref(),
                )?;
                return Err(err);
            }
            persist_services(vat, &services)?;
        }
        persist_services(vat, &services)?;

        // Spawn every runner, THEN wait — concurrency comes from the children
        // running side by side, not from threads in vat.
        let single = runners.len() == 1;
        for runner in runners {
            if let Some(signal) = cancellation.received() {
                let interruption = RunInterrupted::new(signal);
                finalize_configured_children(
                    vat,
                    &mut procs,
                    &mut services,
                    should_delete_clusters(&retention, -1),
                    Some(&interruption),
                )?;
                return Err(interruption.into());
            }
            emit_jsonl(serde_json::json!({
                "type": "runner",
                "id": runner.id.as_str(),
                "state": "started",
            }))?;
            match spawn_runner_process(
                runner,
                &cwd,
                logs_dir,
                &run_env,
                single,
                sandbox_backend.as_ref(),
                &rootfs,
            ) {
                Ok(proc) => procs.push(proc),
                Err(err) => {
                    finalize_configured_children(
                        vat,
                        &mut procs,
                        &mut services,
                        should_delete_clusters(&retention, -1),
                        None,
                    )?;
                    return Err(err);
                }
            }
        }
        // Persist an interim RunnerRunRecord carrying each spawned runner's live
        // pid BEFORE the blocking wait below — mirrors persist_services()'s
        // early-write pattern for services. Required so a concurrent reader
        // (`vat compose down`) can observe a live pid while the runner is still
        // executing; without this, test_run.runner.pid is only ever populated
        // after wait_runner_processes returns, i.e. after the runner has already
        // exited (R9).
        let interim_records: Vec<RunnerRunRecord> = procs
            .iter()
            .map(|proc| RunnerRunRecord {
                id: proc.runner.id.clone(),
                command: proc.runner.cmd.clone(),
                status: ProcessStatus::Running,
                exit_code: None,
                duration_ms: None,
                pid: Some(proc.child.id()),
                cleanup_error: None,
                stdout_log: proc.stdout_log.clone(),
                stderr_log: proc.stderr_log.clone(),
            })
            .collect();
        if let Some(test_run) = vat.meta.test_run.as_mut() {
            test_run.runner = interim_records.first().cloned();
            test_run.runners = interim_records.clone();
        }
        if let Err(error) = vat.save() {
            finalize_configured_children(
                vat,
                &mut procs,
                &mut services,
                should_delete_clusters(&retention, -1),
                None,
            )?;
            return Err(error);
        }

        let stop_request = detached_compose_stop_request_path(vat);
        let records = match wait_runner_processes(&mut procs, &stop_request, cancellation) {
            Ok(records) => records,
            Err(error) => {
                let interruption = run_interruption(&error);
                finalize_configured_children(
                    vat,
                    &mut procs,
                    &mut services,
                    should_delete_clusters(&retention, -1),
                    interruption.as_ref(),
                )?;
                return Err(error);
            }
        };

        // Worst-wins exit: any negative (timeout/kill) is worst, else max code.
        let code = records
            .iter()
            .map(|r| r.exit_code.unwrap_or(-1))
            .fold(0, |acc, c| if acc < 0 || c < 0 { -1 } else { acc.max(c) });
        let evidence = (|| -> Result<()> {
            for record in &records {
                emit_jsonl(serde_json::json!({
                    "type": "runner",
                    "id": record.id.as_str(),
                    "state": "exited",
                    "exit_code": record.exit_code,
                }))?;
            }
            if let Some(test_run) = vat.meta.test_run.as_mut() {
                test_run.runner = records.first().cloned();
                test_run.runners = records.clone();
                let mut artifacts = Vec::new();
                for runner in runners {
                    artifacts.extend(collect_artifacts(&rootfs, &runner.artifacts)?);
                }
                test_run.artifacts = artifacts;
            }
            vat.save()
        })();
        let cleanup = finalize_configured_children(
            vat,
            &mut procs,
            &mut services,
            should_delete_clusters(&retention, code),
            None,
        );
        match (evidence, cleanup) {
            (Ok(()), Ok(())) => {}
            (Err(evidence), Ok(())) => return Err(evidence),
            (Ok(()), Err(cleanup)) => return Err(cleanup),
            (Err(evidence), Err(cleanup)) => {
                return Err(cleanup).context(format!(
                    "configured-run evidence also failed before cleanup: {evidence:#}"
                ));
            }
        }
        cancellation.check()?;
        let summary = records
            .iter()
            .map(|r| format!("{} exited {}", r.id, r.exit_code.unwrap_or(-1)))
            .collect::<Vec<_>>()
            .join("; ");
        vat.log(Event::new(EventKind::RunFinished, summary))?;
        Ok(code)
    })();

    let interruption = execution.as_ref().err().and_then(|error| {
        run_interruption(error)
            .or_else(|| run_cleanup_failure(error).map(|failure| failure.interruption))
    });
    let terminal_code = execution.as_ref().copied().unwrap_or(-1);
    // The outer ordinary-thread finalizer is intentionally redundant with
    // bounded inner checkpoints. Idempotent child finalizers make it a no-op
    // after a handled path, while any forgotten `?` still converges here.
    let child_cleanup = finalize_configured_children(
        vat,
        &mut procs,
        &mut services,
        interruption.is_some() || should_delete_clusters(&retention, terminal_code),
        interruption.as_ref(),
    );
    let prepared_cleanup = cleanup_unstarted_cluster_plans(vat, &service_plans);
    let cleanup = match (child_cleanup, prepared_cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(children), Ok(())) => Err(children),
        (Ok(()), Err(prepared)) => Err(prepared),
        (Err(children), Err(prepared)) => Err(prepared).context(format!(
            "prepared-cluster cleanup also followed child cleanup failure: {children:#}"
        )),
    };
    match (execution, cleanup) {
        (Ok(code), Ok(())) => Ok(code),
        (Err(execution), Ok(())) => Err(execution),
        (Ok(_), Err(cleanup)) => Err(cleanup),
        (Err(execution), Err(cleanup)) => Err(cleanup).context(format!(
            "configured execution also failed before final cleanup: {execution:#}"
        )),
    }
}
// </HANDWRITE>

fn scenario_service_ids(
    cfg: &VatConfig,
    scenario: &ScenarioConfig,
    runner: &RunnerConfig,
) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    for id in std::iter::once(&scenario.app)
        .chain(scenario.requires.iter())
        .chain(runner.requires.iter())
    {
        if !ids.contains(id) {
            ids.push(id.clone());
        }
    }
    let service_ids: Vec<&str> = ids.iter().map(|id| id.as_str()).collect();
    Ok(ordered_required_services(cfg, &service_ids)?
        .into_iter()
        .map(|service| service.id.clone())
        .collect())
}

fn configured_service_ids(
    cfg: &VatConfig,
    runners: &[RunnerConfig],
    extra_service_ids: &[String],
) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    for service_id in extra_service_ids {
        if !ids.contains(service_id) {
            ids.push(service_id.clone());
        }
    }
    for runner in runners {
        for service_id in &runner.requires {
            if !ids.contains(service_id) {
                ids.push(service_id.clone());
            }
        }
    }
    Ok(
        ordered_required_services(cfg, &ids.iter().map(String::as_str).collect::<Vec<_>>())?
            .into_iter()
            .map(|service| service.id.clone())
            .collect(),
    )
}

fn service_set_has_http_mock(cfg: &VatConfig, service_ids: &[String]) -> bool {
    service_ids.iter().any(|id| {
        cfg.service(id)
            .map(|service| service.preset == Some(ServicePreset::HttpMock))
            .unwrap_or(false)
    })
}

fn scenario_network_name(network: ScenarioNetworkMode) -> &'static str {
    match network {
        ScenarioNetworkMode::Open => "open",
        ScenarioNetworkMode::Hermetic => "hermetic",
    }
}

fn persist_scenario_topology(
    vat: &mut store::Vat,
    cfg: &VatConfig,
    plans: &[ServicePlan],
    force_hermetic_proxy: bool,
) -> Result<()> {
    let routes = scenario_route_records(cfg, plans);
    if let Some(test_run) = vat.meta.test_run.as_mut() {
        if let Some(topology) = test_run.topology.as_mut() {
            topology.services = plans.iter().map(|plan| plan.id.clone()).collect();
            topology.hermetic = force_hermetic_proxy;
        }
        if let Some(scenario) = test_run.scenario.as_mut() {
            scenario.services = plans.iter().map(|plan| plan.id.clone()).collect();
            scenario.routes = routes;
            scenario.hermetic = force_hermetic_proxy;
        }
    }
    vat.save()
}

fn scenario_route_records(cfg: &VatConfig, plans: &[ServicePlan]) -> Vec<RouteRecord> {
    let mut records = Vec::new();
    let mut explicit_hosts = BTreeSet::new();
    for (host, target) in explicit_network_routes(cfg) {
        explicit_hosts.insert(host.clone());
        records.push(RouteRecord {
            host,
            target,
            source: "explicit".to_string(),
        });
    }
    let pairs: Vec<(Option<ServicePreset>, Option<u16>)> =
        plans.iter().map(|plan| (plan.preset, plan.port)).collect();
    for (host, target) in preset_auto_routes(&pairs) {
        if explicit_hosts.contains(&host) {
            continue;
        }
        records.push(RouteRecord {
            host,
            target,
            source: "preset".to_string(),
        });
    }
    records
}

fn finalize_runner_processes(procs: &mut [RunnerProc]) -> Result<()> {
    let mut failures = Vec::new();
    for proc in procs {
        match proc.child.finalize(&format!("runner `{}`", proc.runner.id)) {
            Ok(_) => proc.cleanup_error = None,
            Err(error) => {
                let detail = format!("runner `{}`: {error:#}", proc.runner.id);
                proc.cleanup_error = Some(detail.clone());
                // The ordinary owner now carries the durable retry/diagnostic
                // obligation. Drop must not signal the same numeric PGID again
                // after this failed attempt.
                proc.child.preserve_cleanup_obligation();
                failures.push(detail);
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        bail!(
            "VAT-owned runner cleanup unconfirmed: {}",
            failures.join("; ")
        )
    }
}

/// Persist every started runner as terminal when process-group cleanup itself
/// fails. Successful siblings must not remain `Running`, while the failed
/// group's still-live leader PID (if any) and cleanup diagnosis stay durable.
fn record_runner_cleanup_outcomes(vat: &mut store::Vat, procs: &[RunnerProc]) {
    if !procs.iter().any(|proc| proc.cleanup_error.is_some()) {
        return;
    }
    let previous = vat
        .meta
        .test_run
        .as_ref()
        .map(|run| run.runners.clone())
        .unwrap_or_default();
    let records = procs
        .iter()
        .map(|proc| {
            if let Some(error) = proc.cleanup_error.as_deref() {
                if let Ok(mut stderr) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&proc.stderr_log)
                {
                    let _ = writeln!(stderr, "{error}");
                }
                return RunnerRunRecord {
                    id: proc.runner.id.clone(),
                    command: proc.runner.cmd.clone(),
                    status: ProcessStatus::Failed,
                    exit_code: Some(-1),
                    duration_ms: Some(proc.started.elapsed().as_millis() as u64),
                    pid: (!proc.child.leader_reaped()).then_some(proc.child.id()),
                    cleanup_error: Some(error.to_string()),
                    stdout_log: proc.stdout_log.clone(),
                    stderr_log: proc.stderr_log.clone(),
                };
            }
            let mut record = previous
                .iter()
                .find(|record| record.id == proc.runner.id)
                .cloned()
                .unwrap_or_else(|| RunnerRunRecord {
                    id: proc.runner.id.clone(),
                    command: proc.runner.cmd.clone(),
                    status: ProcessStatus::Exited,
                    exit_code: proc.child.final_status.and_then(|status| status.code()),
                    duration_ms: Some(proc.started.elapsed().as_millis() as u64),
                    pid: None,
                    cleanup_error: None,
                    stdout_log: proc.stdout_log.clone(),
                    stderr_log: proc.stderr_log.clone(),
                });
            if record.status == ProcessStatus::Running {
                record.status = ProcessStatus::Exited;
                record.exit_code = proc.child.final_status.and_then(|status| status.code());
                record.duration_ms = Some(proc.started.elapsed().as_millis() as u64);
            }
            record.pid = None;
            record.cleanup_error = None;
            record
        })
        .collect::<Vec<_>>();
    if let Some(test_run) = vat.meta.test_run.as_mut() {
        test_run.runner = records.first().cloned();
        test_run.runners = records;
    }
}

fn ordered_required_services<'a>(
    cfg: &'a VatConfig,
    service_ids: &[&str],
) -> Result<Vec<&'a ServiceConfig>> {
    let mut ordered = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for service_id in service_ids {
        visit_required_service(cfg, service_id, &mut visiting, &mut visited, &mut ordered)?;
    }
    Ok(ordered)
}

fn visit_required_service<'a>(
    cfg: &'a VatConfig,
    service_id: &str,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
    ordered: &mut Vec<&'a ServiceConfig>,
) -> Result<()> {
    if visited.contains(service_id) {
        return Ok(());
    }
    if !visiting.insert(service_id.to_string()) {
        bail!("service dependency cycle includes `{service_id}`");
    }
    let service = cfg.service(service_id)?;
    for required in &service.requires {
        visit_required_service(cfg, required, visiting, visited, ordered)?;
    }
    visiting.remove(service_id);
    visited.insert(service_id.to_string());
    ordered.push(service);
    Ok(())
}

const OWNED_GROUP_TERM_GRACE: Duration = Duration::from_millis(300);
const OWNED_GROUP_STOP_TIMEOUT: Duration = Duration::from_secs(3);
const OWNED_GROUP_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// One direct child and the private process group whose numeric PGID it pins.
/// Every exit path converges here: TERM, bounded grace, KILL, leader reap, and
/// an explicit group-absence proof. A completed finalizer is an idempotent
/// no-op, including when cleanup is requested a second time.
struct OwnedProcessGroup {
    child: Child,
    pgid: u32,
    final_status: Option<ExitStatus>,
    finalized: bool,
    /// A leader was already reaped, but PGID absence could not be confirmed.
    /// The numeric PGID is no longer pinned and must never be signalled again;
    /// repeated cleanup returns this cached failure without side effects.
    post_reap_cleanup_error: Option<String>,
    /// Disabled only after the ordinary owner has persisted an unconfirmed
    /// cleanup obligation. Drop must not silently retry and invalidate that
    /// durable PID/identity evidence behind the owner's back.
    drop_cleanup_enabled: bool,
}

impl OwnedProcessGroup {
    fn new(child: Child) -> Self {
        let pgid = child.id();
        Self {
            child,
            pgid,
            final_status: None,
            finalized: false,
            post_reap_cleanup_error: None,
            drop_cleanup_enabled: true,
        }
    }

    fn id(&self) -> u32 {
        self.pgid
    }

    fn leader_reaped(&self) -> bool {
        self.final_status.is_some()
    }

    fn preserve_cleanup_obligation(&mut self) {
        self.drop_cleanup_enabled = false;
    }

    /// Observe leader completion without reaping it, then finalize the whole
    /// group before returning the leader status. Keeping the leader waitable
    /// until group signalling prevents accidental signalling of a recycled
    /// numeric PGID while an owned descendant remains.
    fn finished_status(&mut self, label: &str) -> Result<Option<ExitStatus>> {
        if self.finalized {
            return Ok(self.final_status);
        }
        if let Some(error) = self.post_reap_cleanup_error.as_deref() {
            bail!("{error}");
        }
        if self.final_status.is_some() {
            bail!("{label} leader was reaped but process-group absence remains unconfirmed");
        }
        #[cfg(not(unix))]
        if let Some(status) = self.child.try_wait()? {
            self.final_status = Some(status);
            self.finalized = true;
            return Ok(Some(status));
        }
        #[cfg(unix)]
        if child_has_exited_without_reap(&self.child)? {
            return self.finalize(label).map(Some);
        }
        Ok(None)
    }

    fn finalize(&mut self, label: &str) -> Result<ExitStatus> {
        if self.finalized {
            return self
                .final_status
                .context("finalized owned process group is missing exit status");
        }
        if let Some(error) = self.post_reap_cleanup_error.as_deref() {
            bail!("{error}");
        }
        if self.final_status.is_some() {
            bail!("{label} leader was reaped but process-group absence remains unconfirmed");
        }
        let status = terminate_and_reap_owned_process_group(
            &mut self.child,
            self.pgid,
            label,
            OWNED_GROUP_TERM_GRACE,
            OWNED_GROUP_STOP_TIMEOUT,
        )?;
        self.final_status = Some(status);
        match confirm_owned_process_group_absent(self.pgid, label, OWNED_GROUP_STOP_TIMEOUT) {
            Ok(()) => {
                self.finalized = true;
                Ok(status)
            }
            Err(error) => {
                let message = format!(
                    "{label} leader {} was reaped, but process-group absence is unconfirmed: {error:#}; numeric PGID will not be signalled again",
                    self.pgid
                );
                self.post_reap_cleanup_error = Some(message.clone());
                bail!("{message}")
            }
        }
    }

    /// Deadline-sharing variant for runtime cleanup helper commands. Unlike
    /// `finalize`, TERM grace, KILL/reap, and group-absence proof all consume
    /// one caller-owned absolute deadline; a timed-out Docker client therefore
    /// cannot silently add the generic 6.3s finalizer tail after its phase.
    fn finalize_before(&mut self, label: &str, deadline: Instant) -> Result<ExitStatus> {
        if self.finalized {
            return self
                .final_status
                .context("finalized owned process group is missing exit status");
        }
        if let Some(error) = self.post_reap_cleanup_error.as_deref() {
            bail!("{error}");
        }
        if self.final_status.is_some() {
            bail!("{label} leader was reaped but process-group absence remains unconfirmed");
        }
        let status = terminate_and_reap_owned_process_group_before(
            &mut self.child,
            self.pgid,
            label,
            deadline,
        )?;
        self.final_status = Some(status);
        match confirm_owned_process_group_absent_before(self.pgid, label, deadline) {
            Ok(()) => {
                self.finalized = true;
                Ok(status)
            }
            Err(error) => {
                let message = format!(
                    "{label} leader {} was reaped, but process-group absence is unconfirmed before the shared deadline: {error:#}; numeric PGID will not be signalled again",
                    self.pgid
                );
                self.post_reap_cleanup_error = Some(message.clone());
                bail!("{message}")
            }
        }
    }
}

impl Drop for OwnedProcessGroup {
    fn drop(&mut self) {
        if self.drop_cleanup_enabled && !self.finalized {
            let _ = self.finalize("dropped VAT-owned child");
        }
    }
}

/// One spawned (not yet reaped) runner child plus its bookkeeping.
struct RunnerProc {
    runner: RunnerConfig,
    child: OwnedProcessGroup,
    cleanup_error: Option<String>,
    started: Instant,
    deadline: Option<Instant>,
    stdout_log: String,
    stderr_log: String,
}

/// Wrap a runner/step command in the run's isolation backend: seatbelt rewrites
/// it as `sandbox-exec -p <profile> -- <cmd>` (confining writes to `rootfs` and
/// applying the `[network].egress` policy), while the process backend is a
/// passthrough (returns the command verbatim). Services are spawned RAW (not via
/// this) so they keep the network needed to serve/forward.
/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-commands-run-rs.md#source
pub(crate) fn sandbox_wrap(
    backend: &dyn sandbox::Sandbox,
    rootfs: &Path,
    cmd: &[String],
) -> Vec<String> {
    if cmd.is_empty() {
        return cmd.to_vec();
    }
    let (prog, argv) = backend.resolve(rootfs, &cmd[0], &cmd[1..]);
    std::iter::once(prog).chain(argv).collect()
}

/// Spawn one runner child with per-runner log files. A single runner keeps
/// the legacy `runner.{stdout,stderr}.log` names; a concurrent set
/// disambiguates as `runner-<id>.{stdout,stderr}.log`.
#[allow(clippy::too_many_arguments)]
fn spawn_runner_process(
    runner: &RunnerConfig,
    cwd: &Path,
    logs_dir: &Path,
    env: &BTreeMap<String, String>,
    single: bool,
    backend: &dyn sandbox::Sandbox,
    rootfs: &Path,
) -> Result<RunnerProc> {
    let (stdout, stderr) = if single {
        (
            logs_dir.join("runner.stdout.log"),
            logs_dir.join("runner.stderr.log"),
        )
    } else {
        (
            logs_dir.join(format!("runner-{}.stdout.log", runner.id)),
            logs_dir.join(format!("runner-{}.stderr.log", runner.id)),
        )
    };
    let started = Instant::now();
    let cmd = sandbox_wrap(backend, rootfs, &runner.cmd);
    let child = command_with_logs(&cmd, cwd, env, &stdout, &stderr)
        .with_context(|| format!("spawn runner `{}`", runner.id))?;
    Ok(RunnerProc {
        runner: runner.clone(),
        deadline: runner.timeout_s.map(|s| started + Duration::from_secs(s)),
        started,
        child,
        cleanup_error: None,
        stdout_log: stdout.to_string_lossy().into_owned(),
        stderr_log: stderr.to_string_lossy().into_owned(),
    })
}

fn wait_owned_process(
    child: &mut OwnedProcessGroup,
    label: &str,
    cancellation: &RunCancellation,
) -> Result<ExitStatus> {
    loop {
        if let Some(signal) = cancellation.received() {
            if let Err(error) = child.finalize(label) {
                let cleanup_error = preserve_auxiliary_cleanup_failure(child, label, &error)
                    .unwrap_or_else(|| format!("{label} cleanup failed: {error:#}"));
                return Err(RunCleanupFailed {
                    interruption: RunInterrupted::new(signal),
                    cleanup_error,
                }
                .into());
            }
            return Err(RunInterrupted::new(signal).into());
        }
        match child.finished_status(label) {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {}
            Err(error) => {
                if let Some(cleanup_error) =
                    preserve_auxiliary_cleanup_failure(child, label, &error)
                {
                    bail!("{cleanup_error}");
                }
                return Err(error);
            }
        }
        std::thread::sleep(OWNED_GROUP_POLL_INTERVAL);
    }
}

fn preserve_auxiliary_cleanup_failure(
    child: &mut OwnedProcessGroup,
    label: &str,
    error: &anyhow::Error,
) -> Option<String> {
    if child.finalized {
        return None;
    }
    let identity = if child.leader_reaped() {
        "leader reaped; numeric PGID intentionally not retained".to_string()
    } else {
        format!("owned PGID {}", child.id())
    };
    child.preserve_cleanup_obligation();
    Some(format!(
        "{label} cleanup unconfirmed ({identity}): {error:#}"
    ))
}

fn preserve_direct_cleanup_failure(
    child: &mut OwnedProcessGroup,
    error: &anyhow::Error,
) -> Option<(Option<u32>, String)> {
    if child.finalized {
        return None;
    }
    let owned_pgid = (!child.leader_reaped()).then_some(child.id());
    let message = format!("direct VAT-owned process-group cleanup unconfirmed: {error:#}");
    child.preserve_cleanup_obligation();
    Some((owned_pgid, message))
}

/// Poll every child to completion, enforcing each runner's own timeout
/// (an elapsed budget kills that child; the others keep running).
fn wait_runner_processes(
    procs: &mut [RunnerProc],
    stop_request_path: &Path,
    cancellation: &RunCancellation,
) -> Result<Vec<RunnerRunRecord>> {
    let mut records: Vec<Option<RunnerRunRecord>> = procs.iter().map(|_| None).collect();
    loop {
        cancellation.check()?;
        if take_detached_compose_stop_request(stop_request_path) {
            // The parent is the sole authority allowed to kill this runner
            // tree. Compose merely writes the request and waits for the
            // resulting terminal VAT state before releasing its registry.
            finalize_runner_processes(procs)?;
        }
        let mut all_done = true;
        for (i, proc) in procs.iter_mut().enumerate() {
            if records[i].is_some() {
                continue;
            }
            if let Some(status) = proc
                .child
                .finished_status(&format!("runner `{}`", proc.runner.id))?
            {
                records[i] = Some(RunnerRunRecord {
                    id: proc.runner.id.clone(),
                    command: proc.runner.cmd.clone(),
                    status: ProcessStatus::Exited,
                    exit_code: Some(status.code().unwrap_or(-1)),
                    duration_ms: Some(proc.started.elapsed().as_millis() as u64),
                    pid: None,
                    cleanup_error: None,
                    stdout_log: proc.stdout_log.clone(),
                    stderr_log: proc.stderr_log.clone(),
                });
                continue;
            }
            if let Some(deadline) = proc.deadline {
                if Instant::now() >= deadline {
                    let _ = proc
                        .child
                        .finalize(&format!("timed-out runner `{}`", proc.runner.id))?;
                    records[i] = Some(RunnerRunRecord {
                        id: proc.runner.id.clone(),
                        command: proc.runner.cmd.clone(),
                        status: ProcessStatus::Timeout,
                        exit_code: Some(-1),
                        duration_ms: Some(proc.started.elapsed().as_millis() as u64),
                        pid: None,
                        cleanup_error: None,
                        stdout_log: proc.stdout_log.clone(),
                        stderr_log: proc.stderr_log.clone(),
                    });
                    continue;
                }
            }
            all_done = false;
        }
        if all_done {
            return Ok(records
                .into_iter()
                .map(|r| r.expect("all recorded"))
                .collect());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn run_setup_step(
    vat: &mut store::Vat,
    step: &crate::config::SetupStep,
    cwd: &Path,
    logs_dir: &Path,
    env: &BTreeMap<String, String>,
    backend: &dyn sandbox::Sandbox,
    rootfs: &Path,
    cancellation: &RunCancellation,
) -> Result<()> {
    let stdout = logs_dir.join(format!("setup-{}.stdout.log", step.id));
    let stderr = logs_dir.join(format!("setup-{}.stderr.log", step.id));
    let cmd = sandbox_wrap(backend, rootfs, &step.cmd);
    cancellation.check()?;
    let mut child = command_with_logs(&cmd, cwd, env, &stdout, &stderr)?;
    let status = match wait_owned_process(&mut child, &format!("setup `{}`", step.id), cancellation)
    {
        Ok(status) => status,
        Err(error) => {
            if !child.finalized {
                let cleanup_error = run_cleanup_failure(&error)
                    .map(|failure| failure.cleanup_error)
                    .unwrap_or_else(|| format!("{error:#}"));
                persist_run_auxiliary_cleanup_failure(vat, &cleanup_error).with_context(|| {
                    format!("persist setup cleanup obligation after original failure: {error:#}")
                })?;
            }
            return Err(error);
        }
    };
    if !status.success() {
        bail!("setup `{}` failed with {:?}", step.id, status.code());
    }
    vat.log(Event::new(EventKind::Setup, format!("setup {}", step.id)))?;
    Ok(())
}

fn persist_run_auxiliary_cleanup_failure(vat: &mut store::Vat, detail: &str) -> Result<()> {
    let test_run = vat
        .meta
        .test_run
        .as_mut()
        .context("configured run is missing test-run evidence")?;
    test_run.cleanup_error = Some(match test_run.cleanup_error.take() {
        Some(existing) => format!("{existing}; {detail}"),
        None => detail.to_string(),
    });
    vat.save()
        .context("persist run auxiliary cleanup obligation")
}

#[derive(Debug)]
struct ServicePlan {
    id: String,
    command: Vec<String>,
    host: Option<String>,
    ready_http: Option<String>,
    ready_probe: ReadyProbe,
    timeout_s: u64,
    preset: Option<ServicePreset>,
    port: Option<u16>,
    prepare_mode: String,
    cache_key: Option<String>,
    prepare_duration_ms: u64,
    env: BTreeMap<String, String>,
    exported_env: Vec<String>,
    /// Set when the service runs as a Docker container; carries the
    /// `--name` so teardown can force-remove the container with no orphans.
    docker_name: Option<String>,
    /// Set when the service runs via Apple's `container` CLI (MicroVM
    /// isolation); carries the `--name` so teardown can force-remove it,
    /// parallel to `docker_name`.
    microvm_name: Option<String>,
    /// The Docker image, when this service runs as a container.
    image: Option<String>,
    /// Set when the service is a local Kubernetes cluster; carries the cluster
    /// evidence so teardown can delete it and `vat state` can surface it.
    cluster: Option<ClusterRunRecord>,
    /// False when the service is provided by CI/local infrastructure and vat is
    /// attaching to it instead of starting/stopping a process.
    owned_by_vat: bool,
    /// True for native command-backed services and Docker's foreground
    /// `start --attach` child, both of which must remain live through endpoint
    /// readiness. Cluster, external, and other launcher-style runtimes preserve
    /// their own readiness/lifecycle semantics.
    requires_live_child: bool,
    /// Literal 127.0.0.1 endpoints held exclusively from planning until the
    /// instant the owned child is spawned. External/attach services never
    /// reserve ports.
    endpoint_reservations: Vec<EndpointReservation>,
}

#[derive(Debug)]
struct EndpointReservation {
    endpoint: SocketAddr,
    listener: TcpListener,
}

#[derive(Debug, Clone)]
enum ReadyProbe {
    None,
    Http(String),
    Tcp {
        host: String,
        port: u16,
    },
    /// A MicroVM-specific HTTP contract. Unlike Docker/native HTTP probes,
    /// terminal endpoint errors are preserved as MicroVM host-port evidence.
    MicroVmHttp(String),
    /// A MicroVM-specific TCP contract. A completed handshake is not enough:
    /// an immediate EOF or reset proves the published host endpoint unusable.
    MicroVmTcp {
        host: String,
        port: u16,
    },
    Cmd(Vec<String>),
}

/// A MicroVM readiness observation distinguishes a retryable published-port
/// state from a terminal endpoint failure. The pending message is persisted on
/// timeout so VAT reports the last concrete observation rather than a generic
/// readiness deadline.
#[derive(Debug, Clone, PartialEq, Eq)]
enum EndpointReadiness {
    Ready,
    Pending(String),
}

struct ServiceHandle {
    record: ServiceRunRecord,
    child: Option<OwnedProcessGroup>,
    timeout_s: u64,
    ready_probe: ReadyProbe,
    /// Endpoints VAT proved unavailable immediately before spawning this
    /// owned child. Readiness requires each one to become reachable afterward.
    owned_endpoints: Vec<SocketAddr>,
    /// Whether this handle owns a native or Docker-attach service child that
    /// must remain live until its endpoint completes readiness.
    requires_live_child: bool,
    /// `docker --name` when the service is a container; force-removed on stop.
    docker_name: Option<String>,
    /// `container --name` when the service runs via Apple's `container` CLI
    /// (MicroVM isolation); force-removed on stop, parallel to `docker_name`.
    microvm_name: Option<String>,
    /// Cluster evidence when the service is a local Kubernetes cluster; the
    /// cluster is deleted on stop subject to the `keep` policy.
    cluster: Option<ClusterRunRecord>,
    /// Runtime helper groups whose shared-deadline cleanup could not be
    /// confirmed. The live handle retains ownership until the corresponding
    /// cleanup_error has been durably persisted; only then may a later
    /// best-effort finalizer release the in-memory owner.
    deadline_cleanup_owners: Vec<OwnedProcessGroup>,
}

fn prepare_service(
    vat: &mut store::Vat,
    cfg: &VatConfig,
    service: &ServiceConfig,
    force_hermetic_proxy: bool,
    cancellation: &RunCancellation,
) -> Result<ServicePlan> {
    let started = Instant::now();
    let plan = if let Some(backend) = service.cluster {
        // Cluster: an ephemeral local Kubernetes cluster (kind / k3d / minikube).
        // Created here in the prepare phase; the runner reaches it via KUBECONFIG.
        prepare_cluster_service(vat, service, backend, cancellation)?
    } else if let Some(image) = &service.image {
        // Explicit image: a container-backed service (e.g. AlloyDB) with no
        // native equivalent. `runtime: microvm` routes to Apple's `container`
        // CLI; every other runtime (Auto, Docker, Native) keeps today's
        // `docker run` path unchanged (R4/R5).
        match service.runtime {
            ServiceRuntime::MicroVm => prepare_microvm_service(vat, service, image, cancellation)?,
            _ => prepare_image_service(vat, service, image, cancellation)?,
        }
    } else if service.external.is_some() {
        prepare_external_service(service)?
    } else if service.preset == Some(ServicePreset::Firebase) {
        // Firebase: a multi-emulator bundle driven by firebase.json — its own
        // prepare path because it is one process exposing many ports.
        if matches!(service.runtime, ServiceRuntime::MicroVm) {
            reject_unsupported_microvm_preset(service, ServicePreset::Firebase)?;
        }
        prepare_firebase_service(vat, cfg, service)?
    } else if let Some(preset) = service.preset {
        // Preset: prefer the native Homebrew binary, fall back to the preset's
        // official container image when the binary is missing (or as forced).
        // An explicit MicroVM runtime must remain entirely on Apple's
        // `container` CLI; it may never silently route through Docker.
        match resolve_preset_runtime(service, preset, cancellation)? {
            ResolvedRuntime::Native => {
                prepare_preset_service(vat, cfg, service, preset, cancellation)?
            }
            ResolvedRuntime::Docker => {
                prepare_preset_docker_service(vat, service, preset, cancellation)?
            }
            ResolvedRuntime::MicroVm => {
                prepare_preset_microvm_service(vat, service, preset, cancellation)?
            }
            ResolvedRuntime::Builtin => {
                // Hermetic when the run confines egress (localhost-only/deny):
                // the http-mock proxy then blocks unmatched requests too.
                let hermetic = force_hermetic_proxy
                    || !matches!(
                        cfg.network.as_ref().map(|n| n.egress).unwrap_or_default(),
                        crate::spec::EgressPolicy::Open
                    );
                prepare_builtin_service(
                    service,
                    preset,
                    &cfg.root,
                    &explicit_network_routes(cfg),
                    hermetic,
                )?
            }
        }
    } else {
        let reservation = command_service_port(service)?;
        let port = reservation.as_ref().map(EndpointReservation::port);
        let command = substitute_service_port(&service.cmd, port);
        let ready_http = service
            .ready_http
            .as_ref()
            .map(|value| substitute_port(value, port));
        let ready_cmd = substitute_service_port(&service.ready_cmd, port);
        let mut service_for_probe = service.clone();
        service_for_probe.ready_http = ready_http.clone();
        service_for_probe.ready_cmd = ready_cmd;
        let env = export_command_service_env(&service_for_probe, port);
        let default_probe = port.map(|port| ReadyProbe::Tcp {
            host: "127.0.0.1".to_string(),
            port,
        });
        ServicePlan {
            id: service.id.clone(),
            command,
            host: Some("127.0.0.1".to_string()).filter(|_| port.is_some()),
            ready_http,
            ready_probe: resolve_ready_probe(&service_for_probe, default_probe),
            timeout_s: service.timeout_s,
            preset: None,
            port,
            prepare_mode: "direct_start".to_string(),
            cache_key: None,
            prepare_duration_ms: 0,
            exported_env: sorted_keys(&env),
            env,
            docker_name: None,
            microvm_name: None,
            image: None,
            cluster: None,
            owned_by_vat: true,
            requires_live_child: true,
            endpoint_reservations: reservation.into_iter().collect(),
        }
    };
    let mut plan = plan;
    plan.prepare_duration_ms = started.elapsed().as_millis() as u64;
    // Cluster services emit their own prepare checkpoint inside
    // `prepare_cluster_service`; the container/preset note below does not apply.
    if plan.prepare_mode != "direct_start" && plan.cluster.is_none() {
        let is_docker = plan.docker_name.is_some();
        let is_microvm = plan.microvm_name.is_some();
        let runtime = if !plan.owned_by_vat {
            "external"
        } else if is_docker {
            "docker"
        } else if is_microvm {
            "microvm"
        } else {
            "native"
        };
        let note = if !plan.owned_by_vat {
            "using external service endpoint (not started or stopped by vat)"
        } else if is_docker {
            "running service via `docker run` (ephemeral, --rm)"
        } else if is_microvm {
            "running service via `container run` (ephemeral, --rm, MicroVM isolation)"
        } else if plan.prepare_mode == "cold_build" {
            "first run slower; cached for future runs"
        } else {
            "using cached service image"
        };
        emit_jsonl(serde_json::json!({
            "type": "prepare",
            "service": plan.id.as_str(),
            "preset": plan.preset.map(service_preset_name),
            "runtime": runtime,
            "image": plan.image.as_deref(),
            "host": plan.host.as_deref(),
            "port": plan.port,
            "owned_by_vat": plan.owned_by_vat,
            "mode": plan.prepare_mode.as_str(),
            "cache_key": plan.cache_key.as_deref(),
            "note": note,
        }))?;
    }
    Ok(plan)
}

/// Prepare a `cluster` service: resolve a backend, create an ephemeral local
/// Kubernetes cluster with an isolated kubeconfig, and model it as a run-scoped
/// service whose readiness is `kubectl get nodes`. The cluster is created here
/// (a one-shot, minutes-long operation) and kept alive by a trivial child so it
/// slots into the existing service start/stop machinery; the runner reaches it
/// through the exported `KUBECONFIG`.
/// @spec apps/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#logic
fn prepare_cluster_service(
    vat: &mut store::Vat,
    service: &ServiceConfig,
    backend: ClusterBackend,
    cancellation: &RunCancellation,
) -> Result<ServicePlan> {
    cancellation.check()?;
    let resolved = match cluster::resolve_backend(backend) {
        Ok(resolved) => resolved,
        Err(unavailable) => {
            emit_jsonl(serde_json::json!({
                "type": "error",
                "code": "cluster_backend_unavailable",
                "service": service.id.as_str(),
                "requested": unavailable.requested_name(),
                "installed": unavailable.installed,
                "docker": unavailable.docker,
            }))?;
            bail!(
                "service `{}` cluster: {}",
                service.id,
                unavailable.message()
            );
        }
    };

    let name = cluster::cluster_name(&vat.meta.id, &service.id);
    let kubeconfig = vat
        .dir
        .join("services")
        .join(&service.id)
        .join("kubeconfig");
    let nodes = service.nodes.unwrap_or(1);

    emit_jsonl(serde_json::json!({
        "type": "prepare",
        "service": service.id.as_str(),
        "kind": "cluster",
        "backend": resolved.name(),
        "note": "creating local Kubernetes cluster (may take minutes)",
    }))?;

    let spec = ClusterSpec {
        name: &name,
        k8s_version: service.k8s_version.as_deref(),
        nodes,
        kubeconfig: &kubeconfig,
    };
    let info =
        match resolved.create_cancellable(&spec, Duration::from_secs(service.timeout_s), &|| {
            cancellation.check()
        }) {
            Ok(info) => info,
            Err(err) => {
                let create_error = format!("{err:#}");
                let interruption = run_interruption(&err)
                    .or_else(|| cancellation.received().map(RunInterrupted::new));
                let command_cleanup_error =
                    cluster::owned_command_cleanup_failure(&err).map(str::to_string);
                // A backend can fail after creating enough resources for its
                // cluster name to remain live. If the compensating delete also
                // fails, persist that exact ownership obligation before returning:
                // no ServicePlan exists yet for the outer finalizer to discover.
                let delete_error = resolved.delete(&name).err();
                let mut cleanup_errors = Vec::new();
                if let Some(error) = command_cleanup_error {
                    cleanup_errors.push(format!(
                        "cluster create command owned process-group cleanup unconfirmed: {error}"
                    ));
                }
                if let Some(error) = delete_error {
                    cleanup_errors.push(format!(
                        "cluster `{name}` ({}) cleanup unconfirmed after create failure: {error:#}",
                        resolved.name()
                    ));
                }
                if !cleanup_errors.is_empty() {
                    let cleanup_error = cleanup_errors.join("; ");
                    let cluster = ClusterRunRecord {
                        backend: resolved.name().to_string(),
                        name: name.clone(),
                        kubeconfig: kubeconfig.to_string_lossy().into_owned(),
                        node_count: nodes,
                        ready_ms: None,
                    };
                    persist_cluster_create_cleanup_failure(
                        vat,
                        service,
                        cluster,
                        cleanup_error.clone(),
                    )
                    .with_context(|| {
                        format!(
                        "persist cluster cleanup obligation after create failed: {create_error}"
                    )
                    })?;
                    emit_jsonl(serde_json::json!({
                        "type": "error",
                        "code": "cluster_create_failed",
                        "service": service.id.as_str(),
                        "backend": resolved.name(),
                        "reason": create_error,
                        "cleanup_error": cleanup_error,
                    }))?;
                    if let Some(interruption) = interruption {
                        let cleanup_failure: anyhow::Error = RunCleanupFailed {
                            interruption,
                            cleanup_error,
                        }
                        .into();
                        return Err(cleanup_failure).with_context(|| {
                            format!("create cluster for service `{}` failed", service.id)
                        });
                    }
                    bail!(
                        "create cluster for service `{}` failed and cleanup is unconfirmed: {}",
                        service.id,
                        cleanup_error
                    );
                }
                emit_jsonl(serde_json::json!({
                    "type": "error",
                    "code": "cluster_create_failed",
                    "service": service.id.as_str(),
                    "backend": resolved.name(),
                    "reason": create_error,
                }))?;
                if let Some(interruption) = interruption {
                    let interruption: anyhow::Error = interruption.into();
                    return Err(interruption).with_context(|| {
                        format!("create cluster for service `{}` failed", service.id)
                    });
                }
                return Err(err)
                    .with_context(|| format!("create cluster for service `{}`", service.id));
            }
        };

    let kubeconfig_str = info.kubeconfig.to_string_lossy().into_owned();
    let mut env = BTreeMap::new();
    for (key, template) in &service.export {
        env.insert(
            key.clone(),
            template.replace("{kubeconfig}", &kubeconfig_str),
        );
    }
    env.insert("KUBECONFIG".to_string(), kubeconfig_str.clone());
    let upper = service.id.to_uppercase().replace(['-', '.'], "_");
    env.insert(
        format!("VAT_SERVICE_{upper}_KUBECONFIG"),
        kubeconfig_str.clone(),
    );

    let record = ClusterRunRecord {
        backend: info.backend.to_string(),
        name: info.name.clone(),
        kubeconfig: kubeconfig_str,
        node_count: info.node_count,
        ready_ms: None,
    };

    Ok(ServicePlan {
        id: service.id.clone(),
        command: vec![
            "sh".to_string(),
            "-c".to_string(),
            "while :; do sleep 3600; done".to_string(),
        ],
        host: None,
        ready_http: None,
        ready_probe: ReadyProbe::Cmd(resolved.ready_argv(&info.kubeconfig)),
        timeout_s: service.timeout_s,
        preset: None,
        port: None,
        prepare_mode: "cluster_create".to_string(),
        cache_key: None,
        prepare_duration_ms: 0,
        exported_env: sorted_keys(&env),
        env,
        docker_name: None,
        microvm_name: None,
        image: None,
        cluster: Some(record),
        owned_by_vat: true,
        requires_live_child: false,
        endpoint_reservations: Vec::new(),
    })
}

fn persist_cluster_create_cleanup_failure(
    vat: &mut store::Vat,
    service: &ServiceConfig,
    cluster: ClusterRunRecord,
    cleanup_error: String,
) -> Result<()> {
    let stdout_log = vat
        .dir
        .join(crate::paths::file::LOGS)
        .join(format!("{}.stdout.log", service.id))
        .to_string_lossy()
        .into_owned();
    let stderr_log = vat
        .dir
        .join(crate::paths::file::LOGS)
        .join(format!("{}.stderr.log", service.id))
        .to_string_lossy()
        .into_owned();
    let record = ServiceRunRecord {
        id: service.id.clone(),
        command: Vec::new(),
        status: ProcessStatus::Failed,
        preset: None,
        host: None,
        port: None,
        owned_by_vat: Some(true),
        prepare_mode: Some("cluster_create".to_string()),
        cache_key: None,
        prepare_duration_ms: None,
        ready_duration_ms: None,
        exported_env: Vec::new(),
        pid: None,
        exit_code: None,
        ready_http: None,
        docker_name: None,
        docker_id: None,
        microvm_name: None,
        readiness_error: None,
        cleanup_error: Some(cleanup_error),
        cluster: Some(cluster),
        stdout_log,
        stderr_log,
    };
    persist_service_record(vat, record).context("persist half-created cluster cleanup obligation")
}

/// vat's own spawned services (emulators, the http-mock/record-replay proxy)
/// are intentionally NEVER sandboxed on the real (non-hermetic-proxy) path —
/// they need their own network access to serve/forward regardless of the
/// run's `--isolation`/`[network].egress`. The single exception is the
/// hermetic-proxy mode, where the proxy itself is deliberately wrapped so it
/// becomes the sole egress point; that mode is unrelated to this WI's
/// runner-mode coverage and is unchanged here. See #1301 (R2/AC2): this
/// helper is the explicit, testable decision point for that exemption,
/// rather than an inlined `if` at the `start_service` call site.
fn service_sandbox_backend(
    force_hermetic_proxy: bool,
    backend: &dyn sandbox::Sandbox,
) -> Option<&dyn sandbox::Sandbox> {
    if force_hermetic_proxy {
        Some(backend)
    } else {
        None
    }
}

fn start_service(
    vat: &mut store::Vat,
    plan: &mut ServicePlan,
    cwd: &Path,
    logs_dir: &Path,
    env: &BTreeMap<String, String>,
    service_sandbox: Option<&dyn sandbox::Sandbox>,
    rootfs: &Path,
    cancellation: &RunCancellation,
) -> Result<ServiceHandle> {
    let stdout = logs_dir.join(format!("{}.stdout.log", plan.id));
    let stderr = logs_dir.join(format!("{}.stderr.log", plan.id));
    let prepared_command = if plan.owned_by_vat {
        service_start_command(plan, service_sandbox, rootfs)
    } else {
        Vec::new()
    };
    let owned_endpoints = if plan.owned_by_vat {
        release_endpoint_reservations_at_spawn(plan)?
    } else {
        Vec::new()
    };
    let docker_id = if plan.docker_name.is_some() {
        Some(create_docker_service_id(
            &prepared_command,
            Duration::from_secs(plan.timeout_s.max(2)),
            cancellation,
            &plan.id,
        )?)
    } else {
        None
    };
    let command = docker_id
        .as_deref()
        .map(docker_start_command)
        .unwrap_or(prepared_command);
    let child = if plan.owned_by_vat && docker_id.is_none() {
        Some(
            command_with_logs(&command, cwd, env, &stdout, &stderr)
                .with_context(|| format!("spawn service `{}`", plan.id))?,
        )
    } else {
        None
    };
    let record = ServiceRunRecord {
        id: plan.id.clone(),
        command: command.clone(),
        status: if docker_id.is_some() {
            ProcessStatus::Created
        } else {
            ProcessStatus::Running
        },
        preset: plan.preset.map(service_preset_name).map(str::to_string),
        host: plan.host.clone(),
        port: plan.port,
        owned_by_vat: Some(plan.owned_by_vat),
        prepare_mode: Some(plan.prepare_mode.clone()),
        cache_key: plan.cache_key.clone(),
        prepare_duration_ms: Some(plan.prepare_duration_ms),
        ready_duration_ms: None,
        exported_env: plan.exported_env.clone(),
        pid: child.as_ref().map(OwnedProcessGroup::id),
        exit_code: None,
        ready_http: plan.ready_http.clone(),
        docker_name: plan.docker_name.clone(),
        docker_id,
        microvm_name: plan.microvm_name.clone(),
        readiness_error: None,
        cleanup_error: None,
        cluster: plan.cluster.clone(),
        stdout_log: stdout.to_string_lossy().into_owned(),
        stderr_log: stderr.to_string_lossy().into_owned(),
    };
    let mut handle = ServiceHandle {
        record,
        child,
        timeout_s: plan.timeout_s,
        ready_probe: plan.ready_probe.clone(),
        owned_endpoints,
        requires_live_child: plan.requires_live_child,
        docker_name: plan.docker_name.clone(),
        microvm_name: plan.microvm_name.clone(),
        // Ownership transfers as soon as the child has spawned, before any
        // fallible logging/persistence. The prepared-plan finalizer must not
        // independently delete the same cluster afterward.
        cluster: plan.cluster.take(),
        deadline_cleanup_owners: Vec::new(),
    };
    if handle.record.docker_id.is_some() {
        // `docker create` stdout is the immutable ownership boundary. Persist
        // Created/name/full-ID before starting the foreground attachment so a
        // start/spawn/persistence failure can clean only that exact object.
        if let Err(error) = persist_service_record(vat, handle.record.clone()) {
            return fail_service_start(vat, &mut handle, error, "persist created Docker identity");
        }
        match command_with_logs(&command, cwd, env, &stdout, &stderr) {
            Ok(child) => {
                handle.record.pid = Some(child.id());
                handle.child = Some(child);
            }
            Err(error) => {
                return fail_service_start(
                    vat,
                    &mut handle,
                    error,
                    "spawn foreground Docker start attachment",
                );
            }
        }
        if let Err(error) = wait_for_docker_running_ack(
            &mut handle,
            Duration::from_secs(plan.timeout_s.max(2)),
            cancellation,
            &detached_compose_stop_request_path(vat),
        ) {
            return fail_service_start(
                vat,
                &mut handle,
                error,
                "acknowledge foreground Docker start attachment",
            );
        }
        handle.record.status = ProcessStatus::Running;
        if let Err(error) = persist_service_record(vat, handle.record.clone()) {
            return fail_service_start(vat, &mut handle, error, "persist running Docker owner");
        }
    }
    if let Err(log_error) = vat.log(Event::new(
        EventKind::RunStarted,
        if plan.owned_by_vat {
            format!("service {}", plan.id)
        } else {
            format!("service {} external", plan.id)
        },
    )) {
        if handle.child.is_none() {
            handle.record.status = ProcessStatus::Failed;
        }
        let cleanup = stop_services(std::slice::from_mut(&mut handle), true);
        let evidence = persist_service_record(vat, handle.record.clone());
        if evidence.is_ok() {
            release_persisted_deadline_cleanup_owners(&mut handle.deadline_cleanup_owners);
        }
        return match (cleanup, evidence) {
            (Ok(()), Ok(())) => Err(log_error),
            (Err(cleanup), Ok(())) => Err(cleanup).context(format!(
                "service start log also failed before cleanup: {log_error:#}"
            )),
            (Ok(()), Err(evidence)) => Err(evidence).context(format!(
                "service start log also failed before evidence persistence: {log_error:#}"
            )),
            (Err(cleanup), Err(evidence)) => Err(evidence).context(format!(
                "service start log failed ({log_error:#}) and cleanup was unconfirmed ({cleanup:#})"
            )),
        };
    }
    Ok(handle)
}

fn create_docker_service_id(
    command: &[String],
    timeout: Duration,
    cancellation: &RunCancellation,
    service_id: &str,
) -> Result<String> {
    let (program, args) = command
        .split_first()
        .context("VAT-owned Docker create command is empty")?;
    let args = args.iter().map(String::as_str).collect::<Vec<_>>();
    let (status, output) = cancellable_command_output(
        program,
        &args,
        timeout,
        cancellation,
        &format!("Docker create for service `{service_id}`"),
    )?;
    if !status.success() {
        bail!(
            "Docker create for service `{service_id}` exited unsuccessfully ({:?})",
            status.code()
        );
    }
    let output = String::from_utf8(output)
        .context("Docker create stdout was not valid UTF-8 full-ID evidence")?;
    let rows = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    if rows.len() != 1 || !valid_full_docker_id(rows[0]) {
        bail!(
            "Docker create for service `{service_id}` did not emit exactly one lowercase 64-hex full container ID"
        );
    }
    Ok(rows[0].to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DockerStartTransition {
    PendingCreated,
    RunningAcknowledged,
    Failed(String),
}

/// Convert the strict immutable-identity observation into the only legal
/// Created -> Running transition. A reusable name is never startup authority:
/// only the same full ID in Docker's `running` state can acknowledge the
/// foreground attachment.
fn docker_start_transition(observation: &DockerIdentityObservation) -> DockerStartTransition {
    match observation {
        DockerIdentityObservation::Exact { state } if state == "created" => {
            DockerStartTransition::PendingCreated
        }
        DockerIdentityObservation::Exact { state } if state == "running" => {
            DockerStartTransition::RunningAcknowledged
        }
        DockerIdentityObservation::Exact { state } => DockerStartTransition::Failed(format!(
            "same-ID Docker object entered `{state}` before startup acknowledgement"
        )),
        DockerIdentityObservation::Absent => DockerStartTransition::Failed(
            "same-ID Docker object disappeared before startup acknowledgement".to_string(),
        ),
        DockerIdentityObservation::Replacement { actual_id } => {
            DockerStartTransition::Failed(format!(
                "Docker name belongs to replacement ID `{actual_id}` before startup acknowledgement"
            ))
        }
    }
}

/// Keep the durable service checkpoint at Created until Docker itself reports
/// the exact immutable ID as running and the foreground `start --attach`
/// owner is still live. Identity queries, cancellation/child observations,
/// polling, and helper process-group cleanup all consume one absolute startup
/// deadline; no retry resets it.
fn wait_for_docker_running_ack(
    handle: &mut ServiceHandle,
    timeout: Duration,
    cancellation: &RunCancellation,
    compose_stop_request: &Path,
) -> Result<()> {
    let name = handle
        .record
        .docker_name
        .clone()
        .context("Docker start acknowledgement is missing the durable container name")?;
    let docker_id = handle
        .record
        .docker_id
        .clone()
        .context("Docker start acknowledgement is missing the durable full container ID")?;
    let deadline = Instant::now() + timeout;

    loop {
        cancellation.check()?;
        consume_detached_compose_stop_request(
            compose_stop_request,
            "Docker startup acknowledgement",
        )?;
        if let Some(status) = service_child_exit_status(handle)? {
            handle.record.exit_code = status.code();
            bail!(
                "foreground Docker start attachment for `{name}` / `{docker_id}` exited {:?} before running acknowledgement",
                status.code()
            );
        }
        if Instant::now() >= deadline {
            bail!(
                "Docker start for `{name}` / `{docker_id}` did not reach the exact-ID running state within {}ms",
                timeout.as_millis()
            );
        }

        let observation = observe_docker_identity(
            &name,
            &docker_id,
            deadline,
            Duration::ZERO,
            &mut handle.deadline_cleanup_owners,
        )
        .with_context(|| {
            format!(
                "observe exact Docker identity `{name}` / `{docker_id}` during startup acknowledgement"
            )
        })?;
        match docker_start_transition(&observation) {
            DockerStartTransition::PendingCreated => {}
            DockerStartTransition::RunningAcknowledged => {
                cancellation.check()?;
                if let Some(status) = service_child_exit_status(handle)? {
                    handle.record.exit_code = status.code();
                    bail!(
                        "foreground Docker start attachment for `{name}` / `{docker_id}` exited {:?} while acknowledging running state",
                        status.code()
                    );
                }
                return Ok(());
            }
            DockerStartTransition::Failed(reason) => {
                bail!("Docker start for `{name}` / `{docker_id}` failed: {reason}");
            }
        }

        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            bail!(
                "Docker start for `{name}` / `{docker_id}` did not reach the exact-ID running state within {}ms",
                timeout.as_millis()
            );
        }
        std::thread::sleep(remaining.min(OWNED_GROUP_POLL_INTERVAL));
    }
}

fn fail_service_start(
    vat: &mut store::Vat,
    handle: &mut ServiceHandle,
    primary: anyhow::Error,
    phase: &str,
) -> Result<ServiceHandle> {
    handle.record.status = ProcessStatus::Failed;
    let cleanup = stop_services(std::slice::from_mut(handle), true);
    let evidence = persist_service_record(vat, handle.record.clone());
    if evidence.is_ok() {
        release_persisted_deadline_cleanup_owners(&mut handle.deadline_cleanup_owners);
    }
    match (cleanup, evidence) {
        (Ok(()), Ok(())) => Err(primary).context(phase.to_string()),
        (Err(cleanup), Ok(())) => Err(cleanup).context(format!("{phase} also failed: {primary:#}")),
        (Ok(()), Err(evidence)) => Err(evidence).context(format!(
            "{phase} failed before evidence persistence: {primary:#}"
        )),
        (Err(cleanup), Err(evidence)) => Err(evidence).context(format!(
            "{phase} failed ({primary:#}) and exact-ID cleanup was unconfirmed ({cleanup:#})"
        )),
    }
}

fn valid_full_docker_id(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn service_start_command(
    plan: &ServicePlan,
    service_sandbox: Option<&dyn sandbox::Sandbox>,
    rootfs: &Path,
) -> Vec<String> {
    if plan.prepare_mode == "direct_start" {
        service_sandbox
            .map(|backend| sandbox_wrap(backend, rootfs, &plan.command))
            .unwrap_or_else(|| plan.command.clone())
    } else {
        plan.command.clone()
    }
}

fn prepare_external_service(service: &ServiceConfig) -> Result<ServicePlan> {
    let endpoint = service
        .external
        .as_ref()
        .context("external service missing endpoint (validated earlier)")?;
    let host = endpoint.host.clone();
    let port = endpoint.port;
    let ready_http = service
        .ready_http
        .as_ref()
        .map(|value| substitute_endpoint(value, &host, port));
    let ready_cmd = substitute_endpoint_values(&service.ready_cmd, &host, port);
    let mut service_for_probe = service.clone();
    service_for_probe.ready_http = ready_http.clone();
    service_for_probe.ready_cmd = ready_cmd;
    let ready_probe = resolve_ready_probe(
        &service_for_probe,
        Some(ReadyProbe::Tcp {
            host: host.clone(),
            port,
        }),
    );
    let env = external_exports(service, &host, port);

    Ok(ServicePlan {
        id: service.id.clone(),
        command: Vec::new(),
        host: Some(host),
        ready_http,
        ready_probe,
        timeout_s: service.timeout_s,
        preset: None,
        port: Some(port),
        prepare_mode: "external_attach".to_string(),
        cache_key: None,
        prepare_duration_ms: 0,
        exported_env: sorted_keys(&env),
        env,
        docker_name: None,
        microvm_name: None,
        image: None,
        cluster: None,
        owned_by_vat: false,
        requires_live_child: false,
        endpoint_reservations: Vec::new(),
    })
}

// <HANDWRITE gap="vat-versioned-native-lumen-preset-runtime" tracker="#1813" reason="Build native Lumen service plans and fail closed for container runtimes.">
fn prepare_preset_service(
    vat: &store::Vat,
    cfg: &VatConfig,
    service: &ServiceConfig,
    preset: ServicePreset,
    cancellation: &RunCancellation,
) -> Result<ServicePlan> {
    if preset == ServicePreset::Lumen {
        let reservation = reserve_native_service_port(&service.id, &service.port)?;
        let port = reservation.port();
        let lumen = lumen_release::resolve(service.version.as_deref())?;
        let env = preset_exports(service, preset, port);
        let command = vec![
            lumen.executable.to_string_lossy().into_owned(),
            "serve".into(),
            "--host".into(),
            "127.0.0.1".into(),
            "--port".into(),
            port.to_string(),
        ];
        let ready_probe = resolve_ready_probe(
            service,
            Some(ReadyProbe::Http(format!("http://127.0.0.1:{port}/readyz"))),
        );
        return Ok(ServicePlan {
            id: service.id.clone(),
            command,
            host: Some("127.0.0.1".into()),
            ready_http: service.ready_http.clone(),
            ready_probe,
            timeout_s: service.timeout_s,
            preset: Some(preset),
            port: Some(port),
            prepare_mode: "lumen_native_cache".into(),
            cache_key: Some(lumen.tag),
            prepare_duration_ms: 0,
            exported_env: sorted_keys(&env),
            env,
            docker_name: None,
            microvm_name: None,
            image: None,
            cluster: None,
            owned_by_vat: true,
            requires_live_child: true,
            endpoint_reservations: vec![reservation],
        });
    }
    ensure_preset_binaries(service, preset)?;
    let reservation = reserve_native_service_port(&service.id, &service.port)?;
    let port = reservation.port();
    let cache_key = service_cache_key(cfg, service, preset)?;
    let cache_dir = crate::paths::root()?
        .join("cache")
        .join("services")
        .join(&cache_key);
    let data_dir = vat.dir.join("services").join(&service.id).join("data");
    let prepare_mode = if preset_uses_service_image(preset) {
        if cache_dir.exists() {
            if data_dir.exists() {
                std::fs::remove_dir_all(&data_dir)
                    .with_context(|| format!("remove {}", data_dir.display()))?;
            }
            overlay::clone_tree(&cache_dir, &data_dir)
                .with_context(|| format!("clone service image {}", cache_key))?;
            "warm_clone"
        } else {
            let cache_parent = cache_dir
                .parent()
                .context("service cache directory has no parent")?;
            let tmp_cache_dir = cache_parent.join(format!("{}.tmp-{}", cache_key, vat.meta.id));
            if tmp_cache_dir.exists() {
                std::fs::remove_dir_all(&tmp_cache_dir)
                    .with_context(|| format!("remove {}", tmp_cache_dir.display()))?;
            }
            std::fs::create_dir_all(&tmp_cache_dir)
                .with_context(|| format!("create {}", tmp_cache_dir.display()))?;
            if let Err(err) =
                cold_prepare_service_image(cfg, service, preset, &tmp_cache_dir, cancellation)
            {
                let _ = std::fs::remove_dir_all(&tmp_cache_dir);
                return Err(err);
            }
            std::fs::rename(&tmp_cache_dir, &cache_dir).with_context(|| {
                format!(
                    "promote service image cache {} to {}",
                    tmp_cache_dir.display(),
                    cache_dir.display()
                )
            })?;
            if data_dir.exists() {
                std::fs::remove_dir_all(&data_dir)
                    .with_context(|| format!("remove {}", data_dir.display()))?;
            }
            overlay::clone_tree(&cache_dir, &data_dir)
                .with_context(|| format!("clone service image {}", cache_key))?;
            "cold_build"
        }
    } else {
        std::fs::create_dir_all(&data_dir)
            .with_context(|| format!("create {}", data_dir.display()))?;
        "direct_start"
    };
    let mut env = preset_exports(service, preset, port);
    add_service_runtime_env(&mut env, preset, &service.id, port, &data_dir);
    let command = preset_command(preset, port, &data_dir);
    // A corpus-aware `ready_cmd` (e.g. a SQL row-count check) wins over the
    // preset's default "server accepts connections" probe so readiness means
    // "corpus loaded". `ready_http` is the next override; otherwise the preset
    // default applies.
    let ready_probe = resolve_ready_probe(service, Some(preset_ready_probe(preset, port)));
    Ok(ServicePlan {
        id: service.id.clone(),
        command,
        host: Some("127.0.0.1".to_string()),
        ready_http: service.ready_http.clone(),
        ready_probe,
        timeout_s: service.timeout_s,
        preset: Some(preset),
        port: Some(port),
        prepare_mode: prepare_mode.to_string(),
        cache_key: Some(cache_key),
        prepare_duration_ms: 0,
        exported_env: sorted_keys(&env),
        env,
        docker_name: None,
        microvm_name: None,
        image: None,
        cluster: None,
        owned_by_vat: true,
        requires_live_child: true,
        endpoint_reservations: vec![reservation],
    })
}
// </HANDWRITE>

/// Which way a `preset` service is actually provided on this host.
#[derive(Debug)]
enum ResolvedRuntime {
    Native,
    Docker,
    MicroVm,
    /// vat's own in-process Rust emulator (the `vat emulator` subcommand).
    Builtin,
}

/// Resolve a preset service's `runtime` against the host. `auto` prefers the
/// native binary and falls back to Docker; `native`/`docker`/`microvm` force
/// their named path. On `auto` with neither available, emit a structured error
/// and bail. An explicit MicroVM path never falls back to Docker.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
fn resolve_preset_runtime(
    service: &ServiceConfig,
    preset: ServicePreset,
    cancellation: &RunCancellation,
) -> Result<ResolvedRuntime> {
    if preset == ServicePreset::Lumen {
        if matches!(
            service.runtime,
            ServiceRuntime::Auto | ServiceRuntime::Native
        ) {
            return Ok(ResolvedRuntime::Native);
        }
        bail!(
            "service `{}` preset `lumen` is native-only; Docker and MicroVM are unsupported",
            service.id
        );
    }
    match service.runtime {
        ServiceRuntime::Native => Ok(ResolvedRuntime::Native),
        ServiceRuntime::Docker => Ok(ResolvedRuntime::Docker),
        ServiceRuntime::MicroVm => {
            if !preset_has_microvm_image_route(preset) {
                reject_unsupported_microvm_preset(service, preset)?;
            }
            if !service.volumes.is_empty() {
                reject_microvm_preset_volumes(service, preset)?;
            }
            Ok(ResolvedRuntime::MicroVm)
        }
        // A preset with a built-in Rust emulator runs vat's own server under
        // `auto` — always available, no external tooling.
        ServiceRuntime::Auto if preset.is_builtin() => Ok(ResolvedRuntime::Builtin),
        ServiceRuntime::Auto => {
            // Native means more than "binary on PATH" for emulators: the gcloud
            // component must also be installed, else native would be chosen and
            // then fail to start. preset_native_available folds that in so a
            // missing component falls back to Docker.
            let has_binaries = required_binaries(preset)
                .iter()
                .all(|binary| which(binary).is_some());
            let component = gcloud_component(preset);
            let installed_components = if component.is_some() {
                installed_gcloud_components(cancellation)?
            } else {
                Vec::new()
            };
            if native_available(has_binaries, component, &installed_components) {
                Ok(ResolvedRuntime::Native)
            } else if docker_available(cancellation)? {
                Ok(ResolvedRuntime::Docker)
            } else {
                let missing: Vec<&str> = required_binaries(preset)
                    .iter()
                    .filter(|binary| which(binary).is_none())
                    .copied()
                    .collect();
                let missing_component = component
                    .filter(|c| !installed_components.iter().any(|installed| installed == c));
                emit_jsonl(serde_json::json!({
                    "type": "error",
                    "code": "service_runtime_unavailable",
                    "service": service.id.as_str(),
                    "preset": service_preset_name(preset),
                    "missing_native": missing,
                    "missing_component": missing_component,
                    "docker": false,
                }))?;
                bail!(
                    "service `{}` preset `{}`: native unavailable (missing binaries: [{}]{}) and Docker is unavailable; \
                     install them, install the gcloud component, install/start Docker, or set runtime explicitly",
                    service.id,
                    service_preset_name(preset),
                    missing.join(", "),
                    missing_component
                        .map(|c| format!(", missing component: {c}"))
                        .unwrap_or_default(),
                );
            }
        }
    }
}

/// Whether a preset has an explicit OCI image route that VAT can ask Apple's
/// `container` CLI to start. Built-in Rust emulators and the Firebase bundle
/// have their own local-process implementations; pretending that a generic
/// Node image provides parity would create a successful-looking but
/// non-functional MicroVM run. Apple Container compatibility is still proven
/// by the runtime readiness gate, not assumed from this route declaration.
pub(crate) fn preset_has_microvm_image_route(preset: ServicePreset) -> bool {
    matches!(
        preset,
        ServicePreset::Postgres
            | ServicePreset::Redis
            | ServicePreset::Nats
            | ServicePreset::Rabbitmq
            | ServicePreset::Mysql
            | ServicePreset::Mongo
            | ServicePreset::Opensearch
            | ServicePreset::Firestore
            | ServicePreset::Pubsub
            | ServicePreset::Datastore
            | ServicePreset::Bigtable
            | ServicePreset::Spanner
    )
}

/// Reject an explicit MicroVM request that has no declared OCI image route.
/// This is intentionally not a native or Docker fallback: the runtime flag is
/// an isolation/backend contract.
fn reject_unsupported_microvm_preset(service: &ServiceConfig, preset: ServicePreset) -> Result<()> {
    emit_jsonl(serde_json::json!({
        "type": "error",
        "code": "container_preset_unsupported",
        "service": service.id.as_str(),
        "preset": service_preset_name(preset),
        "reason": "this preset has no declared Apple Container OCI image route",
        "next": "use runtime = \"auto\" for VAT's native/built-in implementation, or select a preset with an official container image",
    }))?;
    bail!(
        "service `{}` preset `{}` cannot run with runtime `microvm`: VAT has no declared Apple Container OCI image route for it; use runtime `auto` for the native/built-in implementation",
        service.id,
        service_preset_name(preset),
    );
}

/// Avoid declaring named-volume retention parity for preset images before VAT
/// owns their full lifetime. Explicit image services retain the existing
/// volume-aware MicroVM path; the new preset route stays bounded until its
/// ownership/cleanup contract has equivalent proof.
fn reject_microvm_preset_volumes(service: &ServiceConfig, preset: ServicePreset) -> Result<()> {
    emit_jsonl(serde_json::json!({
        "type": "error",
        "code": "container_preset_volumes_unsupported",
        "service": service.id.as_str(),
        "preset": service_preset_name(preset),
        "reason": "named-volume retention is not yet a supported MicroVM preset contract",
        "next": "use an explicit image service for the existing volume-aware MicroVM path, or remove volumes from this preset service",
    }))?;
    bail!(
        "service `{}` preset `{}` cannot run with runtime `microvm` and named volumes: VAT has not established the preset-volume ownership/cleanup contract",
        service.id,
        service_preset_name(preset),
    );
}

/// Run a preset service from its official Docker image instead of the native
/// binary. The exported connection env is identical to the native path — only
/// the process behind the mapped host port differs.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
fn prepare_preset_docker_service(
    vat: &store::Vat,
    service: &ServiceConfig,
    preset: ServicePreset,
    cancellation: &RunCancellation,
) -> Result<ServicePlan> {
    ensure_docker_available(service, cancellation)?;
    let host_port = resolve_service_port(&service.port)?;
    let container_port = service
        .container_port
        .unwrap_or_else(|| preset_container_port(preset));
    let image = preset_image(preset, service.version.as_deref());
    let name = container_name(&vat.meta.id, &service.id);
    let mut container_env = preset_container_env(preset);
    for (key, value) in &service.image_env {
        container_env.insert(key.clone(), value.clone());
    }
    let mut command =
        docker_create_command(&name, &image, host_port, container_port, &container_env);
    // GCP emulators on the cloud-cli image need the emulator start command
    // appended; the datastore/broker official images and Spanner's dedicated
    // image start via their own entrypoint.
    command.extend(preset_docker_command(preset, container_port));
    let env = preset_exports(service, preset, host_port);
    Ok(ServicePlan {
        id: service.id.clone(),
        command,
        host: Some("127.0.0.1".to_string()),
        ready_http: service.ready_http.clone(),
        ready_probe: docker_ready_probe(service, host_port),
        timeout_s: service.timeout_s,
        preset: Some(preset),
        port: Some(host_port),
        prepare_mode: "docker_run".to_string(),
        cache_key: None,
        prepare_duration_ms: 0,
        exported_env: sorted_keys(&env),
        env,
        docker_name: Some(name),
        microvm_name: None,
        image: Some(image),
        cluster: None,
        owned_by_vat: true,
        // `docker start --attach <full-id>` is VAT's foreground runtime
        // owner. Readiness cannot outlive that attachment process.
        requires_live_child: true,
        endpoint_reservations: Vec::new(),
    })
}

/// Run a preset service from its official image through Apple's `container`
/// CLI. This preserves the same image, environment, port, command, and
/// connection exports as the Docker preset path while retaining the explicit
/// MicroVM backend and its stricter published-port readiness evidence. Named
/// volumes are rejected by `resolve_preset_runtime` until their ownership and
/// cleanup contract is proven for this path.
fn prepare_preset_microvm_service(
    vat: &store::Vat,
    service: &ServiceConfig,
    preset: ServicePreset,
    cancellation: &RunCancellation,
) -> Result<ServicePlan> {
    ensure_microvm_available(cancellation)?;
    let host_port = resolve_service_port(&service.port)?;
    let container_port = service
        .container_port
        .unwrap_or_else(|| preset_container_port(preset));
    let image = preset_image(preset, service.version.as_deref());
    ensure_microvm_image_available(&image, cancellation)?;
    let name = container_name(&vat.meta.id, &service.id);
    let mut container_env = preset_container_env(preset);
    for (key, value) in &service.image_env {
        container_env.insert(key.clone(), value.clone());
    }
    let mut command = container_run_command(
        &name,
        &image,
        host_port,
        container_port,
        &container_env,
        &service.volumes,
    );
    command.extend(preset_docker_command(preset, container_port));
    let env = preset_exports(service, preset, host_port);
    let ready_http = service
        .ready_http
        .as_ref()
        .map(|value| substitute_endpoint(value, "127.0.0.1", host_port));
    Ok(ServicePlan {
        id: service.id.clone(),
        command,
        host: Some("127.0.0.1".to_string()),
        ready_http: ready_http.clone(),
        ready_probe: microvm_ready_probe(service.ready_http.as_deref(), host_port),
        timeout_s: service.timeout_s,
        preset: Some(preset),
        port: Some(host_port),
        prepare_mode: "container_run".to_string(),
        cache_key: None,
        prepare_duration_ms: 0,
        exported_env: sorted_keys(&env),
        env,
        docker_name: None,
        microvm_name: Some(name),
        image: Some(image),
        cluster: None,
        owned_by_vat: true,
        requires_live_child: false,
        endpoint_reservations: Vec::new(),
    })
}

/// Prepare the `firebase` bundle: one `firebase emulators:start` process that
/// serves every emulator configured in the workspace `firebase.json`. vat reads
/// firebase.json for the ports (firebase owns them — vat does not auto-allocate),
/// exports the well-known `*_EMULATOR_HOST` vars the client SDKs read, and probes
/// the first configured emulator (or the hub) for readiness. Native-only: there
/// is no reliable official Docker image, so a missing firebase-tools is a
/// structured unavailable error, not a silent Docker attempt.
/// @spec apps/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#logic
fn prepare_firebase_service(
    vat: &store::Vat,
    cfg: &VatConfig,
    service: &ServiceConfig,
) -> Result<ServicePlan> {
    let _ = vat;
    let missing: Vec<&str> = required_binaries(ServicePreset::Firebase)
        .iter()
        .filter(|binary| which(binary).is_none())
        .copied()
        .collect();
    if !missing.is_empty() {
        emit_jsonl(serde_json::json!({
            "type": "error",
            "code": "service_runtime_unavailable",
            "service": service.id.as_str(),
            "preset": "firebase",
            "missing_native": missing,
            "docker": false,
            "note": "the firebase bundle requires firebase-tools (npm i -g firebase-tools); Docker is not a supported fallback for firebase",
        }))?;
        bail!(
            "service `{}` preset `firebase` needs firebase-tools (missing: {}); install via `npm i -g firebase-tools`",
            service.id,
            missing.join(", ")
        );
    }

    let firebase_json = cfg.root.join("firebase.json");
    let raw = std::fs::read_to_string(&firebase_json)
        .with_context(|| format!("read {}", firebase_json.display()))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw).context("parse firebase.json")?;

    let mut env = BTreeMap::new();
    let mut hub_port = 4400u16;
    let mut first_port: Option<u16> = None;
    let mut configured_ports = BTreeSet::new();
    if let Some(emulators) = parsed.get("emulators").and_then(|e| e.as_object()) {
        for (emulator, conf) in emulators {
            let port = conf.get("port").and_then(|p| p.as_u64()).map(|p| p as u16);
            if let Some(port) = port {
                configured_ports.insert(port);
            }
            if emulator == "hub" {
                if let Some(p) = port {
                    hub_port = p;
                }
                continue;
            }
            if let (Some(var), Some(p)) = (firebase_emulator_host_var(emulator), port) {
                env.insert(var.to_string(), format!("127.0.0.1:{p}"));
                first_port.get_or_insert(p);
            }
        }
    }
    env.insert(
        "FIREBASE_EMULATOR_HUB".to_string(),
        format!("127.0.0.1:{hub_port}"),
    );
    add_service_endpoint_env(&mut env, &service.id, "127.0.0.1", hub_port);

    let ready_port = first_port.unwrap_or(hub_port);
    configured_ports.insert(hub_port);
    let endpoint_reservations = configured_ports
        .into_iter()
        .map(|port| reserve_native_service_endpoint(&service.id, port))
        .collect::<Result<Vec<_>>>()?;
    Ok(ServicePlan {
        id: service.id.clone(),
        command: vec![
            "firebase".to_string(),
            "emulators:start".to_string(),
            "--project".to_string(),
            "demo-vat".to_string(),
        ],
        host: Some("127.0.0.1".to_string()),
        ready_http: service.ready_http.clone(),
        ready_probe: ReadyProbe::Tcp {
            host: "127.0.0.1".to_string(),
            port: ready_port,
        },
        timeout_s: service.timeout_s,
        preset: Some(ServicePreset::Firebase),
        port: Some(hub_port),
        prepare_mode: "firebase_emulators".to_string(),
        cache_key: None,
        prepare_duration_ms: 0,
        exported_env: sorted_keys(&env),
        env,
        docker_name: None,
        microvm_name: None,
        image: None,
        cluster: None,
        owned_by_vat: true,
        requires_live_child: true,
        endpoint_reservations,
    })
}

/// The client-SDK host env var for a Firebase emulator, when one exists.
/// @spec apps/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#config
fn firebase_emulator_host_var(emulator: &str) -> Option<&'static str> {
    match emulator {
        "firestore" => Some("FIRESTORE_EMULATOR_HOST"),
        "auth" => Some("FIREBASE_AUTH_EMULATOR_HOST"),
        "database" => Some("FIREBASE_DATABASE_EMULATOR_HOST"),
        "storage" => Some("FIREBASE_STORAGE_EMULATOR_HOST"),
        "pubsub" => Some("PUBSUB_EMULATOR_HOST"),
        _ => None,
    }
}

/// The `vat emulator` kind name and the host env var for a built-in preset.
/// @spec apps/vat/tech-design/logic/built-in-rust-emulators-pub-sub-firebase-auth.md#config
fn builtin_emulator_info(preset: ServicePreset) -> (&'static str, &'static str) {
    match preset {
        ServicePreset::Pubsub => ("pubsub", "PUBSUB_EMULATOR_HOST"),
        ServicePreset::FirebaseAuth => ("firebase-auth", "FIREBASE_AUTH_EMULATOR_HOST"),
        ServicePreset::CloudTasks => ("cloud-tasks", "CLOUD_TASKS_EMULATOR_HOST"),
        ServicePreset::CloudScheduler => ("cloud-scheduler", "CLOUD_SCHEDULER_EMULATOR_HOST"),
        ServicePreset::CloudWorkflows => ("cloud-workflows", "CLOUD_WORKFLOWS_EMULATOR_HOST"),
        ServicePreset::CloudStorage => ("cloud-storage", "STORAGE_EMULATOR_HOST"),
        ServicePreset::HttpMock => ("http-mock", "VAT_HTTP_MOCK_HOST"),
        ServicePreset::Openapi => ("openapi", "OPENAPI_MOCK_HOST"),
        // Non-built-in presets never reach this path.
        _ => ("", ""),
    }
}

fn builtin_emulator_export_value(preset: ServicePreset, host_port: &str) -> String {
    match preset {
        ServicePreset::CloudStorage => format!("http://{host_port}"),
        _ => host_port.to_string(),
    }
}

/// Prepare a built-in emulator service: vat spawns *itself* (`vat emulator
/// <kind> --host-port`) as the service process — a pure Rust in-process server
/// with no external tooling. The runner reaches it via the exported host var.
/// @spec apps/vat/tech-design/logic/built-in-rust-emulators-pub-sub-firebase-auth.md#logic
/// The explicit `[network].routes` from vat.toml as `(host, target)` pairs. These
/// seed the http-mock proxy's routing table at spawn (the targets are literal
/// local base URLs); preset-derived routes are added by
/// [`seed_preset_routes_into_proxy`].
fn explicit_network_routes(cfg: &VatConfig) -> Vec<(String, String)> {
    cfg.network
        .as_ref()
        .map(|n| {
            n.routes
                .iter()
                .map(|r| (r.host.clone(), r.target.clone()))
                .collect()
        })
        .unwrap_or_default()
}

/// Auto-derive transparent routes from declared GCP emulator presets: each
/// preset with a stable public host (`ServicePreset::preset_gcp_host`) maps its
/// real googleapis host to its resolved local endpoint. Pure over
/// `(preset, port)` pairs so it is unit-testable without a `ServicePlan`.
fn preset_auto_routes(services: &[(Option<ServicePreset>, Option<u16>)]) -> Vec<(String, String)> {
    services
        .iter()
        .filter_map(|&(preset, port)| {
            let host = preset?.preset_gcp_host()?;
            let port = port?;
            Some((host.to_string(), format!("http://127.0.0.1:{port}")))
        })
        .collect()
}

/// Append preset-derived `--route real_host=http://127.0.0.1:<port>` args to the
/// http-mock proxy's spawn command (explicit `[network].routes` are already
/// seeded by `prepare_builtin_service` and take precedence — preset routes for an
/// already-explicit host are skipped). If routes exist but no `http-mock` service
/// is declared, emit a one-line note (routing needs a proxy) rather than failing.
fn seed_preset_routes_into_proxy(plans: &mut [ServicePlan], cfg: &VatConfig) {
    let pairs: Vec<(Option<ServicePreset>, Option<u16>)> =
        plans.iter().map(|p| (p.preset, p.port)).collect();
    let explicit: Vec<String> = explicit_network_routes(cfg)
        .into_iter()
        .map(|(h, _)| h)
        .collect();
    let auto: Vec<(String, String)> = preset_auto_routes(&pairs)
        .into_iter()
        .filter(|(host, _)| !explicit.contains(host))
        .collect();
    if auto.is_empty() {
        return;
    }
    match plans
        .iter_mut()
        .find(|p| p.preset == Some(ServicePreset::HttpMock))
    {
        Some(proxy) => {
            for (host, target) in auto {
                proxy.command.push("--route".to_string());
                proxy.command.push(format!("{host}={target}"));
            }
        }
        None => {
            let hosts: Vec<&str> = auto.iter().map(|(h, _)| h.as_str()).collect();
            eprintln!(
                "vat: note: GCP emulator presets ({}) declared but no `http-mock` service — \
                 transparent routing skipped; add a `preset = \"http-mock\"` service to route them.",
                hosts.join(", ")
            );
        }
    }
}

fn prepare_builtin_service(
    service: &ServiceConfig,
    preset: ServicePreset,
    root: &Path,
    network_routes: &[(String, String)],
    hermetic: bool,
) -> Result<ServicePlan> {
    let reservation = reserve_native_service_port(&service.id, &service.port)?;
    let port = reservation.port();
    let exe =
        std::env::current_exe().context("resolve the vat executable for the built-in emulator")?;
    let (kind, default_var) = builtin_emulator_info(preset);
    let host_port = format!("127.0.0.1:{port}");

    let mut command = vec![
        exe.to_string_lossy().into_owned(),
        "emulator".to_string(),
        kind.to_string(),
        "--host-port".to_string(),
        host_port.clone(),
    ];

    let mut env = if preset == ServicePreset::HttpMock {
        // The HTTP mock proxy exports a SET of env: proxy + CA trust. Paths live
        // under the stable store root, keyed by port for this run.
        let base = crate::paths::root()?.join("http-mock");
        let cassette_dir = base.join("cassettes");
        std::fs::create_dir_all(&cassette_dir)
            .with_context(|| format!("create {}", cassette_dir.display()))?;
        let ca_path = base.join(format!("ca-{port}.pem"));
        command.push("--ca-path".to_string());
        command.push(ca_path.to_string_lossy().into_owned());
        command.push("--cassette-dir".to_string());
        command.push(cassette_dir.to_string_lossy().into_owned());
        // Seed explicit `[network].routes` onto the proxy at spawn. Preset-derived
        // routes are appended later by `seed_preset_routes_into_proxy` (once every
        // sibling emulator's port is resolved).
        for (host, target) in network_routes {
            command.push("--route".to_string());
            command.push(format!("{host}={target}"));
        }
        // Hermetic ([network].egress != open): the proxy must not reach the
        // internet either — an unmatched request is blocked, not forwarded.
        if hermetic {
            command.push("--no-forward".to_string());
        }
        http_mock_env(&host_port, &ca_path.to_string_lossy())
    } else {
        // The openapi preset resolves its spec (relative to vat.toml) to an
        // absolute path for the spawned emulator process and serves from it.
        if preset == ServicePreset::Openapi {
            let spec = service.spec.as_deref().unwrap_or_default();
            let spec_path = crate::config::resolve_relative(root, Path::new(spec));
            command.push("--spec".to_string());
            command.push(spec_path.to_string_lossy().into_owned());
        }
        let mut env = BTreeMap::new();
        let default_value = builtin_emulator_export_value(preset, &host_port);
        if service.export.is_empty() {
            env.insert(default_var.to_string(), default_value);
        } else {
            for (key, template) in &service.export {
                if template.contains("{host}") || template.contains("{port}") {
                    env.insert(key.clone(), substitute_port(template, Some(port)));
                } else {
                    env.insert(template.clone(), default_value.clone());
                }
            }
        }
        env
    };
    add_service_endpoint_env(&mut env, &service.id, "127.0.0.1", port);

    Ok(ServicePlan {
        id: service.id.clone(),
        command,
        host: Some("127.0.0.1".to_string()),
        ready_http: service.ready_http.clone(),
        ready_probe: ReadyProbe::Tcp {
            host: "127.0.0.1".to_string(),
            port,
        },
        timeout_s: service.timeout_s,
        preset: Some(preset),
        port: Some(port),
        prepare_mode: "builtin_emulator".to_string(),
        cache_key: None,
        prepare_duration_ms: 0,
        exported_env: sorted_keys(&env),
        env,
        docker_name: None,
        microvm_name: None,
        image: None,
        cluster: None,
        owned_by_vat: true,
        requires_live_child: true,
        endpoint_reservations: vec![reservation],
    })
}

/// The env set the http-mock proxy exports into the runner: proxy vars (so all
/// outbound HTTP/S is intercepted), NO_PROXY (so the runner's other loopback
/// emulators stay direct), and CA-trust vars for every common runtime (so the
/// HTTPS MITM is trusted) — plus the admin host.
/// @spec apps/vat/tech-design/logic/built-in-http-mock-record-replay-proxy.md#config
fn http_mock_env(host_port: &str, ca_path: &str) -> BTreeMap<String, String> {
    let proxy = format!("http://{host_port}");
    let mut env = BTreeMap::new();
    for k in [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "http_proxy",
        "https_proxy",
        "ALL_PROXY",
    ] {
        env.insert(k.to_string(), proxy.clone());
    }
    env.insert("NO_PROXY".to_string(), "localhost,127.0.0.1".to_string());
    env.insert("no_proxy".to_string(), "localhost,127.0.0.1".to_string());
    for k in [
        "SSL_CERT_FILE",
        "CURL_CA_BUNDLE",
        "REQUESTS_CA_BUNDLE",
        "NODE_EXTRA_CA_CERTS",
        "GIT_SSL_CAINFO",
    ] {
        env.insert(k.to_string(), ca_path.to_string());
    }
    env.insert("VAT_HTTP_MOCK_HOST".to_string(), host_port.to_string());
    env
}

/// Run a Docker-only custom service (e.g. AlloyDB) declared with `image`.
/// `export` values are templates: `{host}`/`{port}` are substituted with the
/// mapped host endpoint. `VAT_SERVICE_<ID>_{HOST,PORT}` are always exported.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
fn prepare_image_service(
    vat: &store::Vat,
    service: &ServiceConfig,
    image: &str,
    cancellation: &RunCancellation,
) -> Result<ServicePlan> {
    ensure_docker_available(service, cancellation)?;
    let host_port = resolve_service_port(&service.port)?;
    let container_port = service
        .container_port
        .context("image service missing container_port (validated earlier)")?;
    let name = container_name(&vat.meta.id, &service.id);
    let command =
        docker_create_command(&name, image, host_port, container_port, &service.image_env);
    let env = image_exports(service, host_port);
    Ok(ServicePlan {
        id: service.id.clone(),
        command,
        host: Some("127.0.0.1".to_string()),
        ready_http: service.ready_http.clone(),
        ready_probe: docker_ready_probe(service, host_port),
        timeout_s: service.timeout_s,
        preset: None,
        port: Some(host_port),
        prepare_mode: "docker_run".to_string(),
        cache_key: None,
        prepare_duration_ms: 0,
        exported_env: sorted_keys(&env),
        env,
        docker_name: Some(name),
        microvm_name: None,
        image: Some(image.to_string()),
        cluster: None,
        owned_by_vat: true,
        // `docker start --attach <full-id>` is VAT's foreground runtime
        // owner. Readiness cannot outlive that attachment process.
        requires_live_child: true,
        endpoint_reservations: Vec::new(),
    })
}

/// Run an image-backed service via Apple's `container` CLI (MicroVM
/// isolation) instead of Docker. Structurally mirrors `prepare_image_service`
/// line for line: `ensure_microvm_available` replaces `ensure_docker_available`,
/// `container_run_command` replaces `docker_create_command`, and the returned
/// plan carries `microvm_name: Some(name)` (`docker_name` stays `None`) so
/// teardown force-removes the right container kind (R4/R5).
fn prepare_microvm_service(
    vat: &store::Vat,
    service: &ServiceConfig,
    image: &str,
    cancellation: &RunCancellation,
) -> Result<ServicePlan> {
    ensure_microvm_available(cancellation)?;
    ensure_microvm_image_available(image, cancellation)?;
    let host_port = resolve_service_port(&service.port)?;
    let container_port = service
        .container_port
        .context("image service missing container_port (validated earlier)")?;
    let name = container_name(&vat.meta.id, &service.id);
    let command = container_run_command(
        &name,
        image,
        host_port,
        container_port,
        &service.image_env,
        &service.volumes,
    );
    let env = image_exports(service, host_port);
    let ready_http = service
        .ready_http
        .as_ref()
        .map(|value| substitute_endpoint(value, "127.0.0.1", host_port));
    Ok(ServicePlan {
        id: service.id.clone(),
        command,
        host: Some("127.0.0.1".to_string()),
        ready_http: ready_http.clone(),
        ready_probe: microvm_ready_probe(service.ready_http.as_deref(), host_port),
        timeout_s: service.timeout_s,
        preset: None,
        port: Some(host_port),
        prepare_mode: "container_run".to_string(),
        cache_key: None,
        prepare_duration_ms: 0,
        exported_env: sorted_keys(&env),
        env,
        docker_name: None,
        microvm_name: Some(name),
        image: Some(image.to_string()),
        cluster: None,
        owned_by_vat: true,
        requires_live_child: false,
        endpoint_reservations: Vec::new(),
    })
}

/// Build the bounded identity-producing `docker create` argv. Docker prints the
/// immutable full ID on stdout, so VAT does not assume the daemon can write a
/// client-host cidfile (which is false for Docker Desktop and remote daemons).
/// `--rm` keeps normal exits ephemeral; cleanup still requires exact name+ID.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
fn docker_create_command(
    name: &str,
    image: &str,
    host_port: u16,
    container_port: u16,
    container_env: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut cmd = vec![
        "docker".to_string(),
        "create".to_string(),
        "--rm".to_string(),
        "--name".to_string(),
        name.to_string(),
        "-p".to_string(),
        format!("127.0.0.1:{host_port}:{container_port}"),
    ];
    for (key, value) in container_env {
        cmd.push("-e".to_string());
        cmd.push(format!("{key}={value}"));
    }
    cmd.push(image.to_string());
    cmd
}

fn docker_start_command(docker_id: &str) -> Vec<String> {
    vec![
        "docker".to_string(),
        "start".to_string(),
        "--attach".to_string(),
        docker_id.to_string(),
    ]
}

/// Build a foreground `container run` argv (Apple's `container` CLI, MicroVM
/// isolation). Structurally mirrors `docker_create_command`: `--rm` makes the
/// container ephemeral, `--name` is deterministic so teardown can
/// force-remove it, the port is bound to loopback only, then one `-v
/// name:path` per named-volume entry (compose `volumes:`, R2/R4), then one
/// `-e key=value` per sorted env entry — both volumes and env iterate in
/// deterministic order, matching `docker_create_command`'s guarantee (R5).
fn container_run_command(
    name: &str,
    image: &str,
    host_port: u16,
    container_port: u16,
    container_env: &BTreeMap<String, String>,
    volumes: &[VolumeMount],
) -> Vec<String> {
    let mut cmd = vec![
        "container".to_string(),
        "run".to_string(),
        "--rm".to_string(),
        "--name".to_string(),
        name.to_string(),
        "-p".to_string(),
        format!("127.0.0.1:{host_port}:{container_port}"),
    ];
    for volume in volumes {
        cmd.push("-v".to_string());
        cmd.push(format!("{}:{}", volume.name, volume.path));
    }
    for (key, value) in container_env {
        cmd.push("-e".to_string());
        cmd.push(format!("{key}={value}"));
    }
    cmd.push(image.to_string());
    cmd
}

/// Readiness for a container: an explicit `ready_http` wins, otherwise a TCP
/// connect to the mapped host port — which needs no native client binary.
fn docker_ready_probe(service: &ServiceConfig, host_port: u16) -> ReadyProbe {
    match &service.ready_http {
        Some(url) => ReadyProbe::Http(url.clone()),
        None => ReadyProbe::Tcp {
            host: "127.0.0.1".to_string(),
            port: host_port,
        },
    }
}

/// Readiness for an Apple `container` service. It deliberately does not reuse
/// Docker's TCP-connect-only fallback: a healthy guest with a broken host
/// port-forward must never be recorded as Ready.
fn microvm_ready_probe(ready_http: Option<&str>, host_port: u16) -> ReadyProbe {
    match ready_http {
        Some(url) => ReadyProbe::MicroVmHttp(substitute_endpoint(url, "127.0.0.1", host_port)),
        None => ReadyProbe::MicroVmTcp {
            host: "127.0.0.1".to_string(),
            port: host_port,
        },
    }
}

/// Sanitize a Docker `--name`: keep `[A-Za-z0-9_.-]`, replace the rest with `-`.
fn container_name(vat_id: &str, service_id: &str) -> String {
    format!("{vat_id}-{service_id}")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-') {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Official Docker image for a preset, tagged with `version` when supplied.
fn preset_image(preset: ServicePreset, version: Option<&str>) -> String {
    let (repo, default_tag) = match preset {
        ServicePreset::Postgres => ("postgres", "16"),
        ServicePreset::Redis => ("redis", "7"),
        ServicePreset::Nats => ("nats", "2"),
        ServicePreset::Rabbitmq => ("rabbitmq", "3"),
        ServicePreset::Mysql => ("mysql", "8"),
        ServicePreset::Mongo => ("mongo", "7"),
        ServicePreset::Opensearch => ("opensearchproject/opensearch", "2"),
        // The cloud-cli `:emulators` image bundles the gcloud emulator
        // components and a JVM.
        ServicePreset::Firestore
        | ServicePreset::Pubsub
        | ServicePreset::Datastore
        | ServicePreset::Bigtable => (
            "gcr.io/google.com/cloudsdktool/google-cloud-cli",
            "emulators",
        ),
        // Spanner ships its own emulator image, not the cloud-cli one.
        ServicePreset::Spanner => ("gcr.io/cloud-spanner-emulator/emulator", "latest"),
        // Firebase is routed through prepare_firebase_service, never here.
        ServicePreset::Firebase
        | ServicePreset::FirebaseAuth
        | ServicePreset::CloudTasks
        | ServicePreset::CloudScheduler
        | ServicePreset::CloudWorkflows
        | ServicePreset::CloudStorage
        | ServicePreset::HttpMock
        | ServicePreset::Openapi => ("node", "20-slim"),
        ServicePreset::Lumen => ("lumen", "latest"),
    };
    format!("{repo}:{}", version.unwrap_or(default_tag))
}

/// Port the preset's official image listens on inside the container.
fn preset_container_port(preset: ServicePreset) -> u16 {
    match preset {
        ServicePreset::Postgres => 5432,
        ServicePreset::Redis => 6379,
        ServicePreset::Nats => 4222,
        ServicePreset::Rabbitmq => 5672,
        ServicePreset::Mysql => 3306,
        ServicePreset::Mongo => 27017,
        ServicePreset::Opensearch => 9200,
        ServicePreset::Firestore => 8080,
        ServicePreset::Datastore => 8081,
        ServicePreset::Pubsub => 8085,
        ServicePreset::Bigtable => 8086,
        ServicePreset::Spanner => 9010,
        ServicePreset::Firebase
        | ServicePreset::FirebaseAuth
        | ServicePreset::CloudTasks
        | ServicePreset::CloudScheduler
        | ServicePreset::CloudWorkflows
        | ServicePreset::CloudStorage
        | ServicePreset::HttpMock
        | ServicePreset::Openapi => 4400,
        ServicePreset::Lumen => 7373,
    }
}

/// The emulator-start command appended after the image for GCP emulators on the
/// cloud-cli image. Empty for images that start their server via their own
/// entrypoint (datastore/broker official images, Spanner's dedicated image).
/// @spec apps/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#logic
fn preset_docker_command(preset: ServicePreset, container_port: u16) -> Vec<String> {
    let emulator = |name: &str, extra: &[&str]| {
        let mut cmd = vec![
            "gcloud".to_string(),
            "beta".to_string(),
            "emulators".to_string(),
            name.to_string(),
            "start".to_string(),
            format!("--host-port=0.0.0.0:{container_port}"),
        ];
        cmd.extend(extra.iter().map(|s| s.to_string()));
        cmd
    };
    match preset {
        ServicePreset::Firestore => emulator("firestore", &[]),
        ServicePreset::Pubsub => emulator("pubsub", &["--project=demo-vat"]),
        ServicePreset::Datastore => {
            emulator("datastore", &["--project=demo-vat", "--no-store-on-disk"])
        }
        ServicePreset::Bigtable => emulator("bigtable", &[]),
        _ => Vec::new(),
    }
}

/// Container env that makes the preset's official image accept the same
/// password-less connection the native preset exports a URL for.
fn preset_container_env(preset: ServicePreset) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    match preset {
        ServicePreset::Postgres => {
            env.insert("POSTGRES_HOST_AUTH_METHOD".to_string(), "trust".to_string());
        }
        ServicePreset::Mysql => {
            env.insert("MYSQL_ALLOW_EMPTY_PASSWORD".to_string(), "1".to_string());
        }
        ServicePreset::Redis
        | ServicePreset::Nats
        | ServicePreset::Mongo
        | ServicePreset::Rabbitmq
        | ServicePreset::Firestore
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
        | ServicePreset::Lumen => {}
        ServicePreset::Opensearch => {
            env.insert("discovery.type".to_string(), "single-node".to_string());
            env.insert("plugins.security.disabled".to_string(), "true".to_string());
            env.insert(
                "OPENSEARCH_JAVA_OPTS".to_string(),
                "-Xms512m -Xmx512m".to_string(),
            );
        }
    }
    env
}

/// Exports for a Docker-only `image` service. Each `export` value is a template
/// with `{host}`/`{port}` placeholders; raw endpoint vars are always provided.
fn image_exports(service: &ServiceConfig, host_port: u16) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    for (key, template) in &service.export {
        let value = template
            .replace("{host}", "127.0.0.1")
            .replace("{port}", &host_port.to_string());
        env.insert(key.clone(), value);
    }
    add_service_endpoint_env(&mut env, &service.id, "127.0.0.1", host_port);
    env
}

/// Whether Docker is usable: the binary is on PATH and the daemon answers.
fn docker_available(cancellation: &RunCancellation) -> Result<bool> {
    if which("docker").is_none() {
        return Ok(false);
    }
    docker_daemon_up(cancellation)
}

fn docker_daemon_up(cancellation: &RunCancellation) -> Result<bool> {
    match cancellable_command_status(
        "docker",
        &["info"],
        Duration::from_secs(5),
        cancellation,
        "Docker daemon probe",
    ) {
        Ok(status) => Ok(status.success()),
        Err(error)
            if run_interruption(&error).is_some()
                || run_cleanup_failure(&error).is_some()
                || run_owned_cleanup_failure(&error).is_some() =>
        {
            Err(error)
        }
        Err(_) => Ok(false),
    }
}

/// Gate a Docker-backed service on a reachable daemon, emitting the structured
/// `docker_unavailable` error (never a panic) when it is not.
/// @spec apps/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
fn ensure_docker_available(service: &ServiceConfig, cancellation: &RunCancellation) -> Result<()> {
    if which("docker").is_none() {
        emit_jsonl(serde_json::json!({
            "type": "error",
            "code": "docker_unavailable",
            "service": service.id.as_str(),
            "reason": "docker binary not found on PATH",
        }))?;
        bail!(
            "service `{}` needs Docker but the `docker` binary was not found on PATH",
            service.id
        );
    }
    if !docker_daemon_up(cancellation)? {
        emit_jsonl(serde_json::json!({
            "type": "error",
            "code": "docker_unavailable",
            "service": service.id.as_str(),
            "reason": "docker daemon not reachable (`docker info` failed)",
        }))?;
        bail!(
            "service `{}` needs Docker but the daemon is not reachable (`docker info` failed)",
            service.id
        );
    }
    Ok(())
}

/// Gate a `container`-backed (MicroVM) service on the `container` CLI plus a
/// running container system, emitting the structured `container_unavailable`
/// error (never a panic) when it is not, mirroring `ensure_docker_available`'s
/// `docker_unavailable` shape (R5).
fn ensure_microvm_available(cancellation: &RunCancellation) -> Result<()> {
    cancellation.check()?;
    if !sandbox::microvm::available() {
        emit_jsonl(serde_json::json!({
            "type": "error",
            "code": "container_unavailable",
            "reason": "container binary not found on PATH",
        }))?;
        bail!(
            "service needs Apple's `container` CLI but the `container` binary was not found on PATH"
        );
    }
    if let Err(error) = ensure_microvm_system_started(Duration::from_secs(30), cancellation) {
        if run_interruption(&error).is_some()
            || run_cleanup_failure(&error).is_some()
            || run_owned_cleanup_failure(&error).is_some()
        {
            return Err(error);
        }
        emit_jsonl(serde_json::json!({
            "type": "error",
            "code": "container_unavailable",
            "reason": error.to_string(),
        }))?;
        bail!("service needs the `container` system running but it did not start in time: {error}");
    }
    cancellation.check()?;
    Ok(())
}

fn ensure_microvm_system_started(timeout: Duration, cancellation: &RunCancellation) -> Result<()> {
    ensure_microvm_system_started_with("container", &["system", "status"], timeout, cancellation)
}

fn ensure_microvm_system_started_with(
    program: &str,
    args: &[&str],
    timeout: Duration,
    cancellation: &RunCancellation,
) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        cancellation.check()?;
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            bail!("container system did not respond within {timeout:?}");
        }
        let probe_timeout = Duration::from_secs(1).min(remaining);
        match cancellable_command_status(
            program,
            args,
            probe_timeout,
            cancellation,
            "Apple Container system status probe",
        ) {
            Ok(status) if status.success() => return Ok(()),
            Ok(_) => {}
            Err(error)
                if run_interruption(&error).is_some()
                    || run_cleanup_failure(&error).is_some()
                    || run_owned_cleanup_failure(&error).is_some() =>
            {
                return Err(error);
            }
            Err(_) => {}
        }
        let retry_deadline = (Instant::now() + Duration::from_millis(500)).min(deadline);
        while Instant::now() < retry_deadline {
            cancellation.check()?;
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

/// Ensure an image service has a local Apple Container image before the
/// service child is spawned. Unlike Docker, Apple's `container run` does not
/// reliably pull a missing reference itself, so a preflight pull is required
/// to make `runtime = "micro_vm"` a usable agent-facing path rather than a
/// late, portless child exit. All probes and pulls are bounded and produce
/// JSONL evidence without leaking CLI progress into VAT's structured stdout.
fn ensure_microvm_image_available(image: &str, cancellation: &RunCancellation) -> Result<()> {
    const INSPECT_TIMEOUT: Duration = Duration::from_secs(5);
    const PULL_TIMEOUT: Duration = Duration::from_secs(300);

    let pull_command = format!("container image pull {}", shell_single_quote(image));
    let inspect = match cancellable_command_status(
        "container",
        &["image", "inspect", image],
        INSPECT_TIMEOUT,
        cancellation,
        "Apple Container image inspect",
    ) {
        Ok(status) => status,
        Err(error) => {
            emit_jsonl(serde_json::json!({
                "type": "error",
                "code": "container_image_inspect_unavailable",
                "image": image,
                "next": pull_command.as_str(),
            }))?;
            return Err(error).context("inspect Apple Container image availability");
        }
    };
    if inspect.success() {
        return Ok(());
    }

    emit_jsonl(serde_json::json!({
        "type": "image_pull",
        "runtime": "microvm",
        "image": image,
        "reason": "missing from the local Apple Container image store",
    }))?;
    let pull = match cancellable_command_status(
        "container",
        &["image", "pull", image],
        PULL_TIMEOUT,
        cancellation,
        "Apple Container image pull",
    ) {
        Ok(status) => status,
        Err(error) => {
            emit_jsonl(serde_json::json!({
                "type": "error",
                "code": "container_image_pull_failed",
                "image": image,
                "reason": error.to_string(),
                "next": pull_command.as_str(),
            }))?;
            return Err(error).context("pull missing Apple Container image");
        }
    };
    if !pull.success() {
        emit_jsonl(serde_json::json!({
            "type": "error",
            "code": "container_image_pull_failed",
            "image": image,
            "exit_code": pull.code(),
            "next": pull_command.as_str(),
        }))?;
        bail!(
            "Apple Container image pull for `{image}` failed with status {:?}; retry `{pull_command}`",
            pull.code()
        );
    }

    let verified = match cancellable_command_status(
        "container",
        &["image", "inspect", image],
        INSPECT_TIMEOUT,
        cancellation,
        "Apple Container pulled-image verification",
    ) {
        Ok(status) => status,
        Err(error) => {
            emit_jsonl(serde_json::json!({
                "type": "error",
                "code": "container_image_inspect_unavailable",
                "image": image,
                "next": pull_command.as_str(),
            }))?;
            return Err(error).context("verify pulled Apple Container image");
        }
    };
    if verified.success() {
        return Ok(());
    }

    emit_jsonl(serde_json::json!({
        "type": "error",
        "code": "container_image_pull_unverified",
        "image": image,
        "next": pull_command.as_str(),
    }))?;
    bail!(
        "Apple Container reported a successful pull for `{image}`, but the image is still not inspectable; retry `{pull_command}`"
    );
}

fn cold_prepare_service_image(
    cfg: &VatConfig,
    service: &ServiceConfig,
    preset: ServicePreset,
    cache_dir: &Path,
    cancellation: &RunCancellation,
) -> Result<()> {
    match preset {
        ServicePreset::Postgres => {
            let mut command = Command::new("initdb");
            command
                .args(["-D"])
                .arg(cache_dir)
                .args(["--auth=trust", "--username=postgres"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            let status = run_bounded_owned_command(
                command,
                "postgres initdb",
                Duration::from_secs(service.timeout_s.max(60)),
                Some(cancellation),
                &|| Ok(()),
            )
            .context("run initdb for postgres service image")?;
            if !status.success() {
                bail!("postgres initdb failed for service `{}`", service.id);
            }
            // Apply `.sql` corpus seeds into the data dir ONCE, here in the
            // cold-prepare path. The populated dir is then cached and cloned
            // warm (clonefile COW) on every run, so the corpus is not rebuilt.
            cold_seed_postgres(cfg, service, cache_dir, cancellation)?;
        }
        ServicePreset::Opensearch => {
            cold_prepare_opensearch_image(service, cache_dir)?;
        }
        ServicePreset::Mysql
        | ServicePreset::Mongo
        | ServicePreset::Rabbitmq
        | ServicePreset::Redis
        | ServicePreset::Nats
        // Emulators are stateless per run (preset_uses_service_image is false),
        // so they never reach cold-prepare.
        | ServicePreset::Firestore
        | ServicePreset::Pubsub
        | ServicePreset::Datastore
        | ServicePreset::Bigtable
        | ServicePreset::Spanner
        | ServicePreset::Firebase | ServicePreset::FirebaseAuth | ServicePreset::CloudTasks | ServicePreset::CloudScheduler | ServicePreset::CloudWorkflows | ServicePreset::CloudStorage | ServicePreset::HttpMock | ServicePreset::Openapi | ServicePreset::Lumen => {}
    }
    Ok(())
}

/// Apply each `.sql` seed to a freshly-initdb'd cluster by briefly starting a
/// local postgres on a private socket dir (no TCP), running `psql -f`, then
/// stopping cleanly. Runs during cold prepare so the result is cached.
fn cold_seed_postgres(
    cfg: &VatConfig,
    service: &ServiceConfig,
    data_dir: &Path,
    cancellation: &RunCancellation,
) -> Result<()> {
    if service.seed.is_empty() {
        return Ok(());
    }
    // Unix-socket-only on a per-prepare socket dir keeps the temp server off
    // the network and avoids port races during a cold build.
    let data_dir_abs = data_dir
        .canonicalize()
        .with_context(|| format!("canonicalize postgres data dir {}", data_dir.display()))?;
    let sock_dir = std::env::temp_dir().join(format!(
        "vat-pg-seed-{}-{}",
        service.id,
        digest_bytes(data_dir_abs.to_string_lossy().as_bytes())
    ));
    if sock_dir.exists() {
        std::fs::remove_dir_all(&sock_dir)
            .with_context(|| format!("remove stale {}", sock_dir.display()))?;
    }
    std::fs::create_dir_all(&sock_dir).with_context(|| format!("create {}", sock_dir.display()))?;
    let sock_dir_abs = sock_dir.canonicalize().with_context(|| {
        format!(
            "canonicalize postgres seed socket dir {}",
            sock_dir.display()
        )
    })?;
    let sock_arg = format!(
        "-h '' -k {} -p 5432",
        shell_single_quote(&sock_dir_abs.to_string_lossy())
    );
    let start_stdout_path = sock_dir.join("pg_ctl-start.stdout.log");
    let start_stdout_handle = File::create(&start_stdout_path).with_context(|| {
        format!(
            "create postgres seed start stdout capture {}",
            start_stdout_path.display()
        )
    })?;
    let start_stderr_path = sock_dir.join("pg_ctl-start.stderr.log");
    let start_stderr_handle = File::create(&start_stderr_path).with_context(|| {
        format!(
            "create postgres seed start stderr capture {}",
            start_stderr_path.display()
        )
    })?;
    let mut start_command = Command::new("pg_ctl");
    start_command
        .arg("-D")
        .arg(&data_dir_abs)
        .args(["-w", "-t", "60", "-o"])
        .arg(&sock_arg)
        .arg("start")
        .stdin(Stdio::null())
        .stdout(start_stdout_handle)
        .stderr(start_stderr_handle);
    let start = run_bounded_owned_command(
        start_command,
        "temporary postgres seed startup",
        Duration::from_secs(service.timeout_s.max(65)),
        Some(cancellation),
        &|| Ok(()),
    );
    match start {
        Ok(status) if status.success() => {}
        Ok(_) => {
            let start_error = anyhow::anyhow!(
                "could not start temporary postgres to seed service `{}`: stdout: {}; stderr: {}",
                service.id,
                command_output_file_tail(&start_stdout_path),
                command_output_file_tail(&start_stderr_path)
            );
            return finish_failed_postgres_seed_start(
                &data_dir_abs,
                service,
                &sock_dir,
                start_error,
            );
        }
        Err(error) => {
            return finish_failed_postgres_seed_start(
                &data_dir_abs,
                service,
                &sock_dir,
                error.context("start temporary postgres for corpus seeding"),
            );
        }
    }

    // Apply every seed, stopping the server even if one fails.
    let mut seed_result = Ok(());
    for seed in &service.seed {
        let path = config::resolve_relative(&cfg.root, seed);
        let seed_stderr_path = sock_dir.join(format!(
            "psql-seed-{}.stderr.log",
            seed.file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("seed")
        ));
        let seed_stderr_handle = File::create(&seed_stderr_path).with_context(|| {
            format!(
                "create postgres seed stderr capture {}",
                seed_stderr_path.display()
            )
        })?;
        let mut seed_command = Command::new("psql");
        seed_command
            .args(["-v", "ON_ERROR_STOP=1", "-h"])
            .arg(&sock_dir_abs)
            .args(["-p", "5432", "-U", "postgres", "-d", "postgres", "-f"])
            .arg(&path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(seed_stderr_handle);
        let status = run_bounded_owned_command(
            seed_command,
            "postgres corpus seed",
            Duration::from_secs(service.timeout_s.max(60)),
            Some(cancellation),
            &|| Ok(()),
        );
        match status {
            Ok(s) if s.success() => {}
            Ok(s) => {
                seed_result = Err(anyhow::anyhow!(
                    "seed `{}` failed (exit {:?}) for service `{}`: {}",
                    path.display(),
                    s.code(),
                    service.id,
                    command_output_file_tail(&seed_stderr_path)
                ));
                break;
            }
            Err(error) => {
                seed_result = Err(error.context(format!(
                    "run psql -f {} for service `{}`",
                    path.display(),
                    service.id
                )));
                break;
            }
        }
    }

    let stop_result = stop_postgres_seed_server(&data_dir_abs, service);
    // Drop the throwaway socket dir so it is not baked into the cached image.
    let _ = std::fs::remove_dir_all(&sock_dir);
    match (seed_result, stop_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(seed), Ok(())) => Err(seed),
        (Ok(()), Err(stop)) => Err(stop),
        (Err(seed), Err(stop)) => Err(stop).context(format!(
            "postgres seed execution also failed before stop: {seed:#}"
        )),
    }
}

fn finish_failed_postgres_seed_start(
    data_dir: &Path,
    service: &ServiceConfig,
    sock_dir: &Path,
    start_error: anyhow::Error,
) -> Result<()> {
    let stop_result = stop_postgres_seed_server(data_dir, service);
    let _ = std::fs::remove_dir_all(sock_dir);
    match stop_result {
        Ok(()) => Err(start_error),
        Err(stop_error) => Err(stop_error).context(format!(
            "temporary postgres startup also failed: {start_error:#}"
        )),
    }
}

fn stop_postgres_seed_server(data_dir: &Path, service: &ServiceConfig) -> Result<()> {
    let mut stop_command = Command::new("pg_ctl");
    stop_command
        .arg("-D")
        .arg(data_dir)
        .args(["-w", "-t", "10", "-m", "fast", "stop"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    stop_postgres_seed_server_command(data_dir, service, stop_command, Duration::from_secs(15))
}

fn stop_postgres_seed_server_command(
    data_dir: &Path,
    service: &ServiceConfig,
    command: Command,
    timeout: Duration,
) -> Result<()> {
    let status = match run_bounded_owned_command(
        command,
        "temporary postgres seed shutdown",
        timeout,
        None,
        &|| Ok(()),
    ) {
        Ok(status) => status,
        Err(error) if data_dir.join("postmaster.pid").exists() => {
            return Err(postgres_seed_cleanup_failure(data_dir, service, &error));
        }
        Err(error) => {
            return Err(error).context("stop temporary postgres after corpus seeding");
        }
    };
    if !status.success() {
        if !data_dir.join("postmaster.pid").exists() {
            // A failed stop against a server that never completed startup is
            // an exact local absence proof; preserve the original typed
            // cancellation/startup error instead of replacing it with noise.
            return Ok(());
        }
        let error = anyhow::anyhow!(
            "temporary postgres stop exited unsuccessfully ({:?})",
            status.code()
        );
        return Err(postgres_seed_cleanup_failure(data_dir, service, &error));
    }
    Ok(())
}

fn postgres_seed_cleanup_failure(
    data_dir: &Path,
    service: &ServiceConfig,
    error: &anyhow::Error,
) -> anyhow::Error {
    let postmaster_path = data_dir.join("postmaster.pid");
    let postmaster_pid = std::fs::read_to_string(&postmaster_path)
        .ok()
        .and_then(|contents| contents.lines().next().map(str::trim).map(str::to_string))
        .filter(|pid| !pid.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    RunOwnedCleanupFailed {
        cleanup_error: format!(
            "temporary postgres cleanup unconfirmed for service `{}`: data_dir={}, postmaster_pid={}, recovery=`pg_ctl -D {} -w -t 10 -m fast stop`; cause: {error:#}",
            service.id,
            data_dir.display(),
            postmaster_pid,
            shell_single_quote(&data_dir.to_string_lossy()),
        ),
    }
    .into()
}

fn command_output_file_tail(path: &Path) -> String {
    std::fs::read(path)
        .map(|bytes| command_output_tail(&bytes))
        .unwrap_or_else(|err| format!("<stderr unavailable: {err}>"))
}

fn command_output_tail(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        "<no stderr>".to_string()
    } else {
        trimmed
            .chars()
            .rev()
            .take(2000)
            .collect::<String>()
            .chars()
            .rev()
            .collect()
    }
}

/// Build a single-node dev OpenSearch image: a config dir (security plugin
/// disabled, `discovery.type=single-node`) plus empty data/logs dirs. The
/// `opensearch` binary reads this dir via OPENSEARCH_PATH_CONF at run time.
fn cold_prepare_opensearch_image(service: &ServiceConfig, cache_dir: &Path) -> Result<()> {
    let config_dir = cache_dir.join("config");
    std::fs::create_dir_all(&config_dir)
        .with_context(|| format!("create {}", config_dir.display()))?;
    std::fs::create_dir_all(cache_dir.join("data"))?;
    std::fs::create_dir_all(cache_dir.join("logs"))?;

    // Seed the run-time config from the Homebrew install when present (it
    // carries jvm.options / log4j2.properties); otherwise write the minimum.
    if let Some(brew_conf) = opensearch_brew_config_dir() {
        for name in ["jvm.options", "log4j2.properties", "fips_java.security"] {
            let src = brew_conf.join(name);
            if src.is_file() {
                std::fs::copy(&src, config_dir.join(name))
                    .with_context(|| format!("copy {} into opensearch image", src.display()))?;
            }
        }
        let opts_d = config_dir.join("jvm.options.d");
        std::fs::create_dir_all(&opts_d)?;
    }

    // A dev single-node node. We do NOT set `plugins.security.disabled`: the
    // Homebrew no-jdk distribution ships WITHOUT the security plugin, so that
    // setting is unknown and OpenSearch refuses to boot if it is present. With
    // no security plugin the node is already open (HTTP, no auth/TLS) — exactly
    // what a dev EC peer wants. Network/discovery/paths are forced on the CLI
    // per run, so they are intentionally omitted here.
    let yml = "\
cluster.name: vat-opensearch
node.name: vat-node
bootstrap.memory_lock: false
";
    std::fs::write(config_dir.join("opensearch.yml"), yml)
        .with_context(|| format!("write opensearch.yml for service `{}`", service.id))?;
    Ok(())
}

/// Locate the Homebrew OpenSearch config dir (for jvm.options etc.). Best
/// effort: returns None if the layout is not the expected Homebrew one.
fn opensearch_brew_config_dir() -> Option<PathBuf> {
    for candidate in ["/opt/homebrew/etc/opensearch", "/usr/local/etc/opensearch"] {
        let path = PathBuf::from(candidate);
        if path.join("jvm.options").is_file() {
            return Some(path);
        }
    }
    None
}

/// Single-quote a string for safe inclusion in a `-o` shell-parsed option.
fn shell_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn ensure_preset_binaries(service: &ServiceConfig, preset: ServicePreset) -> Result<()> {
    let missing = required_binaries(preset)
        .iter()
        .filter(|binary| which(binary).is_none())
        .copied()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        emit_jsonl(serde_json::json!({
            "type": "error",
            "code": "missing_service_binary",
            "service": service.id.as_str(),
            "preset": service_preset_name(preset),
            "missing": missing,
        }))?;
        bail!(
            "service `{}` preset `{}` missing binaries: {}",
            service.id,
            service_preset_name(preset),
            missing.join(", ")
        );
    }
    Ok(())
}

fn required_binaries(preset: ServicePreset) -> &'static [&'static str] {
    match preset {
        // pg_ctl + psql are used during cold prepare to apply `.sql` corpus
        // seeds; they ship with the same postgres formula as `postgres`.
        ServicePreset::Postgres => &["postgres", "initdb", "pg_isready", "pg_ctl", "psql"],
        ServicePreset::Redis => &["redis-server"],
        ServicePreset::Nats => &["nats-server"],
        ServicePreset::Rabbitmq => &["rabbitmq-server"],
        ServicePreset::Mysql => &["mysqld", "mysqladmin"],
        ServicePreset::Mongo => &["mongod"],
        // Assume the Homebrew `opensearch` binary is on PATH, matching how the
        // other presets assume their server binary. Readiness uses the built-in
        // HTTP probe, so no extra client binary is required.
        ServicePreset::Opensearch => &["opensearch"],
        // GCP emulators run under the gcloud CLI and a JVM.
        ServicePreset::Firestore
        | ServicePreset::Pubsub
        | ServicePreset::Datastore
        | ServicePreset::Bigtable
        | ServicePreset::Spanner => &["gcloud", "java"],
        // The Firebase Emulator Suite runs under firebase-tools (+ a JVM for
        // its Firestore/Database emulators).
        ServicePreset::Firebase
        | ServicePreset::FirebaseAuth
        | ServicePreset::CloudTasks
        | ServicePreset::CloudScheduler
        | ServicePreset::CloudWorkflows
        | ServicePreset::CloudStorage
        | ServicePreset::HttpMock
        | ServicePreset::Openapi => &["firebase", "java"],
        ServicePreset::Lumen => &[],
    }
}

/// The gcloud component an emulator preset needs locally installed for the
/// native path. `None` for non-gcloud presets (datastore/broker, firebase).
/// @spec apps/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#config
fn gcloud_component(preset: ServicePreset) -> Option<&'static str> {
    match preset {
        ServicePreset::Firestore => Some("cloud-firestore-emulator"),
        ServicePreset::Pubsub => Some("pubsub-emulator"),
        ServicePreset::Datastore => Some("cloud-datastore-emulator"),
        ServicePreset::Bigtable => Some("bigtable"),
        ServicePreset::Spanner => Some("cloud-spanner-emulator"),
        _ => None,
    }
}

/// Locally-installed gcloud component ids (`--only-local-state` lists only the
/// installed ones). Empty when gcloud is absent or the query fails.
fn installed_gcloud_components(cancellation: &RunCancellation) -> Result<Vec<String>> {
    if which("gcloud").is_none() {
        return Ok(Vec::new());
    }
    match cancellable_command_output(
        "gcloud",
        &[
            "components",
            "list",
            "--only-local-state",
            "--format=value(id)",
        ],
        Duration::from_secs(15),
        cancellation,
        "gcloud installed-component query",
    ) {
        Ok((status, stdout)) if status.success() => Ok(String::from_utf8_lossy(&stdout)
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect()),
        Ok(_) => Ok(Vec::new()),
        Err(error)
            if run_interruption(&error).is_some()
                || run_cleanup_failure(&error).is_some()
                || run_owned_cleanup_failure(&error).is_some() =>
        {
            Err(error)
        }
        Err(_) => Ok(Vec::new()),
    }
}

/// Pure native-availability decision: all binaries present, and (for emulator
/// presets) the required gcloud component locally installed.
/// @spec apps/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#logic
fn native_available(has_binaries: bool, component: Option<&str>, installed: &[String]) -> bool {
    has_binaries
        && match component {
            Some(c) => installed.iter().any(|x| x == c),
            None => true,
        }
}

fn preset_uses_service_image(preset: ServicePreset) -> bool {
    matches!(
        preset,
        ServicePreset::Postgres
            | ServicePreset::Mysql
            | ServicePreset::Mongo
            | ServicePreset::Rabbitmq
            | ServicePreset::Opensearch
    )
}

fn resolve_service_port(port: &PortSpec) -> Result<u16> {
    match port {
        PortSpec::Fixed(port) => Ok(*port),
        PortSpec::Auto(_) => free_port(),
    }
}

fn free_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").context("allocate service port")?;
    Ok(listener.local_addr()?.port())
}

impl EndpointReservation {
    fn port(&self) -> u16 {
        self.endpoint.port()
    }
}

/// Reserve one native service endpoint for the whole preparation phase. The
/// listener is released only at the child-spawn boundary; this makes an auto
/// port unique across concurrent VAT runs and makes a fixed conflict terminal
/// before VAT starts any owned process.
fn reserve_native_service_port(service_id: &str, port: &PortSpec) -> Result<EndpointReservation> {
    let requested_port = match port {
        PortSpec::Fixed(port) => *port,
        PortSpec::Auto(_) => 0,
    };
    reserve_native_service_endpoint(service_id, requested_port)
}

fn reserve_native_service_endpoint(
    service_id: &str,
    requested_port: u16,
) -> Result<EndpointReservation> {
    let requested_endpoint = SocketAddr::from(([127, 0, 0, 1], requested_port));
    let listener = match TcpListener::bind(requested_endpoint) {
        Ok(listener) => listener,
        Err(err) => {
            let endpoint = format!("127.0.0.1:{requested_port}");
            emit_owned_endpoint_conflict(service_id, &endpoint, "prepare", &err.to_string());
            bail!(
                "native_service_endpoint_conflict: service `{service_id}` cannot own endpoint `{endpoint}`: {err}; declare `external = {{ host = \"127.0.0.1\", port = {requested_port} }}` only when intentional attachment is required"
            );
        }
    };
    let endpoint = listener
        .local_addr()
        .context("read reserved native service endpoint")?;
    Ok(EndpointReservation { endpoint, listener })
}

fn emit_owned_endpoint_conflict(service_id: &str, endpoint: &str, phase: &str, reason: &str) {
    let _ = emit_jsonl(serde_json::json!({
        "type": "error",
        "code": "native_service_endpoint_conflict",
        "service": service_id,
        "endpoint": endpoint,
        "phase": phase,
        "reason": reason,
        "owned_by_vat": true,
        "external_attach_required_for_reuse": true,
    }));
}

/// Release reservations immediately before spawning the command and prove the
/// endpoint is still unavailable at that boundary. A process that races into
/// the tiny release-to-spawn window is still caught by owned-child liveness;
/// arbitrary commands cannot inherit an already-bound socket without a new
/// public descriptor-passing contract.
fn release_endpoint_reservations_at_spawn(plan: &mut ServicePlan) -> Result<Vec<SocketAddr>> {
    let reservations = std::mem::take(&mut plan.endpoint_reservations);
    let mut endpoints = Vec::with_capacity(reservations.len());
    for EndpointReservation { endpoint, listener } in reservations {
        endpoints.push(endpoint);
        drop(listener);
    }
    for endpoint in &endpoints {
        if TcpStream::connect_timeout(endpoint, Duration::from_millis(50)).is_ok() {
            let endpoint = endpoint.to_string();
            emit_owned_endpoint_conflict(
                &plan.id,
                &endpoint,
                "spawn_boundary",
                "endpoint became reachable before the owned child was spawned",
            );
            bail!(
                "native_service_endpoint_conflict: service `{}` endpoint `{endpoint}` became reachable before spawn; VAT did not start the service",
                plan.id
            );
        }
    }
    Ok(endpoints)
}

fn service_cache_key(
    cfg: &VatConfig,
    service: &ServiceConfig,
    preset: ServicePreset,
) -> Result<String> {
    let mut material = String::new();
    material.push_str(service_preset_name(preset));
    material.push('\n');
    material.push_str(service.version.as_deref().unwrap_or("system"));
    material.push('\n');
    for seed in &service.seed {
        let path = config::resolve_relative(&cfg.root, seed);
        material.push_str(&path.to_string_lossy());
        material.push('\n');
        if path.is_file() {
            let bytes = std::fs::read(&path)
                .with_context(|| format!("read service seed {}", path.display()))?;
            material.push_str(&digest_bytes(&bytes));
            material.push('\n');
        }
    }
    Ok(format!(
        "{}-{}",
        service_preset_name(preset),
        digest_bytes(material.as_bytes())
    ))
}

fn preset_command(preset: ServicePreset, port: u16, data_dir: &Path) -> Vec<String> {
    match preset {
        ServicePreset::Postgres => vec![
            "postgres".to_string(),
            "-D".to_string(),
            data_dir.to_string_lossy().into_owned(),
            "-h".to_string(),
            "127.0.0.1".to_string(),
            "-p".to_string(),
            port.to_string(),
            "-k".to_string(),
            data_dir.to_string_lossy().into_owned(),
        ],
        ServicePreset::Redis => vec![
            "redis-server".to_string(),
            "--port".to_string(),
            port.to_string(),
            "--dir".to_string(),
            data_dir.to_string_lossy().into_owned(),
            "--save".to_string(),
            String::new(),
            "--appendonly".to_string(),
            "no".to_string(),
        ],
        ServicePreset::Nats => vec![
            "nats-server".to_string(),
            "-p".to_string(),
            port.to_string(),
            "-sd".to_string(),
            data_dir.to_string_lossy().into_owned(),
        ],
        ServicePreset::Mysql => vec![
            "mysqld".to_string(),
            format!("--datadir={}", data_dir.display()),
            "--bind-address=127.0.0.1".to_string(),
            format!("--port={port}"),
            format!("--socket={}", data_dir.join("mysql.sock").display()),
            "--skip-networking=0".to_string(),
        ],
        ServicePreset::Mongo => vec![
            "mongod".to_string(),
            "--dbpath".to_string(),
            data_dir.to_string_lossy().into_owned(),
            "--bind_ip".to_string(),
            "127.0.0.1".to_string(),
            "--port".to_string(),
            port.to_string(),
            "--quiet".to_string(),
        ],
        ServicePreset::Rabbitmq => vec!["rabbitmq-server".to_string()],
        // Single-node dev OpenSearch bound to loopback. Paths are forced via
        // -E overrides into the cloned per-run image so concurrent runs never
        // share data/logs; the prepared config dir (no security plugin →
        // open HTTP) is exported via OPENSEARCH_PATH_CONF in
        // add_service_runtime_env.
        ServicePreset::Opensearch => vec![
            "opensearch".to_string(),
            format!("-Ehttp.port={port}"),
            "-Ehttp.host=127.0.0.1".to_string(),
            "-Enetwork.host=127.0.0.1".to_string(),
            "-Ediscovery.type=single-node".to_string(),
            format!("-Epath.data={}", data_dir.join("data").display()),
            format!("-Epath.logs={}", data_dir.join("logs").display()),
        ],
        // GCP emulators: `gcloud (beta) emulators <x> start --host-port`.
        ServicePreset::Firestore => gcloud_emulator_command(true, "firestore", port, &[]),
        ServicePreset::Pubsub => {
            gcloud_emulator_command(true, "pubsub", port, &["--project=demo-vat"])
        }
        ServicePreset::Datastore => gcloud_emulator_command(
            true,
            "datastore",
            port,
            &["--project=demo-vat", "--no-store-on-disk"],
        ),
        ServicePreset::Bigtable => gcloud_emulator_command(true, "bigtable", port, &[]),
        ServicePreset::Spanner => gcloud_emulator_command(false, "spanner", port, &[]),
        // Firebase is routed through prepare_firebase_service, never here.
        ServicePreset::Firebase
        | ServicePreset::FirebaseAuth
        | ServicePreset::CloudTasks
        | ServicePreset::CloudScheduler
        | ServicePreset::CloudWorkflows
        | ServicePreset::CloudStorage
        | ServicePreset::HttpMock
        | ServicePreset::Openapi => {
            vec!["firebase".to_string(), "emulators:start".to_string()]
        }
        ServicePreset::Lumen => Vec::new(),
    }
}

/// `gcloud [beta] emulators <name> start --host-port=127.0.0.1:{port} [extra]`.
/// Spanner is GA (`beta = false`); the others live under `beta`.
fn gcloud_emulator_command(beta: bool, name: &str, port: u16, extra: &[&str]) -> Vec<String> {
    let mut cmd = vec!["gcloud".to_string()];
    if beta {
        cmd.push("beta".to_string());
    }
    cmd.extend([
        "emulators".to_string(),
        name.to_string(),
        "start".to_string(),
        format!("--host-port=127.0.0.1:{port}"),
    ]);
    cmd.extend(extra.iter().map(|s| s.to_string()));
    cmd
}

fn preset_ready_probe(preset: ServicePreset, port: u16) -> ReadyProbe {
    match preset {
        ServicePreset::Postgres => ReadyProbe::Cmd(vec![
            "pg_isready".to_string(),
            "-h".to_string(),
            "127.0.0.1".to_string(),
            "-p".to_string(),
            port.to_string(),
        ]),
        ServicePreset::Mysql => ReadyProbe::Cmd(vec![
            "mysqladmin".to_string(),
            "ping".to_string(),
            "-h".to_string(),
            "127.0.0.1".to_string(),
            "-P".to_string(),
            port.to_string(),
            "--protocol=tcp".to_string(),
        ]),
        ServicePreset::Opensearch => ReadyProbe::Http(format!("http://127.0.0.1:{port}/")),
        ServicePreset::Redis
        | ServicePreset::Nats
        | ServicePreset::Mongo
        | ServicePreset::Rabbitmq
        // Emulators open their port once ready — a TCP connect is enough.
        | ServicePreset::Firestore
        | ServicePreset::Pubsub
        | ServicePreset::Datastore
        | ServicePreset::Bigtable
        | ServicePreset::Spanner
        | ServicePreset::Firebase | ServicePreset::FirebaseAuth | ServicePreset::CloudTasks | ServicePreset::CloudScheduler | ServicePreset::CloudWorkflows | ServicePreset::CloudStorage | ServicePreset::HttpMock | ServicePreset::Openapi => ReadyProbe::Tcp {
            host: "127.0.0.1".to_string(),
            port,
        },
        ServicePreset::Lumen => ReadyProbe::Http(format!("http://127.0.0.1:{port}/readyz")),
    }
}

/// Pick the readiness probe for a service, honoring explicit overrides.
///
/// Precedence: an explicit `ready_cmd` (corpus-aware, e.g. a SQL row-count
/// `>= N`) wins so "ready" means "corpus loaded"; then `ready_http`; then the
/// preset default (`None` for a command-only service).
fn resolve_ready_probe(service: &ServiceConfig, preset_default: Option<ReadyProbe>) -> ReadyProbe {
    if !service.ready_cmd.is_empty() {
        return ReadyProbe::Cmd(service.ready_cmd.clone());
    }
    if let Some(url) = &service.ready_http {
        return ReadyProbe::Http(url.clone());
    }
    preset_default.unwrap_or(ReadyProbe::None)
}

fn preset_exports(
    service: &ServiceConfig,
    preset: ServicePreset,
    port: u16,
) -> BTreeMap<String, String> {
    let default_env = match preset {
        ServicePreset::Postgres => (
            "DATABASE_URL",
            format!("postgres://postgres@127.0.0.1:{port}/postgres"),
        ),
        ServicePreset::Redis => ("REDIS_URL", format!("redis://127.0.0.1:{port}/")),
        ServicePreset::Nats => ("NATS_URL", format!("nats://127.0.0.1:{port}")),
        ServicePreset::Rabbitmq => ("AMQP_URL", format!("amqp://guest:guest@127.0.0.1:{port}/")),
        ServicePreset::Mysql => (
            "DATABASE_URL",
            format!("mysql://root@127.0.0.1:{port}/mysql"),
        ),
        ServicePreset::Mongo => ("MONGODB_URI", format!("mongodb://127.0.0.1:{port}/test")),
        ServicePreset::Opensearch => ("OPENSEARCH_URL", format!("http://127.0.0.1:{port}")),
        // Emulators export the well-known *_EMULATOR_HOST the client SDKs read.
        ServicePreset::Firestore => ("FIRESTORE_EMULATOR_HOST", format!("127.0.0.1:{port}")),
        ServicePreset::Pubsub => ("PUBSUB_EMULATOR_HOST", format!("127.0.0.1:{port}")),
        ServicePreset::Datastore => ("DATASTORE_EMULATOR_HOST", format!("127.0.0.1:{port}")),
        ServicePreset::Bigtable => ("BIGTABLE_EMULATOR_HOST", format!("127.0.0.1:{port}")),
        ServicePreset::Spanner => ("SPANNER_EMULATOR_HOST", format!("127.0.0.1:{port}")),
        // Firebase is routed through prepare_firebase_service, never here.
        ServicePreset::Firebase
        | ServicePreset::FirebaseAuth
        | ServicePreset::CloudTasks
        | ServicePreset::CloudScheduler
        | ServicePreset::CloudWorkflows
        | ServicePreset::CloudStorage
        | ServicePreset::HttpMock
        | ServicePreset::Openapi => ("FIREBASE_EMULATOR_HUB", format!("127.0.0.1:{port}")),
        ServicePreset::Lumen => ("LUMEN_URL", format!("http://127.0.0.1:{port}")),
    };
    let mut env = BTreeMap::new();
    if service.export.is_empty() {
        env.insert(default_env.0.to_string(), default_env.1);
    } else {
        for (key, template) in &service.export {
            if template.contains("{host}") || template.contains("{port}") {
                env.insert(key.clone(), substitute_port(template, Some(port)));
            } else {
                env.insert(template.clone(), default_env.1.clone());
            }
        }
    }
    add_service_endpoint_env(&mut env, &service.id, "127.0.0.1", port);
    env
}

fn command_service_port(service: &ServiceConfig) -> Result<Option<EndpointReservation>> {
    let ready_http_port = command_ready_http_ipv4_port(service)?;
    if let Some(ready_http_port) = ready_http_port {
        if let PortSpec::Fixed(configured_port) = &service.port {
            if *configured_port != ready_http_port {
                let configured_endpoint = format!("127.0.0.1:{configured_port}");
                let ready_endpoint = format!("127.0.0.1:{ready_http_port}");
                let _ = emit_jsonl(serde_json::json!({
                    "type": "error",
                    "code": "native_service_endpoint_mismatch",
                    "service": service.id.as_str(),
                    "configured_endpoint": configured_endpoint,
                    "ready_http_endpoint": ready_endpoint,
                    "owned_by_vat": true,
                }));
                bail!(
                    "native_service_endpoint_mismatch: service `{}` configures port {} but ready_http names 127.0.0.1:{}",
                    service.id,
                    configured_port,
                    ready_http_port
                );
            }
        }
        return Ok(Some(reserve_native_service_endpoint(
            &service.id,
            ready_http_port,
        )?));
    }

    let needs_port = service.cmd.iter().any(|value| value.contains("{port}"))
        || service
            .ready_http
            .as_deref()
            .map(|value| value.contains("{port}"))
            .unwrap_or(false)
        || service
            .ready_cmd
            .iter()
            .any(|value| value.contains("{port}"))
        || service
            .export
            .values()
            .any(|value| value.contains("{port}") || value.contains("{host}"))
        || matches!(service.port, PortSpec::Fixed(_));
    if needs_port {
        Ok(Some(reserve_native_service_port(
            &service.id,
            &service.port,
        )?))
    } else {
        Ok(None)
    }
}

/// Return the fixed port named by an owned command service's literal IPv4
/// readiness endpoint. `{host}` resolves to the same supported 127.0.0.1
/// address; `{port}` remains run-allocated. Other loopback spellings are
/// rejected deliberately because reserving only an IPv4 socket would not prove
/// ownership of `localhost`'s resolver choice or an IPv6 listener.
fn command_ready_http_ipv4_port(service: &ServiceConfig) -> Result<Option<u16>> {
    let Some(raw_url) = service.ready_http.as_deref() else {
        return Ok(None);
    };
    let has_port_placeholder = raw_url
        .split_once("://")
        .and_then(|(_, rest)| rest.split(['/', '?', '#']).next())
        .map(|authority| authority.contains("{port}"))
        .unwrap_or(false);
    let parseable_url = raw_url
        .replace("{host}", "127.0.0.1")
        .replace("{port}", "1");
    let url = url::Url::parse(&parseable_url)
        .with_context(|| format!("parse ready_http for service `{}`", service.id))?;
    let Some(host) = url.host() else {
        return Ok(None);
    };
    match host {
        url::Host::Ipv4(address) if address == std::net::Ipv4Addr::LOCALHOST => {
            if has_port_placeholder {
                Ok(None)
            } else {
                Ok(Some(
                    url.port_or_known_default()
                        .context("127.0.0.1 ready_http missing port")?,
                ))
            }
        }
        url::Host::Domain(domain) if domain.eq_ignore_ascii_case("localhost") => {
            reject_unsupported_native_loopback(service, raw_url, domain)
        }
        url::Host::Ipv4(address) if address.is_loopback() => {
            reject_unsupported_native_loopback(service, raw_url, &address.to_string())
        }
        url::Host::Ipv6(address) if address.is_loopback() => {
            reject_unsupported_native_loopback(service, raw_url, &address.to_string())
        }
        _ => Ok(None),
    }
}

fn reject_unsupported_native_loopback(
    service: &ServiceConfig,
    ready_http: &str,
    host: &str,
) -> Result<Option<u16>> {
    let _ = emit_jsonl(serde_json::json!({
        "type": "error",
        "code": "native_service_loopback_unsupported",
        "service": service.id.as_str(),
        "ready_http": ready_http,
        "host": host,
        "supported_host": "127.0.0.1",
        "owned_by_vat": true,
    }));
    bail!(
        "native_service_loopback_unsupported: service `{}` ready_http host `{host}` cannot be reserved exactly; use literal `127.0.0.1` or `{{host}}`",
        service.id
    )
}

fn substitute_service_port(values: &[String], port: Option<u16>) -> Vec<String> {
    values
        .iter()
        .map(|value| substitute_port(value, port))
        .collect()
}

fn substitute_endpoint_values(values: &[String], host: &str, port: u16) -> Vec<String> {
    values
        .iter()
        .map(|value| substitute_endpoint(value, host, port))
        .collect()
}

fn substitute_port(value: &str, port: Option<u16>) -> String {
    match port {
        Some(port) => value
            .replace("{host}", "127.0.0.1")
            .replace("{port}", &port.to_string()),
        None => value.to_string(),
    }
}

fn substitute_endpoint(value: &str, host: &str, port: u16) -> String {
    value
        .replace("{host}", host)
        .replace("{port}", &port.to_string())
}

fn external_exports(service: &ServiceConfig, host: &str, port: u16) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    for (key, template) in &service.export {
        env.insert(key.clone(), substitute_endpoint(template, host, port));
    }
    add_service_endpoint_env(&mut env, &service.id, host, port);
    env
}

fn add_service_endpoint_env(
    env: &mut BTreeMap<String, String>,
    service_id: &str,
    host: &str,
    port: u16,
) {
    let upper = service_id.to_uppercase().replace(['-', '.'], "_");
    env.insert(format!("VAT_SERVICE_{upper}_HOST"), host.to_string());
    env.insert(format!("VAT_SERVICE_{upper}_PORT"), port.to_string());
}

fn export_command_service_env(
    service: &ServiceConfig,
    port: Option<u16>,
) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    if let Some(ready_http) = &service.ready_http {
        if service.export.is_empty() {
            let upper = service.id.to_uppercase().replace(['-', '.'], "_");
            env.insert(format!("VAT_SERVICE_{upper}_URL"), ready_http.clone());
        } else {
            for (key, template) in &service.export {
                if template.contains("{host}") || template.contains("{port}") {
                    env.insert(key.clone(), substitute_port(template, port));
                } else {
                    env.insert(template.clone(), ready_http.clone());
                }
            }
        }
    }
    if let Some(port) = port {
        add_service_endpoint_env(&mut env, &service.id, "127.0.0.1", port);
    }
    env
}

fn add_service_runtime_env(
    env: &mut BTreeMap<String, String>,
    preset: ServicePreset,
    service_id: &str,
    port: u16,
    data_dir: &Path,
) {
    if preset == ServicePreset::Rabbitmq {
        env.insert("RABBITMQ_NODE_PORT".to_string(), port.to_string());
        env.insert(
            "RABBITMQ_NODENAME".to_string(),
            format!("rabbitmq-vat-{service_id}@localhost"),
        );
        env.insert(
            "RABBITMQ_MNESIA_BASE".to_string(),
            data_dir.to_string_lossy().into_owned(),
        );
    }
    if preset == ServicePreset::Opensearch {
        // Point OpenSearch at the per-run cloned config dir (security disabled,
        // single-node) prepared during cold build. Cap the dev heap so several
        // single-node nodes can coexist on a laptop.
        env.insert(
            "OPENSEARCH_PATH_CONF".to_string(),
            data_dir.join("config").to_string_lossy().into_owned(),
        );
        env.entry("OPENSEARCH_JAVA_OPTS".to_string())
            .or_insert_with(|| "-Xms512m -Xmx512m".to_string());
    }
}

fn sorted_keys(env: &BTreeMap<String, String>) -> Vec<String> {
    env.keys().cloned().collect()
}

fn service_preset_name(preset: ServicePreset) -> &'static str {
    match preset {
        ServicePreset::Postgres => "postgres",
        ServicePreset::Redis => "redis",
        ServicePreset::Nats => "nats",
        ServicePreset::Rabbitmq => "rabbitmq",
        ServicePreset::Mysql => "mysql",
        ServicePreset::Mongo => "mongo",
        ServicePreset::Opensearch => "opensearch",
        ServicePreset::Firestore => "gcloud-firestore",
        ServicePreset::Pubsub => "gcloud-pubsub",
        ServicePreset::Datastore => "gcloud-datastore",
        ServicePreset::Bigtable => "gcloud-bigtable",
        ServicePreset::Spanner => "gcloud-spanner",
        ServicePreset::Firebase => "firebase",
        ServicePreset::FirebaseAuth => "firebase-auth",
        ServicePreset::CloudTasks => "gcloud-cloud-tasks",
        ServicePreset::CloudScheduler => "cloud-scheduler",
        ServicePreset::CloudWorkflows => "cloud-workflows",
        ServicePreset::CloudStorage => "cloud-storage",
        ServicePreset::HttpMock => "http-mock",
        ServicePreset::Openapi => "openapi",
        ServicePreset::Lumen => "lumen",
    }
}

fn emit_jsonl(value: serde_json::Value) -> Result<()> {
    println!("{}", serde_json::to_string(&value)?);
    Ok(())
}

fn digest_bytes(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn which(binary: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(binary);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

// <HANDWRITE gap="vat-microvm-published-endpoint-readiness" tracker="#1526" reason="Route MicroVM service probes through an endpoint-usability check that distinguishes an immediate EOF or reset from an idle but open protocol connection, while retaining explicit HTTP round trips.">
const READY_COMMAND_ATTEMPT_TIMEOUT: Duration = Duration::from_secs(2);

fn readiness_ready(probe: &ReadyProbe, cancellation: &RunCancellation) -> Result<bool> {
    match probe {
        ReadyProbe::None => Ok(true),
        ReadyProbe::Http(url) => http_ready(url),
        ReadyProbe::Tcp { host, port } => tcp_ready(host, *port),
        ReadyProbe::MicroVmHttp(url) => {
            Ok(matches!(http_readiness(url)?, EndpointReadiness::Ready))
        }
        ReadyProbe::MicroVmTcp { host, port } => Ok(matches!(
            tcp_usable_readiness(host, *port)?,
            EndpointReadiness::Ready
        )),
        ReadyProbe::Cmd(cmd) => {
            if cmd.is_empty() {
                return Ok(true);
            }
            readiness_command_with_timeout(cmd, cancellation, READY_COMMAND_ATTEMPT_TIMEOUT)
        }
    }
}

fn readiness_command_with_timeout(
    cmd: &[String],
    cancellation: &RunCancellation,
    timeout: Duration,
) -> Result<bool> {
    let mut command = Command::new(&cmd[0]);
    command
        .args(&cmd[1..])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    set_process_group(&mut command);
    let child = match command.spawn() {
        Ok(child) => child,
        Err(_) => return Ok(false),
    };
    let mut child = OwnedProcessGroup::new(child);
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(signal) = cancellation.received() {
            if let Err(error) = child.finalize("service readiness command") {
                let cleanup_error = preserve_auxiliary_cleanup_failure(
                    &mut child,
                    "service readiness command",
                    &error,
                )
                .unwrap_or_else(|| format!("service readiness command cleanup failed: {error:#}"));
                return Err(RunCleanupFailed {
                    interruption: RunInterrupted::new(signal),
                    cleanup_error,
                }
                .into());
            }
            return Err(RunInterrupted::new(signal).into());
        }
        match child.finished_status("service readiness command") {
            Ok(Some(status)) => return Ok(status.success()),
            Ok(None) => {}
            Err(error) => {
                if let Some(cleanup_error) = preserve_auxiliary_cleanup_failure(
                    &mut child,
                    "service readiness command",
                    &error,
                ) {
                    bail!("{cleanup_error}");
                }
                return Err(error);
            }
        }
        if Instant::now() >= deadline {
            if let Err(error) = child.finalize("timed-out service readiness command") {
                if let Some(cleanup_error) = preserve_auxiliary_cleanup_failure(
                    &mut child,
                    "timed-out service readiness command",
                    &error,
                ) {
                    bail!("{cleanup_error}");
                }
                return Err(error);
            }
            return Ok(false);
        }
        std::thread::sleep(OWNED_GROUP_POLL_INTERVAL);
    }
}
// </HANDWRITE>

fn tcp_ready(host: &str, port: u16) -> Result<bool> {
    let addr = (host, port)
        .to_socket_addrs()?
        .next()
        .context("tcp readiness did not resolve")?;
    Ok(TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok())
}

/// Verify that a MicroVM's loopback-published TCP endpoint stays usable after
/// the three-way handshake. A service can accept then immediately close/reset
/// a connection when Apple's host forwarding is broken; Docker's historic
/// connect-only check cannot distinguish that state.
// <HANDWRITE gap="vat-microvm-published-endpoint-failure-evidence" tracker="#1526" reason="Persist terminal MicroVM readiness evidence, collect best-effort runtime and inspect diagnostics, and leave no VAT-owned MicroVM after an unusable published endpoint.">
fn tcp_usable_readiness(host: &str, port: u16) -> Result<EndpointReadiness> {
    let endpoint = format!("{host}:{port}");
    let addr = (host, port)
        .to_socket_addrs()?
        .next()
        .context("MicroVM TCP readiness did not resolve")?;
    let stream = match TcpStream::connect_timeout(&addr, Duration::from_millis(300)) {
        Ok(stream) => stream,
        Err(err) if readiness_io_pending(&err) => {
            return Ok(EndpointReadiness::Pending(format!(
                "MicroVM published endpoint {endpoint} is not accepting TCP yet: {err}"
            )));
        }
        Err(err) => {
            return Err(err)
                .with_context(|| format!("connect MicroVM published endpoint {endpoint}"));
        }
    };
    stream.set_read_timeout(Some(Duration::from_millis(150)))?;
    let mut byte = [0u8; 1];
    match stream.peek(&mut byte) {
        Ok(0) => {
            bail!("MicroVM published endpoint {endpoint} closed immediately after accepting TCP")
        }
        Ok(_) => Ok(EndpointReadiness::Ready),
        Err(err) if matches!(err.kind(), ErrorKind::TimedOut | ErrorKind::WouldBlock) => {
            Ok(EndpointReadiness::Ready)
        }
        Err(err)
            if matches!(
                err.kind(),
                ErrorKind::ConnectionReset
                    | ErrorKind::ConnectionAborted
                    | ErrorKind::NotConnected
                    | ErrorKind::BrokenPipe
            ) =>
        {
            bail!(
                "MicroVM published endpoint {endpoint} reset immediately after accepting TCP: {err}"
            )
        }
        Err(err) => {
            Err(err).with_context(|| format!("probe MicroVM published endpoint {endpoint}"))
        }
    }
}

fn microvm_readiness(probe: &ReadyProbe) -> Result<EndpointReadiness> {
    match probe {
        ReadyProbe::MicroVmHttp(url) => http_readiness(url),
        ReadyProbe::MicroVmTcp { host, port } => tcp_usable_readiness(host, *port),
        _ => unreachable!("microvm readiness requested for non-MicroVM probe"),
    }
}

fn readiness_io_pending(err: &std::io::Error) -> bool {
    matches!(
        err.kind(),
        ErrorKind::ConnectionRefused
            | ErrorKind::TimedOut
            | ErrorKind::WouldBlock
            | ErrorKind::Interrupted
    )
}

fn is_microvm_probe(probe: &ReadyProbe) -> bool {
    matches!(
        probe,
        ReadyProbe::MicroVmHttp(_) | ReadyProbe::MicroVmTcp { .. }
    )
}

fn microvm_endpoint(service: &ServiceHandle) -> String {
    let host = service.record.host.as_deref().unwrap_or("127.0.0.1");
    service
        .record
        .port
        .map(|port| format!("{host}:{port}"))
        .unwrap_or_else(|| host.to_string())
}

fn compact_container_diagnostic(value: String) -> String {
    const MAX_CHARS: usize = 600;
    let value = value.trim().replace('\n', " ");
    let mut chars = value.chars();
    let compact: String = chars.by_ref().take(MAX_CHARS).collect();
    if chars.next().is_some() {
        format!("{compact}…")
    } else {
        compact
    }
}

const CONTAINER_DIAGNOSTIC_TIMEOUT: Duration = Duration::from_secs(1);

struct ContainerDiagnosticOutcome {
    evidence: String,
    deferred_error: Option<anyhow::Error>,
}

fn container_diagnostic(
    args: &[&str],
    cancellation: &RunCancellation,
) -> ContainerDiagnosticOutcome {
    classify_container_diagnostic(
        args,
        container_diagnostic_cancellable_until(
            args,
            Instant::now() + CONTAINER_DIAGNOSTIC_TIMEOUT,
            cancellation,
        ),
    )
}

fn classify_container_diagnostic(
    args: &[&str],
    result: Result<String>,
) -> ContainerDiagnosticOutcome {
    match result {
        Ok(evidence) => ContainerDiagnosticOutcome {
            evidence,
            deferred_error: None,
        },
        Err(error) => ContainerDiagnosticOutcome {
            evidence: format!("container {}: {error:#}", args.join(" ")),
            deferred_error: Some(error),
        },
    }
}

fn diagnostic_cleanup_error(error: &anyhow::Error) -> Option<String> {
    run_cleanup_failure(error)
        .map(|failure| failure.cleanup_error)
        .or_else(|| run_owned_cleanup_failure(error).map(|failure| failure.cleanup_error.clone()))
}

/// Run one read-only Apple Container diagnostic under a caller-owned hard
/// deadline. The command's output is bounded before it reaches memory and the
/// caller never waits on a descendant that inherited its pipes.
pub(crate) fn container_diagnostic_until(args: &[&str], deadline: Instant) -> String {
    match command_diagnostic_until("container", args, deadline, None) {
        Ok(output) => output,
        Err(error) => format!("container {}: {error:#}", args.join(" ")),
    }
}

fn container_diagnostic_cancellable_until(
    args: &[&str],
    deadline: Instant,
    cancellation: &RunCancellation,
) -> Result<String> {
    match command_diagnostic_until("container", args, deadline, Some(cancellation)) {
        Err(error)
            if run_interruption(&error).is_some()
                || run_cleanup_failure(&error).is_some()
                || run_owned_cleanup_failure(&error).is_some() =>
        {
            Err(error)
        }
        Ok(output) => Ok(output),
        Err(error) => Ok(format!("container {}: {error:#}", args.join(" "))),
    }
}

fn command_diagnostic_until(
    program: &str,
    args: &[&str],
    deadline: Instant,
    cancellation: Option<&RunCancellation>,
) -> Result<String> {
    const MAX_DIAGNOSTIC_BYTES: u64 = 8 * 1024;
    let command_label = format!("{program} {} diagnostic", args.join(" "));
    if deadline <= Instant::now() {
        return Ok(format!(
            "{program} {}: skipped because diagnostic deadline expired",
            args.join(" ")
        ));
    }
    let mut stdout =
        tempfile::tempfile().with_context(|| format!("capture {command_label} stdout"))?;
    let mut stderr =
        tempfile::tempfile().with_context(|| format!("capture {command_label} stderr"))?;
    let child_stdout = stdout
        .try_clone()
        .with_context(|| format!("clone {command_label} stdout"))?;
    let child_stderr = stderr
        .try_clone()
        .with_context(|| format!("clone {command_label} stderr"))?;
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(child_stdout))
        .stderr(Stdio::from(child_stderr));
    let observe = || {
        let stdout_len = stdout
            .metadata()
            .with_context(|| format!("inspect {command_label} stdout"))?
            .len();
        let stderr_len = stderr
            .metadata()
            .with_context(|| format!("inspect {command_label} stderr"))?
            .len();
        if stdout_len > MAX_DIAGNOSTIC_BYTES || stderr_len > MAX_DIAGNOSTIC_BYTES {
            bail!("{command_label} exceeded its bounded diagnostic output budget");
        }
        Ok(())
    };
    let status = run_bounded_owned_command(
        command,
        &command_label,
        deadline.saturating_duration_since(Instant::now()),
        cancellation,
        &observe,
    )?;
    let stdout =
        compact_container_diagnostic(read_diagnostic_capture(&mut stdout, MAX_DIAGNOSTIC_BYTES)?);
    let stderr =
        compact_container_diagnostic(read_diagnostic_capture(&mut stderr, MAX_DIAGNOSTIC_BYTES)?);
    let details = [stdout, stderr]
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(" | ");
    if details.is_empty() {
        Ok(format!("{program} {}: {status}", args.join(" ")))
    } else {
        Ok(format!("{program} {}: {status}; {details}", args.join(" ")))
    }
}

fn read_diagnostic_capture(file: &mut File, limit: u64) -> Result<String> {
    file.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::new();
    std::io::Read::by_ref(file)
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        bytes.truncate(limit as usize);
        bytes.extend_from_slice(b"\n[output truncated]");
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn report_microvm_endpoint_failure(
    vat: &mut store::Vat,
    service: &mut ServiceHandle,
    reason: &str,
    cancellation: &RunCancellation,
) -> Result<()> {
    let endpoint = microvm_endpoint(service);
    let name = service
        .record
        .microvm_name
        .as_deref()
        .unwrap_or("unavailable")
        .to_string();
    let runtime = container_diagnostic(&["--version"], cancellation);
    let inspect = if service.record.microvm_name.is_some() {
        container_diagnostic(&["inspect", &name], cancellation)
    } else {
        ContainerDiagnosticOutcome {
            evidence: "container inspect unavailable because VAT did not record a MicroVM name"
                .to_string(),
            deferred_error: None,
        }
    };
    let inspect_command = format!("container inspect {name}");
    let logs_command = format!("vat logs {} {}", vat.meta.id, service.record.id);
    let state_command = format!("vat state {}", vat.meta.id);

    let diagnostic_cleanup = [&runtime, &inspect]
        .into_iter()
        .filter_map(|outcome| outcome.deferred_error.as_ref())
        .filter_map(diagnostic_cleanup_error)
        .map(|detail| format!("diagnostic helper cleanup unconfirmed: {detail}"))
        .collect::<Vec<_>>();
    let persistence_error = if diagnostic_cleanup.is_empty() {
        None
    } else {
        let detail = diagnostic_cleanup.join("; ");
        service.record.cleanup_error = Some(append_cleanup_detail(
            service.record.cleanup_error.take(),
            detail,
        ));
        persist_service_record(vat, service.record.clone()).err()
    };

    emit_jsonl(serde_json::json!({
        "type": "error",
        "code": "microvm_published_endpoint_unusable",
        "service": service.record.id.as_str(),
        "host_endpoint": endpoint.as_str(),
        "microvm_name": name.as_str(),
        "reason": reason,
        "runtime_evidence": runtime.evidence.as_str(),
        "inspect_evidence": inspect.evidence.as_str(),
        "diagnostic_budget_ms": CONTAINER_DIAGNOSTIC_TIMEOUT.as_millis() as u64,
        "inspect": inspect_command.as_str(),
        "logs": logs_command.as_str(),
        "state": state_command.as_str(),
        "next": state_command.as_str(),
    }))?;

    let endpoint_error = format!(
        "microvm_published_endpoint_unusable: service `{}` host endpoint `{}` failed readiness: {}; runtime: {}; inspect: {}; remediation: `{}` then `{}`",
        service.record.id,
        endpoint,
        reason,
        runtime.evidence,
        inspect.evidence,
        inspect_command,
        logs_command,
    );
    let deferred_error = runtime.deferred_error.or(inspect.deferred_error);
    match (deferred_error, persistence_error) {
        (Some(error), Some(persistence)) => Err(error).context(format!(
            "{endpoint_error}; diagnostic cleanup evidence persistence also failed: {persistence:#}"
        )),
        (Some(error), None) => Err(error).context(endpoint_error),
        (None, Some(persistence)) => Err(persistence).context(endpoint_error),
        (None, None) => bail!("{endpoint_error}"),
    }
}

fn wait_for_services(
    vat: &mut store::Vat,
    services: &mut [ServiceHandle],
    cancellation: &RunCancellation,
) -> Result<()> {
    let compose_stop_request = detached_compose_stop_request_path(vat);
    for service in services {
        cancellation.check()?;
        consume_detached_compose_stop_request(&compose_stop_request, "service readiness")?;
        let started = Instant::now();
        let ready_probe = service.ready_probe.clone();
        let microvm_probe = is_microvm_probe(&ready_probe);
        let enforce_live_child = service.requires_live_child || microvm_probe;
        let mut last_microvm_readiness_error = None;
        let mut docker_attach_ready_candidate = None;
        if matches!(ready_probe, ReadyProbe::None) && service.owned_endpoints.is_empty() {
            if enforce_live_child {
                if let Some(status) = service_child_exit_status(service)? {
                    return report_owned_service_child_exit(
                        vat,
                        service,
                        status,
                        false,
                        None,
                        cancellation,
                    );
                }
            }
            service.record.status = ProcessStatus::Ready;
            service.record.ready_duration_ms = Some(started.elapsed().as_millis() as u64);
            emit_jsonl(serde_json::json!({
                "type": "ready",
                "service": service.record.id.as_str(),
                "ms": service.record.ready_duration_ms,
            }))?;
            continue;
        }
        let deadline = Instant::now() + Duration::from_secs(service.timeout_s);
        loop {
            cancellation.check()?;
            consume_detached_compose_stop_request(&compose_stop_request, "service readiness")?;
            // Probe success is never sufficient for a native command-backed
            // service: a stale listener may answer after the child has failed.
            // MicroVM probes retain their existing equivalent child check;
            // launcher-style Docker/cluster services do not inherit it.
            if enforce_live_child {
                if let Some(status) = service_child_exit_status(service)? {
                    return report_owned_service_child_exit(
                        vat,
                        service,
                        status,
                        microvm_probe,
                        last_microvm_readiness_error.as_deref(),
                        cancellation,
                    );
                }
            }
            let endpoint_transition_ready = owned_service_endpoints_ready(service);
            let readiness = if !endpoint_transition_ready {
                Ok(false)
            } else if microvm_probe {
                match microvm_readiness(&ready_probe) {
                    Ok(EndpointReadiness::Ready) => Ok(true),
                    Ok(EndpointReadiness::Pending(reason)) => {
                        last_microvm_readiness_error = Some(reason);
                        Ok(false)
                    }
                    Err(err) => Err(err),
                }
            } else {
                readiness_ready(&ready_probe, cancellation)
            };
            let docker_attach_ready_stable = if service.record.docker_id.is_some() {
                docker_attach_readiness_is_stable(
                    &mut docker_attach_ready_candidate,
                    Instant::now(),
                    matches!(&readiness, Ok(true)),
                )
            } else {
                true
            };
            match readiness {
                Ok(true) if docker_attach_ready_stable => {
                    if enforce_live_child {
                        if let Some(status) = service_child_exit_status(service)? {
                            return report_owned_service_child_exit(
                                vat,
                                service,
                                status,
                                microvm_probe,
                                last_microvm_readiness_error.as_deref(),
                                cancellation,
                            );
                        }
                    }
                    service.record.status = ProcessStatus::Ready;
                    let ms = started.elapsed().as_millis() as u64;
                    service.record.ready_duration_ms = Some(ms);
                    if let Some(cluster) = service.record.cluster.as_mut() {
                        cluster.ready_ms = Some(ms);
                    }
                    break;
                }
                Ok(true) => {}
                Ok(false) => {}
                Err(err) if run_cleanup_failure(&err).is_some() => {
                    let detail = run_cleanup_failure(&err)
                        .expect("matched cleanup failure")
                        .cleanup_error;
                    service.record.status = ProcessStatus::Failed;
                    service.record.cleanup_error = Some(detail);
                    persist_service_record(vat, service.record.clone())?;
                    return Err(err);
                }
                Err(err) if run_interruption(&err).is_some() => return Err(err),
                Err(err) if matches!(ready_probe, ReadyProbe::Cmd(_)) => {
                    service.record.status = ProcessStatus::Failed;
                    service.record.cleanup_error = Some(format!("{err:#}"));
                    persist_service_record(vat, service.record.clone())?;
                    return Err(err);
                }
                Err(err) if microvm_probe => {
                    let reason = err.to_string();
                    service.record.status = ProcessStatus::Failed;
                    service.record.readiness_error = Some(reason.clone());
                    return report_microvm_endpoint_failure(vat, service, &reason, cancellation);
                }
                Err(_) => {}
            }
            if enforce_live_child {
                if let Some(status) = service_child_exit_status(service)? {
                    return report_owned_service_child_exit(
                        vat,
                        service,
                        status,
                        microvm_probe,
                        last_microvm_readiness_error.as_deref(),
                        cancellation,
                    );
                }
            } else if let Some(status) = service_child_exit_status(service)? {
                // Preserve the pre-existing foreground launcher behavior after
                // a failed probe without treating successful readiness as a
                // native long-lived-child contract.
                service.record.status = ProcessStatus::Failed;
                service.record.exit_code = status.code();
                bail!("service `{}` exited before readiness", service.record.id);
            }
            if Instant::now() >= deadline {
                service.record.status = ProcessStatus::Timeout;
                if microvm_probe {
                    let reason = last_microvm_readiness_error.take().unwrap_or_else(|| {
                        format!(
                            "MicroVM published endpoint {} did not become usable before the {}s readiness deadline",
                            microvm_endpoint(service),
                            service.timeout_s
                        )
                    });
                    service.record.readiness_error = Some(reason.clone());
                    return report_microvm_endpoint_failure(vat, service, &reason, cancellation);
                }
                bail!("service `{}` readiness timed out", service.record.id);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        vat.log(Event::new(
            EventKind::RunStarted,
            format!("service {} ready", service.record.id),
        ))?;
        emit_jsonl(serde_json::json!({
            "type": "ready",
            "service": service.record.id.as_str(),
            "ms": service.record.ready_duration_ms,
        }))?;
        emit_service_runtime_hints(service)?;
    }
    Ok(())
}

const DOCKER_ATTACH_READY_STABILITY: Duration = Duration::from_millis(100);

/// A newly spawned `docker start --attach` can lose a scheduling race to a
/// stale listener: two immediate `try_wait` calls may see the client alive
/// before it executes and exits. Require two Ready observations separated by
/// a normal poll window. The outer loop still observes cancellation and child
/// exit every tick; this helper never sleeps or changes process ownership.
fn docker_attach_readiness_is_stable(
    candidate: &mut Option<Instant>,
    observed_at: Instant,
    probe_ready: bool,
) -> bool {
    if !probe_ready {
        *candidate = None;
        return false;
    }
    match *candidate {
        Some(first_ready) => {
            observed_at.saturating_duration_since(first_ready) >= DOCKER_ATTACH_READY_STABILITY
        }
        None => {
            *candidate = Some(observed_at);
            false
        }
    }
}

fn owned_service_endpoints_ready(service: &ServiceHandle) -> bool {
    service
        .owned_endpoints
        .iter()
        .all(|endpoint| TcpStream::connect_timeout(endpoint, Duration::from_millis(300)).is_ok())
}

/// Observe an owned service process without consuming its handle. MicroVM
/// readiness uses this both before and after its host-endpoint probe.
fn service_child_exit_status(
    service: &mut ServiceHandle,
) -> Result<Option<std::process::ExitStatus>> {
    match service.child.as_mut() {
        Some(child) => child.finished_status(&format!("service `{}`", service.record.id)),
        None => Ok(None),
    }
}

fn report_owned_service_child_exit(
    vat: &mut store::Vat,
    service: &mut ServiceHandle,
    status: std::process::ExitStatus,
    microvm_probe: bool,
    last_readiness_observation: Option<&str>,
    cancellation: &RunCancellation,
) -> Result<()> {
    if microvm_probe {
        return report_microvm_child_exit(
            vat,
            service,
            status,
            last_readiness_observation,
            cancellation,
        );
    }

    service.record.status = ProcessStatus::Failed;
    service.record.exit_code = status.code();
    let endpoint = service
        .record
        .host
        .as_deref()
        .zip(service.record.port)
        .map(|(host, port)| format!("{host}:{port}"))
        .unwrap_or_else(|| "no-network-endpoint".to_string());
    let reason = format!(
        "owned service process exited {:?} before endpoint {endpoint} completed readiness",
        status.code()
    );
    service.record.readiness_error = Some(reason.clone());
    let state_command = format!("vat state {}", vat.meta.id);
    emit_jsonl(serde_json::json!({
        "type": "error",
        "code": "owned_service_exited_before_readiness",
        "service": service.record.id.as_str(),
        "endpoint": endpoint.as_str(),
        "exit_code": status.code(),
        "reason": reason.as_str(),
        "owned_by_vat": true,
        "state": state_command.as_str(),
        "next": state_command.as_str(),
    }))?;
    bail!(
        "owned_service_exited_before_readiness: service `{}` endpoint `{}` exited {:?} before readiness",
        service.record.id,
        endpoint,
        status.code()
    )
}

/// Turn an exited MicroVM launcher into the same terminal, structured
/// published-endpoint failure path as an unusable host endpoint.
fn report_microvm_child_exit(
    vat: &mut store::Vat,
    service: &mut ServiceHandle,
    status: std::process::ExitStatus,
    last_readiness_observation: Option<&str>,
    cancellation: &RunCancellation,
) -> Result<()> {
    service.record.status = ProcessStatus::Failed;
    service.record.exit_code = status.code();
    let last_observation = last_readiness_observation
        .map(|detail| format!("; last endpoint observation: {detail}"))
        .unwrap_or_default();
    let reason = format!(
        "MicroVM service process exited {:?} before published endpoint {} became usable{last_observation}",
        status.code(),
        microvm_endpoint(service),
    );
    service.record.readiness_error = Some(reason.clone());
    report_microvm_endpoint_failure(vat, service, &reason, cancellation)
}
// </HANDWRITE>

fn emit_service_runtime_hints(service: &ServiceHandle) -> Result<()> {
    let stdout = std::fs::read_to_string(&service.record.stdout_log).unwrap_or_default();
    let stderr = std::fs::read_to_string(&service.record.stderr_log).unwrap_or_default();
    let logs = format!("{stdout}\n{stderr}");
    for hint in service_log_hints(&service.record.id, &logs) {
        emit_jsonl(hint)?;
    }
    Ok(())
}

fn service_log_hints(service_id: &str, logs: &str) -> Vec<serde_json::Value> {
    let mut hints = Vec::new();
    if logs.contains("TCP backlog setting") && logs.contains("somaxconn") {
        hints.push(serde_json::json!({
            "type": "hint",
            "code": "macos_tcp_backlog_limited",
            "service": service_id,
            "message": "native TCP service reports the macOS accept backlog is capped by kern.ipc.somaxconn; connection-heavy runners may see ECONNREFUSED even while the service is up",
            "suggestion": "reuse client connection pools or raise the host limit, for example `sudo sysctl -w kern.ipc.somaxconn=1024`, then rerun vat",
        }));
    }
    hints
}

fn emit_run_interrupted(vat: &store::Vat, interruption: &RunInterrupted) -> Result<()> {
    emit_jsonl(serde_json::json!({
        "type": "error",
        "code": "run_interrupted",
        "id": vat.meta.id.as_str(),
        "signal": interruption.signal,
        "signal_name": signal_name(interruption.signal),
        "reason": interruption.reason.as_str(),
        "exit_code": interruption.exit_code(),
        "state": format!("vat state {}", vat.meta.id),
        "next": format!("vat state {}", vat.meta.id),
    }))
}

fn emit_owned_cleanup_unconfirmed(vat: &store::Vat, message: &str) -> Result<()> {
    emit_jsonl(serde_json::json!({
        "type": "error",
        "code": "owned_cleanup_unconfirmed",
        "message": message,
        "state": format!("vat state {}", vat.meta.id),
        "next": format!("vat state {}", vat.meta.id),
    }))?;
    let has_legacy_runtime_cleanup = vat.meta.test_run.as_ref().is_some_and(|run| {
        run.services.iter().any(|service| {
            service.cleanup_error.is_some()
                && (service.docker_name.is_some() || service.microvm_name.is_some())
        })
    });
    if has_legacy_runtime_cleanup {
        // Compatibility alias for existing runtime-cleanup consumers. New
        // callers should use the ownership-generic code above, which also
        // covers native service and runner process-group obligations.
        emit_jsonl(serde_json::json!({
            "type": "error",
            "code": "microvm_cleanup_unconfirmed",
            "alias_for": "owned_cleanup_unconfirmed",
            "deprecated": true,
            "message": message,
        }))?;
    }
    Ok(())
}

fn interrupted_runner_records(
    procs: &[RunnerProc],
    interruption: &RunInterrupted,
) -> Vec<RunnerRunRecord> {
    procs
        .iter()
        .map(|proc| RunnerRunRecord {
            id: proc.runner.id.clone(),
            command: proc.runner.cmd.clone(),
            status: ProcessStatus::Interrupted,
            exit_code: Some(interruption.exit_code()),
            duration_ms: Some(proc.started.elapsed().as_millis() as u64),
            pid: None,
            cleanup_error: None,
            stdout_log: proc.stdout_log.clone(),
            stderr_log: proc.stderr_log.clone(),
        })
        .collect()
}

fn record_runner_interruption(
    vat: &mut store::Vat,
    runners: &[RunnerConfig],
    logs_dir: &Path,
    interruption: &RunInterrupted,
) -> Result<()> {
    let Some(test_run) = vat.meta.test_run.as_mut() else {
        return Ok(());
    };
    if test_run.runners.is_empty() {
        let single = runners.len() == 1;
        test_run.runners = runners
            .iter()
            .map(|runner| RunnerRunRecord {
                id: runner.id.clone(),
                command: runner.cmd.clone(),
                status: ProcessStatus::Interrupted,
                exit_code: Some(interruption.exit_code()),
                duration_ms: None,
                pid: None,
                cleanup_error: None,
                stdout_log: if single {
                    logs_dir.join("runner.stdout.log")
                } else {
                    logs_dir.join(format!("runner-{}.stdout.log", runner.id))
                }
                .to_string_lossy()
                .into_owned(),
                stderr_log: if single {
                    logs_dir.join("runner.stderr.log")
                } else {
                    logs_dir.join(format!("runner-{}.stderr.log", runner.id))
                }
                .to_string_lossy()
                .into_owned(),
            })
            .collect();
    } else {
        for runner in &mut test_run.runners {
            runner.status = ProcessStatus::Interrupted;
            runner.exit_code = Some(interruption.exit_code());
            runner.pid = None;
        }
    }
    test_run.runner = test_run.runners.first().cloned();
    Ok(())
}

fn mark_services_interrupted(services: &mut [ServiceHandle], interruption: &RunInterrupted) {
    for service in services {
        if service.record.owned_by_vat == Some(true)
            && service.child.is_some()
            && service.record.cleanup_error.is_none()
            && !matches!(
                service.record.status,
                ProcessStatus::Failed | ProcessStatus::Timeout
            )
        {
            service.record.status = ProcessStatus::Interrupted;
            service.record.exit_code = Some(interruption.exit_code());
            service.record.pid = None;
        }
    }
}

fn append_test_run_cleanup_error(vat: &mut store::Vat, detail: &str) {
    if let Some(test_run) = vat.meta.test_run.as_mut() {
        test_run.cleanup_error = Some(append_cleanup_detail(
            test_run.cleanup_error.take(),
            detail.to_string(),
        ));
    }
}

/// Failure evidence is useful, but it is not allowed to preempt the terminal
/// metadata save. Preserve an evidence-write failure as a durable fail-closed
/// obligation and return one combined diagnostic for the ordinary error path.
fn record_runner_failure_fail_closed(
    vat: &mut store::Vat,
    runner: &RunnerConfig,
    logs_dir: &Path,
    message: &str,
) -> String {
    match record_runner_failure(vat, runner, logs_dir, message) {
        Ok(()) => message.to_string(),
        Err(error) => {
            let evidence_error = format!(
                "runner failure evidence write failed; terminal metadata still requires persistence: {error:#}"
            );
            append_test_run_cleanup_error(vat, &evidence_error);
            format!("{message}; {evidence_error}")
        }
    }
}

fn record_runner_failure(
    vat: &mut store::Vat,
    runner: &RunnerConfig,
    logs_dir: &Path,
    message: &str,
) -> Result<()> {
    let stderr = logs_dir.join("runner.stderr.log");
    if let Some(test_run) = vat.meta.test_run.as_mut() {
        if test_run.runners.is_empty() {
            test_run.runners.push(RunnerRunRecord {
                id: runner.id.clone(),
                command: runner.cmd.clone(),
                status: ProcessStatus::Failed,
                exit_code: Some(-1),
                duration_ms: None,
                pid: None,
                cleanup_error: None,
                stdout_log: logs_dir
                    .join("runner.stdout.log")
                    .to_string_lossy()
                    .into_owned(),
                stderr_log: stderr.to_string_lossy().into_owned(),
            });
        } else {
            for record in &mut test_run.runners {
                if record.status == ProcessStatus::Running {
                    record.status = ProcessStatus::Failed;
                    record.exit_code = Some(-1);
                    record.pid = None;
                }
            }
            // A later lifecycle/evidence/cleanup failure must not rewrite a
            // runner outcome that was already durably observed. The overall
            // result and cleanup_error carry that later failure; Exited/0 is
            // immutable historical evidence of the target command itself.
        }
        test_run.runner = test_run.runners.first().cloned();
    }
    let mut file = OpenOptions::new().create(true).append(true).open(&stderr)?;
    writeln!(file, "{message}")?;
    Ok(())
}

fn persist_services(vat: &mut store::Vat, services: &[ServiceHandle]) -> Result<()> {
    if let Some(test_run) = vat.meta.test_run.as_mut() {
        let mut records = services
            .iter()
            .map(|service| service.record.clone())
            .collect::<Vec<_>>();
        let current_ids = records
            .iter()
            .map(|record| record.id.clone())
            .collect::<BTreeSet<_>>();
        // A backend can fail before its ServiceHandle reaches the caller. Keep
        // that already-durable Failed checkpoint (and every explicit cleanup
        // obligation) when the ordinary outer finalizer persists an otherwise
        // empty handle set. Other absent records remain stale and are dropped;
        // a current handle with the same ID always wins.
        records.extend(
            test_run
                .services
                .drain(..)
                .filter(|existing| retain_unrepresented_service_record(existing, &current_ids)),
        );
        test_run.services = records;
    }
    vat.save()
}

fn retain_unrepresented_service_record(
    existing: &ServiceRunRecord,
    current_ids: &BTreeSet<String>,
) -> bool {
    !current_ids.contains(existing.id.as_str())
        && (existing.status == ProcessStatus::Failed || existing.cleanup_error.is_some())
}

fn persist_service_record(vat: &mut store::Vat, record: ServiceRunRecord) -> Result<()> {
    let test_run = vat
        .meta
        .test_run
        .as_mut()
        .context("configured run is missing test-run evidence")?;
    if let Some(existing) = test_run
        .services
        .iter_mut()
        .find(|existing| existing.id == record.id)
    {
        *existing = record;
    } else {
        test_run.services.push(record);
    }
    vat.save().context("persist service run evidence")
}

/// A persisted cleanup error is an active resource-ownership obligation, not
/// merely diagnostic text. Retention policies must never erase the only
/// runtime name and retry path while this remains nonempty.
fn unconfirmed_runtime_cleanup_message(vat: &store::Vat) -> Option<String> {
    let test_run = vat.meta.test_run.as_ref()?;
    let mut failures = test_run
        .cleanup_error
        .as_deref()
        .map(|error| vec![format!("run auxiliary cleanup: {error}")])
        .unwrap_or_default();
    failures.extend(test_run.runners.iter().filter_map(|runner| {
        runner
            .cleanup_error
            .as_deref()
            .map(|error| format!("runner `{}`: {error}", runner.id))
    }));
    failures.extend(test_run.services.iter().filter_map(|service| {
        service
            .cleanup_error
            .as_deref()
            .map(|error| format!("service `{}`: {error}", service.id))
    }));
    (!failures.is_empty()).then(|| failures.join("; "))
}

fn should_remove_vat(
    retention: &RetentionPolicy,
    code: i32,
    cleanup_unconfirmed: bool,
    interrupted: bool,
) -> bool {
    if cleanup_unconfirmed || interrupted {
        return false;
    }
    match retention {
        RetentionPolicy::Always => false,
        RetentionPolicy::Never => true,
        RetentionPolicy::Failed => code == 0,
    }
}

// <HANDWRITE gap="vat-microvm-published-endpoint-failure-evidence" tracker="#1526" reason="Bound teardown of VAT-owned MicroVM resources so a degraded Apple Container runtime cannot prevent terminal readiness evidence from being persisted.">
const PRE_SERVICE_OUTPUT_LIMIT: u64 = 1024 * 1024;

/// One bounded owner for every synchronous helper command that runs after a
/// VAT has entered `Running` but before the service child exists. The leader is
/// kept waitable until TERM/grace/KILL has covered its process group, then it is
/// reaped and the group is proven absent. Cancellation is observed on the
/// ordinary run thread, so SIGINT/SIGTERM cannot strand a helper descendant.
fn run_bounded_owned_command(
    mut command: Command,
    label: &str,
    timeout: Duration,
    cancellation: Option<&RunCancellation>,
    observation: &dyn Fn() -> Result<()>,
) -> Result<ExitStatus> {
    set_process_group(&mut command);
    let child = command.spawn().with_context(|| format!("spawn {label}"))?;
    let mut child = OwnedProcessGroup::new(child);
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(signal) = cancellation.and_then(RunCancellation::received) {
            return match child.finalize(label) {
                Ok(_) => Err(RunInterrupted::new(signal).into()),
                Err(error) => {
                    let cleanup_error =
                        preserve_auxiliary_cleanup_failure(&mut child, label, &error)
                            .unwrap_or_else(|| format!("{label} cleanup failed: {error:#}"));
                    Err(RunCleanupFailed {
                        interruption: RunInterrupted::new(signal),
                        cleanup_error,
                    }
                    .into())
                }
            };
        }
        if let Err(observation_error) = observation() {
            return match child.finalize(label) {
                Ok(_) => Err(observation_error)
                    .with_context(|| format!("observe {label} while it was running")),
                Err(cleanup_error) => {
                    let detail =
                        preserve_auxiliary_cleanup_failure(&mut child, label, &cleanup_error)
                            .unwrap_or_else(|| {
                                format!("{label} cleanup failed: {cleanup_error:#}")
                            });
                    Err(RunOwnedCleanupFailed {
                        cleanup_error: format!(
                            "{label} observation failed ({observation_error:#}); {detail}"
                        ),
                    }
                    .into())
                }
            };
        }
        match child.finished_status(label) {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {}
            Err(observation_error) => {
                return match child.finalize(label) {
                    Ok(_) => Err(observation_error)
                        .with_context(|| format!("observe {label} process-group leader")),
                    Err(cleanup_error) => {
                        let detail =
                            preserve_auxiliary_cleanup_failure(&mut child, label, &cleanup_error)
                                .unwrap_or_else(|| {
                                    format!("{label} cleanup failed: {cleanup_error:#}")
                                });
                        Err(RunOwnedCleanupFailed {
                            cleanup_error: format!(
                                "{label} leader observation failed ({observation_error:#}); {detail}"
                            ),
                        }
                        .into())
                    }
                };
            }
        }
        if Instant::now() >= deadline {
            return match child.finalize(label) {
                Ok(_) => bail!("{label} timed out after {}ms", timeout.as_millis()),
                Err(error) => {
                    let detail = preserve_auxiliary_cleanup_failure(&mut child, label, &error)
                        .unwrap_or_else(|| format!("{label} cleanup failed: {error:#}"));
                    Err(RunOwnedCleanupFailed {
                        cleanup_error: format!(
                            "{label} timed out after {}ms; {detail}",
                            timeout.as_millis()
                        ),
                    }
                    .into())
                }
            };
        }
        std::thread::sleep(OWNED_GROUP_POLL_INTERVAL);
    }
}

fn cancellable_command_status(
    program: &str,
    args: &[&str],
    timeout: Duration,
    cancellation: &RunCancellation,
    label: &str,
) -> Result<ExitStatus> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    run_bounded_owned_command(command, label, timeout, Some(cancellation), &|| Ok(()))
}

fn cancellable_command_output(
    program: &str,
    args: &[&str],
    timeout: Duration,
    cancellation: &RunCancellation,
    label: &str,
) -> Result<(ExitStatus, Vec<u8>)> {
    let mut stdout = tempfile::tempfile().with_context(|| format!("capture {label} stdout"))?;
    let child_stdout = stdout
        .try_clone()
        .with_context(|| format!("clone {label} stdout capture"))?;
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(child_stdout))
        .stderr(Stdio::null());
    let observe = || {
        let size = stdout
            .metadata()
            .with_context(|| format!("inspect {label} stdout while command is running"))?
            .len();
        if size > PRE_SERVICE_OUTPUT_LIMIT {
            bail!(
                "{label} stdout exceeded the {} byte safety limit",
                PRE_SERVICE_OUTPUT_LIMIT
            );
        }
        Ok(())
    };
    let status = run_bounded_owned_command(command, label, timeout, Some(cancellation), &observe)?;
    let size = stdout
        .metadata()
        .with_context(|| format!("inspect {label} stdout capture"))?
        .len();
    if size > PRE_SERVICE_OUTPUT_LIMIT {
        bail!(
            "{label} stdout exceeded the {} byte safety limit",
            PRE_SERVICE_OUTPUT_LIMIT
        );
    }
    stdout
        .seek(SeekFrom::Start(0))
        .with_context(|| format!("rewind {label} stdout capture"))?;
    let mut bytes = Vec::with_capacity(size as usize);
    stdout
        .read_to_end(&mut bytes)
        .with_context(|| format!("read {label} stdout capture"))?;
    Ok((status, bytes))
}

/// Runtime cleanup's command runner. The attempt timeout decides when the
/// client is cancelled, while `phase_envelope_deadline` bounds that attempt
/// plus TERM/KILL, leader reap, and process-group absence. Docker callers pass
/// an envelope ending before later proof/remove reserves; no helper finalizer
/// may consume those phases or append a timeout beyond the shared lifecycle.
fn run_bounded_owned_command_before(
    mut command: Command,
    label: &str,
    phase_timeout: Duration,
    phase_envelope_deadline: Instant,
    retained_owners: &mut Vec<OwnedProcessGroup>,
) -> Result<ExitStatus> {
    set_process_group(&mut command);
    let child = command.spawn().with_context(|| format!("spawn {label}"))?;
    let mut child = OwnedProcessGroup::new(child);
    let phase_deadline = (Instant::now() + phase_timeout).min(phase_envelope_deadline);
    loop {
        #[cfg(unix)]
        let leader_finished = child_has_exited_without_reap(&child.child)?;
        #[cfg(not(unix))]
        let leader_finished = false;
        if leader_finished {
            return match child.finalize_before(label, phase_envelope_deadline) {
                Ok(status) => Ok(status),
                Err(error) => {
                    let pgid = child.id();
                    retained_owners.push(child);
                    Err(error).context(format!(
                        "{label} cleanup owner retained with leader PID/PGID {pgid} until its durable cleanup obligation is persisted"
                    ))
                }
            };
        }
        #[cfg(not(unix))]
        if let Some(status) = child.child.try_wait()? {
            child.final_status = Some(status);
            child.finalized = true;
            return Ok(status);
        }
        if Instant::now() >= phase_deadline {
            return match child.finalize_before(label, phase_envelope_deadline) {
                Ok(_) => bail!(
                    "{label} timed out after {}ms within the shared Docker cleanup deadline",
                    phase_timeout.as_millis()
                ),
                Err(error) => {
                    let pgid = child.id();
                    retained_owners.push(child);
                    Err(error).context(format!(
                        "{label} timed out after {}ms and its process-group cleanup was unconfirmed; cleanup owner retained with leader PID/PGID {pgid} until durable evidence is persisted",
                        phase_timeout.as_millis(),
                    ))
                }
            };
        }
        std::thread::sleep(
            OWNED_GROUP_POLL_INTERVAL.min(phase_deadline.saturating_duration_since(Instant::now())),
        );
    }
}

fn bounded_command_status_before(
    program: &str,
    args: &[&str],
    phase_timeout: Duration,
    phase_envelope_deadline: Instant,
    retained_owners: &mut Vec<OwnedProcessGroup>,
) -> Result<ExitStatus> {
    let command_text = format!("{program} {}", args.join(" "));
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    run_bounded_owned_command_before(
        command,
        &format!("teardown command `{command_text}`"),
        phase_timeout,
        phase_envelope_deadline,
        retained_owners,
    )
}

fn run_quiet_bounded_before(
    program: &str,
    args: &[&str],
    phase_timeout: Duration,
    phase_envelope_deadline: Instant,
    retained_owners: &mut Vec<OwnedProcessGroup>,
) -> Result<()> {
    let command = format!("{program} {}", args.join(" "));
    let status = bounded_command_status_before(
        program,
        args,
        phase_timeout,
        phase_envelope_deadline,
        retained_owners,
    )?;
    if status.success() {
        Ok(())
    } else {
        bail!(
            "teardown command `{command}` exited unsuccessfully ({:?})",
            status.code()
        )
    }
}

fn bounded_command_output_before(
    program: &str,
    args: &[&str],
    phase_timeout: Duration,
    phase_envelope_deadline: Instant,
    retained_owners: &mut Vec<OwnedProcessGroup>,
) -> Result<(ExitStatus, String)> {
    const MAX_QUERY_BYTES: u64 = 1024 * 1024;
    let command_text = format!("{program} {}", args.join(" "));
    let path = std::env::temp_dir().join(format!("vat-runtime-query-{}", id::fresh()));
    let result = (|| -> Result<(ExitStatus, String)> {
        let output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .with_context(|| format!("create runtime query output {}", path.display()))?;
        let mut child = Command::new(program);
        child
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::from(output))
            .stderr(Stdio::null());
        let status = run_bounded_owned_command_before(
            child,
            &format!("teardown query `{command_text}`"),
            phase_timeout,
            phase_envelope_deadline,
            retained_owners,
        )?;
        let size = std::fs::metadata(&path)
            .with_context(|| format!("read runtime query metadata {}", path.display()))?
            .len();
        if size > MAX_QUERY_BYTES {
            bail!(
                "teardown query `{command_text}` produced {size} bytes, exceeding the {MAX_QUERY_BYTES}-byte safety limit"
            );
        }
        let bytes = std::fs::read(&path)
            .with_context(|| format!("read runtime query output {}", path.display()))?;
        Ok((status, String::from_utf8_lossy(&bytes).into_owned()))
    })();
    let _ = std::fs::remove_file(&path);
    result
}

#[derive(Debug, Clone, Copy)]
enum RuntimeCleanupKind {
    Docker,
    MicroVm,
}

impl RuntimeCleanupKind {
    fn label(self) -> &'static str {
        match self {
            Self::Docker => "Docker",
            Self::MicroVm => "MicroVM",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Self::Docker => "docker",
            Self::MicroVm => "microvm",
        }
    }
}

const DOCKER_CLEANUP_HARD_TIMEOUT: Duration = Duration::from_secs(15);
const DOCKER_KILL_TIMEOUT: Duration = Duration::from_secs(3);
const DOCKER_TERMINAL_RM_TIMEOUT: Duration = Duration::from_secs(3);
const DOCKER_UNPROVEN_RM_TIMEOUT: Duration = Duration::from_secs(2);
const DOCKER_IDENTITY_QUERY_TIMEOUT: Duration = Duration::from_secs(2);
const DOCKER_IDENTITY_FORMAT: &str = "{{.ID}}\t{{.Names}}\t{{.State}}";
const MICROVM_CLEANUP_HARD_TIMEOUT: Duration = Duration::from_secs(3);
const MICROVM_REMOVE_TIMEOUT: Duration = Duration::from_secs(2);
const MICROVM_ABSENCE_QUERY_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, PartialEq, Eq)]
enum DockerIdentityObservation {
    Absent,
    Exact { state: String },
    Replacement { actual_id: String },
}

const RUNTIME_CLEANUP_BLOCK_END: &str = "[/vat-runtime-cleanup]";

fn runtime_cleanup_block_start(kind: RuntimeCleanupKind, name: &str) -> String {
    format!(
        "[vat-runtime-cleanup kind={} name={}]",
        kind.key(),
        serde_json::to_string(name).expect("runtime name serializes")
    )
}

fn runtime_cleanup_detail(kind: RuntimeCleanupKind, name: &str, error: &anyhow::Error) -> String {
    match kind {
        RuntimeCleanupKind::Docker => format!(
            "{} Docker cleanup for recorded name `{name}` did not finish: {error:#}{RUNTIME_CLEANUP_BLOCK_END}",
            runtime_cleanup_block_start(kind, name),
        ),
        RuntimeCleanupKind::MicroVm => format!(
            "{} MicroVM cleanup `container rm -f {name}` did not finish: {error:#}{RUNTIME_CLEANUP_BLOCK_END}",
            runtime_cleanup_block_start(kind, name),
        ),
    }
}

fn append_cleanup_detail(existing: Option<String>, detail: String) -> String {
    existing
        .map(|existing| format!("{existing}; {detail}"))
        .unwrap_or(detail)
}

/// Remove only the exact structured runtime obligation proved absent. Freeform
/// cleanup text may describe a process group, cluster, or readiness child and
/// must survive a successful Docker/MicroVM absence probe.
fn clear_runtime_cleanup_obligation(
    cleanup_error: &str,
    kind: RuntimeCleanupKind,
    name: &str,
) -> (Option<String>, bool) {
    let marker = runtime_cleanup_block_start(kind, name);
    let mut remaining = cleanup_error.to_string();
    let mut removed = false;
    while let Some(start) = remaining.find(&marker) {
        let body_start = start + marker.len();
        let Some(relative_end) = remaining[body_start..].find(RUNTIME_CLEANUP_BLOCK_END) else {
            break;
        };
        let end = body_start + relative_end + RUNTIME_CLEANUP_BLOCK_END.len();
        let before = remaining[..start]
            .trim_end()
            .trim_end_matches(';')
            .trim_end();
        let after = remaining[end..]
            .trim_start()
            .strip_prefix(';')
            .unwrap_or_else(|| remaining[end..].trim_start())
            .trim_start();
        remaining = match (before.is_empty(), after.is_empty()) {
            (true, true) => String::new(),
            (false, true) => before.to_string(),
            (true, false) => after.to_string(),
            (false, false) => format!("{before}; {after}"),
        };
        removed = true;
    }
    ((!remaining.is_empty()).then_some(remaining), removed)
}

fn microvm_object_confirmed_absent_before(
    name: &str,
    deadline: Instant,
    retained_owners: &mut Vec<OwnedProcessGroup>,
) -> Result<bool> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        bail!("MicroVM cleanup exhausted its shared deadline before the exact JSON absence query");
    }
    let (status, output) = bounded_command_output_before(
        "container",
        &["list", "--all", "--format", "json"],
        MICROVM_ABSENCE_QUERY_TIMEOUT.min(remaining),
        deadline,
        retained_owners,
    )?;
    if !status.success() {
        bail!(
            "exact MicroVM absence query for `{name}` exited unsuccessfully ({:?})",
            status.code()
        );
    }
    let containers = serde_json::from_str::<Vec<serde_json::Value>>(&output)
        .context("exact MicroVM absence query returned malformed JSON")?;
    for container in containers {
        let id = container
            .get("id")
            .and_then(serde_json::Value::as_str)
            .context("exact MicroVM absence query returned an object without a string id")?;
        if id == name {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Remove one VAT-owned Apple Container object under one end-to-end absolute
/// deadline. The remove client, its process-group finalization, and the exact
/// JSON absence proof all consume the same budget; even a successful `rm`
/// cannot release the persisted name without the final list proof.
fn cleanup_microvm_runtime_handle(
    name_slot: &mut Option<String>,
    retained_owners: &mut Vec<OwnedProcessGroup>,
) -> Result<()> {
    let Some(name) = name_slot.clone() else {
        return Ok(());
    };
    let deadline = Instant::now() + MICROVM_CLEANUP_HARD_TIMEOUT;
    let remove = run_quiet_bounded_before(
        "container",
        &["rm", "-f", &name],
        MICROVM_REMOVE_TIMEOUT,
        deadline,
        retained_owners,
    );
    let absence = microvm_object_confirmed_absent_before(&name, deadline, retained_owners);
    match (remove, absence) {
        (_, Ok(true)) => {
            *name_slot = None;
            Ok(())
        }
        (Ok(()), Ok(false)) => {
            bail!(
                "MicroVM remove reported success, but exact JSON evidence still contains `{name}`"
            )
        }
        (Err(remove), Ok(false)) => Err(remove).context(format!(
            "MicroVM object `{name}` remains after the bounded remove attempt"
        )),
        (Ok(()), Err(proof)) => Err(proof).context(format!(
            "MicroVM remove for `{name}` reported success without a final absence proof"
        )),
        (Err(remove), Err(proof)) => Err(proof).context(format!(
            "MicroVM cleanup for `{name}` could not prove absence after remove failure: {remove:#}"
        )),
    }
}

#[derive(Debug, Clone, Copy)]
struct DockerHelperPhaseBudget {
    attempt_timeout: Duration,
    envelope_deadline: Instant,
}

fn docker_cleanup_phase_budget(
    deadline: Instant,
    cap: Duration,
    reserve: Duration,
    phase: &str,
) -> Result<DockerHelperPhaseBudget> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    let available = remaining.checked_sub(reserve).with_context(|| {
        format!(
            "Docker cleanup exhausted its shared {}ms deadline before {phase}; {}ms remained but {}ms is reserved for later proof/finalization",
            DOCKER_CLEANUP_HARD_TIMEOUT.as_millis(),
            remaining.as_millis(),
            reserve.as_millis(),
        )
    })?;
    let attempt_timeout = cap.min(available);
    if attempt_timeout.is_zero() {
        bail!("Docker cleanup has no shared deadline budget left for {phase}");
    }
    // The whole helper lifecycle, including TERM/KILL, leader reap, and group
    // absence, must end before the later-phase reserve begins. Passing the
    // global deadline to helper finalization would let a hung Docker client
    // consume the proof/remove slices this calculation just protected.
    let envelope_deadline = deadline
        .checked_sub(reserve)
        .context("Docker cleanup phase reserve exceeds the monotonic deadline")?;
    Ok(DockerHelperPhaseBudget {
        attempt_timeout,
        envelope_deadline,
    })
}

fn observe_docker_identity(
    name: &str,
    expected_id: &str,
    deadline: Instant,
    reserve: Duration,
    retained_owners: &mut Vec<OwnedProcessGroup>,
) -> Result<DockerIdentityObservation> {
    let budget = docker_cleanup_phase_budget(
        deadline,
        DOCKER_IDENTITY_QUERY_TIMEOUT,
        reserve,
        "the anchored identity query",
    )?;
    let filter = docker_exact_name_filter(name)?;
    let (status, output) = bounded_command_output_before(
        "docker",
        &[
            "container",
            "ls",
            "--all",
            "--no-trunc",
            "--filter",
            &filter,
            "--format",
            DOCKER_IDENTITY_FORMAT,
        ],
        budget.attempt_timeout,
        budget.envelope_deadline,
        retained_owners,
    )?;
    if !status.success() {
        bail!(
            "anchored Docker identity query for `{name}` exited unsuccessfully ({:?})",
            status.code()
        );
    }
    let rows = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return Ok(DockerIdentityObservation::Absent);
    }
    if rows.len() != 1 {
        bail!(
            "anchored Docker identity query for `{name}` returned {} rows; refusing ambiguous cleanup",
            rows.len()
        );
    }
    let columns = rows[0].split('\t').collect::<Vec<_>>();
    if columns.len() != 3 {
        bail!(
            "anchored Docker identity query for `{name}` returned malformed full-ID/name/state output"
        );
    }
    let actual_id = columns[0].trim();
    let actual_name = columns[1].trim();
    let state = columns[2].trim().to_ascii_lowercase();
    if !valid_full_docker_id(actual_id) || actual_name != name || state.is_empty() {
        bail!(
            "anchored Docker identity query for `{name}` returned invalid full-ID/name/state evidence"
        );
    }
    if actual_id != expected_id {
        return Ok(DockerIdentityObservation::Replacement {
            actual_id: actual_id.to_string(),
        });
    }
    Ok(DockerIdentityObservation::Exact { state })
}

fn docker_exact_name_filter(name: &str) -> Result<String> {
    if name.is_empty()
        || !name.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '-')
        })
    {
        bail!("Docker cleanup name `{name}` is not in VAT's sanitized `[A-Za-z0-9_.-]+` form");
    }
    let escaped = name.replace('.', "\\.");
    Ok(format!("name=^/{escaped}$"))
}

fn docker_state_is_terminal(state: &str) -> bool {
    matches!(state, "exited" | "dead" | "removing")
}

fn docker_replacement_error(name: &str, expected_id: &str, actual_id: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "Docker name `{name}` now belongs to replacement ID `{actual_id}`, not VAT-owned ID `{expected_id}`; replacement was not signalled or removed"
    )
}

/// Clean one VAT-owned Docker object without ever treating a reusable name as
/// deletion authority. A single shared deadline covers exact-name/full-ID
/// queries, a force-kill transition, one remove attempt, and final absence.
/// The longer remove slice is granted only after the same immutable ID is
/// observed terminal/removing; an ID alone never makes a hung remove trusted.
fn cleanup_docker_runtime_handle(
    name_slot: &mut Option<String>,
    docker_id: Option<&str>,
    retained_owners: &mut Vec<OwnedProcessGroup>,
) -> Result<()> {
    let Some(name) = name_slot.clone() else {
        return Ok(());
    };
    let docker_id = docker_id.context(format!(
        "VAT-owned Docker service `{name}` has no persisted full container ID; legacy name-only cleanup is fail-closed"
    ))?;
    if !valid_full_docker_id(docker_id) {
        bail!(
            "VAT-owned Docker service `{name}` has invalid persisted full container ID `{docker_id}`; cleanup is fail-closed"
        );
    }

    let deadline = Instant::now() + DOCKER_CLEANUP_HARD_TIMEOUT;
    let initial_state = match observe_docker_identity(
        &name,
        docker_id,
        deadline,
        DOCKER_KILL_TIMEOUT + DOCKER_TERMINAL_RM_TIMEOUT + 2 * DOCKER_IDENTITY_QUERY_TIMEOUT,
        retained_owners,
    )? {
        DockerIdentityObservation::Absent => {
            *name_slot = None;
            return Ok(());
        }
        DockerIdentityObservation::Replacement { actual_id } => {
            return Err(docker_replacement_error(&name, docker_id, &actual_id));
        }
        DockerIdentityObservation::Exact { state } => state,
    };
    let mut terminal = docker_state_is_terminal(&initial_state);
    let created = initial_state == "created";

    let mut kill_error = None;
    if !terminal && !created {
        let kill_budget = docker_cleanup_phase_budget(
            deadline,
            DOCKER_KILL_TIMEOUT,
            DOCKER_TERMINAL_RM_TIMEOUT + 2 * DOCKER_IDENTITY_QUERY_TIMEOUT,
            "the immutable-ID kill",
        )?;
        let retained_before = retained_owners.len();
        match run_quiet_bounded_before(
            "docker",
            &["kill", docker_id],
            kill_budget.attempt_timeout,
            kill_budget.envelope_deadline,
            retained_owners,
        ) {
            Ok(()) => {}
            Err(error) if retained_owners.len() > retained_before => {
                return Err(error).context(
                    "Docker kill helper ownership remained unconfirmed inside its phase envelope",
                );
            }
            Err(error) => kill_error = Some(error),
        }
        match observe_docker_identity(
            &name,
            docker_id,
            deadline,
            DOCKER_TERMINAL_RM_TIMEOUT + DOCKER_IDENTITY_QUERY_TIMEOUT,
            retained_owners,
        )? {
            DockerIdentityObservation::Absent => {
                *name_slot = None;
                return Ok(());
            }
            DockerIdentityObservation::Replacement { actual_id } => {
                return Err(docker_replacement_error(&name, docker_id, &actual_id));
            }
            DockerIdentityObservation::Exact { state } => {
                terminal = docker_state_is_terminal(&state);
            }
        }
    }

    let rm_cap = if terminal {
        DOCKER_TERMINAL_RM_TIMEOUT
    } else {
        DOCKER_UNPROVEN_RM_TIMEOUT
    };
    let rm_budget = docker_cleanup_phase_budget(
        deadline,
        rm_cap,
        DOCKER_IDENTITY_QUERY_TIMEOUT,
        "the single immutable-ID remove attempt",
    )?;
    let retained_before = retained_owners.len();
    let rm_error = match run_quiet_bounded_before(
        "docker",
        &["rm", "-f", docker_id],
        rm_budget.attempt_timeout,
        rm_budget.envelope_deadline,
        retained_owners,
    ) {
        Ok(()) => None,
        Err(error) if retained_owners.len() > retained_before => {
            return Err(error).context(
                "Docker remove helper ownership remained unconfirmed inside its phase envelope",
            );
        }
        Err(error) => Some(error),
    };
    let final_observation =
        observe_docker_identity(&name, docker_id, deadline, Duration::ZERO, retained_owners);
    match final_observation {
        Ok(DockerIdentityObservation::Absent) => {
            *name_slot = None;
            Ok(())
        }
        Ok(DockerIdentityObservation::Replacement { actual_id }) => {
            Err(docker_replacement_error(&name, docker_id, &actual_id))
        }
        Ok(DockerIdentityObservation::Exact { state }) => {
            let rm_error = rm_error
                .map(|error| format!("; remove failed: {error:#}"))
                .unwrap_or_else(|| "; remove reported success but the object remains".to_string());
            let kill_error = kill_error
                .map(|error| format!("; kill failed: {error:#}"))
                .unwrap_or_default();
            bail!(
                "Docker object `{name}` / `{docker_id}` remains in state `{state}` after the one cleanup lifecycle{kill_error}{rm_error}"
            )
        }
        Err(proof_error) => {
            let rm_error = rm_error
                .map(|error| format!("; remove failed: {error:#}"))
                .unwrap_or_default();
            let kill_error = kill_error
                .map(|error| format!("; kill failed: {error:#}"))
                .unwrap_or_default();
            Err(proof_error).context(format!(
                "Docker cleanup for `{name}` / `{docker_id}` could not prove final absence{kill_error}{rm_error}"
            ))
        }
    }
}

fn has_runtime_cleanup_obligation(
    cleanup_error: Option<&str>,
    kind: RuntimeCleanupKind,
    name: &str,
) -> bool {
    cleanup_error
        .map(|error| clear_runtime_cleanup_obligation(error, kind, name).1)
        .unwrap_or(false)
}

fn record_runtime_cleanup_failure(
    service: &mut ServiceHandle,
    kind: RuntimeCleanupKind,
    name: &str,
    error: &anyhow::Error,
) {
    let existing = service.record.cleanup_error.take();
    let existing = existing.and_then(|error| {
        let (remaining, _) = clear_runtime_cleanup_obligation(&error, kind, name);
        remaining
    });
    let detail = runtime_cleanup_detail(kind, name, error);
    service.record.status = ProcessStatus::Failed;
    service.record.cleanup_error = Some(append_cleanup_detail(existing, detail));
}
// </HANDWRITE>

// <HANDWRITE gap="vat-compose-cleanup-confirmation" tracker="#1526" reason="Retry persisted MicroVM cleanup from a retained compose binding before allowing port reuse.">
/// Retry only persisted, previously-unconfirmed runtime teardown. Compose
/// calls this before it can release a retained binding: a successful retry
/// clears the explicit cleanup evidence, while another failure remains
/// durable and fail-closed for both Docker and MicroVM service owners.
pub(crate) fn retry_unconfirmed_service_cleanup(vat: &mut store::Vat) -> Result<()> {
    let Some(test_run) = vat.meta.test_run.as_mut() else {
        return Ok(());
    };

    let mut failures = Vec::new();
    let mut retained_owners = Vec::new();
    for service in &mut test_run.services {
        if service.cleanup_error.is_none() {
            continue;
        }
        let (kind, name) = match (
            service.microvm_name.as_deref(),
            service.docker_name.as_deref(),
        ) {
            (Some(_), Some(_)) => {
                failures.push(format!(
                    "service `{}` has unconfirmed cleanup with both MicroVM and Docker names",
                    service.id
                ));
                continue;
            }
            (Some(name), None) => (RuntimeCleanupKind::MicroVm, name),
            (None, Some(name)) => (RuntimeCleanupKind::Docker, name),
            (None, None) => {
                failures.push(format!(
                    "service `{}` has unconfirmed cleanup but no persisted runtime name",
                    service.id
                ));
                continue;
            }
        };
        let runtime = kind.label();
        let existing = service
            .cleanup_error
            .as_deref()
            .expect("checked cleanup error")
            .to_string();
        let (_, has_runtime_obligation) = clear_runtime_cleanup_obligation(&existing, kind, name);
        if !has_runtime_obligation {
            failures.push(format!(
                "service `{}` has cleanup evidence unrelated to the persisted {runtime} name; retained without deleting `{name}`",
                service.id
            ));
            continue;
        }
        let cleanup = match kind {
            RuntimeCleanupKind::Docker => {
                let mut name_slot = Some(name.to_string());
                cleanup_docker_runtime_handle(
                    &mut name_slot,
                    service.docker_id.as_deref(),
                    &mut retained_owners,
                )
            }
            RuntimeCleanupKind::MicroVm => {
                let mut name_slot = Some(name.to_string());
                cleanup_microvm_runtime_handle(&mut name_slot, &mut retained_owners)
            }
        };
        match cleanup {
            Ok(()) => {
                let (remaining, removed) = clear_runtime_cleanup_obligation(&existing, kind, name);
                debug_assert!(removed);
                service.cleanup_error = remaining;
            }
            Err(error) => {
                let (remaining, removed) = clear_runtime_cleanup_obligation(&existing, kind, name);
                debug_assert!(removed);
                let detail = runtime_cleanup_detail(kind, name, &error);
                service.cleanup_error = Some(append_cleanup_detail(remaining, detail.clone()));
                failures.push(format!("service `{}`: {detail}", service.id));
            }
        }
        if let Some(error) = service.cleanup_error.as_deref() {
            failures.push(format!(
                "service `{}` retains unrelated cleanup obligation: {error}",
                service.id
            ));
        }
    }
    persist_before_releasing_cleanup_owners(
        || vat.save(),
        || release_persisted_deadline_cleanup_owners(&mut retained_owners),
    )?;
    if failures.is_empty() {
        Ok(())
    } else {
        bail!(
            "unconfirmed runtime cleanup remains: {}",
            failures.join("; ")
        )
    }
}

// </HANDWRITE>

const CLUSTER_CLEANUP_BLOCK_END: &str = "[/vat-cluster-cleanup]";

fn cluster_cleanup_block_start(record: &ClusterRunRecord, cli_group_unconfirmed: bool) -> String {
    format!(
        "[vat-cluster-cleanup backend={} name={} cli_group_unconfirmed={cli_group_unconfirmed}]",
        serde_json::to_string(&record.backend).expect("cluster backend serializes"),
        serde_json::to_string(&record.name).expect("cluster name serializes"),
    )
}

fn cluster_cleanup_detail(record: &ClusterRunRecord, error: &anyhow::Error) -> String {
    let cli_group_unconfirmed = cluster::owned_command_cleanup_failure(error).is_some();
    format!(
        "{} cluster cleanup `{}` ({}) unconfirmed: {error:#}{CLUSTER_CLEANUP_BLOCK_END}",
        cluster_cleanup_block_start(record, cli_group_unconfirmed),
        record.name,
        record.backend,
    )
}

/// Remove only an exact structured cluster-resource obligation. A prior CLI
/// process-group obligation is deliberately retained: a later successful
/// resource delete does not prove that the earlier delete command's PGID is
/// absent. Legacy freeform cluster cleanup text remains fail-closed.
fn clear_cluster_cleanup_obligation(
    cleanup_error: &str,
    record: &ClusterRunRecord,
) -> (Option<String>, bool, bool) {
    let mut remaining = cleanup_error.to_string();
    let mut removed_resource = false;
    let mut cli_group_unconfirmed = false;
    for command_group in [false, true] {
        let marker = cluster_cleanup_block_start(record, command_group);
        while let Some(start) = remaining.find(&marker) {
            if command_group {
                cli_group_unconfirmed = true;
                break;
            }
            let body_start = start + marker.len();
            let Some(relative_end) = remaining[body_start..].find(CLUSTER_CLEANUP_BLOCK_END) else {
                break;
            };
            let end = body_start + relative_end + CLUSTER_CLEANUP_BLOCK_END.len();
            let before = remaining[..start]
                .trim_end()
                .trim_end_matches(';')
                .trim_end();
            let after = remaining[end..]
                .trim_start()
                .strip_prefix(';')
                .unwrap_or_else(|| remaining[end..].trim_start())
                .trim_start();
            remaining = match (before.is_empty(), after.is_empty()) {
                (true, true) => String::new(),
                (false, true) => before.to_string(),
                (true, false) => after.to_string(),
                (false, false) => format!("{before}; {after}"),
            };
            removed_resource = true;
        }
    }
    (
        (!remaining.is_empty()).then_some(remaining),
        removed_resource,
        cli_group_unconfirmed,
    )
}

fn record_cluster_delete_success(service: &mut ServiceHandle, record: &ClusterRunRecord) {
    let existing = service.record.cleanup_error.take();
    let (remaining, _, cli_group_unconfirmed) = existing
        .as_deref()
        .map(|error| clear_cluster_cleanup_obligation(error, record))
        .unwrap_or((None, false, false));
    let legacy_cluster_obligation = existing.as_deref().is_some_and(|error| {
        error.contains("cluster cleanup") && !error.contains("[vat-cluster-cleanup")
    });
    if cli_group_unconfirmed || legacy_cluster_obligation {
        // Resource absence cannot discharge an older CLI PGID obligation.
        // Retain both evidence and the ownership handle for manual recovery.
        service.record.cleanup_error = existing;
        service.record.status = ProcessStatus::Failed;
    } else {
        // The live resource handle is discharged. Keep record.cluster as
        // historical evidence while making the outer finalizer idempotent.
        service.cluster = None;
        service.record.cleanup_error = remaining;
    }
}

fn cleanup_docker_service_runtime(service: &mut ServiceHandle) {
    let Some(name) = service.docker_name.clone() else {
        return;
    };
    let cleanup_already_unconfirmed = has_runtime_cleanup_obligation(
        service.record.cleanup_error.as_deref(),
        RuntimeCleanupKind::Docker,
        &name,
    );
    if cleanup_already_unconfirmed {
        // An inner checkpoint already persisted this exact ownership
        // obligation. The redundant outer finalizer must be a true no-op;
        // retry is an explicit later compose-down operation.
        service.record.status = ProcessStatus::Failed;
        return;
    }
    match cleanup_docker_runtime_handle(
        &mut service.docker_name,
        service.record.docker_id.as_deref(),
        &mut service.deadline_cleanup_owners,
    ) {
        Ok(()) => {
            if let Some(existing) = service.record.cleanup_error.take() {
                let (remaining, _) =
                    clear_runtime_cleanup_obligation(&existing, RuntimeCleanupKind::Docker, &name);
                service.record.cleanup_error = remaining;
            }
        }
        Err(error) => {
            record_runtime_cleanup_failure(service, RuntimeCleanupKind::Docker, &name, &error)
        }
    }
}

fn stop_services(services: &mut [ServiceHandle], delete_clusters: bool) -> Result<()> {
    for service in services.iter_mut().rev() {
        // Remove a Docker container while its foreground `docker run --rm`
        // client is still waitable. Killing that client first can race
        // Docker's automatic removal and leave a second `docker rm -f`
        // blocked behind an already-in-progress daemon delete.
        cleanup_docker_service_runtime(service);

        let mut group_cleanup_failed = false;
        let child_exit = match service.child.as_mut() {
            Some(child) => match child.finalize(&format!("service `{}`", service.record.id)) {
                Ok(status) => Some(status),
                Err(error) => {
                    group_cleanup_failed = true;
                    let detail = format!(
                        "owned process-group cleanup unconfirmed for service `{}`: {error:#}",
                        service.record.id
                    );
                    service.record.cleanup_error =
                        Some(match service.record.cleanup_error.take() {
                            Some(existing) => format!("{existing}; {detail}"),
                            None => detail.clone(),
                        });
                    service.record.status = ProcessStatus::Failed;
                    service.record.pid = (!child.leader_reaped()).then_some(child.id());
                    child.preserve_cleanup_obligation();
                    None
                }
            },
            None => None,
        };
        if service.child.is_some() && !group_cleanup_failed {
            service.record.pid = None;
            if let Some(status) = child_exit {
                service.record.exit_code = status.code();
            }
            if !matches!(
                service.record.status,
                ProcessStatus::Failed | ProcessStatus::Timeout
            ) {
                // A service which had been Ready can exit naturally before a
                // compose stop request. It is still terminal; retaining Ready
                // here would make compose wait forever after VAT itself exits.
                service.record.status = ProcessStatus::Exited;
            }
        }
        // Same force-removal guarantee for a `container run` (MicroVM) child,
        // parallel to the docker_name branch above (R5).
        if let Some(name) = service.microvm_name.clone() {
            let cleanup_already_unconfirmed = has_runtime_cleanup_obligation(
                service.record.cleanup_error.as_deref(),
                RuntimeCleanupKind::MicroVm,
                &name,
            );
            if cleanup_already_unconfirmed {
                service.record.status = ProcessStatus::Failed;
            } else {
                match cleanup_microvm_runtime_handle(
                    &mut service.microvm_name,
                    &mut service.deadline_cleanup_owners,
                ) {
                    Ok(()) => {
                        if let Some(existing) = service.record.cleanup_error.take() {
                            let (remaining, _) = clear_runtime_cleanup_obligation(
                                &existing,
                                RuntimeCleanupKind::MicroVm,
                                &name,
                            );
                            service.record.cleanup_error = remaining;
                        }
                    }
                    Err(error) => record_runtime_cleanup_failure(
                        service,
                        RuntimeCleanupKind::MicroVm,
                        &name,
                        &error,
                    ),
                }
            }
        }
        // A cluster is an external object, so removing the vat dir does NOT
        // remove it. Delete it explicitly when the run policy says to; keep it
        // for `kubectl` diagnosis otherwise.
        if delete_clusters {
            if let Some(record) = service.cluster.clone() {
                let cleanup = match ResolvedBackend::from_name(&record.backend) {
                    Some(backend) => backend.delete(&record.name),
                    None => Err(anyhow::anyhow!(
                        "unknown cluster backend `{}`",
                        record.backend
                    )),
                };
                match cleanup {
                    Ok(()) => record_cluster_delete_success(service, &record),
                    Err(error) => {
                        let detail = cluster_cleanup_detail(&record, &error);
                        service.record.status = ProcessStatus::Failed;
                        service.record.cleanup_error = Some(
                            service
                                .record
                                .cleanup_error
                                .take()
                                .map(|existing| format!("{existing}; {detail}"))
                                .unwrap_or(detail),
                        );
                    }
                }
            }
        }
    }
    let cleanup_failures = services
        .iter_mut()
        .filter_map(|service| {
            let error = service.record.cleanup_error.as_deref()?;
            service.record.status = ProcessStatus::Failed;
            Some(format!("service `{}`: {error}", service.record.id))
        })
        .collect::<Vec<_>>();
    if cleanup_failures.is_empty() {
        Ok(())
    } else {
        bail!(
            "VAT-owned service cleanup unconfirmed: {}",
            cleanup_failures.join("; ")
        )
    }
}

fn finalize_services_and_persist(
    vat: &mut store::Vat,
    services: &mut [ServiceHandle],
    delete_clusters: bool,
    interruption: Option<&RunInterrupted>,
) -> Result<()> {
    let cleanup = stop_services(services, delete_clusters);
    if cleanup.is_ok() {
        if let Some(interruption) = interruption {
            mark_services_interrupted(services, interruption);
        }
    } else if let Some(interruption) = interruption {
        for service in services.iter_mut() {
            if let Some(error) = service.record.cleanup_error.as_mut() {
                let signal_context = format!("cleanup attempted after {}", interruption.reason);
                if !error.contains(&signal_context) {
                    *error = format!("{error}; {signal_context}");
                }
            }
        }
    }
    let persistence = persist_services(vat, services);
    if persistence.is_ok() {
        for service in services.iter_mut() {
            release_persisted_deadline_cleanup_owners(&mut service.deadline_cleanup_owners);
        }
    }
    match (cleanup, persistence) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(cleanup), Ok(())) => Err(cleanup),
        (Ok(()), Err(persistence)) => Err(persistence),
        (Err(cleanup), Err(persistence)) => Err(persistence).context(format!(
            "persist service cleanup evidence after cleanup failure: {cleanup:#}"
        )),
    }
}

fn release_persisted_deadline_cleanup_owners(owners: &mut Vec<OwnedProcessGroup>) {
    for mut owner in owners.drain(..) {
        // The durable cleanup_error already names this leader PID/PGID. A
        // second best-effort finalizer is therefore allowed to outlive the
        // original operation deadline without ever losing the in-memory owner
        // before persistence. Cached post-reap failures remain non-signalling.
        let _ = owner.finalize("persisted runtime cleanup helper");
    }
}

fn persist_before_releasing_cleanup_owners<T>(
    persist: impl FnOnce() -> Result<T>,
    release: impl FnOnce(),
) -> Result<T> {
    let persisted = persist()?;
    release();
    Ok(persisted)
}

/// The configured-run teardown owner. It always attempts runner groups first,
/// then services in reverse start order, and persists both outcome sets before
/// returning either failure. No caller may short-circuit between those phases.
fn finalize_configured_children(
    vat: &mut store::Vat,
    procs: &mut [RunnerProc],
    services: &mut [ServiceHandle],
    delete_clusters: bool,
    interruption: Option<&RunInterrupted>,
) -> Result<()> {
    let runner_cleanup = finalize_runner_processes(procs);
    match (&runner_cleanup, interruption) {
        (Ok(()), Some(interruption)) => {
            let records = interrupted_runner_records(procs, interruption);
            if let Some(test_run) = vat.meta.test_run.as_mut() {
                test_run.runner = records.first().cloned();
                test_run.runners = records;
            }
        }
        (Err(_), _) => {
            record_runner_cleanup_outcomes(vat, procs);
            if let (Some(test_run), Some(interruption)) = (vat.meta.test_run.as_mut(), interruption)
            {
                for runner in &mut test_run.runners {
                    if let Some(error) = runner.cleanup_error.as_mut() {
                        let signal_context =
                            format!("cleanup attempted after {}", interruption.reason);
                        if !error.contains(&signal_context) {
                            *error = format!("{error}; {signal_context}");
                        }
                    }
                }
                test_run.runner = test_run.runners.first().cloned();
            }
        }
        (Ok(()), None) => {}
    }
    let service_cleanup =
        finalize_services_and_persist(vat, services, delete_clusters, interruption);
    let cleanup = match (runner_cleanup, service_cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(runners), Ok(())) => Err(runners),
        (Ok(()), Err(services)) => Err(services),
        (Err(runners), Err(services)) => Err(services).context(format!(
            "service cleanup also followed runner cleanup failure: {runners:#}"
        )),
    };
    match (cleanup, interruption) {
        (Err(error), Some(interruption)) => Err(RunCleanupFailed {
            interruption: interruption.clone(),
            cleanup_error: format!("{error:#}"),
        }
        .into()),
        (result, _) => result,
    }
}

#[derive(Debug)]
struct ClusterCleanupFailure {
    backend: String,
    name: String,
    error: String,
}

impl ClusterCleanupFailure {
    fn detail(&self) -> String {
        format!(
            "prepared cluster `{}` ({}) cleanup unconfirmed: {}",
            self.name, self.backend, self.error
        )
    }
}

fn cleanup_unstarted_cluster_plans(vat: &mut store::Vat, plans: &[ServicePlan]) -> Result<()> {
    let failures = cleanup_cluster_records_with(
        plans.iter().filter_map(|plan| plan.cluster.as_ref()),
        |backend, name| backend.delete(name),
    );
    if failures.is_empty() {
        return Ok(());
    }
    if let Some(test_run) = vat.meta.test_run.as_mut() {
        for failure in &failures {
            let Some(plan) = plans.iter().find(|plan| {
                plan.cluster
                    .as_ref()
                    .is_some_and(|cluster| cluster.name == failure.name)
            }) else {
                continue;
            };
            let detail = failure.detail();
            let record = ServiceRunRecord {
                id: plan.id.clone(),
                command: plan.command.clone(),
                status: ProcessStatus::Failed,
                preset: plan.preset.map(service_preset_name).map(str::to_string),
                host: plan.host.clone(),
                port: plan.port,
                owned_by_vat: Some(true),
                prepare_mode: Some(plan.prepare_mode.clone()),
                cache_key: plan.cache_key.clone(),
                prepare_duration_ms: Some(plan.prepare_duration_ms),
                ready_duration_ms: None,
                exported_env: plan.exported_env.clone(),
                pid: None,
                exit_code: None,
                ready_http: plan.ready_http.clone(),
                docker_name: None,
                docker_id: None,
                microvm_name: None,
                readiness_error: None,
                cleanup_error: Some(detail),
                cluster: plan.cluster.clone(),
                stdout_log: vat
                    .dir
                    .join(crate::paths::file::LOGS)
                    .join(format!("{}.stdout.log", plan.id))
                    .to_string_lossy()
                    .into_owned(),
                stderr_log: vat
                    .dir
                    .join(crate::paths::file::LOGS)
                    .join(format!("{}.stderr.log", plan.id))
                    .to_string_lossy()
                    .into_owned(),
            };
            if let Some(existing) = test_run
                .services
                .iter_mut()
                .find(|service| service.id == plan.id)
            {
                *existing = record;
            } else {
                test_run.services.push(record);
            }
        }
    }
    vat.save()
        .context("persist prepared cluster cleanup obligations")?;
    bail!(
        "prepared-but-unstarted cluster cleanup unconfirmed: {}",
        failures
            .iter()
            .map(ClusterCleanupFailure::detail)
            .collect::<Vec<_>>()
            .join("; ")
    )
}

fn cleanup_cluster_records_with<'a>(
    records: impl IntoIterator<Item = &'a ClusterRunRecord>,
    mut delete: impl FnMut(ResolvedBackend, &str) -> Result<()>,
) -> Vec<ClusterCleanupFailure> {
    let mut failures = Vec::new();
    for record in records {
        let Some(backend) = ResolvedBackend::from_name(&record.backend) else {
            failures.push(ClusterCleanupFailure {
                backend: record.backend.clone(),
                name: record.name.clone(),
                error: "unknown cluster backend".to_string(),
            });
            continue;
        };
        if let Err(error) = delete(backend, &record.name) {
            failures.push(ClusterCleanupFailure {
                backend: record.backend.clone(),
                name: record.name.clone(),
                error: format!("{error:#}"),
            });
        }
    }
    failures
}

/// Whether run-scoped clusters should be deleted at teardown, mirroring the
/// workspace removal decision: removed → delete the cluster; kept → keep it for
/// diagnosis. `code < 0` (an error before a clean exit) is treated as failure.
fn should_delete_clusters(keep: &RetentionPolicy, code: i32) -> bool {
    match keep {
        RetentionPolicy::Always => false,
        RetentionPolicy::Never => true,
        RetentionPolicy::Failed => code == 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::ExternalServiceConfig;

    fn test_cancellation() -> RunCancellation {
        RunCancellation::with_observed_signal(0)
    }

    #[test]
    fn service_startup_stop_request_is_consumed_once_by_the_vat_owner() {
        let temp = tempfile::tempdir().expect("compose stop request tempdir");
        let request = temp.path().join(".compose-stop-request");
        std::fs::write(&request, b"stop").expect("seed compose stop request");
        let error = consume_detached_compose_stop_request(&request, "service readiness")
            .expect_err("first owner poll must consume the stop request");
        assert!(error.to_string().contains("service readiness"));
        assert!(!request.exists(), "the stop request must be one-shot");
        consume_detached_compose_stop_request(&request, "service readiness")
            .expect("a consumed request cannot stop the owner twice");
    }

    fn pre_child_test_vat(mode: &str) -> (tempfile::TempDir, store::Vat) {
        let temp = tempfile::tempdir().expect("pre-child VAT tempdir");
        let now = Utc::now();
        let last_run = (mode == "direct").then(|| RunRecord {
            command: vec!["missing-runtime".to_string()],
            started_at: now,
            finished_at: None,
            exit_code: None,
            duration_ms: None,
            signal: None,
            owned_pgid: None,
            cleanup_error: None,
        });
        let test_run = (mode != "direct").then(|| TestRunEvidence {
            config: ConfigRef {
                path: "vat.toml".to_string(),
                digest: "test".to_string(),
            },
            runner_id: "runner".to_string(),
            retention: RetentionPolicy::Always,
            services: Vec::new(),
            scenario: (mode == "scenario").then(|| ScenarioRunRecord {
                id: "scenario".to_string(),
                app: "app".to_string(),
                runner: "runner".to_string(),
                network: "open".to_string(),
                services: Vec::new(),
                routes: Vec::new(),
                hermetic: false,
            }),
            runner: None,
            runners: Vec::new(),
            artifacts: Vec::new(),
            cleanup_error: None,
            plan: None,
            topology: None,
        });
        let mut vat = store::Vat {
            dir: temp.path().to_path_buf(),
            meta: crate::state::VatMeta {
                id: format!("vat-pre-child-{mode}"),
                name: None,
                status: Status::Running,
                created_at: now,
                updated_at: now,
                spec: EnvSpec::default(),
                lineage: Vec::new(),
                last_run,
                test_run,
                plan: None,
            },
        };
        vat.save().expect("persist Running pre-child state");
        (temp, vat)
    }

    fn assert_pre_child_failure_persisted(mode: &str) -> crate::state::VatMeta {
        let (_temp, mut vat) = pre_child_test_vat(mode);
        let cancellation = RunCancellation::new().expect("test cancellation observer");
        let step: Result<()> = Err(anyhow::anyhow!("injected {mode} pre-child failure"));
        let error = finish_pre_child_step(
            &mut vat,
            &cancellation,
            Instant::now(),
            step,
            "injected pre-child step",
        )
        .expect_err("pre-child failure must propagate");
        assert!(error.to_string().contains("injected pre-child step"));
        serde_json::from_slice(&std::fs::read(vat.meta_path()).expect("read persisted meta"))
            .expect("parse persisted meta")
    }

    #[test]
    fn direct_pre_child_failure_terminalizes_running_record() {
        let meta = assert_pre_child_failure_persisted("direct");
        assert_eq!(meta.status, Status::Exited { code: -1 });
        let run = meta.last_run.expect("direct run record");
        assert!(run.finished_at.is_some());
        assert_eq!(run.exit_code, Some(-1));
        assert!(run.duration_ms.is_some());
        assert_eq!(run.signal, None);
        assert_eq!(run.owned_pgid, None);
        assert_eq!(run.cleanup_error, None);
    }

    #[test]
    fn configured_runner_pre_child_failure_terminalizes_running_vat() {
        let meta = assert_pre_child_failure_persisted("runner");
        assert_eq!(meta.status, Status::Exited { code: -1 });
        assert!(meta.test_run.is_some());
        assert!(meta.last_run.is_none());
    }

    #[test]
    fn scenario_pre_child_failure_terminalizes_running_vat() {
        let meta = assert_pre_child_failure_persisted("scenario");
        assert_eq!(meta.status, Status::Exited { code: -1 });
        assert!(meta
            .test_run
            .and_then(|run| run.scenario)
            .is_some_and(|scenario| scenario.id == "scenario"));
    }

    #[test]
    fn concurrent_pre_child_signal_is_terminal_and_returns_signal_exit_for_all_modes() {
        for mode in ["direct", "runner", "scenario"] {
            let (_temp, mut vat) = pre_child_test_vat(mode);
            let cancellation = RunCancellation::with_observed_signal(libc::SIGTERM);
            let error = finish_pre_child_step(
                &mut vat,
                &cancellation,
                Instant::now(),
                Err::<(), _>(anyhow::anyhow!("injected {mode} evidence failure")),
                "injected signal-racing pre-child step",
            )
            .expect_err("signal must win the process exit contract");
            assert_eq!(
                run_interruption(&error).map(|interruption| interruption.signal),
                Some(libc::SIGTERM)
            );
            let exit = finish_exec_result(Err(error)).expect("signal maps to a process exit code");
            assert_eq!(exit, ExitCode::from(143));
            let persisted: crate::state::VatMeta = serde_json::from_slice(
                &std::fs::read(vat.meta_path()).expect("read persisted signal state"),
            )
            .expect("parse persisted signal state");
            assert!(matches!(
                persisted.status,
                Status::Interrupted {
                    signal: libc::SIGTERM,
                    ..
                }
            ));
            if let Some(run) = persisted.last_run {
                assert_eq!(run.exit_code, Some(143));
                assert_eq!(run.signal, Some(libc::SIGTERM));
                assert!(run.finished_at.is_some());
            }
        }
    }

    #[test]
    fn runner_failure_log_error_cannot_preempt_terminal_metadata_persistence() {
        let (_temp, mut vat) = pre_child_test_vat("runner");
        let logs_blocker = vat.dir.join("logs-blocker");
        std::fs::write(&logs_blocker, b"not a directory").expect("create log-path blocker");
        let runner = RunnerConfig {
            id: "runner".to_string(),
            requires: Vec::new(),
            cmd: vec!["false".to_string()],
            timeout_s: None,
            artifacts: Vec::new(),
        };
        let message = record_runner_failure_fail_closed(
            &mut vat,
            &runner,
            &logs_blocker,
            "injected execution failure",
        );
        assert!(message.contains("runner failure evidence write failed"));
        vat.meta.status = Status::Exited { code: -1 };
        vat.save()
            .expect("terminal metadata save must remain reachable after log failure");

        let persisted: crate::state::VatMeta = serde_json::from_slice(
            &std::fs::read(vat.meta_path()).expect("read terminal metadata"),
        )
        .expect("parse terminal metadata");
        assert_eq!(persisted.status, Status::Exited { code: -1 });
        let run = persisted.test_run.expect("configured run evidence");
        assert!(run
            .cleanup_error
            .as_deref()
            .is_some_and(|error| error.contains("runner failure evidence write failed")));
        assert!(run.runners.iter().any(|runner| {
            runner.id == "runner"
                && runner.status == ProcessStatus::Failed
                && runner.exit_code == Some(-1)
        }));
    }

    #[test]
    fn later_failure_evidence_does_not_rewrite_completed_runner_outcome() {
        let (_temp, mut vat) = pre_child_test_vat("runner");
        let logs_dir = vat.dir.join("logs");
        std::fs::create_dir_all(&logs_dir).expect("create runner logs");
        let runner = RunnerConfig {
            id: "runner".to_string(),
            requires: Vec::new(),
            cmd: vec!["true".to_string()],
            timeout_s: None,
            artifacts: Vec::new(),
        };
        let completed = RunnerRunRecord {
            id: runner.id.clone(),
            command: runner.cmd.clone(),
            status: ProcessStatus::Exited,
            exit_code: Some(0),
            duration_ms: Some(12),
            pid: None,
            cleanup_error: None,
            stdout_log: logs_dir
                .join("runner.stdout.log")
                .to_string_lossy()
                .into_owned(),
            stderr_log: logs_dir
                .join("runner.stderr.log")
                .to_string_lossy()
                .into_owned(),
        };
        let test_run = vat.meta.test_run.as_mut().expect("configured run evidence");
        test_run.runner = Some(completed.clone());
        test_run.runners = vec![completed];

        record_runner_failure(&mut vat, &runner, &logs_dir, "later MicroVM cleanup failed")
            .expect("append later failure evidence");

        let test_run = vat.meta.test_run.as_ref().expect("configured run evidence");
        assert_eq!(test_run.runners.len(), 1);
        assert_eq!(test_run.runners[0].status, ProcessStatus::Exited);
        assert_eq!(test_run.runners[0].exit_code, Some(0));
        assert_eq!(test_run.runners[0].duration_ms, Some(12));
        let compatibility = test_run.runner.as_ref().expect("compatibility runner");
        assert_eq!(compatibility.status, ProcessStatus::Exited);
        assert_eq!(compatibility.exit_code, Some(0));
    }

    #[test]
    fn container_diagnostic_cleanup_failure_is_deferred_after_one_second_policy() {
        assert_eq!(CONTAINER_DIAGNOSTIC_TIMEOUT, Duration::from_secs(1));
        let cleanup_detail = "diagnostic process group 4242 remains";
        let outcome = classify_container_diagnostic(
            &["inspect", "owned-microvm"],
            Err(RunOwnedCleanupFailed {
                cleanup_error: cleanup_detail.to_string(),
            }
            .into()),
        );
        assert!(outcome.evidence.contains("container inspect owned-microvm"));
        assert!(outcome.evidence.contains(cleanup_detail));
        let deferred = outcome
            .deferred_error
            .expect("cleanup-unconfirmed diagnostic must be deferred");
        assert_eq!(
            diagnostic_cleanup_error(&deferred).as_deref(),
            Some(cleanup_detail)
        );
    }

    #[cfg(unix)]
    #[test]
    fn pre_service_cancellation_cleans_term_resistant_helper_descendants() {
        let temp = tempfile::tempdir().expect("tempdir");
        let leader_marker = temp.path().join("leader.pid");
        let descendant_marker = temp.path().join("descendant.pid");
        let cancellation = test_cancellation();
        let signal = Arc::clone(&cancellation.first_signal);
        let leader_for_signal = leader_marker.clone();
        let descendant_for_signal = descendant_marker.clone();
        let signaler = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(2);
            while !(leader_for_signal.exists() && descendant_for_signal.exists()) {
                assert!(
                    Instant::now() < deadline,
                    "helper never published pid markers"
                );
                std::thread::sleep(OWNED_GROUP_POLL_INTERVAL);
            }
            signal.store(libc::SIGTERM, Ordering::Release);
        });
        let mut command = Command::new("/bin/sh");
        command
            .env("LEADER_MARKER", &leader_marker)
            .env("DESCENDANT_MARKER", &descendant_marker)
            .args([
                "-c",
                "echo $$ > \"$LEADER_MARKER\"; /bin/sh -c 'trap \"\" TERM; echo $$ > \"$DESCENDANT_MARKER\"; while :; do :; done' & trap '' TERM; while :; do :; done",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let started = Instant::now();
        let error = run_bounded_owned_command(
            command,
            "configured pre-service helper",
            Duration::from_secs(5),
            Some(&cancellation),
            &|| Ok(()),
        )
        .expect_err("helper must observe cancellation");
        signaler.join().expect("signal thread");
        assert_eq!(
            run_interruption(&error).map(|interruption| interruption.signal),
            Some(libc::SIGTERM)
        );
        assert!(started.elapsed() < Duration::from_secs(3));
        assert_test_pid_markers_absent(&[leader_marker, descendant_marker]);
    }

    #[cfg(unix)]
    fn assert_cancellable_microvm_helper_cleans_descendants(
        invoke: impl FnOnce(&[&str], &RunCancellation) -> Result<()>,
    ) {
        let temp = tempfile::tempdir().expect("tempdir");
        let leader_marker = temp.path().join("leader.pid");
        let descendant_marker = temp.path().join("descendant.pid");
        let leader_path = leader_marker.to_string_lossy().into_owned();
        let descendant_path = descendant_marker.to_string_lossy().into_owned();
        let script = "trap '' TERM; echo $$ > \"$1\"; DESCENDANT_MARKER=\"$2\" /bin/sh -c 'trap \"\" TERM; echo $$ > \"$DESCENDANT_MARKER\"; while :; do :; done' & while :; do :; done";
        let args = [
            "-c",
            script,
            "vat-microvm-helper-test",
            leader_path.as_str(),
            descendant_path.as_str(),
        ];
        let cancellation = test_cancellation();
        let signal = Arc::clone(&cancellation.first_signal);
        let leader_for_signal = leader_marker.clone();
        let descendant_for_signal = descendant_marker.clone();
        let signaler = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(2);
            while !(leader_for_signal.exists() && descendant_for_signal.exists()) {
                assert!(
                    Instant::now() < deadline,
                    "helper never published pid markers"
                );
                std::thread::sleep(OWNED_GROUP_POLL_INTERVAL);
            }
            signal.store(libc::SIGTERM, Ordering::Release);
        });

        let started = Instant::now();
        let error = invoke(&args, &cancellation).expect_err("helper must observe cancellation");
        signaler.join().expect("signal thread");
        assert_eq!(
            run_interruption(&error).map(|interruption| interruption.signal),
            Some(libc::SIGTERM)
        );
        assert!(started.elapsed() < Duration::from_secs(3));
        assert_test_pid_markers_absent(&[leader_marker, descendant_marker]);
    }

    #[cfg(unix)]
    #[test]
    fn microvm_system_probe_cancellation_cleans_helper_descendants() {
        assert_cancellable_microvm_helper_cleans_descendants(|args, cancellation| {
            ensure_microvm_system_started_with(
                "/bin/sh",
                args,
                Duration::from_secs(5),
                cancellation,
            )
        });
    }

    #[cfg(unix)]
    #[test]
    fn microvm_diagnostic_cancellation_cleans_helper_descendants() {
        assert_cancellable_microvm_helper_cleans_descendants(|args, cancellation| {
            command_diagnostic_until(
                "/bin/sh",
                args,
                Instant::now() + Duration::from_secs(5),
                Some(cancellation),
            )
            .map(|_| ())
        });
    }

    #[cfg(unix)]
    #[test]
    fn postgres_seed_stop_faults_persist_typed_recovery_obligation() {
        let temp = tempfile::tempdir().expect("tempdir");
        let data_dir = temp.path().join("postgres-data");
        std::fs::create_dir_all(&data_dir).expect("create postgres data dir");
        std::fs::write(data_dir.join("postmaster.pid"), b"4242\n")
            .expect("write detached postgres identity");
        let service = test_service("postgres", &[]);
        let mut cleanup_details = Vec::new();

        for (label, script, timeout) in [
            ("nonzero", "exit 7", Duration::from_secs(1)),
            (
                "timeout",
                "trap '' TERM; while :; do :; done",
                Duration::from_millis(50),
            ),
        ] {
            let mut command = Command::new("/bin/sh");
            command
                .args(["-c", script])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            let error = stop_postgres_seed_server_command(&data_dir, &service, command, timeout)
                .expect_err("postmaster identity requires a typed cleanup obligation");
            let contextual = error.context(format!(
                "postgres seed {label} path also failed before shutdown"
            ));
            let failure = run_owned_cleanup_failure(&contextual)
                .expect("cleanup type must survive seed/start/psql context");
            assert!(failure.cleanup_error.contains("service `postgres`"));
            assert!(failure
                .cleanup_error
                .contains(&format!("data_dir={}", data_dir.display())));
            assert!(failure.cleanup_error.contains("postmaster_pid=4242"));
            assert!(failure.cleanup_error.contains("recovery=`pg_ctl -D"));
            assert!(failure.cleanup_error.contains("-m fast stop`"));
            cleanup_details.push(failure.cleanup_error.clone());
        }

        let (_vat_temp, mut vat) = pre_child_test_vat("runner");
        for detail in &cleanup_details {
            append_test_run_cleanup_error(&mut vat, detail);
        }
        let logs_dir = vat.dir.join("logs");
        std::fs::create_dir_all(&logs_dir).expect("create runner logs");
        let runner = RunnerConfig {
            id: "runner".to_string(),
            requires: Vec::new(),
            cmd: vec!["true".to_string()],
            timeout_s: None,
            artifacts: Vec::new(),
        };
        record_runner_failure_fail_closed(
            &mut vat,
            &runner,
            &logs_dir,
            "temporary postgres cleanup failed",
        );
        vat.save().expect("persist typed cleanup evidence");
        let persisted: crate::state::VatMeta = serde_json::from_slice(
            &std::fs::read(vat.meta_path()).expect("read persisted cleanup evidence"),
        )
        .expect("parse persisted cleanup evidence");
        let cleanup_error = persisted
            .test_run
            .and_then(|run| run.cleanup_error)
            .expect("test_run cleanup obligation");
        assert!(cleanup_error.contains(&data_dir.to_string_lossy().into_owned()));
        assert!(cleanup_error.contains("postmaster_pid=4242"));
        assert!(cleanup_error.contains("pg_ctl -D"));
        assert!(!should_remove_vat(&RetentionPolicy::Never, -1, true, false));
    }

    #[cfg(unix)]
    fn assert_test_pid_markers_absent(markers: &[PathBuf]) {
        for marker in markers {
            let pid = std::fs::read_to_string(marker)
                .unwrap_or_else(|error| panic!("read {}: {error}", marker.display()))
                .trim()
                .parse::<i32>()
                .expect("fixture pid");
            let deadline = Instant::now() + Duration::from_secs(1);
            loop {
                let result = unsafe { libc::kill(pid, 0) };
                if result == -1
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
                {
                    break;
                }
                assert!(Instant::now() < deadline, "pid {pid} survived cleanup");
                std::thread::sleep(OWNED_GROUP_POLL_INTERVAL);
            }
        }
    }

    #[test]
    fn runtime_retry_clears_only_its_structured_cleanup_obligation() {
        let runtime = runtime_cleanup_detail(
            RuntimeCleanupKind::Docker,
            "owned-container",
            &anyhow::anyhow!("daemon timeout; diagnostic detail"),
        );
        let mixed = format!(
            "owned process-group cleanup unconfirmed; {runtime}; cluster cleanup unconfirmed; readiness command cleanup unconfirmed"
        );
        let (remaining, removed) =
            clear_runtime_cleanup_obligation(&mixed, RuntimeCleanupKind::Docker, "owned-container");
        assert!(removed);
        let remaining = remaining.expect("unrelated obligations remain");
        assert!(remaining.contains("owned process-group cleanup unconfirmed"));
        assert!(remaining.contains("cluster cleanup unconfirmed"));
        assert!(remaining.contains("readiness command cleanup unconfirmed"));
        assert!(!remaining.contains("vat-runtime-cleanup"));

        let (unchanged, removed) = clear_runtime_cleanup_obligation(
            &mixed,
            RuntimeCleanupKind::Docker,
            "replacement-container",
        );
        assert!(!removed);
        assert_eq!(unchanged.as_deref(), Some(mixed.as_str()));

        let legacy_mixed = "Docker cleanup `docker rm -f owned-container` did not finish: timeout; cluster cleanup `owned-cluster` unconfirmed";
        let (unchanged, removed) = clear_runtime_cleanup_obligation(
            legacy_mixed,
            RuntimeCleanupKind::Docker,
            "owned-container",
        );
        assert!(!removed, "legacy mixed text must remain fail-closed");
        assert_eq!(unchanged.as_deref(), Some(legacy_mixed));
    }

    #[test]
    fn deadline_cleanup_owner_is_released_only_after_durable_persistence() {
        let events = std::cell::RefCell::new(Vec::new());
        persist_before_releasing_cleanup_owners(
            || {
                events.borrow_mut().push("persist");
                Ok(())
            },
            || events.borrow_mut().push("release"),
        )
        .expect("persistence before release");
        assert_eq!(&*events.borrow(), &["persist", "release"]);

        let failed_events = std::cell::RefCell::new(Vec::new());
        let error = persist_before_releasing_cleanup_owners::<()>(
            || {
                failed_events.borrow_mut().push("persist-failed");
                bail!("injected persistence failure")
            },
            || failed_events.borrow_mut().push("release"),
        )
        .expect_err("failed persistence must retain the owner");
        assert!(error.to_string().contains("injected persistence failure"));
        assert_eq!(&*failed_events.borrow(), &["persist-failed"]);
    }

    /// R10/AC4: the run.rs `gpu_satisfied()` preflight helper — the second,
    /// independent fail-closed layer alongside `sandbox::pick()` — must
    /// reject `--isolation micro_vm --gpu required` even when the host GPU
    /// is genuinely accessible. Before this helper existed, the three
    /// preflight call sites in `exec_direct`/`exec_runner`/`exec_scenario`
    /// checked only `gpu_info.accessible`, so on a real Apple Silicon host
    /// (accessible=true) `--isolation micro_vm --gpu required` would
    /// silently pass preflight and only fail later inside `pick()` — after
    /// EnvSpec construction, i.e. much closer to (and per the TD, "before
    /// any workspace clone begins" is the guarantee this test pins down).
    #[test]
    fn gpu_satisfied_rejects_microvm_required_before_workspace_clone() {
        let accessible_info = gpu::GpuInfo {
            vendor: "apple".to_string(),
            chip: Some("Apple M-series".to_string()),
            backends: vec!["metal".to_string()],
            accessible: true,
            note: "GPU is accessible".to_string(),
        };

        // Host genuinely has a GPU, but MicroVm isolation can never reach it:
        // gpu_satisfied() must say "not satisfied" independent of pick().
        assert!(
            !gpu_satisfied(GpuRequest::Required, Isolation::MicroVm, &accessible_info),
            "gpu_satisfied() must reject MicroVm even when the host GPU is accessible"
        );

        // Sanity: the same accessible info still satisfies non-MicroVm
        // isolation modes (no regression on the existing None/Seatbelt path).
        assert!(gpu_satisfied(
            GpuRequest::Required,
            Isolation::None,
            &accessible_info
        ));
        assert!(gpu_satisfied(
            GpuRequest::Required,
            Isolation::Seatbelt,
            &accessible_info
        ));

        // And when the host genuinely has no GPU, MicroVm is still rejected
        // (not a special case — just categorically never satisfied).
        let inaccessible_info = gpu::GpuInfo {
            vendor: "none".to_string(),
            chip: None,
            backends: vec![],
            accessible: false,
            note: "no GPU".to_string(),
        };
        assert!(!gpu_satisfied(
            GpuRequest::Required,
            Isolation::MicroVm,
            &inaccessible_info
        ));
    }

    #[test]
    fn sandbox_wrap_wraps_runner_under_seatbelt_passthrough_under_none() {
        let cmd = vec!["echo".to_string(), "hi".to_string()];

        // isolation=none + egress=open → process backend → byte-identical
        // passthrough (the shape services keep, since they bypass
        // sandbox_wrap entirely). This is the one combination that must keep
        // succeeding unchanged (issue #1300 AC2).
        let none = sandbox::pick(&EnvSpec {
            isolation: Isolation::None,
            ..EnvSpec::default()
        })
        .expect("isolation=none + egress=open must still succeed");
        assert_eq!(
            sandbox_wrap(none.as_ref(), Path::new("/tmp/vat-x"), &cmd),
            cmd
        );
        // empty command is a no-op.
        assert!(sandbox_wrap(none.as_ref(), Path::new("/tmp/vat-x"), &[]).is_empty());

        // isolation=seatbelt → runner cmd is wrapped in `sandbox-exec -p <profile>`
        // (the same profile #518 proves denies external egress) when seatbelt is
        // available; when it's unavailable (e.g. off-macOS CI), pick() now fails
        // closed instead of silently falling back to the process backend (#1300).
        match sandbox::pick(&EnvSpec {
            isolation: Isolation::Seatbelt,
            egress: crate::spec::EgressPolicy::LocalhostOnly,
            ..EnvSpec::default()
        }) {
            Ok(sb) => {
                let wrapped = sandbox_wrap(sb.as_ref(), Path::new("/tmp/vat-x"), &cmd);
                assert_eq!(sb.name(), "seatbelt");
                assert_eq!(wrapped[0], "sandbox-exec");
                assert_eq!(wrapped[1], "-p");
                // the original command is appended verbatim after the profile.
                assert_eq!(&wrapped[wrapped.len() - 2..], cmd.as_slice());
            }
            Err(message) => {
                assert!(message.contains("sandbox-exec"), "message: {message}");
            }
        }
    }

    #[test]
    fn stop_services_stops_in_reverse_start_order() {
        let temp = tempfile::tempdir().expect("tempdir");
        let order_path = temp.path().join("stop-order.txt");
        let mut services = vec![
            spawn_trapping_service(temp.path(), &order_path, "postgres"),
            spawn_trapping_service(temp.path(), &order_path, "backend"),
            spawn_trapping_service(temp.path(), &order_path, "frontend"),
        ];

        std::thread::sleep(Duration::from_millis(100));
        stop_services(&mut services, false).expect("stop services");

        let order = std::fs::read_to_string(&order_path).expect("stop order");
        assert_eq!(
            order.lines().collect::<Vec<_>>(),
            vec!["frontend", "backend", "postgres"]
        );
        assert!(services
            .iter()
            .all(|service| service.record.status == ProcessStatus::Exited));

        // R4: the one process-group finalizer is idempotent. A repeated
        // lifecycle cleanup must neither signal a recycled PGID nor replay the
        // service's TERM handler.
        stop_services(&mut services, false).expect("repeat stop is a no-op");
        let repeated = std::fs::read_to_string(&order_path).expect("repeat stop order");
        assert_eq!(
            repeated.lines().collect::<Vec<_>>(),
            vec!["frontend", "backend", "postgres"]
        );
    }

    #[test]
    fn cleanup_failure_after_signal_never_publishes_interrupted_success() {
        let interruption = RunInterrupted::new(libc::SIGTERM);
        let error: anyhow::Error = RunCleanupFailed {
            interruption: interruption.clone(),
            cleanup_error: "runner group 42 remains".to_string(),
        }
        .into();
        let failure = run_cleanup_failure(&error).expect("typed cleanup failure");
        assert_eq!(failure.interruption.signal, libc::SIGTERM);
        assert!(failure.cleanup_error.contains("group 42"));
        assert_eq!(
            configured_terminal_status(Some(&interruption), true, -1),
            Status::Exited { code: -1 }
        );
        assert!(matches!(
            configured_terminal_status(Some(&interruption), false, 143),
            Status::Interrupted {
                signal: libc::SIGTERM,
                ..
            }
        ));
    }

    #[test]
    fn cleanup_obligation_is_failed_and_not_overwritten_by_interruption() {
        let temp = tempfile::tempdir().expect("tempdir");
        let order_path = temp.path().join("stop-order.txt");
        let mut services = vec![spawn_trapping_service(temp.path(), &order_path, "owned")];
        services[0].record.cleanup_error = Some("synthetic PGID absence failure".to_string());
        std::thread::sleep(Duration::from_millis(100));

        let error = stop_services(&mut services, false).expect_err("cleanup must fail closed");
        assert!(error.to_string().contains("synthetic PGID absence failure"));
        mark_services_interrupted(&mut services, &RunInterrupted::new(libc::SIGINT));
        assert_eq!(services[0].record.status, ProcessStatus::Failed);
        assert!(services[0].record.pid.is_none());
        assert!(services[0].record.cleanup_error.is_some());
    }

    #[cfg(unix)]
    #[test]
    fn configured_cleanup_failure_disables_drop_retry() {
        let temp = tempfile::tempdir().expect("tempdir");
        let command = vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            "trap 'exit 0' TERM; while :; do sleep 1; done".to_string(),
        ];
        let mut runner_child = command_with_logs(
            &command,
            temp.path(),
            &BTreeMap::new(),
            &temp.path().join("runner.stdout"),
            &temp.path().join("runner.stderr"),
        )
        .expect("spawn runner child");
        runner_child.post_reap_cleanup_error =
            Some("injected pre-reap cleanup failure".to_string());
        let mut runners = vec![RunnerProc {
            runner: RunnerConfig {
                id: "runner".to_string(),
                requires: Vec::new(),
                cmd: command,
                timeout_s: None,
                artifacts: Vec::new(),
            },
            child: runner_child,
            cleanup_error: None,
            started: Instant::now(),
            deadline: None,
            stdout_log: String::new(),
            stderr_log: String::new(),
        }];
        assert!(finalize_runner_processes(&mut runners).is_err());
        assert!(runners[0].cleanup_error.is_some());
        assert!(!runners[0].child.drop_cleanup_enabled);

        // Explicit test cleanup after proving Drop cannot retry implicitly.
        runners[0].child.post_reap_cleanup_error = None;
        runners[0].child.drop_cleanup_enabled = true;
        runners[0]
            .child
            .finalize("test runner cleanup")
            .expect("clean test runner");

        let order_path = temp.path().join("stop-order.txt");
        let mut service = spawn_trapping_service(temp.path(), &order_path, "service");
        let child = service.child.as_mut().expect("owned service child");
        child.post_reap_cleanup_error = Some("injected pre-reap cleanup failure".to_string());
        assert!(stop_services(std::slice::from_mut(&mut service), false).is_err());
        let child = service.child.as_mut().expect("owned service child");
        assert!(!child.drop_cleanup_enabled);
        assert!(service.record.pid.is_some());
        assert!(service.record.cleanup_error.is_some());

        child.post_reap_cleanup_error = None;
        child.drop_cleanup_enabled = true;
        child
            .finalize("test service cleanup")
            .expect("clean test service");
    }

    #[cfg(unix)]
    #[test]
    fn readiness_command_is_bounded_and_observes_cancellation() {
        let command = vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            "trap '' INT TERM; while :; do sleep 1; done".to_string(),
        ];
        let no_signal = RunCancellation::new().expect("install scoped handlers");
        let started = Instant::now();
        assert!(
            !readiness_command_with_timeout(&command, &no_signal, Duration::from_millis(40),)
                .expect("bounded readiness timeout")
        );
        assert!(started.elapsed() < Duration::from_secs(2));

        let interrupted = RunCancellation::with_observed_signal(libc::SIGTERM);
        let error = readiness_command_with_timeout(&command, &interrupted, Duration::from_secs(5))
            .expect_err("readiness command must observe cancellation");
        assert_eq!(
            run_interruption(&error).expect("typed interruption").signal,
            libc::SIGTERM
        );
    }

    #[test]
    fn prepared_cluster_cleanup_attempts_every_record_after_one_failure() {
        let records = vec![
            ClusterRunRecord {
                backend: "kind".to_string(),
                name: "first".to_string(),
                kubeconfig: "first.kubeconfig".to_string(),
                node_count: 1,
                ready_ms: None,
            },
            ClusterRunRecord {
                backend: "k3d".to_string(),
                name: "second".to_string(),
                kubeconfig: "second.kubeconfig".to_string(),
                node_count: 1,
                ready_ms: None,
            },
        ];
        let mut attempted = Vec::new();
        let failures = cleanup_cluster_records_with(&records, |_, name| {
            attempted.push(name.to_string());
            if name == "first" {
                bail!("injected delete failure");
            }
            Ok(())
        });
        assert_eq!(attempted, vec!["first", "second"]);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].detail().contains("first"));
    }

    #[test]
    fn started_cluster_delete_error_becomes_durable_service_obligation() {
        let temp = tempfile::tempdir().expect("tempdir");
        let order_path = temp.path().join("stop-order.txt");
        let mut service = spawn_trapping_service(temp.path(), &order_path, "cluster");
        let cluster = ClusterRunRecord {
            backend: "injected-unknown".to_string(),
            name: "owned-cluster".to_string(),
            kubeconfig: "owned.kubeconfig".to_string(),
            node_count: 1,
            ready_ms: None,
        };
        service.cluster = Some(cluster.clone());
        service.record.cluster = Some(cluster);
        std::thread::sleep(Duration::from_millis(100));

        let error = stop_services(std::slice::from_mut(&mut service), true)
            .expect_err("unknown backend must fail closed");
        assert!(error.to_string().contains("cluster cleanup"));
        assert_eq!(service.record.status, ProcessStatus::Failed);
        assert!(service
            .record
            .cleanup_error
            .as_deref()
            .is_some_and(|error| error.contains("unknown cluster backend")));
    }

    #[test]
    fn later_resource_delete_success_cannot_clear_prior_cli_group_obligation() {
        let temp = tempfile::tempdir().expect("tempdir");
        let order_path = temp.path().join("stop-order.txt");
        let mut service = spawn_trapping_service(temp.path(), &order_path, "cluster-retry");
        let record = ClusterRunRecord {
            backend: "kind".to_string(),
            name: "owned-cluster".to_string(),
            kubeconfig: "owned.kubeconfig".to_string(),
            node_count: 1,
            ready_ms: None,
        };
        service.cluster = Some(record.clone());
        service.record.cluster = Some(record.clone());
        let first_error = cluster::injected_owned_command_cleanup_failure(
            "delete command PGID 4242 remains after TERM/KILL",
        );
        service.record.cleanup_error = Some(cluster_cleanup_detail(&record, &first_error));

        record_cluster_delete_success(&mut service, &record);

        assert!(
            service.cluster.is_some(),
            "ownership handle must be retained"
        );
        assert_eq!(service.record.status, ProcessStatus::Failed);
        assert!(service
            .record
            .cleanup_error
            .as_deref()
            .is_some_and(|error| error.contains("cli_group_unconfirmed=true")));
    }

    #[test]
    fn confirmed_cluster_resource_cleanup_preserves_unrelated_obligations() {
        let temp = tempfile::tempdir().expect("tempdir");
        let order_path = temp.path().join("stop-order.txt");
        let mut service = spawn_trapping_service(temp.path(), &order_path, "cluster-confirmed");
        let record = ClusterRunRecord {
            backend: "kind".to_string(),
            name: "owned-cluster".to_string(),
            kubeconfig: "owned.kubeconfig".to_string(),
            node_count: 1,
            ready_ms: None,
        };
        service.cluster = Some(record.clone());
        let resource_error = anyhow::anyhow!("backend resource delete timed out");
        service.record.cleanup_error = Some(format!(
            "readiness cleanup unconfirmed; {}",
            cluster_cleanup_detail(&record, &resource_error)
        ));

        record_cluster_delete_success(&mut service, &record);

        assert!(service.cluster.is_none());
        assert_eq!(
            service.record.cleanup_error.as_deref(),
            Some("readiness cleanup unconfirmed")
        );
    }

    #[cfg(unix)]
    #[test]
    fn post_reap_cleanup_failure_never_resignals_a_numeric_pgid() {
        let temp = tempfile::tempdir().expect("tempdir");
        let marker = temp.path().join("unexpected-term.txt");
        let command = vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            "trap 'printf term > \"$VAT_TERM_MARKER\"; exit 0' TERM; while :; do sleep 1; done"
                .to_string(),
        ];
        let env = BTreeMap::from([(
            "VAT_TERM_MARKER".to_string(),
            marker.to_string_lossy().into_owned(),
        )]);
        let mut unrelated = command_with_logs(
            &command,
            temp.path(),
            &env,
            &temp.path().join("unrelated.stdout"),
            &temp.path().join("unrelated.stderr"),
        )
        .expect("spawn unrelated process group");
        std::thread::sleep(Duration::from_millis(100));

        let mut exited_command = Command::new("/usr/bin/true");
        set_process_group(&mut exited_command);
        let mut reaped_child = exited_command.spawn().expect("spawn reaped leader");
        let status = reaped_child.wait().expect("reap leader");
        let mut stale = OwnedProcessGroup {
            child: reaped_child,
            // Model the numeric-reuse hazard explicitly: if repeated cleanup
            // signalled after reap, it would touch this unrelated live group.
            pgid: unrelated.id(),
            final_status: Some(status),
            finalized: false,
            post_reap_cleanup_error: Some("synthetic post-reap PGID-absence failure".to_string()),
            drop_cleanup_enabled: true,
        };

        let injected = anyhow::anyhow!("injected direct cleanup failure");
        let (owned_pgid, cleanup_error) = preserve_direct_cleanup_failure(&mut stale, &injected)
            .expect("unconfirmed direct cleanup evidence");
        assert_eq!(
            owned_pgid, None,
            "reaped leader must not persist a stale PGID"
        );
        assert!(cleanup_error.contains("injected direct cleanup failure"));
        assert!(!stale.drop_cleanup_enabled);

        assert!(stale.finalize("synthetic reaped group").is_err());
        assert!(stale.finalize("synthetic reaped group").is_err());
        std::thread::sleep(Duration::from_millis(100));
        assert!(
            !marker.exists(),
            "repeated finalization signalled reused PGID"
        );
        assert!(unrelated
            .finished_status("unrelated process group")
            .expect("observe unrelated process")
            .is_none());
        unrelated
            .finalize("unrelated process group")
            .expect("cleanup unrelated fixture");
    }

    #[test]
    fn ordered_required_services_expands_dependencies_first() {
        let cfg = VatConfig {
            version: 1,
            network: None,
            name: None,
            default_runner: None,
            workspace: crate::config::WorkspaceConfig::default(),
            env: BTreeMap::new(),
            setup: Vec::new(),
            services: vec![
                test_service("frontend", &["backend"]),
                test_service("backend", &["postgres"]),
                test_service("postgres", &[]),
            ],
            runners: vec![RunnerConfig {
                id: "e2e".to_string(),
                requires: vec!["frontend".to_string()],
                cmd: vec!["true".to_string()],
                timeout_s: None,
                artifacts: Vec::new(),
            }],
            scenarios: Vec::new(),
            path: PathBuf::from("vat.toml"),
            root: PathBuf::from("."),
            digest: String::new(),
        };

        let ids: Vec<&str> = cfg.runners[0].requires.iter().map(|s| s.as_str()).collect();
        let ordered = ordered_required_services(&cfg, &ids).expect("service order");

        assert_eq!(
            ordered
                .iter()
                .map(|service| service.id.as_str())
                .collect::<Vec<_>>(),
            vec!["postgres", "backend", "frontend"]
        );
    }

    #[test]
    fn scenario_service_ids_expand_dependencies_before_hermetic_check() {
        let mut http = test_service("http", &[]);
        http.preset = Some(ServicePreset::HttpMock);
        let cfg = VatConfig {
            version: 1,
            network: None,
            name: None,
            default_runner: None,
            workspace: crate::config::WorkspaceConfig::default(),
            env: BTreeMap::new(),
            setup: Vec::new(),
            services: vec![
                test_service("api", &["worker"]),
                test_service("worker", &["http"]),
                http,
            ],
            runners: vec![RunnerConfig {
                id: "e2e".to_string(),
                requires: Vec::new(),
                cmd: vec!["true".to_string()],
                timeout_s: None,
                artifacts: Vec::new(),
            }],
            scenarios: Vec::new(),
            path: PathBuf::from("vat.toml"),
            root: PathBuf::from("."),
            digest: String::new(),
        };
        let scenario = ScenarioConfig {
            id: "prod-like".to_string(),
            app: "api".to_string(),
            requires: Vec::new(),
            runner: "e2e".to_string(),
            network: ScenarioNetworkMode::Hermetic,
        };

        let ids = scenario_service_ids(&cfg, &scenario, &cfg.runners[0]).expect("scenario ids");

        assert_eq!(ids, vec!["http", "worker", "api"]);
        assert!(service_set_has_http_mock(&cfg, &ids));
    }

    struct TestSandbox;

    impl crate::sandbox::Sandbox for TestSandbox {
        fn name(&self) -> &'static str {
            "test"
        }

        fn resolve(&self, rootfs: &Path, program: &str, args: &[String]) -> (String, Vec<String>) {
            (
                "sandboxed".to_string(),
                std::iter::once(rootfs.display().to_string())
                    .chain(std::iter::once(program.to_string()))
                    .chain(args.iter().cloned())
                    .collect(),
            )
        }
    }

    #[test]
    fn direct_start_service_command_uses_supplied_sandbox_only_for_direct_services() {
        let mut plan = ServicePlan {
            id: "api".to_string(),
            command: vec!["python3".to_string(), "-m".to_string(), "app".to_string()],
            host: None,
            ready_http: None,
            ready_probe: ReadyProbe::None,
            timeout_s: 1,
            preset: None,
            port: None,
            prepare_mode: "direct_start".to_string(),
            cache_key: None,
            prepare_duration_ms: 0,
            env: BTreeMap::new(),
            exported_env: Vec::new(),
            docker_name: None,
            microvm_name: None,
            image: None,
            cluster: None,
            owned_by_vat: true,
            requires_live_child: true,
            endpoint_reservations: Vec::new(),
        };

        let wrapped = service_start_command(&plan, Some(&TestSandbox), Path::new("/vat/root"));

        assert_eq!(
            wrapped,
            vec!["sandboxed", "/vat/root", "python3", "-m", "app"]
        );

        plan.prepare_mode = "builtin_emulator".to_string();
        assert_eq!(
            service_start_command(&plan, Some(&TestSandbox), Path::new("/vat/root")),
            plan.command
        );
    }

    #[test]
    fn native_auto_endpoint_reservations_are_unique_and_release_deterministically() {
        let first = reserve_native_service_port("first", &PortSpec::default())
            .expect("reserve first native endpoint");
        let second = reserve_native_service_port("second", &PortSpec::default())
            .expect("reserve second native endpoint");
        let first_endpoint = first.endpoint;
        let second_endpoint = second.endpoint;

        assert_ne!(first_endpoint, second_endpoint);
        assert!(TcpListener::bind(first_endpoint).is_err());
        assert!(TcpListener::bind(second_endpoint).is_err());

        drop(first);
        let rebound = TcpListener::bind(first_endpoint)
            .expect("dropping a failed/finished plan must release its reservation");
        drop(rebound);
        assert!(TcpListener::bind(second_endpoint).is_err());
    }

    #[test]
    fn native_fixed_endpoint_rejects_an_existing_listener_without_touching_it() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind unrelated listener");
        let endpoint = listener.local_addr().expect("unrelated endpoint");

        let err = reserve_native_service_port("api", &PortSpec::Fixed(endpoint.port()))
            .expect_err("owned native service must not attach to an occupied endpoint");

        assert!(err.to_string().contains("native_service_endpoint_conflict"));
        assert!(err.to_string().contains(&endpoint.to_string()));
        assert!(TcpStream::connect(endpoint).is_ok());
    }

    #[test]
    fn native_command_rejects_non_literal_loopback_readiness_hosts() {
        let mut service = test_service("api", &["server", "--port", "{port}"]);
        service.ready_http = Some("http://localhost:{port}/ready".to_string());

        let err = command_service_port(&service)
            .expect_err("localhost cannot be reserved as one exact IPv4 endpoint");

        assert!(err
            .to_string()
            .contains("native_service_loopback_unsupported"));
        assert!(err.to_string().contains("localhost"));
    }

    /// UT3 (#1301 R2/AC2): the real (non-hermetic-proxy) service call path
    /// never threads a `Some(sandbox)` backend into `service_start_command` —
    /// vat's own spawned services (emulators, http-mock/record-replay proxy)
    /// stay unsandboxed by construction, not by a permissive default. The
    /// hermetic-proxy mode is the one deliberate exception, asserted here
    /// too so the boundary of the exemption is explicit.
    #[test]
    fn start_service_call_site_never_sandboxes_real_services() {
        assert!(
            service_sandbox_backend(false, &TestSandbox).is_none(),
            "real (non-hermetic-proxy) services must never be sandbox-wrapped"
        );
        assert!(
            service_sandbox_backend(true, &TestSandbox).is_some(),
            "hermetic-proxy mode is the sole intentional exception"
        );
    }

    #[test]
    fn ready_cmd_overrides_http_and_preset_default() {
        let mut service = test_service("pg", &[]);
        service.preset = Some(ServicePreset::Postgres);
        service.ready_http = Some("http://127.0.0.1:7373/".to_string());
        service.ready_cmd = vec!["sh".to_string(), "-c".to_string(), "exit 0".to_string()];
        let preset_default = preset_ready_probe(ServicePreset::Postgres, 5432);
        match resolve_ready_probe(&service, Some(preset_default)) {
            ReadyProbe::Cmd(cmd) => assert_eq!(cmd, service.ready_cmd),
            other => panic!("expected ready_cmd to win, got {other:?}"),
        }
    }

    #[test]
    fn ready_http_overrides_preset_default_when_no_ready_cmd() {
        let mut service = test_service("pg", &[]);
        service.preset = Some(ServicePreset::Postgres);
        service.ready_http = Some("http://127.0.0.1:9200/".to_string());
        let preset_default = preset_ready_probe(ServicePreset::Postgres, 5432);
        match resolve_ready_probe(&service, Some(preset_default)) {
            ReadyProbe::Http(url) => assert_eq!(url, "http://127.0.0.1:9200/"),
            other => panic!("expected ready_http, got {other:?}"),
        }
    }

    #[test]
    fn preset_default_applies_when_no_override() {
        let service = test_service("pg", &[]);
        match resolve_ready_probe(
            &service,
            Some(preset_ready_probe(ServicePreset::Postgres, 5432)),
        ) {
            ReadyProbe::Cmd(cmd) => assert_eq!(cmd[0], "pg_isready"),
            other => panic!("expected pg_isready probe, got {other:?}"),
        }
        // No preset default and no override => no probe.
        assert!(matches!(
            resolve_ready_probe(&service, None),
            ReadyProbe::None
        ));
    }

    #[test]
    fn prepare_external_service_attaches_endpoint_without_child_command() {
        let mut service = test_service("pg-ci", &[]);
        service.cmd.clear();
        service.external = Some(ExternalServiceConfig {
            host: "postgres".to_string(),
            port: 5432,
        });
        service.export.insert(
            "DATABASE_URL".to_string(),
            "postgres://postgres@{host}:{port}/app".to_string(),
        );

        let plan = prepare_external_service(&service).expect("external plan");

        assert!(!plan.owned_by_vat);
        assert!(plan.command.is_empty());
        assert_eq!(plan.host.as_deref(), Some("postgres"));
        assert_eq!(plan.port, Some(5432));
        assert_eq!(plan.prepare_mode, "external_attach");
        assert_eq!(
            plan.env.get("DATABASE_URL").map(String::as_str),
            Some("postgres://postgres@postgres:5432/app")
        );
        assert_eq!(
            plan.env.get("VAT_SERVICE_PG_CI_HOST").map(String::as_str),
            Some("postgres")
        );
        assert_eq!(
            plan.env.get("VAT_SERVICE_PG_CI_PORT").map(String::as_str),
            Some("5432")
        );
        assert_eq!(
            plan.exported_env,
            vec![
                "DATABASE_URL".to_string(),
                "VAT_SERVICE_PG_CI_HOST".to_string(),
                "VAT_SERVICE_PG_CI_PORT".to_string()
            ]
        );
        match plan.ready_probe {
            ReadyProbe::Tcp { host, port } => {
                assert_eq!(host, "postgres");
                assert_eq!(port, 5432);
            }
            other => panic!("expected default TCP probe, got {other:?}"),
        }
    }

    #[test]
    fn prepare_external_service_substitutes_readiness_templates() {
        let mut service = test_service("api-ci", &[]);
        service.cmd.clear();
        service.external = Some(ExternalServiceConfig {
            host: "api".to_string(),
            port: 8080,
        });
        service.ready_http = Some("http://{host}:{port}/ready".to_string());
        service.ready_cmd = vec!["probe".to_string(), "{host}:{port}".to_string()];

        let plan = prepare_external_service(&service).expect("external plan");

        assert_eq!(plan.ready_http.as_deref(), Some("http://api:8080/ready"));
        match plan.ready_probe {
            ReadyProbe::Cmd(cmd) => {
                assert_eq!(cmd, vec!["probe".to_string(), "api:8080".to_string()]);
            }
            other => panic!("expected substituted ready_cmd, got {other:?}"),
        }
    }

    #[test]
    fn opensearch_preset_command_and_exports() {
        let data_dir = PathBuf::from("/tmp/vat-os/data");
        let command = preset_command(ServicePreset::Opensearch, 9250, &data_dir);
        assert_eq!(command[0], "opensearch");
        assert!(command.iter().any(|a| a == "-Ehttp.port=9250"));
        assert!(command.iter().any(|a| a == "-Ediscovery.type=single-node"));
        assert!(command
            .iter()
            .any(|a| a.starts_with("-Epath.data=") && a.contains("data")));

        match preset_ready_probe(ServicePreset::Opensearch, 9250) {
            ReadyProbe::Http(url) => assert_eq!(url, "http://127.0.0.1:9250/"),
            other => panic!("expected http ready probe, got {other:?}"),
        }

        let service = {
            let mut s = test_service("search", &[]);
            s.preset = Some(ServicePreset::Opensearch);
            s
        };
        let exports = preset_exports(&service, ServicePreset::Opensearch, 9250);
        assert_eq!(
            exports.get("OPENSEARCH_URL").map(String::as_str),
            Some("http://127.0.0.1:9250")
        );

        let mut env = exports;
        add_service_runtime_env(
            &mut env,
            ServicePreset::Opensearch,
            "search",
            9250,
            &data_dir,
        );
        assert!(env
            .get("OPENSEARCH_PATH_CONF")
            .map(|p| p.ends_with("config"))
            .unwrap_or(false));
    }

    #[test]
    fn opensearch_uses_service_image_cache() {
        assert!(preset_uses_service_image(ServicePreset::Opensearch));
    }

    #[test]
    fn opensearch_cold_prepare_writes_dev_config() {
        let temp = tempfile::tempdir().expect("tempdir");
        let cache = temp.path().join("image");
        std::fs::create_dir_all(&cache).unwrap();
        let service = {
            let mut s = test_service("search", &[]);
            s.preset = Some(ServicePreset::Opensearch);
            s
        };
        cold_prepare_opensearch_image(&service, &cache).expect("prepare opensearch image");
        let yml = std::fs::read_to_string(cache.join("config").join("opensearch.yml"))
            .expect("opensearch.yml written");
        assert!(yml.contains("cluster.name: vat-opensearch"));
        // The Homebrew no-jdk build has no security plugin, so this setting is
        // UNKNOWN and would make OpenSearch refuse to boot — it must be absent.
        assert!(!yml.contains("plugins.security.disabled"));
        assert!(cache.join("data").is_dir());
        assert!(cache.join("logs").is_dir());
    }

    /// End-to-end pg corpus seeding. Skips gracefully when the postgres
    /// toolchain is not installed (vat's standard skip pattern).
    #[test]
    fn postgres_cold_seed_applies_sql_corpus() {
        for binary in ["initdb", "postgres", "pg_ctl", "psql"] {
            if which(binary).is_none() {
                eprintln!("skipping postgres_cold_seed_applies_sql_corpus: `{binary}` not on PATH");
                return;
            }
        }
        let temp = tempfile::tempdir().expect("tempdir");
        let seed = temp.path().join("schema.sql");
        std::fs::write(
            &seed,
            "CREATE TABLE docs (id int primary key);\nINSERT INTO docs VALUES (1),(2),(3);\n",
        )
        .unwrap();

        let mut service = test_service("pg", &[]);
        service.preset = Some(ServicePreset::Postgres);
        service.seed = vec![PathBuf::from("schema.sql")];
        let cfg = VatConfig {
            version: 1,
            network: None,
            name: None,
            default_runner: None,
            workspace: crate::config::WorkspaceConfig::default(),
            env: BTreeMap::new(),
            setup: Vec::new(),
            services: vec![service.clone()],
            runners: vec![RunnerConfig {
                id: "ec".to_string(),
                requires: vec!["pg".to_string()],
                cmd: vec!["true".to_string()],
                timeout_s: None,
                artifacts: Vec::new(),
            }],
            scenarios: Vec::new(),
            path: temp.path().join("vat.toml"),
            root: temp.path().to_path_buf(),
            digest: String::new(),
        };

        let cache = temp.path().join("image");
        std::fs::create_dir_all(&cache).unwrap();
        // Full cold-prepare path: initdb + seed apply + clean shutdown.
        cold_prepare_service_image(
            &cfg,
            &service,
            ServicePreset::Postgres,
            &cache,
            &test_cancellation(),
        )
        .expect("cold prepare + seed postgres");

        // The throwaway seed socket dir must not be baked into the cached image.
        assert!(!cache.join("seed-sock").exists());
        // A real cluster directory was produced.
        assert!(cache.join("PG_VERSION").is_file());

        // Re-open the cached cluster and verify the corpus survived caching.
        let sock = temp.path().join("verify-sock");
        std::fs::create_dir_all(&sock).unwrap();
        let opt = format!("-h '' -k {} -p 5432", sock.display());
        let start = Command::new("pg_ctl")
            .arg("-D")
            .arg(&cache)
            .args(["-w", "-t", "60", "-o"])
            .arg(&opt)
            .arg("start")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("start cached postgres");
        assert!(start.success(), "cached postgres should start");
        let out = Command::new("psql")
            .args(["-tAq", "-h"])
            .arg(&sock)
            .args([
                "-p",
                "5432",
                "-U",
                "postgres",
                "-d",
                "postgres",
                "-c",
                "select count(*) from docs",
            ])
            .output()
            .expect("query cached corpus");
        let _ = Command::new("pg_ctl")
            .arg("-D")
            .arg(&cache)
            .args(["-w", "-t", "60", "-m", "fast", "stop"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let count = String::from_utf8_lossy(&out.stdout);
        assert_eq!(
            count.trim(),
            "3",
            "seeded corpus rows must persist in cache"
        );
    }

    fn test_service(id: &str, requires: &[&str]) -> ServiceConfig {
        ServiceConfig {
            id: id.to_string(),
            requires: requires.iter().map(|value| value.to_string()).collect(),
            cmd: vec!["true".to_string()],
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
            timeout_s: 60,
            volumes: Vec::new(),
        }
    }

    fn image_service(id: &str, image: &str, container_port: u16) -> ServiceConfig {
        ServiceConfig {
            id: id.to_string(),
            requires: Vec::new(),
            cmd: Vec::new(),
            preset: None,
            image: Some(image.to_string()),
            container_port: Some(container_port),
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
            timeout_s: 60,
            volumes: Vec::new(),
        }
    }

    #[test]
    fn docker_create_and_start_commands_are_well_formed_and_deterministic() {
        let mut env = BTreeMap::new();
        env.insert("POSTGRES_HOST_AUTH_METHOD".to_string(), "trust".to_string());
        env.insert("POSTGRES_DB".to_string(), "app".to_string());
        let cmd = docker_create_command("vat-abc-pg", "postgres:16", 54321, 5432, &env);
        assert_eq!(
            cmd,
            vec![
                "docker",
                "create",
                "--rm",
                "--name",
                "vat-abc-pg",
                "-p",
                "127.0.0.1:54321:5432",
                // BTreeMap iteration is sorted -> deterministic argv.
                "-e",
                "POSTGRES_DB=app",
                "-e",
                "POSTGRES_HOST_AUTH_METHOD=trust",
                "postgres:16",
            ]
        );
        assert_eq!(
            docker_start_command(
                "1111111111111111111111111111111111111111111111111111111111111111"
            ),
            vec![
                "docker",
                "start",
                "--attach",
                "1111111111111111111111111111111111111111111111111111111111111111",
            ]
        );
    }

    #[test]
    fn docker_created_checkpoint_transitions_only_on_exact_running_ack() {
        assert_eq!(
            docker_start_transition(&DockerIdentityObservation::Exact {
                state: "created".to_string(),
            }),
            DockerStartTransition::PendingCreated
        );
        assert_eq!(
            docker_start_transition(&DockerIdentityObservation::Exact {
                state: "running".to_string(),
            }),
            DockerStartTransition::RunningAcknowledged
        );
        assert!(matches!(
            docker_start_transition(&DockerIdentityObservation::Exact {
                state: "exited".to_string(),
            }),
            DockerStartTransition::Failed(reason) if reason.contains("exited")
        ));
        assert!(matches!(
            docker_start_transition(&DockerIdentityObservation::Absent),
            DockerStartTransition::Failed(reason) if reason.contains("disappeared")
        ));
        assert!(matches!(
            docker_start_transition(&DockerIdentityObservation::Replacement {
                actual_id:
                    "2222222222222222222222222222222222222222222222222222222222222222"
                        .to_string(),
            }),
            DockerStartTransition::Failed(reason) if reason.contains("replacement ID")
        ));
    }

    #[test]
    fn service_persistence_retains_only_unrepresented_failures_or_cleanup_obligations() {
        let record = |status, cleanup_error: Option<&str>| ServiceRunRecord {
            id: "web".to_string(),
            command: Vec::new(),
            status,
            preset: None,
            host: None,
            port: None,
            owned_by_vat: Some(true),
            prepare_mode: None,
            cache_key: None,
            prepare_duration_ms: None,
            ready_duration_ms: None,
            exported_env: Vec::new(),
            pid: None,
            exit_code: None,
            ready_http: None,
            docker_name: None,
            docker_id: None,
            microvm_name: None,
            readiness_error: None,
            cleanup_error: cleanup_error.map(str::to_string),
            cluster: None,
            stdout_log: String::new(),
            stderr_log: String::new(),
        };
        let no_current = BTreeSet::new();
        assert!(retain_unrepresented_service_record(
            &record(ProcessStatus::Failed, None),
            &no_current,
        ));
        assert!(!retain_unrepresented_service_record(
            &record(ProcessStatus::Running, None),
            &no_current,
        ));
        assert!(retain_unrepresented_service_record(
            &record(ProcessStatus::Running, Some("cleanup unconfirmed")),
            &no_current,
        ));

        let current_ids = BTreeSet::from(["web".to_string()]);
        assert!(
            !retain_unrepresented_service_record(
                &record(ProcessStatus::Failed, Some("older obligation")),
                &current_ids,
            ),
            "a current handle with the same ID must overwrite older evidence"
        );
    }

    #[test]
    fn docker_attach_readiness_requires_consecutive_stable_observations() {
        let mut candidate = None;
        let first = Instant::now();
        assert!(!docker_attach_readiness_is_stable(
            &mut candidate,
            first,
            true
        ));
        assert!(!docker_attach_readiness_is_stable(
            &mut candidate,
            first + Duration::from_millis(99),
            true
        ));
        assert!(docker_attach_readiness_is_stable(
            &mut candidate,
            first + Duration::from_millis(100),
            true
        ));

        assert!(!docker_attach_readiness_is_stable(
            &mut candidate,
            first + Duration::from_millis(200),
            false
        ));
        assert_eq!(candidate, None, "a non-ready probe resets the candidate");
        assert!(!docker_attach_readiness_is_stable(
            &mut candidate,
            first + Duration::from_millis(300),
            true
        ));
    }

    #[test]
    fn microvm_ready_probe_substitutes_allocated_http_port() {
        match microvm_ready_probe(Some("http://{host}:{port}/ready"), 43123) {
            ReadyProbe::MicroVmHttp(url) => {
                assert_eq!(url, "http://127.0.0.1:43123/ready");
            }
            other => panic!("expected MicroVmHttp probe, got {other:?}"),
        }
    }

    #[test]
    fn microvm_tcp_probe_rejects_immediate_close_after_handshake() {
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listener,
            Err(err) if err.kind() == ErrorKind::PermissionDenied => {
                eprintln!("skip: loopback sockets are unavailable ({err})");
                return;
            }
            Err(err) => panic!("bind listener: {err}"),
        };
        let port = listener.local_addr().expect("listener address").port();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept client");
            let _ = stream.shutdown(std::net::Shutdown::Both);
        });

        let err = tcp_usable_readiness("127.0.0.1", port).expect_err("EOF must not be ready");
        server.join().expect("server thread");
        let message = err.to_string();
        assert!(
            message.contains("closed immediately") || message.contains("reset immediately"),
            "unexpected endpoint error: {message}"
        );
    }

    #[test]
    fn microvm_http_probe_retains_non_ready_response() {
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listener,
            Err(err) if err.kind() == ErrorKind::PermissionDenied => {
                eprintln!("skip: loopback sockets are unavailable ({err})");
                return;
            }
            Err(err) => panic!("bind listener: {err}"),
        };
        let port = listener.local_addr().expect("listener address").port();
        let server = std::thread::spawn(move || -> Result<(), String> {
            listener
                .set_nonblocking(true)
                .map_err(|err| err.to_string())?;
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream
                            .write_all(
                                b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n",
                            )
                            .map_err(|err| err.to_string())?;
                        return Ok(());
                    }
                    Err(err) if err.kind() == ErrorKind::WouldBlock => {
                        if Instant::now() >= deadline {
                            return Err("HTTP readiness client did not connect".to_string());
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => return Err(err.to_string()),
                }
            }
        });

        let observation = http_readiness(&format!("http://127.0.0.1:{port}/ready"))
            .expect("HTTP readiness observation");
        server
            .join()
            .expect("HTTP readiness server thread")
            .expect("HTTP readiness server");
        assert!(matches!(
            observation,
            EndpointReadiness::Pending(reason) if reason.contains("HTTP/1.1 503")
        ));
    }

    #[test]
    fn preset_image_uses_version_tag_when_present() {
        assert_eq!(preset_image(ServicePreset::Postgres, None), "postgres:16");
        assert_eq!(
            preset_image(ServicePreset::Postgres, Some("15")),
            "postgres:15"
        );
        assert_eq!(preset_image(ServicePreset::Redis, None), "redis:7");
        assert_eq!(
            preset_image(ServicePreset::Opensearch, None),
            "opensearchproject/opensearch:2"
        );
        assert_eq!(
            preset_image(ServicePreset::Opensearch, Some("2.15.0")),
            "opensearchproject/opensearch:2.15.0"
        );
    }

    #[test]
    fn opensearch_docker_defaults_are_passwordless_single_node() {
        assert_eq!(preset_container_port(ServicePreset::Opensearch), 9200);

        let env = preset_container_env(ServicePreset::Opensearch);

        assert_eq!(
            env.get("discovery.type").map(String::as_str),
            Some("single-node")
        );
        assert_eq!(
            env.get("plugins.security.disabled").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            env.get("OPENSEARCH_JAVA_OPTS").map(String::as_str),
            Some("-Xms512m -Xmx512m")
        );
    }

    #[test]
    fn emulator_image_defaults() {
        assert_eq!(
            preset_image(ServicePreset::Firestore, None),
            "gcr.io/google.com/cloudsdktool/google-cloud-cli:emulators"
        );
        assert_eq!(
            preset_image(ServicePreset::Spanner, None),
            "gcr.io/cloud-spanner-emulator/emulator:latest"
        );
    }

    #[test]
    fn native_available_requires_gcloud_component() {
        // Binary present but the gcloud component is not installed → not native.
        assert!(!native_available(true, Some("pubsub-emulator"), &[]));
        // Component installed → native.
        assert!(native_available(
            true,
            Some("pubsub-emulator"),
            &["pubsub-emulator".to_string()]
        ));
        // No component gate (datastore/broker presets) → binary presence wins.
        assert!(native_available(true, None, &[]));
        assert!(!native_available(false, None, &[]));
    }

    #[test]
    fn emulator_exports_well_known_host_var() {
        let svc = test_service("db", &[]);
        let env = preset_exports(&svc, ServicePreset::Firestore, 8080);
        assert_eq!(
            env.get("FIRESTORE_EMULATOR_HOST").map(String::as_str),
            Some("127.0.0.1:8080")
        );
        let env = preset_exports(&svc, ServicePreset::Pubsub, 8085);
        assert_eq!(
            env.get("PUBSUB_EMULATOR_HOST").map(String::as_str),
            Some("127.0.0.1:8085")
        );
    }

    #[test]
    fn cloud_storage_builtin_export_includes_http_scheme() {
        let svc = test_service("gcs", &[]);
        let plan = prepare_builtin_service(
            &svc,
            ServicePreset::CloudStorage,
            Path::new("."),
            &[],
            false,
        )
        .unwrap();
        let port = plan.port.unwrap();
        let expected_host = format!("http://127.0.0.1:{port}");
        let expected_port = port.to_string();

        assert_eq!(
            plan.env.get("STORAGE_EMULATOR_HOST").map(String::as_str),
            Some(expected_host.as_str())
        );
        assert_eq!(
            plan.env.get("VAT_SERVICE_GCS_HOST").map(String::as_str),
            Some("127.0.0.1")
        );
        assert_eq!(
            plan.env.get("VAT_SERVICE_GCS_PORT").map(String::as_str),
            Some(expected_port.as_str())
        );
    }

    #[test]
    fn preset_exports_substitute_template_with_declared_env_key() {
        let mut svc = test_service("mongo", &[]);
        svc.export.insert(
            "MONGODB_URL".to_string(),
            "mongodb://{host}:{port}/tech-platform-e2e".to_string(),
        );
        let env = preset_exports(&svc, ServicePreset::Mongo, 60736);
        assert_eq!(
            env.get("MONGODB_URL").map(String::as_str),
            Some("mongodb://127.0.0.1:60736/tech-platform-e2e")
        );
        assert_eq!(
            env.get("VAT_SERVICE_MONGO_HOST").map(String::as_str),
            Some("127.0.0.1")
        );
        assert_eq!(
            env.get("VAT_SERVICE_MONGO_PORT").map(String::as_str),
            Some("60736")
        );
        assert!(
            !env.contains_key("mongodb://{host}:{port}/tech-platform-e2e"),
            "template values must not become environment variable names"
        );
    }

    #[test]
    fn preset_exports_keep_legacy_target_name_shorthand() {
        let mut svc = test_service("redis", &[]);
        svc.export
            .insert("ignored".to_string(), "CACHE_URL".to_string());
        let env = preset_exports(&svc, ServicePreset::Redis, 60738);
        assert_eq!(
            env.get("CACHE_URL").map(String::as_str),
            Some("redis://127.0.0.1:60738/")
        );
    }

    #[test]
    fn lumen_preset_is_native_and_exports_loopback_ready_endpoint() {
        let mut svc = test_service("lumen", &[]);
        svc.runtime = ServiceRuntime::Auto;

        assert!(matches!(
            resolve_preset_runtime(&svc, ServicePreset::Lumen, &test_cancellation()).unwrap(),
            ResolvedRuntime::Native
        ));
        assert!(matches!(
            preset_ready_probe(ServicePreset::Lumen, 7373),
            ReadyProbe::Http(url) if url == "http://127.0.0.1:7373/readyz"
        ));

        let env = preset_exports(&svc, ServicePreset::Lumen, 7373);
        assert_eq!(
            env.get("LUMEN_URL").map(String::as_str),
            Some("http://127.0.0.1:7373")
        );
        assert_eq!(
            env.get("VAT_SERVICE_LUMEN_HOST").map(String::as_str),
            Some("127.0.0.1")
        );
        assert_eq!(
            env.get("VAT_SERVICE_LUMEN_PORT").map(String::as_str),
            Some("7373")
        );
    }

    #[test]
    fn lumen_preset_rejects_docker_runtime_without_a_fallback() {
        let mut svc = test_service("lumen", &[]);
        svc.runtime = ServiceRuntime::Docker;
        match resolve_preset_runtime(&svc, ServicePreset::Lumen, &test_cancellation()) {
            Err(error) => assert!(error.to_string().contains("native-only")),
            Ok(_) => panic!("lumen must not fall back to Docker"),
        }
    }

    #[test]
    fn builtin_exports_support_templates_and_raw_endpoint_vars() {
        let mut svc = test_service("tasks", &[]);
        svc.export.insert(
            "TASKS_URL".to_string(),
            "http://{host}:{port}/v2/projects/demo".to_string(),
        );
        let plan =
            prepare_builtin_service(&svc, ServicePreset::CloudTasks, Path::new("."), &[], false)
                .unwrap();

        let port = plan.port.unwrap();
        let expected_url = format!("http://127.0.0.1:{port}/v2/projects/demo");
        let expected_port = port.to_string();
        assert_eq!(
            plan.env.get("TASKS_URL").map(String::as_str),
            Some(expected_url.as_str())
        );
        assert_eq!(
            plan.env.get("VAT_SERVICE_TASKS_HOST").map(String::as_str),
            Some("127.0.0.1")
        );
        assert_eq!(
            plan.env.get("VAT_SERVICE_TASKS_PORT").map(String::as_str),
            Some(expected_port.as_str())
        );
    }

    #[test]
    fn service_stderr_hints_detect_macos_somaxconn_warning() {
        let stderr = "WARNING: The TCP backlog setting of 511 cannot be enforced because kern.ipc.somaxconn is set to the lower value of 128.";
        let hints = service_log_hints("redis", stderr);

        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0]["type"], "hint");
        assert_eq!(hints[0]["code"], "macos_tcp_backlog_limited");
        assert_eq!(hints[0]["service"], "redis");
    }

    #[test]
    fn emulator_docker_command_appends_start_for_cloud_cli() {
        let cmd = preset_docker_command(ServicePreset::Firestore, 8080);
        assert_eq!(
            cmd,
            vec![
                "gcloud",
                "beta",
                "emulators",
                "firestore",
                "start",
                "--host-port=0.0.0.0:8080"
            ]
        );
        // Spanner's dedicated image starts via its own entrypoint.
        assert!(preset_docker_command(ServicePreset::Spanner, 9010).is_empty());
    }

    #[test]
    fn builtin_presets_resolve_to_builtin_under_auto() {
        let svc = test_service("svc", &[]);
        assert!(matches!(
            resolve_preset_runtime(&svc, ServicePreset::Pubsub, &test_cancellation()).unwrap(),
            ResolvedRuntime::Builtin
        ));
        assert!(matches!(
            resolve_preset_runtime(&svc, ServicePreset::FirebaseAuth, &test_cancellation(),)
                .unwrap(),
            ResolvedRuntime::Builtin
        ));
    }

    #[test]
    fn preset_auto_routes_maps_gcp_hosts_to_local_endpoints() {
        let routes = preset_auto_routes(&[
            (Some(ServicePreset::CloudTasks), Some(8085)),
            (Some(ServicePreset::CloudScheduler), Some(8086)),
            (Some(ServicePreset::HttpMock), Some(9000)), // proxy itself → no route
            (Some(ServicePreset::Postgres), Some(5432)), // not a GCP host → skipped
            (Some(ServicePreset::Pubsub), None),         // unresolved port → skipped
        ]);
        assert_eq!(
            routes,
            vec![
                (
                    "cloudtasks.googleapis.com".to_string(),
                    "http://127.0.0.1:8085".to_string()
                ),
                (
                    "cloudscheduler.googleapis.com".to_string(),
                    "http://127.0.0.1:8086".to_string()
                ),
            ]
        );
    }

    #[test]
    fn prepare_builtin_service_exports_host_and_self_command() {
        let svc = test_service("auth", &[]);
        let plan = prepare_builtin_service(
            &svc,
            ServicePreset::FirebaseAuth,
            Path::new("."),
            &[],
            false,
        )
        .unwrap();
        assert_eq!(plan.prepare_mode, "builtin_emulator");
        assert!(plan
            .exported_env
            .iter()
            .any(|k| k == "FIREBASE_AUTH_EMULATOR_HOST"));
        assert_eq!(plan.command[1], "emulator");
        assert_eq!(plan.command[2], "firebase-auth");
        assert_eq!(plan.command[3], "--host-port");

        let plan = prepare_builtin_service(
            &test_service("ps", &[]),
            ServicePreset::Pubsub,
            Path::new("."),
            &[],
            false,
        )
        .unwrap();
        assert!(plan
            .exported_env
            .iter()
            .any(|k| k == "PUBSUB_EMULATOR_HOST"));
        assert_eq!(plan.command[2], "pubsub");
    }

    #[test]
    fn http_mock_env_exports_proxy_and_ca_trust() {
        let env = http_mock_env("127.0.0.1:9", "/tmp/ca.pem");
        assert_eq!(env.get("HTTP_PROXY").unwrap(), "http://127.0.0.1:9");
        assert_eq!(env.get("HTTPS_PROXY").unwrap(), "http://127.0.0.1:9");
        // Other loopback emulators stay direct.
        assert_eq!(env.get("NO_PROXY").unwrap(), "localhost,127.0.0.1");
        // CA trust for the common runtimes points at the minted CA.
        for k in [
            "SSL_CERT_FILE",
            "CURL_CA_BUNDLE",
            "REQUESTS_CA_BUNDLE",
            "NODE_EXTRA_CA_CERTS",
        ] {
            assert_eq!(env.get(k).unwrap(), "/tmp/ca.pem");
        }
        assert_eq!(env.get("VAT_HTTP_MOCK_HOST").unwrap(), "127.0.0.1:9");
    }

    #[test]
    fn cloud_builtin_presets_resolve_and_export() {
        let svc = test_service("svc", &[]);
        for (preset, kind, var) in [
            (
                ServicePreset::CloudTasks,
                "cloud-tasks",
                "CLOUD_TASKS_EMULATOR_HOST",
            ),
            (
                ServicePreset::CloudScheduler,
                "cloud-scheduler",
                "CLOUD_SCHEDULER_EMULATOR_HOST",
            ),
        ] {
            assert!(matches!(
                resolve_preset_runtime(&svc, preset, &test_cancellation()).unwrap(),
                ResolvedRuntime::Builtin
            ));
            let plan = prepare_builtin_service(&svc, preset, Path::new("."), &[], false).unwrap();
            assert_eq!(plan.command[2], kind);
            assert!(plan.exported_env.iter().any(|k| k == var));
        }
    }

    #[test]
    fn forced_runtime_does_not_probe_host() {
        let mut svc = test_service("pg", &[]);
        svc.cmd = Vec::new();
        svc.preset = Some(ServicePreset::Postgres);
        svc.runtime = ServiceRuntime::Native;
        assert!(matches!(
            resolve_preset_runtime(&svc, ServicePreset::Postgres, &test_cancellation()).unwrap(),
            ResolvedRuntime::Native
        ));
        svc.runtime = ServiceRuntime::Docker;
        assert!(matches!(
            resolve_preset_runtime(&svc, ServicePreset::Postgres, &test_cancellation()).unwrap(),
            ResolvedRuntime::Docker
        ));
        svc.runtime = ServiceRuntime::MicroVm;
        assert!(matches!(
            resolve_preset_runtime(&svc, ServicePreset::Postgres, &test_cancellation()).unwrap(),
            ResolvedRuntime::MicroVm
        ));
    }

    #[test]
    fn microvm_preset_runtime_rejects_unmapped_builtin_images() {
        let mut svc = test_service("auth", &[]);
        svc.cmd = Vec::new();
        svc.preset = Some(ServicePreset::FirebaseAuth);
        svc.runtime = ServiceRuntime::MicroVm;
        let error = resolve_preset_runtime(&svc, ServicePreset::FirebaseAuth, &test_cancellation())
            .expect_err("builtin-only preset must not pretend a generic image is equivalent");
        assert!(error
            .to_string()
            .contains("no declared Apple Container OCI image route"));
    }

    #[test]
    fn microvm_preset_runtime_rejects_unproven_named_volume_lifecycle() {
        let mut svc = test_service("cache", &[]);
        svc.cmd = Vec::new();
        svc.preset = Some(ServicePreset::Redis);
        svc.runtime = ServiceRuntime::MicroVm;
        svc.volumes.push(VolumeMount {
            name: "cache-data".to_string(),
            path: "/data".to_string(),
        });
        let error = resolve_preset_runtime(&svc, ServicePreset::Redis, &test_cancellation())
            .expect_err("MicroVM preset volume route must fail until lifetime proof exists");
        assert!(error
            .to_string()
            .contains("preset-volume ownership/cleanup contract"));
    }

    #[test]
    fn container_name_sanitizes_disallowed_chars() {
        assert_eq!(container_name("vat-5oyh3vc", "pg"), "vat-5oyh3vc-pg");
        assert_eq!(container_name("vat/x", "a b"), "vat-x-a-b");
    }

    #[test]
    fn docker_identity_filter_is_anchored_and_regex_escapes_sanitized_dots() {
        assert_eq!(
            docker_exact_name_filter("vat-a.b-c_d").expect("sanitized Docker name"),
            r"name=^/vat-a\.b-c_d$"
        );
        assert!(docker_exact_name_filter("vat/name").is_err());
        assert!(docker_exact_name_filter("").is_err());
    }

    #[test]
    fn docker_cleanup_budget_reserves_every_later_phase() {
        let after_initial_query = DOCKER_KILL_TIMEOUT
            + DOCKER_IDENTITY_QUERY_TIMEOUT
            + DOCKER_TERMINAL_RM_TIMEOUT
            + DOCKER_IDENTITY_QUERY_TIMEOUT;
        let after_kill = DOCKER_IDENTITY_QUERY_TIMEOUT
            + DOCKER_TERMINAL_RM_TIMEOUT
            + DOCKER_IDENTITY_QUERY_TIMEOUT;
        let after_post_kill_query = DOCKER_TERMINAL_RM_TIMEOUT + DOCKER_IDENTITY_QUERY_TIMEOUT;
        let after_remove = DOCKER_IDENTITY_QUERY_TIMEOUT;
        let running_attempt_budget =
            DOCKER_IDENTITY_QUERY_TIMEOUT + DOCKER_KILL_TIMEOUT + after_kill;

        assert_eq!(after_initial_query, Duration::from_secs(10));
        assert_eq!(after_kill, Duration::from_secs(7));
        assert_eq!(after_post_kill_query, Duration::from_secs(5));
        assert_eq!(after_remove, Duration::from_secs(2));
        assert_eq!(running_attempt_budget, Duration::from_secs(12));
        assert!(running_attempt_budget < DOCKER_CLEANUP_HARD_TIMEOUT);
    }

    #[test]
    fn image_exports_substitute_host_and_port_and_add_raw_vars() {
        let mut svc = image_service("alloy-db", "google/alloydbomni:latest", 5432);
        svc.export.insert(
            "DATABASE_URL".to_string(),
            "postgres://postgres:pw@{host}:{port}/db".to_string(),
        );
        let env = image_exports(&svc, 6000);
        assert_eq!(
            env.get("DATABASE_URL").unwrap(),
            "postgres://postgres:pw@127.0.0.1:6000/db"
        );
        assert_eq!(env.get("VAT_SERVICE_ALLOY_DB_HOST").unwrap(), "127.0.0.1");
        assert_eq!(env.get("VAT_SERVICE_ALLOY_DB_PORT").unwrap(), "6000");
    }

    fn spawn_trapping_service(root: &Path, order_path: &Path, id: &str) -> ServiceHandle {
        let ready_path = root.join(format!("{id}.ready"));
        let command = vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            "trap 'printf \"%s\\n\" \"$VAT_STOP_ID\" >> \"$VAT_STOP_ORDER\"; exit 0' TERM; : > \"$VAT_STOP_READY\"; while :; do :; done".to_string(),
        ];
        let mut env = BTreeMap::new();
        env.insert("VAT_STOP_ID".to_string(), id.to_string());
        env.insert(
            "VAT_STOP_ORDER".to_string(),
            order_path.to_string_lossy().into_owned(),
        );
        env.insert(
            "VAT_STOP_READY".to_string(),
            ready_path.to_string_lossy().into_owned(),
        );
        let stdout = root.join(format!("{id}.stdout.log"));
        let stderr = root.join(format!("{id}.stderr.log"));
        let child =
            command_with_logs(&command, root, &env, &stdout, &stderr).expect("service child");
        let ready_deadline = Instant::now() + Duration::from_secs(2);
        while !ready_path.exists() {
            assert!(
                Instant::now() < ready_deadline,
                "service `{id}` did not install its TERM handler"
            );
            std::thread::sleep(OWNED_GROUP_POLL_INTERVAL);
        }
        ServiceHandle {
            record: ServiceRunRecord {
                id: id.to_string(),
                command,
                status: ProcessStatus::Ready,
                preset: None,
                host: None,
                port: None,
                owned_by_vat: Some(true),
                prepare_mode: Some("direct_start".to_string()),
                cache_key: None,
                prepare_duration_ms: Some(0),
                ready_duration_ms: Some(0),
                exported_env: Vec::new(),
                pid: Some(child.id()),
                exit_code: None,
                ready_http: None,
                docker_name: None,
                docker_id: None,
                microvm_name: None,
                readiness_error: None,
                cleanup_error: None,
                cluster: None,
                stdout_log: stdout.to_string_lossy().into_owned(),
                stderr_log: stderr.to_string_lossy().into_owned(),
            },
            child: Some(child),
            timeout_s: 1,
            ready_probe: ReadyProbe::None,
            owned_endpoints: Vec::new(),
            requires_live_child: true,
            docker_name: None,
            microvm_name: None,
            cluster: None,
            deadline_cleanup_owners: Vec::new(),
        }
    }
}

fn command_with_logs(
    cmd: &[String],
    cwd: &Path,
    env: &std::collections::BTreeMap<String, String>,
    stdout: &Path,
    stderr: &Path,
) -> Result<OwnedProcessGroup> {
    if cmd.is_empty() {
        bail!("empty command");
    }
    if let Some(parent) = stdout.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let out = File::create(stdout).with_context(|| format!("create {}", stdout.display()))?;
    let err = File::create(stderr).with_context(|| format!("create {}", stderr.display()))?;
    let mut command = Command::new(&cmd[0]);
    command
        .args(&cmd[1..])
        .current_dir(cwd)
        .stdout(Stdio::from(out))
        .stderr(Stdio::from(err));
    for (key, value) in env {
        command.env(key, value);
    }
    set_process_group(&mut command);
    command
        .spawn()
        .map(OwnedProcessGroup::new)
        .with_context(|| format!("spawn `{}`", cmd[0]))
}

#[cfg(unix)]
fn set_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn set_process_group(_command: &mut Command) {}

#[cfg(unix)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProcessGroupSignalOutcome {
    DeliveredOrGone,
    PermissionPartial,
}

#[cfg(unix)]
fn terminate_and_reap_owned_process_group_before(
    child: &mut Child,
    pgid: u32,
    label: &str,
    deadline: Instant,
) -> Result<ExitStatus> {
    if Instant::now() >= deadline {
        bail!("{label} has no shared deadline budget left for TERM/KILL/reap");
    }
    let term = signal_owned_process_group(pgid, libc::SIGTERM, label)?;
    let remaining = deadline.saturating_duration_since(Instant::now());
    // Reserve at least half of the remaining budget for KILL/reap and the
    // subsequent explicit group-absence proof.
    let grace_deadline = Instant::now() + Duration::from_millis(100).min(remaining / 4);
    while Instant::now() < grace_deadline {
        if child_has_exited_without_reap(child)? || !process_group_exists(pgid)? {
            break;
        }
        std::thread::sleep(
            OWNED_GROUP_POLL_INTERVAL.min(grace_deadline.saturating_duration_since(Instant::now())),
        );
    }

    let kill = if process_group_exists(pgid)? {
        signal_owned_process_group(pgid, libc::SIGKILL, label)?
    } else {
        ProcessGroupSignalOutcome::DeliveredOrGone
    };
    let permission_partial = matches!(term, ProcessGroupSignalOutcome::PermissionPartial)
        || matches!(kill, ProcessGroupSignalOutcome::PermissionPartial);
    if permission_partial && !child_has_exited_without_reap(child)? {
        match child.kill() {
            Ok(()) => {}
            Err(error) if error.raw_os_error() == Some(libc::ESRCH) => {}
            Err(error) => {
                return Err(error).with_context(|| format!("send direct KILL to {label}"));
            }
        }
    }

    let remaining = deadline.saturating_duration_since(Instant::now());
    let reap_deadline = Instant::now() + remaining / 2;
    loop {
        if let Some(status) = child
            .try_wait()
            .with_context(|| format!("reap {label} process-group leader {pgid}"))?
        {
            return Ok(status);
        }
        if Instant::now() >= reap_deadline {
            bail!("{label} process-group leader {pgid} did not exit before its shared cleanup deadline");
        }
        std::thread::sleep(
            OWNED_GROUP_POLL_INTERVAL.min(reap_deadline.saturating_duration_since(Instant::now())),
        );
    }
}

#[cfg(not(unix))]
fn terminate_and_reap_owned_process_group_before(
    child: &mut Child,
    _pgid: u32,
    label: &str,
    deadline: Instant,
) -> Result<ExitStatus> {
    if Instant::now() >= deadline {
        bail!("{label} has no shared deadline budget left for termination");
    }
    match child.try_wait()? {
        Some(status) => Ok(status),
        None => {
            child
                .kill()
                .with_context(|| format!("stop {label} child"))?;
            while Instant::now() < deadline {
                if let Some(status) = child.try_wait()? {
                    return Ok(status);
                }
                std::thread::sleep(
                    OWNED_GROUP_POLL_INTERVAL
                        .min(deadline.saturating_duration_since(Instant::now())),
                );
            }
            bail!("{label} child did not exit before its shared cleanup deadline")
        }
    }
}

#[cfg(unix)]
fn confirm_owned_process_group_absent_before(
    pgid: u32,
    label: &str,
    deadline: Instant,
) -> Result<()> {
    while process_group_exists(pgid)? && Instant::now() < deadline {
        std::thread::sleep(
            OWNED_GROUP_POLL_INTERVAL.min(deadline.saturating_duration_since(Instant::now())),
        );
    }
    if process_group_exists(pgid)? {
        bail!("{label} process group {pgid} remains at the shared cleanup deadline");
    }
    Ok(())
}

#[cfg(not(unix))]
fn confirm_owned_process_group_absent_before(
    _pgid: u32,
    _label: &str,
    _deadline: Instant,
) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn terminate_and_reap_owned_process_group(
    child: &mut Child,
    pgid: u32,
    label: &str,
    term_grace: Duration,
    stop_timeout: Duration,
) -> Result<ExitStatus> {
    let term = signal_owned_process_group(pgid, libc::SIGTERM, label)?;
    let grace_deadline = Instant::now() + term_grace;
    while Instant::now() < grace_deadline {
        // A waitable exited leader still makes kill(-pgid, 0) report that the
        // group exists. Observe that exit without reaping so TERM-responsive
        // commands do not burn the full grace period, while the unreaped
        // leader continues pinning the numeric PGID until KILL covers any
        // resistant descendants.
        if child_has_exited_without_reap(child)? || !process_group_exists(pgid)? {
            break;
        }
        std::thread::sleep(OWNED_GROUP_POLL_INTERVAL);
    }

    let kill = if process_group_exists(pgid)? {
        signal_owned_process_group(pgid, libc::SIGKILL, label)?
    } else {
        ProcessGroupSignalOutcome::DeliveredOrGone
    };
    let permission_partial = matches!(term, ProcessGroupSignalOutcome::PermissionPartial)
        || matches!(kill, ProcessGroupSignalOutcome::PermissionPartial);
    if permission_partial && !child_has_exited_without_reap(child)? {
        match child.kill() {
            Ok(()) => {}
            Err(error) if error.raw_os_error() == Some(libc::ESRCH) => {}
            Err(error) => {
                return Err(error).with_context(|| format!("send direct KILL to {label}"));
            }
        }
    }

    let reap_deadline = Instant::now() + stop_timeout;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .with_context(|| format!("reap {label} process-group leader {pgid}"))?
        {
            break status;
        }
        if Instant::now() >= reap_deadline {
            bail!("{label} process-group leader {pgid} did not exit after TERM/KILL");
        }
        std::thread::sleep(OWNED_GROUP_POLL_INTERVAL);
    };

    Ok(status)
}

#[cfg(not(unix))]
fn terminate_and_reap_owned_process_group(
    child: &mut Child,
    _pgid: u32,
    label: &str,
    _term_grace: Duration,
    _stop_timeout: Duration,
) -> Result<ExitStatus> {
    match child.try_wait()? {
        Some(status) => Ok(status),
        None => {
            child
                .kill()
                .with_context(|| format!("stop {label} child"))?;
            child.wait().with_context(|| format!("reap {label} child"))
        }
    }
}

#[cfg(unix)]
fn confirm_owned_process_group_absent(pgid: u32, label: &str, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    while process_group_exists(pgid)? && Instant::now() < deadline {
        std::thread::sleep(OWNED_GROUP_POLL_INTERVAL);
    }
    if process_group_exists(pgid)? {
        bail!("{label} process group {pgid} remains after TERM/KILL and leader reap");
    }
    Ok(())
}

#[cfg(not(unix))]
fn confirm_owned_process_group_absent(_pgid: u32, _label: &str, _timeout: Duration) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn child_has_exited_without_reap(child: &Child) -> Result<bool> {
    let mut info = unsafe { std::mem::zeroed::<libc::siginfo_t>() };
    let result = unsafe {
        libc::waitid(
            libc::P_PID,
            child.id() as libc::id_t,
            &mut info,
            libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
        )
    };
    if result != 0 {
        return Err(std::io::Error::last_os_error())
            .context("observe VAT-owned process-group leader without reaping");
    }
    Ok(unsafe { info.si_pid() } != 0)
}

#[cfg(not(unix))]
fn child_has_exited_without_reap(child: &Child) -> Result<bool> {
    // Non-Unix platforms do not expose wait-without-reap here. VAT's native
    // process-group contract is Unix-only; this conservative fallback avoids
    // consuming the handle before the common finalizer.
    let _ = child;
    Ok(false)
}

#[cfg(unix)]
fn process_group_exists(pgid: u32) -> Result<bool> {
    let result = unsafe { libc::kill(-(pgid as libc::pid_t), 0) };
    if result == 0 {
        return Ok(true);
    }
    let error = std::io::Error::last_os_error();
    match error.raw_os_error() {
        Some(libc::ESRCH) => Ok(false),
        Some(libc::EPERM) => Ok(true),
        _ => Err(error).with_context(|| format!("inspect VAT-owned process group {pgid}")),
    }
}

#[cfg(unix)]
fn signal_owned_process_group(
    pgid: u32,
    signal: i32,
    label: &str,
) -> Result<ProcessGroupSignalOutcome> {
    let result = unsafe { libc::kill(-(pgid as libc::pid_t), signal) };
    if result == 0 {
        return Ok(ProcessGroupSignalOutcome::DeliveredOrGone);
    }
    let error = std::io::Error::last_os_error();
    match error.raw_os_error() {
        Some(libc::ESRCH) => Ok(ProcessGroupSignalOutcome::DeliveredOrGone),
        Some(libc::EPERM) => Ok(ProcessGroupSignalOutcome::PermissionPartial),
        _ => Err(error)
            .with_context(|| format!("send signal {signal} to {label} process group {pgid}")),
    }
}

fn http_readiness(raw_url: &str) -> Result<EndpointReadiness> {
    let url = url::Url::parse(raw_url).with_context(|| format!("parse ready_http {raw_url}"))?;
    let host = url.host_str().context("ready_http missing host")?;
    let port = url
        .port_or_known_default()
        .context("ready_http missing port")?;
    let addr = (host, port)
        .to_socket_addrs()?
        .next()
        .context("ready_http did not resolve")?;
    let mut stream = match TcpStream::connect_timeout(&addr, Duration::from_millis(300)) {
        Ok(stream) => stream,
        Err(err) if readiness_io_pending(&err) => {
            return Ok(EndpointReadiness::Pending(format!(
                "ready_http endpoint {raw_url} is not accepting TCP yet: {err}"
            )));
        }
        Err(err) => return Err(err).with_context(|| format!("connect ready_http {raw_url}")),
    };
    stream.set_read_timeout(Some(Duration::from_millis(300)))?;
    let path = if url.path().is_empty() {
        "/"
    } else {
        url.path()
    };
    if let Err(err) = write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
    ) {
        if readiness_io_pending(&err) {
            return Ok(EndpointReadiness::Pending(format!(
                "ready_http endpoint {raw_url} did not accept the HTTP request yet: {err}"
            )));
        }
        return Err(err).with_context(|| format!("write ready_http request to {raw_url}"));
    }
    let mut buf = [0u8; 64];
    let n = match stream.read(&mut buf) {
        Ok(n) => n,
        Err(err) if readiness_io_pending(&err) => {
            return Ok(EndpointReadiness::Pending(format!(
                "ready_http endpoint {raw_url} did not return a response yet: {err}"
            )));
        }
        Err(err) => {
            return Err(err).with_context(|| format!("read ready_http response from {raw_url}"));
        }
    };
    if n == 0 {
        bail!("ready_http endpoint {raw_url} closed before responding");
    }
    let head = String::from_utf8_lossy(&buf[..n]);
    if head.starts_with("HTTP/1.0 2")
        || head.starts_with("HTTP/1.1 2")
        || head.starts_with("HTTP/1.0 3")
        || head.starts_with("HTTP/1.1 3")
    {
        Ok(EndpointReadiness::Ready)
    } else {
        let status = head.lines().next().unwrap_or("non-HTTP response");
        Ok(EndpointReadiness::Pending(format!(
            "ready_http endpoint {raw_url} returned non-ready response `{status}`"
        )))
    }
}

fn http_ready(raw_url: &str) -> Result<bool> {
    Ok(matches!(http_readiness(raw_url)?, EndpointReadiness::Ready))
}

fn collect_artifacts(rootfs: &Path, patterns: &[String]) -> Result<Vec<ArtifactRecord>> {
    let mut out = Vec::new();
    for pattern in patterns {
        if let Some(prefix) = pattern.strip_suffix("/**") {
            let dir = rootfs.join(prefix);
            if !dir.exists() {
                continue;
            }
            for entry in WalkDir::new(&dir).into_iter().filter_map(Result::ok) {
                if !entry.file_type().is_file() {
                    continue;
                }
                out.push(artifact_record(rootfs, entry.path())?);
            }
        } else {
            let path = rootfs.join(pattern);
            if path.is_file() {
                out.push(artifact_record(rootfs, &path)?);
            }
        }
    }
    Ok(out)
}

fn artifact_record(rootfs: &Path, path: &Path) -> Result<ArtifactRecord> {
    let rel = path
        .strip_prefix(rootfs)
        .context("artifact outside rootfs")?
        .to_string_lossy()
        .into_owned();
    Ok(ArtifactRecord {
        path: rel,
        size_bytes: path.metadata().ok().map(|m| m.len()),
    })
}

#[allow(clippy::too_many_arguments)]
fn print_summary(
    vat: &store::Vat,
    code: i32,
    duration_ms: u64,
    changes: &crate::state::ChangeSet,
    backend: &str,
    gpu: &gpu::GpuInfo,
) {
    let id = &vat.meta.id;
    println!(
        "{id} · exited {code} in {duration_ms}ms · {backend} · changes {}",
        changes.oneline()
    );
    let chip = gpu.chip.as_deref().unwrap_or("unknown");
    let mark = if gpu.accessible { "✓" } else { "✗" };
    println!("gpu {mark} {chip} [{}]", gpu.backends.join(", "));
    println!("→ vat state {id}    # full JSON for an agent");
}
// CODEGEN-END

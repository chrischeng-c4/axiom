// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-commands.md#schema
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
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use chrono::Utc;
use walkdir::WalkDir;

use crate::cluster::{self, ClusterSpec, ResolvedBackend};
use crate::config::{
    self, ClusterBackend, PortSpec, RetentionPolicy, RunnerConfig, ServiceConfig, ServicePreset,
    ServiceRuntime, VatConfig,
};
use crate::event::{Event, EventKind};
use crate::gpu;
use crate::overlay;
use crate::sandbox;
use crate::spec::{Base, EnvSpec, GpuRequest, Isolation};
use crate::state::{
    ArtifactRecord, ClusterRunRecord, ConfigRef, ProcessStatus, RunRecord, RunnerRunRecord,
    ServiceRunRecord, Status, TestRunEvidence,
};
use crate::{id, store};

/// Inputs for `vat run`, already parsed by the CLI layer.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-commands-run-rs.md#source
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#cli
pub struct Args {
    pub target: Target,
    /// Clone from this host directory (default: current directory).
    pub base: Option<PathBuf>,
    /// Fork from an existing vat instead of a host directory.
    pub from: Option<String>,
    pub name: Option<String>,
    pub isolation: Isolation,
    pub gpu: GpuRequest,
    /// Direct mode prints full VatState JSON instead of a human summary.
    pub json: bool,
}

/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#cli
pub enum Target {
    Direct {
        program: String,
        program_args: Vec<String>,
    },
    Runner {
        /// Empty = default selection; several = run CONCURRENTLY in one vat.
        runner_ids: Vec<String>,
    },
}

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-commands-run-rs.md#source
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
pub fn exec(args: Args) -> Result<ExitCode> {
    let Args {
        target,
        base,
        from,
        name,
        isolation,
        gpu,
        json,
    } = args;
    match target {
        Target::Direct {
            program,
            program_args,
        } => exec_direct(DirectArgs {
            program,
            program_args,
            base,
            from,
            name,
            isolation,
            gpu,
            json,
        }),
        Target::Runner { runner_ids } => exec_runner(RunnerArgs {
            base,
            from,
            name,
            isolation,
            gpu,
            runner_ids,
        }),
    }
}

struct RunnerArgs {
    base: Option<PathBuf>,
    from: Option<String>,
    name: Option<String>,
    isolation: Isolation,
    gpu: GpuRequest,
    runner_ids: Vec<String>,
}

struct DirectArgs {
    program: String,
    program_args: Vec<String>,
    base: Option<PathBuf>,
    from: Option<String>,
    name: Option<String>,
    isolation: Isolation,
    gpu: GpuRequest,
    json: bool,
}

fn exec_direct(args: DirectArgs) -> Result<ExitCode> {
    let gpu_info = gpu::detect();
    if args.gpu == GpuRequest::Required && !gpu_info.accessible {
        bail!(
            "spec requires a GPU but none is accessible on this host ({})",
            gpu_info.note
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

    let command: Vec<String> = std::iter::once(args.program.clone())
        .chain(args.program_args.iter().cloned())
        .collect();
    vat.meta.status = Status::Running;
    vat.meta.last_run = Some(RunRecord {
        command: command.clone(),
        started_at: Utc::now(),
        finished_at: None,
        exit_code: None,
        duration_ms: None,
    });
    vat.save()?;
    let backend = sandbox::pick(&spec);
    vat.log(
        Event::new(EventKind::RunStarted, format!("run: {}", command.join(" ")))
            .with_data(serde_json::json!({ "backend": backend.name() })),
    )?;

    let rootfs = vat.rootfs();
    let (prog, argv) = backend.resolve(&rootfs, &args.program, &args.program_args);
    let cwd = rootfs.join(&spec.workdir);
    let started = Instant::now();
    let mut cmd = Command::new(&prog);
    cmd.args(&argv).current_dir(&cwd);
    for (key, value) in &spec.env {
        cmd.env(key, value);
    }
    let status = cmd
        .status()
        .with_context(|| format!("spawn `{prog}` inside vat rootfs"))?;
    let duration_ms = started.elapsed().as_millis() as u64;
    let code = status.code().unwrap_or(-1);

    vat.meta.status = Status::Exited { code };
    if let Some(run) = vat.meta.last_run.as_mut() {
        run.finished_at = Some(Utc::now());
        run.exit_code = Some(code);
        run.duration_ms = Some(duration_ms);
    }
    vat.save()?;
    let changes = vat.changes().unwrap_or_default();
    vat.log(
        Event::new(
            EventKind::RunFinished,
            format!("exit {code} in {duration_ms}ms · {}", changes.oneline()),
        )
        .with_data(serde_json::json!({
            "exit_code": code,
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

fn exec_runner(args: RunnerArgs) -> Result<ExitCode> {
    let cwd = std::env::current_dir().context("get current directory")?;
    let cfg = config::load_nearest(&cwd)?;
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
    if args.gpu == GpuRequest::Required && !gpu_info.accessible {
        emit_jsonl(serde_json::json!({
            "type": "error",
            "code": "gpu_required",
            "message": gpu_info.note.as_str(),
        }))?;
        bail!(
            "spec requires a GPU but none is accessible on this host ({})",
            gpu_info.note
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

    vat.meta.status = Status::Running;
    vat.meta.test_run = Some(TestRunEvidence {
        config: ConfigRef {
            path: cfg.path.to_string_lossy().into_owned(),
            digest: cfg.digest.clone(),
        },
        runner_id: joined_ids.clone(),
        retention: cfg.workspace.keep,
        services: Vec::new(),
        runner: None,
        runners: Vec::new(),
        artifacts: Vec::new(),
    });
    vat.save()?;
    vat.log(Event::new(
        EventKind::RunStarted,
        format!("runner: {joined_ids}"),
    ))?;

    let result = run_configured(&mut vat, &cfg, &runners, &logs_dir);
    let code = match result {
        Ok(code) => code,
        Err(err) => {
            emit_jsonl(serde_json::json!({
                "type": "error",
                "code": "run_failed",
                "message": err.to_string(),
            }))?;
            record_runner_failure(&mut vat, &runners[0], &logs_dir, &err.to_string())?;
            -1
        }
    };

    vat.meta.status = Status::Exited { code };
    vat.save()?;
    let state = vat.project()?;
    let should_remove = match cfg.workspace.keep {
        RetentionPolicy::Always => false,
        RetentionPolicy::Never => true,
        RetentionPolicy::Failed => code == 0,
    };

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

fn run_configured(
    vat: &mut store::Vat,
    cfg: &VatConfig,
    runners: &[RunnerConfig],
    logs_dir: &Path,
) -> Result<i32> {
    let rootfs = vat.rootfs();
    let cwd = rootfs.join(&vat.meta.spec.workdir);
    std::fs::create_dir_all(&cwd).with_context(|| format!("create {}", cwd.display()))?;

    // Services: the UNION of every runner's requires, order-preserving and
    // deduplicated — one shared instance set serves all concurrent runners.
    let mut service_ids: Vec<&str> = Vec::new();
    for runner in runners {
        for service_id in &runner.requires {
            if !service_ids.contains(&service_id.as_str()) {
                service_ids.push(service_id);
            }
        }
    }
    let mut service_plans = Vec::new();
    let mut run_env = vat.meta.spec.env.clone();
    for service in ordered_required_services(cfg, &service_ids)? {
        let plan = prepare_service(vat, cfg, service)?;
        for (key, value) in &plan.env {
            run_env.insert(key.clone(), value.clone());
        }
        service_plans.push(plan);
    }

    // Transparent service routing (network sandbox): every declared GCP emulator
    // preset's real googleapis host -> its resolved local endpoint, seeded onto
    // the http-mock proxy so the runner reaches the local emulator with NO
    // hand-written [network.routes]. Ports are resolved during the prepare loop
    // above, so the proxy is spawned with the full (explicit + preset-derived)
    // route set.
    seed_preset_routes_into_proxy(&mut service_plans, cfg);

    for step in &cfg.setup {
        if !config::should_run_setup(&rootfs, step) {
            continue;
        }
        run_setup_step(vat, step, &cwd, logs_dir, &run_env)?;
    }

    let mut services = Vec::new();
    for plan in &service_plans {
        let handle = match start_service(vat, plan, &cwd, logs_dir, &run_env) {
            Ok(handle) => handle,
            Err(err) => {
                stop_services(
                    &mut services,
                    should_delete_clusters(&cfg.workspace.keep, -1),
                );
                persist_services(vat, &services)?;
                return Err(err);
            }
        };
        services.push(handle);
        let last = services.len() - 1;
        if let Err(err) = wait_for_services(vat, &mut services[last..]) {
            stop_services(
                &mut services,
                should_delete_clusters(&cfg.workspace.keep, -1),
            );
            persist_services(vat, &services)?;
            return Err(err);
        }
        persist_services(vat, &services)?;
    }
    persist_services(vat, &services)?;

    // Spawn every runner, THEN wait — concurrency comes from the children
    // running side by side, not from threads in vat.
    let single = runners.len() == 1;
    let mut procs = Vec::new();
    for runner in runners {
        emit_jsonl(serde_json::json!({
            "type": "runner",
            "id": runner.id.as_str(),
            "state": "started",
        }))?;
        match spawn_runner_process(runner, &cwd, logs_dir, &run_env, single) {
            Ok(proc) => procs.push(proc),
            Err(err) => {
                kill_runner_processes(&mut procs);
                stop_services(
                    &mut services,
                    should_delete_clusters(&cfg.workspace.keep, -1),
                );
                persist_services(vat, &services)?;
                return Err(err);
            }
        }
    }
    let records = wait_runner_processes(procs)?;

    // Worst-wins exit: any negative (timeout/kill) is worst, else max code.
    let code = records
        .iter()
        .map(|r| r.exit_code.unwrap_or(-1))
        .fold(0, |acc, c| if acc < 0 || c < 0 { -1 } else { acc.max(c) });
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
    vat.save()?;
    stop_services(
        &mut services,
        should_delete_clusters(&cfg.workspace.keep, code),
    );
    persist_services(vat, &services)?;
    let summary = records
        .iter()
        .map(|r| format!("{} exited {}", r.id, r.exit_code.unwrap_or(-1)))
        .collect::<Vec<_>>()
        .join("; ");
    vat.log(Event::new(EventKind::RunFinished, summary))?;
    Ok(code)
}

fn kill_runner_processes(procs: &mut [RunnerProc]) {
    for proc in procs {
        kill_child(&mut proc.child);
        let _ = proc.child.wait();
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

/// One spawned (not yet reaped) runner child plus its bookkeeping.
struct RunnerProc {
    runner: RunnerConfig,
    child: Child,
    started: Instant,
    deadline: Option<Instant>,
    stdout_log: String,
    stderr_log: String,
}

/// Spawn one runner child with per-runner log files. A single runner keeps
/// the legacy `runner.{stdout,stderr}.log` names; a concurrent set
/// disambiguates as `runner-<id>.{stdout,stderr}.log`.
fn spawn_runner_process(
    runner: &RunnerConfig,
    cwd: &Path,
    logs_dir: &Path,
    env: &BTreeMap<String, String>,
    single: bool,
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
    let child = command_with_logs(&runner.cmd, cwd, env, &stdout, &stderr)
        .with_context(|| format!("spawn runner `{}`", runner.id))?;
    Ok(RunnerProc {
        runner: runner.clone(),
        deadline: runner.timeout_s.map(|s| started + Duration::from_secs(s)),
        started,
        child,
        stdout_log: stdout.to_string_lossy().into_owned(),
        stderr_log: stderr.to_string_lossy().into_owned(),
    })
}

/// Poll every child to completion, enforcing each runner's own timeout
/// (an elapsed budget kills that child; the others keep running).
fn wait_runner_processes(mut procs: Vec<RunnerProc>) -> Result<Vec<RunnerRunRecord>> {
    let mut records: Vec<Option<RunnerRunRecord>> = procs.iter().map(|_| None).collect();
    loop {
        let mut all_done = true;
        for (i, proc) in procs.iter_mut().enumerate() {
            if records[i].is_some() {
                continue;
            }
            if let Some(status) = proc.child.try_wait()? {
                records[i] = Some(RunnerRunRecord {
                    id: proc.runner.id.clone(),
                    command: proc.runner.cmd.clone(),
                    status: ProcessStatus::Exited,
                    exit_code: Some(status.code().unwrap_or(-1)),
                    duration_ms: Some(proc.started.elapsed().as_millis() as u64),
                    stdout_log: proc.stdout_log.clone(),
                    stderr_log: proc.stderr_log.clone(),
                });
                continue;
            }
            if let Some(deadline) = proc.deadline {
                if Instant::now() >= deadline {
                    kill_child(&mut proc.child);
                    let _ = proc.child.wait();
                    records[i] = Some(RunnerRunRecord {
                        id: proc.runner.id.clone(),
                        command: proc.runner.cmd.clone(),
                        status: ProcessStatus::Exited,
                        exit_code: Some(-1),
                        duration_ms: Some(proc.started.elapsed().as_millis() as u64),
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
    vat: &store::Vat,
    step: &crate::config::SetupStep,
    cwd: &Path,
    logs_dir: &Path,
    env: &BTreeMap<String, String>,
) -> Result<()> {
    let stdout = logs_dir.join(format!("setup-{}.stdout.log", step.id));
    let stderr = logs_dir.join(format!("setup-{}.stderr.log", step.id));
    let status = command_with_logs(&step.cmd, cwd, env, &stdout, &stderr)?
        .wait()
        .with_context(|| format!("wait setup `{}`", step.id))?;
    if !status.success() {
        bail!("setup `{}` failed with {:?}", step.id, status.code());
    }
    vat.log(Event::new(EventKind::Setup, format!("setup {}", step.id)))?;
    Ok(())
}

#[derive(Debug, Clone)]
struct ServicePlan {
    id: String,
    command: Vec<String>,
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
    /// The Docker image, when this service runs as a container.
    image: Option<String>,
    /// Set when the service is a local Kubernetes cluster; carries the cluster
    /// evidence so teardown can delete it and `vat state` can surface it.
    cluster: Option<ClusterRunRecord>,
}

#[derive(Debug, Clone)]
enum ReadyProbe {
    None,
    Http(String),
    Tcp { host: String, port: u16 },
    Cmd(Vec<String>),
}

struct ServiceHandle {
    record: ServiceRunRecord,
    child: Child,
    timeout_s: u64,
    ready_probe: ReadyProbe,
    /// `docker --name` when the service is a container; force-removed on stop.
    docker_name: Option<String>,
    /// Cluster evidence when the service is a local Kubernetes cluster; the
    /// cluster is deleted on stop subject to the `keep` policy.
    cluster: Option<ClusterRunRecord>,
}

fn prepare_service(
    vat: &store::Vat,
    cfg: &VatConfig,
    service: &ServiceConfig,
) -> Result<ServicePlan> {
    let started = Instant::now();
    let plan = if let Some(backend) = service.cluster {
        // Cluster: an ephemeral local Kubernetes cluster (kind / k3d / minikube).
        // Created here in the prepare phase; the runner reaches it via KUBECONFIG.
        prepare_cluster_service(vat, service, backend)?
    } else if let Some(image) = &service.image {
        // Explicit image: a Docker-only service (e.g. AlloyDB) with no native
        // equivalent. Always a container.
        prepare_image_service(vat, service, image)?
    } else if service.preset == Some(ServicePreset::Firebase) {
        // Firebase: a multi-emulator bundle driven by firebase.json — its own
        // prepare path because it is one process exposing many ports.
        prepare_firebase_service(vat, cfg, service)?
    } else if let Some(preset) = service.preset {
        // Preset: prefer the native Homebrew binary, fall back to the preset's
        // official Docker image when the binary is missing (or as forced).
        match resolve_preset_runtime(service, preset)? {
            ResolvedRuntime::Native => prepare_preset_service(vat, cfg, service, preset)?,
            ResolvedRuntime::Docker => prepare_preset_docker_service(vat, service, preset)?,
            ResolvedRuntime::Builtin => {
                prepare_builtin_service(service, preset, &cfg.root, &explicit_network_routes(cfg))?
            }
        }
    } else {
        let env = export_command_service_env(service);
        ServicePlan {
            id: service.id.clone(),
            command: service.cmd.clone(),
            ready_http: service.ready_http.clone(),
            ready_probe: resolve_ready_probe(service, None),
            timeout_s: service.timeout_s,
            preset: None,
            port: None,
            prepare_mode: "direct_start".to_string(),
            cache_key: None,
            prepare_duration_ms: 0,
            exported_env: sorted_keys(&env),
            env,
            docker_name: None,
            image: None,
            cluster: None,
        }
    };
    let mut plan = plan;
    plan.prepare_duration_ms = started.elapsed().as_millis() as u64;
    // Cluster services emit their own prepare checkpoint inside
    // `prepare_cluster_service`; the container/preset note below does not apply.
    if plan.prepare_mode != "direct_start" && plan.cluster.is_none() {
        let is_docker = plan.docker_name.is_some();
        let note = if is_docker {
            "running service via `docker run` (ephemeral, --rm)"
        } else if plan.prepare_mode == "cold_build" {
            "first run slower; cached for future runs"
        } else {
            "using cached service image"
        };
        emit_jsonl(serde_json::json!({
            "type": "prepare",
            "service": plan.id.as_str(),
            "preset": plan.preset.map(service_preset_name),
            "runtime": if is_docker { "docker" } else { "native" },
            "image": plan.image.as_deref(),
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
/// @spec projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#logic
fn prepare_cluster_service(
    vat: &store::Vat,
    service: &ServiceConfig,
    backend: ClusterBackend,
) -> Result<ServicePlan> {
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
    let info = match resolved.create(&spec, Duration::from_secs(service.timeout_s)) {
        Ok(info) => info,
        Err(err) => {
            // Best-effort cleanup so a half-created cluster does not leak.
            let _ = resolved.delete(&name);
            emit_jsonl(serde_json::json!({
                "type": "error",
                "code": "cluster_create_failed",
                "service": service.id.as_str(),
                "backend": resolved.name(),
                "reason": err.to_string(),
            }))?;
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
        image: None,
        cluster: Some(record),
    })
}

fn start_service(
    vat: &mut store::Vat,
    plan: &ServicePlan,
    cwd: &Path,
    logs_dir: &Path,
    env: &BTreeMap<String, String>,
) -> Result<ServiceHandle> {
    let stdout = logs_dir.join(format!("{}.stdout.log", plan.id));
    let stderr = logs_dir.join(format!("{}.stderr.log", plan.id));
    let child = command_with_logs(&plan.command, cwd, env, &stdout, &stderr)
        .with_context(|| format!("spawn service `{}`", plan.id))?;
    let record = ServiceRunRecord {
        id: plan.id.clone(),
        command: plan.command.clone(),
        status: ProcessStatus::Running,
        preset: plan.preset.map(service_preset_name).map(str::to_string),
        port: plan.port,
        prepare_mode: Some(plan.prepare_mode.clone()),
        cache_key: plan.cache_key.clone(),
        prepare_duration_ms: Some(plan.prepare_duration_ms),
        ready_duration_ms: None,
        exported_env: plan.exported_env.clone(),
        pid: Some(child.id()),
        exit_code: None,
        ready_http: plan.ready_http.clone(),
        cluster: plan.cluster.clone(),
        stdout_log: stdout.to_string_lossy().into_owned(),
        stderr_log: stderr.to_string_lossy().into_owned(),
    };
    vat.log(Event::new(
        EventKind::RunStarted,
        format!("service {}", plan.id),
    ))?;
    Ok(ServiceHandle {
        record,
        child,
        timeout_s: plan.timeout_s,
        ready_probe: plan.ready_probe.clone(),
        docker_name: plan.docker_name.clone(),
        cluster: plan.cluster.clone(),
    })
}

fn prepare_preset_service(
    vat: &store::Vat,
    cfg: &VatConfig,
    service: &ServiceConfig,
    preset: ServicePreset,
) -> Result<ServicePlan> {
    ensure_preset_binaries(service, preset)?;
    let port = resolve_service_port(&service.port)?;
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
            if let Err(err) = cold_prepare_service_image(cfg, service, preset, &tmp_cache_dir) {
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
        image: None,
        cluster: None,
    })
}

/// Which way a `preset` service is actually provided on this host.
enum ResolvedRuntime {
    Native,
    Docker,
    /// vat's own in-process Rust emulator (the `vat emulator` subcommand).
    Builtin,
}

/// Resolve a preset service's `runtime` against the host. `auto` prefers the
/// native binary and falls back to Docker; `native`/`docker` force one path.
/// On `auto` with neither available, emit a structured error and bail.
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
fn resolve_preset_runtime(
    service: &ServiceConfig,
    preset: ServicePreset,
) -> Result<ResolvedRuntime> {
    match service.runtime {
        ServiceRuntime::Native => Ok(ResolvedRuntime::Native),
        ServiceRuntime::Docker => Ok(ResolvedRuntime::Docker),
        // A preset with a built-in Rust emulator runs vat's own server under
        // `auto` — always available, no external tooling.
        ServiceRuntime::Auto if preset.is_builtin() => Ok(ResolvedRuntime::Builtin),
        ServiceRuntime::Auto => {
            // Native means more than "binary on PATH" for emulators: the gcloud
            // component must also be installed, else native would be chosen and
            // then fail to start. preset_native_available folds that in so a
            // missing component falls back to Docker.
            if preset_native_available(preset) {
                Ok(ResolvedRuntime::Native)
            } else if docker_available() {
                Ok(ResolvedRuntime::Docker)
            } else {
                let missing: Vec<&str> = required_binaries(preset)
                    .iter()
                    .filter(|binary| which(binary).is_none())
                    .copied()
                    .collect();
                let missing_component = gcloud_component(preset)
                    .filter(|c| !installed_gcloud_components().iter().any(|x| x == c));
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

/// Run a preset service from its official Docker image instead of the native
/// binary. The exported connection env is identical to the native path — only
/// the process behind the mapped host port differs.
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
fn prepare_preset_docker_service(
    vat: &store::Vat,
    service: &ServiceConfig,
    preset: ServicePreset,
) -> Result<ServicePlan> {
    ensure_docker_available(service)?;
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
    let mut command = docker_run_command(&name, &image, host_port, container_port, &container_env);
    // GCP emulators on the cloud-cli image need the emulator start command
    // appended; the datastore/broker official images and Spanner's dedicated
    // image start via their own entrypoint.
    command.extend(preset_docker_command(preset, container_port));
    let env = preset_exports(service, preset, host_port);
    Ok(ServicePlan {
        id: service.id.clone(),
        command,
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
        image: Some(image),
        cluster: None,
    })
}

/// Prepare the `firebase` bundle: one `firebase emulators:start` process that
/// serves every emulator configured in the workspace `firebase.json`. vat reads
/// firebase.json for the ports (firebase owns them — vat does not auto-allocate),
/// exports the well-known `*_EMULATOR_HOST` vars the client SDKs read, and probes
/// the first configured emulator (or the hub) for readiness. Native-only: there
/// is no reliable official Docker image, so a missing firebase-tools is a
/// structured unavailable error, not a silent Docker attempt.
/// @spec projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#logic
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
    if let Some(emulators) = parsed.get("emulators").and_then(|e| e.as_object()) {
        for (emulator, conf) in emulators {
            let port = conf.get("port").and_then(|p| p.as_u64()).map(|p| p as u16);
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

    let ready_port = first_port.unwrap_or(hub_port);
    Ok(ServicePlan {
        id: service.id.clone(),
        command: vec![
            "firebase".to_string(),
            "emulators:start".to_string(),
            "--project".to_string(),
            "demo-vat".to_string(),
        ],
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
        image: None,
        cluster: None,
    })
}

/// The client-SDK host env var for a Firebase emulator, when one exists.
/// @spec projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#config
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
/// @spec projects/vat/tech-design/logic/built-in-rust-emulators-pub-sub-firebase-auth.md#config
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

/// Prepare a built-in emulator service: vat spawns *itself* (`vat emulator
/// <kind> --host-port`) as the service process — a pure Rust in-process server
/// with no external tooling. The runner reaches it via the exported host var.
/// @spec projects/vat/tech-design/logic/built-in-rust-emulators-pub-sub-firebase-auth.md#logic
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
) -> Result<ServicePlan> {
    let port = resolve_service_port(&service.port)?;
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

    let env = if preset == ServicePreset::HttpMock {
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
        if service.export.is_empty() {
            env.insert(default_var.to_string(), host_port.clone());
        } else {
            for target in service.export.values() {
                env.insert(target.clone(), host_port.clone());
            }
        }
        env
    };

    Ok(ServicePlan {
        id: service.id.clone(),
        command,
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
        image: None,
        cluster: None,
    })
}

/// The env set the http-mock proxy exports into the runner: proxy vars (so all
/// outbound HTTP/S is intercepted), NO_PROXY (so the runner's other loopback
/// emulators stay direct), and CA-trust vars for every common runtime (so the
/// HTTPS MITM is trusted) — plus the admin host.
/// @spec projects/vat/tech-design/logic/built-in-http-mock-record-replay-proxy.md#config
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
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
fn prepare_image_service(
    vat: &store::Vat,
    service: &ServiceConfig,
    image: &str,
) -> Result<ServicePlan> {
    ensure_docker_available(service)?;
    let host_port = resolve_service_port(&service.port)?;
    let container_port = service
        .container_port
        .context("image service missing container_port (validated earlier)")?;
    let name = container_name(&vat.meta.id, &service.id);
    let command = docker_run_command(&name, image, host_port, container_port, &service.image_env);
    let env = image_exports(service, host_port);
    Ok(ServicePlan {
        id: service.id.clone(),
        command,
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
        image: Some(image.to_string()),
        cluster: None,
    })
}

/// Build a foreground `docker run` argv. `--rm` makes the container ephemeral;
/// `--name` is deterministic so teardown can force-remove it; the port is bound
/// to loopback only. Container env is emitted in sorted order (deterministic).
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
fn docker_run_command(
    name: &str,
    image: &str,
    host_port: u16,
    container_port: u16,
    container_env: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut cmd = vec![
        "docker".to_string(),
        "run".to_string(),
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
    }
}

/// The emulator-start command appended after the image for GCP emulators on the
/// cloud-cli image. Empty for images that start their server via their own
/// entrypoint (datastore/broker official images, Spanner's dedicated image).
/// @spec projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#logic
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
        | ServicePreset::Openapi => {}
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
    let upper = service.id.to_uppercase().replace(['-', '.'], "_");
    env.insert(format!("VAT_SERVICE_{upper}_HOST"), "127.0.0.1".to_string());
    env.insert(format!("VAT_SERVICE_{upper}_PORT"), host_port.to_string());
    env
}

/// Whether Docker is usable: the binary is on PATH and the daemon answers.
fn docker_available() -> bool {
    which("docker").is_some() && docker_daemon_up()
}

fn docker_daemon_up() -> bool {
    Command::new("docker")
        .arg("info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Gate a Docker-backed service on a reachable daemon, emitting the structured
/// `docker_unavailable` error (never a panic) when it is not.
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#logic
fn ensure_docker_available(service: &ServiceConfig) -> Result<()> {
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
    if !docker_daemon_up() {
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

fn cold_prepare_service_image(
    cfg: &VatConfig,
    service: &ServiceConfig,
    preset: ServicePreset,
    cache_dir: &Path,
) -> Result<()> {
    match preset {
        ServicePreset::Postgres => {
            let status = Command::new("initdb")
                .args(["-D"])
                .arg(cache_dir)
                .args(["--auth=trust", "--username=postgres"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .context("run initdb for postgres service image")?;
            if !status.success() {
                bail!("postgres initdb failed for service `{}`", service.id);
            }
            // Apply `.sql` corpus seeds into the data dir ONCE, here in the
            // cold-prepare path. The populated dir is then cached and cloned
            // warm (clonefile COW) on every run, so the corpus is not rebuilt.
            cold_seed_postgres(cfg, service, cache_dir)?;
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
        | ServicePreset::Firebase | ServicePreset::FirebaseAuth | ServicePreset::CloudTasks | ServicePreset::CloudScheduler | ServicePreset::CloudWorkflows | ServicePreset::CloudStorage | ServicePreset::HttpMock | ServicePreset::Openapi => {}
    }
    Ok(())
}

/// Apply each `.sql` seed to a freshly-initdb'd cluster by briefly starting a
/// local postgres on a private socket dir (no TCP), running `psql -f`, then
/// stopping cleanly. Runs during cold prepare so the result is cached.
fn cold_seed_postgres(cfg: &VatConfig, service: &ServiceConfig, data_dir: &Path) -> Result<()> {
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
    let sock_dir_abs = sock_dir
        .canonicalize()
        .with_context(|| format!("canonicalize postgres seed socket dir {}", sock_dir.display()))?;
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
    let start = Command::new("pg_ctl")
        .arg("-D")
        .arg(&data_dir_abs)
        .args(["-w", "-t", "60", "-o"])
        .arg(&sock_arg)
        .arg("start")
        .stdout(start_stdout_handle)
        .stderr(start_stderr_handle)
        .status()
        .context("start temporary postgres for corpus seeding")?;
    if !start.success() {
        bail!(
            "could not start temporary postgres to seed service `{}`: stdout: {}; stderr: {}",
            service.id,
            command_output_file_tail(&start_stdout_path),
            command_output_file_tail(&start_stderr_path)
        );
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
        let status = Command::new("psql")
            .args(["-v", "ON_ERROR_STOP=1", "-h"])
            .arg(&sock_dir_abs)
            .args(["-p", "5432", "-U", "postgres", "-d", "postgres", "-f"])
            .arg(&path)
            .stdout(Stdio::null())
            .stderr(seed_stderr_handle)
            .status();
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
            Err(err) => {
                seed_result = Err(anyhow::Error::from(err).context(format!(
                    "run psql -f {} for service `{}`",
                    path.display(),
                    service.id
                )));
                break;
            }
        }
    }

    let stop = Command::new("pg_ctl")
        .arg("-D")
        .arg(&data_dir_abs)
        .args(["-w", "-t", "60", "-m", "fast", "stop"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("stop temporary postgres after corpus seeding")?;
    seed_result?;
    if !stop.success() {
        bail!(
            "temporary postgres did not stop cleanly after seeding service `{}`",
            service.id
        );
    }
    // Drop the throwaway socket dir so it is not baked into the cached image.
    let _ = std::fs::remove_dir_all(&sock_dir);
    Ok(())
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
    }
}

/// The gcloud component an emulator preset needs locally installed for the
/// native path. `None` for non-gcloud presets (datastore/broker, firebase).
/// @spec projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#config
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
fn installed_gcloud_components() -> Vec<String> {
    Command::new("gcloud")
        .args([
            "components",
            "list",
            "--only-local-state",
            "--format=value(id)",
        ])
        .stderr(Stdio::null())
        .output()
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Pure native-availability decision: all binaries present, and (for emulator
/// presets) the required gcloud component locally installed.
/// @spec projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#logic
fn native_available(has_binaries: bool, component: Option<&str>, installed: &[String]) -> bool {
    has_binaries
        && match component {
            Some(c) => installed.iter().any(|x| x == c),
            None => true,
        }
}

/// Whether a preset's native path is usable on this host. For emulator presets
/// this checks the gcloud component, not just the binary, so `runtime = auto`
/// falls back to Docker when the component is missing rather than choosing
/// native and failing to start.
/// @spec projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#logic
fn preset_native_available(preset: ServicePreset) -> bool {
    let has_binaries = required_binaries(preset)
        .iter()
        .all(|binary| which(binary).is_some());
    let component = gcloud_component(preset);
    // Only pay the gcloud query when a component actually gates this preset.
    let installed = if component.is_some() {
        installed_gcloud_components()
    } else {
        Vec::new()
    };
    native_available(has_binaries, component, &installed)
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
    };
    let mut env = BTreeMap::new();
    if service.export.is_empty() {
        env.insert(default_env.0.to_string(), default_env.1);
    } else {
        for target in service.export.values() {
            env.insert(target.clone(), default_env.1.clone());
        }
    }
    env
}

fn export_command_service_env(service: &ServiceConfig) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    if let Some(ready_http) = &service.ready_http {
        for target in service.export.values() {
            env.insert(target.clone(), ready_http.clone());
        }
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
        ServicePreset::Firestore => "firestore",
        ServicePreset::Pubsub => "pubsub",
        ServicePreset::Datastore => "datastore",
        ServicePreset::Bigtable => "bigtable",
        ServicePreset::Spanner => "spanner",
        ServicePreset::Firebase => "firebase",
        ServicePreset::FirebaseAuth => "firebase-auth",
        ServicePreset::CloudTasks => "cloud-tasks",
        ServicePreset::CloudScheduler => "cloud-scheduler",
        ServicePreset::CloudWorkflows => "cloud-workflows",
        ServicePreset::CloudStorage => "cloud-storage",
        ServicePreset::HttpMock => "http-mock",
        ServicePreset::Openapi => "openapi",
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

fn readiness_ready(probe: &ReadyProbe) -> Result<bool> {
    match probe {
        ReadyProbe::None => Ok(true),
        ReadyProbe::Http(url) => http_ready(url),
        ReadyProbe::Tcp { host, port } => tcp_ready(host, *port),
        ReadyProbe::Cmd(cmd) => {
            if cmd.is_empty() {
                return Ok(true);
            }
            Ok(Command::new(&cmd[0])
                .args(&cmd[1..])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|status| status.success())
                .unwrap_or(false))
        }
    }
}

fn tcp_ready(host: &str, port: u16) -> Result<bool> {
    let addr = (host, port)
        .to_socket_addrs()?
        .next()
        .context("tcp readiness did not resolve")?;
    Ok(TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok())
}

fn wait_for_services(vat: &mut store::Vat, services: &mut [ServiceHandle]) -> Result<()> {
    for service in services {
        let started = Instant::now();
        let ready_probe = service.ready_probe.clone();
        if matches!(ready_probe, ReadyProbe::None) {
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
            if readiness_ready(&ready_probe).unwrap_or(false) {
                service.record.status = ProcessStatus::Ready;
                let ms = started.elapsed().as_millis() as u64;
                service.record.ready_duration_ms = Some(ms);
                if let Some(cluster) = service.record.cluster.as_mut() {
                    cluster.ready_ms = Some(ms);
                }
                break;
            }
            if let Some(status) = service.child.try_wait()? {
                service.record.status = ProcessStatus::Failed;
                service.record.exit_code = status.code();
                bail!("service `{}` exited before readiness", service.record.id);
            }
            if Instant::now() >= deadline {
                service.record.status = ProcessStatus::Timeout;
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
    }
    Ok(())
}

fn record_runner_failure(
    vat: &mut store::Vat,
    runner: &RunnerConfig,
    logs_dir: &Path,
    message: &str,
) -> Result<()> {
    let stderr = logs_dir.join("runner.stderr.log");
    let mut file = OpenOptions::new().create(true).append(true).open(&stderr)?;
    writeln!(file, "{message}")?;
    if let Some(test_run) = vat.meta.test_run.as_mut() {
        test_run.runner = Some(RunnerRunRecord {
            id: runner.id.clone(),
            command: runner.cmd.clone(),
            status: ProcessStatus::Failed,
            exit_code: Some(-1),
            duration_ms: None,
            stdout_log: logs_dir
                .join("runner.stdout.log")
                .to_string_lossy()
                .into_owned(),
            stderr_log: stderr.to_string_lossy().into_owned(),
        });
    }
    vat.save()?;
    Ok(())
}

fn persist_services(vat: &mut store::Vat, services: &[ServiceHandle]) -> Result<()> {
    if let Some(test_run) = vat.meta.test_run.as_mut() {
        test_run.services = services.iter().map(|s| s.record.clone()).collect();
    }
    vat.save()
}

fn stop_services(services: &mut [ServiceHandle], delete_clusters: bool) {
    for service in services.iter_mut().rev() {
        if service.child.try_wait().ok().flatten().is_none() {
            kill_child(&mut service.child);
            let _ = service.child.wait();
            if service.record.status == ProcessStatus::Running
                || service.record.status == ProcessStatus::Ready
            {
                service.record.status = ProcessStatus::Exited;
            }
        }
        // Force-remove the container regardless of how the `docker run` child
        // fared — a detached or wedged container must never outlive the run.
        if let Some(name) = &service.docker_name {
            let _ = Command::new("docker")
                .args(["rm", "-f", name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        // A cluster is an external object, so removing the vat dir does NOT
        // remove it. Delete it explicitly when the run policy says to; keep it
        // for `kubectl` diagnosis otherwise.
        if delete_clusters {
            if let Some(record) = &service.cluster {
                if let Some(backend) = ResolvedBackend::from_name(&record.backend) {
                    let _ = backend.delete(&record.name);
                }
            }
        }
    }
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
        stop_services(&mut services, false);

        let order = std::fs::read_to_string(&order_path).expect("stop order");
        assert_eq!(
            order.lines().collect::<Vec<_>>(),
            vec!["frontend", "backend", "postgres"]
        );
        assert!(services
            .iter()
            .all(|service| service.record.status == ProcessStatus::Exited));
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
            path: temp.path().join("vat.toml"),
            root: temp.path().to_path_buf(),
            digest: String::new(),
        };

        let cache = temp.path().join("image");
        std::fs::create_dir_all(&cache).unwrap();
        // Full cold-prepare path: initdb + seed apply + clean shutdown.
        cold_prepare_service_image(&cfg, &service, ServicePreset::Postgres, &cache)
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
        }
    }

    #[test]
    fn docker_run_command_is_well_formed_and_deterministic() {
        let mut env = BTreeMap::new();
        env.insert("POSTGRES_HOST_AUTH_METHOD".to_string(), "trust".to_string());
        env.insert("POSTGRES_DB".to_string(), "app".to_string());
        let cmd = docker_run_command("vat-abc-pg", "postgres:16", 54321, 5432, &env);
        assert_eq!(
            cmd,
            vec![
                "docker",
                "run",
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
            resolve_preset_runtime(&svc, ServicePreset::Pubsub).unwrap(),
            ResolvedRuntime::Builtin
        ));
        assert!(matches!(
            resolve_preset_runtime(&svc, ServicePreset::FirebaseAuth).unwrap(),
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
        let plan = prepare_builtin_service(&svc, ServicePreset::FirebaseAuth, Path::new("."), &[])
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
                resolve_preset_runtime(&svc, preset).unwrap(),
                ResolvedRuntime::Builtin
            ));
            let plan = prepare_builtin_service(&svc, preset, Path::new("."), &[]).unwrap();
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
            resolve_preset_runtime(&svc, ServicePreset::Postgres).unwrap(),
            ResolvedRuntime::Native
        ));
        svc.runtime = ServiceRuntime::Docker;
        assert!(matches!(
            resolve_preset_runtime(&svc, ServicePreset::Postgres).unwrap(),
            ResolvedRuntime::Docker
        ));
    }

    #[test]
    fn container_name_sanitizes_disallowed_chars() {
        assert_eq!(container_name("vat-5oyh3vc", "pg"), "vat-5oyh3vc-pg");
        assert_eq!(container_name("vat/x", "a b"), "vat-x-a-b");
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
        let command = vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            "trap 'printf \"%s\\n\" \"$VAT_STOP_ID\" >> \"$VAT_STOP_ORDER\"; exit 0' TERM; while :; do sleep 1; done".to_string(),
        ];
        let mut env = BTreeMap::new();
        env.insert("VAT_STOP_ID".to_string(), id.to_string());
        env.insert(
            "VAT_STOP_ORDER".to_string(),
            order_path.to_string_lossy().into_owned(),
        );
        let stdout = root.join(format!("{id}.stdout.log"));
        let stderr = root.join(format!("{id}.stderr.log"));
        let child =
            command_with_logs(&command, root, &env, &stdout, &stderr).expect("service child");
        ServiceHandle {
            record: ServiceRunRecord {
                id: id.to_string(),
                command,
                status: ProcessStatus::Ready,
                preset: None,
                port: None,
                prepare_mode: Some("direct_start".to_string()),
                cache_key: None,
                prepare_duration_ms: Some(0),
                ready_duration_ms: Some(0),
                exported_env: Vec::new(),
                pid: Some(child.id()),
                exit_code: None,
                ready_http: None,
                cluster: None,
                stdout_log: stdout.to_string_lossy().into_owned(),
                stderr_log: stderr.to_string_lossy().into_owned(),
            },
            child,
            timeout_s: 1,
            ready_probe: ReadyProbe::None,
            docker_name: None,
            cluster: None,
        }
    }
}

fn command_with_logs(
    cmd: &[String],
    cwd: &Path,
    env: &std::collections::BTreeMap<String, String>,
    stdout: &Path,
    stderr: &Path,
) -> Result<Child> {
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
fn kill_child(child: &mut Child) {
    let pgid = -(child.id() as i32);
    unsafe {
        libc::kill(pgid, libc::SIGTERM);
    }
    std::thread::sleep(Duration::from_millis(200));
    if child.try_wait().ok().flatten().is_none() {
        unsafe {
            libc::kill(pgid, libc::SIGKILL);
        }
    }
}

#[cfg(not(unix))]
fn kill_child(child: &mut Child) {
    let _ = child.kill();
}

fn http_ready(raw_url: &str) -> Result<bool> {
    let url = url::Url::parse(raw_url).with_context(|| format!("parse ready_http {raw_url}"))?;
    let host = url.host_str().context("ready_http missing host")?;
    let port = url
        .port_or_known_default()
        .context("ready_http missing port")?;
    let addr = (host, port)
        .to_socket_addrs()?
        .next()
        .context("ready_http did not resolve")?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(300))?;
    stream.set_read_timeout(Some(Duration::from_millis(300)))?;
    let path = if url.path().is_empty() {
        "/"
    } else {
        url.path()
    };
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
    )?;
    let mut buf = [0u8; 64];
    let n = stream.read(&mut buf)?;
    let head = String::from_utf8_lossy(&buf[..n]);
    Ok(head.starts_with("HTTP/1.0 2")
        || head.starts_with("HTTP/1.1 2")
        || head.starts_with("HTTP/1.0 3")
        || head.starts_with("HTTP/1.1 3"))
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

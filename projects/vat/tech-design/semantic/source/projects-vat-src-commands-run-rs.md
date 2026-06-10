---
id: vat-source-projects-vat-src-commands-run-rs
summary: Source replay payload for projects/vat/src/commands/run.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: copy-on-write-fork-and-snapshot-lifecycle
    claim: copy-on-write-fork-and-snapshot-lifecycle
    coverage: full
    rationale: "This source replay TD preserves vat's copy-on-write workspace, agent-legible state, resource isolation, and host GPU behavior."
---

# Source TD: projects/vat/src/commands/run.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/commands/run.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Args` | projects/vat/src/commands/run.rs | struct | pub | 36 |  |
| `Target` | projects/vat/src/commands/run.rs | enum | pub | 50 |  |
| `exec` | projects/vat/src/commands/run.rs | function | pub | 62 | exec(args: Args) -> Result<ExitCode> |
## Source
<!-- type: source lang: rust -->

`````rust
//! `vat run` — direct command mode plus vat.toml runner mode.
//!
//! Direct mode (`vat run -- <cmd>`) preserves the original foreground behavior.
//! Runner mode (`vat run [runner-id]`) treats `vat.toml` as the project-local
//! agent test protocol: prepare a COW workspace, run setup, start run-scoped
//! services, wait for readiness, execute the runner, capture evidence, and
//! clean up services.

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use chrono::Utc;
use walkdir::WalkDir;

use crate::config::{
    self, PortSpec, RetentionPolicy, RunnerConfig, RunnerSelectionReason, ServiceConfig,
    ServicePreset, VatConfig,
};
use crate::event::{Event, EventKind};
use crate::gpu;
use crate::overlay;
use crate::sandbox;
use crate::spec::{Base, EnvSpec, GpuRequest, Isolation};
use crate::state::{
    ArtifactRecord, ConfigRef, ProcessStatus, RunRecord, RunnerRunRecord, ServiceRunRecord, Status,
    TestRunEvidence,
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

    let spec = EnvSpec {
        base: Some(spec_base),
        isolation: args.isolation,
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
    let spec = EnvSpec {
        base: Some(Base::Dir(source.clone())),
        workdir: cfg.workspace.workdir.clone(),
        env: cfg.env.clone(),
        isolation: args.isolation,
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
                "state": format!("vat state {} --json", state.id),
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
    for service_id in service_ids {
        let service = cfg.service(service_id)?;
        let plan = prepare_service(vat, cfg, service)?;
        for (key, value) in &plan.env {
            run_env.insert(key.clone(), value.clone());
        }
        service_plans.push(plan);
    }

    for step in &cfg.setup {
        if !config::should_run_setup(&rootfs, step) {
            continue;
        }
        run_setup_step(vat, step, &cwd, logs_dir, &run_env)?;
    }

    let mut services = Vec::new();
    for plan in &service_plans {
        services.push(start_service(vat, plan, &cwd, logs_dir, &run_env)?);
    }

    let readiness = wait_for_services(vat, &mut services);
    if let Err(err) = readiness {
        stop_services(&mut services);
        persist_services(vat, &services)?;
        return Err(err);
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
        procs.push(spawn_runner_process(runner, &cwd, logs_dir, &run_env, single)?);
    }
    let records = wait_runner_processes(procs)?;

    // Worst-wins exit: any negative (timeout/kill) is worst, else max code.
    let code = records
        .iter()
        .map(|r| r.exit_code.unwrap_or(-1))
        .fold(0, |acc, c| {
            if acc < 0 || c < 0 {
                -1
            } else {
                acc.max(c)
            }
        });
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
    stop_services(&mut services);
    persist_services(vat, &services)?;
    let summary = records
        .iter()
        .map(|r| format!("{} exited {}", r.id, r.exit_code.unwrap_or(-1)))
        .collect::<Vec<_>>()
        .join("; ");
    vat.log(Event::new(EventKind::RunFinished, summary))?;
    Ok(code)
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
            return Ok(records.into_iter().map(|r| r.expect("all recorded")).collect());
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
}

fn prepare_service(
    vat: &store::Vat,
    cfg: &VatConfig,
    service: &ServiceConfig,
) -> Result<ServicePlan> {
    let started = Instant::now();
    let plan = if let Some(preset) = service.preset {
        prepare_preset_service(vat, cfg, service, preset)?
    } else {
        let env = export_command_service_env(service);
        ServicePlan {
            id: service.id.clone(),
            command: service.cmd.clone(),
            ready_http: service.ready_http.clone(),
            ready_probe: service
                .ready_http
                .clone()
                .map(ReadyProbe::Http)
                .unwrap_or(ReadyProbe::None),
            timeout_s: service.timeout_s,
            preset: None,
            port: None,
            prepare_mode: "direct_start".to_string(),
            cache_key: None,
            prepare_duration_ms: 0,
            exported_env: sorted_keys(&env),
            env,
        }
    };
    let mut plan = plan;
    plan.prepare_duration_ms = started.elapsed().as_millis() as u64;
    if plan.preset.is_some() {
        emit_jsonl(serde_json::json!({
            "type": "prepare",
            "service": plan.id.as_str(),
            "preset": plan.preset.map(service_preset_name),
            "mode": plan.prepare_mode.as_str(),
            "cache_key": plan.cache_key.as_deref(),
            "note": if plan.prepare_mode == "cold_build" {
                "first run slower; cached for future runs"
            } else {
                "using cached service image"
            },
        }))?;
    }
    Ok(plan)
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
            std::fs::create_dir_all(&cache_dir)
                .with_context(|| format!("create {}", cache_dir.display()))?;
            cold_prepare_service_image(service, preset, &cache_dir)?;
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
    Ok(ServicePlan {
        id: service.id.clone(),
        command,
        ready_http: service.ready_http.clone(),
        ready_probe: preset_ready_probe(preset, port),
        timeout_s: service.timeout_s,
        preset: Some(preset),
        port: Some(port),
        prepare_mode: prepare_mode.to_string(),
        cache_key: Some(cache_key),
        prepare_duration_ms: 0,
        exported_env: sorted_keys(&env),
        env,
    })
}

fn cold_prepare_service_image(
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
        }
        ServicePreset::Mysql
        | ServicePreset::Mongo
        | ServicePreset::Rabbitmq
        | ServicePreset::Redis
        | ServicePreset::Nats => {}
    }
    Ok(())
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
        ServicePreset::Postgres => &["postgres", "initdb", "pg_isready"],
        ServicePreset::Redis => &["redis-server"],
        ServicePreset::Nats => &["nats-server"],
        ServicePreset::Rabbitmq => &["rabbitmq-server"],
        ServicePreset::Mysql => &["mysqld", "mysqladmin"],
        ServicePreset::Mongo => &["mongod"],
    }
}

fn preset_uses_service_image(preset: ServicePreset) -> bool {
    matches!(
        preset,
        ServicePreset::Postgres
            | ServicePreset::Mysql
            | ServicePreset::Mongo
            | ServicePreset::Rabbitmq
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
    }
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
        ServicePreset::Redis
        | ServicePreset::Nats
        | ServicePreset::Mongo
        | ServicePreset::Rabbitmq => ReadyProbe::Tcp {
            host: "127.0.0.1".to_string(),
            port,
        },
    }
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
                service.record.ready_duration_ms = Some(started.elapsed().as_millis() as u64);
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

fn stop_services(services: &mut [ServiceHandle]) {
    for service in services {
        if service.child.try_wait().ok().flatten().is_some() {
            continue;
        }
        kill_child(&mut service.child);
        let _ = service.child.wait();
        if service.record.status == ProcessStatus::Running
            || service.record.status == ProcessStatus::Ready
        {
            service.record.status = ProcessStatus::Exited;
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
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/commands/run.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-commands.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-commands-run-rs-source-replay-superseded>"
```

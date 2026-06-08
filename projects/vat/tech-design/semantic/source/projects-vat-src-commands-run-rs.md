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
//! Runner mode (`vat run <runner-id>`) treats `vat.toml` as the project-local
//! agent test protocol: prepare a COW workspace, run setup, start run-scoped
//! services, wait for readiness, execute the runner, capture evidence, and
//! clean up services.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use chrono::Utc;
use walkdir::WalkDir;

use crate::config::{self, RetentionPolicy, RunnerConfig, ServiceConfig, VatConfig};
use crate::event::{Event, EventKind};
use crate::gpu;
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
    /// Print full VatState JSON instead of a human summary.
    pub json: bool,
}

/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#cli
pub enum Target {
    Direct {
        program: String,
        program_args: Vec<String>,
    },
    Runner {
        runner_id: String,
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
        Target::Runner { runner_id } => exec_runner(RunnerArgs {
            base,
            from,
            name,
            isolation,
            gpu,
            json,
            runner_id,
        }),
    }
}

struct RunnerArgs {
    base: Option<PathBuf>,
    from: Option<String>,
    name: Option<String>,
    isolation: Isolation,
    gpu: GpuRequest,
    json: bool,
    runner_id: String,
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

    Ok(ExitCode::from(code.clamp(0, 255) as u8))
}

fn exec_runner(args: RunnerArgs) -> Result<ExitCode> {
    let cwd = std::env::current_dir().context("get current directory")?;
    let cfg = config::load_nearest(&cwd)?;
    if args.base.is_some() || args.from.is_some() {
        bail!(
            "vat run <runner-id> uses vat.toml workspace.base; --base/--from are direct mode only"
        );
    }
    let runner = cfg.runner(&args.runner_id)?.clone();
    let gpu_info = gpu::detect();
    if args.gpu == GpuRequest::Required && !gpu_info.accessible {
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
        .or(Some(runner.id.clone()));
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
        runner_id: runner.id.clone(),
        retention: cfg.workspace.keep,
        services: Vec::new(),
        runner: None,
        artifacts: Vec::new(),
    });
    vat.save()?;
    vat.log(Event::new(
        EventKind::RunStarted,
        format!("runner: {}", runner.id),
    ))?;

    let result = run_configured(&mut vat, &cfg, &runner, &logs_dir);
    let code = match result {
        Ok(code) => code,
        Err(err) => {
            record_runner_failure(&mut vat, &runner, &logs_dir, &err.to_string())?;
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

    if args.json {
        crate::commands::print_json(&state, false)?;
    } else {
        println!(
            "{} · runner {} exited {} · retention {:?}",
            state.id, runner.id, code, cfg.workspace.keep
        );
        println!("→ vat state {}", state.id);
    }

    if should_remove {
        let _ = store::remove(&state.id);
    }

    Ok(ExitCode::from(code.clamp(0, 255) as u8))
}

fn run_configured(
    vat: &mut store::Vat,
    cfg: &VatConfig,
    runner: &RunnerConfig,
    logs_dir: &Path,
) -> Result<i32> {
    let rootfs = vat.rootfs();
    let cwd = rootfs.join(&vat.meta.spec.workdir);
    std::fs::create_dir_all(&cwd).with_context(|| format!("create {}", cwd.display()))?;

    for step in &cfg.setup {
        if !config::should_run_setup(&rootfs, step) {
            continue;
        }
        run_setup_step(vat, step, &cwd, logs_dir)?;
    }

    let mut services = Vec::new();
    for service_id in &runner.requires {
        let service = cfg.service(service_id)?;
        services.push(start_service(vat, service, &cwd, logs_dir)?);
    }

    let readiness = wait_for_services(vat, &mut services);
    if let Err(err) = readiness {
        stop_services(&mut services);
        persist_services(vat, &services)?;
        return Err(err);
    }
    persist_services(vat, &services)?;

    let runner_record = run_runner_process(vat, runner, &cwd, logs_dir)?;
    let code = runner_record.exit_code.unwrap_or(-1);
    if let Some(test_run) = vat.meta.test_run.as_mut() {
        test_run.runner = Some(runner_record);
        test_run.artifacts = collect_artifacts(&rootfs, &runner.artifacts)?;
    }
    vat.save()?;
    stop_services(&mut services);
    persist_services(vat, &services)?;
    vat.log(Event::new(
        EventKind::RunFinished,
        format!("runner {} exited {code}", runner.id),
    ))?;
    Ok(code)
}

fn run_setup_step(
    vat: &store::Vat,
    step: &crate::config::SetupStep,
    cwd: &Path,
    logs_dir: &Path,
) -> Result<()> {
    let stdout = logs_dir.join(format!("setup-{}.stdout.log", step.id));
    let stderr = logs_dir.join(format!("setup-{}.stderr.log", step.id));
    let status = command_with_logs(&step.cmd, cwd, &vat.meta.spec.env, &stdout, &stderr)?
        .wait()
        .with_context(|| format!("wait setup `{}`", step.id))?;
    if !status.success() {
        bail!("setup `{}` failed with {:?}", step.id, status.code());
    }
    vat.log(Event::new(EventKind::Setup, format!("setup {}", step.id)))?;
    Ok(())
}

struct ServiceHandle {
    record: ServiceRunRecord,
    child: Child,
    timeout_s: u64,
}

fn start_service(
    vat: &mut store::Vat,
    service: &ServiceConfig,
    cwd: &Path,
    logs_dir: &Path,
) -> Result<ServiceHandle> {
    let stdout = logs_dir.join(format!("{}.stdout.log", service.id));
    let stderr = logs_dir.join(format!("{}.stderr.log", service.id));
    let child = command_with_logs(&service.cmd, cwd, &vat.meta.spec.env, &stdout, &stderr)
        .with_context(|| format!("spawn service `{}`", service.id))?;
    let record = ServiceRunRecord {
        id: service.id.clone(),
        command: service.cmd.clone(),
        status: ProcessStatus::Running,
        pid: Some(child.id()),
        exit_code: None,
        ready_http: service.ready_http.clone(),
        stdout_log: stdout.to_string_lossy().into_owned(),
        stderr_log: stderr.to_string_lossy().into_owned(),
    };
    vat.log(Event::new(
        EventKind::RunStarted,
        format!("service {}", service.id),
    ))?;
    Ok(ServiceHandle {
        record,
        child,
        timeout_s: service.timeout_s,
    })
}

fn wait_for_services(vat: &mut store::Vat, services: &mut [ServiceHandle]) -> Result<()> {
    for service in services {
        let Some(url) = service.record.ready_http.clone() else {
            service.record.status = ProcessStatus::Ready;
            continue;
        };
        let deadline = Instant::now() + Duration::from_secs(service.timeout_s);
        loop {
            if http_ready(&url).unwrap_or(false) {
                service.record.status = ProcessStatus::Ready;
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
    }
    Ok(())
}

fn run_runner_process(
    vat: &store::Vat,
    runner: &RunnerConfig,
    cwd: &Path,
    logs_dir: &Path,
) -> Result<RunnerRunRecord> {
    let stdout = logs_dir.join("runner.stdout.log");
    let stderr = logs_dir.join("runner.stderr.log");
    let started = Instant::now();
    let mut child = command_with_logs(&runner.cmd, cwd, &vat.meta.spec.env, &stdout, &stderr)
        .with_context(|| format!("spawn runner `{}`", runner.id))?;
    let status = wait_child(&mut child, runner.timeout_s)?;
    let duration_ms = started.elapsed().as_millis() as u64;
    let exit_code = status;
    Ok(RunnerRunRecord {
        id: runner.id.clone(),
        command: runner.cmd.clone(),
        status: ProcessStatus::Exited,
        exit_code: Some(exit_code),
        duration_ms: Some(duration_ms),
        stdout_log: stdout.to_string_lossy().into_owned(),
        stderr_log: stderr.to_string_lossy().into_owned(),
    })
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

fn wait_child(child: &mut Child, timeout_s: Option<u64>) -> Result<i32> {
    let deadline = timeout_s.map(|s| Instant::now() + Duration::from_secs(s));
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status.code().unwrap_or(-1));
        }
        if let Some(deadline) = deadline {
            if Instant::now() >= deadline {
                kill_child(child);
                let _ = child.wait();
                return Ok(-1);
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

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

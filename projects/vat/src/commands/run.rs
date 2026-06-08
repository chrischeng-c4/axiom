// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-commands-run-rs.md#source
// CODEGEN-BEGIN
//! `vat run` — create a fresh vat and run a command inside it.
//!
//! Flow: resolve the base (a host dir, or another vat to fork) → COW-clone it
//! into a new rootfs → run the command via the chosen sandbox with live stdio
//! → record the run, recompute the filesystem diff, and either print a compact
//! human summary or the full [`VatState`] JSON. The child's exit code is
//! forwarded so `vat run -- pytest` behaves in scripts like `pytest` would.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use chrono::Utc;

use crate::event::{Event, EventKind};
use crate::gpu;
use crate::sandbox;
use crate::spec::{Base, EnvSpec, GpuRequest, Isolation};
use crate::state::{RunRecord, Status};
use crate::{id, store};

/// Inputs for `vat run`, already parsed by the CLI layer.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-commands-run-rs.md#source
pub struct Args {
    /// The program to run and its arguments.
    pub program: String,
    pub program_args: Vec<String>,
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

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-commands-run-rs.md#source
pub fn exec(args: Args) -> Result<ExitCode> {
    // 1. GPU pre-flight — fail fast if the spec requires a GPU we can't see.
    let gpu_info = gpu::detect();
    if args.gpu == GpuRequest::Required && !gpu_info.accessible {
        bail!(
            "spec requires a GPU but none is accessible on this host ({})",
            gpu_info.note
        );
    }

    // 2. Resolve the workspace source and lineage.
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

    // 3. Create the vat (clones source → rootfs, captures diff baseline).
    let new_id = id::fresh();
    let mut vat = store::create(
        &new_id,
        args.name.clone(),
        spec.clone(),
        Some(&source),
        lineage,
    )
    .context("create vat")?;

    // 4. Mark running, record the command.
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

    // 5. Resolve the command for the backend, then run it inside the vat's
    //    isolated workspace. Scheduling is deliberately external to vat.
    let rootfs = vat.rootfs();
    let (prog, argv) = backend.resolve(&rootfs, &args.program, &args.program_args);
    let cwd = rootfs.join(&spec.workdir);
    let started = Instant::now();
    let mut cmd = std::process::Command::new(&prog);
    cmd.args(&argv).current_dir(&cwd);
    for (key, value) in &spec.env {
        cmd.env(key, value);
    }
    let status = cmd
        .status()
        .with_context(|| format!("spawn `{prog}` inside vat rootfs"))?;
    let duration_ms = started.elapsed().as_millis() as u64;
    let code = status.code().unwrap_or(-1);

    // 6. Record the result and recompute the diff.
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

    // 7. Report.
    if args.json {
        crate::commands::print_json(&vat.project()?, false)?;
    } else {
        print_summary(&vat, code, duration_ms, &changes, backend.name(), &gpu_info);
    }

    Ok(ExitCode::from(code.clamp(0, 255) as u8))
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

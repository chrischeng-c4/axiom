//! Compose lifecycle orchestration: import/up/down/ps/logs for docker-compose projects.
//!
//! Manages a registry at `root/compose/<project>/project.json` to track
//! running compose projects, their vat_id, and service list. Up runs in two
//! modes: foreground (poll in-process, then run), or --detach (re-exec self).

use crate::cli::ComposeCmd;
use crate::config::ServiceRuntime;
use crate::spec::{GpuRequest, Isolation};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

/// Compose project registry entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComposeRecord {
    project: String,
    vat_id: Option<String>,
    service_ids: Vec<String>,
    status: String, // starting, started, running
    created_at: String,
}

/// Id of the synthesized runner every imported project gets (see `compose::materialize`).
const RUNNER_ID: &str = "project.up";

/// Main dispatch for compose subcommands.
pub fn exec(cmd: ComposeCmd) -> Result<ExitCode> {
    match cmd {
        ComposeCmd::Import {
            file,
            project,
            runtime,
        } => import_cmd(file, project, runtime),
        ComposeCmd::Up { project, detach } => up_cmd(project, detach),
        ComposeCmd::Down { project } => down_cmd(project),
        ComposeCmd::Ps { project } => ps_cmd(project),
        ComposeCmd::Logs { project, service } => logs_cmd(project, service),
    }
}

/// Import a compose file as a vat.toml project.
fn import_cmd(
    file: PathBuf,
    project: Option<String>,
    runtime: ServiceRuntime,
) -> Result<ExitCode> {
    let compose_file = crate::compose::parse(&file)?;

    let project_name = if let Some(p) = project {
        sanitize_project_name(&p)
    } else {
        file.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(sanitize_project_name)
            .ok_or_else(|| anyhow::anyhow!("cannot infer project name from compose file path"))?
    };

    let services = crate::compose::expand(&compose_file, &project_name, runtime)?;
    let service_ids: Vec<String> = services.iter().map(|s| s.id.clone()).collect();

    let registry_dir = registry_dir_for_project(&project_name)?;
    fs::create_dir_all(&registry_dir)
        .with_context(|| format!("create registry dir {}", registry_dir.display()))?;
    let vat_toml = registry_dir.join("vat.toml");
    crate::compose::materialize(&services, &vat_toml)?;

    write_registry(
        &registry_dir,
        &ComposeRecord {
            project: project_name.clone(),
            vat_id: None,
            service_ids,
            status: "imported".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        },
    )?;

    println!(
        "Imported compose project `{project_name}` -> {}",
        vat_toml.display()
    );
    Ok(ExitCode::SUCCESS)
}

/// Start a compose project (foreground or detached).
fn up_cmd(project: Option<String>, detach: bool) -> Result<ExitCode> {
    let project_name = sanitize_project_name(
        &project.ok_or_else(|| anyhow::anyhow!("--project required for up"))?,
    );
    let registry_dir = registry_dir_for_project(&project_name)?;
    let vat_toml = registry_dir.join("vat.toml");
    if !vat_toml.exists() {
        bail!(
            "no imported compose project `{project_name}` -- run `vat compose import` first"
        );
    }

    let mut record = read_registry(&registry_dir).unwrap_or(ComposeRecord {
        project: project_name.clone(),
        vat_id: None,
        service_ids: Vec::new(),
        status: "imported".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    });
    record.status = "starting".to_string();
    record.vat_id = None;
    write_registry(&registry_dir, &record)?;

    if detach {
        Command::new(std::env::current_exe()?)
            .arg("run")
            .arg(RUNNER_ID)
            .arg("--name")
            .arg(&project_name)
            .current_dir(&registry_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("spawn detached `vat run`")?;

        record.vat_id = poll_for_vat_id(&project_name, Duration::from_secs(10));
        record.status = if record.vat_id.is_some() {
            "started"
        } else {
            "starting"
        }
        .to_string();
        write_registry(&registry_dir, &record)?;

        crate::commands::print_json(
            &serde_json::json!({
                "project": project_name,
                "vat_id": record.vat_id,
                "status": record.status,
            }),
            true,
        )?;
        return Ok(ExitCode::SUCCESS);
    }

    // Foreground: poll for vat_id on a background thread while the runner
    // blocks below in commands::run::exec.
    let poll_project = project_name.clone();
    let poll_registry_dir = registry_dir.clone();
    std::thread::spawn(move || {
        if let Some(vat_id) = poll_for_vat_id(&poll_project, Duration::from_secs(10)) {
            if let Ok(mut record) = read_registry(&poll_registry_dir) {
                record.vat_id = Some(vat_id);
                record.status = "running".to_string();
                let _ = write_registry(&poll_registry_dir, &record);
            }
        }
    });

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
    })
}

/// Stop a running compose project.
fn down_cmd(project: String) -> Result<ExitCode> {
    let project_name = sanitize_project_name(&project);
    let registry_dir = registry_dir_for_project(&project_name)?;
    let record = read_registry(&registry_dir)
        .with_context(|| format!("no compose project `{project_name}` in registry"))?;

    let Some(vat_id) = record.vat_id.clone() else {
        remove_registry(&registry_dir)?;
        bail!("compose project `{project_name}` has no running vat_id yet -- still starting?");
    };

    let vat = crate::store::load(&vat_id)
        .with_context(|| format!("load vat {vat_id} for compose project `{project_name}`"))?;

    let pid = vat
        .meta
        .test_run
        .as_ref()
        .and_then(|tr| tr.runner.as_ref())
        .and_then(|r| r.pid);

    match pid {
        Some(pid) => {
            #[cfg(unix)]
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
            println!("Sent SIGTERM to runner pid {pid} for compose project `{project_name}`");
        }
        None => {
            println!("compose project `{project_name}` runner already exited");
        }
    }

    remove_registry(&registry_dir)?;
    Ok(ExitCode::SUCCESS)
}

/// List services in a compose project.
fn ps_cmd(project: String) -> Result<ExitCode> {
    let project_name = sanitize_project_name(&project);
    let registry_dir = registry_dir_for_project(&project_name)?;
    let record = read_registry(&registry_dir)
        .with_context(|| format!("no compose project `{project_name}` in registry"))?;

    let Some(vat_id) = record.vat_id.clone() else {
        println!("compose project `{project_name}` is still starting (no vat_id yet)");
        return Ok(ExitCode::SUCCESS);
    };

    let vat = crate::store::load(&vat_id)
        .with_context(|| format!("load vat {vat_id} for compose project `{project_name}`"))?;

    let Some(test_run) = vat.meta.test_run.as_ref() else {
        println!("compose project `{project_name}` has no runner evidence yet");
        return Ok(ExitCode::SUCCESS);
    };

    for service in &test_run.services {
        if record.service_ids.contains(&service.id) {
            println!(
                "{}\t{:?}\t{}",
                service.id,
                service.status,
                service
                    .port
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "-".to_string())
            );
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// Print logs from a service in a compose project.
fn logs_cmd(project: String, service: String) -> Result<ExitCode> {
    let project_name = sanitize_project_name(&project);
    let registry_dir = registry_dir_for_project(&project_name)?;
    let record = read_registry(&registry_dir)
        .with_context(|| format!("no compose project `{project_name}` in registry"))?;

    let Some(vat_id) = record.vat_id.clone() else {
        bail!("compose project `{project_name}` is still starting (no vat_id yet)");
    };

    if !record.service_ids.contains(&service) {
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

    print_file(&svc.stdout_log)?;
    print_file(&svc.stderr_log)?;
    Ok(ExitCode::SUCCESS)
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

/// Poll the vat store for a vat matching `--name <project>`, returning its id
/// once found or `None` after `timeout` elapses.
fn poll_for_vat_id(project_name: &str, timeout: Duration) -> Option<String> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Ok(vats) = crate::store::list() {
            let mut matches: Vec<_> = vats
                .into_iter()
                .filter(|v| v.meta.name.as_deref() == Some(project_name))
                .collect();
            matches.sort_by_key(|v| v.meta.created_at);
            if let Some(v) = matches.pop() {
                return Some(v.meta.id);
            }
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
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
    let content = fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    let record = serde_json::from_str(&content)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(record)
}

/// Write the compose registry entry for a project.
fn write_registry(registry_dir: &Path, record: &ComposeRecord) -> Result<()> {
    fs::create_dir_all(registry_dir)?;
    let path = registry_dir.join("project.json");
    let json = serde_json::to_string_pretty(record)?;
    fs::write(path, json)?;
    Ok(())
}

/// Remove the compose registry entry for a project.
fn remove_registry(registry_dir: &Path) -> Result<()> {
    let path = registry_dir.join("project.json");
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
    }
    Ok(())
}

/// Sanitize a project name (simple alphanumeric + dash/underscore).
fn sanitize_project_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase()
}

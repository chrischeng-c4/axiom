//! Spawn a child command and report its PID before waiting.
//!
//! The only resource knob applied here is `nice` — memory pressure is handled
//! by the daemon via live SIGSTOP / SIGCONT / SIGKILL on the child PID. The
//! child is placed in its own process group so the throttler can signal the
//! whole subtree (`cargo test → rustc → …`) via `kill(-pid, sig)`.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Stdio;

use anyhow::{anyhow, Context, Result};
use tokio::process::{Child, Command};

pub struct SpawnOpts {
    pub nice: i32,
}

/// Everything needed to launch the child. Generalized from "(program, args)"
/// so callers that wrap the command (vat's seatbelt backend) or need a cwd /
/// extra env (vat's per-vat workspace) can express it.
#[derive(Debug, Clone, Default)]
pub struct SpawnSpec {
    pub program: String,
    pub args: Vec<String>,
    /// Working directory for the child (default: inherit caller's cwd).
    pub cwd: Option<PathBuf>,
    /// Extra environment variables layered onto the inherited environment.
    pub env: BTreeMap<String, String>,
}

impl SpawnSpec {
    pub fn new(program: impl Into<String>, args: Vec<String>) -> Self {
        SpawnSpec {
            program: program.into(),
            args,
            cwd: None,
            env: BTreeMap::new(),
        }
    }
}

/// A spawned-but-not-yet-waited child. Callers must read `pid` first
/// (to register it with the daemon), then call `wait()`.
pub struct RunningChild {
    pub pid: i32,
    child: Child,
}

impl RunningChild {
    pub async fn wait(mut self) -> Result<std::process::ExitStatus> {
        self.child.wait().await.context("waiting for child to exit")
    }
}

/// Spawn from a [`SpawnSpec`], inheriting stdio, in its own process group.
pub fn spawn(spec: &SpawnSpec, opts: SpawnOpts) -> Result<RunningChild> {
    let mut cmd = Command::new(&spec.program);
    cmd.args(&spec.args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if let Some(dir) = &spec.cwd {
        cmd.current_dir(dir);
    }
    for (k, v) in &spec.env {
        cmd.env(k, v);
    }

    let nice = opts.nice;
    unsafe {
        cmd.pre_exec(move || {
            // Make the child the leader of a new process group. PGID =
            // child's PID, so the throttler can target the whole subtree by
            // signalling `kill(-pid, sig)`. Without this, SIGSTOP/SIGKILL on
            // the immediate child leaks: grandchildren keep eating RAM until
            // reparented to init.
            if libc::setpgid(0, 0) != 0 {
                let _ = std::io::Error::last_os_error();
            }
            apply_priority(nice)
        });
    }

    let child = cmd
        .spawn()
        .with_context(|| format!("spawning {}", spec.program))?;
    let pid = child.id().ok_or_else(|| anyhow!("spawn produced no PID"))? as i32;
    Ok(RunningChild { pid, child })
}

fn apply_priority(nice: i32) -> std::io::Result<()> {
    if nice != 0 {
        unsafe {
            // setpriority can fail (EPERM if lowering nice as non-root);
            // not fatal — child still runs.
            let _ = libc::setpriority(libc::PRIO_PROCESS, 0, nice);
        }
    }
    Ok(())
}

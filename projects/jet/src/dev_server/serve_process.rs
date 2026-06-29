// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
//! Detached `jet serve` process lifecycle shared by CLI and E2E.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use super::session::{self, ServeSession};

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
#[derive(Debug, Clone)]
pub struct ServeProcessOptions {
    pub root_dir: PathBuf,
    pub host: String,
    pub port: u16,
    pub prod: bool,
    pub wasm: bool,
    pub debug: bool,
    pub ready_timeout: Duration,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl ServeProcessOptions {
    pub fn dom_dev(root_dir: PathBuf) -> Self {
        Self {
            root_dir,
            host: "127.0.0.1".to_string(),
            port: 0,
            prod: false,
            wasm: false,
            debug: false,
            ready_timeout: Duration::from_secs(30),
        }
    }

    pub fn dom_prod(root_dir: PathBuf) -> Self {
        Self {
            prod: true,
            ..Self::dom_dev(root_dir)
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
#[derive(Debug, Clone)]
pub struct ServeLaunch {
    pub session: ServeSession,
    pub session_file: PathBuf,
    pub log_file: PathBuf,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServeLaunchPayload {
    pub schema_version: String,
    pub mode: String,
    pub target: String,
    pub url: String,
    pub host: String,
    pub port: u16,
    pub pid: u32,
    pub session_file: String,
    pub log_file: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl ServeLaunch {
    pub fn payload(&self) -> ServeLaunchPayload {
        ServeLaunchPayload {
            schema_version: self.session.schema_version.clone(),
            mode: self.session.mode.clone(),
            target: self.session.target.clone(),
            url: self.session.url.clone(),
            host: self.session.host.clone(),
            port: self.session.port,
            pid: self.session.pid,
            session_file: self.session_file.display().to_string(),
            log_file: self.log_file.display().to_string(),
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn serve_session_target(prod: bool, wasm: bool) -> &'static str {
    match (prod, wasm) {
        (true, true) => session::TARGET_WASM_PROD,
        (true, false) => session::TARGET_DOM_PROD,
        (false, true) => session::TARGET_WASM,
        (false, false) => session::TARGET_DOM,
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub async fn launch_detached(opts: ServeProcessOptions) -> Result<ServeLaunch> {
    let jet_dir = opts.root_dir.join(".jet");
    std::fs::create_dir_all(&jet_dir).with_context(|| format!("creating {}", jet_dir.display()))?;

    let session_file = session::session_path(&opts.root_dir);
    let log_file = session::log_path(&opts.root_dir);
    session::clear(&opts.root_dir);

    let log = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_file)
        .with_context(|| format!("opening {}", log_file.display()))?;
    let log_for_stderr = log
        .try_clone()
        .with_context(|| format!("cloning {}", log_file.display()))?;

    let exe = std::env::current_exe().context("resolving current jet executable")?;
    let mut cmd = std::process::Command::new(exe);
    cmd.current_dir(&opts.root_dir);
    if opts.prod {
        // `jet serve` is the production static surface; the detached child is
        // signaled via JET_SERVE_CHILD (no `--prod` flag on the public surface).
        cmd.arg("serve");
        if opts.wasm {
            cmd.arg("--wasm");
        }
        cmd.env("JET_SERVE_CHILD", "1");
    } else {
        cmd.arg("dev");
        if opts.wasm {
            cmd.arg("--wasm");
            if opts.debug {
                cmd.arg("--debug");
            }
        }
    }
    cmd.arg("--host")
        .arg(&opts.host)
        .arg("--port")
        .arg(opts.port.to_string())
        .env("JET_SERVE_SESSION_FILE", &session_file)
        .env("JET_SERVE_SESSION_MODE", session::MODE_DETACHED)
        .env(
            "JET_SERVE_SESSION_TARGET",
            serve_session_target(opts.prod, opts.wasm),
        )
        .env("JET_SERVE_LOG_FILE", &log_file)
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_for_stderr));

    let mut child = cmd.spawn().context("spawning detached jet serve process")?;
    let deadline = Instant::now() + opts.ready_timeout;
    loop {
        if let Ok(session) = session::read(&opts.root_dir) {
            return Ok(ServeLaunch {
                session,
                session_file,
                log_file,
            });
        }

        if let Some(status) = child.try_wait().context("checking detached serve child")? {
            anyhow::bail!(
                "jet serve child exited before writing {}; status={status}; log:\n{}",
                session_file.display(),
                read_log_tail(&log_file)
            );
        }

        if Instant::now() >= deadline {
            let _ = child.kill();
            anyhow::bail!(
                "jet serve timed out waiting for {}; log:\n{}",
                session_file.display(),
                read_log_tail(&log_file)
            );
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub async fn shutdown_active(root_dir: &Path) -> Result<(ServeSession, String)> {
    let active = session::read(root_dir)?;
    let body = shutdown_host_port(&active.host, active.port).await?;
    session::clear(root_dir);
    Ok((active, body))
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub async fn shutdown_host_port(host: &str, port: u16) -> Result<String> {
    let url = shutdown_url(host, port);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .context("creating dev shutdown HTTP client")?;
    let response = client
        .post(&url)
        .send()
        .await
        .with_context(|| format!("requesting Jet dev shutdown at {url}"))?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        anyhow::bail!(
            "jet dev shutdown failed at {url}: status={status} body={}",
            body.trim()
        );
    }
    Ok(body)
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn shutdown_url(host: &str, port: u16) -> String {
    format!("http://{host}:{port}/__jet_shutdown")
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn read_log_tail(path: &Path) -> String {
    let Ok(body) = std::fs::read_to_string(path) else {
        return String::new();
    };
    let mut lines = body.lines().rev().take(40).collect::<Vec<_>>();
    lines.reverse();
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn target_names_cover_dom_wasm_and_prod_modes() {
        assert_eq!(serve_session_target(false, false), session::TARGET_DOM);
        assert_eq!(serve_session_target(false, true), session::TARGET_WASM);
        assert_eq!(serve_session_target(true, false), session::TARGET_DOM_PROD);
        assert_eq!(serve_session_target(true, true), session::TARGET_WASM_PROD);
    }

    #[test]
    fn launch_payload_is_agent_readable() {
        let dir = tempdir().unwrap();
        let launch = ServeLaunch {
            session: ServeSession {
                schema_version: session::SCHEMA_VERSION.to_string(),
                mode: session::MODE_DETACHED.to_string(),
                target: session::TARGET_DOM.to_string(),
                host: "127.0.0.1".to_string(),
                port: 43127,
                url: "http://127.0.0.1:43127/".to_string(),
                pid: 123,
                root_dir: dir.path().display().to_string(),
                log_file: Some(session::log_path(dir.path()).display().to_string()),
                started_at: session::now_unix(),
            },
            session_file: session::session_path(dir.path()),
            log_file: session::log_path(dir.path()),
        };

        let payload = launch.payload();
        assert_eq!(payload.target, session::TARGET_DOM);
        assert_eq!(payload.url, "http://127.0.0.1:43127/");
        assert!(payload.session_file.ends_with(".jet/serve-session.json"));
        assert!(payload.log_file.ends_with(".jet/serve.log"));
    }
}
// CODEGEN-END

// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-session.md#schema
// CODEGEN-BEGIN
//! Persistent `jet dev` lifecycle state shared by DOM and WASM dev servers.
//!
//! `jet dev` writes `.jet/dev-session.json` once the server is listening.
//! `jet dev shutdown` writes `.jet/dev-shutdown-request`; the running dev
//! server polls that file and exits through Axum graceful shutdown. Tests use
//! this surface so lifecycle coverage does not rely on killing child processes.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-session.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DevSessionMode {
    Dom,
    Wasm,
    WasmDebug,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-session.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevSession {
    pub mode: DevSessionMode,
    pub url: String,
    pub host: String,
    pub port: u16,
    pub pid: u32,
    pub started_at: u64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-session.md#schema
pub fn session_path(root_dir: &Path) -> PathBuf {
    root_dir.join(".jet").join("dev-session.json")
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-session.md#schema
pub fn shutdown_request_path(root_dir: &Path) -> PathBuf {
    root_dir.join(".jet").join("dev-shutdown-request")
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-session.md#schema
pub fn write(root_dir: &Path, session: &DevSession) -> Result<()> {
    let path = session_path(root_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let body = serde_json::to_string_pretty(session).context("serializing dev session")?;
    std::fs::write(&path, body).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-session.md#schema
pub fn read(root_dir: &Path) -> Result<DevSession> {
    let path = session_path(root_dir);
    let body = std::fs::read_to_string(&path)
        .with_context(|| format!("no dev session at {} - run `jet dev` first", path.display()))?;
    serde_json::from_str(&body).with_context(|| format!("parsing {}", path.display()))
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-session.md#schema
pub fn clear(root_dir: &Path) {
    let path = session_path(root_dir);
    match std::fs::remove_file(&path) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            tracing::warn!(
                target: "jet::dev_session",
                path = %path.display(),
                error = %err,
                "failed to clear dev session file; a later `jet dev shutdown` may target a stale session"
            );
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-session.md#schema
pub fn request_shutdown(root_dir: &Path) -> Result<()> {
    let path = shutdown_request_path(root_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    std::fs::write(&path, now_unix().to_string())
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-session.md#schema
pub fn shutdown_requested(root_dir: &Path) -> bool {
    shutdown_request_path(root_dir).exists()
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-session.md#schema
pub fn clear_shutdown_request(root_dir: &Path) {
    let path = shutdown_request_path(root_dir);
    match std::fs::remove_file(&path) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            tracing::warn!(
                target: "jet::dev_session",
                path = %path.display(),
                error = %err,
                "failed to clear dev shutdown request file"
            );
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-session.md#schema
pub fn now_unix() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(dur) => dur.as_secs(),
        Err(err) => {
            tracing::warn!(
                target: "jet::dev_session",
                error = %err,
                "SystemTime::now() is before UNIX_EPOCH while writing dev session metadata; using started_at=0"
            );
            0
        }
    }
}

// CODEGEN-END

// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
//! Persistent serve-session state for agent-first `jet serve`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const SCHEMA_VERSION: &str = "jet.serve.session.v1";
pub const MODE_DETACHED: &str = "detached";
pub const TARGET_DOM: &str = "dom";
pub const TARGET_WASM: &str = "wasm";
pub const TARGET_DOM_PROD: &str = "dom-prod";
pub const TARGET_WASM_PROD: &str = "wasm-prod";

const SESSION_FILE_ENV: &str = "JET_SERVE_SESSION_FILE";
const SESSION_MODE_ENV: &str = "JET_SERVE_SESSION_MODE";
const SESSION_TARGET_ENV: &str = "JET_SERVE_SESSION_TARGET";
const SESSION_LOG_ENV: &str = "JET_SERVE_LOG_FILE";

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServeSession {
    pub schema_version: String,
    pub mode: String,
    pub target: String,
    pub host: String,
    pub port: u16,
    pub url: String,
    pub pid: u32,
    pub root_dir: String,
    pub log_file: Option<String>,
    pub started_at: u64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn session_path(root_dir: &Path) -> PathBuf {
    root_dir.join(".jet").join("serve-session.json")
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn log_path(root_dir: &Path) -> PathBuf {
    root_dir.join(".jet").join("serve.log")
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn write(path: &Path, session: &ServeSession) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let body = serde_json::to_string_pretty(session).context("serializing serve session")?;
    std::fs::write(path, body).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn read(root_dir: &Path) -> Result<ServeSession> {
    let path = session_path(root_dir);
    let body = std::fs::read_to_string(&path)
        .with_context(|| format!("no serve session at {}", path.display()))?;
    serde_json::from_str(&body).with_context(|| format!("parsing {}", path.display()))
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn clear(root_dir: &Path) {
    match std::fs::remove_file(session_path(root_dir)) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            tracing::warn!(
                target: "jet::dev_server::session",
                error = %err,
                "failed to clear serve session file"
            );
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn write_from_env(
    root_dir: &Path,
    addr: SocketAddr,
    default_target: &str,
) -> Result<Option<ServeSession>> {
    let Some(path) = std::env::var_os(SESSION_FILE_ENV).map(PathBuf::from) else {
        return Ok(None);
    };

    let mode = std::env::var(SESSION_MODE_ENV).unwrap_or_else(|_| MODE_DETACHED.to_string());
    let target = std::env::var(SESSION_TARGET_ENV).unwrap_or_else(|_| default_target.to_string());
    let log_file = std::env::var(SESSION_LOG_ENV).ok();
    let session = ServeSession {
        schema_version: SCHEMA_VERSION.to_string(),
        mode,
        target,
        host: addr.ip().to_string(),
        port: addr.port(),
        url: format!("http://{addr}/"),
        pid: std::process::id(),
        root_dir: root_dir.display().to_string(),
        log_file,
        started_at: now_unix(),
    };
    write(&path, &session)?;
    Ok(Some(session))
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|dur| dur.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn roundtrips_serve_session_file() {
        let dir = tempdir().unwrap();
        let session = ServeSession {
            schema_version: SCHEMA_VERSION.to_string(),
            mode: MODE_DETACHED.to_string(),
            target: TARGET_DOM.to_string(),
            host: "127.0.0.1".to_string(),
            port: 43127,
            url: "http://127.0.0.1:43127/".to_string(),
            pid: 42,
            root_dir: dir.path().display().to_string(),
            log_file: Some(log_path(dir.path()).display().to_string()),
            started_at: now_unix(),
        };

        write(&session_path(dir.path()), &session).unwrap();
        let back = read(dir.path()).unwrap();
        assert_eq!(back.schema_version, SCHEMA_VERSION);
        assert_eq!(back.mode, MODE_DETACHED);
        assert_eq!(back.target, TARGET_DOM);
        assert_eq!(back.port, 43127);
        assert_eq!(back.log_file, session.log_file);
    }

    #[test]
    fn clear_is_idempotent() {
        let dir = tempdir().unwrap();
        clear(dir.path());
        clear(dir.path());
        assert!(read(dir.path()).is_err());
    }
}
// CODEGEN-END

// SPEC-MANAGED: projects/cap/tech-design/semantic/cap-src.md#schema
// CODEGEN-BEGIN
//! Filesystem locations for cap state.
//!
//! Every cap client resolves the **same** socket and pidfile. That single shared
//! location is what keeps one daemon as the sole arbiter across the cap CLI and
//! anything else that registers leases.

use anyhow::{Context, Result};
use std::path::PathBuf;

/// All cap state lives under `$CAP_HOME` (default `~/.cap`).
/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub fn home() -> Result<PathBuf> {
    if let Ok(custom) = std::env::var("CAP_HOME") {
        return Ok(PathBuf::from(custom));
    }
    let base = dirs::home_dir().context("could not resolve $HOME")?;
    Ok(base.join(".cap"))
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub fn ensure_home() -> Result<PathBuf> {
    let h = home()?;
    std::fs::create_dir_all(&h).with_context(|| format!("creating {}", h.display()))?;
    Ok(h)
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub fn socket_path() -> Result<PathBuf> {
    Ok(home()?.join("cap.sock"))
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub fn pid_path() -> Result<PathBuf> {
    Ok(home()?.join("cap.pid"))
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub fn log_path() -> Result<PathBuf> {
    Ok(home()?.join("cap.log"))
}

/// Directory holding the structured per-command run logs
/// (`logs/events-YYYY-MM-DD.jsonl`). Distinct from `log_path()`, which
/// is the daemon's operational tracing output.
/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub fn logs_dir() -> Result<PathBuf> {
    Ok(home()?.join("logs"))
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub fn lock_path() -> Result<PathBuf> {
    Ok(home()?.join("cap.lock"))
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub fn config_path() -> Result<PathBuf> {
    Ok(home()?.join("config.toml"))
}
// CODEGEN-END

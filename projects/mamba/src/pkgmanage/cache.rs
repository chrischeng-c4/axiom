// `mamba cache` — inspect / clean / prune the package cache root.
//
// Acceptance (tests/governance/gates/pkgmgr/cache/manifest.toml, schema gate
// pkgmgr_cache_isolation_fixture_2685.rs):
//
//   - $MAMBA_CACHE_DIR overrides the platform default — the gate
//     points this at a tempdir so user home is never read or written.
//   - `cache dir` prints the resolved cache root (debug aid).
//   - `cache clean` removes every entry under the root but keeps the
//     root itself; safe to call when the root does not yet exist.
//   - `cache prune` is currently equivalent to `clean` (placeholder
//     for age-based eviction; out of scope here per #2685).
//   - Offline; no implicit network or user-home traversal.

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;

const CACHE_DIR_ENV: &str = "MAMBA_CACHE_DIR";

pub fn cmd_cache(sub: &ArgMatches) -> Result<()> {
    match sub.subcommand() {
        Some(("dir", _)) => action_dir(),
        Some(("clean", _)) => action_clean(),
        Some(("prune", _)) => action_prune(),
        Some((other, _)) => bail!("unknown cache subcommand `{other}`"),
        None => bail!("`mamba cache` requires a subcommand: dir | clean | prune"),
    }
}

fn action_dir() -> Result<()> {
    let root = resolve_cache_root()?;
    // Single line, ends in newline — easy for scripts to consume.
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "{}", root.display()).context("write cache root")?;
    Ok(())
}

fn action_clean() -> Result<()> {
    let root = resolve_cache_root()?;
    if !root.exists() {
        // Idempotent — cleaning a non-existent cache is a no-op.
        eprintln!("no_op: cache root {} did not exist", root.display());
        return Ok(());
    }
    let mut removed = 0usize;
    for entry in fs::read_dir(&root)
        .with_context(|| format!("read {}", root.display()))?
    {
        let entry = entry.with_context(|| format!("walk {}", root.display()))?;
        let path = entry.path();
        let kind = entry
            .file_type()
            .with_context(|| format!("stat {}", path.display()))?;
        if kind.is_dir() {
            fs::remove_dir_all(&path)
                .with_context(|| format!("remove dir {}", path.display()))?;
        } else {
            fs::remove_file(&path)
                .with_context(|| format!("remove file {}", path.display()))?;
        }
        removed += 1;
    }
    eprintln!(
        "cleaned: {removed} entr{plural} under {root}",
        plural = if removed == 1 { "y" } else { "ies" },
        root = root.display()
    );
    Ok(())
}

fn action_prune() -> Result<()> {
    // For now `prune` is identical to `clean` — age-based eviction
    // arrives once there's real wheel cache content (Tick 8+ has
    // hash-addressed entries; eviction policy is a follow-up).
    eprintln!("prune: applying clean policy (age-based eviction not yet enabled)");
    action_clean()
}

/// Resolve the package cache root, in order:
///   1. `$MAMBA_CACHE_DIR` if set.
///   2. `$XDG_CACHE_HOME/mamba` (Linux convention).
///   3. `~/Library/Caches/mamba` on macOS, `~/.cache/mamba` elsewhere.
pub fn resolve_cache_root() -> Result<PathBuf> {
    if let Some(env_root) = std::env::var_os(CACHE_DIR_ENV) {
        let p = PathBuf::from(env_root);
        if p.as_os_str().is_empty() {
            bail!("${CACHE_DIR_ENV} is set but empty");
        }
        return Ok(p);
    }
    if let Some(xdg) = std::env::var_os("XDG_CACHE_HOME") {
        let p = PathBuf::from(xdg);
        if !p.as_os_str().is_empty() {
            return Ok(p.join("mamba"));
        }
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .context("no $HOME and no $MAMBA_CACHE_DIR — pass --cache-dir or set $MAMBA_CACHE_DIR")?;
    #[cfg(target_os = "macos")]
    {
        Ok(home.join("Library").join("Caches").join("mamba"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(home.join(".cache").join("mamba"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Env var mutation is process-global; serialize tests that read it.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env_var<F: FnOnce()>(key: &str, value: Option<&str>, body: F) {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var_os(key);
        unsafe {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
        body();
        unsafe {
            match prev {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }

    #[test]
    fn env_var_takes_priority() {
        let tmp = tempfile::tempdir().unwrap();
        with_env_var(CACHE_DIR_ENV, tmp.path().to_str(), || {
            let root = resolve_cache_root().unwrap();
            assert_eq!(root, tmp.path());
        });
    }

    #[test]
    fn empty_env_var_is_rejected() {
        with_env_var(CACHE_DIR_ENV, Some(""), || {
            assert!(resolve_cache_root().is_err());
        });
    }
}

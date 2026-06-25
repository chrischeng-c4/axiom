// `mamba cache` — inspect / clean / prune the package cache root.
//
// Acceptance (tests/governance/gates/pkgmgr/cache/manifest.toml, schema gate
// pkgmgr_cache_isolation_fixture_2685.rs):
//
//   - $MAMBA_CACHE_DIR overrides the platform default — the gate
//     points this at a tempdir so user home is never read or written.
//   - `cache dir` prints the resolved cache root (debug aid).
//   - `cache size` / `cache info` report exact cache bytes and category
//     counts.
//   - `cache clean` removes every entry under the root but keeps the
//     root itself; safe to call when the root does not yet exist.
//   - `cache prune` supports dry-run, age, total-size, and package
//     targeted policies.
//   - Offline; no implicit network or user-home traversal.

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use std::io::Write as _;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::pkgmanage::pkgmgr::cache_prune::{
    CacheCategory, PrunePolicy, apply_prune_plan, collapse_empty_dirs, enumerate_cache, plan_prune,
};

const CACHE_DIR_ENV: &str = "MAMBA_CACHE_DIR";

pub fn cmd_cache(sub: &ArgMatches) -> Result<()> {
    match sub.subcommand() {
        Some(("dir", _)) => action_dir(),
        Some(("size", cmd)) => action_size(cmd),
        Some(("info", cmd)) => action_info(cmd),
        Some(("clean", _)) => action_clean(),
        Some(("prune", cmd)) => action_prune(cmd),
        Some((other, _)) => bail!("unknown cache subcommand `{other}`"),
        None => bail!("`mamba cache` requires a subcommand: dir | size | info | clean | prune"),
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
    let inv = enumerate_cache(&root)?;
    let mut policy = PrunePolicy::clean_all();
    policy.all_unknown_too = true;
    let plan = plan_prune(&inv, &policy, SystemTime::now());
    let summary = apply_prune_plan(&plan, false);
    let empty_dirs = collapse_empty_dirs(&root);
    report_prune_failures(&summary.failures)?;
    eprintln!(
        "cleaned: {files} file{file_plural}, {bytes} bytes freed, {dirs} empty dir{dir_plural} removed under {root}",
        files = summary.removed,
        file_plural = if summary.removed == 1 { "" } else { "s" },
        bytes = summary.bytes_freed,
        dirs = empty_dirs,
        dir_plural = if empty_dirs == 1 { "" } else { "s" },
        root = root.display()
    );
    Ok(())
}

fn action_size(sub: &ArgMatches) -> Result<()> {
    let root = resolve_cache_root()?;
    let inv = enumerate_cache(&root)?;
    if sub.get_flag("json") {
        println!(
            "{}",
            serde_json::json!({
                "root": root,
                "bytes": inv.total_bytes(),
                "entries": inv.count(),
            })
        );
    } else {
        println!("{} bytes", inv.total_bytes());
    }
    Ok(())
}

fn action_info(sub: &ArgMatches) -> Result<()> {
    let root = resolve_cache_root()?;
    let inv = enumerate_cache(&root)?;
    if sub.get_flag("json") {
        println!(
            "{}",
            serde_json::json!({
                "root": root,
                "entries": inv.count(),
                "bytes": inv.total_bytes(),
                "metadata": category_json(&inv, CacheCategory::Metadata),
                "artifacts": category_json(&inv, CacheCategory::Artifact),
                "content": category_json(&inv, CacheCategory::ContentAddressed),
                "other": category_json(&inv, CacheCategory::Other),
            })
        );
        return Ok(());
    }

    println!("root: {}", root.display());
    println!("entries: {}", inv.count());
    println!("bytes: {}", inv.total_bytes());
    println!(
        "metadata: {} entries, {} bytes",
        inv.count_in(CacheCategory::Metadata),
        inv.bytes_in(CacheCategory::Metadata)
    );
    println!(
        "artifacts: {} entries, {} bytes",
        inv.count_in(CacheCategory::Artifact),
        inv.bytes_in(CacheCategory::Artifact)
    );
    println!(
        "content: {} entries, {} bytes",
        inv.count_in(CacheCategory::ContentAddressed),
        inv.bytes_in(CacheCategory::ContentAddressed)
    );
    println!(
        "other: {} entries, {} bytes",
        inv.count_in(CacheCategory::Other),
        inv.bytes_in(CacheCategory::Other)
    );
    Ok(())
}

fn action_prune(sub: &ArgMatches) -> Result<()> {
    let root = resolve_cache_root()?;
    if !root.exists() {
        eprintln!("no_op: cache root {} did not exist", root.display());
        return Ok(());
    }

    let mut policy = PrunePolicy {
        all_unknown_too: sub.get_flag("all-unknown"),
        ..Default::default()
    };
    if let Some(seconds) = sub.get_one::<String>("older-than-seconds") {
        policy.max_age = Some(Duration::from_secs(parse_u64(
            seconds,
            "older-than-seconds",
        )?));
    }
    if let Some(bytes) = sub.get_one::<String>("max-size") {
        policy.max_total_bytes = Some(parse_u64(bytes, "max-size")?);
    }
    if let Some(packages) = sub.get_many::<String>("package") {
        policy.only_packages = packages.cloned().collect();
    }
    if policy.max_age.is_none()
        && policy.max_total_bytes.is_none()
        && policy.only_packages.is_empty()
    {
        policy.wipe = true;
        policy.all_unknown_too = true;
    }

    let inv = enumerate_cache(&root)?;
    let plan = plan_prune(&inv, &policy, SystemTime::now());
    let dry_run = sub.get_flag("dry-run");
    let summary = apply_prune_plan(&plan, dry_run);
    let empty_dirs = if dry_run {
        0
    } else {
        collapse_empty_dirs(&root)
    };
    report_prune_failures(&summary.failures)?;
    eprintln!(
        "{verb}: {files} file{file_plural}, {bytes} bytes {suffix}, {dirs} empty dir{dir_plural} removed under {root}",
        verb = if dry_run { "would_prune" } else { "pruned" },
        files = summary.removed,
        file_plural = if summary.removed == 1 { "" } else { "s" },
        bytes = summary.bytes_freed,
        suffix = if dry_run { "selected" } else { "freed" },
        dirs = empty_dirs,
        dir_plural = if empty_dirs == 1 { "" } else { "s" },
        root = root.display()
    );
    Ok(())
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

fn parse_u64(raw: &str, name: &str) -> Result<u64> {
    raw.parse::<u64>()
        .with_context(|| format!("parse --{name} value `{raw}`"))
}

fn category_json(
    inv: &crate::pkgmanage::pkgmgr::cache_prune::CacheInventory,
    cat: CacheCategory,
) -> serde_json::Value {
    serde_json::json!({
        "entries": inv.count_in(cat),
        "bytes": inv.bytes_in(cat),
    })
}

fn report_prune_failures(failures: &[(PathBuf, String)]) -> Result<()> {
    if failures.is_empty() {
        return Ok(());
    }
    let mut detail = String::new();
    for (path, err) in failures {
        detail.push_str(&format!("{}: {}; ", path.display(), err));
    }
    bail!(
        "cache prune failed for {} path(s): {detail}",
        failures.len()
    )
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

// `mamba run` package-manager preflight — closes #2684.
//
// Acceptance (tests/governance/gates/pkgmgr/run/manifest.toml, schema gate
// pkgmgr_run_fixture_2684.rs):
//
//   - Running before sync fails with "environment is not synced".
//   - Running after sync proceeds, with `.venv/site-packages` injected
//     into the import path.
//   - No global PATH or user shell env mutation: we set PYTHONPATH on
//     the current process only.
//   - Offline against the frozen local index (sync owns that).
//
// Hook shape: callers invoke `preflight(project_dir)` BEFORE handing
// the file off to the compiler. Legacy `mamba run <file>` outside a
// mamba project is untouched (returns `Mode::Legacy`).

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::pkgmanage::sync::parse_locked_packages;

const MANIFEST_FILE: &str = "mamba.toml";
const LOCKFILE_FILE: &str = "mamba.lock";
const VENV_DIR: &str = ".venv";
const SITE_PACKAGES: &str = "site-packages";

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    /// No mamba.toml here — caller should run the file with legacy
    /// semantics (no env contract).
    Legacy,
    /// mamba.toml present; venv is in sync with the lockfile and
    /// PYTHONPATH has been pointed at `.venv/site-packages`.
    Project { site_packages: PathBuf },
    /// mamba.toml present, lockfile has zero packages, so no env
    /// is required. Caller runs the file with no PYTHONPATH override.
    EmptyLock,
}

/// Project-aware preflight for `mamba run`. Returns the execution
/// mode and, as a side effect, sets `PYTHONPATH` for the current
/// process when the project demands it. Bails when the project has
/// a non-empty lockfile but the env has not been synced.
pub fn preflight(project_dir: &Path) -> Result<Mode> {
    let manifest = project_dir.join(MANIFEST_FILE);
    if !manifest.exists() {
        return Ok(Mode::Legacy);
    }
    let lock_path = project_dir.join(LOCKFILE_FILE);
    if !lock_path.exists() {
        // mamba.toml without a lockfile is a still-initializing
        // project. Don't gate run on it; treat as legacy.
        return Ok(Mode::Legacy);
    }
    let lock_src = std::fs::read_to_string(&lock_path)
        .with_context(|| format!("read {}", lock_path.display()))?;
    let packages = parse_locked_packages(&lock_src)?;
    if packages.is_empty() {
        return Ok(Mode::EmptyLock);
    }

    let site = project_dir.join(VENV_DIR).join(SITE_PACKAGES);
    if !site.exists() {
        bail!(
            "environment is not synced — run `mamba sync` to install \
             {n} locked package(s) into {venv}",
            n = packages.len(),
            venv = project_dir.join(VENV_DIR).display()
        );
    }

    // Inject the project's site-packages at the front of PYTHONPATH
    // for the lifetime of the current process. No global PATH /
    // user shell env mutation.
    if let Some(joined) = joined_path_front("PYTHONPATH", [site.clone()]) {
        // SAFETY: env mutation is process-local; documented and
        // observed by the compiler session that follows.
        unsafe {
            std::env::set_var("PYTHONPATH", joined);
        }
    }
    Ok(Mode::Project {
        site_packages: site,
    })
}

/// Apply the project environment to a subprocess for `mamba run -- <cmd>`.
/// This never mutates the user's shell. Executables from `.venv/bin` (or
/// `.venv/Scripts`) win over host PATH when a synced venv exists, and
/// site-packages remains importable for host Python fallbacks.
pub fn configure_command_environment(command: &mut Command, project_dir: &Path, mode: &Mode) {
    let venv = project_dir.join(VENV_DIR);
    if venv.join("pyvenv.cfg").exists() {
        command.env("VIRTUAL_ENV", &venv);
        let bin_dirs = [venv.join("bin"), venv.join("Scripts")]
            .into_iter()
            .filter(|path| path.is_dir());
        if let Some(joined) = joined_path_front("PATH", bin_dirs) {
            command.env("PATH", joined);
        }
    }

    let site = match mode {
        Mode::Project { site_packages } => Some(site_packages.clone()),
        Mode::Legacy | Mode::EmptyLock => {
            let candidate = venv.join(SITE_PACKAGES);
            candidate.exists().then_some(candidate)
        }
    };
    if let Some(site) = site {
        if let Some(joined) = joined_path_front("PYTHONPATH", [site]) {
            command.env("PYTHONPATH", joined);
        }
    }
}

fn joined_path_front<I>(key: &str, front: I) -> Option<std::ffi::OsString>
where
    I: IntoIterator<Item = PathBuf>,
{
    let existing = std::env::var_os(key);
    let mut paths: Vec<PathBuf> = Vec::new();
    for path in front {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    if let Some(v) = existing {
        for p in std::env::split_paths(&v) {
            if !paths.contains(&p) {
                paths.push(p);
            }
        }
    }
    std::env::join_paths(paths.iter()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_when_no_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(preflight(tmp.path()).unwrap(), Mode::Legacy);
    }

    #[test]
    fn legacy_when_no_lockfile() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("mamba.toml"),
            "[project]\nname = \"x\"\nversion = \"0.1.0\"\npython-requires = \">=3.12\"\ndependencies = []\ndev-dependencies = []\n",
        )
        .unwrap();
        assert_eq!(preflight(tmp.path()).unwrap(), Mode::Legacy);
    }

    #[test]
    fn empty_lock_does_not_require_venv() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("mamba.toml"), "[project]\n").unwrap();
        std::fs::write(
            tmp.path().join("mamba.lock"),
            "format_version = 1\ninput_hash = \"x\"\n",
        )
        .unwrap();
        assert_eq!(preflight(tmp.path()).unwrap(), Mode::EmptyLock);
    }

    #[test]
    fn non_empty_lock_without_venv_bails() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("mamba.toml"), "[project]\n").unwrap();
        std::fs::write(
            tmp.path().join("mamba.lock"),
            "format_version = 1\ninput_hash = \"x\"\n\n[[package]]\nname = \"foo\"\nversion = \"1.0\"\nsha256 = \"\"\nsource = \"pypi://foo/1.0\"\ndependencies = []\n",
        )
        .unwrap();
        let err = preflight(tmp.path()).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("environment is not synced"), "{msg}");
    }
}

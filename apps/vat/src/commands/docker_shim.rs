// HANDWRITE-BEGIN gap="vat-headless-docker-command-shim-installer" tracker="#1685" reason="The multicall Docker shim must be opt-in and must never overwrite a real Docker binary. Its symlink ownership and actionable JSON handoff are user-facing filesystem policy, not a generic generator primitive."
//! Management commands for VAT's opt-in `docker -> vat` multicall shim.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{bail, Context, Result};

/// Install a `docker` symlink that dispatches through VAT's fail-closed Apple
/// Container compatibility layer.  The caller must name a directory explicitly
/// so VAT never changes PATH or shadows a real Docker client by surprise.
pub fn install_shim(dir: PathBuf) -> Result<ExitCode> {
    let executable = std::env::current_exe()
        .context("resolve the running vat executable for Docker shim installation")?
        .canonicalize()
        .context("canonicalize the running vat executable for Docker shim installation")?;
    fs::create_dir_all(&dir)
        .with_context(|| format!("create Docker shim directory {}", dir.display()))?;
    let shim = dir.join("docker");

    let state = match fs::symlink_metadata(&shim) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            if symlink_points_to(&shim, &executable)? {
                "already_installed"
            } else {
                bail!(
                    "refusing to replace existing Docker shim {}: it does not point to this vat executable {}; choose another --dir or remove it explicitly",
                    shim.display(),
                    executable.display()
                );
            }
        }
        Ok(_) => {
            bail!(
                "refusing to overwrite existing path {}: VAT only installs into an absent path or its own existing symlink",
                shim.display()
            );
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            create_symlink(&executable, &shim)?;
            "installed"
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspect Docker shim destination {}", shim.display()));
        }
    };

    print_install_result(state, &shim, &executable, &dir);
    Ok(ExitCode::SUCCESS)
}

/// Print whether an explicit directory currently contains a safe VAT Docker
/// shim.  This is read-only and never follows a foreign symlink as ownership.
pub fn shim_status(dir: PathBuf) -> Result<ExitCode> {
    let executable = std::env::current_exe()
        .context("resolve the running vat executable for Docker shim status")?
        .canonicalize()
        .context("canonicalize the running vat executable for Docker shim status")?;
    let shim = dir.join("docker");
    let installed = match fs::symlink_metadata(&shim) {
        Ok(metadata) if metadata.file_type().is_symlink() => symlink_points_to(&shim, &executable)?,
        Ok(_) => false,
        Err(error) if error.kind() == ErrorKind::NotFound => false,
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspect Docker shim destination {}", shim.display()));
        }
    };
    let next = if installed {
        format!("{} && docker --help", path_export(&dir))
    } else {
        format!("vat docker install-shim --dir {}", shell_quote(&dir))
    };
    println!(
        "{}",
        serde_json::json!({
            "type": "docker_shim",
            "installed": installed,
            "path": shim,
            "target": executable,
            "next": next,
        })
    );
    Ok(ExitCode::SUCCESS)
}

fn print_install_result(state: &str, shim: &Path, executable: &Path, dir: &Path) {
    println!(
        "{}",
        serde_json::json!({
            "type": "docker_shim",
            "status": state,
            "path": shim,
            "target": executable,
            "engine_api": false,
            "next": format!("{} && docker --help", path_export(dir)),
        })
    );
}

fn path_export(dir: &Path) -> String {
    format!("export PATH={}:$PATH", shell_quote(dir))
}

fn shell_quote(path: &Path) -> String {
    let value = path.to_string_lossy();
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn symlink_points_to(shim: &Path, executable: &Path) -> Result<bool> {
    let target = fs::read_link(shim)
        .with_context(|| format!("read Docker shim symlink {}", shim.display()))?;
    let resolved = if target.is_absolute() {
        target
    } else {
        shim.parent()
            .context("Docker shim path has no parent directory")?
            .join(target)
    };
    Ok(resolved.canonicalize().ok().as_deref() == Some(executable))
}

#[cfg(unix)]
fn create_symlink(executable: &Path, shim: &Path) -> Result<()> {
    std::os::unix::fs::symlink(executable, shim).with_context(|| {
        format!(
            "create Docker shim {} -> {}",
            shim.display(),
            executable.display()
        )
    })
}

#[cfg(not(unix))]
fn create_symlink(_executable: &Path, _shim: &Path) -> Result<()> {
    bail!("VAT's Docker shim installer currently requires a Unix symlink-capable host")
}
// HANDWRITE-END

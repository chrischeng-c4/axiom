// `mamba python` — local Python discovery and .python-version pinning.

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::pkgmanage::pkgmgr::shell::Shell;
use crate::pkgmanage::pkgmgr::toolchain::{
    PythonRequest, PythonVersion, discover_system_pythons, probe_python_version, read_python_pin,
    select_python, write_python_pin,
};
use crate::pkgmanage::pkgmgr::uv_dirs::{EnvLookup, Platform, resolve_dirs};

pub fn cmd_python(sub: &ArgMatches) -> Result<()> {
    match sub.subcommand() {
        Some(("install", cmd)) => cmd_install(cmd, "Installed"),
        Some(("download", cmd)) => cmd_install(cmd, "Downloaded"),
        Some(("uninstall", cmd)) => cmd_uninstall(cmd),
        Some(("list", cmd)) => cmd_list(cmd),
        Some(("find", cmd)) => cmd_find(cmd),
        Some(("pin", cmd)) => cmd_pin(cmd),
        Some(("dir", cmd)) => cmd_dir(cmd),
        Some(("update-shell", cmd)) => cmd_update_shell(cmd),
        Some((other, _)) => bail!("unknown python subcommand `{other}`"),
        None => bail!(
            "`mamba python` requires a subcommand: install | download | uninstall | list | find | pin | dir | update-shell"
        ),
    }
}

fn cmd_install(sub: &ArgMatches, verb: &str) -> Result<()> {
    let request = requested_python(sub)?;
    let source = match sub.get_one::<String>("source") {
        Some(path) => PathBuf::from(path),
        None => {
            let pythons = discover_system_pythons()?;
            select_python(&request, &pythons)
                .with_context(|| format!("no local Python matches {}", display_request(&request)))?
                .path
                .clone()
        }
    };
    let version = probe_python_version(&source)
        .map_err(|err| anyhow::anyhow!("probe source python {}: {err}", source.display()))?;
    if !request.matches(&version) {
        bail!(
            "source Python {} does not satisfy request {}",
            version,
            display_request(&request)
        );
    }

    let root = managed_python_root();
    let version_dir = root.join(version.to_string());
    let bin_dir = version_dir.join("bin");
    fs::create_dir_all(&bin_dir).with_context(|| format!("create {}", bin_dir.display()))?;
    write_launcher(&bin_dir.join("python"), &source)?;
    let manifest = format!(
        "implementation = \"cpython\"\nversion = \"{version}\"\nsource = \"{}\"\n",
        source.display()
    );
    fs::write(version_dir.join("manifest.toml"), manifest)
        .with_context(|| format!("write {}", version_dir.display()))?;
    rebuild_python_bin(&root)?;
    println!("{verb} Python {version} at {}", version_dir.display());
    Ok(())
}

fn cmd_list(sub: &ArgMatches) -> Result<()> {
    let pythons = discover_system_pythons()?;
    if sub.get_flag("json") {
        let rows: Vec<_> = pythons
            .iter()
            .map(|p| {
                serde_json::json!({
                    "version": p.version.to_string(),
                    "path": p.path,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }
    for py in pythons {
        println!("{}\t{}", py.version, py.path.display());
    }
    Ok(())
}

fn cmd_find(sub: &ArgMatches) -> Result<()> {
    let request = requested_python(sub)?;
    let pythons = discover_system_pythons()?;
    let Some(selected) = select_python(&request, &pythons) else {
        bail!("no installed Python matches {}", display_request(&request));
    };
    println!("{}", selected.path.display());
    Ok(())
}

fn cmd_pin(sub: &ArgMatches) -> Result<()> {
    let raw = sub
        .get_one::<String>("request")
        .context("python pin requires a version request")?;
    let request = PythonRequest::from_str(raw).map_err(anyhow::Error::msg)?;
    let dir = project_dir(sub);
    write_python_pin(&dir, &request)?;
    println!(
        "Pinned Python {} in {}",
        display_request(&request),
        dir.display()
    );
    Ok(())
}

fn cmd_uninstall(sub: &ArgMatches) -> Result<()> {
    let request = requested_python(sub)?;
    let root = managed_python_root();
    let mut removed = Vec::new();
    for (version, dir) in managed_versions(&root)? {
        if request.matches(&version) {
            fs::remove_dir_all(&dir).with_context(|| format!("remove {}", dir.display()))?;
            removed.push(version.to_string());
        }
    }
    rebuild_python_bin(&root)?;
    if removed.is_empty() {
        eprintln!(
            "no_op: no managed Python matches {}",
            display_request(&request)
        );
    } else {
        println!("Uninstalled Python {}", removed.join(", "));
    }
    Ok(())
}

fn cmd_dir(sub: &ArgMatches) -> Result<()> {
    let root = managed_python_root();
    if sub.get_flag("bin") {
        println!("{}", root.join("bin").display());
    } else {
        println!("{}", root.display());
    }
    Ok(())
}

fn cmd_update_shell(sub: &ArgMatches) -> Result<()> {
    let shell = sub
        .get_one::<String>("shell")
        .map(|raw| {
            Shell::parse(raw).with_context(|| {
                format!(
                    "unsupported shell `{raw}`; expected bash|zsh|fish|powershell|cmd|nushell|elvish"
                )
            })
        })
        .transpose()?
        .unwrap_or(Shell::Bash);
    let bin_dir = sub
        .get_one::<String>("bin-dir")
        .map(PathBuf::from)
        .unwrap_or(managed_python_root().join("bin"));
    let snippet = shell.prepend_path_snippet(&bin_dir.display().to_string());
    print!("{}", shell.wrap_managed_block(&snippet));
    Ok(())
}

fn requested_python(sub: &ArgMatches) -> Result<PythonRequest> {
    if let Some(raw) = sub.get_one::<String>("request") {
        return PythonRequest::from_str(raw).map_err(anyhow::Error::msg);
    }
    let cwd = std::env::current_dir().context("read current directory")?;
    Ok(read_python_pin(&cwd)?.unwrap_or(PythonRequest::Any))
}

fn project_dir(sub: &ArgMatches) -> PathBuf {
    sub.get_one::<String>("project")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn display_request(request: &PythonRequest) -> String {
    match request {
        PythonRequest::Any => "any".to_string(),
        PythonRequest::Major(major) => major.to_string(),
        PythonRequest::MajorMinor(major, minor) => format!("{major}.{minor}"),
        PythonRequest::Exact(version) => version.to_string(),
    }
}

fn managed_python_root() -> PathBuf {
    let dirs = resolve_dirs(
        &EnvLookup::from_process_env(),
        Platform::current(),
        &std::env::temp_dir().display().to_string(),
    );
    dirs.python_install_dir()
}

fn managed_versions(root: &Path) -> Result<Vec<(PythonVersion, PathBuf)>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(root).with_context(|| format!("read {}", root.display()))? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let Some(raw) = name.to_str() else {
            continue;
        };
        if raw == "bin" {
            continue;
        }
        if let Ok(version) = PythonVersion::from_str(raw) {
            out.push((version, entry.path()));
        }
    }
    out.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    Ok(out)
}

fn rebuild_python_bin(root: &Path) -> Result<()> {
    let versions = managed_versions(root)?;
    let bin = root.join("bin");
    if bin.exists() {
        fs::remove_dir_all(&bin).with_context(|| format!("remove {}", bin.display()))?;
    }
    fs::create_dir_all(&bin).with_context(|| format!("create {}", bin.display()))?;

    for (version, dir) in &versions {
        let target = dir.join("bin").join("python");
        write_launcher(&bin.join(format!("python{version}")), &target)?;
    }

    if let Some((latest, dir)) = versions.first() {
        let target = dir.join("bin").join("python");
        write_launcher(&bin.join("python"), &target)?;
        write_launcher(&bin.join(format!("python{}", latest.major)), &target)?;
        write_launcher(
            &bin.join(format!("python{}.{}", latest.major, latest.minor)),
            &target,
        )?;
    }
    Ok(())
}

fn write_launcher(path: &Path, target: &Path) -> Result<()> {
    let body = format!("#!/bin/sh\nexec {} \"$@\"\n", shell_quote_path(target));
    fs::write(path, body).with_context(|| format!("write {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(path)
            .with_context(|| format!("stat {}", path.display()))?
            .permissions();
        perm.set_mode(0o755);
        fs::set_permissions(path, perm).with_context(|| format!("chmod {}", path.display()))?;
    }
    Ok(())
}

fn shell_quote_path(path: &Path) -> String {
    let raw = path.to_string_lossy();
    format!("'{}'", raw.replace('\'', "'\\''"))
}

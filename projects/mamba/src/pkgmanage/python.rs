// `mamba python` — local Python discovery and .python-version pinning.

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use std::path::PathBuf;
use std::str::FromStr;

use crate::pkgmanage::pkgmgr::toolchain::{
    PythonRequest, discover_system_pythons, read_python_pin, select_python, write_python_pin,
};
use crate::pkgmanage::pkgmgr::uv_dirs::{EnvLookup, Platform, resolve_dirs};

pub fn cmd_python(sub: &ArgMatches) -> Result<()> {
    match sub.subcommand() {
        Some(("list", cmd)) => cmd_list(cmd),
        Some(("find", cmd)) => cmd_find(cmd),
        Some(("pin", cmd)) => cmd_pin(cmd),
        Some(("dir", _)) => cmd_dir(),
        Some((other, _)) => bail!("unknown python subcommand `{other}`"),
        None => bail!("`mamba python` requires a subcommand: list | find | pin | dir"),
    }
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

fn cmd_dir() -> Result<()> {
    let dirs = resolve_dirs(
        &EnvLookup::from_process_env(),
        Platform::current(),
        &std::env::temp_dir().display().to_string(),
    );
    println!("{}", dirs.python_install_dir().display());
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

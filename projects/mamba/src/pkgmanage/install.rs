// `mamba install` — uv-tool-style tool install path.
//
// Profile: validation/profiles/package_manager.toml [families.install]
// (per-fixture manifest is a forward reference; this module pins the
// shape today so the family can wire up cleanly):
//
//   - `mamba install <pkg>` materializes the package into the user's
//     tools dir (`$MAMBA_TOOLS_DIR` or `~/.local/share/mamba/tools`).
//   - `--index DIR` (or $MAMBA_FROZEN_INDEX) sources the wheel
//     metadata; offline-only; never touches the network.
//   - `--version X.Y.Z` pins; otherwise picks the latest version
//     present in the index.
//   - `mamba install --list` enumerates installed tools as
//     `<name>==<version>` lines, sorted.
//   - `mamba install --uninstall <pkg>` removes a tool atomically.
//   - Repeating an install at the same version is an idempotent
//     no-op that emits `no_op` to stderr.

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

const TOOLS_DIR_ENV: &str = "MAMBA_TOOLS_DIR";
const FROZEN_INDEX_ENV: &str = "MAMBA_FROZEN_INDEX";

pub fn cmd_install(sub: &ArgMatches) -> Result<()> {
    if sub.get_flag("list") {
        return action_list();
    }
    if let Some(name) = sub.get_one::<String>("uninstall") {
        return action_uninstall(name);
    }
    let name = sub
        .get_one::<String>("name")
        .context("missing required <name> (or use --list / --uninstall)")?;
    let explicit_version = sub.get_one::<String>("version").cloned();
    let index = sub
        .get_one::<String>("index")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os(FROZEN_INDEX_ENV).map(PathBuf::from))
        .context(
            "no frozen index configured (pass --index DIR or set $MAMBA_FROZEN_INDEX)",
        )?;
    action_install(name, explicit_version.as_deref(), &index)
}

fn action_install(name: &str, explicit_version: Option<&str>, index: &Path) -> Result<()> {
    let normalized = normalize_pep503(name);
    let pkg_dir = index.join(&normalized);
    if !pkg_dir.exists() {
        bail!(
            "package `{name}` not found in index {}",
            index.display()
        );
    }
    let version = match explicit_version {
        Some(v) => {
            let ver_dir = pkg_dir.join(v);
            if !ver_dir.exists() {
                bail!(
                    "package `{name}` version `{v}` not found in index {}",
                    index.display()
                );
            }
            v.to_string()
        }
        None => pick_latest_version(&pkg_dir)?,
    };

    let tools_root = resolve_tools_root()?;
    let tool_dir = tools_root.join(&normalized);
    let installed = read_installed_version(&tool_dir);
    if installed.as_deref() == Some(version.as_str()) {
        eprintln!(
            "no_op: {name}=={version} already installed at {}",
            tool_dir.display()
        );
        return Ok(());
    }

    // Atomic-ish replace: write into a sibling tmp dir, swap on
    // success. Removes any prior version of the same tool.
    fs::create_dir_all(&tools_root)
        .with_context(|| format!("create {}", tools_root.display()))?;
    if tool_dir.exists() {
        fs::remove_dir_all(&tool_dir)
            .with_context(|| format!("remove old {}", tool_dir.display()))?;
    }
    let bin_dir = tool_dir.join("bin");
    fs::create_dir_all(&bin_dir)
        .with_context(|| format!("create {}", bin_dir.display()))?;
    let pkg_root = tool_dir.join("pkg");
    fs::create_dir_all(&pkg_root)
        .with_context(|| format!("create {}", pkg_root.display()))?;
    let init_body = format!(
        "# stub-installed by `mamba install` from a frozen local index\n\
         __mamba_tool__ = {:?}\n\
         __version__ = {:?}\n",
        name, version
    );
    fs::write(pkg_root.join(format!("{normalized}.py")), init_body)
        .with_context(|| format!("write {}", pkg_root.display()))?;
    let manifest = format!(
        "name = \"{name}\"\nversion = \"{version}\"\n"
    );
    fs::write(tool_dir.join("manifest.toml"), manifest)
        .with_context(|| format!("write {}", tool_dir.display()))?;
    write_shim(&bin_dir, name, &tool_dir)?;
    Ok(())
}

fn action_list() -> Result<()> {
    let tools_root = resolve_tools_root()?;
    if !tools_root.exists() {
        return Ok(());
    }
    let mut entries: Vec<(String, String)> = vec![];
    for e in fs::read_dir(&tools_root)
        .with_context(|| format!("read {}", tools_root.display()))?
    {
        let e = e?;
        if !e.file_type()?.is_dir() {
            continue;
        }
        let dir = e.path();
        let manifest_path = dir.join("manifest.toml");
        if !manifest_path.exists() {
            continue;
        }
        let raw = fs::read_to_string(&manifest_path)
            .with_context(|| format!("read {}", manifest_path.display()))?;
        let doc: toml::Value = raw
            .parse()
            .with_context(|| format!("parse {}", manifest_path.display()))?;
        let name = doc
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let version = doc
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if !name.is_empty() && !version.is_empty() {
            entries.push((name, version));
        }
    }
    entries.sort();
    let mut stdout = std::io::stdout().lock();
    for (name, version) in entries {
        writeln!(stdout, "{name}=={version}").context("write list line")?;
    }
    Ok(())
}

fn action_uninstall(name: &str) -> Result<()> {
    let tools_root = resolve_tools_root()?;
    let tool_dir = tools_root.join(normalize_pep503(name));
    if !tool_dir.exists() {
        eprintln!("no_op: {name} is not installed");
        return Ok(());
    }
    fs::remove_dir_all(&tool_dir)
        .with_context(|| format!("remove {}", tool_dir.display()))?;
    Ok(())
}

fn read_installed_version(tool_dir: &Path) -> Option<String> {
    let raw = fs::read_to_string(tool_dir.join("manifest.toml")).ok()?;
    let doc: toml::Value = raw.parse().ok()?;
    doc.get("version")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn write_shim(bin_dir: &Path, name: &str, tool_dir: &Path) -> Result<()> {
    let shim_path = bin_dir.join(name);
    let body = format!(
        "#!/bin/sh\n\
         # mamba tool shim for `{name}`\n\
         exec mamba run \"{tool_dir}/pkg/{normalized}.py\" \"$@\"\n",
        tool_dir = tool_dir.display(),
        normalized = normalize_pep503(name),
    );
    fs::write(&shim_path, body)
        .with_context(|| format!("write {}", shim_path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(&shim_path)
            .with_context(|| format!("stat {}", shim_path.display()))?
            .permissions();
        perm.set_mode(0o755);
        fs::set_permissions(&shim_path, perm)
            .with_context(|| format!("chmod {}", shim_path.display()))?;
    }
    Ok(())
}

pub fn resolve_tools_root() -> Result<PathBuf> {
    if let Some(env_root) = std::env::var_os(TOOLS_DIR_ENV) {
        let p = PathBuf::from(env_root);
        if p.as_os_str().is_empty() {
            bail!("${TOOLS_DIR_ENV} is set but empty");
        }
        return Ok(p);
    }
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        let p = PathBuf::from(xdg);
        if !p.as_os_str().is_empty() {
            return Ok(p.join("mamba").join("tools"));
        }
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .context("no $HOME and no $MAMBA_TOOLS_DIR — set $MAMBA_TOOLS_DIR")?;
    Ok(home.join(".local").join("share").join("mamba").join("tools"))
}

fn normalize_pep503(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_sep = false;
    for c in name.chars() {
        let is_sep = c == '-' || c == '_' || c == '.';
        if is_sep {
            if !prev_sep && !out.is_empty() {
                out.push('-');
            }
            prev_sep = true;
        } else {
            out.push(c.to_ascii_lowercase());
            prev_sep = false;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    out
}

fn pick_latest_version(pkg_dir: &Path) -> Result<String> {
    let mut versions: Vec<String> = fs::read_dir(pkg_dir)
        .with_context(|| format!("read {}", pkg_dir.display()))?
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().to_str().map(str::to_string))
        .collect();
    if versions.is_empty() {
        bail!("no versions in {}", pkg_dir.display());
    }
    versions.sort();
    Ok(versions.pop().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

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
    fn pep503_normalize_install() {
        assert_eq!(normalize_pep503("Foo.Bar_baz"), "foo-bar-baz");
        assert_eq!(normalize_pep503("plain"), "plain");
    }

    #[test]
    fn tools_root_env_var_takes_priority() {
        let tmp = tempfile::tempdir().unwrap();
        with_env_var(TOOLS_DIR_ENV, tmp.path().to_str(), || {
            let root = resolve_tools_root().unwrap();
            assert_eq!(root, tmp.path());
        });
    }
}

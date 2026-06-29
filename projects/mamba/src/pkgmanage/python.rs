// `mamba python` — local Python discovery and .python-version pinning.

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Cursor, Read};
use std::path::Component;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use crate::pkgmanage::pkgmgr::pbs_host::detect_host_target;
use crate::pkgmanage::pkgmgr::pbs_url::PbsArtifact;
use crate::pkgmanage::pkgmgr::shell::Shell;
use crate::pkgmanage::pkgmgr::toolchain::{
    PythonRequest, PythonVersion, discover_system_pythons, probe_python_version, read_python_pin,
    select_python, write_python_pin,
};
use crate::pkgmanage::pkgmgr::url_redact::redact_credentials;
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
    if sub.get_one::<String>("url").is_some()
        || sub.get_one::<String>("release-tag").is_some()
        || (verb == "Downloaded" && sub.get_one::<String>("source").is_none())
    {
        return install_from_archive(sub, &request, verb);
    }
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
    install_from_source(&source, &request, verb)
}

fn install_from_source(source: &Path, request: &PythonRequest, verb: &str) -> Result<()> {
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

fn install_from_archive(sub: &ArgMatches, request: &PythonRequest, verb: &str) -> Result<()> {
    let url = resolve_standalone_url(sub, request)?;
    let archive = download_python_archive(&url)?;
    if let Some(expected) = sub.get_one::<String>("sha256") {
        let actual = sha256_hex(&archive);
        if !expected.eq_ignore_ascii_case(&actual) {
            bail!(
                "downloaded Python archive sha256 mismatch: expected {}, got {}",
                expected,
                actual
            );
        }
    }

    let root = managed_python_root();
    fs::create_dir_all(&root).with_context(|| format!("create {}", root.display()))?;
    let scratch = root.join(format!(
        ".download-{}-{}",
        std::process::id(),
        monotonic_nanos()
    ));
    if scratch.exists() {
        fs::remove_dir_all(&scratch).with_context(|| format!("remove {}", scratch.display()))?;
    }
    fs::create_dir_all(&scratch).with_context(|| format!("create {}", scratch.display()))?;

    extract_python_archive(&url, &archive, &scratch)?;
    let extracted_python = find_extracted_python(&scratch).with_context(|| {
        format!(
            "no Python executable found in archive {}",
            redact_credentials(&url)
        )
    })?;
    let version = probe_python_version(&extracted_python).map_err(|err| {
        anyhow::anyhow!(
            "probe downloaded python {}: {err}",
            extracted_python.display()
        )
    })?;
    if !request.matches(&version) {
        let _ = fs::remove_dir_all(&scratch);
        bail!(
            "downloaded Python {} does not satisfy request {}",
            version,
            display_request(request)
        );
    }

    let relative_python = extracted_python
        .strip_prefix(&scratch)
        .context("resolve downloaded python path")?
        .to_path_buf();
    let version_dir = root.join(version.to_string());
    if version_dir.exists() {
        fs::remove_dir_all(&version_dir)
            .with_context(|| format!("remove {}", version_dir.display()))?;
    }
    fs::rename(&scratch, &version_dir)
        .with_context(|| format!("move {} to {}", scratch.display(), version_dir.display()))?;

    let installed_python = version_dir.join(relative_python);
    let bin_dir = version_dir.join("bin");
    fs::create_dir_all(&bin_dir).with_context(|| format!("create {}", bin_dir.display()))?;
    let launcher = bin_dir.join("python");
    if installed_python != launcher {
        write_launcher(&launcher, &installed_python)?;
    }
    let manifest = format!(
        "implementation = \"cpython\"\nversion = \"{version}\"\nsource = \"{}\"\narchive_sha256 = \"{}\"\n",
        redact_credentials(&url),
        sha256_hex(&archive)
    );
    fs::write(version_dir.join("manifest.toml"), manifest)
        .with_context(|| format!("write {}", version_dir.display()))?;
    rebuild_python_bin(&root)?;
    println!(
        "{verb} Python {version} at {} from {}",
        version_dir.display(),
        redact_credentials(&url)
    );
    Ok(())
}

fn resolve_standalone_url(sub: &ArgMatches, request: &PythonRequest) -> Result<String> {
    if let Some(url) = sub.get_one::<String>("url") {
        return Ok(url.clone());
    }
    let release_tag = sub
        .get_one::<String>("release-tag")
        .cloned()
        .or_else(|| std::env::var(PBS_RELEASE_ENV).ok())
        .with_context(|| {
            format!(
                "standalone Python download requires --url, --release-tag, or ${PBS_RELEASE_ENV}"
            )
        })?;
    let version = match request {
        PythonRequest::Exact(version) => *version,
        _ => bail!(
            "python-build-standalone URL composition requires an exact Python version; pass 3.x.y or --url"
        ),
    };
    let target = detect_host_target().map_err(anyhow::Error::from)?;
    Ok(PbsArtifact {
        release_tag,
        version,
        target,
    }
    .url())
}

fn download_python_archive(url: &str) -> Result<Vec<u8>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("build tokio runtime for Python download")?;
    rt.block_on(async {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(180))
            .build()
            .context("build Python download HTTP client")?;
        let response = client
            .get(url)
            .send()
            .await
            .with_context(|| format!("download Python archive {}", redact_credentials(url)))?;
        let status = response.status();
        if !status.is_success() {
            bail!(
                "download Python archive {} failed with HTTP {}",
                redact_credentials(url),
                status.as_u16()
            );
        }
        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("read Python archive {}", redact_credentials(url)))?;
        Ok(bytes.to_vec())
    })
}

fn extract_python_archive(url: &str, archive: &[u8], dest: &Path) -> Result<()> {
    if url.ends_with(".tar.zst") || url.ends_with(".tzst") {
        let decoder = zstd::stream::read::Decoder::new(Cursor::new(archive))
            .context("decode Python .tar.zst archive")?;
        let mut tar = tar::Archive::new(decoder);
        unpack_safe_tar(&mut tar, dest)
    } else if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
        let decoder = GzDecoder::new(Cursor::new(archive));
        let mut tar = tar::Archive::new(decoder);
        unpack_safe_tar(&mut tar, dest)
    } else if url.ends_with(".tar") {
        let mut tar = tar::Archive::new(Cursor::new(archive));
        unpack_safe_tar(&mut tar, dest)
    } else {
        bail!(
            "unsupported Python archive format for {}; expected .tar.zst, .tar.gz, .tgz, or .tar",
            redact_credentials(url)
        )
    }
}

fn unpack_safe_tar<R: Read>(archive: &mut tar::Archive<R>, dest: &Path) -> Result<()> {
    for entry in archive.entries().context("read Python archive entries")? {
        let mut entry = entry.context("read Python archive entry")?;
        let path = entry
            .path()
            .context("read Python archive entry path")?
            .to_path_buf();
        if !safe_archive_path(&path) {
            bail!("unsafe Python archive path {}", path.display());
        }
        let dest_path = dest.join(path);
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        entry
            .unpack(dest_path)
            .context("unpack Python archive entry")?;
    }
    Ok(())
}

fn safe_archive_path(path: &Path) -> bool {
    path.components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
}

fn find_extracted_python(root: &Path) -> Result<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    let mut candidates = Vec::new();
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                stack.push(path);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if matches!(name, "python" | "python3" | "python.exe")
                && probe_python_version(&path).is_ok()
            {
                candidates.push(path);
            }
        }
    }
    candidates.sort_by_key(|path| python_candidate_rank(path));
    candidates
        .into_iter()
        .next()
        .context("missing Python executable")
}

fn python_candidate_rank(path: &Path) -> (u8, String) {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let rank = if normalized.ends_with("/python/install/bin/python3") {
        0
    } else if normalized.ends_with("/python/install/bin/python") {
        1
    } else if normalized.ends_with("/install/bin/python3") {
        2
    } else if normalized.ends_with("/bin/python3") {
        3
    } else if normalized.ends_with("/bin/python") {
        4
    } else {
        5
    };
    (rank, normalized)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn monotonic_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
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

const PBS_RELEASE_ENV: &str = "MAMBA_PYTHON_BUILD_STANDALONE_RELEASE";

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

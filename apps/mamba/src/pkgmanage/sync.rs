// `mamba sync` — converge `.venv` to mamba.lock idempotently.
//
// Acceptance (tests/governance/gates/pkgmgr/sync/manifest.toml, schema gate
// pkgmgr_sync_idempotence_fixture_2683.rs):
//
//   - First run creates the env and "installs" every locked package.
//   - Second run is a clean no-op: no env mutation, mamba.lock byte-
//     identical, stderr reports a structured no-op signal.
//   - Import probe (`<pkg>/__init__.py`) is present after both runs.
//   - Offline against the frozen local index; never touches global cache.
//
// Materialization model (real venv, real wheel):
//   <venv>/lib/pythonX.Y/site-packages/<dist>-<version>.dist-info/RECORD
//   (POSIX; `Lib/site-packages` on Windows) — the venv's own `pyvenv.cfg`
// names the interpreter version, `VenvLayout::for_current_platform` derives
// the layout from it, and `Installer::install` unpacks the real wheel
// there. A lock entry's `path` (index, direct_file) is installed directly;
// a `url` plus `sha256` entry is downloaded and hash-verified first, then
// installed the same way; a `mamba_provider` entry keeps its
// generated-file materialization. No stub is written for any kind.
//
// When `.venv` does not already exist (no prior `mamba venv`), `sync`
// creates a real PEP 405 environment itself via `pkgmgr::venv::create_venv`
// so a package is always installed into a real interpreter's
// site-packages.
//
// Idempotence signal: when the second run sees every locked package
// already present, it writes `no_op` to stderr and exits 0 — the
// installer's own dist-info fast path (`read_installed_dist_info`) is
// what makes a second install of the same version a no-op.
//
// No partial state on failure: lockfile is never rewritten by sync.

use anyhow::{bail, Context, Result};
use clap::ArgMatches;
use std::fs;
use std::path::{Path, PathBuf};

use crate::pkgmanage::manifest::pyproject;
use crate::pkgmanage::pkgmgr::installer::{InstallMode, InstallRequest, Installer};
use crate::pkgmanage::pkgmgr::venv::{
    create_venv, first_python_on_path, layout_from_pyvenv_cfg, VenvCreationOutcome, VenvOptions,
};
use crate::pkgmanage::provider;

const LOCKFILE_FILE: &str = "mamba.lock";
const VENV_DIR: &str = ".venv";
const SITE_PACKAGES: &str = "site-packages";

/// Resolve the site-packages directory `sync` installs into for a given
/// `.venv` root: the venv's own `VenvLayout` derived from its own
/// `pyvenv.cfg` (`lib/pythonX.Y/site-packages` on POSIX, `Lib/site-packages`
/// on Windows) — never the flat `<venv>/site-packages` directory. Falls
/// back to the flat layout only when `pyvenv.cfg` doesn't exist yet or
/// can't be parsed, so callers that probe before a venv exists still get a
/// stable (if unused) path.
pub(crate) fn resolve_site_packages(venv_dir: &Path) -> PathBuf {
    layout_from_pyvenv_cfg(venv_dir)
        .map(|layout| layout.site_packages)
        .unwrap_or_else(|_| venv_dir.join(SITE_PACKAGES))
}

/// Resolve the artifact `Installer::install` should unpack for one locked,
/// non-provider package: a non-empty `path` (index, direct_file) names it
/// directly and must exist on disk; otherwise a non-empty `url` plus
/// `sha256` names an already-downloaded, hash-verified artifact in the
/// sync cache; an entry with neither fails, naming the package.
pub(crate) fn resolve_artifact_path(pkg: &LockedPkg) -> Result<PathBuf> {
    if !pkg.path.is_empty() {
        let path = PathBuf::from(&pkg.path);
        if !path.is_file() {
            bail!(
                "cannot sync package `{}`: artifact path {} does not exist",
                pkg.name,
                path.display()
            );
        }
        return Ok(path);
    }
    if !pkg.url.is_empty() && !pkg.sha256.is_empty() {
        let cache_dir = sync_cache_dir();
        let filename = derive_filename(&pkg.url, &pkg.name, &pkg.version);
        return Ok(crate::pkgmanage::pkgmgr::cache::artifact_path(
            &cache_dir, &pkg.name, &filename,
        ));
    }
    bail!(
        "cannot sync package `{}`: mamba.lock entry names neither `path` nor `url`",
        pkg.name
    );
}

/// Create a real PEP 405 environment at `venv_dir` when `mamba sync` runs
/// before any `mamba venv`. Uses the first `python3`/`python` found on
/// `PATH` (falling back to the bare name `python3`, matching `mamba venv`'s
/// own default) and lays down the real interpreter tree — sync never seeds
/// `pip`, it doesn't need it.
fn ensure_real_venv(venv_dir: &Path) -> Result<()> {
    let python = first_python_on_path().unwrap_or_else(|| PathBuf::from("python3"));
    let opts = VenvOptions::new(python, venv_dir);
    match create_venv(&opts) {
        Ok(VenvCreationOutcome::Created { .. }) => Ok(()),
        Ok(VenvCreationOutcome::Refused { reason }) => bail!(reason),
        Err(e) => Err(anyhow::anyhow!(
            "create virtual environment at {}: {e}",
            venv_dir.display()
        )),
    }
}

pub fn cmd_sync(sub: &ArgMatches) -> Result<()> {
    let project_dir = std::env::current_dir().context("read current directory")?;
    pyproject::locate(&project_dir)?;
    let lock_path = project_dir.join(LOCKFILE_FILE);
    if !lock_path.exists() {
        bail!(
            "no {LOCKFILE_FILE} in {} — run `mamba lock` or `mamba add <dep>` first",
            project_dir.display()
        );
    }

    let lock_src =
        fs::read_to_string(&lock_path).with_context(|| format!("read {}", lock_path.display()))?;
    let packages = parse_locked_packages(&lock_src)?;

    let venv_dir = project_dir.join(VENV_DIR);

    // `plan_install` below only needs to read an existing site-packages;
    // when the venv doesn't exist yet, treat every package as pending so
    // `--check` and the no-op probe both see the real state.
    let probe_site = venv_dir
        .join("pyvenv.cfg")
        .exists()
        .then(|| resolve_site_packages(&venv_dir));
    let plan = plan_install(&packages, probe_site.as_deref());
    let extraneous: Vec<(String, String)> = probe_site
        .as_deref()
        .map(|site| plan_extraneous(&packages, site))
        .unwrap_or_default();
    if sub.get_flag("check") {
        if !venv_dir.join("pyvenv.cfg").exists() {
            bail!("environment is not synchronized with mamba.lock; missing .venv/pyvenv.cfg");
        }
        if !plan.is_empty() || !extraneous.is_empty() {
            let mut message = String::from("environment is not synchronized with mamba.lock");
            if !plan.is_empty() {
                let missing = plan
                    .iter()
                    .map(|p| format!("{}=={}", p.name, p.version))
                    .collect::<Vec<_>>()
                    .join(", ");
                message.push_str(&format!("; pending packages: {missing}"));
            }
            if !extraneous.is_empty() {
                let extra = extraneous
                    .iter()
                    .map(|(name, version)| format!("{name}=={version}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                message.push_str(&format!("; extraneous packages: {extra}"));
            }
            bail!(message);
        }
        println!("environment is synchronized with mamba.lock");
        return Ok(());
    }
    if plan.is_empty() && extraneous.is_empty() && venv_dir.join("pyvenv.cfg").exists() {
        eprintln!("no_op: environment already in sync with mamba.lock");
        return Ok(());
    }

    if !venv_dir.join("pyvenv.cfg").exists() {
        ensure_real_venv(&venv_dir)?;
    }
    let site = resolve_site_packages(&venv_dir);
    fs::create_dir_all(&site).with_context(|| format!("create {}", site.display()))?;

    // Tick 15: when the lockfile carries `url` + `sha256` for a package, fetch
    // the artifact through the streaming, sha-verifying IndexClient before
    // installing it. Tick 16: fan downloads out under a Semaphore so large
    // lockfiles don't sit on a single network connection.
    let downloadable: Vec<LockedPkg> = plan
        .iter()
        .filter(|p| !p.url.is_empty() && !p.sha256.is_empty())
        .cloned()
        .collect();
    if !downloadable.is_empty() {
        let jobs = resolve_jobs(sub);
        download_and_verify_parallel(&downloadable, jobs)?;
    }

    let layout = layout_from_pyvenv_cfg(&venv_dir)
        .map_err(|e| anyhow::anyhow!("resolve venv layout at {}: {e}", venv_dir.display()))?;
    let installer = Installer::new();

    // Prune every distribution the lock no longer pins before installing the
    // pending set — a package whose installed version differs from its pin
    // is both extraneous (old version) and pending (new version), and must
    // be removed before the new version is placed.
    for (name, _version) in &extraneous {
        prune_distribution(&installer, &site, name)?;
    }

    for pkg in &plan {
        if pkg.source_kind == "mamba_provider" {
            materialize_mamba_provider(&site, pkg)?;
            continue;
        }
        let artifact_path = resolve_artifact_path(pkg)?;
        let req = InstallRequest {
            artifact_path,
            site_packages: site.clone(),
            python_executable: layout.python_executable.clone(),
            mode: InstallMode::Purelib,
        };
        installer
            .install(req)
            .map_err(|e| anyhow::anyhow!("install {}=={}: {e}", pkg.name, pkg.version))?;
    }
    Ok(())
}

/// Resolve the concurrency bound for `mamba sync` downloads. Precedence:
/// `--jobs N` CLI flag, then `$MAMBA_JOBS` env var, falling back to 8 — a
/// healthy default for residential networks against PyPI's CDN. uv's default
/// is the higher of 8 or cpu_count; we hold flat at 8 until benchmarks
/// justify scaling.
fn resolve_jobs(sub: &ArgMatches) -> usize {
    if let Some(s) = sub.get_one::<String>("jobs") {
        if let Ok(n) = s.parse::<usize>() {
            if n > 0 {
                return n;
            }
        }
    }
    if let Ok(s) = std::env::var("MAMBA_JOBS") {
        if let Ok(n) = s.parse::<usize>() {
            if n > 0 {
                return n;
            }
        }
    }
    8
}

#[derive(Debug, Clone)]
pub(crate) struct LockedPkg {
    pub(crate) name: String,
    pub(crate) version: String,
    /// Canonical artifact URL recorded by `mamba lock` / `mamba add`. Empty
    /// when the lockfile was produced from a frozen local index (no URL is
    /// known) — sync then falls back to a stub install for that package.
    pub(crate) url: String,
    /// Lower-hex sha256 of the artifact at `url`. Empty for the local-frozen
    /// path. When non-empty alongside `url`, sync streams the artifact via
    /// [`IndexClient::download_artifact`] which verifies the hash.
    pub(crate) sha256: String,
    /// Optional source kind recorded by the lockfile. Direct local wheel
    /// entries use `direct_file` and intentionally leave `url` empty so sync
    /// stays offline.
    pub(crate) source_kind: String,
    /// Optional source path for local/direct entries.
    pub(crate) path: String,
    /// First-party provider name for mamba-owned replacement packages.
    pub(crate) provider: String,
    /// Import/API packages exposed by a provider distribution.
    pub(crate) provides: Vec<String>,
    /// Upstream API surface this provider targets.
    pub(crate) compatibility: String,
    /// Provider maturity label, e.g. experimental.
    pub(crate) maturity: String,
}

pub(crate) fn parse_locked_packages(lock_src: &str) -> Result<Vec<LockedPkg>> {
    let doc: toml::Value = lock_src.parse().context("parse mamba.lock")?;
    let arr = match doc.get("package") {
        Some(toml::Value::Array(a)) => a.clone(),
        Some(_) => bail!("mamba.lock `package` is not an array"),
        None => return Ok(vec![]),
    };
    let mut out = Vec::with_capacity(arr.len());
    for entry in arr {
        let tbl = entry
            .as_table()
            .context("mamba.lock package entry is not a table")?;
        let name = tbl
            .get("name")
            .and_then(|v| v.as_str())
            .context("mamba.lock package missing `name`")?
            .to_string();
        let version = tbl
            .get("version")
            .and_then(|v| v.as_str())
            .context("mamba.lock package missing `version`")?
            .to_string();
        let url = tbl
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let sha256 = tbl
            .get("sha256")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let source_kind = tbl
            .get("source_kind")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let path = tbl
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let provider = tbl
            .get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let provides = string_array(tbl, "provides");
        let compatibility = tbl
            .get("compatibility")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let maturity = tbl
            .get("maturity")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        out.push(LockedPkg {
            name,
            version,
            url,
            sha256,
            source_kind,
            path,
            provider,
            provides,
            compatibility,
            maturity,
        });
    }
    Ok(out)
}

fn string_array(tbl: &toml::value::Table, key: &str) -> Vec<String> {
    tbl.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Stream every locked artifact through the shared async IndexClient,
/// sha-verifying each one in flight. Tick 16: downloads run concurrently
/// bounded by a Semaphore of `max_concurrent` permits — order-independent,
/// first-error-wins. The shared cache_dir means parallel writes for distinct
/// (name, filename) pairs never collide on the .tmp/.sha256 sidecar.
///
/// The caller has already gated the call on every entry having url+sha, so
/// an error here is a real failure (network, hash mismatch, 404), never a
/// "no URL configured".
fn download_and_verify_parallel(plan: &[LockedPkg], max_concurrent: usize) -> Result<()> {
    use crate::pkgmanage::pkgmgr::types::{FileHash, IndexClient, ReleaseFile};
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    // Multi-thread runtime sized to min(jobs, 4) — we cap worker threads to
    // avoid spinning up dozens of OS threads for what is overwhelmingly I/O
    // wait. The Semaphore is the real concurrency knob.
    let worker_threads = max_concurrent.clamp(1, 4);
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(worker_threads)
        .build()
        .context("build tokio runtime for sync download")?;

    let cache_dir = sync_cache_dir();
    let cache_str: Arc<String> = Arc::new(cache_dir.to_string_lossy().into_owned());
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    rt.block_on(async {
        let mut set = tokio::task::JoinSet::new();
        for pkg in plan.iter().cloned() {
            let sem = semaphore.clone();
            let cache = cache_str.clone();
            set.spawn(async move {
                // Permit drops at end-of-scope; failures propagate as the
                // task's Result, never via panic.
                let _permit = sem
                    .acquire()
                    .await
                    .expect("sync semaphore never closes mid-flight");
                let auth_header = crate::pkgmanage::auth::authorization_for_url(&pkg.url)?;
                let client = IndexClient {
                    index_url: derive_index_url(&pkg.url),
                    cache_dir: (*cache).clone(),
                    max_concurrent: 4,
                    timeout_secs: 60,
                    retry_max: 3,
                    auth_header,
                };
                let file = ReleaseFile {
                    filename: derive_filename(&pkg.url, &pkg.name, &pkg.version),
                    url: pkg.url.clone(),
                    hash: FileHash {
                        algorithm: "sha256".to_string(),
                        digest: pkg.sha256.clone(),
                    },
                    requires_python: None,
                    size: None,
                    upload_time: None,
                    yanked: false,
                    yanked_reason: None,
                    dist_info_metadata: serde_json::Value::Null,
                    source: None,
                };
                client
                    .download_artifact(&pkg.name, &file)
                    .await
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "download {}=={} from {}: {}",
                            pkg.name,
                            pkg.version,
                            pkg.url,
                            e
                        )
                    })?;
                anyhow::Ok(())
            });
        }
        // First-error-wins: abort remaining tasks the moment one fails so
        // tampered shas can't keep eating bandwidth.
        while let Some(joined) = set.join_next().await {
            match joined {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    set.abort_all();
                    return Err(e);
                }
                Err(join_err) => {
                    set.abort_all();
                    return Err(anyhow::anyhow!("download task panicked: {join_err}"));
                }
            }
        }
        Ok(())
    })?;

    Ok(())
}

fn sync_cache_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("MAMBA_CACHE_DIR") {
        return PathBuf::from(d);
    }
    if let Some(xdg) = std::env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(xdg).join("mamba");
    }
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        if cfg!(target_os = "macos") {
            return home.join("Library/Caches/mamba");
        }
        return home.join(".cache/mamba");
    }
    PathBuf::from("/tmp/mamba-cache")
}

/// Strip the path off a wheel URL to get a usable base for IndexClient. The
/// IndexClient only uses `index_url` for /pypi/.../json calls — for raw
/// download_artifact we just need the scheme+host so retry/error context
/// renders meaningfully.
fn derive_index_url(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("https://") {
        if let Some(slash) = rest.find('/') {
            return format!("https://{}", &rest[..slash]);
        }
        return format!("https://{rest}");
    }
    if let Some(rest) = url.strip_prefix("http://") {
        if let Some(slash) = rest.find('/') {
            return format!("http://{}", &rest[..slash]);
        }
        return format!("http://{rest}");
    }
    url.to_string()
}

/// Recover the wheel/sdist filename from the URL's last path segment. Falls
/// back to `{name}-{version}.unknown` when the URL has no clear filename.
fn derive_filename(url: &str, name: &str, version: &str) -> String {
    if let Some(idx) = url.rfind('/') {
        let tail = &url[idx + 1..];
        if !tail.is_empty() {
            return tail.to_string();
        }
    }
    format!("{name}-{version}.unknown")
}

/// Filter `packages` down to those not yet installed at `site`. `site` is
/// `None` when the venv doesn't exist yet (nothing can be installed there
/// yet), which trivially makes every package pending.
pub(crate) fn plan_install(packages: &[LockedPkg], site: Option<&Path>) -> Vec<LockedPkg> {
    packages
        .iter()
        .filter(|p| !site.is_some_and(|site| is_installed(site, p)))
        .cloned()
        .collect()
}

/// Enumerate distributions installed at `site` whose `(normalized name,
/// version)` matches no entry in `packages` -- the set `mamba sync` must
/// uninstall to converge on `mamba.lock`. Only `*.dist-info` directories are
/// considered: a provider directory written by `materialize_mamba_provider`
/// carries no dist-info and so is never reported or pruned. Sorted by name
/// so the `--check` report is stable.
pub(crate) fn plan_extraneous(packages: &[LockedPkg], site: &Path) -> Vec<(String, String)> {
    let Ok(entries) = fs::read_dir(site) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let s = file_name.to_string_lossy();
        let Some(stem) = s.strip_suffix(".dist-info") else {
            continue;
        };
        let Some((dist, version)) = stem.rsplit_once('-') else {
            continue;
        };
        let normalized = normalize_dist_name(dist);
        let still_locked = packages
            .iter()
            .any(|p| normalize_dist_name(&p.name) == normalized && p.version == version);
        if !still_locked {
            out.push((dist.to_string(), version.to_string()));
        }
    }
    out.sort();
    out
}

/// Locate `name`'s own `*.dist-info` directory at `site`, the same
/// normalized-name match `Installer::uninstall` itself uses.
fn find_dist_info_dir(site: &Path, name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(site).ok()?;
    let target = normalize_dist_name(name);
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let s = file_name.to_string_lossy();
        let Some(stem) = s.strip_suffix(".dist-info") else {
            continue;
        };
        let Some((dist, _ver)) = stem.rsplit_once('-') else {
            continue;
        };
        if normalize_dist_name(dist) == target {
            return Some(entry.path());
        }
    }
    None
}

/// Remove `name`'s installed distribution from `site`, bounded to exactly
/// what its own dist-info owns: the files `Installer::uninstall` removes per
/// RECORD, plus the `.pyc` bytecode cache `import` derives from each
/// RECORD-listed `.py` file, plus any directory left empty by that removal.
/// Never a wholesale directory delete -- a namespace portion shared with a
/// surviving distribution (`ns/two.py` still locked while `ns/one.py`'s
/// distribution is pruned) must keep the survivor's files in place.
///
/// pip's own uninstall algorithm is the model: RECORD names the files a
/// distribution owns, never the derived bytecode cache next to them, so a
/// distribution is not "gone" from the interpreter's point of view until
/// that cache is cleared too -- otherwise a package directory a RECORD-listed
/// `.py` shared with nothing else survives as an importable implicit
/// namespace package with no source, and `import` succeeds instead of
/// raising `ModuleNotFoundError`.
pub(crate) fn prune_distribution(installer: &Installer, site: &Path, name: &str) -> Result<()> {
    // Read the dist-info's own RECORD *before* `Installer::uninstall` deletes
    // it, collecting the `.py` entries: `(parent dir under site, stem)`.
    let py_entries: Vec<(PathBuf, String)> = find_dist_info_dir(site, name)
        .and_then(|dist_info| {
            let dist_info_name = dist_info.file_name()?.to_string_lossy().into_owned();
            let record_path = dist_info.join("RECORD");
            let record_text = fs::read_to_string(&record_path).ok()?;
            let entries = crate::pkgmanage::pkgmgr::installer::record::parse(&record_text).ok()?;
            Some(
                entries
                    .into_iter()
                    .filter(|e| !e.path.starts_with(&format!("{dist_info_name}/")))
                    .filter_map(|e| {
                        let rel = e.path.strip_suffix(".py")?;
                        let (dir, stem) = rel.rsplit_once('/').unwrap_or(("", rel));
                        Some((site.join(dir), stem.to_string()))
                    })
                    .collect(),
            )
        })
        .unwrap_or_default();

    installer
        .uninstall(name, site)
        .map_err(|e| anyhow::anyhow!("uninstall {name}: {e}"))?;

    // Remove the bytecode cache `import` derived from each RECORD-listed
    // `.py` -- `<stem>.cpython-3XY.pyc`, `<stem>.cpython-3XY.opt-1.pyc`,
    // `<stem>.cpython-3XY.opt-2.pyc` -- then walk from that file's own
    // directory up towards `site` (exclusive), best-effort removing each
    // level while it is empty. `fs::remove_dir` refuses on a non-empty
    // directory, which is exactly the boundary a shared namespace portion
    // needs: it stops the walk at the first level another distribution's
    // files still occupy.
    let mut swept_dirs: Vec<PathBuf> = Vec::new();
    for (dir, stem) in &py_entries {
        let pycache = dir.join("__pycache__");
        if let Ok(entries) = fs::read_dir(&pycache) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let s = file_name.to_string_lossy();
                if s.starts_with(stem.as_str())
                    && s.starts_with(&format!("{stem}."))
                    && s.ends_with(".pyc")
                {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
        let _ = fs::remove_dir(&pycache);
        if !swept_dirs.contains(dir) {
            swept_dirs.push(dir.clone());
        }
    }
    for dir in swept_dirs {
        let mut cursor = dir.as_path();
        while cursor != site && cursor.starts_with(site) {
            if fs::remove_dir(cursor).is_err() {
                break;
            }
            let Some(parent) = cursor.parent() else {
                break;
            };
            cursor = parent;
        }
    }

    Ok(())
}

fn is_installed(site: &Path, pkg: &LockedPkg) -> bool {
    if pkg.source_kind == "mamba_provider" {
        return dist_marker_installed(site, pkg)
            && pkg
                .provides
                .iter()
                .all(|alias| provider_alias_installed(site, alias, &pkg.name));
    }
    dist_info_installed(site, &pkg.name, &pkg.version)
}

/// Whether `site` already holds a `<dist>-<version>.dist-info/` directory
/// for `name`/`version` — the RECORD-backed signal a real `Installer`
/// install leaves behind, replacing the retired stub-marker probe
/// (`__init__.py` / `INSTALLER` / `VERSION`) for every non-provider entry.
fn dist_info_installed(site: &Path, name: &str, version: &str) -> bool {
    let Ok(entries) = fs::read_dir(site) else {
        return false;
    };
    let target = normalize_dist_name(name);
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let s = file_name.to_string_lossy();
        let Some(stem) = s.strip_suffix(".dist-info") else {
            continue;
        };
        let Some((dist, ver)) = stem.rsplit_once('-') else {
            continue;
        };
        if normalize_dist_name(dist) == target && ver == version {
            return true;
        }
    }
    false
}

/// PEP 503-style normalization used to match a dist-info directory name
/// back to a locked package name: lowercase, runs of `[-_.]` collapse to a
/// single `-`.
fn normalize_dist_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_sep = false;
    for c in name.chars() {
        if c == '-' || c == '_' || c == '.' {
            if !prev_sep {
                out.push('-');
                prev_sep = true;
            }
        } else {
            out.push(c.to_ascii_lowercase());
            prev_sep = false;
        }
    }
    out
}

fn dist_marker_installed(site: &Path, pkg: &LockedPkg) -> bool {
    let dir = site.join(normalize_module_name(&pkg.name));
    dir.join("__init__.py").exists()
        && dir.join("INSTALLER").exists()
        && fs::read_to_string(dir.join("VERSION"))
            .ok()
            .map(|v| v.trim() == pkg.version)
            .unwrap_or(false)
}

fn materialize_mamba_provider(site: &Path, pkg: &LockedPkg) -> Result<()> {
    let provider_pkg = provider::locked_mamba_package(
        &pkg.name,
        &pkg.version,
        &pkg.provider,
        &pkg.provides,
        &pkg.compatibility,
        &pkg.maturity,
    )?;
    for alias in &provider_pkg.provides {
        ensure_provider_alias_available(site, alias, &provider_pkg.distribution)?;
    }
    for file in provider::provider_files(&provider_pkg)? {
        let path = site.join(&file.relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(&path, file.body).with_context(|| format!("write {}", path.display()))?;
    }

    let dist_dir = site.join(normalize_module_name(&pkg.name));
    fs::create_dir_all(&dist_dir).with_context(|| format!("create {}", dist_dir.display()))?;
    fs::write(dist_dir.join("INSTALLER"), b"mamba\n")
        .with_context(|| format!("write {}/INSTALLER", dist_dir.display()))?;
    fs::write(dist_dir.join("VERSION"), format!("{}\n", pkg.version))
        .with_context(|| format!("write {}/VERSION", dist_dir.display()))?;
    fs::write(dist_dir.join("PROVIDER"), format!("{}\n", pkg.provider))
        .with_context(|| format!("write {}/PROVIDER", dist_dir.display()))?;
    Ok(())
}

fn provider_alias_installed(site: &Path, alias: &str, distribution: &str) -> bool {
    let init = site.join(normalize_module_name(alias)).join("__init__.py");
    fs::read_to_string(init)
        .ok()
        .map(|body| {
            body.contains(&format!(
                "__mamba_provider_distribution__ = {:?}",
                distribution
            ))
        })
        .unwrap_or(false)
}

fn ensure_provider_alias_available(site: &Path, alias: &str, distribution: &str) -> Result<()> {
    let module = normalize_module_name(alias);
    let package_dir = site.join(&module);
    let init = package_dir.join("__init__.py");
    let module_file = site.join(format!("{module}.py"));
    if module_file.exists() {
        bail!(
            "mamba provider package `{distribution}` would overwrite existing module file `{}`",
            module_file.display()
        );
    }
    if init.exists() {
        let body = fs::read_to_string(&init).with_context(|| format!("read {}", init.display()))?;
        if !body.contains(&format!(
            "__mamba_provider_distribution__ = {:?}",
            distribution
        )) {
            bail!(
                "mamba provider package `{distribution}` would overwrite existing import package `{alias}`"
            );
        }
    } else if package_dir.exists() {
        bail!(
            "mamba provider package `{distribution}` would overwrite existing import package directory `{}`",
            package_dir.display()
        );
    }
    Ok(())
}

/// Python import name normalization: `-` and `.` become `_`, lowercase.
fn normalize_module_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if c == '-' || c == '.' {
            out.push('_');
        } else {
            out.push(c.to_ascii_lowercase());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_lock_is_ok() {
        let pkgs = parse_locked_packages("format_version = 1\ninput_hash = \"x\"\n").unwrap();
        assert!(pkgs.is_empty());
    }

    #[test]
    fn parse_one_package() {
        let src = r#"
format_version = 1
input_hash = "x"

[[package]]
name = "foo"
version = "1.2.3"
sha256 = ""
source = "pypi://foo/1.2.3"
dependencies = []
"#;
        let pkgs = parse_locked_packages(src).unwrap();
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0].name, "foo");
        assert_eq!(pkgs[0].version, "1.2.3");
    }

    #[test]
    fn parse_provider_metadata() {
        let src = r#"
format_version = 1
input_hash = "x"

[[package]]
name = "mamba-httpx-compat"
version = "0.1.0"
sha256 = ""
url = ""
source_kind = "mamba_provider"
provider = "mamba"
provides = ["httpx"]
compatibility = "httpx"
maturity = "experimental"
source = "mamba-provider://mamba/mamba-httpx-compat/0.1.0"
dependencies = []
"#;
        let pkgs = parse_locked_packages(src).unwrap();
        assert_eq!(pkgs[0].provider, "mamba");
        assert_eq!(pkgs[0].provides, vec!["httpx"]);
        assert_eq!(pkgs[0].compatibility, "httpx");
    }

    #[test]
    fn normalize_module() {
        assert_eq!(normalize_module_name("Foo.Bar-baz"), "foo_bar_baz");
        assert_eq!(normalize_module_name("plain"), "plain");
    }
}

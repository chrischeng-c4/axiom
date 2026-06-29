// `uv pip install` / `uv pip sync` / `uv pip uninstall` offline install path.
//
// This layer is intentionally local-first: it installs direct wheel files and
// frozen-index package requirements into a selected site-packages directory,
// then uses installed RECORD files for uninstall/prune.

use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::pip_check::version_satisfies_all;
use crate::pkgmanage::pkgmgr::pip_inventory::{
    InstalledDist, enumerate_installed, find_by_name, parse_metadata,
};
use crate::pkgmanage::pkgmgr::pip_registry::{download_registry_artifact, resolve_registry};
use crate::pkgmanage::pkgmgr::requirements_loader::{LoadedRequirements, load_requirements_file};
use crate::pkgmanage::pkgmgr::requirements_parse::{
    PackageRequirement, RequirementLine, parse_one_line,
};
use crate::pkgmanage::pkgmgr::{
    InstallKind, InstallMode, InstallRequest, Installer, name_normalize::pep503_normalize, pep440,
};

#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub site_packages: PathBuf,
    pub python: PathBuf,
    pub index: Option<InstallIndex>,
}

#[derive(Debug, Clone)]
pub enum InstallIndex {
    Frozen(PathBuf),
    Registry(String),
}

#[derive(Debug, Clone)]
pub enum InstallSource {
    Wheel(PathBuf),
    Requirement(PackageRequirement),
}

#[derive(Debug, Default)]
struct InstallState {
    seen_wheels: BTreeSet<PathBuf>,
    desired: BTreeSet<String>,
    actions: Vec<String>,
}

pub fn install_sources(sources: &[InstallSource], opts: &InstallOptions) -> Result<Vec<String>> {
    let mut state = InstallState::default();
    for source in sources {
        install_source(source, opts, &mut state)?;
    }
    Ok(state.actions)
}

pub fn sync_sources(sources: &[InstallSource], opts: &InstallOptions) -> Result<Vec<String>> {
    let mut state = InstallState::default();
    for source in sources {
        install_source(source, opts, &mut state)?;
    }

    let installed = enumerate_installed(&opts.site_packages);
    let installer = Installer::new();
    for dist in installed {
        if state.desired.contains(&dist.canonical_name) {
            continue;
        }
        installer.uninstall(&dist.name, &opts.site_packages)?;
        state
            .actions
            .push(format!("uninstalled {}=={}", dist.name, dist.version));
    }

    Ok(state.actions)
}

pub fn uninstall_packages(packages: &[String], site_packages: &Path) -> Result<Vec<String>> {
    let installer = Installer::new();
    let mut actions = Vec::with_capacity(packages.len());
    for package in packages {
        installer.uninstall(package, site_packages)?;
        actions.push(format!("uninstalled {package}"));
    }
    Ok(actions)
}

pub fn parse_install_source(raw: &str) -> Result<InstallSource> {
    if looks_like_wheel_path(raw) {
        return Ok(InstallSource::Wheel(PathBuf::from(raw)));
    }
    match parse_one_line(raw)? {
        RequirementLine::Package(package) => Ok(InstallSource::Requirement(package)),
        other => bail!("unsupported pip install input `{raw}`: {other:?}"),
    }
}

pub fn load_requirements_sources(path: &Path) -> Result<Vec<InstallSource>> {
    let loaded = load_requirements_file(path)?;
    reject_unsupported_loaded_requirements(&loaded, path)?;
    Ok(loaded
        .primary
        .into_iter()
        .map(InstallSource::Requirement)
        .collect())
}

fn reject_unsupported_loaded_requirements(loaded: &LoadedRequirements, path: &Path) -> Result<()> {
    if !loaded.constraints.is_empty() {
        bail!(
            "{} uses constraints; mamba pip install/sync does not resolve constraints yet",
            path.display()
        );
    }
    if !loaded.editables.is_empty() {
        bail!(
            "{} uses editable requirements; mamba pip install/sync does not install editables yet",
            path.display()
        );
    }
    if !loaded.unknown.is_empty() {
        bail!(
            "{} has unsupported requirement directives: {}",
            path.display(),
            loaded.unknown.join(", ")
        );
    }
    Ok(())
}

fn install_source(
    source: &InstallSource,
    opts: &InstallOptions,
    state: &mut InstallState,
) -> Result<()> {
    fs::create_dir_all(&opts.site_packages)
        .with_context(|| format!("create {}", opts.site_packages.display()))?;
    match source {
        InstallSource::Wheel(path) => install_wheel_recursive(path, opts, state),
        InstallSource::Requirement(req) => install_requirement(req, opts, state),
    }
}

fn install_requirement(
    req: &PackageRequirement,
    opts: &InstallOptions,
    state: &mut InstallState,
) -> Result<()> {
    if let Some(existing) = installed_satisfying(req, &opts.site_packages) {
        let first_seen = state.desired.insert(existing.canonical_name.clone());
        state.actions.push(format!(
            "already installed {}=={}",
            existing.name, existing.version
        ));
        if first_seen {
            install_metadata_dependencies(&existing.requires, opts, state, &existing.name)?;
        }
        return Ok(());
    }

    if let Some(url) = &req.direct_url {
        if let Some(path) = url.strip_prefix("file://") {
            return install_wheel_recursive(&PathBuf::from(path), opts, state);
        }
        bail!(
            "direct URL requirement for `{}` is not a local file URL: {url}",
            req.name
        );
    }

    match opts.index.as_ref() {
        Some(InstallIndex::Frozen(index)) => {
            let wheel = find_index_wheel(req, index)?;
            install_wheel_recursive(&wheel, opts, state)
        }
        Some(InstallIndex::Registry(index_url)) => {
            install_registry_requirement(req, index_url, opts, state)
        }
        None => bail!(
            "mamba pip install/sync requires --index DIR or --index-url URL for package requirements"
        ),
    }
}

fn install_registry_requirement(
    req: &PackageRequirement,
    index_url: &str,
    opts: &InstallOptions,
    state: &mut InstallState,
) -> Result<()> {
    let resolved = resolve_registry(index_url, std::slice::from_ref(req))?;
    for node in &resolved.graph.nodes {
        let pinned_req = PackageRequirement {
            name: node.name.clone(),
            specifiers: vec![format!("=={}", node.version)],
            ..PackageRequirement::default()
        };
        if let Some(existing) = installed_satisfying(&pinned_req, &opts.site_packages) {
            state.desired.insert(existing.canonical_name.clone());
            state.actions.push(format!(
                "already installed {}=={}",
                existing.name, existing.version
            ));
            continue;
        }
        let file = resolved
            .artifacts
            .get(&node.name)
            .with_context(|| format!("missing artifact for {}=={}", node.name, node.version))?;
        let wheel = download_registry_artifact(index_url, &node.name, file)?;
        let _ = install_wheel_once(&wheel, opts, state)?;
    }
    Ok(())
}

fn install_wheel_recursive(
    wheel: &Path,
    opts: &InstallOptions,
    state: &mut InstallState,
) -> Result<()> {
    if let Some(metadata) = install_wheel_once(wheel, opts, state)? {
        install_metadata_dependencies(&metadata.requires, opts, state, &metadata.name)?;
    }
    Ok(())
}

fn install_wheel_once(
    wheel: &Path,
    opts: &InstallOptions,
    state: &mut InstallState,
) -> Result<Option<InstalledDist>> {
    let wheel = absolute_path(wheel)?;
    if !state.seen_wheels.insert(wheel.clone()) {
        return Ok(None);
    }

    let metadata = read_wheel_metadata(&wheel)?;
    state.desired.insert(metadata.canonical_name.clone());

    let result = Installer::new().install(InstallRequest {
        artifact_path: wheel.clone(),
        site_packages: opts.site_packages.clone(),
        python_executable: opts.python.clone(),
        mode: InstallMode::Purelib,
    })?;
    let verb = match result.kind {
        InstallKind::Installed => "installed",
        InstallKind::AlreadyInstalled => "already installed",
    };
    state.actions.push(format!(
        "{verb} {}=={}",
        result.distribution, result.version
    ));

    Ok(Some(metadata))
}

fn install_metadata_dependencies(
    requires: &[String],
    opts: &InstallOptions,
    state: &mut InstallState,
    context: &str,
) -> Result<()> {
    for dep in requires {
        if is_extra_marker(dep) {
            continue;
        }
        match parse_one_line(dep)? {
            RequirementLine::Package(req) => install_requirement(&req, opts, state)?,
            other => bail!("unsupported dependency metadata for {context}: {other:?}"),
        }
    }
    Ok(())
}

fn installed_satisfying(req: &PackageRequirement, site: &Path) -> Option<InstalledDist> {
    let dists = enumerate_installed(site);
    let dist = find_by_name(&dists, &req.name)?.clone();
    if req.specifiers.is_empty() || version_satisfies_all(&dist.version, &req.specifiers) {
        return Some(dist);
    }
    None
}

pub(crate) fn find_index_wheel(req: &PackageRequirement, index: &Path) -> Result<PathBuf> {
    let pkg_dir = index.join(pep503_normalize(&req.name));
    if !pkg_dir.exists() {
        bail!(
            "package `{}` not found in frozen index {}",
            req.name,
            index.display()
        );
    }
    let versions = candidate_versions(&pkg_dir, &req.specifiers)?;
    for version in versions {
        if let Some(wheel) = first_wheel_in(&pkg_dir.join(&version))? {
            return Ok(wheel);
        }
    }
    bail!(
        "no wheel candidate for `{}` matching {:?} in {}",
        req.name,
        req.specifiers,
        index.display()
    )
}

fn candidate_versions(pkg_dir: &Path, specifiers: &[String]) -> Result<Vec<String>> {
    let mut versions = fs::read_dir(pkg_dir)
        .with_context(|| format!("read package index {}", pkg_dir.display()))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if !entry.path().is_dir() {
                return None;
            }
            entry.file_name().to_str().map(str::to_string)
        })
        .filter(|version| specifiers.is_empty() || version_satisfies_all(version, specifiers))
        .collect::<Vec<_>>();
    versions.sort_by(|a, b| compare_versions_desc(a, b));
    Ok(versions)
}

fn first_wheel_in(version_dir: &Path) -> Result<Option<PathBuf>> {
    let mut wheels = fs::read_dir(version_dir)
        .with_context(|| format!("read version index {}", version_dir.display()))?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension().and_then(|s| s.to_str()) == Some("whl") {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    wheels.sort();
    Ok(wheels.into_iter().next())
}

fn compare_versions_desc(a: &str, b: &str) -> std::cmp::Ordering {
    match (pep440::parse(a), pep440::parse(b)) {
        (Some(left), Some(right)) => right.cmp(&left),
        _ => b.cmp(a),
    }
}

pub(crate) fn read_wheel_metadata(path: &Path) -> Result<InstalledDist> {
    let file = fs::File::open(path).with_context(|| format!("open wheel {}", path.display()))?;
    let mut zip =
        zip::ZipArchive::new(file).with_context(|| format!("read wheel {}", path.display()))?;
    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .with_context(|| format!("read wheel entry {i} in {}", path.display()))?;
        let name = entry.name().to_string();
        if !name.ends_with("/METADATA") || !name.contains(".dist-info/") {
            continue;
        }
        let mut body = String::new();
        use std::io::Read;
        entry
            .read_to_string(&mut body)
            .with_context(|| format!("read METADATA in {}", path.display()))?;
        if let Some(dist) = parse_metadata(&body, Path::new(&name)) {
            return Ok(dist);
        }
        bail!("wheel {} METADATA missing Name or Version", path.display());
    }
    bail!("wheel {} has no dist-info/METADATA", path.display())
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(std::env::current_dir()?.join(path))
}

fn looks_like_wheel_path(raw: &str) -> bool {
    raw.ends_with(".whl") || raw.contains('/') || (cfg!(windows) && raw.contains('\\'))
}

pub(crate) fn is_extra_marker(req: &str) -> bool {
    let Some((_, marker)) = req.split_once(';') else {
        return false;
    };
    let marker = marker.trim();
    marker.contains("extra ==") || marker.contains("extra==")
}

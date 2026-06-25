// Shared explicit-index support for `mamba pip compile/install/sync`.
//
// This keeps pip-family registry resolution on the same IndexClient +
// resolver path as `mamba add` / `mamba lock`, including stored auth,
// marker filtering, yanked filtering, cache use, and tag-aware wheel choice.

use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::pkgmanage::pkgmgr::requirements_parse::PackageRequirement;
use crate::pkgmanage::pkgmgr::resolver::pubgrub_glue::IndexClientProvider;
use crate::pkgmanage::pkgmgr::resolver::{Requirement, ResolvedGraph, Resolver, parse_requirement};
use crate::pkgmanage::pkgmgr::tags::{TagSelector, parse_wheel_filename};
use crate::pkgmanage::pkgmgr::types::{FileHash, IndexClient, ReleaseFile};

#[derive(Debug, Clone)]
pub struct RegistryResolution {
    pub graph: ResolvedGraph,
    pub artifacts: BTreeMap<String, ReleaseFile>,
}

pub fn resolve_registry(
    index_url: &str,
    reqs: &[PackageRequirement],
) -> Result<RegistryResolution> {
    use crate::pkgmanage::pkgmgr::markers::{MarkerEnv, evaluate as eval_marker};

    let roots = reqs
        .iter()
        .map(package_to_resolver_requirement)
        .collect::<Result<Vec<_>>>()?;
    let client = index_client(index_url)?;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .context("build tokio runtime for pip registry resolve")?;
    let handle = rt.handle().clone();
    let provider = IndexClientProvider::new(client.clone(), handle.clone());
    let host_env = MarkerEnv::current_host();
    let resolver = Resolver::new(provider).with_marker_eval(move |_version, marker| match marker {
        Some(m) => match eval_marker(m, &host_env) {
            Ok(v) => !v,
            Err(_) => false,
        },
        None => false,
    });
    let graph = resolver
        .resolve(&roots)
        .map_err(|e| anyhow::anyhow!("resolution failed: {e}"))?;
    let selector = TagSelector::current_host();
    let mut artifacts = BTreeMap::new();
    for node in &graph.nodes {
        let file = pick_release_file(&client, &handle, &node.name, &node.version, &selector)?
            .with_context(|| {
                format!(
                    "no downloadable artifact for {}=={}",
                    node.name, node.version
                )
            })?;
        artifacts.insert(node.name.clone(), file);
    }
    Ok(RegistryResolution { graph, artifacts })
}

pub fn download_registry_artifact(
    index_url: &str,
    name: &str,
    file: &ReleaseFile,
) -> Result<PathBuf> {
    let client = index_client(index_url)?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("build tokio runtime for pip registry artifact download")?;
    rt.block_on(client.download_artifact(name, file))
        .map_err(|e| anyhow::anyhow!("download {name} from {}: {e}", file.url))
}

pub fn package_to_resolver_requirement(req: &PackageRequirement) -> Result<Requirement> {
    if req.direct_url.is_some() {
        bail!(
            "explicit registry resolution does not support direct URL requirement for `{}`",
            req.name
        );
    }
    let mut raw = req.name.clone();
    if !req.extras.is_empty() {
        raw.push('[');
        raw.push_str(&req.extras.join(","));
        raw.push(']');
    }
    if !req.specifiers.is_empty() {
        raw.push_str(&req.specifiers.join(","));
    }
    if let Some(marker) = &req.marker {
        raw.push_str(" ; ");
        raw.push_str(marker);
    }
    parse_requirement(&raw).map_err(|e| anyhow::anyhow!("requirement parse: {e}"))
}

fn index_client(index_url: &str) -> Result<IndexClient> {
    let cache_dir = pypi_cache_dir();
    Ok(IndexClient {
        index_url: index_url.trim_end_matches('/').to_string(),
        cache_dir: cache_dir.to_string_lossy().into_owned(),
        max_concurrent: 8,
        timeout_secs: 30,
        retry_max: 3,
        auth_header: crate::pkgmanage::auth::authorization_for_url(index_url)?,
    })
}

fn pick_release_file(
    client: &IndexClient,
    runtime: &tokio::runtime::Handle,
    name: &str,
    version: &str,
    selector: &TagSelector,
) -> Result<Option<ReleaseFile>> {
    let meta = runtime
        .block_on(client.fetch_metadata(name))
        .map_err(|e| anyhow::anyhow!("fetch metadata for {name}: {e}"))?;
    let Some(files) = meta.releases.get(version) else {
        return Ok(None);
    };
    let mut best: Option<(u32, ReleaseFile)> = None;
    let mut fallback_wheel: Option<ReleaseFile> = None;
    let mut fallback_sdist: Option<ReleaseFile> = None;
    for file in files {
        if file.yanked || !has_sha256(&file.hash) {
            continue;
        }
        if let Some(wheel_tag) = parse_wheel_filename(&file.filename) {
            if let Some(score) = selector.score(&wheel_tag) {
                if best
                    .as_ref()
                    .map(|(current, _)| score > *current)
                    .unwrap_or(true)
                {
                    best = Some((score, file.clone()));
                }
            } else if fallback_wheel.is_none() {
                fallback_wheel = Some(file.clone());
            }
        } else if fallback_sdist.is_none() {
            fallback_sdist = Some(file.clone());
        }
    }
    Ok(best
        .map(|(_, file)| file)
        .or(fallback_wheel)
        .or(fallback_sdist))
}

fn has_sha256(hash: &FileHash) -> bool {
    hash.algorithm == "sha256" && !hash.digest.is_empty()
}

fn pypi_cache_dir() -> PathBuf {
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

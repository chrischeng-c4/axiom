// `mamba lock` — regenerate mamba.lock from mamba.toml against a frozen
// local index, resolving transitive deps.
//
// Acceptance (tests/governance/gates/pkgmgr/lock/manifest.toml, schema gate
// pkgmgr_lock_fixture_2682.rs):
//
//   - Lockfile contains direct AND transitive deps.
//   - Lockfile distinguishes direct from transitive (per-package `direct`
//     bool).
//   - No package files installed (lock-only path: never touches .venv /
//     site-packages).
//   - Failure to resolve a dep exits 1 with stderr containing
//     "no candidate" + the failing dep name; no partial lockfile is
//     written.
//   - Byte-identical on replay.
//
// Index layout consumed:
//   <INDEX>/<normalized_name>/<version>/                # presence => known
//   <INDEX>/<normalized_name>/<version>/metadata.toml   # optional;
//     requires = ["other_pkg==X.Y.Z", ...]              # transitive edges

use anyhow::{bail, Context, Result};
use clap::ArgMatches;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use crate::pkgmanage::add::{
    append_lock_source_fields, atomic_write, dep_name, source_meta_from_manifest, ManifestState,
    SourceMeta,
};
use crate::pkgmanage::pkgmgr::pip_install::is_extra_marker;

const MANIFEST_FILE: &str = "mamba.toml";
const LOCKFILE_FILE: &str = "mamba.lock";
const FROZEN_INDEX_ENV: &str = "MAMBA_FROZEN_INDEX";
const INDEX_URL_ENV: &str = "MAMBA_INDEX_URL";

pub fn cmd_lock(sub: &ArgMatches) -> Result<()> {
    let project_dir = std::env::current_dir().context("read current directory")?;
    let manifest_path = project_dir.join(MANIFEST_FILE);
    if !manifest_path.exists() {
        bail!(
            "no {MANIFEST_FILE} in {} — run `mamba init` first",
            project_dir.display()
        );
    }

    let manifest_src = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let state = ManifestState::parse(&manifest_src)?;

    let offline = sub.get_flag("offline");
    let registry_deps = registry_dependency_strings(&state);
    let body = if let Some(idx) = resolve_index_dir(sub) {
        resolve_and_render_via_index(&state, &idx)?
    } else if registry_deps.is_empty() {
        let mut resolved = resolve_manifest_provider_deps(&state)?;
        resolved.sort_by(|a, b| a.pin.name.cmp(&b.pin.name));
        render_lockfile(&state.dependencies, &resolved)
    } else if offline {
        bail!(
            "no frozen index configured and --offline set (pass --index DIR \
             or set {FROZEN_INDEX_ENV})"
        );
    } else {
        match resolve_index_url(sub) {
            Some(index_url) => resolve_and_render_via_registry(&state, &index_url)?,
            None => bail!(
                "no package source configured for `mamba lock`; pass --index DIR, \
                 set {FROZEN_INDEX_ENV}, pass --index-url URL, or set {INDEX_URL_ENV}"
            ),
        }
    };

    let lock_path = project_dir.join(LOCKFILE_FILE);
    if sub.get_flag("check") {
        let existing = fs::read_to_string(&lock_path)
            .with_context(|| format!("read {}", lock_path.display()))?;
        if existing != body {
            bail!("{LOCKFILE_FILE} is out of date; run `mamba lock`");
        }
        println!("{LOCKFILE_FILE} is up to date");
        return Ok(());
    }
    atomic_write(&lock_path, body.as_bytes())?;
    Ok(())
}

fn resolve_index_url(sub: &ArgMatches) -> Option<String> {
    sub.get_one::<String>("index-url")
        .cloned()
        .or_else(|| std::env::var(INDEX_URL_ENV).ok())
}

fn resolve_via_pypi(deps: &[String], index_url: &str) -> Result<Vec<Resolved>> {
    use crate::pkgmanage::pkgmgr::http::index_client_for_url;
    use crate::pkgmanage::pkgmgr::markers::{evaluate as eval_marker, MarkerEnv};
    use crate::pkgmanage::pkgmgr::resolver::pubgrub_glue::IndexClientProvider;
    use crate::pkgmanage::pkgmgr::resolver::{parse_requirement, Resolver};

    let roots: Vec<crate::pkgmanage::pkgmgr::resolver::Requirement> = deps
        .iter()
        .map(|d| parse_requirement(d))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("requirement parse: {e}"))?;
    let root_names: BTreeSet<String> = roots.iter().map(|r| r.name.clone()).collect();

    let cache_dir = pypi_cache_dir();
    let client = index_client_for_url(
        index_url,
        cache_dir.to_string_lossy().into_owned(),
        8,
        30,
        3,
        crate::pkgmanage::auth::authorization_for_url(index_url)?,
    );

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .context("build tokio runtime for PyPI resolve")?;

    let handle = rt.handle().clone();
    // Tick 15: keep a sibling client+handle alive past the resolver move so we
    // can fetch artifact URLs after resolution without re-creating the runtime.
    let url_client = client.clone();
    let url_handle = handle.clone();
    let provider = IndexClientProvider::new(client, handle);

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

    let mut out = Vec::with_capacity(graph.nodes.len());
    let selector = crate::pkgmanage::pkgmgr::tags::TagSelector::current_host();
    // Build name → pinned-version map across the resolved graph so we can
    // emit transitive edges as `name==version` (matching the local-frozen
    // path) instead of bare names.
    let pinned: BTreeMap<String, String> = graph
        .nodes
        .iter()
        .map(|n| (n.name.clone(), n.version.clone()))
        .collect();
    for node in &graph.nodes {
        // Look up the canonical artifact through the sibling client. Tick
        // 15: the URL travels with the sha so `mamba sync` can perform a
        // download_artifact() + sha-verify pass without re-resolving. #4220:
        // `sha256` and `url` must name the same file, so both come from the
        // one paired selection below rather than `sha256` from `node.files`
        // (page order) and `url` from an independent pick.
        let picked = pick_artifact_url(
            &url_client,
            &url_handle,
            &node.name,
            &node.version,
            &selector,
        );
        let (sha, url) = match picked {
            Some((url, sha256)) => (Some(sha256), Some(url)),
            // Metadata fetch failed or no acceptable file: preserve today's
            // behaviour of falling back to the resolver's own first sha256,
            // with no url.
            None => (
                node.files
                    .iter()
                    .find(|h| h.algorithm == "sha256" && !h.digest.is_empty())
                    .map(|h| h.digest.clone()),
                None,
            ),
        };
        out.push(Resolved {
            pin: Pin {
                name: node.name.clone(),
                version: node.version.clone(),
            },
            direct: root_names.contains(&node.name),
            requires: node
                .requires
                .iter()
                .map(|r| match pinned.get(&r.name) {
                    Some(v) => format!("{}=={}", r.name, v),
                    None => r.name.clone(),
                })
                .collect(),
            sha256: sha,
            url,
            source: SourceMeta::Default,
        });
    }
    out.sort_by(|a, b| a.pin.name.cmp(&b.pin.name));
    Ok(out)
}

/// Pure artifact-selection seam: given one release's files, pick the same
/// artifact `mamba add`'s `pick_best_wheel` would (best-scoring wheel for
/// the host per PEP 425, falling back to any non-yanked wheel, then sdist)
/// and return the chosen file's `(url, sha256)` together, so `mamba lock`
/// can never pair one file's digest with a different file's url (#4220).
pub(crate) fn select_artifact(
    files: &[crate::pkgmanage::pkgmgr::types::ReleaseFile],
    selector: &crate::pkgmanage::pkgmgr::tags::TagSelector,
) -> Option<(String, String)> {
    use crate::pkgmanage::pkgmgr::tags::parse_wheel_filename;
    let mut best: Option<(u32, String, String)> = None;
    let mut fallback_wheel: Option<(String, String)> = None;
    let mut fallback_sdist: Option<(String, String)> = None;
    for f in files {
        if f.yanked || f.hash.algorithm != "sha256" || f.hash.digest.is_empty() {
            continue;
        }
        if let Some(wt) = parse_wheel_filename(&f.filename) {
            if let Some(score) = selector.score(&wt) {
                if best.as_ref().map(|(s, _, _)| score > *s).unwrap_or(true) {
                    best = Some((score, f.url.clone(), f.hash.digest.clone()));
                }
            } else if fallback_wheel.is_none() {
                fallback_wheel = Some((f.url.clone(), f.hash.digest.clone()));
            }
        } else if fallback_sdist.is_none() {
            fallback_sdist = Some((f.url.clone(), f.hash.digest.clone()));
        }
    }
    best.map(|(_, u, s)| (u, s))
        .or(fallback_wheel)
        .or(fallback_sdist)
}

/// Choose the canonical artifact `(url, sha256)` pair for a resolved
/// (name, version) pair. Delegates to `select_artifact` so there is one
/// selection path shared by every caller.
fn pick_artifact_url(
    client: &crate::pkgmanage::pkgmgr::IndexClient,
    runtime: &tokio::runtime::Handle,
    name: &str,
    version: &str,
    selector: &crate::pkgmanage::pkgmgr::tags::TagSelector,
) -> Option<(String, String)> {
    let meta = runtime.block_on(client.fetch_metadata(name)).ok()?;
    let files = meta.releases.get(version)?;
    select_artifact(files, selector)
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

fn resolve_index_dir(sub: &ArgMatches) -> Option<PathBuf> {
    sub.get_one::<String>("index")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os(FROZEN_INDEX_ENV).map(PathBuf::from))
}

#[derive(Debug, Clone)]
pub(crate) struct Pin {
    pub(crate) name: String,
    pub(crate) version: String,
}

impl Pin {
    pub(crate) fn parse(spec: &str) -> Result<Self> {
        let (n, v) = spec
            .split_once("==")
            .with_context(|| format!("malformed dep spec `{spec}` (expected NAME==VERSION)"))?;
        let n = n.trim();
        let v = v.trim();
        if n.is_empty() || v.is_empty() {
            bail!("malformed dep spec `{spec}` (expected NAME==VERSION)");
        }
        Ok(Pin {
            name: n.to_string(),
            version: v.to_string(),
        })
    }

    fn key(&self) -> String {
        format!("{}=={}", self.name, self.version)
    }
}

/// PEP 503 normalize: lowercase + collapse `-`/`_`/`.` to single `-`.
fn normalize_name(name: &str) -> String {
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

#[derive(Debug, Clone)]
pub(crate) struct Resolved {
    pub(crate) pin: Pin,
    pub(crate) direct: bool,
    pub(crate) requires: Vec<String>,
    pub(crate) sha256: Option<String>,
    /// Canonical artifact URL for the picked wheel/sdist; populated on the
    /// live-PyPI path so `mamba sync` can fetch + sha-verify. Empty for
    /// frozen-local-index resolves where the URL is not known.
    pub(crate) url: Option<String>,
    pub(crate) source: SourceMeta,
}

fn registry_dependency_strings(state: &ManifestState) -> Vec<String> {
    state
        .dependencies
        .iter()
        .filter(|dep| !state.source_overrides.contains_key(dep_name(dep)))
        .cloned()
        .collect()
}

fn resolve_manifest_provider_deps(state: &ManifestState) -> Result<Vec<Resolved>> {
    let mut out = Vec::new();
    for dep in &state.dependencies {
        let name = dep_name(dep);
        let Some(source) = state.source_overrides.get(name) else {
            continue;
        };
        let pin = Pin::parse(dep)?;
        out.push(Resolved {
            source: source_meta_from_manifest(&pin.name, &pin.version, source)?,
            pin,
            direct: true,
            requires: Vec::new(),
            sha256: None,
            url: None,
        });
    }
    Ok(out)
}

/// Resolve `state`'s registry dependencies against a frozen local index and
/// render the same `mamba.lock` body `mamba lock --index` would write for
/// this manifest, so `mamba add --index` and `mamba lock --index` agree byte
/// for byte on the same project (frozen decision, #4206).
pub(crate) fn resolve_and_render_via_index(state: &ManifestState, index: &Path) -> Result<String> {
    let provider_resolved = resolve_manifest_provider_deps(state)?;
    let registry_deps = registry_dependency_strings(state);
    let mut resolved = if registry_deps.is_empty() {
        Vec::new()
    } else {
        let direct: Vec<Pin> = registry_deps
            .iter()
            .map(|d| Pin::parse(d))
            .collect::<Result<Vec<_>>>()?;
        resolve_transitive(&direct, index)?
    };
    resolved.extend(provider_resolved);
    resolved.sort_by(|a, b| a.pin.name.cmp(&b.pin.name));
    Ok(render_lockfile(&state.dependencies, &resolved))
}

/// Resolve `state`'s registry dependencies against a live PyPI-style index
/// and render the same `mamba.lock` body `mamba lock --index-url` would
/// write for this manifest, so `mamba add --index-url` and `mamba lock
/// --index-url` agree byte for byte through this one seam (#4221).
pub(crate) fn resolve_and_render_via_registry(
    state: &ManifestState,
    index_url: &str,
) -> Result<String> {
    let registry_deps = registry_dependency_strings(state);
    let mut resolved = if registry_deps.is_empty() {
        Vec::new()
    } else {
        resolve_via_pypi(&registry_deps, index_url)?
    };
    resolved.extend(resolve_manifest_provider_deps(state)?);
    resolved.sort_by(|a, b| a.pin.name.cmp(&b.pin.name));
    Ok(render_lockfile(&state.dependencies, &resolved))
}

fn resolve_transitive(direct: &[Pin], index: &Path) -> Result<Vec<Resolved>> {
    let direct_keys: BTreeSet<String> = direct.iter().map(|p| p.key()).collect();
    let mut seen: BTreeMap<String, Resolved> = BTreeMap::new();
    let mut queue: VecDeque<String> = direct.iter().map(|p| p.key()).collect();
    while let Some(raw) = queue.pop_front() {
        let (pin, meta) = load_metadata(&raw, index)?;
        let key = pin.key();
        if seen.contains_key(&key) {
            continue;
        }
        let is_direct = direct_keys.contains(&key);
        let filtered_requires: Vec<String> = meta
            .requires
            .iter()
            .filter(|r| !is_extra_marker(r))
            .cloned()
            .collect();
        let mut requires_keys = Vec::with_capacity(filtered_requires.len());
        for req in &filtered_requires {
            let (dep_pin, _) = load_metadata(req, index)?;
            requires_keys.push(dep_pin.key());
        }
        let source = if meta.path.is_empty() {
            SourceMeta::Default
        } else {
            SourceMeta::Index {
                path: meta.path.clone(),
            }
        };
        seen.insert(
            key,
            Resolved {
                pin: pin.clone(),
                direct: is_direct,
                requires: requires_keys,
                sha256: if meta.sha256.is_empty() {
                    None
                } else {
                    Some(meta.sha256.clone())
                },
                url: None,
                source,
            },
        );
        for req in filtered_requires {
            queue.push_back(req);
        }
    }
    let mut out: Vec<Resolved> = seen.into_values().collect();
    out.sort_by(|a, b| a.pin.name.cmp(&b.pin.name));
    Ok(out)
}

/// Metadata read from `<INDEX>/<name>/<version>/metadata.toml` (or, when a
/// key is absent — the legacy fixture shape `apps/mamba/tests/pkgmgr/fixtures.rs`
/// still writes — derived from the version directory's one wheel).
pub(crate) struct IndexMetadata {
    pub(crate) sha256: String,
    pub(crate) path: String,
    pub(crate) requires: Vec<String>,
}

/// Resolve a requirement (`name`, `name>=1`, `name==1.0`, `name>=1,<3`)
/// against a frozen index: select the highest version directory that
/// satisfies every specifier (PEP 440 comparison, no backtracking, via
/// `pip_install::candidate_versions`, the same selection `pip install`
/// already does against a frozen index), then read that version's metadata.
pub(crate) fn load_metadata(requirement: &str, index: &Path) -> Result<(Pin, IndexMetadata)> {
    use crate::pkgmanage::pkgmgr::pip_install::candidate_versions;
    use crate::pkgmanage::pkgmgr::requirements_parse::{parse_one_line, RequirementLine};

    let req = match parse_one_line(requirement) {
        Ok(RequirementLine::Package(p)) => p,
        _ => bail!("malformed dependency requirement `{requirement}`"),
    };
    let pkg_dir = index.join(normalize_name(&req.name));
    if !pkg_dir.exists() {
        bail!(
            "no candidate for `{}` matching {:?} in index {} (resolver failure)",
            req.name,
            req.specifiers,
            index.display()
        );
    }
    let versions = candidate_versions(&pkg_dir, &req.specifiers)?;
    let version = versions.into_iter().next().with_context(|| {
        format!(
            "no candidate for `{}` matching {:?} in index {}",
            req.name,
            req.specifiers,
            index.display()
        )
    })?;
    let ver_dir = pkg_dir.join(&version);
    let meta_path = ver_dir.join("metadata.toml");
    let mut sha256 = String::new();
    let mut wheel_path: Option<PathBuf> = None;
    let mut requires: Vec<String> = Vec::new();
    if meta_path.exists() {
        let raw = fs::read_to_string(&meta_path)
            .with_context(|| format!("read {}", meta_path.display()))?;
        let doc: toml::Value = raw
            .parse()
            .with_context(|| format!("parse {}", meta_path.display()))?;
        sha256 = doc
            .get("sha256")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if let Some(fname) = doc.get("filename").and_then(|v| v.as_str()) {
            let candidate = ver_dir.join(fname);
            if candidate.is_file() {
                wheel_path = Some(candidate);
            }
        }
        requires = doc
            .get("requires")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
    }
    if wheel_path.is_none() || sha256.is_empty() {
        if let Some(found) = find_single_wheel(&ver_dir)? {
            if sha256.is_empty() {
                sha256 = sha256_of_file(&found)?;
            }
            if wheel_path.is_none() {
                wheel_path = Some(found);
            }
        }
    }
    let path = match wheel_path {
        Some(p) => absolutize(&p)?.to_string_lossy().into_owned(),
        None => String::new(),
    };
    let pin = Pin {
        name: req.name,
        version,
    };
    Ok((pin, IndexMetadata {
        sha256,
        path,
        requires,
    }))
}

/// The one `.whl` file staged in a version directory, if any — the fallback
/// source of `sha256`/`path` for a `metadata.toml` written before those keys
/// existed (`apps/mamba/tests/pkgmgr/fixtures.rs`).
fn find_single_wheel(ver_dir: &Path) -> Result<Option<PathBuf>> {
    if !ver_dir.is_dir() {
        return Ok(None);
    }
    let mut wheels: Vec<PathBuf> = fs::read_dir(ver_dir)
        .with_context(|| format!("read {}", ver_dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("whl"))
        .collect();
    wheels.sort();
    Ok(wheels.into_iter().next())
}

fn sha256_of_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

fn absolutize(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()
            .context("read current directory")?
            .join(path))
    }
}

fn render_lockfile(direct_deps: &[String], resolved: &[Resolved]) -> String {
    let mut input = direct_deps.to_vec();
    input.sort();
    input.dedup();
    let input_hash = compute_input_hash(&input);

    let mut out = String::with_capacity(512);
    out.push_str("format_version = 1\n");
    out.push_str(&format!("input_hash = \"{input_hash}\"\n"));
    for r in resolved {
        out.push('\n');
        out.push_str("[[package]]\n");
        out.push_str(&format!("name = \"{}\"\n", r.pin.name));
        out.push_str(&format!("version = \"{}\"\n", r.pin.version));
        out.push_str(&format!(
            "sha256 = \"{}\"\n",
            r.sha256.as_deref().unwrap_or("")
        ));
        append_lock_source_fields(
            &mut out,
            &r.pin.name,
            &r.pin.version,
            r.url.as_deref().unwrap_or(""),
            &r.source,
        );
        out.push_str(&format!("direct = {}\n", r.direct));
        out.push_str(&format!(
            "dependencies = {}\n",
            render_string_list(&r.requires)
        ));
    }
    out
}

fn render_string_list(items: &[String]) -> String {
    if items.is_empty() {
        return "[]".to_string();
    }
    let mut sorted = items.to_vec();
    sorted.sort();
    sorted.dedup();
    let mut out = String::from("[");
    for (i, item) in sorted.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push('"');
        out.push_str(item);
        out.push('"');
    }
    out.push(']');
    out
}

fn compute_input_hash(deps: &[String]) -> String {
    let mut hasher = Sha256::new();
    for d in deps {
        hasher.update(d.as_bytes());
        hasher.update(b"\n");
    }
    let bytes = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for b in bytes {
        hex.push_str(&format!("{b:02x}"));
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_parse_well_formed() {
        let p = Pin::parse("foo==1.0.0").unwrap();
        assert_eq!(p.name, "foo");
        assert_eq!(p.version, "1.0.0");
    }

    #[test]
    fn pin_parse_rejects_unpinned() {
        assert!(Pin::parse("foo").is_err());
        assert!(Pin::parse("").is_err());
    }

    #[test]
    fn render_is_deterministic() {
        let resolved = vec![
            Resolved {
                pin: Pin {
                    name: "a".into(),
                    version: "1.0".into(),
                },
                direct: true,
                requires: vec!["b==2.0".into()],
                sha256: None,
                url: None,
                source: SourceMeta::Default,
            },
            Resolved {
                pin: Pin {
                    name: "b".into(),
                    version: "2.0".into(),
                },
                direct: false,
                requires: vec![],
                sha256: None,
                url: None,
                source: SourceMeta::Default,
            },
        ];
        let a = render_lockfile(&["a==1.0".to_string()], &resolved);
        let b = render_lockfile(&["a==1.0".to_string()], &resolved);
        assert_eq!(a, b);
    }
}

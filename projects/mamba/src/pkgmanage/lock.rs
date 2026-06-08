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

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use crate::pkgmanage::add::{ManifestState, atomic_write};

const MANIFEST_FILE: &str = "mamba.toml";
const LOCKFILE_FILE: &str = "mamba.lock";
const FROZEN_INDEX_ENV: &str = "MAMBA_FROZEN_INDEX";
const INDEX_URL_ENV: &str = "MAMBA_INDEX_URL";
const DEFAULT_INDEX_URL: &str = "https://pypi.org";

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
    let resolved = match resolve_index_dir(sub) {
        Some(idx) => {
            let direct: Vec<Pin> = state
                .dependencies
                .iter()
                .map(|d| Pin::parse(d))
                .collect::<Result<Vec<_>>>()?;
            resolve_transitive(&direct, &idx)?
        }
        None => {
            if offline {
                bail!(
                    "no frozen index configured and --offline set (pass --index DIR \
                     or drop --offline to use PyPI)"
                );
            }
            let index_url = resolve_index_url(sub);
            resolve_via_pypi(&state.dependencies, &index_url)?
        }
    };

    let body = render_lockfile(&state.dependencies, &resolved);
    let lock_path = project_dir.join(LOCKFILE_FILE);
    atomic_write(&lock_path, body.as_bytes())?;
    Ok(())
}

fn resolve_index_url(sub: &ArgMatches) -> String {
    sub.get_one::<String>("index-url")
        .cloned()
        .or_else(|| std::env::var(INDEX_URL_ENV).ok())
        .unwrap_or_else(|| DEFAULT_INDEX_URL.to_string())
}

fn resolve_via_pypi(deps: &[String], index_url: &str) -> Result<Vec<Resolved>> {
    use crate::pkgmanage::pkgmgr::markers::{evaluate as eval_marker, MarkerEnv};
    use crate::pkgmanage::pkgmgr::resolver::{parse_requirement, Resolver};
    use crate::pkgmanage::pkgmgr::resolver::pubgrub_glue::IndexClientProvider;
    use crate::pkgmanage::pkgmgr::IndexClient;

    let roots: Vec<crate::pkgmanage::pkgmgr::resolver::Requirement> = deps
        .iter()
        .map(|d| parse_requirement(d))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("requirement parse: {e}"))?;
    let root_names: BTreeSet<String> = roots.iter().map(|r| r.name.clone()).collect();

    let cache_dir = pypi_cache_dir();
    let client = IndexClient {
        index_url: index_url.trim_end_matches('/').to_string(),
        cache_dir: cache_dir.to_string_lossy().into_owned(),
        max_concurrent: 8,
        timeout_secs: 30,
        retry_max: 3,
    };

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
        // Resolver carries hashes per node but not filenames — for now we
        // accept the first sha256 it surfaces (host-aware filtering happens
        // in `mamba add` where filenames are available; once the resolver
        // also threads filenames through ResolvedNode we'll switch this to
        // tag scoring too).
        let _ = &selector;
        let sha = node
            .files
            .iter()
            .find(|h| h.algorithm == "sha256" && !h.digest.is_empty())
            .map(|h| h.digest.clone());
        // Look up the canonical artifact URL through the sibling client. Tick
        // 15: the URL travels with the sha so `mamba sync` can perform a
        // download_artifact() + sha-verify pass without re-resolving.
        let url = pick_artifact_url(&url_client, &url_handle, &node.name, &node.version, &selector);
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
        });
    }
    out.sort_by(|a, b| a.pin.name.cmp(&b.pin.name));
    Ok(out)
}

/// Choose the canonical artifact URL for a resolved (name, version) pair,
/// preferring the best-scoring wheel for the current host (PEP 425) and
/// falling back to any non-yanked wheel, then sdist. Mirrors `mamba add`'s
/// `pick_best_wheel` selection logic so `mamba sync` downloads the same
/// artifact that `mamba add` would have recorded.
fn pick_artifact_url(
    client: &crate::pkgmanage::pkgmgr::IndexClient,
    runtime: &tokio::runtime::Handle,
    name: &str,
    version: &str,
    selector: &crate::pkgmanage::pkgmgr::tags::TagSelector,
) -> Option<String> {
    use crate::pkgmanage::pkgmgr::tags::parse_wheel_filename;
    let meta = runtime.block_on(client.fetch_metadata(name)).ok()?;
    let files = meta.releases.get(version)?;
    let mut best: Option<(u32, String)> = None;
    let mut fallback_wheel: Option<String> = None;
    let mut fallback_sdist: Option<String> = None;
    for f in files {
        if f.yanked || f.hash.algorithm != "sha256" || f.hash.digest.is_empty() {
            continue;
        }
        if let Some(wt) = parse_wheel_filename(&f.filename) {
            if let Some(score) = selector.score(&wt) {
                if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                    best = Some((score, f.url.clone()));
                }
            } else if fallback_wheel.is_none() {
                fallback_wheel = Some(f.url.clone());
            }
        } else if fallback_sdist.is_none() {
            fallback_sdist = Some(f.url.clone());
        }
    }
    best.map(|(_, u)| u).or(fallback_wheel).or(fallback_sdist)
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
}

fn resolve_transitive(direct: &[Pin], index: &Path) -> Result<Vec<Resolved>> {
    let direct_keys: BTreeSet<String> = direct.iter().map(|p| p.key()).collect();
    let mut seen: BTreeMap<String, Resolved> = BTreeMap::new();
    let mut queue: VecDeque<Pin> = VecDeque::new();
    for p in direct {
        queue.push_back(p.clone());
    }
    while let Some(pin) = queue.pop_front() {
        let key = pin.key();
        if seen.contains_key(&key) {
            continue;
        }
        let meta = load_metadata(&pin, index)?;
        let requires: Vec<String> = meta
            .iter()
            .map(|p| p.key())
            .collect();
        let is_direct = direct_keys.contains(&key);
        seen.insert(
            key,
            Resolved {
                pin: pin.clone(),
                direct: is_direct,
                requires: requires.clone(),
                sha256: None,
                url: None,
            },
        );
        for r in meta {
            queue.push_back(r);
        }
    }
    let mut out: Vec<Resolved> = seen.into_values().collect();
    out.sort_by(|a, b| a.pin.name.cmp(&b.pin.name));
    Ok(out)
}

fn load_metadata(pin: &Pin, index: &Path) -> Result<Vec<Pin>> {
    let pkg_dir = index.join(normalize_name(&pin.name));
    let ver_dir = pkg_dir.join(&pin.version);
    if !ver_dir.exists() {
        bail!(
            "no candidate for `{name}=={version}` in index {index} (resolver failure)",
            name = pin.name,
            version = pin.version,
            index = index.display()
        );
    }
    let meta_path = ver_dir.join("metadata.toml");
    if !meta_path.exists() {
        return Ok(vec![]);
    }
    let raw = fs::read_to_string(&meta_path)
        .with_context(|| format!("read {}", meta_path.display()))?;
    let doc: toml::Value = raw
        .parse()
        .with_context(|| format!("parse {}", meta_path.display()))?;
    let reqs = doc
        .get("requires")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    reqs.into_iter()
        .map(|s| Pin::parse(&s))
        .collect::<Result<Vec<_>>>()
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
        out.push_str(&format!(
            "url = \"{}\"\n",
            r.url.as_deref().unwrap_or("")
        ));
        out.push_str(&format!(
            "source = \"pypi://{}/{}\"\n",
            r.pin.name, r.pin.version
        ));
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
                pin: Pin { name: "a".into(), version: "1.0".into() },
                direct: true,
                requires: vec!["b==2.0".into()],
                sha256: None,
                url: None,
            },
            Resolved {
                pin: Pin { name: "b".into(), version: "2.0".into() },
                direct: false,
                requires: vec![],
                sha256: None,
                url: None,
            },
        ];
        let a = render_lockfile(&["a==1.0".to_string()], &resolved);
        let b = render_lockfile(&["a==1.0".to_string()], &resolved);
        assert_eq!(a, b);
    }
}

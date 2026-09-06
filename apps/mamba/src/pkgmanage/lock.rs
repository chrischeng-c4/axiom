// `mamba lock` — regenerate mamba.lock from pyproject.toml against a frozen
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
use crate::pkgmanage::manifest::pyproject::{self, LockRoot};
use crate::pkgmanage::pkgmgr::pip_install::is_extra_marker;
use crate::pkgmanage::sync::parse_locked_packages;

const LOCKFILE_FILE: &str = "mamba.lock";
const FROZEN_INDEX_ENV: &str = "MAMBA_FROZEN_INDEX";
const INDEX_URL_ENV: &str = "MAMBA_INDEX_URL";

pub fn cmd_lock(sub: &ArgMatches) -> Result<()> {
    let project_dir = std::env::current_dir().context("read current directory")?;
    let manifest_path = pyproject::locate(&project_dir)?;

    let manifest_src = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let state = ManifestState::parse(&manifest_src)?;

    let offline = sub.get_flag("offline");
    let upgrade_packages: Vec<String> = sub
        .get_many::<String>("upgrade-package")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();
    let prefs = lock_preferences(&project_dir, sub.get_flag("upgrade"), &upgrade_packages);
    let registry_deps = registry_dependency_strings(&state);
    let body = if let Some(idx) = resolve_index_dir(sub) {
        resolve_and_render_via_index(&state, &idx, &prefs)?
    } else if registry_deps.is_empty() {
        let mut resolved = resolve_manifest_provider_deps(&state)?;
        assign_membership(&mut resolved, &state);
        resolved.sort_by(|a, b| a.pin.name.cmp(&b.pin.name));
        render_lockfile(&state, &resolved)
    } else if offline {
        bail!(
            "no frozen index configured and --offline set (pass --index DIR \
             or set {FROZEN_INDEX_ENV})"
        );
    } else {
        match resolve_index_url(sub) {
            Some(index_url) => resolve_and_render_via_registry(&state, &index_url, &prefs)?,
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

/// The versions an existing `mamba.lock` already pinned, PEP 503 name →
/// version. A resolve prefers these over the newest candidate whenever they
/// still satisfy the requirement, so `mamba lock` on an unchanged manifest
/// is a no-op and `mamba add` moves only the package it adds. `--upgrade`
/// empties the map and `--upgrade-package NAME` drops one entry, which is
/// exactly how `uv lock` decides what may move.
pub(crate) type Preferences = BTreeMap<String, String>;

/// Read `project_dir`'s lock into a [`Preferences`] map, minus what the
/// upgrade flags release. A missing or unreadable lock prefers nothing.
pub(crate) fn lock_preferences(
    project_dir: &Path,
    upgrade_all: bool,
    upgrade_packages: &[String],
) -> Preferences {
    if upgrade_all {
        return Preferences::new();
    }
    let Ok(src) = fs::read_to_string(project_dir.join(LOCKFILE_FILE)) else {
        return Preferences::new();
    };
    let Ok(packages) = parse_locked_packages(&src) else {
        return Preferences::new();
    };
    let released: BTreeSet<String> = upgrade_packages.iter().map(|n| normalize_name(n)).collect();
    packages
        .into_iter()
        .filter(|p| !released.contains(&normalize_name(&p.name)))
        .map(|p| (normalize_name(&p.name), p.version))
        .collect()
}

fn resolve_via_pypi(
    deps: &[String],
    index_url: &str,
    prefs: &Preferences,
) -> Result<Vec<Resolved>> {
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
    let resolver = Resolver::new(provider)
        .with_preferences(prefs.clone())
        .with_marker_eval(move |_version, marker| match marker {
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
            project: true,
            groups: Vec::new(),
            extras: Vec::new(),
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
    /// Reachable from `[project] dependencies`. `false` only for a package
    /// that groups or extras alone pull in; `mamba sync` installs it only
    /// when one of those is selected. Set by [`assign_membership`].
    pub(crate) project: bool,
    /// The `[dependency-groups]` groups whose closure contains this package.
    pub(crate) groups: Vec<String>,
    /// The `[project.optional-dependencies]` extras whose closure contains
    /// this package.
    pub(crate) extras: Vec<String>,
}

/// Which manifest lists reach one lock entry. `project` is the plain
/// `[project] dependencies` closure; `groups` and `extras` name every group
/// and extra whose own closure contains the entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Membership {
    pub(crate) project: bool,
    pub(crate) groups: Vec<String>,
    pub(crate) extras: Vec<String>,
}

/// Walk `edges` (index → dependency indices) from every root's entry and
/// record, per entry, which selections reach it. `index` maps a PEP 503
/// name to its entry; a root whose name has no entry contributes nothing.
pub(crate) fn compute_membership(
    edges: &[Vec<usize>],
    index: &BTreeMap<String, usize>,
    roots: &[LockRoot],
) -> Vec<Membership> {
    let mut out: Vec<Membership> = (0..edges.len())
        .map(|_| Membership {
            project: false,
            groups: Vec::new(),
            extras: Vec::new(),
        })
        .collect();
    // (kind, name) → seed entries; kind 0 = project, 1 = group, 2 = extra.
    let mut seeds: BTreeMap<(u8, String), Vec<usize>> = BTreeMap::new();
    for root in roots {
        let Ok(name) = requirement_name(&root.spec) else {
            continue;
        };
        let Some(&i) = index.get(&normalize_name(&name)) else {
            continue;
        };
        if root.project {
            seeds.entry((0, String::new())).or_default().push(i);
        }
        for g in &root.groups {
            seeds.entry((1, g.clone())).or_default().push(i);
        }
        for e in &root.extras {
            seeds.entry((2, e.clone())).or_default().push(i);
        }
    }
    for ((kind, name), starts) in seeds {
        let mut reached: BTreeSet<usize> = BTreeSet::new();
        let mut queue: VecDeque<usize> = starts.into_iter().collect();
        while let Some(i) = queue.pop_front() {
            if !reached.insert(i) {
                continue;
            }
            for &next in &edges[i] {
                if !reached.contains(&next) {
                    queue.push_back(next);
                }
            }
        }
        for i in reached {
            match kind {
                0 => out[i].project = true,
                1 => out[i].groups.push(name.clone()),
                _ => out[i].extras.push(name.clone()),
            }
        }
    }
    for m in &mut out {
        m.groups.sort();
        m.groups.dedup();
        m.extras.sort();
        m.extras.dedup();
    }
    out
}

/// Stamp every resolved node with the manifest lists that reach it, walking
/// each node's own `requires` edges from `state`'s lock roots.
pub(crate) fn assign_membership(resolved: &mut [Resolved], state: &ManifestState) {
    let index: BTreeMap<String, usize> = resolved
        .iter()
        .enumerate()
        .map(|(i, r)| (normalize_name(&r.pin.name), i))
        .collect();
    let edges: Vec<Vec<usize>> = resolved
        .iter()
        .map(|r| {
            r.requires
                .iter()
                .filter_map(|e| requirement_name(e).ok())
                .filter_map(|n| index.get(&normalize_name(&n)).copied())
                .collect()
        })
        .collect();
    let membership = compute_membership(&edges, &index, &state.lock_roots());
    for (r, m) in resolved.iter_mut().zip(membership) {
        r.project = m.project;
        r.groups = m.groups;
        r.extras = m.extras;
    }
}

/// The `project = false` / `groups = […]` / `extras = […]` lines one lock
/// entry carries, each written only when it says something: a package the
/// project dependencies reach and no group or extra names emits nothing, so
/// a lock for a manifest without groups is byte-identical to one written
/// before groups were locked.
pub(crate) fn append_membership_fields(
    out: &mut String,
    project: bool,
    groups: &[String],
    extras: &[String],
) {
    if !project {
        out.push_str("project = false\n");
    }
    if !groups.is_empty() {
        out.push_str(&format!("groups = {}\n", render_string_list(groups)));
    }
    if !extras.is_empty() {
        out.push_str(&format!("extras = {}\n", render_string_list(extras)));
    }
}

fn registry_dependency_strings(state: &ManifestState) -> Vec<String> {
    state
        .all_dependency_specs()
        .into_iter()
        .filter(|dep| !state.source_overrides.contains_key(dep_name(dep)))
        .collect()
}

fn resolve_manifest_provider_deps(state: &ManifestState) -> Result<Vec<Resolved>> {
    let mut out = Vec::new();
    for dep in state.all_dependency_specs() {
        let name = dep_name(&dep);
        let Some(source) = state.source_overrides.get(name) else {
            continue;
        };
        let pin = Pin::parse(&dep)?;
        out.push(Resolved {
            source: source_meta_from_manifest(&pin.name, &pin.version, source)?,
            pin,
            direct: true,
            requires: Vec::new(),
            sha256: None,
            url: None,
            project: true,
            groups: Vec::new(),
            extras: Vec::new(),
        });
    }
    Ok(out)
}

/// Resolve `state`'s registry dependencies against a frozen local index and
/// render the same `mamba.lock` body `mamba lock --index` would write for
/// this manifest, so `mamba add --index` and `mamba lock --index` agree byte
/// for byte on the same project (frozen decision, #4206).
pub(crate) fn resolve_and_render_via_index(
    state: &ManifestState,
    index: &Path,
    prefs: &Preferences,
) -> Result<String> {
    let provider_resolved = resolve_manifest_provider_deps(state)?;
    let registry_deps = registry_dependency_strings(state);
    let mut resolved = if registry_deps.is_empty() {
        Vec::new()
    } else {
        resolve_transitive(&registry_deps, index, prefs)?
    };
    resolved.extend(provider_resolved);
    assign_membership(&mut resolved, state);
    resolved.sort_by(|a, b| a.pin.name.cmp(&b.pin.name));
    Ok(render_lockfile(state, &resolved))
}

/// Resolve `state`'s registry dependencies against a live PyPI-style index
/// and render the same `mamba.lock` body `mamba lock --index-url` would
/// write for this manifest, so `mamba add --index-url` and `mamba lock
/// --index-url` agree byte for byte through this one seam (#4221).
pub(crate) fn resolve_and_render_via_registry(
    state: &ManifestState,
    index_url: &str,
    prefs: &Preferences,
) -> Result<String> {
    let registry_deps = registry_dependency_strings(state);
    let mut resolved = if registry_deps.is_empty() {
        Vec::new()
    } else {
        resolve_via_pypi(&registry_deps, index_url, prefs)?
    };
    resolved.extend(resolve_manifest_provider_deps(state)?);
    assign_membership(&mut resolved, state);
    resolved.sort_by(|a, b| a.pin.name.cmp(&b.pin.name));
    Ok(render_lockfile(state, &resolved))
}

/// The bare package name a raw requirement string (`name`, `name>=1`,
/// `name==1.0`, `name>=1,<3`) names, without resolving it against any index.
fn requirement_name(raw: &str) -> Result<String> {
    use crate::pkgmanage::pkgmgr::requirements_parse::{parse_one_line, RequirementLine};
    match parse_one_line(raw) {
        Ok(RequirementLine::Package(p)) => Ok(p.name),
        _ => bail!("malformed dependency requirement `{raw}`"),
    }
}

/// Whether `candidate` (a PEP 440 version) satisfies every specifier in the
/// raw requirement string `raw`. Used to detect a name reached by two
/// requirements whose specifier sets disagree (#4225), via the same
/// `specifier::all_match` predicate the registry-path resolver
/// (`resolver/mod.rs`) already applies to its own already-decided branch.
fn requirement_matches_version(raw: &str, candidate: &str) -> Result<bool> {
    use crate::pkgmanage::pkgmgr::requirements_parse::{parse_one_line, RequirementLine};
    use crate::pkgmanage::pkgmgr::resolver::specifier;
    let req = match parse_one_line(raw) {
        Ok(RequirementLine::Package(p)) => p,
        _ => bail!("malformed dependency requirement `{raw}`"),
    };
    let specs: Vec<specifier::VersionSpecifier> = req
        .specifiers
        .iter()
        .map(|s| specifier::parse_one(s))
        .collect::<std::result::Result<_, _>>()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(specifier::all_match(&specs, candidate))
}

/// Resolve `roots` (raw requirement strings: `name`, `name==1.0`,
/// `name>=1,<3`) and their closure against a frozen index. A root is
/// `direct`; a version already in `prefs` is kept whenever the requirement
/// still admits it.
fn resolve_transitive(
    roots: &[String],
    index: &Path,
    prefs: &Preferences,
) -> Result<Vec<Resolved>> {
    let direct_names: BTreeSet<String> = roots
        .iter()
        .map(|r| Ok(normalize_name(&requirement_name(r)?)))
        .collect::<Result<_>>()?;
    // Keyed by name (not `name==version`, #4225): two requirements on one
    // name must be compared against the version already picked, not treated
    // as two independent, never-intersecting resolutions.
    let mut seen: BTreeMap<String, Resolved> = BTreeMap::new();
    // The requirement text that decided each name, so a later-arriving
    // requirement that disagrees can be named in the refusal alongside it.
    let mut deciding_requirement: BTreeMap<String, String> = BTreeMap::new();
    let mut queue: VecDeque<String> = roots.iter().cloned().collect();
    while let Some(raw) = queue.pop_front() {
        let name_key = normalize_name(&requirement_name(&raw)?);
        if let Some(prior) = seen.get(&name_key) {
            if !requirement_matches_version(&raw, &prior.pin.version)? {
                let prior_req = deciding_requirement
                    .get(&name_key)
                    .cloned()
                    .unwrap_or_default();
                bail!(
                    "conflicting requirements on {}: `{prior_req}` decided {}=={} but a \
                     later requirement needs `{raw}`, which that version does not satisfy",
                    prior.pin.name,
                    prior.pin.name,
                    prior.pin.version
                );
            }
            continue;
        }

        let (pin, meta) = load_metadata_with(&raw, index, prefs)?;
        let is_direct = direct_names.contains(&name_key);
        let filtered_requires: Vec<String> = meta
            .requires
            .iter()
            .filter(|r| !is_extra_marker(r))
            .cloned()
            .collect();
        let mut requires_keys = Vec::with_capacity(filtered_requires.len());
        for req in &filtered_requires {
            let (dep_pin, _) = load_metadata_with(req, index, prefs)?;
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
            name_key.clone(),
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
                project: true,
                groups: Vec::new(),
                extras: Vec::new(),
            },
        );
        deciding_requirement.insert(name_key, raw.clone());
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
    load_metadata_with(requirement, index, &Preferences::new())
}

/// [`load_metadata`] that keeps the version `prefs` names for this package
/// when it is among the candidates the requirement admits, and falls back
/// to the highest otherwise.
pub(crate) fn load_metadata_with(
    requirement: &str,
    index: &Path,
    prefs: &Preferences,
) -> Result<(Pin, IndexMetadata)> {
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
    let preferred = prefs
        .get(&normalize_name(&req.name))
        .and_then(|p| versions.iter().find(|v| *v == p))
        .cloned();
    let version = match preferred {
        Some(v) => v,
        None => versions.into_iter().next().with_context(|| {
            format!(
                "no candidate for `{}` matching {:?} in index {}",
                req.name,
                req.specifiers,
                index.display()
            )
        })?,
    };
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
    Ok((
        pin,
        IndexMetadata {
            sha256,
            path,
            requires,
        },
    ))
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

fn render_lockfile(state: &ManifestState, resolved: &[Resolved]) -> String {
    let input_hash = compute_input_hash(&state.lock_inputs());

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
        append_membership_fields(&mut out, r.project, &r.groups, &r.extras);
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

/// One `[[package]]` entry read from an on-disk `mamba.lock`, carried as raw
/// field bytes: the prune below must never re-resolve or blank a surviving
/// pin, only decide whether it survives at all.
struct PruneEntry {
    name: String,
    version: String,
    sha256: String,
    url: String,
    source_kind: String,
    path: String,
    provider: String,
    provides: Vec<String>,
    compatibility: String,
    maturity: String,
    /// Whether the entry carried a `direct` key at all. The manifest-shaped
    /// renderer (`add::render_lockfile_for_manifest`) omits the key
    /// entirely, and a prune over that shape must keep it absent rather
    /// than inventing one.
    direct_present: bool,
    dependencies: Vec<String>,
}

fn parse_prune_entries(lock_src: &str) -> Result<Vec<PruneEntry>> {
    let doc: toml::Value = lock_src.parse().context("parse mamba.lock")?;
    let arr = match doc.get("package") {
        Some(toml::Value::Array(a)) => a.clone(),
        Some(_) => bail!("mamba.lock `package` is not an array"),
        None => return Ok(Vec::new()),
    };
    let mut out = Vec::with_capacity(arr.len());
    for entry in arr {
        let tbl = entry
            .as_table()
            .context("mamba.lock package entry is not a table")?;
        let str_field = |key: &str| -> String {
            tbl.get(key)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        };
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
        let dependencies = tbl
            .get("dependencies")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let provides = tbl
            .get("provides")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        out.push(PruneEntry {
            name,
            version,
            sha256: str_field("sha256"),
            url: str_field("url"),
            source_kind: str_field("source_kind"),
            path: str_field("path"),
            provider: str_field("provider"),
            provides,
            compatibility: str_field("compatibility"),
            maturity: str_field("maturity"),
            direct_present: tbl.get("direct").and_then(|v| v.as_bool()).is_some(),
            dependencies,
        });
    }
    Ok(out)
}

/// The canonical `name==version` identity a dependency edge or a manifest
/// root is matched by: PEP 503 normalized name, exact version.
fn prune_edge_key(name: &str, version: &str) -> String {
    format!("{}=={}", normalize_name(name), version)
}

/// Rebuild the `SourceMeta` an entry's raw `source_kind`/`path`/provider
/// fields describe, so the surviving render can go back through
/// `append_lock_source_fields` -- the same writer `mamba add`/`mamba lock`
/// use -- rather than re-deriving the field shape by hand.
fn reconstruct_source(e: &PruneEntry) -> SourceMeta {
    match e.source_kind.as_str() {
        "direct_file" => SourceMeta::DirectFile {
            path: e.path.clone(),
        },
        "index" => SourceMeta::Index {
            path: e.path.clone(),
        },
        "mamba_provider" => SourceMeta::MambaProvider {
            provider: e.provider.clone(),
            provides: e.provides.clone(),
            compatibility: e.compatibility.clone(),
            maturity: e.maturity.clone(),
        },
        _ => SourceMeta::Default,
    }
}

/// Prune an existing `mamba.lock` body down to the transitive closure the
/// manifest's remaining dependencies still reach, carrying every surviving
/// pin's `sha256`, `url`, and source fields through byte-for-byte and
/// recomputing only `direct` (and only where the entry already carried that
/// key). Returns `None` when the lock cannot be trusted for this prune --
/// unparsable text, or a remaining manifest root with no matching entry --
/// so the caller (`remove::cmd_remove`) falls back to a full
/// manifest-shaped render.
pub(crate) fn prune_lock_for_remaining_roots(
    state: &ManifestState,
    lock_src: &str,
) -> Option<String> {
    let entries = parse_prune_entries(lock_src).ok()?;
    let mut by_key: BTreeMap<String, usize> = BTreeMap::new();
    for (i, e) in entries.iter().enumerate() {
        by_key.insert(prune_edge_key(&e.name, &e.version), i);
    }

    let mut by_name: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, e) in entries.iter().enumerate() {
        by_name.entry(normalize_name(&e.name)).or_default().push(i);
    }

    // Every remaining manifest root -- project, group, or extra -- must
    // already have a recorded pin: an exact `==` root by name and version,
    // any other requirement by name alone when the lock holds exactly one
    // entry for it. A root the lock never named means the lock cannot be
    // trusted for this prune at all.
    let roots = state.lock_roots();
    let mut root_indices: BTreeSet<usize> = BTreeSet::new();
    for root in &roots {
        let idx = match Pin::parse(&root.spec) {
            Ok(pin) => *by_key.get(&prune_edge_key(&pin.name, &pin.version))?,
            Err(_) => {
                let name = normalize_name(&requirement_name(&root.spec).ok()?);
                match by_name.get(&name)?.as_slice() {
                    [only] => *only,
                    _ => return None,
                }
            }
        };
        root_indices.insert(idx);
    }

    // Each entry's own recorded `dependencies` edges, as indices. An edge
    // that resolves to no known entry is dropped silently rather than
    // panicking -- the lock it was read from may already be stale in ways
    // this prune does not have to fix.
    let edges: Vec<Vec<usize>> = entries
        .iter()
        .map(|e| {
            e.dependencies
                .iter()
                .filter_map(|edge| {
                    let (n, v) = edge.split_once("==")?;
                    by_key.get(&prune_edge_key(n.trim(), v.trim())).copied()
                })
                .collect()
        })
        .collect();

    // BFS from the remaining roots.
    let mut reachable: BTreeSet<usize> = BTreeSet::new();
    let mut queue: VecDeque<usize> = root_indices.iter().copied().collect();
    while let Some(idx) = queue.pop_front() {
        if !reachable.insert(idx) {
            continue;
        }
        for &next in &edges[idx] {
            if !reachable.contains(&next) {
                queue.push_back(next);
            }
        }
    }

    let name_index: BTreeMap<String, usize> = by_name
        .iter()
        .filter_map(|(n, idxs)| idxs.first().map(|&i| (n.clone(), i)))
        .collect();
    let membership = compute_membership(&edges, &name_index, &roots);

    let mut kept: Vec<usize> = reachable.iter().copied().collect();
    kept.sort_by(|&a, &b| entries[a].name.cmp(&entries[b].name));

    let input_hash = compute_input_hash(&state.lock_inputs());

    let mut out = String::with_capacity(512);
    out.push_str("format_version = 1\n");
    out.push_str(&format!("input_hash = \"{input_hash}\"\n"));
    for idx in kept {
        let e = &entries[idx];
        out.push('\n');
        out.push_str("[[package]]\n");
        out.push_str(&format!("name = \"{}\"\n", e.name));
        out.push_str(&format!("version = \"{}\"\n", e.version));
        out.push_str(&format!("sha256 = \"{}\"\n", e.sha256));
        let source = reconstruct_source(e);
        append_lock_source_fields(&mut out, &e.name, &e.version, &e.url, &source);
        if e.direct_present {
            let is_direct = root_indices.contains(&idx);
            out.push_str(&format!("direct = {is_direct}\n"));
        }
        let m = &membership[idx];
        append_membership_fields(&mut out, m.project, &m.groups, &m.extras);
        out.push_str(&format!(
            "dependencies = {}\n",
            render_string_list(&e.dependencies)
        ));
    }
    Some(out)
}

/// The lock's `input_hash`: sha256 over the sorted, deduplicated inputs
/// (`ManifestState::lock_inputs`), one per line. Shared by every lock
/// writer so `add`, `lock`, `remove`, and `sync --locked` agree on it.
pub(crate) fn compute_input_hash(deps: &[String]) -> String {
    let mut sorted = deps.to_vec();
    sorted.sort();
    sorted.dedup();
    let mut hasher = Sha256::new();
    for d in &sorted {
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
                project: true,
                groups: Vec::new(),
                extras: Vec::new(),
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
                project: true,
                groups: Vec::new(),
                extras: Vec::new(),
            },
        ];
        let state = manifest_state(&["a==1.0"], &[], &[]);
        let a = render_lockfile(&state, &resolved);
        let b = render_lockfile(&state, &resolved);
        assert_eq!(a, b);
        assert!(!a.contains("project = "), "{a}");
        assert!(!a.contains("groups = "), "{a}");
    }

    fn manifest_state(deps: &[&str], dev: &[&str], extras: &[(&str, &[&str])]) -> ManifestState {
        let mut src = String::from(
            "[project]\nname = \"demo\"\nversion = \"0.1.0\"\nrequires-python = \">=3.12\"\n",
        );
        src.push_str(&format!(
            "dependencies = {}\n",
            render_string_list(&deps.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        ));
        if !extras.is_empty() {
            src.push_str("\n[project.optional-dependencies]\n");
            for (name, members) in extras {
                src.push_str(&format!(
                    "{name} = {}\n",
                    render_string_list(&members.iter().map(|s| s.to_string()).collect::<Vec<_>>())
                ));
            }
        }
        if !dev.is_empty() {
            src.push_str(&format!(
                "\n[dependency-groups]\ndev = {}\n",
                render_string_list(&dev.iter().map(|s| s.to_string()).collect::<Vec<_>>())
            ));
        }
        ManifestState::parse(&src).unwrap()
    }

    fn node(name: &str, version: &str, direct: bool, requires: &[&str]) -> Resolved {
        Resolved {
            pin: Pin {
                name: name.into(),
                version: version.into(),
            },
            direct,
            requires: requires.iter().map(|s| s.to_string()).collect(),
            sha256: None,
            url: None,
            source: SourceMeta::Default,
            project: true,
            groups: Vec::new(),
            extras: Vec::new(),
        }
    }

    #[test]
    fn membership_follows_each_selection_closure() {
        // core -> shared; pytest (dev) -> shared, pluggy; fastdep (extra
        // "fast") stands alone.
        let state = manifest_state(
            &["core==1.0"],
            &["pytest==8.0"],
            &[("fast", &["fastdep==2.0"])],
        );
        let mut resolved = vec![
            node("core", "1.0", true, &["shared==0.1"]),
            node("fastdep", "2.0", true, &[]),
            node("pluggy", "1.5", false, &[]),
            node("pytest", "8.0", true, &["shared==0.1", "pluggy==1.5"]),
            node("shared", "0.1", false, &[]),
        ];
        assign_membership(&mut resolved, &state);
        let by_name = |n: &str| resolved.iter().find(|r| r.pin.name == n).unwrap();
        assert!(by_name("core").project && by_name("core").groups.is_empty());
        assert!(by_name("shared").project);
        assert_eq!(by_name("shared").groups, vec!["dev"]);
        assert!(!by_name("pytest").project);
        assert_eq!(by_name("pytest").groups, vec!["dev"]);
        assert!(!by_name("pluggy").project);
        assert_eq!(by_name("pluggy").groups, vec!["dev"]);
        assert!(!by_name("fastdep").project);
        assert_eq!(by_name("fastdep").extras, vec!["fast"]);
        assert!(by_name("fastdep").groups.is_empty());

        let body = render_lockfile(&state, &resolved);
        assert!(
            body.contains("name = \"pluggy\"\nversion = \"1.5\"\nsha256 = \"\"\nurl = \"\"\nsource = \"pypi://pluggy/1.5\"\ndirect = false\nproject = false\ngroups = [\"dev\"]\ndependencies = []\n"),
            "{body}"
        );
        assert!(
            body.contains("name = \"shared\"\nversion = \"0.1\"\nsha256 = \"\"\nurl = \"\"\nsource = \"pypi://shared/0.1\"\ndirect = false\ngroups = [\"dev\"]\ndependencies = []\n"),
            "{body}"
        );
        assert!(
            body.contains(
                "direct = true\nproject = false\nextras = [\"fast\"]\ndependencies = []\n"
            ),
            "{body}"
        );
        assert!(body.contains("name = \"core\"\nversion = \"1.0\"\nsha256 = \"\"\nurl = \"\"\nsource = \"pypi://core/1.0\"\ndirect = true\ndependencies = [\"shared==0.1\"]\n"), "{body}");
        // The hash covers the group and extra members too.
        let plain = manifest_state(&["core==1.0"], &[], &[]);
        assert_ne!(
            compute_input_hash(&state.lock_inputs()),
            compute_input_hash(&plain.lock_inputs())
        );
        assert_eq!(
            compute_input_hash(&plain.lock_inputs()),
            compute_input_hash(&["core==1.0".to_string()])
        );
    }

    #[test]
    fn preferences_keep_a_locked_version_only_while_it_still_matches() {
        let index = tempfile::tempdir().unwrap();
        for v in ["1.0", "2.0", "3.0"] {
            let dir = index.path().join("lib").join(v);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("metadata.toml"), "requires = []\n").unwrap();
        }
        let mut prefs = Preferences::new();
        prefs.insert("lib".into(), "2.0".into());
        let (pin, _) = load_metadata_with("lib", index.path(), &prefs).unwrap();
        assert_eq!(pin.version, "2.0");
        let (pin, _) = load_metadata_with("lib>=2.5", index.path(), &prefs).unwrap();
        assert_eq!(pin.version, "3.0");
        let (pin, _) = load_metadata_with("lib", index.path(), &Preferences::new()).unwrap();
        assert_eq!(pin.version, "3.0");

        // `lock_preferences` reads the lock, honours --upgrade-package, and
        // --upgrade releases everything.
        let project = tempfile::tempdir().unwrap();
        fs::write(
            project.path().join(LOCKFILE_FILE),
            "format_version = 1\ninput_hash = \"x\"\n\n[[package]]\nname = \"Lib\"\nversion = \"2.0\"\nsha256 = \"\"\nurl = \"\"\ndependencies = []\n\n[[package]]\nname = \"other\"\nversion = \"1.0\"\nsha256 = \"\"\nurl = \"\"\ndependencies = []\n",
        )
        .unwrap();
        let all = lock_preferences(project.path(), false, &[]);
        assert_eq!(all.get("lib").map(String::as_str), Some("2.0"));
        assert_eq!(all.get("other").map(String::as_str), Some("1.0"));
        let one = lock_preferences(project.path(), false, &["LIB".to_string()]);
        assert!(!one.contains_key("lib"));
        assert!(one.contains_key("other"));
        assert!(lock_preferences(project.path(), true, &[]).is_empty());
        assert!(lock_preferences(index.path(), false, &[]).is_empty());
    }
}

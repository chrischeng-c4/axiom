// `mamba add` — uv-style dependency recording.
//
// Acceptance (tests/governance/gates/pkgmgr/add/manifest.toml, schema gate
// pkgmgr_add_fixture_2681.rs):
//
//   - Records the requested dep in mamba.toml deterministically.
//   - Records a pinned entry in mamba.lock deterministically.
//   - Replaying the same `add` against the same setup yields byte-identical
//     mamba.toml and mamba.lock.
//   - Missing package against a configured frozen index fails exit 1 with
//     "not found" in stderr, and does NOT mutate the manifest or lockfile.
//   - Offline: no network, no $HOME / global cache reads.
//
// The shape locked here is the MVP scope of the fixture: one package, one
// `name==version` specifier per invocation. Transitive resolution and
// complex version selection are out of scope (per #2681 issue body).

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

const MANIFEST_FILE: &str = "mamba.toml";
const LOCKFILE_FILE: &str = "mamba.lock";
const FROZEN_INDEX_ENV: &str = "MAMBA_FROZEN_INDEX";
const INDEX_URL_ENV: &str = "MAMBA_INDEX_URL";

pub fn cmd_add(sub: &ArgMatches) -> Result<()> {
    let spec_raw = sub
        .get_one::<String>("spec")
        .context("missing required argument <spec>")?;
    let project_dir = std::env::current_dir().context("read current directory")?;
    let manifest_path = project_dir.join(MANIFEST_FILE);
    if !manifest_path.exists() {
        bail!(
            "no {MANIFEST_FILE} in {} — run `mamba init` first",
            project_dir.display()
        );
    }

    let resolved = if let Some(provider) = sub.get_one::<String>("provider") {
        resolve_with_provider(spec_raw, provider)?
    } else if looks_like_wheel_path(spec_raw) {
        resolve_with_local_wheel(spec_raw, &project_dir)?
    } else {
        let spec = DepSpec::parse(spec_raw)?;
        let index_dir = resolve_index_dir(sub);
        let offline = sub.get_flag("offline");
        let index_url = resolve_index_url(sub);
        resolve_dep(&spec, index_dir.as_deref(), offline, index_url.as_deref())?
    };

    let manifest_src = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let mut state = ManifestState::parse(&manifest_src)?;
    state.upsert_dependency(&resolved.dep_string());
    match &resolved.source {
        SourceMeta::MambaProvider { provider, .. } => {
            state.upsert_source(
                &resolved.name,
                ManifestSource::MambaProvider {
                    provider: provider.clone(),
                },
            );
        }
        SourceMeta::Default | SourceMeta::DirectFile { .. } => {
            state.remove_source(&resolved.name);
        }
    }
    let new_manifest = state.render();

    let new_lockfile = render_lockfile_for_manifest_with_resolved(&state, &resolved)?;

    let lock_path = project_dir.join(LOCKFILE_FILE);
    atomic_write(&manifest_path, new_manifest.as_bytes())?;
    atomic_write(&lock_path, new_lockfile.as_bytes())?;

    Ok(())
}

#[derive(Debug, Clone)]
struct DepSpec {
    name: String,
    version: Option<String>,
}

impl DepSpec {
    fn parse(raw: &str) -> Result<Self> {
        let raw = raw.trim();
        if raw.is_empty() {
            bail!("empty dependency spec");
        }
        if let Some((name, version)) = raw.split_once("==") {
            let name = name.trim();
            let version = version.trim();
            if name.is_empty() || version.is_empty() {
                bail!("malformed spec `{raw}` (expected NAME==VERSION)");
            }
            return Ok(DepSpec {
                name: name.to_string(),
                version: Some(version.to_string()),
            });
        }
        Ok(DepSpec {
            name: raw.to_string(),
            version: None,
        })
    }
}

#[derive(Debug)]
struct ResolvedDep {
    name: String,
    version: String,
    sha256: Option<String>,
    /// Canonical artifact URL for the picked wheel/sdist — used by
    /// `mamba sync` to perform a sha-verified download. None for the
    /// local-frozen / --offline paths where we don't know the URL.
    url: Option<String>,
    source: SourceMeta,
}

impl ResolvedDep {
    fn dep_string(&self) -> String {
        format!("{}=={}", self.name, self.version)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum SourceMeta {
    Default,
    DirectFile {
        path: String,
    },
    MambaProvider {
        provider: String,
        provides: Vec<String>,
        compatibility: String,
        maturity: String,
    },
}

fn resolve_index_dir(sub: &ArgMatches) -> Option<PathBuf> {
    sub.get_one::<String>("index")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os(FROZEN_INDEX_ENV).map(PathBuf::from))
}

fn resolve_index_url(sub: &ArgMatches) -> Option<String> {
    sub.get_one::<String>("index-url")
        .cloned()
        .or_else(|| std::env::var(INDEX_URL_ENV).ok())
}

fn resolve_dep(
    spec: &DepSpec,
    index_dir: Option<&Path>,
    offline: bool,
    index_url: Option<&str>,
) -> Result<ResolvedDep> {
    // Local frozen index path — always wins when configured.
    if let Some(idx) = index_dir {
        return resolve_with_local_index(spec, idx);
    }
    // Offline + pinned: trust the user.
    if offline {
        return match spec.version.as_deref() {
            Some(v) => Ok(ResolvedDep {
                name: spec.name.clone(),
                version: v.to_string(),
                sha256: None,
                url: None,
                source: SourceMeta::Default,
            }),
            None => bail!(
                "version required for `mamba add {}` in --offline mode \
                 (use `mamba add {}==X.Y.Z` or drop --offline)",
                spec.name,
                spec.name
            ),
        };
    }
    if let Some(index_url) = index_url {
        return resolve_with_pypi(spec, index_url);
    }

    bail!(
        "no package source configured for `mamba add {}`; pass --index DIR, \
         set {FROZEN_INDEX_ENV}, pass --index-url URL, set {INDEX_URL_ENV}, \
         or use --offline with NAME==VERSION",
        spec.name
    )
}

fn resolve_with_provider(raw: &str, provider: &str) -> Result<ResolvedDep> {
    if provider != "mamba" {
        bail!("unsupported provider `{provider}` (supported: mamba)");
    }
    if looks_like_wheel_path(raw) {
        bail!("--provider mamba expects a mamba-owned package name, not a path");
    }
    let spec = DepSpec::parse(raw)?;
    let pkg =
        crate::pkgmanage::provider::resolve_mamba_package(&spec.name, spec.version.as_deref())?;
    Ok(ResolvedDep {
        name: pkg.distribution,
        version: pkg.version,
        sha256: None,
        url: None,
        source: SourceMeta::MambaProvider {
            provider: pkg.provider,
            provides: pkg.provides,
            compatibility: pkg.compatibility,
            maturity: pkg.maturity,
        },
    })
}

fn resolve_with_local_index(spec: &DepSpec, idx: &Path) -> Result<ResolvedDep> {
    let normalized = normalize_name(&spec.name);
    let pkg_dir = idx.join(&normalized);
    if !pkg_dir.exists() {
        bail!(
            "package `{}` not found in index {}",
            spec.name,
            idx.display()
        );
    }
    let version = match spec.version.as_deref() {
        Some(v) => {
            let ver_marker = pkg_dir.join(v);
            if !ver_marker.exists() {
                bail!(
                    "package `{}` version `{}` not found in index {}",
                    spec.name,
                    v,
                    idx.display()
                );
            }
            v.to_string()
        }
        None => pick_latest_version(&pkg_dir)?,
    };
    Ok(ResolvedDep {
        name: spec.name.clone(),
        version,
        sha256: None,
        url: None,
        source: SourceMeta::Default,
    })
}

fn resolve_with_pypi(spec: &DepSpec, index_url: &str) -> Result<ResolvedDep> {
    use crate::pkgmanage::pkgmgr::{IndexClient, IndexError};

    let cache_dir = pypi_cache_dir();
    let client = IndexClient {
        index_url: index_url.trim_end_matches('/').to_string(),
        cache_dir: cache_dir.to_string_lossy().into_owned(),
        max_concurrent: 8,
        timeout_secs: 30,
        retry_max: 3,
        auth_header: crate::pkgmanage::auth::authorization_for_url(index_url)?,
    };

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("build tokio runtime for PyPI fetch")?;

    let meta = rt
        .block_on(client.fetch_metadata(&spec.name))
        .map_err(|e| match e {
            IndexError::NotFound { name } => {
                anyhow::anyhow!("package `{name}` not found on index {index_url}")
            }
            other => anyhow::anyhow!("PyPI fetch failed: {other}"),
        })?;

    let version = match spec.version.as_deref() {
        Some(v) => {
            if !meta.releases.contains_key(v) {
                bail!(
                    "package `{}` version `{}` not on index {}",
                    spec.name,
                    v,
                    index_url
                );
            }
            v.to_string()
        }
        None => pick_pypi_latest(&meta.versions)
            .with_context(|| format!("no releases for `{}` on {}", spec.name, index_url))?,
    };

    let pick = pick_best_wheel(&meta, &version);
    let (sha256, url) = match pick {
        Some(WheelPick { sha256, url }) => (Some(sha256), Some(url)),
        None => (None, None),
    };

    Ok(ResolvedDep {
        name: spec.name.clone(),
        version,
        sha256,
        url,
        source: SourceMeta::Default,
    })
}

fn looks_like_wheel_path(raw: &str) -> bool {
    raw.ends_with(".whl") || raw.contains('/') || (cfg!(windows) && raw.contains('\\'))
}

fn resolve_with_local_wheel(raw: &str, project_dir: &Path) -> Result<ResolvedDep> {
    let raw_path = Path::new(raw);
    if raw_path.extension().and_then(|s| s.to_str()) != Some("whl") {
        bail!("direct local dependency `{raw}` is not a wheel file (expected .whl)");
    }
    let abs_path = if raw_path.is_absolute() {
        raw_path.to_path_buf()
    } else {
        project_dir.join(raw_path)
    };
    if !abs_path.exists() {
        bail!("local wheel path does not exist: {}", raw_path.display());
    }
    if !abs_path.is_file() {
        bail!("local wheel path is not a file: {}", raw_path.display());
    }
    let filename = abs_path
        .file_name()
        .and_then(|s| s.to_str())
        .context("local wheel path is not valid UTF-8")?;
    let wheel = crate::pkgmanage::pkgmgr::wheel_filename::parse_wheel_filename(filename)
        .map_err(|e| anyhow::anyhow!("parse local wheel filename `{filename}`: {e}"))?;
    let bytes = fs::read(&abs_path).with_context(|| format!("read {}", abs_path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let sha256 = format!("{:x}", hasher.finalize());
    Ok(ResolvedDep {
        name: wheel.distribution,
        version: wheel.version,
        sha256: Some(sha256),
        url: None,
        source: SourceMeta::DirectFile {
            path: lockfile_path(raw_path, &abs_path, project_dir)?,
        },
    })
}

fn lockfile_path(raw_path: &Path, abs_path: &Path, project_dir: &Path) -> Result<String> {
    if raw_path.is_absolute() {
        let canonical_project = fs::canonicalize(project_dir)
            .with_context(|| format!("canonicalize {}", project_dir.display()))?;
        let canonical_abs = fs::canonicalize(abs_path)
            .with_context(|| format!("canonicalize {}", abs_path.display()))?;
        if let Ok(rel) = canonical_abs.strip_prefix(&canonical_project) {
            return Ok(path_to_forward_slashes(rel));
        }
        return Ok(path_to_forward_slashes(&canonical_abs));
    }

    Ok(path_to_forward_slashes(&normalize_relative_path(raw_path)))
}

fn normalize_relative_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn path_to_forward_slashes(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Best-wheel pick result: the sha256 to record in the lockfile and the
/// canonical download URL of that artifact. Tick 15 carries both so
/// `mamba sync` can fetch + sha-verify without re-resolving.
#[derive(Debug, Clone)]
pub(crate) struct WheelPick {
    pub(crate) sha256: String,
    pub(crate) url: String,
}

/// Pick the best-scoring wheel for the current host, falling back to any
/// non-yanked file in the release. Driven by PEP 425 tag selection
/// (`pkgmgr::tags`) — uv-parity behavior. Sdists are accepted only as a
/// last resort, matching uv's preference for binaries.
fn pick_best_wheel(
    meta: &crate::pkgmanage::pkgmgr::PackageMetadata,
    version: &str,
) -> Option<WheelPick> {
    use crate::pkgmanage::pkgmgr::tags::{TagSelector, parse_wheel_filename};
    let files = meta.releases.get(version)?;
    let selector = TagSelector::current_host();
    let mut best: Option<(u32, &str, &str)> = None;
    let mut fallback_wheel: Option<(&str, &str)> = None;
    let mut fallback_sdist: Option<(&str, &str)> = None;
    for f in files {
        if f.yanked || f.hash.algorithm != "sha256" || f.hash.digest.is_empty() {
            continue;
        }
        let digest = f.hash.digest.as_str();
        let url = f.url.as_str();
        if let Some(wt) = parse_wheel_filename(&f.filename) {
            if let Some(score) = selector.score(&wt) {
                if best.map(|(s, _, _)| score > s).unwrap_or(true) {
                    best = Some((score, digest, url));
                }
            } else if fallback_wheel.is_none() {
                fallback_wheel = Some((digest, url));
            }
        } else if fallback_sdist.is_none() {
            fallback_sdist = Some((digest, url));
        }
    }
    best.map(|(_, d, u)| WheelPick {
        sha256: d.to_string(),
        url: u.to_string(),
    })
    .or_else(|| {
        fallback_wheel.map(|(d, u)| WheelPick {
            sha256: d.to_string(),
            url: u.to_string(),
        })
    })
    .or_else(|| {
        fallback_sdist.map(|(d, u)| WheelPick {
            sha256: d.to_string(),
            url: u.to_string(),
        })
    })
}

/// Pick the lexicographically-newest non-pre/dev version from a PyPI
/// version list. Filters out obvious pre-release strings (a/b/rc/dev) so
/// `mamba add requests` lands on a stable release. The full PEP 440 sort
/// arrives with the PubGrub resolver in Tick 13.
fn pick_pypi_latest(versions: &[String]) -> Option<String> {
    let mut stable: Vec<&String> = versions.iter().filter(|v| !is_prerelease(v)).collect();
    if stable.is_empty() {
        stable = versions.iter().collect();
    }
    stable.sort_by(|a, b| pep440_lite_cmp(a, b));
    stable.last().map(|s| (*s).clone())
}

fn is_prerelease(v: &str) -> bool {
    let lower = v.to_lowercase();
    ["a", "b", "rc", "dev", "alpha", "beta", "pre", "post"]
        .iter()
        .any(|tag| lower.contains(tag) && lower.chars().any(|c| !c.is_ascii_digit() && c != '.'))
}

/// Coarse PEP 440 ordering: split on `.` and compare numeric segments as
/// integers, falling back to lexical for non-numeric segments. Good enough
/// to pick "2.31.0" over "2.4.0" without pulling in the full PEP 440 parser
/// (which lives in `pkgmgr::pep440`).
fn pep440_lite_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let parts_a: Vec<&str> = a.split('.').collect();
    let parts_b: Vec<&str> = b.split('.').collect();
    for i in 0..parts_a.len().max(parts_b.len()) {
        let pa = parts_a.get(i).copied().unwrap_or("0");
        let pb = parts_b.get(i).copied().unwrap_or("0");
        match (pa.parse::<u64>(), pb.parse::<u64>()) {
            (Ok(x), Ok(y)) => match x.cmp(&y) {
                std::cmp::Ordering::Equal => continue,
                ord => return ord,
            },
            _ => match pa.cmp(pb) {
                std::cmp::Ordering::Equal => continue,
                ord => return ord,
            },
        }
    }
    std::cmp::Ordering::Equal
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

/// PEP 503 normalize: lowercase + collapse `-`, `_`, `.` runs to a single `-`.
fn normalize_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_was_sep = false;
    for c in name.chars() {
        let is_sep = c == '-' || c == '_' || c == '.';
        if is_sep {
            if !prev_was_sep && !out.is_empty() {
                out.push('-');
            }
            prev_was_sep = true;
        } else {
            out.push(c.to_ascii_lowercase());
            prev_was_sep = false;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    out
}

fn pick_latest_version(pkg_dir: &Path) -> Result<String> {
    let mut versions: Vec<String> = fs::read_dir(pkg_dir)
        .with_context(|| format!("read index dir {}", pkg_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type()
                .map(|t| t.is_dir() || t.is_file())
                .unwrap_or(false)
        })
        .filter_map(|e| e.file_name().to_str().map(str::to_string))
        .collect();
    if versions.is_empty() {
        bail!("no versions for package in {}", pkg_dir.display());
    }
    versions.sort();
    Ok(versions.pop().unwrap())
}

pub(crate) struct ManifestState {
    pub(crate) project_name: String,
    pub(crate) project_version: String,
    pub(crate) python_requires: String,
    pub(crate) dependencies: Vec<String>,
    pub(crate) dev_dependencies: Vec<String>,
    pub(crate) source_overrides: BTreeMap<String, ManifestSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ManifestSource {
    MambaProvider { provider: String },
}

impl ManifestState {
    pub(crate) fn parse(src: &str) -> Result<Self> {
        let doc: toml::Value = src.parse().context("parse mamba.toml")?;
        let project = doc
            .get("project")
            .and_then(|v| v.as_table())
            .context("mamba.toml missing [project] table")?;
        let project_name = project
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("mamba-project")
            .to_string();
        let project_version = project
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.1.0")
            .to_string();
        let python_requires = project
            .get("python-requires")
            .and_then(|v| v.as_str())
            .unwrap_or(">=3.12")
            .to_string();
        let dependencies = extract_string_list(project, "dependencies");
        let dev_dependencies = extract_string_list(project, "dev-dependencies");
        let source_overrides = extract_source_overrides(&doc)?;

        Ok(ManifestState {
            project_name,
            project_version,
            python_requires,
            dependencies,
            dev_dependencies,
            source_overrides,
        })
    }

    pub(crate) fn upsert_dependency(&mut self, spec: &str) {
        let new_name = dep_name(spec);
        self.dependencies.retain(|d| dep_name(d) != new_name);
        self.dependencies.push(spec.to_string());
        self.dependencies.sort();
        self.dependencies.dedup();
    }

    pub(crate) fn remove_dependency(&mut self, name: &str) {
        let name = name.trim();
        self.dependencies.retain(|d| dep_name(d) != name);
        self.dev_dependencies.retain(|d| dep_name(d) != name);
        self.source_overrides.remove(name);
    }

    pub(crate) fn upsert_source(&mut self, name: &str, source: ManifestSource) {
        self.source_overrides.insert(name.to_string(), source);
    }

    pub(crate) fn remove_source(&mut self, name: &str) {
        self.source_overrides.remove(name);
    }

    pub(crate) fn render(&self) -> String {
        let mut out = String::with_capacity(256);
        out.push_str("[project]\n");
        out.push_str(&format!("name = \"{}\"\n", self.project_name));
        out.push_str(&format!("version = \"{}\"\n", self.project_version));
        out.push_str(&format!("python-requires = \"{}\"\n", self.python_requires));
        out.push_str(&format!(
            "dependencies = {}\n",
            render_string_list(&self.dependencies)
        ));
        out.push_str(&format!(
            "dev-dependencies = {}\n",
            render_string_list(&self.dev_dependencies)
        ));
        if !self.source_overrides.is_empty() {
            out.push('\n');
            out.push_str("[tool.mamba.sources]\n");
            for (name, source) in &self.source_overrides {
                match source {
                    ManifestSource::MambaProvider { provider } => {
                        out.push_str(&format!(
                            "{} = {{ provider = \"{}\" }}\n",
                            render_toml_key(name),
                            escape_toml_string(provider)
                        ));
                    }
                }
            }
        }
        out
    }
}

fn extract_source_overrides(doc: &toml::Value) -> Result<BTreeMap<String, ManifestSource>> {
    let mut out = BTreeMap::new();
    let Some(sources) = doc
        .get("tool")
        .and_then(|t| t.get("mamba"))
        .and_then(|m| m.get("sources"))
    else {
        return Ok(out);
    };
    let sources = sources
        .as_table()
        .context("[tool.mamba.sources] must be a table")?;
    for (name, value) in sources {
        let entry = value
            .as_table()
            .with_context(|| format!("[tool.mamba.sources] `{name}` must be an inline table"))?;
        let provider = entry
            .get("provider")
            .and_then(|v| v.as_str())
            .with_context(|| format!("[tool.mamba.sources] `{name}` missing provider"))?;
        if provider != "mamba" {
            bail!("[tool.mamba.sources] `{name}` uses unsupported provider `{provider}`");
        }
        out.insert(
            name.clone(),
            ManifestSource::MambaProvider {
                provider: provider.to_string(),
            },
        );
    }
    Ok(out)
}

fn extract_string_list(tbl: &toml::value::Table, key: &str) -> Vec<String> {
    tbl.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn render_string_list(items: &[String]) -> String {
    if items.is_empty() {
        return "[]".to_string();
    }
    let mut out = String::from("[\n");
    for item in items {
        out.push_str("    \"");
        out.push_str(item);
        out.push_str("\",\n");
    }
    out.push(']');
    out
}

fn render_inline_string_list(items: &[String]) -> String {
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
        out.push_str(&escape_toml_string(item));
        out.push('"');
    }
    out.push(']');
    out
}

fn render_toml_key(key: &str) -> String {
    format!("\"{}\"", escape_toml_string(key))
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub(crate) fn dep_name(spec: &str) -> &str {
    spec.split_once("==")
        .map(|(n, _)| n.trim())
        .unwrap_or(spec.trim())
}

#[cfg(test)]
fn render_lockfile_with_hashes(deps: &[String], just_added: &ResolvedDep) -> String {
    let mut hashes = std::collections::BTreeMap::new();
    let mut urls = std::collections::BTreeMap::new();
    let mut sources = std::collections::BTreeMap::new();
    if let Some(h) = just_added.sha256.as_deref() {
        hashes.insert(just_added.name.clone(), h.to_string());
    }
    if let Some(u) = just_added.url.as_deref() {
        urls.insert(just_added.name.clone(), u.to_string());
    }
    if let SourceMeta::DirectFile { path } = &just_added.source {
        sources.insert(
            just_added.name.clone(),
            SourceMeta::DirectFile { path: path.clone() },
        );
    }
    render_lockfile_with_known_hashes(deps, &hashes, &urls, &sources)
}

fn render_lockfile_for_manifest_with_resolved(
    state: &ManifestState,
    just_added: &ResolvedDep,
) -> Result<String> {
    let mut hashes = BTreeMap::new();
    let mut urls = BTreeMap::new();
    let mut sources = collect_manifest_sources(state)?;
    if let Some(h) = just_added.sha256.as_deref() {
        hashes.insert(just_added.name.clone(), h.to_string());
    }
    if let Some(u) = just_added.url.as_deref() {
        urls.insert(just_added.name.clone(), u.to_string());
    }
    match &just_added.source {
        SourceMeta::Default => {}
        SourceMeta::DirectFile { path } => {
            sources.insert(
                just_added.name.clone(),
                SourceMeta::DirectFile { path: path.clone() },
            );
        }
        SourceMeta::MambaProvider {
            provider,
            provides,
            compatibility,
            maturity,
        } => {
            sources.insert(
                just_added.name.clone(),
                SourceMeta::MambaProvider {
                    provider: provider.clone(),
                    provides: provides.clone(),
                    compatibility: compatibility.clone(),
                    maturity: maturity.clone(),
                },
            );
        }
    }
    Ok(render_lockfile_with_known_hashes(
        &state.dependencies,
        &hashes,
        &urls,
        &sources,
    ))
}

pub(crate) fn render_lockfile_for_manifest(state: &ManifestState) -> Result<String> {
    let sources = collect_manifest_sources(state)?;
    Ok(render_lockfile_with_known_hashes(
        &state.dependencies,
        &BTreeMap::new(),
        &BTreeMap::new(),
        &sources,
    ))
}

fn collect_manifest_sources(state: &ManifestState) -> Result<BTreeMap<String, SourceMeta>> {
    let mut sources = BTreeMap::new();
    for dep in &state.dependencies {
        let Some((name, version)) = dep.split_once("==") else {
            continue;
        };
        let name = name.trim();
        let version = version.trim();
        if let Some(source) = state.source_overrides.get(name) {
            sources.insert(
                name.to_string(),
                source_meta_from_manifest(name, version, source)?,
            );
        }
    }
    Ok(sources)
}

pub(crate) fn source_meta_from_manifest(
    name: &str,
    version: &str,
    source: &ManifestSource,
) -> Result<SourceMeta> {
    match source {
        ManifestSource::MambaProvider { provider } => {
            if provider != "mamba" {
                bail!("unsupported provider `{provider}` for `{name}`");
            }
            let pkg = crate::pkgmanage::provider::resolve_mamba_package(name, Some(version))?;
            Ok(SourceMeta::MambaProvider {
                provider: pkg.provider,
                provides: pkg.provides,
                compatibility: pkg.compatibility,
                maturity: pkg.maturity,
            })
        }
    }
}

pub(crate) fn render_lockfile_with_known_hashes(
    deps: &[String],
    hashes: &std::collections::BTreeMap<String, String>,
    urls: &std::collections::BTreeMap<String, String>,
    sources: &std::collections::BTreeMap<String, SourceMeta>,
) -> String {
    let input_hash = compute_input_hash(deps);
    let mut entries: Vec<(String, String)> = deps
        .iter()
        .filter_map(|d| {
            let (n, v) = d.split_once("==")?;
            Some((n.trim().to_string(), v.trim().to_string()))
        })
        .collect();
    entries.sort();
    entries.dedup_by(|a, b| a.0 == b.0);

    let mut out = String::with_capacity(256);
    out.push_str("format_version = 1\n");
    out.push_str(&format!("input_hash = \"{input_hash}\"\n"));
    for (name, version) in &entries {
        let sha = hashes.get(name).map(String::as_str).unwrap_or("");
        let url = urls.get(name).map(String::as_str).unwrap_or("");
        out.push('\n');
        out.push_str("[[package]]\n");
        out.push_str(&format!("name = \"{name}\"\n"));
        out.push_str(&format!("version = \"{version}\"\n"));
        out.push_str(&format!("sha256 = \"{sha}\"\n"));
        match sources.get(name) {
            Some(SourceMeta::DirectFile { path }) => {
                append_lock_source_fields(
                    &mut out,
                    name,
                    version,
                    "",
                    &SourceMeta::DirectFile { path: path.clone() },
                );
            }
            Some(SourceMeta::MambaProvider { .. }) => {
                append_lock_source_fields(&mut out, name, version, "", sources.get(name).unwrap());
            }
            Some(SourceMeta::Default) | None => {
                append_lock_source_fields(&mut out, name, version, url, &SourceMeta::Default);
            }
        }
        out.push_str("dependencies = []\n");
    }
    out
}

pub(crate) fn append_lock_source_fields(
    out: &mut String,
    name: &str,
    version: &str,
    url: &str,
    source: &SourceMeta,
) {
    match source {
        SourceMeta::Default => {
            out.push_str(&format!("url = \"{}\"\n", escape_toml_string(url)));
            out.push_str(&format!(
                "source = \"pypi://{}/{}\"\n",
                escape_toml_string(name),
                escape_toml_string(version)
            ));
        }
        SourceMeta::DirectFile { path } => {
            out.push_str("url = \"\"\n");
            out.push_str("source_kind = \"direct_file\"\n");
            out.push_str(&format!("path = \"{}\"\n", escape_toml_string(path)));
            out.push_str(&format!(
                "source = \"direct-file://{}\"\n",
                escape_toml_string(path)
            ));
        }
        SourceMeta::MambaProvider {
            provider,
            provides,
            compatibility,
            maturity,
        } => {
            out.push_str("url = \"\"\n");
            out.push_str("source_kind = \"mamba_provider\"\n");
            out.push_str(&format!(
                "provider = \"{}\"\n",
                escape_toml_string(provider)
            ));
            out.push_str(&format!(
                "provides = {}\n",
                render_inline_string_list(provides)
            ));
            out.push_str(&format!(
                "compatibility = \"{}\"\n",
                escape_toml_string(compatibility)
            ));
            out.push_str(&format!(
                "maturity = \"{}\"\n",
                escape_toml_string(maturity)
            ));
            out.push_str(&format!(
                "source = \"mamba-provider://{}/{}/{}\"\n",
                escape_toml_string(provider),
                escape_toml_string(name),
                escape_toml_string(version)
            ));
        }
    }
}

fn compute_input_hash(deps: &[String]) -> String {
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

pub(crate) fn atomic_write(dest: &Path, body: &[u8]) -> Result<()> {
    let tmp = dest.with_extension({
        let mut ext = dest
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if ext.is_empty() {
            "tmp".to_string()
        } else {
            ext.push_str(".tmp");
            ext
        }
    });
    fs::write(&tmp, body).with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, dest).with_context(|| format!("rename to {}", dest.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_name_only() {
        let s = DepSpec::parse("foo").unwrap();
        assert_eq!(s.name, "foo");
        assert_eq!(s.version, None);
    }

    #[test]
    fn parse_name_eq_version() {
        let s = DepSpec::parse("foo==1.2.3").unwrap();
        assert_eq!(s.name, "foo");
        assert_eq!(s.version.as_deref(), Some("1.2.3"));
    }

    #[test]
    fn parse_malformed_rejected() {
        assert!(DepSpec::parse("==1.0").is_err());
        assert!(DepSpec::parse("foo==").is_err());
        assert!(DepSpec::parse("").is_err());
    }

    #[test]
    fn pep503_normalize() {
        assert_eq!(normalize_name("Foo.Bar_baz"), "foo-bar-baz");
        assert_eq!(normalize_name("FOO__bar"), "foo-bar");
        assert_eq!(normalize_name("plain"), "plain");
    }

    #[test]
    fn upsert_dedupes_by_name() {
        let mut s = ManifestState {
            project_name: "p".into(),
            project_version: "0.1.0".into(),
            python_requires: ">=3.12".into(),
            dependencies: vec!["a==1.0".into(), "b==2.0".into()],
            dev_dependencies: vec![],
            source_overrides: BTreeMap::new(),
        };
        s.upsert_dependency("a==1.1");
        assert_eq!(s.dependencies, vec!["a==1.1", "b==2.0"]);
    }

    #[test]
    fn manifest_renders_mamba_provider_source() {
        let mut s = ManifestState {
            project_name: "p".into(),
            project_version: "0.1.0".into(),
            python_requires: ">=3.12".into(),
            dependencies: vec!["mamba-httpx-compat==0.1.0".into()],
            dev_dependencies: vec![],
            source_overrides: BTreeMap::new(),
        };
        s.upsert_source(
            "mamba-httpx-compat",
            ManifestSource::MambaProvider {
                provider: "mamba".into(),
            },
        );
        let rendered = s.render();
        assert!(rendered.contains("[tool.mamba.sources]"));
        assert!(rendered.contains("\"mamba-httpx-compat\" = { provider = \"mamba\" }"));
        let parsed = ManifestState::parse(&rendered).unwrap();
        assert_eq!(
            parsed.source_overrides.get("mamba-httpx-compat"),
            Some(&ManifestSource::MambaProvider {
                provider: "mamba".into()
            })
        );
    }

    #[test]
    fn lockfile_is_deterministic() {
        let r = ResolvedDep {
            name: "foo".into(),
            version: "1.0".into(),
            sha256: None,
            url: None,
            source: SourceMeta::Default,
        };
        let deps = vec!["foo==1.0".to_string()];
        let a = render_lockfile_with_hashes(&deps, &r);
        let b = render_lockfile_with_hashes(&deps, &r);
        assert_eq!(a, b);
    }

    #[test]
    fn lockfile_includes_just_added_sha() {
        let r = ResolvedDep {
            name: "foo".into(),
            version: "1.0".into(),
            sha256: Some("deadbeef".repeat(8)),
            url: Some("https://example.invalid/foo-1.0.whl".into()),
            source: SourceMeta::Default,
        };
        let deps = vec!["foo==1.0".to_string()];
        let body = render_lockfile_with_hashes(&deps, &r);
        assert!(
            body.contains(&format!("sha256 = \"{}\"", "deadbeef".repeat(8))),
            "lockfile must carry sha256 for just-added pkg: {body}"
        );
    }

    #[test]
    fn pep440_lite_orders_numerically() {
        let mut v = vec!["2.4.0", "2.31.0", "2.10.0", "1.0.0"];
        v.sort_by(|a, b| pep440_lite_cmp(a, b));
        assert_eq!(v, vec!["1.0.0", "2.4.0", "2.10.0", "2.31.0"]);
    }

    #[test]
    fn pick_pypi_latest_excludes_prereleases() {
        let versions = vec![
            "1.0.0".to_string(),
            "2.0.0a1".to_string(),
            "1.5.0".to_string(),
            "2.0.0rc1".to_string(),
        ];
        assert_eq!(pick_pypi_latest(&versions), Some("1.5.0".to_string()));
    }

    #[test]
    fn pick_pypi_latest_falls_back_when_only_prerelease() {
        let versions = vec!["2.0.0a1".to_string(), "2.0.0rc1".to_string()];
        let picked = pick_pypi_latest(&versions);
        assert!(picked.is_some(), "must still return something");
    }
}

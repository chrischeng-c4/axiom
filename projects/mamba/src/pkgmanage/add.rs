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
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_FILE: &str = "mamba.toml";
const LOCKFILE_FILE: &str = "mamba.lock";
const FROZEN_INDEX_ENV: &str = "MAMBA_FROZEN_INDEX";
const INDEX_URL_ENV: &str = "MAMBA_INDEX_URL";
const DEFAULT_INDEX_URL: &str = "https://pypi.org";

pub fn cmd_add(sub: &ArgMatches) -> Result<()> {
    let spec_raw = sub
        .get_one::<String>("spec")
        .context("missing required argument <spec>")?;
    let spec = DepSpec::parse(spec_raw)?;
    let project_dir = std::env::current_dir().context("read current directory")?;
    let manifest_path = project_dir.join(MANIFEST_FILE);
    if !manifest_path.exists() {
        bail!(
            "no {MANIFEST_FILE} in {} — run `mamba init` first",
            project_dir.display()
        );
    }

    let index_dir = resolve_index_dir(sub);
    let offline = sub.get_flag("offline");
    let index_url = resolve_index_url(sub);
    let resolved = resolve_dep(&spec, index_dir.as_deref(), offline, &index_url)?;

    let manifest_src = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let mut state = ManifestState::parse(&manifest_src)?;
    state.upsert_dependency(&resolved.dep_string());
    let new_manifest = state.render();

    let new_lockfile = render_lockfile_with_hashes(&state.dependencies, &resolved);

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
}

impl ResolvedDep {
    fn dep_string(&self) -> String {
        format!("{}=={}", self.name, self.version)
    }
}

fn resolve_index_dir(sub: &ArgMatches) -> Option<PathBuf> {
    sub.get_one::<String>("index")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os(FROZEN_INDEX_ENV).map(PathBuf::from))
}

fn resolve_index_url(sub: &ArgMatches) -> String {
    sub.get_one::<String>("index-url")
        .cloned()
        .or_else(|| std::env::var(INDEX_URL_ENV).ok())
        .unwrap_or_else(|| DEFAULT_INDEX_URL.to_string())
}

fn resolve_dep(
    spec: &DepSpec,
    index_dir: Option<&Path>,
    offline: bool,
    index_url: &str,
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
            }),
            None => bail!(
                "version required for `mamba add {}` in --offline mode \
                 (use `mamba add {}==X.Y.Z` or drop --offline)",
                spec.name,
                spec.name
            ),
        };
    }
    // Default: PyPI.
    resolve_with_pypi(spec, index_url)
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
    })
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
    use crate::pkgmanage::pkgmgr::tags::{parse_wheel_filename, TagSelector};
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
    let mut stable: Vec<&String> = versions
        .iter()
        .filter(|v| !is_prerelease(v))
        .collect();
    if stable.is_empty() {
        stable = versions.iter().collect();
    }
    stable.sort_by(|a, b| pep440_lite_cmp(a, b));
    stable.last().map(|s| (*s).clone())
}

fn is_prerelease(v: &str) -> bool {
    let lower = v.to_lowercase();
    [
        "a", "b", "rc", "dev", "alpha", "beta", "pre", "post",
    ]
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
        .filter(|e| e.file_type().map(|t| t.is_dir() || t.is_file()).unwrap_or(false))
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

        Ok(ManifestState {
            project_name,
            project_version,
            python_requires,
            dependencies,
            dev_dependencies,
        })
    }

    pub(crate) fn upsert_dependency(&mut self, spec: &str) {
        let new_name = dep_name(spec);
        self.dependencies
            .retain(|d| dep_name(d) != new_name);
        self.dependencies.push(spec.to_string());
        self.dependencies.sort();
        self.dependencies.dedup();
    }

    pub(crate) fn remove_dependency(&mut self, name: &str) {
        self.dependencies.retain(|d| dep_name(d) != name);
        self.dev_dependencies.retain(|d| dep_name(d) != name);
    }

    pub(crate) fn render(&self) -> String {
        let mut out = String::with_capacity(256);
        out.push_str("[project]\n");
        out.push_str(&format!("name = \"{}\"\n", self.project_name));
        out.push_str(&format!("version = \"{}\"\n", self.project_version));
        out.push_str(&format!(
            "python-requires = \"{}\"\n",
            self.python_requires
        ));
        out.push_str(&format!(
            "dependencies = {}\n",
            render_string_list(&self.dependencies)
        ));
        out.push_str(&format!(
            "dev-dependencies = {}\n",
            render_string_list(&self.dev_dependencies)
        ));
        out
    }
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

pub(crate) fn dep_name(spec: &str) -> &str {
    spec.split_once("==")
        .map(|(n, _)| n.trim())
        .unwrap_or(spec.trim())
}

fn render_lockfile_with_hashes(deps: &[String], just_added: &ResolvedDep) -> String {
    let mut hashes = std::collections::BTreeMap::new();
    let mut urls = std::collections::BTreeMap::new();
    if let Some(h) = just_added.sha256.as_deref() {
        hashes.insert(just_added.name.clone(), h.to_string());
    }
    if let Some(u) = just_added.url.as_deref() {
        urls.insert(just_added.name.clone(), u.to_string());
    }
    render_lockfile_with_known_hashes(deps, &hashes, &urls)
}

/// Deterministic lockfile rendering for an arbitrary dep list. Sorted +
/// deduped by package name; byte-identical for the same input.
pub(crate) fn render_lockfile_from_deps(deps: &[String]) -> String {
    render_lockfile_with_known_hashes(
        deps,
        &std::collections::BTreeMap::new(),
        &std::collections::BTreeMap::new(),
    )
}

pub(crate) fn render_lockfile_with_known_hashes(
    deps: &[String],
    hashes: &std::collections::BTreeMap<String, String>,
    urls: &std::collections::BTreeMap<String, String>,
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
        out.push_str(&format!("url = \"{url}\"\n"));
        out.push_str(&format!("source = \"pypi://{name}/{version}\"\n"));
        out.push_str("dependencies = []\n");
    }
    out
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
        };
        s.upsert_dependency("a==1.1");
        assert_eq!(s.dependencies, vec!["a==1.1", "b==2.0"]);
    }

    #[test]
    fn lockfile_is_deterministic() {
        let r = ResolvedDep {
            name: "foo".into(),
            version: "1.0".into(),
            sha256: None,
            url: None,
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

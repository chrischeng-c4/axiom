// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
// CODEGEN-BEGIN
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;

pub mod audit;
pub mod gc;
pub mod lockfile;
pub mod npmrc;
pub mod nx;
pub mod patch;
pub mod platform;
pub mod publish;
pub mod registry;
pub mod resolver;
pub mod store;
pub mod workspace;

use lockfile::{Lockfile, LockfileEntry, Resolution};
use npmrc::NpmrcConfig;
use registry::RegistryClient;
use resolver::{DependencyResolver, ResolvedPackage};
use store::StoreManager;

const MAX_CONCURRENT_DOWNLOADS: usize = 50;

/// Package manager for installing and managing npm dependencies.
///
/// Uses a global store (`~/.jet-store/`) to cache extracted packages
/// and hardlinks them into per-project `node_modules/`.
/// Parallel downloads are bounded by a semaphore.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub struct PackageManager {
    root_dir: PathBuf,
    store: Arc<StoreManager>,
    registry: Arc<RegistryClient>,
    resolver: DependencyResolver,
    semaphore: Arc<Semaphore>,
}

/// package.json structure (extended for pnpm parity).
///
/// Unknown top-level fields (e.g. `private`, `type`, `scripts`,
/// `engines`, `repository`, `keywords`, `author`, `license`,
/// `exports`, `bin`, …) are captured in [`PackageJson::extra`] via
/// `#[serde(flatten)]` so that round-tripping the document through
/// `serde_json` does not silently drop them. This view is *read-only*
/// inside the package-manager — mutating operations such as
/// `jet add` / `jet remove` operate on the raw `serde_json::Value`
/// via [`PackageManager::read_package_json_raw`] to preserve key
/// order and any non-managed fields verbatim. See issue #1941.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageJson {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(rename = "devDependencies", default)]
    pub dev_dependencies: HashMap<String, String>,
    #[serde(rename = "optionalDependencies", default)]
    pub optional_dependencies: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub overrides: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workspaces: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub os: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cpu: Vec<String>,
    /// Any other top-level fields preserved verbatim so that re-serialisation
    /// does not drop unknown content (issue #1941).
    #[serde(default, flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Parse a `jet add` package spec into `(name, optional version range)`.
///
/// Accepts:
/// - bare names: `react` → `("react", None)`
/// - scoped names: `@mui/material` → `("@mui/material", None)`
/// - `name@range`: `react@^18` → `("react", Some("^18"))`
/// - scoped with range: `@mui/material@^5` → `("@mui/material", Some("^5"))`
///
/// The leading `@` of scoped names is not treated as a version separator —
/// only an `@` past the first character separates name from range.
fn parse_package_spec(spec: &str) -> Result<(String, Option<String>)> {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        anyhow::bail!("empty package spec");
    }

    // Look for the version separator: an '@' that is not the leading
    // scope marker. For scoped names, the version `@` must come after
    // the closing `/` of the scope; for bare names, any non-leading `@`.
    let search_start = if trimmed.starts_with('@') { 1 } else { 0 };
    let sep = trimmed[search_start..].find('@').map(|i| i + search_start);

    match sep {
        Some(i) => {
            let name = trimmed[..i].to_string();
            let range = trimmed[i + 1..].to_string();
            if name.is_empty() {
                anyhow::bail!("package spec '{}' has empty name", spec);
            }
            if range.is_empty() {
                anyhow::bail!(
                    "package spec '{}' has empty version range \
                     after '@'",
                    spec
                );
            }
            Ok((name, Some(range)))
        }
        None => Ok((trimmed.to_string(), None)),
    }
}

/// Format the warning emitted when `HOME` is unset / empty and jet falls
/// back to using the current working directory as a pseudo-home for the
/// global store and metadata cache.
///
/// The message names the env vars the user can set to silence the warning
/// (`HOME` for the store, `XDG_CACHE_HOME` for the metadata cache override)
/// and spells out the consequence — store/cache become per-cwd, so every
/// `cd` looks like a fresh install. Tagged `GH #3550` so grepping a
/// "why are my installs always cold" symptom lands on this line.
///
/// Extracted as a free function so unit tests can pin the message shape
/// without inspecting log capture.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_pkg_manager_home_warn(err: &std::env::VarError) -> String {
    format!(
        "GH #3550 std::env::var(\"HOME\") failed ({err}); jet will place its global package store and metadata cache under the current working directory (\".\") instead of $HOME. The on-disk store + registry metadata cache become per-cwd, so every `cd` to a new directory behaves like a fresh install and the one-time migration from ~/.jet-store/ silently no-ops. Set HOME explicitly (or XDG_CACHE_HOME for the metadata cache alone) to restore the cross-directory cache."
    )
}

/// Resolve a usable home directory for jet's package-manager data files
/// (global store + metadata cache).
///
/// Returns the env value verbatim when `HOME` is set and non-empty.
/// On `Err` or empty-string, emits a single `tracing::warn!` (target
/// `jet::pkg_manager::env`) with [`format_pkg_manager_home_warn`] and
/// returns `PathBuf::from(".")`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn pkg_manager_home_or_fallback() -> PathBuf {
    pkg_manager_home_from_result(std::env::var("HOME"))
}

/// Pure helper that takes the `std::env::var` result so tests can drive
/// the branching contract without mutating process-global env state.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn pkg_manager_home_from_result(res: Result<String, std::env::VarError>) -> PathBuf {
    match res {
        Ok(home) if !home.is_empty() => PathBuf::from(home),
        Ok(_) => {
            let err = std::env::VarError::NotPresent;
            tracing::warn!(
                target: "jet::pkg_manager::env",
                env_var = "HOME",
                "{}",
                format_pkg_manager_home_warn(&err)
            );
            PathBuf::from(".")
        }
        Err(err) => {
            tracing::warn!(
                target: "jet::pkg_manager::env",
                env_var = "HOME",
                error = %err,
                "{}",
                format_pkg_manager_home_warn(&err)
            );
            PathBuf::from(".")
        }
    }
}

/// GH #3625 — env vars consulted by `is_ci_env`. Centralised so the
/// presence-detection contract is the same for production and tests.
pub(crate) const CI_ENV_VARS: &[&str] = &["CI", "GITHUB_ACTIONS", "GITLAB_CI", "JENKINS_URL"];

/// GH #3625 — `is_ci_env` previously called `std::env::var(name).is_ok()`
/// for every detection var, which collapses `Err(VarError::NotUnicode)`
/// (the var IS set, but contains non-UTF-8 bytes) into the same `false`
/// branch as `Err(VarError::NotPresent)`. A CI environment that exports
/// e.g. `CI` to a non-UTF-8 byte string is therefore treated as "not CI"
/// and the auto-frozen-lockfile guard at `install_all` quietly turns off,
/// letting `jet install` rewrite `jet-lock.yaml` on a CI machine.
///
/// The fix distinguishes the two error kinds. `NotPresent` is silent.
/// `NotUnicode` IS evidence of a CI signal: we report `true` and return a
/// human-readable warning string so the caller can route it through its
/// own `tracing` target with a compile-time-constant value.
///
/// Returned as `(bool, Option<String>)` to match the family-wide
/// `safe_*` helper shape (cf. [`safe_jit_stem`], [`safe_shard_key`],
/// [`safe_relative_path`]).
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn safe_is_ci_env<F>(vars: &[&str], mut read: F) -> (bool, Option<String>)
where
    F: FnMut(&str) -> Result<String, std::env::VarError>,
{
    let mut warn_msg: Option<String> = None;
    let mut detected = false;
    for name in vars {
        match read(name) {
            Ok(_) => {
                detected = true;
            }
            Err(std::env::VarError::NotPresent) => {}
            Err(err @ std::env::VarError::NotUnicode(_)) => {
                detected = true;
                let line = format_safe_is_ci_env_warn(name, &err);
                warn_msg = Some(match warn_msg {
                    Some(prev) => format!("{prev}; {line}"),
                    None => line,
                });
            }
        }
    }
    (detected, warn_msg)
}

/// GH #3625 — format the warning line for a single CI detection var
/// whose value is non-UTF-8. Names the variable, the underlying error,
/// the GH #3625 tag, and the operator-visible consequence (auto-frozen
/// lockfile guard re-engaged) so a grep on the symptom lands on this
/// line.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_safe_is_ci_env_warn(name: &str, err: &std::env::VarError) -> String {
    format!(
        "GH #3625 std::env::var(\"{name}\") returned {err}; the variable IS set but cannot be decoded as UTF-8. Treating this as a positive CI signal — jet's auto-frozen-lockfile guard is engaged for this `install`. Set {name} to a UTF-8 string (or unset it) to silence this warning."
    )
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl PackageManager {
    pub fn new(root_dir: PathBuf) -> Result<Self> {
        Self::new_with_flags(root_dir, false)
    }

    /// Create a package manager with optional `--no-cache` behaviour.
    ///
    /// When `no_cache` is `true`, the registry client skips disk metadata
    /// cache reads/writes and always fetches fresh data from the registry.
    pub fn new_with_flags(root_dir: PathBuf, no_cache: bool) -> Result<Self> {
        // Load .npmrc config (project > user > global)
        let npmrc = NpmrcConfig::load(&root_dir);

        // Global store at ~/.jet-store/
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let store_dir = PathBuf::from(home).join(".jet-store");
        let store = Arc::new(StoreManager::new(store_dir)?);

        let registry = Arc::new(RegistryClient::new_with_options(
            &npmrc.registry,
            &npmrc,
            no_cache,
        )?);
        let resolver = DependencyResolver::new();
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_DOWNLOADS));

        Ok(Self {
            root_dir,
            store,
            registry,
            resolver,
            semaphore,
        })
    }

    /// Install all dependencies from package.json.
    ///
    /// Fast-path: if `jet-lock.yaml` exists and every entry is valid
    /// in the store, skip resolution and install directly from the
    /// lockfile.
    pub async fn install(&self) -> Result<()> {
        self.install_with_options(false).await
    }

    /// Install with frozen lockfile support.
    /// In CI (CI=true, GITHUB_ACTIONS, etc.) frozen lockfile is auto-enabled.
    pub async fn install_with_options(&self, frozen_lockfile: bool) -> Result<()> {
        tracing::info!("Installing dependencies...");

        // Detect workspace mode first. Jet workspaces use a separate install
        // path that handles workspace: protocol deps via relative symlinks and
        // installs all packages in topological order.
        if let workspace::WorkspaceMode::Jet(mut ws_mgr) =
            workspace::WorkspaceMode::detect(&self.root_dir)?
        {
            let frozen = frozen_lockfile || Self::is_ci_env();
            return self.workspace_install_all(&mut ws_mgr, frozen).await;
        }

        let frozen = frozen_lockfile || Self::is_ci_env();
        let package_json = self.read_package_json()?;
        let mut all_deps = package_json.dependencies.clone();
        all_deps.extend(package_json.dev_dependencies.clone());

        // Apply overrides: force specific versions across dep tree
        let overrides = package_json.overrides.clone();
        let node_modules = self.root_dir.join("node_modules");

        // Frozen lockfile check
        if frozen {
            let lockfile_path = self.root_dir.join("jet-lock.yaml");
            if !lockfile_path.exists() {
                anyhow::bail!(
                    "Frozen lockfile: jet-lock.yaml not found. \
                     Run 'jet install' locally first."
                );
            }
            let lockfile = Lockfile::read(&lockfile_path)?;

            // Verify depsHash
            let current_hash = Lockfile::compute_deps_hash(&all_deps);
            if let Some(stored_hash) = &lockfile.deps_hash {
                if *stored_hash != current_hash {
                    anyhow::bail!(
                        "Frozen lockfile drift detected: \
                         package.json deps changed since lockfile was written. \
                         Run 'jet install' locally and commit jet-lock.yaml."
                    );
                }
            }

            // Frozen warm path: when the store, root node_modules links,
            // and the GH #3211 marker all still match the verified
            // lockfile, relinking every package is pure waste — npm/pnpm
            // warm installs no-op here and the basic.install.replacement
            // performance gate measures exactly this run. Same hydration
            // contract as the non-frozen ultra-fast path (GH #1930,
            // GH #1941): a stale marker must never hide a wiped store or
            // a broken root symlink.
            let nm_marker = node_modules.join(".jet-marker");
            if lockfile.verify_hydrated(&self.store, &node_modules).is_ok()
                && lockfile.is_valid(&self.store)
                && nm_marker.exists()
            {
                if let Ok(stored) = std::fs::read_to_string(&nm_marker) {
                    if stored.trim() == current_hash
                        && lockfile_root_links_valid(&node_modules, &lockfile)
                    {
                        tracing::info!("Already up to date");
                        println!("Already up to date");
                        return Ok(());
                    }
                }
            }

            let resolved = lockfile.to_resolved();
            self.install_resolved(&resolved).await?;
            // GH #3211 — marker write so the next frozen run can take the
            // warm path above; warn-only on failure, install already
            // succeeded.
            write_jet_marker(&node_modules, &nm_marker, &current_hash);
            tracing::info!("Dependencies installed (frozen lockfile)");
            return Ok(());
        }

        // Lockfile fast-path
        //
        // Issue #1930 — the marker fast-path used to short-circuit on
        // `is_valid()` (store-only) + hash match, which printed
        // `Already up to date` even when the store had been wiped or a
        // root `node_modules/` symlink was broken. We now strictly
        // verify both halves of the hydration contract before
        // returning, and on any defect fall through to the lockfile
        // install path so the broken state is repaired.
        let lockfile_path = self.root_dir.join("jet-lock.yaml");
        if lockfile_path.exists() {
            if let Ok(lockfile) = Lockfile::read(&lockfile_path) {
                let hydration = lockfile.verify_hydrated(&self.store, &node_modules);
                if let Err(defect) = &hydration {
                    tracing::info!(
                        "Lockfile present but project not fully \
                         hydrated ({}); running lockfile install \
                         to repair",
                        defect
                    );
                }

                if lockfile.is_valid(&self.store)
                    && lockfile_hash_matches_current_deps(&lockfile, &all_deps)
                {
                    // Ultra-fast path: if node_modules marker matches
                    // lockfile hash, everything is up-to-date.
                    let nm_marker = node_modules.join(".jet-marker");
                    let lf_hash = Lockfile::compute_deps_hash(&all_deps);

                    // Ultra-fast path: marker must match AND every
                    // store entry plus root node_modules link must be
                    // physically present. Without the hydration check
                    // a stale marker would hide a missing store dir
                    // or a broken symlink (Issue #1930, #1941).
                    if hydration.is_ok() && nm_marker.exists() {
                        if let Ok(stored) = std::fs::read_to_string(&nm_marker) {
                            if stored.trim() == lf_hash
                                && lockfile_root_links_valid(&node_modules, &lockfile)
                            {
                                tracing::info!("Already up to date");
                                println!("Already up to date");
                                return Ok(());
                            }
                        }
                    }

                    tracing::info!("Lockfile valid, using fast-path");
                    let resolved = lockfile.to_resolved();
                    self.install_resolved(&resolved).await?;

                    // GH #3211 — write the next-run marker. The prior
                    // `let _ = ...` silently dropped both create_dir_all
                    // and write failures; a failed marker write means
                    // every subsequent `jet install` pays the full
                    // cold-install cost with no diagnostic. Install
                    // already succeeded, so on write failure we warn
                    // and still return Ok — the user only notices the
                    // next slow install, and now the warn gives them a
                    // breadcrumb.
                    write_jet_marker(&node_modules, &nm_marker, &lf_hash);

                    tracing::info!("Dependencies installed (from lockfile)");
                    return Ok(());
                }
            }
        }

        // Full resolution with overlapping tarball prefetch (R7).
        //
        // As each package version is selected by the resolver, it is sent
        // immediately on an unbounded channel.  A background task concurrently
        // checks the store and begins tarball downloads so that network I/O
        // overlaps with the ongoing BFS resolution.  After resolution
        // completes the sender is dropped, causing the background task to
        // finish its remaining downloads before we proceed.
        let (prefetch_tx, mut prefetch_rx) =
            tokio::sync::mpsc::unbounded_channel::<ResolvedPackage>();

        let store_bg = self.store.clone();
        let registry_bg = self.registry.clone();
        let semaphore_bg = self.semaphore.clone();
        let prefetch_handle = tokio::spawn(async move {
            let mut prefetch_tasks = tokio::task::JoinSet::new();
            while let Some(pkg) = prefetch_rx.recv().await {
                if store_bg.has_package(&pkg.name, &pkg.version, &pkg.shasum) {
                    tracing::debug!("Prefetch: store hit {}@{}", pkg.name, pkg.version);
                    continue;
                }
                let store_inner = store_bg.clone();
                let reg_inner = registry_bg.clone();
                let sema_inner = semaphore_bg.clone();
                prefetch_tasks.spawn(async move {
                    let _permit = sema_inner.acquire().await.unwrap();
                    match reg_inner.download_package(&pkg.name, &pkg.version).await {
                        Ok(tarball) => {
                            prefetch_install_one(
                                &store_inner,
                                &pkg.name,
                                &pkg.version,
                                &tarball,
                                &pkg.shasum,
                            );
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Prefetch download failed {}@{}: {}",
                                pkg.name,
                                pkg.version,
                                e
                            );
                        }
                    }
                });
            }

            while let Some(join_result) = prefetch_tasks.join_next().await {
                if let Err(join_err) = join_result {
                    tracing::warn!(
                        target: "jet::pkg_manager::prefetch",
                        error = %join_err,
                        "{}",
                        format_prefetch_join_warn(&join_err)
                    );
                }
            }
        });

        let resolved = self
            .resolver
            .resolve_with_prefetch(&all_deps, &self.registry, &overrides, Some(prefetch_tx))
            .await?;

        // Wait for the background prefetch dispatcher and all child
        // download/install tasks to drain. If the detached children are still
        // extracting when foreground install starts, both paths can race on the
        // same store entry and leave a partial package with a valid marker.
        // GH #3518 — surface JoinError (panic / cancellation) so the dev sees
        // the partial-store consequence in the log instead of misattributing
        // later cold installs to "first-run slowness".
        if let Err(join_err) = prefetch_handle.await {
            tracing::warn!(
                target: "jet::pkg_manager::prefetch",
                error = %join_err,
                "{}",
                format_prefetch_join_warn(&join_err)
            );
        }

        self.install_resolved(&resolved).await?;
        self.write_lockfile(&resolved, &all_deps, &overrides)?;

        // GH #3211 — write the next-run marker. Same rationale as the
        // fast-path call site above: a silent write failure forces the
        // next `jet install` to pay the full cold-install cost; warn so
        // the next slow install is diagnosable.
        let lf_hash = Lockfile::compute_deps_hash(&all_deps);
        let nm_marker = node_modules.join(".jet-marker");
        write_jet_marker(&node_modules, &nm_marker, &lf_hash);

        tracing::info!("Dependencies installed successfully");
        Ok(())
    }

    /// Lock-only mode (R10): resolve the full dependency graph and write
    /// `jet-lock.yaml` without downloading or extracting any tarballs.
    ///
    /// Equivalent to `jet install --no-install`. Useful for CI pipelines that
    /// want a deterministic lockfile without materialising `node_modules`.
    pub async fn install_lockfile_only(&self) -> Result<()> {
        tracing::info!("Resolving dependencies (lock-only mode)...");

        let package_json = self.read_package_json()?;
        let mut all_deps = package_json.dependencies.clone();
        all_deps.extend(package_json.dev_dependencies.clone());
        let overrides = package_json.overrides.clone();

        let resolved = self
            .resolver
            .resolve(&all_deps, &self.registry, &overrides)
            .await?;

        self.write_lockfile(&resolved, &all_deps, &overrides)?;

        tracing::info!(
            "Lockfile written: jet-lock.yaml ({} packages, no downloads)",
            resolved.len()
        );
        println!(
            "Lockfile written: jet-lock.yaml ({} packages)",
            resolved.len()
        );
        Ok(())
    }

    /// Update packages to latest matching versions.
    pub async fn update(&self, package: Option<&str>, latest: bool) -> Result<()> {
        if let Some(pkg_name) = package {
            let new_version = if latest {
                self.registry.get_latest_version(pkg_name).await?
            } else {
                self.registry.get_latest_version(pkg_name).await?
            };
            let range = if latest {
                new_version.clone()
            } else {
                format!("^{}", new_version)
            };

            let (mut doc, indent, trailing_newline) = self.read_package_json_raw()?;
            if replace_existing_dep_entry(&mut doc, pkg_name, &range) {
                self.write_package_json_raw(&doc, &indent, trailing_newline)?;
            }
        }

        // Re-install with fresh resolution
        let package_json = self.read_package_json()?;
        let mut all_deps = package_json.dependencies.clone();
        all_deps.extend(package_json.dev_dependencies.clone());
        let overrides = package_json.overrides.clone();

        let resolved = self
            .resolver
            .resolve(&all_deps, &self.registry, &overrides)
            .await?;
        self.install_resolved(&resolved).await?;
        self.write_lockfile(&resolved, &all_deps, &overrides)?;

        tracing::info!("Dependencies updated successfully");
        Ok(())
    }

    /// Run security audit against installed packages.
    pub async fn audit(&self) -> Result<audit::AuditReport> {
        let lockfile_path = self.root_dir.join("jet-lock.yaml");
        let lockfile = Lockfile::read(&lockfile_path)
            .context("No jet-lock.yaml found. Run 'jet install' first.")?;

        let packages: HashMap<String, String> = lockfile
            .to_resolved()
            .into_iter()
            .map(|(name, pkg)| (name, pkg.version))
            .collect();

        let npmrc = NpmrcConfig::load(&self.root_dir);
        let client = audit::AuditClient::new(&npmrc.registry);
        client.audit(&packages).await
    }

    /// Detect CI environment.
    ///
    /// GH #3625 — delegates to [`safe_is_ci_env`] so a non-UTF-8
    /// detection var (`Err(VarError::NotUnicode)`) still counts as
    /// "CI is present" instead of collapsing to "not CI" the way
    /// the previous `.is_ok()` chain did.
    fn is_ci_env() -> bool {
        let (is_ci, warn) = safe_is_ci_env(CI_ENV_VARS, |name| std::env::var(name));
        if let Some(msg) = warn {
            tracing::warn!(
                target: "jet::pkg_manager::env",
                "{}",
                msg
            );
        }
        is_ci
    }

    /// Add a single package to dependencies (or devDependencies) and
    /// re-install. Thin wrapper around [`add_many`] preserved for
    /// backward compatibility.
    pub async fn add(&self, package: &str, dev: bool) -> Result<()> {
        self.add_many(&[package], dev).await
    }

    /// Add one or more package specs to dependencies (or devDependencies)
    /// and re-install once for the whole set.
    ///
    /// Each spec may include an explicit version range (`react@^18`,
    /// `lodash@4.17.21`) or omit it (`@mui/material`) to fetch the
    /// registry's latest version. Scoped names (`@scope/name`) are
    /// supported — the leading `@` is not treated as a version separator.
    ///
    /// `package.json` is updated atomically for the whole set before any
    /// install runs. If a registry lookup fails the error names the
    /// offending package and `package.json` is left untouched.
    ///
    /// Issue #1941: this method intentionally operates on the raw JSON
    /// document so that unknown top-level fields (`private`, `type`,
    /// `scripts`, …), key order, indentation, and trailing newline are
    /// preserved.  When `dev` is `false` and the package is already a
    /// `devDependencies` entry, the version is updated **in place** so
    /// that we do not produce a duplicate `dependencies`/
    /// `devDependencies` pair pointing at the same package name.
    pub async fn add_many(&self, packages: &[&str], dev: bool) -> Result<()> {
        if packages.is_empty() {
            anyhow::bail!("jet add: at least one package is required");
        }

        // Resolve every spec first so a single bad spec aborts the whole
        // batch before we mutate package.json.
        let mut resolved: Vec<(String, String)> = Vec::with_capacity(packages.len());
        for spec in packages {
            tracing::info!("Adding package: {}", spec);
            let (name, range) = self
                .resolve_add_spec(spec)
                .await
                .with_context(|| format!("Failed to resolve package spec '{}'", spec))?;
            resolved.push((name, range));
        }

        let (mut doc, indent, trailing_newline) = self.read_package_json_raw()?;
        for (name, range) in &resolved {
            update_dep_entry(&mut doc, name, range, dev);
        }
        self.write_package_json_raw(&doc, &indent, trailing_newline)?;

        self.install().await?;
        Ok(())
    }

    /// Resolve a `jet add` spec into a `(name, version-range)` pair.
    ///
    /// If the spec carries an explicit range (`name@range`) it is kept
    /// verbatim. Otherwise the registry's latest version is fetched and
    /// pinned with a caret prefix (`^x.y.z`).
    async fn resolve_add_spec(&self, spec: &str) -> Result<(String, String)> {
        let (name, explicit_range) = parse_package_spec(spec)?;
        let range = match explicit_range {
            Some(r) => r,
            None => {
                let version = self.registry.get_latest_version(&name).await?;
                format!("^{}", version)
            }
        };
        Ok((name, range))
    }

    /// Remove a package from both dependencies and devDependencies.
    pub async fn remove(&self, package: &str) -> Result<()> {
        tracing::info!("Removing package: {}", package);

        let (mut doc, indent, trailing_newline) = self.read_package_json_raw()?;
        let changed = remove_dep_entry(&mut doc, package);
        if changed {
            self.write_package_json_raw(&doc, &indent, trailing_newline)?;
            self.install().await?;
            let lockfile_path = self.root_dir.join("jet-lock.yaml");
            if lockfile_path.exists() {
                let lockfile = Lockfile::read(&lockfile_path)?;
                prune_node_modules_to_lockfile(&self.root_dir.join("node_modules"), &lockfile)?;
            }
        }

        Ok(())
    }

    /// Download, extract, link, setup bins, and run lifecycle hooks.
    async fn install_resolved(&self, resolved: &HashMap<String, ResolvedPackage>) -> Result<()> {
        let node_modules = self.root_dir.join("node_modules");
        std::fs::create_dir_all(&node_modules)?;

        // Phase 1: Download + extract + hardlink (parallel)
        let futures: Vec<_> = resolved
            .values()
            .map(|pkg| {
                let store = self.store.clone();
                let registry = self.registry.clone();
                let semaphore = self.semaphore.clone();
                let node_modules = node_modules.clone();
                let pkg = pkg.clone();

                async move {
                    // Fast-path: skip link if already installed
                    let link_target = if let Some(ref parent) = pkg.nested_in {
                        let nested_nm = node_modules.join(parent).join("node_modules");
                        nested_nm
                    } else {
                        node_modules.clone()
                    };

                    // Ensure store has the package
                    if !store.has_package(&pkg.name, &pkg.version, &pkg.shasum) {
                        let _permit = semaphore.acquire().await.unwrap();
                        let tarball = registry.download_package(&pkg.name, &pkg.version).await?;
                        store.install_package(&pkg.name, &pkg.version, &tarball, &pkg.shasum)?;
                    }

                    if is_pkg_installed(&link_target, &pkg.name, &pkg.version) {
                        return Ok::<_, anyhow::Error>(());
                    }

                    // Link from store into node_modules
                    if pkg.nested_in.is_some() {
                        std::fs::create_dir_all(&link_target)?;
                    }
                    store.link_package(&pkg.name, &pkg.version, &link_target)?;

                    Ok::<_, anyhow::Error>(())
                }
            })
            .collect();

        futures::future::try_join_all(futures).await?;

        // Phase 2: Create nested node_modules for transitive dep resolution
        {
            // Build version map: pkg_name → resolved_version
            let version_map: HashMap<String, String> = resolved
                .iter()
                .map(|(name, pkg)| (name.clone(), pkg.version.clone()))
                .collect();

            for pkg in resolved.values() {
                if let Err(e) = self.store.create_nested_node_modules(
                    &pkg.name,
                    &pkg.version,
                    &version_map,
                    &version_map, // peer versions from project root
                ) {
                    tracing::debug!(
                        "Failed to create nested node_modules for {}@{}: {}",
                        pkg.name,
                        pkg.version,
                        e
                    );
                }
            }
        }

        // Phase 3: Link bin scripts (sequential — fast, disk only)
        for pkg in resolved.values() {
            if !pkg.bin.is_empty() {
                self.store.link_bins(&pkg.name, &pkg.bin, &node_modules)?;
            }
        }

        // Phase 4: Run lifecycle scripts (sequential — must be ordered)
        for pkg in resolved.values() {
            if pkg.has_install_script {
                for script in ["preinstall", "install", "postinstall"] {
                    self.store
                        .run_lifecycle_script(&pkg.name, script, &node_modules)
                        .await?;
                }
            }
        }

        prune_third_party_node_modules_layout(&node_modules)?;

        Ok(())
    }

    fn read_package_json(&self) -> Result<PackageJson> {
        let path = self.root_dir.join("package.json");
        let content =
            std::fs::read_to_string(&path).with_context(|| format!("Failed to read {:?}", path))?;
        let package: PackageJson = serde_json::from_str(&content)?;
        Ok(package)
    }

    /// Raw read variant used by mutating operations (`add`, `remove`,
    /// `update`).  Returns the parsed `Value` plus the detected indent
    /// string and whether the original file ended with a newline so
    /// that the rewrite preserves the document's surface formatting.
    ///
    /// Issue #1941: we never project through the typed `PackageJson`
    /// view here — going through serde would drop unknown top-level
    /// fields the moment `extra` was stale relative to the file on
    /// disk.
    fn read_package_json_raw(&self) -> Result<(serde_json::Value, String, bool)> {
        let path = self.root_dir.join("package.json");
        let content =
            std::fs::read_to_string(&path).with_context(|| format!("Failed to read {:?}", path))?;
        let value: serde_json::Value = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {:?}", path))?;
        let indent = detect_json_indent(&content);
        let trailing_newline = content.ends_with('\n');
        Ok((value, indent, trailing_newline))
    }

    fn write_package_json_raw(
        &self,
        value: &serde_json::Value,
        indent: &str,
        trailing_newline: bool,
    ) -> Result<()> {
        let path = self.root_dir.join("package.json");
        let mut buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        use serde::Serialize;
        value
            .serialize(&mut ser)
            .with_context(|| package_json_serialize_ctx(&path))?;
        let mut content = String::from_utf8(buf).with_context(|| package_json_utf8_ctx(&path))?;
        if trailing_newline && !content.ends_with('\n') {
            content.push('\n');
        }
        std::fs::write(&path, content).with_context(|| package_json_write_io_ctx(&path))?;
        Ok(())
    }

    fn write_lockfile(
        &self,
        resolved: &HashMap<String, ResolvedPackage>,
        all_deps: &HashMap<String, String>,
        overrides: &HashMap<String, String>,
    ) -> Result<()> {
        let mut lockfile = Lockfile::from_resolved(resolved);
        lockfile.deps_hash = Some(Lockfile::compute_deps_hash(all_deps));
        lockfile.overrides = overrides.clone();
        let path = self.root_dir.join("jet-lock.yaml");
        lockfile.write(&path)?;
        Ok(())
    }
}

/// Compute the relative path from directory `from_dir` to directory `to_dir`.
fn make_relative_path(from_dir: &Path, to_dir: &Path) -> PathBuf {
    let from: Vec<_> = from_dir.components().collect();
    let to: Vec<_> = to_dir.components().collect();

    let common = from
        .iter()
        .zip(to.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let mut rel = PathBuf::new();
    for _ in 0..(from.len() - common) {
        rel.push("..");
    }
    for comp in &to[common..] {
        rel.push(comp.as_os_str());
    }
    rel
}

/// Create (or update idempotently) a relative symlink at `link_path` pointing
/// to `target_dir`. The symlink target is expressed as a relative path so that
/// the workspace tree is fully portable.
fn create_relative_symlink(link_path: &Path, target_dir: &Path) -> Result<()> {
    let link_parent = link_path.parent().unwrap_or(Path::new("."));

    // Resolve both paths to absolute for reliable relative-path computation.
    let abs_target = if target_dir.is_absolute() {
        target_dir.to_path_buf()
    } else {
        std::env::current_dir()?.join(target_dir)
    };
    let abs_link_parent = if link_parent.is_absolute() {
        link_parent.to_path_buf()
    } else {
        std::env::current_dir()?.join(link_parent)
    };

    let rel = make_relative_path(&abs_link_parent, &abs_target);

    // Idempotency: if an existing symlink already points to the same target, skip.
    if link_path.is_symlink() {
        if let Ok(existing) = std::fs::read_link(link_path) {
            if existing == rel {
                return Ok(());
            }
        }
        std::fs::remove_file(link_path)?;
    } else if link_path.exists() {
        std::fs::remove_dir_all(link_path)?;
    }

    // Ensure parent directory exists.
    if let Some(parent) = link_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(&rel, link_path)?;

    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&rel, link_path)?;

    Ok(())
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl PackageManager {
    /// Install workspace packages in topological order.
    ///
    /// For each workspace package:
    /// - `workspace:*` / `workspace:^` / `workspace:~` deps → relative symlink
    /// - External deps → standard registry resolve / download / hardlink
    ///
    /// A single `jet-lock.yaml` is written at the workspace root after all
    /// packages are installed.
    async fn workspace_install_all(
        &self,
        ws_mgr: &mut workspace::WorkspaceManager,
        frozen_lockfile: bool,
    ) -> Result<()> {
        let order = ws_mgr.topological_order()?;
        let ws_root = &self.root_dir;
        let deps_hash = workspace_deps_hash(ws_mgr);

        if frozen_lockfile {
            let lockfile_path = ws_root.join("jet-lock.yaml");
            if !lockfile_path.exists() {
                anyhow::bail!(
                    "Frozen lockfile: jet-lock.yaml not found. \
                     Run 'jet install' locally first."
                );
            }
            let lockfile = Lockfile::read(&lockfile_path)?;
            match lockfile.deps_hash.as_deref() {
                Some(stored) if stored == deps_hash => {}
                _ => {
                    anyhow::bail!(
                        "Frozen lockfile drift detected: workspace package deps changed since \
                         lockfile was written. Run 'jet install' locally and commit jet-lock.yaml."
                    );
                }
            }
        }

        let mut lockfile = Lockfile::new();
        lockfile.deps_hash = Some(deps_hash);

        for pkg_name in &order {
            let pkg = ws_mgr
                .get_package(pkg_name)
                .ok_or_else(|| anyhow::anyhow!("Workspace package '{}' not found", pkg_name))?
                .clone();

            let pkg_dir = ws_root.join(&pkg.path);

            // Read direct deps from this package's package.json.
            let pkg_json_path = pkg_dir.join("package.json");
            let content = std::fs::read_to_string(&pkg_json_path)
                .with_context(|| format!("Cannot read {:?}", pkg_json_path))?;
            let pkg_json: serde_json::Value = serde_json::from_str(&content)?;

            let mut all_deps: HashMap<String, String> = HashMap::new();
            for field in &["dependencies", "devDependencies"] {
                if let Some(deps) = pkg_json.get(field).and_then(|v| v.as_object()) {
                    for (name, ver) in deps {
                        if let Some(ver_str) = ver.as_str() {
                            all_deps.insert(name.clone(), ver_str.to_string());
                        }
                    }
                }
            }

            let node_modules = pkg_dir.join("node_modules");
            std::fs::create_dir_all(&node_modules)?;

            let mut workspace_deps: Vec<(String, String)> = Vec::new();
            let mut external_deps: HashMap<String, String> = HashMap::new();

            for (dep_name, dep_spec) in &all_deps {
                if workspace::WorkspaceManager::is_workspace_protocol(dep_spec) {
                    workspace_deps.push((dep_name.clone(), dep_spec.clone()));
                } else {
                    external_deps.insert(dep_name.clone(), dep_spec.clone());
                }
            }

            // --- Workspace deps: symlink only, no registry call ---
            for (dep_name, dep_spec) in &workspace_deps {
                let target_pkg = ws_mgr.get_package(dep_name).ok_or_else(|| {
                    anyhow::anyhow!(
                        "Workspace package '{}' not found (required by '{}')",
                        dep_name,
                        pkg_name
                    )
                })?;

                let resolved_version = ws_mgr
                    .resolve_workspace_protocol(dep_spec, dep_name)
                    .unwrap_or_else(|| target_pkg.version.clone());

                let abs_target = ws_root.join(&target_pkg.path);
                let link_path = node_modules.join(dep_name);
                create_relative_symlink(&link_path, &abs_target)?;

                let lf_key = format!("/{}@{}", dep_name, resolved_version);
                let local_path = target_pkg.path.to_string_lossy().to_string();
                lockfile
                    .packages
                    .entry(lf_key)
                    .or_insert_with(|| LockfileEntry {
                        version: resolved_version.clone(),
                        resolution: Resolution {
                            tarball: String::new(),
                            shasum: String::new(),
                            integrity: None,
                        },
                        workspace: true,
                        local_path: Some(local_path),
                        dependencies: HashMap::new(),
                        peer_dependencies: HashMap::new(),
                        bin: HashMap::new(),
                        has_install_script: false,
                        nested_in: None,
                    });
            }

            // --- External deps: standard registry flow ---
            if !external_deps.is_empty() {
                let overrides = HashMap::new();
                let resolved = self
                    .resolver
                    .resolve(&external_deps, &self.registry, &overrides)
                    .await?;

                self.install_resolved_to(&resolved, &pkg_dir).await?;

                for (name, rp) in &resolved {
                    let lf_key = format!("/{}@{}", name, rp.version);
                    lockfile
                        .packages
                        .entry(lf_key)
                        .or_insert_with(|| LockfileEntry {
                            version: rp.version.clone(),
                            resolution: Resolution {
                                tarball: rp.tarball_url.clone(),
                                shasum: rp.shasum.clone(),
                                integrity: rp.integrity.clone(),
                            },
                            workspace: false,
                            local_path: None,
                            dependencies: rp.dependencies.clone(),
                            peer_dependencies: rp.peer_dependencies.clone(),
                            bin: rp.bin.clone(),
                            has_install_script: rp.has_install_script,
                            nested_in: rp.nested_in.clone(),
                        });
                }
            }
        }

        if !frozen_lockfile {
            let lf_path = ws_root.join("jet-lock.yaml");
            lockfile.write(&lf_path)?;
        }

        tracing::info!("Workspace install complete ({} packages)", order.len());
        Ok(())
    }

    /// Like `install_resolved` but installs into an arbitrary `pkg_dir` instead
    /// of `self.root_dir`. Used by `workspace_install_all` for per-package installs.
    async fn install_resolved_to(
        &self,
        resolved: &HashMap<String, ResolvedPackage>,
        pkg_dir: &Path,
    ) -> Result<()> {
        let node_modules = pkg_dir.join("node_modules");
        std::fs::create_dir_all(&node_modules)?;

        let futures: Vec<_> = resolved
            .values()
            .map(|pkg| {
                let store = self.store.clone();
                let registry = self.registry.clone();
                let semaphore = self.semaphore.clone();
                let node_modules = node_modules.clone();
                let pkg = pkg.clone();

                async move {
                    // Skip workspace packages — they are handled via symlinks.
                    if pkg.workspace {
                        return Ok::<_, anyhow::Error>(());
                    }

                    let link_target = if let Some(ref parent) = pkg.nested_in {
                        node_modules.join(parent).join("node_modules")
                    } else {
                        node_modules.clone()
                    };

                    if !store.has_package(&pkg.name, &pkg.version, &pkg.shasum) {
                        let _permit = semaphore.acquire().await.unwrap();
                        let tarball = registry.download_package(&pkg.name, &pkg.version).await?;
                        store.install_package(&pkg.name, &pkg.version, &tarball, &pkg.shasum)?;
                    }

                    if is_pkg_installed(&link_target, &pkg.name, &pkg.version) {
                        return Ok(());
                    }

                    if pkg.nested_in.is_some() {
                        std::fs::create_dir_all(&link_target)?;
                    }
                    store.link_package(&pkg.name, &pkg.version, &link_target)?;

                    Ok::<_, anyhow::Error>(())
                }
            })
            .collect();

        futures::future::try_join_all(futures).await?;

        for pkg in resolved.values() {
            if !pkg.bin.is_empty() {
                self.store.link_bins(&pkg.name, &pkg.bin, &node_modules)?;
            }
        }

        prune_third_party_node_modules_layout(&node_modules)?;

        Ok(())
    }
}

/// Fast check: does `node_modules/{name}/package.json` already
/// have the expected version?  Avoids re-linking unchanged packages.
fn is_pkg_installed(node_modules: &std::path::Path, name: &str, version: &str) -> bool {
    let pkg_json = node_modules.join(name).join("package.json");
    let Ok(content) = std::fs::read_to_string(&pkg_json) else {
        return false;
    };
    // Lightweight version extraction without full JSON parse
    // Look for "version": "X.Y.Z"
    if let Some(pos) = content.find("\"version\"") {
        let rest = &content[pos + 9..];
        if let Some(colon) = rest.find(':') {
            let after_colon = rest[colon + 1..].trim_start();
            if after_colon.starts_with('"') {
                let inner = &after_colon[1..];
                if let Some(end) = inner.find('"') {
                    return &inner[..end] == version;
                }
            }
        }
    }
    false
}

/// Verify root lockfile packages are visible from `node_modules` before the
/// marker fast path reports that the install is already current.
fn lockfile_root_links_valid(node_modules: &Path, lockfile: &Lockfile) -> bool {
    lockfile
        .to_resolved()
        .values()
        .filter(|pkg| !pkg.workspace && pkg.nested_in.is_none())
        .all(|pkg| is_pkg_installed(node_modules, &pkg.name, &pkg.version))
}

fn lockfile_hash_matches_current_deps(
    lockfile: &Lockfile,
    all_deps: &HashMap<String, String>,
) -> bool {
    lockfile
        .deps_hash
        .as_deref()
        .is_some_and(|stored| stored == Lockfile::compute_deps_hash(all_deps))
}

fn workspace_deps_hash(ws_mgr: &workspace::WorkspaceManager) -> String {
    let mut deps = HashMap::new();
    for pkg in &ws_mgr.packages {
        for (name, spec) in &pkg.dependencies {
            deps.insert(format!("{}:dependencies:{}", pkg.name, name), spec.clone());
        }
        for (name, spec) in &pkg.dev_dependencies {
            deps.insert(
                format!("{}:devDependencies:{}", pkg.name, name),
                spec.clone(),
            );
        }
    }
    Lockfile::compute_deps_hash(&deps)
}

/// GH #3211 — write the `.jet-marker` file containing the current
/// lockfile hash. The prior call sites used `let _ = std::fs::write(...)`
/// (and `let _ = std::fs::create_dir_all(...)`), silently swallowing
/// failures. A failed marker write means every subsequent `jet install`
/// pays the full cold-install cost with no diagnostic; this helper
/// surfaces failures via `tracing::warn!`. Install itself already
/// succeeded by the time we get here, so we never propagate the error —
/// the warn is the only contract.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn write_jet_marker(node_modules: &Path, nm_marker: &Path, lf_hash: &str) {
    if let Err(e) = std::fs::create_dir_all(node_modules) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            tracing::warn!(
                target: "jet::pkg_manager::install",
                "GH #3211 failed to create node_modules at {} for marker write: {} — \
                 the next `jet install` will skip the fast-path and pay a full \
                 cold-install cost",
                node_modules.display(),
                e,
            );
            // Don't bother trying the write — it will also fail.
            return;
        }
    }
    if let Err(e) = std::fs::write(nm_marker, lf_hash) {
        tracing::warn!(
            target: "jet::pkg_manager::install",
            "GH #3211 failed to write {} marker: {} — \
             the next `jet install` will skip the fast-path and pay a full \
             cold-install cost",
            nm_marker.display(),
            e,
        );
    }
}

/// Detect the indent string used by the first nested member of a JSON
/// document.  Falls back to two spaces — the npm/pnpm default — for
/// flat documents, empty objects, or anything we cannot recognise.
///
/// Only the two common indents are detected (spaces and tabs); any
/// other whitespace style is normalised to two spaces.  This is the
/// same heuristic pnpm uses (`detect-indent`-equivalent).
fn detect_json_indent(content: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim_start_matches(|c: char| c == ' ' || c == '\t');
        if trimmed.len() == line.len() || trimmed.is_empty() {
            continue;
        }
        let leading = &line[..line.len() - trimmed.len()];
        if leading.starts_with('\t') {
            return "\t".to_string();
        }
        let spaces = leading.chars().take_while(|c| *c == ' ').count();
        if spaces > 0 {
            return " ".repeat(spaces);
        }
    }
    "  ".to_string()
}

/// Insert or update `package` in the appropriate dependency group on
/// the raw package.json `Value`.
///
/// Issue #1941: when `dev == false` and the package is already a
/// `devDependencies` entry we update it **in place** rather than
/// inserting a duplicate under `dependencies`.  Symmetrically, when
/// `dev == true` we move an existing `dependencies` entry into
/// `devDependencies`.  This matches the user's expectation that
/// `jet add foo` should be idempotent against an existing
/// installation.
fn update_dep_entry(doc: &mut serde_json::Value, package: &str, range: &str, dev: bool) {
    let obj = match doc.as_object_mut() {
        Some(o) => o,
        None => return,
    };

    let in_deps = obj
        .get("dependencies")
        .and_then(|v| v.as_object())
        .map(|o| o.contains_key(package))
        .unwrap_or(false);
    let in_dev_deps = obj
        .get("devDependencies")
        .and_then(|v| v.as_object())
        .map(|o| o.contains_key(package))
        .unwrap_or(false);

    let target = if dev {
        "devDependencies"
    } else if in_dev_deps && !in_deps {
        // Already a dev dep — keep it there (R2).
        "devDependencies"
    } else {
        "dependencies"
    };
    let other = if target == "dependencies" {
        "devDependencies"
    } else {
        "dependencies"
    };

    // Remove the entry from the *other* group so we never produce a
    // duplicate dependencies/devDependencies pair pointing at the same
    // package name (issue #1941 actual bullet 2).
    if let Some(entry) = obj.get_mut(other).and_then(|v| v.as_object_mut()) {
        entry.shift_remove(package);
    }

    // Ensure the target group exists and insert / update the package.
    if !obj.get(target).is_some_and(|v| v.is_object()) {
        obj.insert(
            target.to_string(),
            serde_json::Value::Object(serde_json::Map::new()),
        );
    }
    if let Some(entry) = obj.get_mut(target).and_then(|v| v.as_object_mut()) {
        entry.insert(
            package.to_string(),
            serde_json::Value::String(range.to_string()),
        );
    }
}

fn replace_existing_dep_entry(doc: &mut serde_json::Value, package: &str, range: &str) -> bool {
    let Some(obj) = doc.as_object_mut() else {
        return false;
    };

    for group in ["dependencies", "devDependencies", "optionalDependencies"] {
        if let Some(entry) = obj.get_mut(group).and_then(|v| v.as_object_mut()) {
            if entry.contains_key(package) {
                entry.insert(
                    package.to_string(),
                    serde_json::Value::String(range.to_string()),
                );
                return true;
            }
        }
    }
    false
}

fn remove_dep_entry(doc: &mut serde_json::Value, package: &str) -> bool {
    let Some(obj) = doc.as_object_mut() else {
        return false;
    };

    let mut changed = false;
    for group in ["dependencies", "devDependencies", "optionalDependencies"] {
        if let Some(entry) = obj.get_mut(group).and_then(|v| v.as_object_mut()) {
            changed |= entry.shift_remove(package).is_some();
        }
    }
    changed
}

fn lockfile_root_package_names(lockfile: &Lockfile) -> HashSet<String> {
    lockfile
        .to_resolved()
        .into_values()
        .filter(|pkg| pkg.nested_in.is_none())
        .map(|pkg| pkg.name)
        .collect()
}

fn lockfile_bin_names(lockfile: &Lockfile) -> HashSet<String> {
    lockfile
        .packages
        .values()
        .flat_map(|entry| entry.bin.keys().cloned())
        .collect()
}

fn prune_node_modules_to_lockfile(node_modules: &Path, lockfile: &Lockfile) -> Result<()> {
    if !node_modules.exists() {
        return Ok(());
    }

    let desired = lockfile_root_package_names(lockfile);
    for (name, path) in list_root_node_modules_packages(node_modules)? {
        if !desired.contains(&name) {
            remove_node_modules_entry(&path)
                .with_context(|| format!("Failed to remove stale node_modules entry {}", name))?;
        }
    }

    let desired_bins = lockfile_bin_names(lockfile);
    let bin_dir = node_modules.join(".bin");
    if bin_dir.is_dir() {
        for entry in std::fs::read_dir(&bin_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if !desired_bins.contains(&name) {
                let path = entry.path();
                std::fs::remove_file(&path).with_context(|| {
                    format!("Failed to remove stale bin shim {}", path.display())
                })?;
            }
        }
    }

    Ok(())
}

fn prune_third_party_node_modules_layout(node_modules: &Path) -> Result<()> {
    for name in [
        ".pnpm",
        ".package-lock.json",
        ".modules.yaml",
        ".yarn-state.yml",
        ".bun-tag",
    ] {
        let path = node_modules.join(name);
        if path.exists() || path.is_symlink() {
            remove_node_modules_entry(&path)
                .with_context(|| format!("Failed to remove stale package-manager layout {name}"))?;
        }
    }
    Ok(())
}

fn list_root_node_modules_packages(node_modules: &Path) -> Result<Vec<(String, PathBuf)>> {
    let mut packages = Vec::new();
    for entry in std::fs::read_dir(node_modules)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if matches!(
            name.as_str(),
            ".bin" | ".jet" | ".jet-marker" | ".vite-temp"
        ) {
            continue;
        }

        let path = entry.path();
        if name.starts_with('@') && path.is_dir() {
            for scoped_entry in std::fs::read_dir(&path)? {
                let scoped_entry = scoped_entry?;
                let scoped_name = scoped_entry.file_name().to_string_lossy().to_string();
                packages.push((format!("{name}/{scoped_name}"), scoped_entry.path()));
            }
        } else {
            packages.push((name, path));
        }
    }
    Ok(packages)
}

fn remove_node_modules_entry(path: &Path) -> Result<()> {
    let metadata = path.symlink_metadata()?;
    if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }

    if let Some(scope_dir) = path.parent().filter(|p| {
        p.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with('@'))
    }) {
        if scope_dir
            .read_dir()
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false)
        {
            let _ = std::fs::remove_dir(scope_dir);
        }
    }

    Ok(())
}

/// GH #3522 — build the warn message for a `Lockfile::read` failure
/// at the install entry point. Extracted so the wording is unit
/// testable without provoking a real corrupt-YAML scenario in the
/// integration path.
///
/// The message names the lockfile path verbatim so the dev can `cat`
/// it, preserves the underlying error (typically a `serde_yaml`
/// diagnostic with line/column or an io::Error), and explains the
/// user-visible "every install is slow" consequence.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_lockfile_read_warn(path: &std::path::Path, err: &anyhow::Error) -> String {
    format!(
        "GH #3522 lockfile read failed at {}: {err}; install will \
         fall through to slow regeneration. Common causes: merge \
         conflict markers in jet-lock.yaml, interrupted write, schema \
         drift from a downgrade. Inspect the file and either fix the \
         markers or delete the lockfile to force a clean regeneration.",
        path.display()
    )
}

/// GH #3518 — build the warn message for a `JoinError` that escaped
/// the prefetch dispatcher. Extracted so the wording is unit-testable
/// without provoking a real panic in a Tokio task.
///
/// Distinguishes `is_panic()` from `is_cancelled()` because the
/// operator response is different:
/// - Panic → a real bug in the dispatcher; the store is partial.
/// - Cancelled → only happens during runtime teardown (e.g. ^C); the
///   store is partial but the user already knows install was aborted.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_prefetch_join_warn(err: &tokio::task::JoinError) -> String {
    let kind = if err.is_panic() {
        "panicked"
    } else if err.is_cancelled() {
        "was cancelled"
    } else {
        "joined with an unknown error"
    };
    format!(
        "GH #3518 prefetch dispatcher {kind}: {err}; the on-disk store \
         may be missing some packages. The foreground installer will \
         redo missing downloads, but subsequent installs may also be \
         slow until the store warms back up."
    )
}

/// GH #3562 — context string for `std::fs::read_to_string` failures
/// inside `PackageManager::read_package_json`. Names the offending
/// `package.json` path so that, in monorepo trees with many workspace
/// packages, the dev can land on which file failed.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn package_json_read_io_ctx(path: &Path) -> String {
    format!(
        "GH #3562 reading package.json from {} failed (e.g. missing file, EACCES, EIO); the package.json at this path could not be opened",
        path.display()
    )
}

/// GH #3562 — context string for `serde_json::from_str` failures inside
/// `PackageManager::read_package_json`. Distinguishes a malformed
/// package.json from a missing one so the dev can target the syntax
/// rather than the filesystem.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn package_json_parse_ctx(path: &Path) -> String {
    format!(
        "GH #3562 parsing package.json from {} failed; the package.json at this path is malformed JSON (typical cause: stray git merge markers, trailing comma, or a hand-edited field out of shape)",
        path.display()
    )
}

/// GH #3562 — context string for `serde_json::to_string_pretty` and
/// `serde_json::Value::serialize` failures inside
/// `PackageManager::write_package_json` / `write_package_json_raw`.
/// Names the destination path so the dev can correlate the in-memory
/// failure with the file that was about to be rewritten.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn package_json_serialize_ctx(path: &Path) -> String {
    format!(
        "GH #3562 serializing package.json for write to {} failed; the in-memory package representation could not be encoded as JSON",
        path.display()
    )
}

/// GH #3562 — context string for `std::fs::write` failures inside
/// `PackageManager::write_package_json` / `write_package_json_raw`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn package_json_write_io_ctx(path: &Path) -> String {
    format!(
        "GH #3562 writing package.json to {} failed (e.g. read-only mount, ENOSPC, EACCES); the package.json at this path could not be persisted and the add/remove/update operation will not be reflected",
        path.display()
    )
}

/// GH #3562 — context string for `String::from_utf8` failures inside
/// `PackageManager::write_package_json_raw`. Should be unreachable in
/// practice because `serde_json` only emits ASCII, but propagating it
/// names the path so a corrupt formatter would not surface as an
/// orphan FromUtf8Error.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn package_json_utf8_ctx(path: &Path) -> String {
    format!(
        "GH #3562 encoding serialized package.json for write to {} as UTF-8 failed; this indicates a serializer-emitted non-UTF8 byte sequence and should not happen in practice",
        path.display()
    )
}

/// GH #3492 — run `install_package` for one prefetched tarball and
/// surface the Result. The prior call site discarded errors via
/// `let _ = ...`, so shasum mismatches and filesystem write failures
/// were invisible — the user only saw a generic "package missing
/// from store" later, with no breadcrumb back to the silent install
/// drop. The debug "downloaded" line is now gated to the Ok branch
/// so it cannot misrepresent a failed install as a success.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn prefetch_install_one(
    store: &StoreManager,
    name: &str,
    version: &str,
    tarball: &[u8],
    shasum: &str,
) {
    match store.install_package(name, version, tarball, shasum) {
        Ok(()) => {
            tracing::debug!("Prefetch: downloaded {}@{}", name, version);
        }
        Err(err) => {
            tracing::warn!(
                target: "jet::pkg_manager::prefetch",
                package = %name,
                version = %version,
                error = %err,
                "GH #3492 prefetch install_package failed; the package \
                 is NOT in the store. The main resolver will redo the \
                 download/install on the foreground path. Shasum \
                 mismatches here are a SECURITY signal — investigate \
                 whether the registry response was tampered with."
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── GH #3522: corrupt jet-lock.yaml silent fall-through ─────────────

    /// GH #3522 — the lockfile-read warn message must name the
    /// lockfile path, preserve the underlying error verbatim, AND
    /// include the GH #3522 tag.
    #[test]
    fn gh3522_lockfile_read_warn_names_path_error_and_issue() {
        let path = std::path::Path::new("/proj/jet-lock.yaml");
        let err = anyhow::anyhow!("invalid YAML at line 17: expected mapping, found scalar");
        let msg = format_lockfile_read_warn(path, &err);

        assert!(
            msg.contains("/proj/jet-lock.yaml"),
            "must name lockfile path, got: {msg}"
        );
        assert!(
            msg.contains("invalid YAML at line 17"),
            "must preserve underlying error verbatim, got: {msg}"
        );
        assert!(
            msg.contains("GH #3522"),
            "must include searchable issue tag, got: {msg}"
        );
    }

    /// GH #3522 — the message must call out common causes so the dev
    /// does not have to guess (merge markers is the #1 cause; a
    /// rebase-conflict lockfile is the typical repro).
    #[test]
    fn gh3522_lockfile_read_warn_hints_common_causes() {
        let path = std::path::Path::new("/proj/jet-lock.yaml");
        let err = anyhow::anyhow!("io error: EOF while parsing");
        let msg = format_lockfile_read_warn(path, &err);

        assert!(
            msg.contains("merge conflict") || msg.contains("merge"),
            "must hint at merge markers, got: {msg}"
        );
        assert!(
            msg.contains("regeneration") || msg.contains("regenerate"),
            "must mention regeneration, got: {msg}"
        );
        assert!(
            msg.contains("slow"),
            "must explain user-visible symptom, got: {msg}"
        );
    }

    /// GH #3522 — drive the formatter against a real corrupt YAML
    /// scenario: write a file with merge-conflict markers, call
    /// Lockfile::read, format the resulting error, and confirm the
    /// underlying error text comes through.
    #[tokio::test]
    async fn gh3522_corrupt_lockfile_produces_formatted_warn() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("jet-lock.yaml");
        std::fs::write(
            &path,
            "<<<<<<< HEAD\nlockfile_version: 1\n=======\nlockfile_version: 2\n>>>>>>> branch\n",
        )
        .expect("write corrupt lockfile");

        let err =
            Lockfile::read(&path).expect_err("corrupt YAML with merge markers must fail to parse");
        let msg = format_lockfile_read_warn(&path, &err);

        assert!(msg.contains("jet-lock.yaml"), "must echo path: {msg}");
        assert!(msg.contains("GH #3522"), "must tag issue: {msg}");
    }

    // ─── GH #3518: prefetch dispatcher JoinError silent swallow ──────────

    /// GH #3518 — JoinError from a panicked dispatcher task must be
    /// formatted with "panicked", the partial-store consequence, and
    /// the GH #3518 tag so the operator can chase later cold installs.
    #[tokio::test]
    async fn gh3518_panic_join_error_formats_as_panicked_with_store_caveat() {
        let handle: tokio::task::JoinHandle<()> = tokio::spawn(async {
            panic!("simulated dispatcher bug");
        });
        let join_err = handle.await.expect_err("task must have panicked");
        assert!(join_err.is_panic(), "join error must be panic");

        let msg = format_prefetch_join_warn(&join_err);
        assert!(
            msg.contains("panicked"),
            "must name failure kind, got: {msg}"
        );
        assert!(
            msg.contains("GH #3518"),
            "must include searchable issue tag, got: {msg}"
        );
        assert!(
            msg.contains("store"),
            "must explain on-disk consequence, got: {msg}"
        );
        assert!(
            !msg.contains("was cancelled"),
            "must not mislabel panic as cancellation, got: {msg}"
        );
    }

    /// GH #3518 — JoinError from a cancelled task must be formatted
    /// with "was cancelled" so the operator can tell at a glance
    /// whether they ^C-ed (case 2) or a real bug fired (case 1).
    #[tokio::test]
    async fn gh3518_cancelled_join_error_formats_as_cancelled() {
        // Spawn a long-running task, abort it, then await for the
        // cancellation-flavoured JoinError.
        let handle: tokio::task::JoinHandle<()> = tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        });
        handle.abort();
        let join_err = handle.await.expect_err("task must have been cancelled");
        assert!(join_err.is_cancelled(), "join error must be cancellation");

        let msg = format_prefetch_join_warn(&join_err);
        assert!(
            msg.contains("was cancelled"),
            "must name cancellation, got: {msg}"
        );
        assert!(
            msg.contains("GH #3518"),
            "must include searchable issue tag, got: {msg}"
        );
        assert!(
            !msg.contains("panicked"),
            "must not mislabel cancellation as panic, got: {msg}"
        );
    }

    #[test]
    fn test_package_json_parse() {
        let json = r#"{
            "name": "test-app",
            "version": "1.0.0",
            "dependencies": {
                "react": "^18.0.0"
            }
        }"#;

        let package: PackageJson = serde_json::from_str(json).unwrap();
        assert_eq!(package.name.as_deref(), Some("test-app"));
        assert_eq!(
            package.dependencies.get("react"),
            Some(&"^18.0.0".to_string())
        );
    }

    #[test]
    fn test_package_json_with_dev_deps() {
        let json = r#"{
            "name": "my-app",
            "version": "0.1.0",
            "dependencies": {},
            "devDependencies": {
                "typescript": "^5.0.0"
            }
        }"#;

        let package: PackageJson = serde_json::from_str(json).unwrap();
        assert_eq!(
            package.dev_dependencies.get("typescript"),
            Some(&"^5.0.0".to_string())
        );
    }

    /// Issue #1941 (R1): round-trip must preserve unknown top-level
    /// fields such as `private`, `type`, and `scripts`.
    #[test]
    fn test_package_json_roundtrip_preserves_unknown_fields() {
        let json = r#"{
  "name": "artifact-studio",
  "version": "0.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build"
  },
  "dependencies": {
    "react": "^18.0.0"
  },
  "devDependencies": {
    "@playwright/test": "^1.58.2"
  }
}"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let obj = value.as_object().unwrap();
        // All the unrelated root fields must round-trip via a typed
        // PackageJson view.
        let typed: PackageJson = serde_json::from_str(json).unwrap();
        assert_eq!(typed.extra.get("private"), obj.get("private"));
        assert_eq!(typed.extra.get("type"), obj.get("type"));
        assert_eq!(typed.extra.get("scripts"), obj.get("scripts"));
        // Re-serialise and confirm the unknown keys come back.
        let re = serde_json::to_string(&typed).unwrap();
        let re_value: serde_json::Value = serde_json::from_str(&re).unwrap();
        let re_obj = re_value.as_object().unwrap();
        assert!(re_obj.contains_key("private"));
        assert!(re_obj.contains_key("type"));
        assert!(re_obj.contains_key("scripts"));
    }

    /// Issue #1941 (R1, R2): `update_dep_entry` against an existing
    /// devDependency must update in place rather than insert a duplicate
    /// `dependencies` entry, and all other root fields must survive
    /// verbatim through the raw mutation path.
    #[test]
    fn test_update_dep_entry_preserves_unknown_fields_and_group() {
        let json = r#"{
  "name": "artifact-studio",
  "version": "0.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite"
  },
  "dependencies": {
    "react": "^18.0.0"
  },
  "devDependencies": {
    "@playwright/test": "^1.58.2"
  }
}"#;
        let mut doc: serde_json::Value = serde_json::from_str(json).unwrap();
        // Equivalent of `jet add @playwright/test` (no --dev).
        update_dep_entry(&mut doc, "@playwright/test", "^1.59.1", false);

        let obj = doc.as_object().unwrap();
        // R1: every unrelated root field still present.
        assert_eq!(obj.get("private").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(obj.get("type").and_then(|v| v.as_str()), Some("module"));
        assert!(obj.get("scripts").is_some());
        // Original `name` / `version` preserved.
        assert_eq!(
            obj.get("name").and_then(|v| v.as_str()),
            Some("artifact-studio")
        );
        assert_eq!(obj.get("version").and_then(|v| v.as_str()), Some("0.0.0"));

        // R2: package stayed in devDependencies, updated in place.
        let dev_deps = obj
            .get("devDependencies")
            .and_then(|v| v.as_object())
            .expect("devDependencies must remain an object");
        assert_eq!(
            dev_deps.get("@playwright/test").and_then(|v| v.as_str()),
            Some("^1.59.1"),
        );
        // And no duplicate entry leaked into `dependencies`.
        let deps = obj
            .get("dependencies")
            .and_then(|v| v.as_object())
            .expect("dependencies must remain an object");
        assert!(!deps.contains_key("@playwright/test"));
        // Top-level key order survives — `scripts` still appears before deps.
        let keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
        let scripts_idx = keys.iter().position(|k| *k == "scripts").unwrap();
        let deps_idx = keys.iter().position(|k| *k == "dependencies").unwrap();
        assert!(scripts_idx < deps_idx, "key order regressed: {:?}", keys);
    }

    /// Issue #1941 (R2 inverse): `--dev` on a package already living
    /// in `dependencies` should move it to `devDependencies` and not
    /// leave a duplicate behind.
    #[test]
    fn test_update_dep_entry_dev_flag_moves_existing_dep() {
        let json = r#"{
  "dependencies": {
    "react": "^18.0.0"
  }
}"#;
        let mut doc: serde_json::Value = serde_json::from_str(json).unwrap();
        update_dep_entry(&mut doc, "react", "^18.2.0", true);

        let obj = doc.as_object().unwrap();
        let deps = obj.get("dependencies").unwrap().as_object().unwrap();
        let dev_deps = obj.get("devDependencies").unwrap().as_object().unwrap();
        assert!(!deps.contains_key("react"));
        assert_eq!(
            dev_deps.get("react").and_then(|v| v.as_str()),
            Some("^18.2.0"),
        );
    }

    #[test]
    fn test_replace_existing_dep_entry_preserves_group_and_unknown_fields() {
        let json = r#"{
  "private": true,
  "scripts": {
    "build": "jet build"
  },
  "dependencies": {
    "react": "^18.0.0"
  },
  "devDependencies": {
    "vite": "^5.0.0"
  }
}"#;
        let mut doc: serde_json::Value = serde_json::from_str(json).unwrap();

        assert!(replace_existing_dep_entry(&mut doc, "vite", "^5.4.19"));
        assert!(!replace_existing_dep_entry(&mut doc, "missing", "^1.0.0"));

        let obj = doc.as_object().unwrap();
        assert_eq!(obj.get("private").and_then(|v| v.as_bool()), Some(true));
        assert!(obj.get("scripts").is_some());
        assert_eq!(
            obj.get("dependencies")
                .and_then(|v| v.as_object())
                .and_then(|deps| deps.get("react"))
                .and_then(|v| v.as_str()),
            Some("^18.0.0"),
        );
        assert_eq!(
            obj.get("devDependencies")
                .and_then(|v| v.as_object())
                .and_then(|deps| deps.get("vite"))
                .and_then(|v| v.as_str()),
            Some("^5.4.19"),
        );
    }

    #[test]
    fn test_remove_dep_entry_clears_all_dependency_groups() {
        let json = r#"{
  "dependencies": {
    "is-number": "7.0.0"
  },
  "devDependencies": {
    "vite": "^5.0.0"
  },
  "optionalDependencies": {
    "is-number": "7.0.0"
  }
}"#;
        let mut doc: serde_json::Value = serde_json::from_str(json).unwrap();

        assert!(remove_dep_entry(&mut doc, "is-number"));
        assert!(!remove_dep_entry(&mut doc, "is-number"));

        let obj = doc.as_object().unwrap();
        for group in ["dependencies", "optionalDependencies"] {
            assert!(
                !obj.get(group)
                    .and_then(|v| v.as_object())
                    .unwrap()
                    .contains_key("is-number"),
                "{group} still contains removed package"
            );
        }
        assert!(
            obj.get("devDependencies")
                .and_then(|v| v.as_object())
                .unwrap()
                .contains_key("vite"),
            "unrelated devDependency should remain"
        );
    }

    #[test]
    fn test_prune_node_modules_to_lockfile_removes_stale_packages_and_bins() {
        let dir = tempfile::tempdir().unwrap();
        let node_modules = dir.path().join("node_modules");
        std::fs::create_dir_all(node_modules.join(".bin")).unwrap();
        std::fs::create_dir_all(node_modules.join("react")).unwrap();
        std::fs::create_dir_all(node_modules.join("is-number")).unwrap();
        std::fs::create_dir_all(node_modules.join("@mui").join("material")).unwrap();
        std::fs::create_dir_all(node_modules.join("@types").join("stale")).unwrap();
        std::fs::write(node_modules.join(".bin").join("vite"), "#!/bin/sh\n").unwrap();
        std::fs::write(node_modules.join(".bin").join("stale-cli"), "#!/bin/sh\n").unwrap();

        let mut lockfile = Lockfile::new();
        lockfile.packages.insert(
            "/react@18.2.0".to_string(),
            LockfileEntry {
                version: "18.2.0".to_string(),
                resolution: Resolution {
                    tarball: "https://example.com/react.tgz".to_string(),
                    shasum: "react-sha".to_string(),
                    integrity: None,
                },
                workspace: false,
                local_path: None,
                dependencies: HashMap::new(),
                peer_dependencies: HashMap::new(),
                bin: HashMap::from([("vite".to_string(), "bin/vite.js".to_string())]),
                has_install_script: false,
                nested_in: None,
            },
        );
        lockfile.packages.insert(
            "/@mui/material@5.15.0".to_string(),
            LockfileEntry {
                version: "5.15.0".to_string(),
                resolution: Resolution {
                    tarball: "https://example.com/mui.tgz".to_string(),
                    shasum: "mui-sha".to_string(),
                    integrity: None,
                },
                workspace: false,
                local_path: None,
                dependencies: HashMap::new(),
                peer_dependencies: HashMap::new(),
                bin: HashMap::new(),
                has_install_script: false,
                nested_in: None,
            },
        );

        prune_node_modules_to_lockfile(&node_modules, &lockfile).unwrap();

        assert!(node_modules.join("react").exists());
        assert!(node_modules.join("@mui").join("material").exists());
        assert!(!node_modules.join("is-number").exists());
        assert!(!node_modules.join("@types").exists());
        assert!(node_modules.join(".bin").join("vite").exists());
        assert!(!node_modules.join(".bin").join("stale-cli").exists());
    }

    #[test]
    fn test_prune_third_party_node_modules_layout_removes_manager_residue() {
        let dir = tempfile::tempdir().unwrap();
        let node_modules = dir.path().join("node_modules");
        std::fs::create_dir_all(node_modules.join(".pnpm")).unwrap();
        std::fs::write(node_modules.join(".package-lock.json"), "{}\n").unwrap();
        std::fs::write(node_modules.join(".modules.yaml"), "layoutVersion: 5\n").unwrap();
        std::fs::write(node_modules.join("react"), "not touched\n").unwrap();

        prune_third_party_node_modules_layout(&node_modules).unwrap();

        assert!(!node_modules.join(".pnpm").exists());
        assert!(!node_modules.join(".package-lock.json").exists());
        assert!(!node_modules.join(".modules.yaml").exists());
        assert!(node_modules.join("react").exists());
    }

    #[test]
    fn test_lockfile_hash_matches_current_deps() {
        let mut deps = HashMap::from([("react".to_string(), "^18.2.0".to_string())]);
        let mut lockfile = Lockfile::new();
        lockfile.deps_hash = Some(Lockfile::compute_deps_hash(&deps));

        assert!(lockfile_hash_matches_current_deps(&lockfile, &deps));

        deps.insert("@playwright/test".to_string(), "^1.59.1".to_string());
        assert!(!lockfile_hash_matches_current_deps(&lockfile, &deps));
    }

    #[test]
    fn test_workspace_deps_hash_tracks_workspace_member_dep_specs() {
        let mut ws_mgr = workspace::WorkspaceManager {
            root: PathBuf::from("/repo"),
            config: workspace::WorkspaceConfig::default(),
            packages: vec![workspace::WorkspacePackage {
                name: "app".to_string(),
                version: "0.0.0".to_string(),
                path: PathBuf::from("packages/app"),
                dependencies: HashMap::from([
                    ("shared".to_string(), "workspace:*".to_string()),
                    ("react".to_string(), "^18.2.0".to_string()),
                ]),
                dev_dependencies: HashMap::from([("vite".to_string(), "^6.0.0".to_string())]),
                deps_on_workspace: Vec::new(),
            }],
        };

        let before = workspace_deps_hash(&ws_mgr);
        ws_mgr.packages[0]
            .dependencies
            .insert("react".to_string(), "^19.0.0".to_string());
        let after = workspace_deps_hash(&ws_mgr);

        assert_ne!(before, after);
    }

    fn lockfile_with_scoped_prop_types() -> Lockfile {
        let mut lockfile = Lockfile::new();
        lockfile.packages.insert(
            "/@types/prop-types@15.7.15".to_string(),
            LockfileEntry {
                version: "15.7.15".to_string(),
                resolution: Resolution {
                    tarball: "https://example.com/prop-types.tgz".to_string(),
                    shasum: "abc123".to_string(),
                    integrity: None,
                },
                workspace: false,
                local_path: None,
                dependencies: HashMap::new(),
                peer_dependencies: HashMap::new(),
                bin: HashMap::new(),
                has_install_script: false,
                nested_in: None,
            },
        );
        lockfile
    }

    #[test]
    fn test_lockfile_root_links_valid_accepts_scoped_package_link() {
        let dir = tempfile::tempdir().unwrap();
        let package_dir = dir
            .path()
            .join("node_modules")
            .join("@types")
            .join("prop-types");
        std::fs::create_dir_all(&package_dir).unwrap();
        std::fs::write(
            package_dir.join("package.json"),
            r#"{"name":"@types/prop-types","version":"15.7.15"}"#,
        )
        .unwrap();

        assert!(lockfile_root_links_valid(
            &dir.path().join("node_modules"),
            &lockfile_with_scoped_prop_types(),
        ));
    }

    #[test]
    fn test_lockfile_root_links_valid_detects_missing_scoped_link() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("node_modules").join("@types")).unwrap();

        #[cfg(unix)]
        std::os::unix::fs::symlink(
            dir.path()
                .join(".jet-store")
                .join("@types")
                .join("prop-types@15.7.15"),
            dir.path()
                .join("node_modules")
                .join("@types")
                .join("prop-types"),
        )
        .unwrap();

        assert!(!lockfile_root_links_valid(
            &dir.path().join("node_modules"),
            &lockfile_with_scoped_prop_types(),
        ));
    }

    #[test]
    fn test_detect_json_indent_two_space() {
        let json = "{\n  \"a\": 1\n}";
        assert_eq!(detect_json_indent(json), "  ");
    }

    #[test]
    fn test_detect_json_indent_four_space() {
        let json = "{\n    \"a\": 1\n}";
        assert_eq!(detect_json_indent(json), "    ");
    }

    #[test]
    fn test_detect_json_indent_tab() {
        let json = "{\n\t\"a\": 1\n}";
        assert_eq!(detect_json_indent(json), "\t");
    }

    /// Issue #1941: end-to-end write path on disk must preserve unknown
    /// top-level fields, key order, indent style, and the trailing
    /// newline.  This exercises `read_package_json_raw` +
    /// `update_dep_entry` + `write_package_json_raw` together.
    #[test]
    fn test_write_package_json_raw_preserves_unknown_fields_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let original = "{\n  \"name\": \"artifact-studio\",\n  \"version\": \"0.0.0\",\n  \"private\": true,\n  \"type\": \"module\",\n  \"scripts\": {\n    \"dev\": \"vite\"\n  },\n  \"dependencies\": {\n    \"react\": \"^18.0.0\"\n  },\n  \"devDependencies\": {\n    \"@playwright/test\": \"^1.58.2\"\n  }\n}\n";
        let pkg_json_path = dir.path().join("package.json");
        std::fs::write(&pkg_json_path, original).unwrap();

        let pm = PackageManager::new(dir.path().to_path_buf()).unwrap();
        let (mut doc, indent, trailing_newline) = pm.read_package_json_raw().unwrap();
        // Same scenario as `jet add @playwright/test` (no --dev).
        update_dep_entry(&mut doc, "@playwright/test", "^1.59.1", false);
        pm.write_package_json_raw(&doc, &indent, trailing_newline)
            .unwrap();

        let written = std::fs::read_to_string(&pkg_json_path).unwrap();

        // Trailing newline preserved.
        assert!(
            written.ends_with('\n'),
            "trailing newline missing: {:?}",
            written
        );
        // Indent preserved (two-space).
        assert!(
            written.contains("\n  \"name\""),
            "expected two-space indent, got: {}",
            written,
        );
        // Unknown root fields still present.
        assert!(written.contains("\"private\": true"));
        assert!(written.contains("\"type\": \"module\""));
        assert!(written.contains("\"dev\": \"vite\""));
        // No duplicate dependency entry.
        let parsed: serde_json::Value = serde_json::from_str(&written).unwrap();
        let deps = parsed.get("dependencies").unwrap().as_object().unwrap();
        let dev_deps = parsed.get("devDependencies").unwrap().as_object().unwrap();
        assert!(!deps.contains_key("@playwright/test"));
        assert_eq!(
            dev_deps.get("@playwright/test").and_then(|v| v.as_str()),
            Some("^1.59.1"),
        );
    }

    #[test]
    fn test_parse_spec_bare_name() {
        let (name, range) = parse_package_spec("react").unwrap();
        assert_eq!(name, "react");
        assert_eq!(range, None);
    }

    #[test]
    fn test_parse_spec_scoped_name() {
        let (name, range) = parse_package_spec("@mui/material").unwrap();
        assert_eq!(name, "@mui/material");
        assert_eq!(range, None);
    }

    #[test]
    fn test_parse_spec_with_caret_range() {
        let (name, range) = parse_package_spec("react@^18").unwrap();
        assert_eq!(name, "react");
        assert_eq!(range.as_deref(), Some("^18"));
    }

    #[test]
    fn test_parse_spec_with_exact_version() {
        let (name, range) = parse_package_spec("lodash@4.17.21").unwrap();
        assert_eq!(name, "lodash");
        assert_eq!(range.as_deref(), Some("4.17.21"));
    }

    #[test]
    fn test_parse_spec_scoped_with_range() {
        let (name, range) = parse_package_spec("@emotion/react@^11.11.0").unwrap();
        assert_eq!(name, "@emotion/react");
        assert_eq!(range.as_deref(), Some("^11.11.0"));
    }

    #[test]
    fn test_parse_spec_rejects_empty() {
        assert!(parse_package_spec("").is_err());
        assert!(parse_package_spec("   ").is_err());
    }

    #[test]
    fn test_parse_spec_rejects_trailing_at() {
        assert!(parse_package_spec("react@").is_err());
    }

    #[test]
    fn test_jet_add_cli_accepts_single_package() {
        let cmd = crate::cli::command();
        let m = cmd
            .try_get_matches_from(["jet", "add", "react"])
            .expect("single-package form should parse");
        let (sub, sm) = m.subcommand().unwrap();
        assert_eq!(sub, "add");
        let pkgs: Vec<&String> = sm.get_many::<String>("packages").unwrap().collect();
        assert_eq!(pkgs, vec![&"react".to_string()]);
        assert!(!sm.get_flag("dev"));
    }

    #[test]
    fn test_jet_add_cli_accepts_multiple_packages() {
        // Reproducer for issue #1907: `jet add foo bar baz` must accept
        // all three positional package specs in a single invocation.
        let cmd = crate::cli::command();
        let m = cmd
            .try_get_matches_from(["jet", "add", "@mui/material", "@emotion/react", "react@^18"])
            .expect("multi-package form should parse (issue #1907)");
        let (sub, sm) = m.subcommand().unwrap();
        assert_eq!(sub, "add");
        let pkgs: Vec<&str> = sm
            .get_many::<String>("packages")
            .unwrap()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(pkgs, vec!["@mui/material", "@emotion/react", "react@^18"]);
        assert!(!sm.get_flag("dev"));
    }

    #[test]
    fn test_jet_add_cli_multi_with_dev_flag() {
        let cmd = crate::cli::command();
        let m = cmd
            .try_get_matches_from(["jet", "add", "vitest", "@types/node", "--dev"])
            .expect("multi-package + --dev should parse");
        let (_, sm) = m.subcommand().unwrap();
        let pkgs: Vec<&str> = sm
            .get_many::<String>("packages")
            .unwrap()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(pkgs, vec!["vitest", "@types/node"]);
        assert!(sm.get_flag("dev"));
    }

    #[test]
    fn test_jet_add_cli_requires_at_least_one_package() {
        let cmd = crate::cli::command();
        assert!(
            cmd.try_get_matches_from(["jet", "add"]).is_err(),
            "bare `jet add` must error: at least one package required"
        );
    }

    /// GH #3211 — happy path: when the target directory exists and the
    /// marker write succeeds, the function must leave a file containing
    /// exactly the lockfile hash. Pins the on-disk contract that
    /// `read_to_string(marker).trim() == lf_hash` is what the
    /// ultra-fast-path comparison at install-time relies on.
    #[test]
    fn write_jet_marker_writes_hash_to_existing_dir() {
        let dir = tempfile::tempdir().unwrap();
        let node_modules = dir.path().join("node_modules");
        std::fs::create_dir_all(&node_modules).unwrap();
        let nm_marker = node_modules.join(".jet-marker");

        write_jet_marker(&node_modules, &nm_marker, "deadbeef");

        let stored = std::fs::read_to_string(&nm_marker).unwrap();
        assert_eq!(stored.trim(), "deadbeef");
    }

    /// GH #3211 — when the parent does not yet exist, the helper must
    /// create it and still write the marker (matches the prior contract
    /// of `let _ = create_dir_all(...)` + `let _ = write(...)`).
    #[test]
    fn write_jet_marker_creates_missing_node_modules_dir() {
        let dir = tempfile::tempdir().unwrap();
        let node_modules = dir.path().join("does-not-exist-yet");
        let nm_marker = node_modules.join(".jet-marker");

        write_jet_marker(&node_modules, &nm_marker, "cafef00d");

        assert!(node_modules.exists(), "node_modules should be created");
        assert_eq!(
            std::fs::read_to_string(&nm_marker).unwrap().trim(),
            "cafef00d"
        );
    }

    /// GH #3211 — when both create_dir_all and write fail (e.g. the
    /// parent of node_modules is read-only), the helper must NOT panic
    /// and must NOT propagate the error. The install itself already
    /// succeeded; the only contract is that the helper logs a warn and
    /// returns. We can't easily assert the warn from a unit test, but
    /// we can pin that the function returns cleanly when the target
    /// path is unwritable, exercising the failure branch.
    #[cfg(unix)]
    #[test]
    fn write_jet_marker_does_not_panic_when_parent_is_readonly() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        // Make the tempdir read-only so create_dir_all on a child fails.
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o500)).unwrap();

        let node_modules = dir.path().join("blocked").join("node_modules");
        let nm_marker = node_modules.join(".jet-marker");

        // Must not panic; warn is logged internally.
        write_jet_marker(&node_modules, &nm_marker, "irrelevant");

        // Restore so tempdir can clean up.
        let _ = std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755));
    }

    /// GH #3492 — build a minimal `.tgz` containing a single
    /// `package/package.json` so `install_package` succeeds. Used by the
    /// prefetch surfacing tests below.
    fn build_minimal_tarball(name: &str, version: &str) -> Vec<u8> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let pkg_json = format!("{{ \"name\": \"{}\", \"version\": \"{}\" }}", name, version);
        let mut builder = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_size(pkg_json.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "package/package.json", pkg_json.as_bytes())
            .unwrap();
        let tar_bytes = builder.into_inner().unwrap();
        let mut gz = GzEncoder::new(Vec::new(), Compression::fast());
        gz.write_all(&tar_bytes).unwrap();
        gz.finish().unwrap()
    }

    /// GH #3492 — happy path: a successful prefetch install lands the
    /// package in the store. Pins the contract that `prefetch_install_one`
    /// actually installs (the prior implementation's `let _ = ...` was an
    /// observability bug, not a behavior bug — this test catches a
    /// regression that turns the helper into a no-op).
    #[test]
    fn gh3492_prefetch_install_one_success_lands_in_store() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let store = store::StoreManager::new(dir.path().to_path_buf()).unwrap();

        let tarball = build_minimal_tarball("acme-pkg", "1.0.0");
        // 40-hex (SHA-1 length) — `verify_shasum` skips full verification
        // for SHA-1 shasums and just records them.
        let shasum = "0".repeat(40);

        super::prefetch_install_one(&store, "acme-pkg", "1.0.0", &tarball, &shasum);

        assert!(
            store.has_package("acme-pkg", "1.0.0", &shasum),
            "successful prefetch install must land the package in the store"
        );
    }

    /// GH #3492 — failure surfacing: a shasum mismatch must NOT install,
    /// must NOT panic, and must leave the store entry absent. The warn
    /// emission is asserted indirectly via the package-absent observable
    /// (a regression that re-introduces `let _ = ...` would silently
    /// "succeed" — the install would still run, partially write files,
    /// fail mid-way, and leave a broken/empty package dir). Here we use
    /// a SHA-256-length shasum so `verify_shasum` actually runs and
    /// rejects pre-extraction.
    #[test]
    fn gh3492_prefetch_install_one_shasum_mismatch_does_not_install() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let store = store::StoreManager::new(dir.path().to_path_buf()).unwrap();

        let tarball = build_minimal_tarball("acme-pkg", "1.0.0");
        // 64-hex (SHA-256 length) all-zeros — guaranteed to mismatch the
        // real SHA-256 of `tarball`. `verify_shasum` bails before any
        // filesystem writes.
        let bogus = "0".repeat(64);

        super::prefetch_install_one(&store, "acme-pkg", "1.0.0", &tarball, &bogus);

        assert!(
            !store.has_package("acme-pkg", "1.0.0", &bogus),
            "shasum-mismatched prefetch must NOT register the package in the store"
        );
        // The package directory must not exist either — install_package
        // bails before create_dir_all.
        let pkg_dir = store.get_package_path("acme-pkg", "1.0.0");
        assert!(
            !pkg_dir.exists(),
            "shasum-mismatched prefetch must leave the package dir absent"
        );
    }

    // ─── GH #3550: HOME-unset silent relocation of store + cache ─────────

    /// GH #3550 — the HOME-fallback warn must name the env var, tag the
    /// issue, and explain the cwd-relocation consequence so the user can
    /// match a "why are my installs always cold" symptom to this line.
    #[test]
    fn gh3550_pkg_manager_home_warn_names_var_issue_and_consequence() {
        let msg = format_pkg_manager_home_warn(&std::env::VarError::NotPresent);

        assert!(
            msg.contains("GH #3550"),
            "must include searchable issue tag, got: {msg}"
        );
        assert!(
            msg.contains("HOME"),
            "must name the HOME env var, got: {msg}"
        );
        assert!(
            msg.contains("current working directory"),
            "must explain the cwd-fallback consequence, got: {msg}"
        );
        assert!(
            msg.contains(".jet-store") || msg.contains("global package store"),
            "must name the affected jet store, got: {msg}"
        );
    }

    /// GH #3550 — the warning must name BOTH user-facing knobs (HOME for
    /// the store, XDG_CACHE_HOME for the metadata cache) so the dev can
    /// pick the minimal fix for their env.
    #[test]
    fn gh3550_pkg_manager_home_warn_names_both_env_knobs() {
        let msg = format_pkg_manager_home_warn(&std::env::VarError::NotPresent);

        assert!(msg.contains("HOME"), "must name HOME, got: {msg}");
        assert!(
            msg.contains("XDG_CACHE_HOME"),
            "must name XDG_CACHE_HOME as the cache-only knob, got: {msg}"
        );
    }

    /// GH #3550 — branching contract: when `HOME` is set and non-empty,
    /// return its value verbatim. When unset, return `PathBuf::from(".")`.
    /// Drives the pure helper so the test does not mutate process env.
    #[test]
    fn gh3550_pkg_manager_home_from_result_branches_on_env() {
        // Set + non-empty → verbatim.
        assert_eq!(
            pkg_manager_home_from_result(Ok("/home/alice".to_string())),
            PathBuf::from("/home/alice")
        );

        // Set but empty → fallback (some shells export HOME="").
        assert_eq!(
            pkg_manager_home_from_result(Ok(String::new())),
            PathBuf::from(".")
        );

        // NotPresent → fallback.
        assert_eq!(
            pkg_manager_home_from_result(Err(std::env::VarError::NotPresent)),
            PathBuf::from(".")
        );

        // NotUnicode → fallback (e.g. non-UTF8 HOME on broken environments).
        assert_eq!(
            pkg_manager_home_from_result(Err(std::env::VarError::NotUnicode(
                std::ffi::OsString::from("/tmp")
            ))),
            PathBuf::from(".")
        );
    }

    // ─── GH #3562: package.json read/write errors drop the file path ─────

    /// GH #3562 — each package.json context helper must include the
    /// issue tag and the offending path verbatim so that a stack of
    /// `Failed to read /proj/foo/package.json` lines in a monorepo
    /// CI log lands on the right workspace member.
    #[test]
    fn gh3562_package_json_ctx_strings_name_tag_and_path() {
        let p = std::path::Path::new("/proj/web/package.json");
        let read = package_json_read_io_ctx(p);
        let parse = package_json_parse_ctx(p);
        let serialize = package_json_serialize_ctx(p);
        let write = package_json_write_io_ctx(p);
        let utf8 = package_json_utf8_ctx(p);

        for (label, msg) in [
            ("read", &read),
            ("parse", &parse),
            ("serialize", &serialize),
            ("write", &write),
            ("utf8", &utf8),
        ] {
            assert!(
                msg.contains("GH #3562"),
                "{label} ctx must include searchable issue tag, got: {msg}"
            );
            assert!(
                msg.contains("/proj/web/package.json"),
                "{label} ctx must name the offending path, got: {msg}"
            );
        }
    }

    /// GH #3562 — read vs parse vs serialize vs write vs utf8 must be
    /// pairwise distinct so the dev can tell from the message which
    /// step blew up (filesystem vs syntax vs encoding).
    #[test]
    fn gh3562_package_json_ctx_strings_are_pairwise_distinct() {
        let p = std::path::Path::new("/proj/web/package.json");
        let messages = [
            package_json_read_io_ctx(p),
            package_json_parse_ctx(p),
            package_json_serialize_ctx(p),
            package_json_write_io_ctx(p),
            package_json_utf8_ctx(p),
        ];
        for (i, a) in messages.iter().enumerate() {
            for (j, b) in messages.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        a, b,
                        "ctx strings must be distinct (index {i} vs {j}), got duplicate: {a}"
                    );
                }
            }
        }
    }

    /// GH #3562 — verb-content contract: each helper must name the
    /// step the dev is actually looking at (read / parse / serialize /
    /// write / encoding) so the message is not a generic
    /// "package.json failed".
    #[test]
    fn gh3562_package_json_ctx_strings_name_their_step() {
        let p = std::path::Path::new("/proj/web/package.json");
        assert!(
            package_json_read_io_ctx(p).contains("reading"),
            "read ctx must say `reading`, got: {}",
            package_json_read_io_ctx(p)
        );
        assert!(
            package_json_parse_ctx(p).contains("parsing"),
            "parse ctx must say `parsing`, got: {}",
            package_json_parse_ctx(p)
        );
        assert!(
            package_json_serialize_ctx(p).contains("serializing"),
            "serialize ctx must say `serializing`, got: {}",
            package_json_serialize_ctx(p)
        );
        assert!(
            package_json_write_io_ctx(p).contains("writing"),
            "write ctx must say `writing`, got: {}",
            package_json_write_io_ctx(p)
        );
        assert!(
            package_json_utf8_ctx(p).contains("UTF-8")
                || package_json_utf8_ctx(p).contains("encoding"),
            "utf8 ctx must name the encoding step, got: {}",
            package_json_utf8_ctx(p)
        );
    }

    /// GH #3562 — wiring contract: `read_package_json` must
    /// `with_context` both the read and the parse step. Drive the
    /// helpers themselves so the test does not need a real
    /// `PackageManager` instance; if a caller drops the
    /// `with_context` call, the parse tag below would no longer
    /// appear in the surfaced error chain.
    #[test]
    fn gh3562_package_json_parse_ctx_appears_in_with_context_chain() {
        let p = std::path::Path::new("/proj/web/package.json");
        let raw: anyhow::Result<()> =
            (|| -> anyhow::Result<()> { Err(anyhow::anyhow!("bad json")) })()
                .with_context(|| package_json_parse_ctx(p));
        let err = raw.expect_err("synthetic parse failure");
        let chain = format!("{err:#}");
        assert!(
            chain.contains("GH #3562"),
            "chained error must include issue tag, got: {chain}"
        );
        assert!(
            chain.contains("/proj/web/package.json"),
            "chained error must name the file, got: {chain}"
        );
        assert!(
            chain.contains("bad json"),
            "chained error must preserve the underlying cause, got: {chain}"
        );
    }

    // ─── GH #3625: is_ci_env NotUnicode silent miss ─────────────────────

    /// GH #3625 — every var unset → no CI detected, no warning.
    #[test]
    fn gh3625_safe_is_ci_env_all_absent_returns_false_no_warn() {
        let (is_ci, warn) = safe_is_ci_env(&["CI", "GITHUB_ACTIONS"], |_| {
            Err(std::env::VarError::NotPresent)
        });
        assert!(!is_ci, "no var set => not CI");
        assert!(warn.is_none(), "no warn when all absent, got: {:?}", warn);
    }

    /// GH #3625 — Ok value sets is_ci=true with no warning (normal CI path).
    #[test]
    fn gh3625_safe_is_ci_env_present_utf8_sets_true_no_warn() {
        let (is_ci, warn) = safe_is_ci_env(&["CI", "GITHUB_ACTIONS"], |name| {
            if name == "CI" {
                Ok("1".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert!(is_ci, "Ok value => CI detected");
        assert!(warn.is_none(), "no warn on Ok path, got: {:?}", warn);
    }

    /// GH #3625 — `NotUnicode` is the bug-of-record: previously
    /// collapsed to `false` via `.is_ok()`. Helper MUST return
    /// `is_ci=true` AND surface a warning naming the offending var.
    #[test]
    fn gh3625_safe_is_ci_env_notunicode_sets_true_and_warns() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let bad = OsString::from_vec(vec![0xFF, 0xFE]);
        let (is_ci, warn) = safe_is_ci_env(&["CI", "GITHUB_ACTIONS"], |name| {
            if name == "CI" {
                Err(std::env::VarError::NotUnicode(bad.clone()))
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert!(is_ci, "NotUnicode MUST count as a positive CI signal");
        let msg = warn.expect("NotUnicode must produce a warn message");
        assert!(msg.contains("GH #3625"), "msg: {msg}");
        assert!(msg.contains("CI"), "msg must name the var: {msg}");
    }

    /// GH #3625 — NotUnicode on a single var still detects CI even
    /// when no other var is set (regression guard for the original
    /// `.is_ok() || .is_ok() || …` collapse).
    #[test]
    fn gh3625_safe_is_ci_env_single_notunicode_alone_detects_ci() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let bad = OsString::from_vec(vec![0xC3, 0x28]);
        let (is_ci, warn) = safe_is_ci_env(&["JENKINS_URL"], |_| {
            Err(std::env::VarError::NotUnicode(bad.clone()))
        });
        assert!(is_ci, "single NotUnicode var must NOT collapse to false");
        let msg = warn.expect("warn must fire");
        assert!(msg.contains("JENKINS_URL"), "msg: {msg}");
    }

    /// GH #3625 — multiple NotUnicode vars must each appear in the
    /// composite warn (so the operator can fix all of them, not just
    /// the first one logged).
    #[test]
    fn gh3625_safe_is_ci_env_multiple_notunicode_concatenates_warns() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let bad = OsString::from_vec(vec![0xFF]);
        let (is_ci, warn) = safe_is_ci_env(&["CI", "GITHUB_ACTIONS"], |_| {
            Err(std::env::VarError::NotUnicode(bad.clone()))
        });
        assert!(is_ci);
        let msg = warn.expect("warn must fire");
        assert!(msg.contains("CI"), "msg must include CI: {msg}");
        assert!(
            msg.contains("GITHUB_ACTIONS"),
            "msg must include GITHUB_ACTIONS: {msg}"
        );
    }

    /// GH #3625 — formatter pins issue tag, var name, and consequence.
    #[test]
    fn gh3625_format_safe_is_ci_env_warn_shape() {
        let err = std::env::VarError::NotPresent;
        let msg = format_safe_is_ci_env_warn("GITLAB_CI", &err);
        assert!(msg.contains("GH #3625"), "msg: {msg}");
        assert!(msg.contains("GITLAB_CI"), "msg: {msg}");
        assert!(
            msg.contains("frozen") || msg.contains("frozen-lockfile"),
            "msg must explain consequence: {msg}"
        );
    }

    /// GH #3625 — CI_ENV_VARS constant pins the production set so
    /// renaming one of the four (e.g. dropping GitHub Actions) trips
    /// a test instead of silently turning off detection.
    #[test]
    fn gh3625_ci_env_vars_constant_pins_production_set() {
        assert_eq!(
            CI_ENV_VARS,
            &["CI", "GITHUB_ACTIONS", "GITLAB_CI", "JENKINS_URL"],
            "removing a CI var here disables its detection — be deliberate"
        );
    }
}
// CODEGEN-END

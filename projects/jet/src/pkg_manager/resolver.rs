// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
// CODEGEN-BEGIN
use anyhow::{Context, Result};
use dashmap::DashMap;
use semver::{Version, VersionReq};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Notify};

use super::registry::RegistryClient;

/// GH #3516 — speculative pre-warm of a single dep. Previously the body
/// was `let _ = reg.get_package_metadata(&name).await;`, which silently
/// swallowed every registry failure (HTTP 5xx / 429, DNS / TLS, malformed
/// manifest, integrity error). When the pre-warm failed for all 7
/// SPECULATIVE_DEPS the dev just saw BFS resolution be slow with zero
/// breadcrumb in the log.
///
/// The wrapper logs at warn for Err and debug for Ok. Happy path is debug
/// because this fires on every resolve (7 of them) and would otherwise
/// flood logs.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) async fn prewarm_speculative_dep(reg: &RegistryClient, name: &str) {
    match reg.get_package_metadata(name).await {
        Ok(_) => {
            tracing::debug!(
                target: "jet::pkg_manager::prewarm",
                package = %name,
                "Speculative pre-warm hit"
            );
        }
        Err(err) => {
            tracing::warn!(
                target: "jet::pkg_manager::prewarm",
                package = %name,
                error = %err,
                "{}",
                format_speculative_prewarm_warn(name, &err)
            );
        }
    }
}

/// GH #3516 — message shape for the speculative pre-warm Err arm.
/// Extracted so the wording (which is what the dev greps for) is
/// pinned by a unit test without provoking a real registry failure.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_speculative_prewarm_warn(name: &str, err: &anyhow::Error) -> String {
    format!(
        "GH #3516 speculative pre-warm of '{name}' failed: {err}; \
         BFS will fetch on demand and the install may appear slower than \
         a warm run. Check registry reachability / .npmrc proxy / auth."
    )
}

/// Common transitive deps speculatively prefetched on the first BFS level.
///
/// These packages appear as transitive deps in the vast majority of JS
/// projects. By pre-warming the registry cache at BFS level 0 (alongside
/// direct deps), subsequent BFS levels get instant in-memory hits instead
/// of round-trip fetches.
const SPECULATIVE_DEPS: &[&str] = &[
    "react",
    "react-dom",
    "scheduler",
    "lodash",
    "tslib",
    "object-assign",
    "prop-types",
];

/// A fully resolved package with all metadata needed for installation.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: String,
    pub tarball_url: String,
    pub shasum: String,
    pub integrity: Option<String>,
    pub dependencies: HashMap<String, String>,
    pub peer_dependencies: HashMap<String, String>,
    /// Resolved bin entries: command name → relative path inside package.
    pub bin: HashMap<String, String>,
    /// Whether this package has a postinstall (or preinstall/install) script.
    pub has_install_script: bool,
    /// If set, this package should be nested inside the given parent's
    /// node_modules instead of top-level (for version conflicts).
    pub nested_in: Option<String>,
    /// True when this package is a local workspace package (workspace: protocol).
    /// Propagates to LockfileEntry.workspace.
    pub workspace: bool,
    /// Relative path from workspace root to workspace package directory.
    /// Only present when workspace == true. Propagates to LockfileEntry.local_path.
    pub local_path: Option<String>,
}

/// Dependency resolver with transitive resolution and circular dependency
/// detection. Uses greedy strategy: pick highest compatible version.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub struct DependencyResolver {}

/// Shared state threaded through all concurrent resolution tasks.
#[derive(Clone)]
struct ResolverState {
    /// Packages that have been fully resolved.
    resolved: Arc<DashMap<String, ResolvedPackage>>,
    /// Packages that have been claimed for resolution (prevents duplicate work).
    visited: Arc<DashMap<String, ()>>,
    /// Number of tasks currently in-flight (spawned but not yet finished).
    pending: Arc<AtomicUsize>,
    /// Wakes the main task when `pending` reaches 0 or a fatal error occurs.
    notify: Arc<Notify>,
    /// Shared registry client (all clones share the same L1 cache via Arc).
    registry: RegistryClient,
    /// Override map: package name → forced version range.
    overrides: Arc<HashMap<String, String>>,
    /// Optional channel for notifying a background downloader of resolved pkgs.
    prefetch_tx: Arc<Option<mpsc::UnboundedSender<ResolvedPackage>>>,
    /// Stores the first fatal error produced by a direct-dep task.
    error_slot: Arc<std::sync::Mutex<Option<anyhow::Error>>>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl DependencyResolver {
    pub fn new() -> Self {
        Self {}
    }

    /// Resolve all dependencies without overlapping prefetch.
    /// Thin wrapper around `resolve_with_prefetch`.
    pub async fn resolve(
        &self,
        deps: &HashMap<String, String>,
        registry: &RegistryClient,
        overrides: &HashMap<String, String>,
    ) -> Result<HashMap<String, ResolvedPackage>> {
        self.resolve_with_prefetch(deps, registry, overrides, None)
            .await
    }

    /// Resolve all dependencies (direct + transitive) using a stream-based
    /// approach. Each resolved package **immediately** spawns `tokio::spawn`
    /// tasks for its transitive dependencies, without waiting for sibling
    /// packages at the same logical level to finish.
    ///
    /// Shared state:
    /// - `DashMap<String, ResolvedPackage>` — resolved packages (lock-free).
    /// - `DashMap<String, ()>` — visited set; atomically claims a package for
    ///   resolution, preventing duplicate work across concurrent tasks.
    /// - `Arc<AtomicUsize>` — pending task counter; incremented before each
    ///   `tokio::spawn`, decremented when the task finishes. Reaching 0 signals
    ///   completion.
    ///
    /// When `prefetch_tx` is `Some`, each `ResolvedPackage` is sent on the
    /// channel immediately after version selection so that a background
    /// consumer can start tarball downloads while resolution continues,
    /// overlapping network I/O.
    pub async fn resolve_with_prefetch(
        &self,
        deps: &HashMap<String, String>,
        registry: &RegistryClient,
        overrides: &HashMap<String, String>,
        prefetch_tx: Option<mpsc::UnboundedSender<ResolvedPackage>>,
    ) -> Result<HashMap<String, ResolvedPackage>> {
        let notify = Arc::new(Notify::new());
        let pending = Arc::new(AtomicUsize::new(0));

        let state = ResolverState {
            resolved: Arc::new(DashMap::new()),
            visited: Arc::new(DashMap::new()),
            pending: Arc::clone(&pending),
            notify: Arc::clone(&notify),
            registry: registry.clone(),
            overrides: Arc::new(overrides.clone()),
            prefetch_tx: Arc::new(prefetch_tx),
            error_slot: Arc::new(std::sync::Mutex::new(None)),
        };

        // Speculative pre-warm: fire off cache-warming fetches for common
        // transitive deps in parallel with direct dep resolution, so that
        // when BFS naturally reaches them they hit the in-memory cache.
        for &spec_name in SPECULATIVE_DEPS {
            let reg = state.registry.clone();
            let name = spec_name.to_string();
            tokio::spawn(async move {
                prewarm_speculative_dep(&reg, &name).await;
            });
        }

        // Collect seeds (direct deps) and increment pending atomically before
        // spawning any task, so that the wait loop never sees pending == 0
        // while tasks are still being set up.
        let seeds: Vec<(String, String)> = deps.iter().map(|(n, r)| resolve_alias(n, r)).collect();

        if seeds.is_empty() {
            return Ok(HashMap::new());
        }

        pending.fetch_add(seeds.len(), Ordering::SeqCst);
        for (real_name, real_range) in seeds {
            let task_state = state.clone();
            tokio::spawn(async move {
                stream_resolve_package(
                    task_state,
                    real_name,
                    real_range,
                    vec!["(root)".to_string()],
                )
                .await;
            });
        }

        // Wait until all tasks complete (pending == 0) or a fatal error is set.
        // The `notified()` future is created *before* checking the condition so
        // that any `notify_one()` fired between the check and the `.await` is
        // not lost.
        loop {
            let notified = notify.notified();

            if let Some(err) = state.error_slot.lock().unwrap().take() {
                return Err(err);
            }

            if pending.load(Ordering::SeqCst) == 0 {
                break;
            }

            notified.await;
        }

        // Final check: a fatal error might have arrived concurrently with
        // pending reaching 0.
        if let Some(err) = state.error_slot.lock().unwrap().take() {
            return Err(err);
        }

        let result = state
            .resolved
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect();

        Ok(result)
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Decrement the pending counter and wake the main task if it reaches zero.
#[inline]
fn decrement_pending(pending: &Arc<AtomicUsize>, notify: &Arc<Notify>) {
    let prev = pending.fetch_sub(1, Ordering::SeqCst);
    if prev == 1 {
        notify.notify_one();
    }
}

/// Atomically claim a package for resolution using DashMap's entry API.
///
/// Returns `true` if this call claimed the package (the caller should proceed
/// with resolution). Returns `false` if another task already claimed it.
///
/// This is a **sync** function so that no DashMap shard guard ever crosses an
/// `await` point — a guard held inside an `async fn` would make the resulting
/// future non-`Send` and break `tokio::spawn`.
fn try_claim_package(visited: &DashMap<String, ()>, name: &str) -> bool {
    match visited.entry(name.to_string()) {
        dashmap::Entry::Occupied(_) => false,
        dashmap::Entry::Vacant(e) => {
            e.insert(());
            true
        }
    }
}

/// Async task that resolves a single package and immediately spawns tasks for
/// its transitive dependencies.
///
/// Calls `try_claim_package` (sync) to atomically claim the package before any
/// `await` — only one concurrent task will process a given package name; others
/// detect the existing entry and return after optionally emitting a
/// version-conflict warning.
fn stream_resolve_package(
    state: ResolverState,
    name: String,
    range_str: String,
    dep_chain: Vec<String>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>> {
    Box::pin(async move {
        // Apply override: forced version range takes precedence.
        let range_str = state.overrides.get(&name).cloned().unwrap_or(range_str);

        // Atomically claim this package (sync — no guard crosses an await).
        // If already claimed, emit a version-conflict warning and bail out.
        if !try_claim_package(&state.visited, &name) {
            // Another task is processing (or has processed) this package.
            // Emit a version-conflict warning if the resolved version does not
            // satisfy our requested range.
            if let Some(existing) = state.resolved.get(&name) {
                if let Ok(reqs) = parse_all_version_ranges(&range_str) {
                    if let Ok(ev) = Version::parse(&existing.version) {
                        let satisfied = reqs.iter().any(|r| r.matches(&ev));
                        if !satisfied {
                            tracing::warn!(
                                "Version conflict '{}': \
                             using {}@{}, wanted '{}' by {}",
                                name,
                                name,
                                existing.version,
                                range_str,
                                dep_chain.last().unwrap_or(&"(root)".to_string())
                            );
                        }
                    }
                }
            }
            decrement_pending(&state.pending, &state.notify);
            return;
        }

        // Fetch registry metadata (hits in-memory cache if already warm).
        let metadata = match state.registry.get_package_metadata(&name).await {
            Ok(m) => m,
            Err(e) => {
                if dep_chain.len() > 1 {
                    tracing::warn!("Skipping {}: {} (transitive)", name, e);
                } else {
                    let mut slot = state.error_slot.lock().unwrap();
                    if slot.is_none() {
                        *slot = Some(anyhow::anyhow!(
                            "Failed to fetch metadata for '{}': {}",
                            name,
                            e
                        ));
                        state.notify.notify_one();
                    }
                }
                decrement_pending(&state.pending, &state.notify);
                return;
            }
        };

        // Parse version range (with OR support).
        let reqs = match parse_all_version_ranges(&range_str) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Bad version range for {}: {}", name, e);
                decrement_pending(&state.pending, &state.notify);
                return;
            }
        };

        // Select the best matching version.
        let best_version = match find_best_version(&metadata.versions, &reqs, &name) {
            Ok(v) => v,
            Err(e) => {
                if dep_chain.len() > 1 {
                    tracing::warn!("Skipping {}: {} (transitive)", name, e);
                } else {
                    let mut slot = state.error_slot.lock().unwrap();
                    if slot.is_none() {
                        *slot = Some(e);
                        state.notify.notify_one();
                    }
                }
                decrement_pending(&state.pending, &state.notify);
                return;
            }
        };

        // Retrieve version metadata (clone to avoid lifetime issues in async).
        let version_meta = match metadata.versions.get(&best_version).cloned() {
            Some(v) => v,
            None => {
                decrement_pending(&state.pending, &state.notify);
                return;
            }
        };

        // Skip platform-incompatible optional packages.
        if should_skip_optional(&version_meta) {
            tracing::debug!("Skipping {} (platform mismatch)", name);
            decrement_pending(&state.pending, &state.notify);
            return;
        }

        let pkg_deps = version_meta.dependencies.clone().unwrap_or_default();
        let peer_deps = version_meta.peer_dependencies.clone().unwrap_or_default();
        let optional_deps = version_meta
            .optional_dependencies
            .clone()
            .unwrap_or_default();

        let bin = resolve_bin_field(&name, &version_meta.bin);
        let has_install_script = version_meta
            .scripts
            .as_ref()
            .map(|s| {
                s.contains_key("preinstall")
                    || s.contains_key("install")
                    || s.contains_key("postinstall")
            })
            .unwrap_or(false);

        let resolved_pkg = ResolvedPackage {
            name: name.clone(),
            version: best_version.clone(),
            tarball_url: version_meta.dist.tarball.clone(),
            shasum: version_meta.dist.shasum.clone(),
            integrity: version_meta.dist.integrity.clone(),
            dependencies: pkg_deps.clone(),
            peer_dependencies: peer_deps.clone(),
            bin,
            has_install_script,
            nested_in: None,
            workspace: false,
            local_path: None,
        };

        // Notify background downloader immediately (overlapping tarball fetch).
        if let Some(tx) = state.prefetch_tx.as_ref() {
            let _ = tx.send(resolved_pkg.clone());
        }

        state.resolved.insert(name.clone(), resolved_pkg);

        // Build dep chain for children.
        let mut child_chain = dep_chain;
        child_chain.push(format!("{}@{}", name, best_version));

        // Collect child tasks:
        // - Regular deps: always spawn (visited check inside the task).
        // - Peer/optional deps: skip if already resolved (same as BFS behavior).
        let mut children: Vec<(String, String)> = pkg_deps
            .iter()
            .map(|(dn, dr)| resolve_alias(dn, dr))
            .collect();

        for (dn, dr) in &peer_deps {
            let (rn, rr) = resolve_alias(dn, dr);
            if !state.resolved.contains_key(&rn) {
                children.push((rn, rr));
            }
        }
        for (dn, dr) in &optional_deps {
            let (rn, rr) = resolve_alias(dn, dr);
            if !state.resolved.contains_key(&rn) {
                children.push((rn, rr));
            }
        }

        // Increment pending for ALL children before spawning any, so that the
        // main loop never observes pending == 0 prematurely.
        if !children.is_empty() {
            state.pending.fetch_add(children.len(), Ordering::SeqCst);
            for (child_name, child_range) in children {
                let child_state = state.clone();
                let chain = child_chain.clone();
                tokio::spawn(async move {
                    stream_resolve_package(child_state, child_name, child_range, chain).await;
                });
            }
        }

        // This task is complete.
        decrement_pending(&state.pending, &state.notify);
    }) // end Box::pin(async move { ... })
}

/// Resolve npm: alias protocol and bare package name aliases.
///
/// Handles:
/// - `"npm:actual-pkg@^1.0"` → `("actual-pkg", "^1.0")`
/// - `"npm:@scope/pkg@^1.0"` → `("@scope/pkg", "^1.0")`
/// - `"npm:actual-pkg"` → `("actual-pkg", "*")`
/// - `"storybook-jest"` (bare name, no semver) → `("storybook-jest", "*")`
///
/// Regular deps pass through unchanged.
fn resolve_alias(name: &str, range: &str) -> (String, String) {
    if let Some(alias_spec) = range.strip_prefix("npm:") {
        // npm:actual-pkg@^1.0 or npm:@scope/pkg@^1.0
        if let Some(at_pos) = alias_spec.rfind('@') {
            if at_pos > 0 {
                let real_name = &alias_spec[..at_pos];
                let real_range = &alias_spec[at_pos + 1..];
                return (real_name.to_string(), real_range.to_string());
            }
        }
        // npm:actual-pkg (no version)
        return (alias_spec.to_string(), "*".to_string());
    }
    // Bare package name as implicit npm alias.
    // e.g. `"@storybook/expect": "storybook-jest"` means "install
    // storybook-jest instead" — equivalent to `npm:storybook-jest@latest`.
    // Treat as the aliased package at any version ("*" = highest stable).
    if is_bare_package_name(range) {
        return (range.to_string(), "*".to_string());
    }
    (name.to_string(), range.to_string())
}

/// Returns `true` when `s` is a valid npm package name rather than a version
/// range.  Used to detect implicit npm-alias specs such as
/// `"@storybook/expect": "storybook-jest"`.
///
/// A bare package name:
/// - Contains no semver operators (`^`, `~`, `>`, `<`, `=`, `|`, ` `, `*`).
/// - Does not start with a digit (version numbers like `1.2.3` do).
/// - For scoped packages (`@scope/name`): non-empty scope and name separated
///   by `/`.
/// - All characters are alphanumeric, `-`, `_`, or `.` (plus `@`/`/` for
///   scoped names).
fn is_bare_package_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // Any semver operator or whitespace → version range, not a package name.
    if s.chars()
        .any(|c| matches!(c, '^' | '~' | '>' | '<' | '=' | '|' | ' ' | '*'))
    {
        return false;
    }
    // Starts with digit → version number (e.g. "1.2.3", "1.x").
    if s.starts_with(|c: char| c.is_ascii_digit()) {
        return false;
    }
    if let Some(rest) = s.strip_prefix('@') {
        // Scoped package: @scope/name — both parts must be non-empty.
        let mut parts = rest.splitn(2, '/');
        let scope = parts.next().unwrap_or("");
        let pkg = parts.next().unwrap_or("");
        if scope.is_empty() || pkg.is_empty() {
            return false;
        }
        scope
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
            && pkg
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    } else {
        // Unscoped: no `/` allowed (would look like a path).
        if s.contains('/') {
            return false;
        }
        s.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    }
}

/// Check if a package should be skipped based on platform (os/cpu).
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub fn should_skip_optional(version_meta: &super::registry::VersionMetadata) -> bool {
    // Check os field
    if let Some(os_list) = version_meta.os.as_ref() {
        if !os_list.is_empty() {
            let current_os = std::env::consts::OS;
            let mapped = match current_os {
                "macos" => "darwin",
                other => other,
            };
            let included = os_list
                .iter()
                .any(|o| !o.starts_with('!') && (o == mapped || o == current_os));
            let excluded = os_list.iter().any(|o| {
                o.starts_with('!')
                    && (o.trim_start_matches('!') == mapped
                        || o.trim_start_matches('!') == current_os)
            });
            if excluded || (!included && !os_list.iter().all(|o| o.starts_with('!'))) {
                return true;
            }
        }
    }

    // Check cpu field
    if let Some(cpu_list) = version_meta.cpu.as_ref() {
        if !cpu_list.is_empty() {
            let current_cpu = std::env::consts::ARCH;
            let mapped = match current_cpu {
                "aarch64" => "arm64",
                "x86_64" => "x64",
                other => other,
            };
            let included = cpu_list
                .iter()
                .any(|c| !c.starts_with('!') && (c == mapped || c == current_cpu));
            let excluded = cpu_list.iter().any(|c| {
                c.starts_with('!')
                    && (c.trim_start_matches('!') == mapped
                        || c.trim_start_matches('!') == current_cpu)
            });
            if excluded || (!included && !cpu_list.iter().all(|c| c.starts_with('!'))) {
                return true;
            }
        }
    }

    false
}

/// Resolve the npm `bin` field into a flat `HashMap<command, path>`.
///
/// - `BinField::Single("./cli.js")` → `{"<package-name>": "./cli.js"}`
/// - `BinField::Map({...})` → used as-is
/// - `None` → empty map
fn resolve_bin_field(
    package_name: &str,
    bin: &Option<super::registry::BinField>,
) -> HashMap<String, String> {
    match bin {
        Some(super::registry::BinField::Single(path)) => {
            let cmd = package_name.rsplit('/').next().unwrap_or(package_name);
            HashMap::from([(cmd.to_string(), path.clone())])
        }
        Some(super::registry::BinField::Map(map)) => map.clone(),
        None => HashMap::new(),
    }
}

/// Parse an npm version range string into a `semver::VersionReq`.
///
/// Handles npm-specific syntax:
/// - `^1.2.3` → `>=1.2.3, <2.0.0`
/// - `~1.2.3` → `>=1.2.3, <1.3.0`
/// - `>=1.0.0` / `<2.0.0` → passed through
/// - `*` or empty → any version
/// - Exact version `1.2.3` → `=1.2.3`
///
/// For `||` OR ranges, returns only the **first** parseable alternative.
/// Use `parse_all_version_ranges` when full OR semantics are needed.
#[allow(dead_code)]
fn parse_version_range(range: &str) -> Result<VersionReq> {
    let alternatives = parse_all_version_ranges(range)?;
    Ok(alternatives.into_iter().next().unwrap_or(VersionReq::STAR))
}

/// Parse a potentially OR-chained npm version range into **all** alternatives.
///
/// `^1.0.0 || ^2.0.0` → `[^1.0.0, ^2.0.0]`
/// Simple ranges return a single-element `Vec`.  An empty or `*` range
/// returns `[VersionReq::STAR]`.
///
/// This is the function used internally by `find_best_version` so that every
/// alternative in an `||` chain is considered during version selection.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn parse_all_version_ranges(range: &str) -> Result<Vec<VersionReq>> {
    let trimmed = range.trim();

    if trimmed.is_empty() || trimmed == "*" {
        return Ok(vec![VersionReq::STAR]);
    }

    if trimmed.contains("||") {
        let mut reqs: Vec<VersionReq> = Vec::new();
        for part in trimmed.split("||") {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            // Strip pre-release suffixes like -0 that the semver crate rejects.
            let cleaned = part
                .split(',')
                .map(|s| {
                    let s = s.trim();
                    if let Some(stripped) = s.strip_suffix("-0") {
                        stripped.to_string()
                    } else {
                        s.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let normalized = normalize_npm_range(&cleaned);
            if let Ok(req) = VersionReq::parse(&normalized) {
                reqs.push(req);
            }
        }
        if reqs.is_empty() {
            // Nothing parsed — try the raw string as a last resort.
            let req = VersionReq::parse(trimmed)
                .with_context(|| format!("Failed to parse version range '{}'", trimmed))?;
            return Ok(vec![req]);
        }
        return Ok(reqs);
    }

    // npm allows space-separated ranges like ">=1.0.0 <2.0.0";
    // the semver crate requires comma separation.
    let normalized = normalize_npm_range(trimmed);
    let req = VersionReq::parse(&normalized)
        .with_context(|| format!("Failed to parse version range '{}'", trimmed))?;
    Ok(vec![req])
}

/// Normalize npm-style version ranges to semver crate format.
/// - `>=1.0.0 <2.0.0` → `>=1.0.0, <2.0.0`
/// - `1.x` or `1.*` → `>=1.0.0, <2.0.0`
/// - `2 - 4` → `>=2.0.0, <5.0.0` (hyphen range)
fn normalize_npm_range(range: &str) -> String {
    let trimmed = range.trim();

    // Handle npm hyphen ranges: "X - Y"
    if let Some(expanded) = expand_hyphen_range(trimmed) {
        return expanded;
    }

    // Handle x-ranges: 1.x, 1.*, 1.2.x
    if trimmed.ends_with(".x") || trimmed.ends_with(".*") {
        let base = &trimmed[..trimmed.len() - 2];
        let parts: Vec<&str> = base.split('.').collect();
        return match parts.len() {
            1 => {
                if let Ok(major) = parts[0].parse::<u64>() {
                    format!(">={}.0.0, <{}.0.0", major, major + 1)
                } else {
                    trimmed.to_string()
                }
            }
            2 => {
                if let (Ok(major), Ok(minor)) = (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                    format!(">={}.{}.0, <{}.{}.0", major, minor, major, minor + 1)
                } else {
                    trimmed.to_string()
                }
            }
            _ => trimmed.to_string(),
        };
    }

    // Insert commas between space-separated range comparators
    // e.g., ">=1.0.0 <2.0.0" → ">=1.0.0, <2.0.0"
    let mut result = String::with_capacity(trimmed.len() + 4);
    let mut chars = trimmed.chars().peekable();
    let mut last_was_version = false;

    while let Some(ch) = chars.next() {
        if ch == ' ' && last_was_version {
            // Check if next non-space char is a range operator
            let rest: String = chars.clone().collect();
            let rest_trimmed = rest.trim_start();
            if rest_trimmed.starts_with('<')
                || rest_trimmed.starts_with('>')
                || rest_trimmed.starts_with('=')
            {
                result.push(',');
                result.push(' ');
                // Skip remaining spaces
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
                continue;
            }
        }
        last_was_version = ch.is_ascii_digit() || ch == '.';
        result.push(ch);
    }

    result
}

/// Expand npm hyphen range syntax `X - Y` into a comma-separated semver
/// comparator pair understood by the `semver` crate.
///
/// Mapping rules (npm spec):
/// - Y fully specified (`M.m.p`) → `>=X.0.0, <=M.m.p`
/// - Y major.minor only (`M.m`)  → `>=X.0.0, <M.(m+1).0`
/// - Y major only (`M`)          → `>=X.0.0, <(M+1).0.0`
///
/// Returns `None` if the input is not a well-formed hyphen range.
fn expand_hyphen_range(range: &str) -> Option<String> {
    // Require exactly one " - " separator (space-hyphen-space).
    let mut iter = range.splitn(2, " - ");
    let lo = iter.next()?.trim();
    let hi = iter.next()?.trim();

    // Both sides must consist of digits and dots only (no operators/letters).
    if !is_version_token(lo) || !is_version_token(hi) {
        return None;
    }

    let lo_parts: Vec<u64> = lo
        .split('.')
        .map(|s| s.parse::<u64>())
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let hi_parts: Vec<u64> = hi
        .split('.')
        .map(|s| s.parse::<u64>())
        .collect::<Result<Vec<_>, _>>()
        .ok()?;

    if lo_parts.is_empty() || hi_parts.is_empty() {
        return None;
    }

    let lo_major = lo_parts[0];
    let lo_minor = lo_parts.get(1).copied().unwrap_or(0);
    let lo_patch = lo_parts.get(2).copied().unwrap_or(0);

    let hi_major = hi_parts[0];
    let lower = format!(">={}.{}.{}", lo_major, lo_minor, lo_patch);
    let upper = match hi_parts.len() {
        1 => format!("<{}.0.0", hi_major + 1),
        2 => {
            let hi_minor = hi_parts[1];
            format!("<{}.{}.0", hi_major, hi_minor + 1)
        }
        _ => {
            let hi_minor = hi_parts.get(1).copied().unwrap_or(0);
            let hi_patch = hi_parts.get(2).copied().unwrap_or(0);
            format!("<={}.{}.{}", hi_major, hi_minor, hi_patch)
        }
    };

    Some(format!("{}, {}", lower, upper))
}

/// Returns `true` if `s` looks like a bare version token: non-empty, composed
/// only of ASCII digits and dots (no operators, letters, or spaces).
fn is_version_token(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit() || c == '.')
}

/// Find the highest version from `versions` that matches any of `reqs`.
///
/// Selection rules:
/// 1. Stable versions that satisfy any req are preferred.
/// 2. Pre-release versions that satisfy any req are the second choice.
/// 3. If no version satisfies any req even with direct matching, a **relaxed**
///    pre-release fallback is used: the pre-release tag is stripped and the
///    resulting base version is checked against the reqs.  This mirrors npm's
///    behaviour where `2.0.0-alpha.1` is returned when no stable
///    `>=2.0.0` exists.
fn find_best_version(
    versions: &HashMap<String, super::registry::VersionMetadata>,
    reqs: &[VersionReq],
    package_name: &str,
) -> Result<String> {
    let mut best: Option<Version> = None;
    let mut best_pre: Option<Version> = None;

    for key in versions.keys() {
        if let Ok(ver) = Version::parse(key) {
            let matches = reqs.iter().any(|r| r.matches(&ver));
            if matches {
                if ver.pre.is_empty() {
                    match &best {
                        Some(current) if &ver > current => best = Some(ver),
                        None => best = Some(ver),
                        _ => {}
                    }
                } else {
                    match &best_pre {
                        Some(current) if &ver > current => best_pre = Some(ver),
                        None => best_pre = Some(ver),
                        _ => {}
                    }
                }
            } else if !ver.pre.is_empty() {
                // Relaxed pre-release fallback: the semver crate does not
                // match pre-release versions against ranges that only
                // specify stable comparators (e.g. `>=2.0.0` does not
                // match `2.0.0-alpha.1`).  Strip the pre-release tag and
                // check the resulting base version.
                let base = Version::new(ver.major, ver.minor, ver.patch);
                let base_matches = reqs.iter().any(|r| r.matches(&base));
                if base_matches {
                    match &best_pre {
                        Some(current) if &ver > current => best_pre = Some(ver),
                        None => best_pre = Some(ver),
                        _ => {}
                    }
                }
            }
        }
    }

    // Prefer stable over pre-release.
    best.or(best_pre)
        .map(|v| v.to_string())
        .ok_or_else(|| anyhow::anyhow!("No version of '{}' matches requirement", package_name,))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolver_creation() {
        let _resolver = DependencyResolver::new();
        let _default = DependencyResolver::default();
    }

    // ─── GH #3516: resolver speculative pre-warm silent error swallow ────

    /// GH #3516 — the warn message emitted when speculative pre-warm
    /// hits a registry error must name the failing package, name the
    /// underlying error verbatim, AND carry the `GH #3516` tag so the
    /// dev has a searchable breadcrumb when installs go slow.
    #[test]
    fn gh3516_prewarm_warn_names_package_error_and_issue() {
        let err = anyhow::anyhow!("Registry returned 503 for package 'react'");
        let msg = format_speculative_prewarm_warn("react", &err);
        assert!(
            msg.contains("'react'"),
            "must name failing package, got: {msg}"
        );
        assert!(
            msg.contains("Registry returned 503"),
            "must preserve underlying error verbatim, got: {msg}"
        );
        assert!(
            msg.contains("GH #3516"),
            "must include searchable issue tag, got: {msg}"
        );
    }

    /// GH #3516 — the warn message must explain the dev-visible
    /// symptom (installs appear slower) so the operator does not
    /// mis-attribute the timing to BFS itself.
    #[test]
    fn gh3516_prewarm_warn_explains_dev_visible_symptom() {
        let err = anyhow::anyhow!("dns resolution failed");
        let msg = format_speculative_prewarm_warn("lodash", &err);
        assert!(
            msg.contains("BFS will fetch on demand"),
            "must explain that resolution still works, got: {msg}"
        );
        assert!(
            msg.contains("slower"),
            "must call out the user-visible symptom, got: {msg}"
        );
        assert!(
            msg.contains(".npmrc"),
            "must point at typical fix surfaces, got: {msg}"
        );
    }

    /// GH #3516 — the helper must not panic / propagate when the
    /// registry call errors. Speculative pre-warm is best-effort; a
    /// hard failure here must never abort BFS resolution.
    ///
    /// Drives the failure path with a registry URL that cannot resolve
    /// AND `no_cache: true` so no XDG dir setup is needed in tests.
    #[tokio::test]
    async fn gh3516_prewarm_speculative_dep_swallows_registry_error() {
        let npmrc = super::super::npmrc::NpmrcConfig::default();
        // Use an obviously-unreachable URL so the underlying reqwest
        // fails. `no_cache: true` to keep test hermetic — no $HOME
        // / XDG side effects.
        let reg = RegistryClient::new_with_options("http://127.0.0.1:1", &npmrc, true)
            .expect("RegistryClient::new_with_options");

        // Must not panic and must return Ok/unit even though the
        // underlying fetch fails. We don't assert on tracing output
        // here — the message contract is pinned by the
        // `format_speculative_prewarm_warn` tests above.
        prewarm_speculative_dep(&reg, "definitely-nonexistent-package-3516").await;
    }

    #[test]
    fn test_version_range_parsing() {
        // Caret range
        let req = parse_version_range("^1.2.3").unwrap();
        assert!(req.matches(&Version::new(1, 9, 0)));
        assert!(!req.matches(&Version::new(2, 0, 0)));

        // Tilde range
        let req = parse_version_range("~1.2.3").unwrap();
        assert!(req.matches(&Version::new(1, 2, 9)));
        assert!(!req.matches(&Version::new(1, 3, 0)));

        // Exact version
        let req = parse_version_range("1.2.3").unwrap();
        assert!(req.matches(&Version::new(1, 2, 3)));

        // Star / wildcard
        let req = parse_version_range("*").unwrap();
        assert!(req.matches(&Version::new(99, 99, 99)));

        // Empty string
        let req = parse_version_range("").unwrap();
        assert!(req.matches(&Version::new(0, 0, 1)));

        // Greater-than-or-equal
        let req = parse_version_range(">=2.0.0").unwrap();
        assert!(req.matches(&Version::new(3, 0, 0)));
        assert!(!req.matches(&Version::new(1, 0, 0)));
    }

    #[test]
    fn test_find_best_version() {
        use super::super::registry::{DistInfo, VersionMetadata};

        let mut versions = HashMap::new();
        for v in ["1.0.0", "1.1.0", "1.2.0", "2.0.0"] {
            versions.insert(
                v.to_string(),
                VersionMetadata {
                    version: v.to_string(),
                    dist: DistInfo {
                        tarball: format!("https://example.com/{}.tgz", v),
                        shasum: "abc".to_string(),
                        integrity: None,
                    },
                    dependencies: None,
                    peer_dependencies: None,
                    optional_dependencies: None,
                    bin: None,
                    scripts: None,
                    os: None,
                    cpu: None,
                },
            );
        }

        let reqs = parse_all_version_ranges("^1.0.0").unwrap();
        let best = find_best_version(&versions, &reqs, "test").unwrap();
        assert_eq!(best, "1.2.0");

        let reqs = parse_all_version_ranges("~1.0.0").unwrap();
        let best = find_best_version(&versions, &reqs, "test").unwrap();
        assert_eq!(best, "1.0.0");

        let reqs = parse_all_version_ranges(">=2.0.0").unwrap();
        let best = find_best_version(&versions, &reqs, "test").unwrap();
        assert_eq!(best, "2.0.0");
    }

    // ──────────────────────────────────────────────────────────────
    // R4: Additional resolver tests (||, npm:, space-sep, pre-release,
    //     optional deps, x-ranges, no-match)
    // ──────────────────────────────────────────────────────────────

    /// OR syntax `||` — parse_all_version_ranges returns all alternatives.
    ///
    /// The `semver::VersionReq` type does not support OR semantics, so we
    /// represent them as a `Vec<VersionReq>`.  Any version satisfying at
    /// least one element is considered a match.
    #[test]
    fn test_version_range_or_syntax() {
        // "^1.0.0 || ^2.0.0" must produce two requirements.
        let reqs = parse_all_version_ranges("^1.0.0 || ^2.0.0").unwrap();
        assert_eq!(reqs.len(), 2, "Expected 2 alternatives from ||");

        let matches = |v: &Version| reqs.iter().any(|r| r.matches(v));

        assert!(
            matches(&Version::new(1, 5, 0)),
            "^1.0.0 || ^2.0.0 must match 1.5.0"
        );
        assert!(
            matches(&Version::new(2, 0, 0)),
            "^1.0.0 || ^2.0.0 must match 2.0.0"
        );
        assert!(
            !matches(&Version::new(3, 0, 0)),
            "^1.0.0 || ^2.0.0 must not match 3.0.0"
        );
    }

    /// OR syntax with three alternatives produces three VersionReq objects.
    #[test]
    fn test_version_range_or_syntax_three_alternatives() {
        let reqs = parse_all_version_ranges("^1.0.0 || ^2.0.0 || ^3.0.0").unwrap();
        assert_eq!(reqs.len(), 3, "Expected 3 alternatives from ||");

        let matches = |v: &Version| reqs.iter().any(|r| r.matches(v));
        assert!(matches(&Version::new(1, 0, 0)));
        assert!(matches(&Version::new(2, 0, 0)));
        assert!(matches(&Version::new(3, 0, 0)));
        assert!(!matches(&Version::new(4, 0, 0)));
    }

    /// find_best_version picks the highest version satisfying any OR branch.
    #[test]
    fn test_find_best_version_or_picks_highest_across_branches() {
        use super::super::registry::{DistInfo, VersionMetadata};

        let make_ver = |v: &str| VersionMetadata {
            version: v.to_string(),
            dist: DistInfo {
                tarball: format!("https://example.com/{}.tgz", v),
                shasum: "abc".to_string(),
                integrity: None,
            },
            dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            bin: None,
            scripts: None,
            os: None,
            cpu: None,
        };

        let mut versions = HashMap::new();
        for v in ["1.5.0", "2.0.0", "2.1.0"] {
            versions.insert(v.to_string(), make_ver(v));
        }

        // "^1.0.0 || ^2.0.0" should select 2.1.0 (highest across both branches)
        let reqs = parse_all_version_ranges("^1.0.0 || ^2.0.0").unwrap();
        let best = find_best_version(&versions, &reqs, "or-test").unwrap();
        assert_eq!(best, "2.1.0");
    }

    /// Space-separated ranges: `>=1.0.0 <2.0.0` (npm shorthand).
    #[test]
    fn test_version_range_space_separated() {
        let req = parse_version_range(">=1.0.0 <2.0.0").unwrap();
        assert!(req.matches(&Version::new(1, 0, 0)));
        assert!(req.matches(&Version::new(1, 99, 0)));
        assert!(!req.matches(&Version::new(2, 0, 0)));
        assert!(!req.matches(&Version::new(0, 9, 0)));
    }

    /// Space-separated with patch pin: `>=1.2.3 <1.3.0`.
    #[test]
    fn test_version_range_space_separated_patch_pin() {
        let req = parse_version_range(">=1.2.3 <1.3.0").unwrap();
        assert!(req.matches(&Version::new(1, 2, 3)));
        assert!(req.matches(&Version::new(1, 2, 9)));
        assert!(!req.matches(&Version::new(1, 3, 0)));
    }

    /// x-range `1.x` matches any patch/minor under major 1.
    #[test]
    fn test_version_range_x_range_major() {
        let req = parse_version_range("1.x").unwrap();
        assert!(req.matches(&Version::new(1, 0, 0)));
        assert!(req.matches(&Version::new(1, 99, 0)));
        assert!(!req.matches(&Version::new(2, 0, 0)));
    }

    /// x-range `1.2.x` matches any patch under minor 1.2.
    #[test]
    fn test_version_range_x_range_minor() {
        let req = parse_version_range("1.2.x").unwrap();
        assert!(req.matches(&Version::new(1, 2, 0)));
        assert!(req.matches(&Version::new(1, 2, 9)));
        assert!(!req.matches(&Version::new(1, 3, 0)));
    }

    /// Wildcard `*` matches everything including 0.x.
    #[test]
    fn test_version_range_wildcard_matches_all() {
        let req = parse_version_range("*").unwrap();
        assert!(req.matches(&Version::new(0, 0, 1)));
        assert!(req.matches(&Version::new(100, 0, 0)));
    }

    /// `npm:` alias protocol is stripped and the real package/range returned.
    #[test]
    fn test_npm_alias_resolution() {
        let (name, range) = resolve_alias("lodash-es", "npm:lodash@^4.17.0");
        assert_eq!(name, "lodash");
        assert_eq!(range, "^4.17.0");
    }

    /// `npm:` alias for a scoped package.
    #[test]
    fn test_npm_alias_scoped_package() {
        let (name, range) = resolve_alias("react-compat", "npm:@legacy/react@^17.0.0");
        assert_eq!(name, "@legacy/react");
        assert_eq!(range, "^17.0.0");
    }

    /// `npm:` alias without version falls back to `*`.
    #[test]
    fn test_npm_alias_no_version() {
        let (name, range) = resolve_alias("my-alias", "npm:actual-pkg");
        assert_eq!(name, "actual-pkg");
        assert_eq!(range, "*");
    }

    /// Regular dependency passes through resolve_alias unchanged.
    #[test]
    fn test_resolve_alias_passthrough() {
        let (name, range) = resolve_alias("react", "^18.0.0");
        assert_eq!(name, "react");
        assert_eq!(range, "^18.0.0");
    }

    /// Bare package name as version spec is treated as an implicit npm alias.
    /// e.g. `"@storybook/expect": "storybook-jest"` → install storybook-jest@*.
    #[test]
    fn test_resolve_alias_bare_package_name() {
        let (name, range) = resolve_alias("@storybook/expect", "storybook-jest");
        assert_eq!(name, "storybook-jest");
        assert_eq!(range, "*");
    }

    /// Bare scoped package name as version spec.
    #[test]
    fn test_resolve_alias_bare_scoped_package_name() {
        let (name, range) = resolve_alias("some-alias", "@my-scope/actual-pkg");
        assert_eq!(name, "@my-scope/actual-pkg");
        assert_eq!(range, "*");
    }

    // ── is_bare_package_name ──────────────────────────────────────────────

    #[test]
    fn test_is_bare_package_name_accepts_simple_names() {
        assert!(is_bare_package_name("storybook-jest"));
        assert!(is_bare_package_name("lodash"));
        assert!(is_bare_package_name("react_dom"));
        assert!(is_bare_package_name("some.pkg"));
    }

    #[test]
    fn test_is_bare_package_name_accepts_scoped_names() {
        assert!(is_bare_package_name("@storybook/jest"));
        assert!(is_bare_package_name("@my-scope/actual-pkg"));
    }

    #[test]
    fn test_is_bare_package_name_rejects_semver_ranges() {
        assert!(!is_bare_package_name("^1.0.0"));
        assert!(!is_bare_package_name("~1.0.0"));
        assert!(!is_bare_package_name(">=1.0.0"));
        assert!(!is_bare_package_name("1.2.3"));
        assert!(!is_bare_package_name("*"));
        assert!(!is_bare_package_name(""));
        assert!(!is_bare_package_name("^1.0.0 || ^2.0.0"));
        assert!(!is_bare_package_name(">=1.0.0 <2.0.0"));
    }

    #[test]
    fn test_is_bare_package_name_rejects_incomplete_scoped() {
        // Missing name after slash
        assert!(!is_bare_package_name("@scope/"));
        // Missing slash entirely
        assert!(!is_bare_package_name("@scope"));
        // Empty scope
        assert!(!is_bare_package_name("@/name"));
    }

    /// Pre-release fallback: when no stable version satisfies the range,
    /// find_best_version must fall back to a pre-release.
    #[test]
    fn test_find_best_version_prerelease_fallback() {
        use super::super::registry::{DistInfo, VersionMetadata};

        let make_ver = |v: &str| VersionMetadata {
            version: v.to_string(),
            dist: DistInfo {
                tarball: format!("https://example.com/{}.tgz", v),
                shasum: "abc".to_string(),
                integrity: None,
            },
            dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            bin: None,
            scripts: None,
            os: None,
            cpu: None,
        };

        let mut versions = HashMap::new();
        versions.insert("2.0.0-alpha.1".to_string(), make_ver("2.0.0-alpha.1"));
        versions.insert("2.0.0-beta.1".to_string(), make_ver("2.0.0-beta.1"));

        // No stable version exists — should fall back to highest pre-release.
        let reqs = parse_all_version_ranges(">=2.0.0").unwrap();
        let best = find_best_version(&versions, &reqs, "pre-only-pkg").unwrap();
        // Must pick the highest pre-release: beta > alpha
        assert_eq!(best, "2.0.0-beta.1");
    }

    /// Stable version wins over pre-release when both satisfy the range.
    #[test]
    fn test_find_best_version_stable_wins_over_prerelease() {
        use super::super::registry::{DistInfo, VersionMetadata};

        let make_ver = |v: &str| VersionMetadata {
            version: v.to_string(),
            dist: DistInfo {
                tarball: format!("https://example.com/{}.tgz", v),
                shasum: "abc".to_string(),
                integrity: None,
            },
            dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            bin: None,
            scripts: None,
            os: None,
            cpu: None,
        };

        let mut versions = HashMap::new();
        versions.insert("1.0.0".to_string(), make_ver("1.0.0"));
        versions.insert("1.1.0-alpha.1".to_string(), make_ver("1.1.0-alpha.1"));

        let reqs = parse_all_version_ranges("^1.0.0").unwrap();
        let best = find_best_version(&versions, &reqs, "mixed-pkg").unwrap();
        // Stable 1.0.0 must be preferred over pre-release 1.1.0-alpha.1
        assert_eq!(best, "1.0.0");
    }

    /// No matching version returns an error.
    #[test]
    fn test_find_best_version_no_match_returns_error() {
        use super::super::registry::{DistInfo, VersionMetadata};

        let mut versions = HashMap::new();
        versions.insert(
            "1.0.0".to_string(),
            VersionMetadata {
                version: "1.0.0".to_string(),
                dist: DistInfo {
                    tarball: "https://example.com/1.0.0.tgz".to_string(),
                    shasum: "abc".to_string(),
                    integrity: None,
                },
                dependencies: None,
                peer_dependencies: None,
                optional_dependencies: None,
                bin: None,
                scripts: None,
                os: None,
                cpu: None,
            },
        );

        let reqs = parse_all_version_ranges("^2.0.0").unwrap();
        let result = find_best_version(&versions, &reqs, "constrained-pkg");
        assert!(
            result.is_err(),
            "Must error when no version satisfies range"
        );
    }

    /// should_skip_optional skips packages when OS does not match.
    #[test]
    fn test_optional_dep_skipped_wrong_os() {
        use super::super::registry::{DistInfo, VersionMetadata};

        let current_os = std::env::consts::OS;
        // Pick the opposite OS to force a skip.
        let other_os = if current_os == "linux" {
            "darwin"
        } else {
            "linux"
        };

        let meta = VersionMetadata {
            version: "1.0.0".to_string(),
            dist: DistInfo {
                tarball: "https://example.com/1.0.0.tgz".to_string(),
                shasum: "abc".to_string(),
                integrity: None,
            },
            dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            bin: None,
            scripts: None,
            os: Some(vec![other_os.to_string()]),
            cpu: None,
        };

        assert!(
            should_skip_optional(&meta),
            "Package targeting a different OS must be skipped"
        );
    }

    /// should_skip_optional keeps packages when OS matches.
    #[test]
    fn test_optional_dep_included_correct_os() {
        use super::super::registry::{DistInfo, VersionMetadata};

        // Map Rust OS name to npm convention.
        let current_os = match std::env::consts::OS {
            "macos" => "darwin",
            other => other,
        };

        let meta = VersionMetadata {
            version: "1.0.0".to_string(),
            dist: DistInfo {
                tarball: "https://example.com/1.0.0.tgz".to_string(),
                shasum: "abc".to_string(),
                integrity: None,
            },
            dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            bin: None,
            scripts: None,
            os: Some(vec![current_os.to_string()]),
            cpu: None,
        };

        assert!(
            !should_skip_optional(&meta),
            "Package targeting the current OS must not be skipped"
        );
    }

    /// should_skip_optional skips packages when CPU architecture does not match.
    #[test]
    fn test_optional_dep_skipped_wrong_cpu() {
        use super::super::registry::{DistInfo, VersionMetadata};

        let current_arch = std::env::consts::ARCH;
        let other_arch = if current_arch.contains("64") {
            "ia32"
        } else {
            "x64"
        };

        let meta = VersionMetadata {
            version: "1.0.0".to_string(),
            dist: DistInfo {
                tarball: "https://example.com/1.0.0.tgz".to_string(),
                shasum: "abc".to_string(),
                integrity: None,
            },
            dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            bin: None,
            scripts: None,
            os: None,
            cpu: Some(vec![other_arch.to_string()]),
        };

        assert!(
            should_skip_optional(&meta),
            "Package targeting a different CPU must be skipped"
        );
    }

    /// should_skip_optional keeps packages with no OS/CPU restrictions.
    #[test]
    fn test_optional_dep_included_no_restrictions() {
        use super::super::registry::{DistInfo, VersionMetadata};

        let meta = VersionMetadata {
            version: "1.0.0".to_string(),
            dist: DistInfo {
                tarball: "https://example.com/1.0.0.tgz".to_string(),
                shasum: "abc".to_string(),
                integrity: None,
            },
            dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            bin: None,
            scripts: None,
            os: None,
            cpu: None,
        };

        assert!(
            !should_skip_optional(&meta),
            "Package with no OS/CPU restrictions must never be skipped"
        );
    }

    /// should_skip_optional handles negated OS exclusions (`!darwin`).
    #[test]
    fn test_optional_dep_negated_os_exclusion() {
        use super::super::registry::{DistInfo, VersionMetadata};

        // A package that excludes darwin: should skip on macOS, keep elsewhere.
        let meta = VersionMetadata {
            version: "1.0.0".to_string(),
            dist: DistInfo {
                tarball: "https://example.com/1.0.0.tgz".to_string(),
                shasum: "abc".to_string(),
                integrity: None,
            },
            dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            bin: None,
            scripts: None,
            os: Some(vec!["!darwin".to_string()]),
            cpu: None,
        };

        let is_mac = std::env::consts::OS == "macos";
        let skipped = should_skip_optional(&meta);
        if is_mac {
            assert!(skipped, "!darwin must skip on macOS");
        } else {
            assert!(!skipped, "!darwin must not skip on non-macOS");
        }
    }

    // ── Hyphen range syntax (#960) ────────────────────────────────────────

    /// Hyphen range with major-only upper bound: `2 - 4` → `>=2.0.0 <5.0.0`.
    #[test]
    fn test_hyphen_range_major_only() {
        let req = parse_version_range("2 - 4").unwrap();
        assert!(req.matches(&Version::new(2, 0, 0)), "2.0.0 in [2-4]");
        assert!(req.matches(&Version::new(4, 9, 9)), "4.9.9 in [2-4]");
        assert!(!req.matches(&Version::new(5, 0, 0)), "5.0.0 not in [2-4]");
        assert!(!req.matches(&Version::new(1, 9, 9)), "1.9.9 not in [2-4]");
    }

    /// Hyphen range with major.minor upper bound: `1.0 - 2.0` → `>=1.0.0 <2.1.0`.
    #[test]
    fn test_hyphen_range_major_minor() {
        let req = parse_version_range("1.0 - 2.0").unwrap();
        assert!(req.matches(&Version::new(1, 0, 0)), "1.0.0 in [1.0-2.0]");
        assert!(req.matches(&Version::new(2, 0, 9)), "2.0.9 in [1.0-2.0]");
        assert!(
            !req.matches(&Version::new(2, 1, 0)),
            "2.1.0 not in [1.0-2.0]"
        );
        assert!(
            !req.matches(&Version::new(0, 9, 9)),
            "0.9.9 not in [1.0-2.0]"
        );
    }

    /// Hyphen range with fully specified bounds: `1.0.0 - 2.0.0` → `>=1.0.0 <=2.0.0`.
    #[test]
    fn test_hyphen_range_fully_specified() {
        let req = parse_version_range("1.0.0 - 2.0.0").unwrap();
        assert!(
            req.matches(&Version::new(1, 0, 0)),
            "1.0.0 in [1.0.0-2.0.0]"
        );
        assert!(
            req.matches(&Version::new(2, 0, 0)),
            "2.0.0 in [1.0.0-2.0.0]"
        );
        assert!(
            !req.matches(&Version::new(2, 0, 1)),
            "2.0.1 not in [1.0.0-2.0.0]"
        );
        assert!(
            !req.matches(&Version::new(0, 9, 9)),
            "0.9.9 not in [1.0.0-2.0.0]"
        );
    }

    /// expand_hyphen_range rejects non-hyphen-range strings without panicking.
    #[test]
    fn test_expand_hyphen_range_rejects_non_hyphen() {
        assert!(
            expand_hyphen_range("^1.0.0").is_none(),
            "caret range must not match"
        );
        assert!(
            expand_hyphen_range(">=1.0.0 <2.0.0").is_none(),
            "space-separated range must not match"
        );
        assert!(
            expand_hyphen_range("storybook-jest").is_none(),
            "bare package name with hyphen must not match"
        );
        assert!(
            expand_hyphen_range("1.0.0-alpha - 2.0.0").is_none(),
            "pre-release lower bound must not match"
        );
    }

    /// normalize_npm_range converts space-separated comparators correctly.
    #[test]
    fn test_normalize_npm_range_space_to_comma() {
        let result = normalize_npm_range(">=1.0.0 <2.0.0");
        assert!(
            result.contains(','),
            "Expected comma in normalized range, got: {}",
            result
        );
        let req = VersionReq::parse(&result).unwrap();
        assert!(req.matches(&Version::new(1, 5, 0)));
        assert!(!req.matches(&Version::new(2, 0, 0)));
    }

    /// resolve_bin_field handles the single-string form.
    #[test]
    fn test_resolve_bin_field_single_string() {
        use super::super::registry::BinField;

        let bin = Some(BinField::Single("./bin/cli.js".to_string()));
        let result = resolve_bin_field("my-tool", &bin);
        assert_eq!(result.get("my-tool"), Some(&"./bin/cli.js".to_string()));
    }

    /// resolve_bin_field handles the map form.
    #[test]
    fn test_resolve_bin_field_map() {
        use super::super::registry::BinField;

        let mut map = HashMap::new();
        map.insert("foo".to_string(), "./foo.js".to_string());
        map.insert("bar".to_string(), "./bar.js".to_string());
        let bin = Some(BinField::Map(map));
        let result = resolve_bin_field("my-pkg", &bin);
        assert_eq!(result.get("foo"), Some(&"./foo.js".to_string()));
        assert_eq!(result.get("bar"), Some(&"./bar.js".to_string()));
    }

    /// resolve_bin_field with None returns empty map.
    #[test]
    fn test_resolve_bin_field_none() {
        let result = resolve_bin_field("my-pkg", &None);
        assert!(result.is_empty());
    }
}
// CODEGEN-END

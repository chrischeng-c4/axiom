---
id: implementation
type: change_implementation
change_id: stream-resolver
---

# Implementation

## Summary

Two-file change in crates/cclab-jet/src/pkg_manager/ (+340/-274): (1) registry.rs — added #[derive(Clone)] to RegistryClient so it can be cloned into concurrent tokio::spawn tasks (+1/-0); (2) resolver.rs — complete rewrite of resolve_with_prefetch from BFS level-based to stream-based: introduces ResolverState (DashMap<String,ResolvedPackage> resolved, DashMap<String,()> visited, Arc<AtomicUsize> pending, Arc<Notify> wakeup, RegistryClient clone, overrides, prefetch_tx, error_slot), replaces the 3-phase BFS loop (filter→prefetch→process) with tokio::spawn tasks via stream_resolve_package (Box::pin async fn for recursion), adds try_claim_package (sync DashMap entry API — no guard across await), adds decrement_pending (atomic sub + notify_one on zero), and converts speculative SPECULATIVE_DEPS pre-warm to fire-and-forget spawns.

## Diff

```diff
diff --git a/crates/cclab-jet/src/pkg_manager/registry.rs b/crates/cclab-jet/src/pkg_manager/registry.rs
index 800563d4..b833ca55 100644
--- a/crates/cclab-jet/src/pkg_manager/registry.rs
+++ b/crates/cclab-jet/src/pkg_manager/registry.rs
@@ -21,6 +21,7 @@ const DISK_CACHE_TTL: Duration = Duration::from_secs(300);
 ///
 /// Pass `no_cache: true` to skip disk reads/writes (still uses L1 memory
 /// cache within a single process run). Useful for `jet install --no-cache`.
+#[derive(Clone)]
 pub struct RegistryClient {
     client: reqwest::Client,
     #[allow(dead_code)]
diff --git a/crates/cclab-jet/src/pkg_manager/resolver.rs b/crates/cclab-jet/src/pkg_manager/resolver.rs
index 77323d3f..38c5c25c 100644
--- a/crates/cclab-jet/src/pkg_manager/resolver.rs
+++ b/crates/cclab-jet/src/pkg_manager/resolver.rs
@@ -1,7 +1,10 @@
 use anyhow::{Context, Result};
+use dashmap::DashMap;
 use semver::{Version, VersionReq};
-use std::collections::{HashMap, HashSet};
-use tokio::sync::mpsc;
+use std::collections::HashMap;
+use std::sync::atomic::{AtomicUsize, Ordering};
+use std::sync::Arc;
+use tokio::sync::{mpsc, Notify};
 
 use super::registry::RegistryClient;
 
@@ -44,6 +47,27 @@ pub struct ResolvedPackage {
 /// detection. Uses greedy strategy: pick highest compatible version.
 pub struct DependencyResolver {}
 
+/// Shared state threaded through all concurrent resolution tasks.
+#[derive(Clone)]
+struct ResolverState {
+    /// Packages that have been fully resolved.
+    resolved: Arc<DashMap<String, ResolvedPackage>>,
+    /// Packages that have been claimed for resolution (prevents duplicate work).
+    visited: Arc<DashMap<String, ()>>,
+    /// Number of tasks currently in-flight (spawned but not yet finished).
+    pending: Arc<AtomicUsize>,
+    /// Wakes the main task when `pending` reaches 0 or a fatal error occurs.
+    notify: Arc<Notify>,
+    /// Shared registry client (all clones share the same L1 cache via Arc).
+    registry: RegistryClient,
+    /// Override map: package name → forced version range.
+    overrides: Arc<HashMap<String, String>>,
+    /// Optional channel for notifying a background downloader of resolved pkgs.
+    prefetch_tx: Arc<Option<mpsc::UnboundedSender<ResolvedPackage>>>,
+    /// Stores the first fatal error produced by a direct-dep task.
+    error_slot: Arc<std::sync::Mutex<Option<anyhow::Error>>>,
+}
+
 impl DependencyResolver {
     pub fn new() -> Self {
         Self {}
@@ -61,15 +85,23 @@ impl DependencyResolver {
             .await
     }
 
-    /// Resolve all dependencies (direct + transitive) using
-    /// parallel-prefetch BFS. Each BFS level collects all unresolved
-    /// package names, fetches their metadata concurrently, then
-    /// processes results and enqueues the next level.
+    /// Resolve all dependencies (direct + transitive) using a stream-based
+    /// approach. Each resolved package **immediately** spawns `tokio::spawn`
+    /// tasks for its transitive dependencies, without waiting for sibling
+    /// packages at the same logical level to finish.
+    ///
+    /// Shared state:
+    /// - `DashMap<String, ResolvedPackage>` — resolved packages (lock-free).
+    /// - `DashMap<String, ()>` — visited set; atomically claims a package for
+    ///   resolution, preventing duplicate work across concurrent tasks.
+    /// - `Arc<AtomicUsize>` — pending task counter; incremented before each
+    ///   `tokio::spawn`, decremented when the task finishes. Reaching 0 signals
+    ///   completion.
     ///
     /// When `prefetch_tx` is `Some`, each `ResolvedPackage` is sent on the
-    /// channel **immediately after version selection**, before the next BFS
-    /// level is processed. A background consumer can start tarball downloads
-    /// while resolution continues, overlapping network I/O (R7).
+    /// channel immediately after version selection so that a background
+    /// consumer can start tarball downloads while resolution continues,
+    /// overlapping network I/O.
     pub async fn resolve_with_prefetch(
         &self,
         deps: &HashMap<String, String>,
@@ -77,297 +109,331 @@ impl DependencyResolver {
         overrides: &HashMap<String, String>,
         prefetch_tx: Option<mpsc::UnboundedSender<ResolvedPackage>>,
     ) -> Result<HashMap<String, ResolvedPackage>> {
-        let mut resolved: HashMap<String, ResolvedPackage> =
-            HashMap::new();
-        let mut visited: HashSet<String> = HashSet::new();
-        let mut queue: Vec<(String, String, Vec<String>)> = Vec::new();
+        let notify = Arc::new(Notify::new());
+        let pending = Arc::new(AtomicUsize::new(0));
 
-        // Seed with direct dependencies
-        for (name, range) in deps {
-            let (real_name, real_range) = resolve_alias(name, range);
-            queue.push((
-                real_name,
-                real_range,
-                vec!["(root)".to_string()],
-            ));
-        }
+        let state = ResolverState {
+            resolved: Arc::new(DashMap::new()),
+            visited: Arc::new(DashMap::new()),
+            pending: Arc::clone(&pending),
+            notify: Arc::clone(&notify),
+            registry: registry.clone(),
+            overrides: Arc::new(overrides.clone()),
+            prefetch_tx: Arc::new(prefetch_tx),
+            error_slot: Arc::new(std::sync::Mutex::new(None)),
+        };
 
-        let mut is_first_bfs_level = true;
+        // Speculative pre-warm: fire off cache-warming fetches for common
+        // transitive deps in parallel with direct dep resolution, so that
+        // when BFS naturally reaches them they hit the in-memory cache.
+        for &spec_name in SPECULATIVE_DEPS {
+            let reg = state.registry.clone();
+            let name = spec_name.to_string();
+            tokio::spawn(async move {
+                let _ = reg.get_package_metadata(&name).await;
+            });
+        }
 
-        while !queue.is_empty() {
-            // --- Phase 1: filter queue, collect unique names to fetch ---
-            let mut to_process: Vec<(String, String, Vec<String>)> =
-                Vec::new();
-            let mut names_to_fetch: Vec<String> = Vec::new();
-            let mut seen_this_round: HashSet<String> = HashSet::new();
+        // Collect seeds (direct deps) and increment pending atomically before
+        // spawning any task, so that the wait loop never sees pending == 0
+        // while tasks are still being set up.
+        let seeds: Vec<(String, String)> = deps
+            .iter()
+            .map(|(n, r)| resolve_alias(n, r))
+            .collect();
 
-            for (name, range_str, dep_chain) in queue.drain(..) {
-                let range_str =
-                    if let Some(forced) = overrides.get(&name) {
-                        forced.clone()
-                    } else {
-                        range_str
-                    };
+        if seeds.is_empty() {
+            return Ok(HashMap::new());
+        }
 
-                if resolved.contains_key(&name) {
-                    // Version conflict check (warn only).
-                    // Use all OR alternatives so that `^1.0.0 || ^2.0.0`
-                    // does not spuriously warn when the selected version
-                    // satisfies the second branch.
-                    let existing = &resolved[&name];
-                    if let Ok(reqs) = parse_all_version_ranges(&range_str) {
-                        if let Ok(ev) =
-                            Version::parse(&existing.version)
-                        {
-                            let satisfied =
-                                reqs.iter().any(|r| r.matches(&ev));
-                            if !satisfied {
-                                tracing::warn!(
-                                    "Version conflict '{}': \
-                                     using {}@{}, wanted '{}' by {}",
-                                    name,
-                                    name,
-                                    existing.version,
-                                    range_str,
-                                    dep_chain
-                                        .last()
-                                        .unwrap_or(
-                                            &"(root)".to_string()
-                                        )
-                                );
-                            }
-                        }
-                    }
-                    continue;
-                }
+        pending.fetch_add(seeds.len(), Ordering::SeqCst);
+        for (real_name, real_range) in seeds {
+            let task_state = state.clone();
+            tokio::spawn(async move {
+                stream_resolve_package(
+                    task_state,
+                    real_name,
+                    real_range,
+                    vec!["(root)".to_string()],
+                )
+                .await;
+            });
+        }
 
-                if visited.contains(&name) {
-                    continue;
-                }
+        // Wait until all tasks complete (pending == 0) or a fatal error is set.
+        // The `notified()` future is created *before* checking the condition so
+        // that any `notify_one()` fired between the check and the `.await` is
+        // not lost.
+        loop {
+            let notified = notify.notified();
 
-                if seen_this_round.insert(name.clone()) {
-                    names_to_fetch.push(name.clone());
-                }
-                to_process.push((name, range_str, dep_chain));
+            if let Some(err) = state.error_slot.lock().unwrap().take() {
+                return Err(err);
             }
 
-            if to_process.is_empty() {
+            if pending.load(Ordering::SeqCst) == 0 {
                 break;
             }
 
-            // Speculative pre-warm: on the first BFS level, piggyback
-            // common transitive deps into the fetch batch so they are
-            // cached before BFS naturally reaches them.
-            if is_first_bfs_level {
-                is_first_bfs_level = false;
-                for &spec_name in SPECULATIVE_DEPS {
-                    if !visited.contains(spec_name)
-                        && !seen_this_round.contains(spec_name)
-                    {
-                        names_to_fetch
-                            .push(spec_name.to_string());
-                    }
-                }
-            }
-
-            // --- Phase 2: parallel metadata prefetch ---
-            let fetch_futs: Vec<_> = names_to_fetch
-                .iter()
-                .map(|n| {
-                    let name = n.clone();
-                    let reg = registry;
-                    async move {
-                        let result =
-                            reg.get_package_metadata(&name).await;
-                        (name, result)
-                    }
-                })
-                .collect();
-
-            let fetch_results =
-                futures::future::join_all(fetch_futs).await;
-
-            // Populate cache (registry already caches internally,
-            // but handle errors here)
-            let mut fetch_errors: HashMap<String, String> =
-                HashMap::new();
-            for (name, result) in &fetch_results {
-                if let Err(e) = result {
-                    fetch_errors
-                        .insert(name.clone(), e.to_string());
-                }
-            }
-
-            // --- Phase 3: process fetched metadata ---
-            let mut next_queue: Vec<(String, String, Vec<String>)> =
-                Vec::new();
-
-            for (name, range_str, dep_chain) in to_process {
-                // Re-check (another entry in same batch may have
-                // resolved it)
-                if resolved.contains_key(&name) || visited.contains(&name) {
-                    continue;
-                }
-                visited.insert(name.clone());
-
-                // Check fetch errors
-                if let Some(err) = fetch_errors.get(&name) {
-                    if dep_chain.len() > 1 {
+            notified.await;
+        }
+
+        // Final check: a fatal error might have arrived concurrently with
+        // pending reaching 0.
+        if let Some(err) = state.error_slot.lock().unwrap().take() {
+            return Err(err);
+        }
+
+        let result = state
+            .resolved
+            .iter()
+            .map(|e| (e.key().clone(), e.value().clone()))
+            .collect();
+
+        Ok(result)
+    }
+}
+
+impl Default for DependencyResolver {
+    fn default() -> Self {
+        Self::new()
+    }
+}
+
+/// Decrement the pending counter and wake the main task if it reaches zero.
+#[inline]
+fn decrement_pending(pending: &Arc<AtomicUsize>, notify: &Arc<Notify>) {
+    let prev = pending.fetch_sub(1, Ordering::SeqCst);
+    if prev == 1 {
+        notify.notify_one();
+    }
+}
+
+/// Atomically claim a package for resolution using DashMap's entry API.
+///
+/// Returns `true` if this call claimed the package (the caller should proceed
+/// with resolution). Returns `false` if another task already claimed it.
+///
+/// This is a **sync** function so that no DashMap shard guard ever crosses an
+/// `await` point — a guard held inside an `async fn` would make the resulting
+/// future non-`Send` and break `tokio::spawn`.
+fn try_claim_package(visited: &DashMap<String, ()>, name: &str) -> bool {
+    match visited.entry(name.to_string()) {
+        dashmap::Entry::Occupied(_) => false,
+        dashmap::Entry::Vacant(e) => {
+            e.insert(());
+            true
+        }
+    }
+}
+
+/// Async task that resolves a single package and immediately spawns tasks for
+/// its transitive dependencies.
+///
+/// Calls `try_claim_package` (sync) to atomically claim the package before any
+/// `await` — only one concurrent task will process a given package name; others
+/// detect the existing entry and return after optionally emitting a
+/// version-conflict warning.
+fn stream_resolve_package(
+    state: ResolverState,
+    name: String,
+    range_str: String,
+    dep_chain: Vec<String>,
+) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>> {
+    Box::pin(async move {
+    // Apply override: forced version range takes precedence.
+    let range_str = state
+        .overrides
+        .get(&name)
+        .cloned()
+        .unwrap_or(range_str);
+
+    // Atomically claim this package (sync — no guard crosses an await).
+    // If already claimed, emit a version-conflict warning and bail out.
+    if !try_claim_package(&state.visited, &name) {
+        // Another task is processing (or has processed) this package.
+        // Emit a version-conflict warning if the resolved version does not
+        // satisfy our requested range.
+        if let Some(existing) = state.resolved.get(&name) {
+            if let Ok(reqs) = parse_all_version_ranges(&range_str) {
+                if let Ok(ev) = Version::parse(&existing.version) {
+                    let satisfied = reqs.iter().any(|r| r.matches(&ev));
+                    if !satisfied {
                         tracing::warn!(
-                            "Skipping {}: {} (transitive)",
-                            name, err
+                            "Version conflict '{}': \
+                             using {}@{}, wanted '{}' by {}",
+                            name,
+                            name,
+                            existing.version,
+                            range_str,
+                            dep_chain
+                                .last()
+                                .unwrap_or(&"(root)".to_string())
                         );
-                        continue;
                     }
-                    anyhow::bail!(
+                }
+            }
+        }
+        decrement_pending(&state.pending, &state.notify);
+        return;
+    }
+
+    // Fetch registry metadata (hits in-memory cache if already warm).
+    let metadata = match state.registry.get_package_metadata(&name).await {
+        Ok(m) => m,
+        Err(e) => {
+            if dep_chain.len() > 1 {
+                tracing::warn!("Skipping {}: {} (transitive)", name, e);
+            } else {
+                let mut slot = state.error_slot.lock().unwrap();
+                if slot.is_none() {
+                    *slot = Some(anyhow::anyhow!(
                         "Failed to fetch metadata for '{}': {}",
                         name,
-                        err
-                    );
-                }
-
-                let metadata = registry
-                    .get_package_metadata(&name)
-                    .await?; // hits cache
-
-                let reqs = parse_all_version_ranges(&range_str)?;
-                let best_version = match find_best_version(
-                    &metadata.versions,
-                    &reqs,
-                    &name,
-                ) {
-                    Ok(v) => v,
-                    Err(e) => {
-                        if dep_chain.len() > 1 {
-                            tracing::warn!(
-                                "Skipping {}: {} (transitive)",
-                                name, e
-                            );
-                            continue;
-                        }
-                        return Err(e);
-                    }
-                };
-
-                let version_meta = match metadata
-                    .versions
-                    .get(&best_version)
-                {
-                    Some(v) => v,
-                    None => continue,
-                };
-
-                if should_skip_optional(version_meta) {
-                    tracing::debug!(
-                        "Skipping {} (platform mismatch)",
-                        name
-                    );
-                    continue;
-                }
-
-                let pkg_deps = version_meta
-                    .dependencies
-                    .clone()
-                    .unwrap_or_default();
-                let peer_deps = version_meta
-                    .peer_dependencies
-                    .clone()
-                    .unwrap_or_default();
-                let optional_deps = version_meta
-                    .optional_dependencies
-                    .clone()
-                    .unwrap_or_default();
-
-                let bin =
-                    resolve_bin_field(&name, &version_meta.bin);
-
-                let has_install_script = version_meta
-                    .scripts
-                    .as_ref()
-                    .map(|s| {
-                        s.contains_key("preinstall")
-                            || s.contains_key("install")
-                            || s.contains_key("postinstall")
-                    })
-                    .unwrap_or(false);
-
-                let resolved_pkg = ResolvedPackage {
-                    name: name.clone(),
-                    version: best_version.clone(),
-                    tarball_url: version_meta
-                        .dist
-                        .tarball
-                        .clone(),
-                    shasum: version_meta
-                        .dist
-                        .shasum
-                        .clone(),
-                    integrity: version_meta
-                        .dist
-                        .integrity
-                        .clone(),
-                    dependencies: pkg_deps.clone(),
-                    peer_dependencies: peer_deps.clone(),
-                    bin,
-                    has_install_script,
-                    nested_in: None,
-                };
-
-                // R7: notify background downloader immediately so tarball
-                // fetches can overlap with ongoing resolution.
-                if let Some(tx) = &prefetch_tx {
-                    let _ = tx.send(resolved_pkg.clone());
-                }
-
-                resolved.insert(name.clone(), resolved_pkg);
-
-                let mut child_chain = dep_chain;
-                child_chain.push(format!(
-                    "{}@{}",
-                    name, best_version
-                ));
-
-                for (dn, dr) in &pkg_deps {
-                    let (rn, rr) = resolve_alias(dn, dr);
-                    next_queue.push((
-                        rn,
-                        rr,
-                        child_chain.clone(),
+                        e
                     ));
+                    state.notify.notify_one();
                 }
-                for (dn, dr) in &peer_deps {
-                    let (rn, rr) = resolve_alias(dn, dr);
-                    if !resolved.contains_key(&rn) {
-                        next_queue.push((
-                            rn,
-                            rr,
-                            child_chain.clone(),
-                        ));
-                    }
-                }
-                for (dn, dr) in &optional_deps {
-                    let (rn, rr) = resolve_alias(dn, dr);
-                    if !resolved.contains_key(&rn) {
-                        next_queue.push((
-                            rn,
-                            rr,
-                            child_chain.clone(),
-                        ));
+            }
+            decrement_pending(&state.pending, &state.notify);
+            return;
+        }
+    };
+
+    // Parse version range (with OR support).
+    let reqs = match parse_all_version_ranges(&range_str) {
+        Ok(r) => r,
+        Err(e) => {
+            tracing::warn!("Bad version range for {}: {}", name, e);
+            decrement_pending(&state.pending, &state.notify);
+            return;
+        }
+    };
+
+    // Select the best matching version.
+    let best_version =
+        match find_best_version(&metadata.versions, &reqs, &name) {
+            Ok(v) => v,
+            Err(e) => {
+                if dep_chain.len() > 1 {
+                    tracing::warn!(
+                        "Skipping {}: {} (transitive)",
+                        name,
+                        e
+                    );
+                } else {
+                    let mut slot = state.error_slot.lock().unwrap();
+                    if slot.is_none() {
+                        *slot = Some(e);
+                        state.notify.notify_one();
                     }
                 }
+                decrement_pending(&state.pending, &state.notify);
+                return;
             }
+        };
 
-            queue = next_queue;
-        }
+    // Retrieve version metadata (clone to avoid lifetime issues in async).
+    let version_meta =
+        match metadata.versions.get(&best_version).cloned() {
+            Some(v) => v,
+            None => {
+                decrement_pending(&state.pending, &state.notify);
+                return;
+            }
+        };
+
+    // Skip platform-incompatible optional packages.
+    if should_skip_optional(&version_meta) {
+        tracing::debug!("Skipping {} (platform mismatch)", name);
+        decrement_pending(&state.pending, &state.notify);
+        return;
+    }
+
+    let pkg_deps = version_meta.dependencies.clone().unwrap_or_default();
+    let peer_deps =
+        version_meta.peer_dependencies.clone().unwrap_or_default();
+    let optional_deps =
+        version_meta.optional_dependencies.clone().unwrap_or_default();
 
-        Ok(resolved)
+    let bin = resolve_bin_field(&name, &version_meta.bin);
+    let has_install_script = version_meta
+        .scripts
+        .as_ref()
+        .map(|s| {
+            s.contains_key("preinstall")
+                || s.contains_key("install")
+                || s.contains_key("postinstall")
+        })
+        .unwrap_or(false);
+
+    let resolved_pkg = ResolvedPackage {
+        name: name.clone(),
+        version: best_version.clone(),
+        tarball_url: version_meta.dist.tarball.clone(),
+        shasum: version_meta.dist.shasum.clone(),
+        integrity: version_meta.dist.integrity.clone(),
+        dependencies: pkg_deps.clone(),
+        peer_dependencies: peer_deps.clone(),
+        bin,
+        has_install_script,
+        nested_in: None,
+    };
+
+    // Notify background downloader immediately (overlapping tarball fetch).
+    if let Some(tx) = state.prefetch_tx.as_ref() {
+        let _ = tx.send(resolved_pkg.clone());
+    }
+
+    state.resolved.insert(name.clone(), resolved_pkg);
+
+    // Build dep chain for children.
+    let mut child_chain = dep_chain;
+    child_chain.push(format!("{}@{}", name, best_version));
+
+    // Collect child tasks:
+    // - Regular deps: always spawn (visited check inside the task).
+    // - Peer/optional deps: skip if already resolved (same as BFS behavior).
+    let mut children: Vec<(String, String)> = pkg_deps
+        .iter()
+        .map(|(dn, dr)| resolve_alias(dn, dr))
+        .collect();
+
+    for (dn, dr) in &peer_deps {
+        let (rn, rr) = resolve_alias(dn, dr);
+        if !state.resolved.contains_key(&rn) {
+            children.push((rn, rr));
+        }
+    }
+    for (dn, dr) in &optional_deps {
+        let (rn, rr) = resolve_alias(dn, dr);
+        if !state.resolved.contains_key(&rn) {
+            children.push((rn, rr));
+        }
     }
-}
 
-impl Default for DependencyResolver {
-    fn default() -> Self {
-        Self::new()
+    // Increment pending for ALL children before spawning any, so that the
+    // main loop never observes pending == 0 prematurely.
+    if !children.is_empty() {
+        state.pending.fetch_add(children.len(), Ordering::SeqCst);
+        for (child_name, child_range) in children {
+            let child_state = state.clone();
+            let chain = child_chain.clone();
+            tokio::spawn(async move {
+                stream_resolve_package(
+                    child_state,
+                    child_name,
+                    child_range,
+                    chain,
+                )
+                .await;
+            });
+        }
     }
+
+    // This task is complete.
+    decrement_pending(&state.pending, &state.notify);
+    }) // end Box::pin(async move { ... })
 }
 
 /// Resolve npm: alias protocol and bare package name aliases.

```

## Review: stream-resolver-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: stream-resolver

**Summary**: Stream-based rewrite matches all requirements: each resolved package immediately spawns tokio::spawn tasks for transitive deps without waiting for BFS level siblings; DashMap<String,ResolvedPackage> shared resolved map; AtomicUsize pending counter + Notify wakeup correctly signals completion; try_claim_package uses sync DashMap entry API (no shard guard crosses await, ensuring Send futures); version conflict detection preserved via existing/range check on claim failure; override support preserved at task entry; RegistryClient derives Clone for multi-task sharing.


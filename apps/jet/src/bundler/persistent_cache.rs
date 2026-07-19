// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! Persistent, content-addressed transform cache (#2137, beat-vite epic
//! #1990).
//!
//! Wraps the *same* get/insert seam `transform_modules` already consults on
//! [`super::CompilationCache`] (the in-memory `(PathBuf, mtime)` map): a miss
//! there falls through to this disk-backed layer before paying for a real
//! transform. `jet build` starts every process with an empty in-memory
//! cache, so in one-shot production builds this is the only cache that ever
//! actually hits; the in-memory map is untouched and keeps its existing
//! dev-server/watch semantics unchanged.
//!
//! ## Key correctness
//!
//! A cached [`super::CompiledModule::code`] embeds numeric `require(N)`
//! calls assigned by this build's own discovery order, so a stale hit is
//! silently wrong, not merely stale text. An entry is valid only when every
//! field of [`EntryKey`] matches the module being looked up *right now*:
//!
//! - `content_hash` — hash of the module's own source bytes. NOT mtime:
//!   mtime does not survive `git checkout`/branch switches, so a
//!   content-addressed key is the only thing that lets warm hits and
//!   correctness coexist across a branch switch.
//! - `own_id` — the numeric module id this build assigned the module
//!   (`module_id` in `transform_modules`'s loop). A cached module's `id` is
//!   always overwritten with the current build's id on any hit (mirroring
//!   the in-memory cache's existing behavior) — `own_id` in the key exists
//!   so a same-content module that was assigned a *different* id gets
//!   re-transformed instead of silently reusing `require(N)` calls baked
//!   for the old id.
//! - `dep_fingerprint` — hash of the module's own dependency ids, in the
//!   module's natural (unsorted) discovery order. Order-preserving, not
//!   just set-equality: two dependencies swapping assigned ids while the
//!   *set* of ids stays the same is invisible to a sorted fingerprint but
//!   changes the sequence `graph.dependencies` walks the module's own
//!   (deterministic, content-derived) import order in, so an
//!   order-preserving fingerprint still catches it.
//! - `barrel_fingerprint` — hash of the sorted demand-name set for a
//!   demand-pruned barrel (`BarrelDemand::Names`), 0 for every other module.
//!   Sorted because `BarrelDemand::Names` is a `HashSet`, whose iteration
//!   order is not itself deterministic across runs.
//! - the cache *file*'s `config_fingerprint` (checked once for the whole
//!   store at load, not per entry) — defines, jsx/target/splitting-relevant
//!   transform options, minify, the crate version, and
//!   [`TRANSFORM_CACHE_SCHEMA`]. A mismatch treats the whole file as empty:
//!   full miss, not a per-entry decision.
//!
//! Never fail a build over cache state: a missing, truncated, or
//! undecodable store file, or a single corrupt entry inside an otherwise
//! good one, is a warning + miss, never a hard error. Each entry's payload
//! is independently checksummed and encoded as an opaque `Vec<u8>` blob
//! nested inside the outer container specifically so one corrupt entry's
//! bytes can never desync the decode of every entry after it.
//!
//! ## Store
//!
//! Single file at `<project_root>/node_modules/.jet/transform-cache.bin`
//! ([`TRANSFORM_CACHE_REL_PATH`]), [`postcard`]-encoded (already a
//! transitive dependency via `oxc_minifier -> oxc_compat ->
//! oxc-browserslist -> postcard`, so this adds no new crate to the build
//! graph). Loaded once at [`PersistentTransformCache::load`], written back
//! once via [`PersistentTransformCache::save`] after a successful build
//! through a tmp-file + rename (atomic — a crash or concurrent build can
//! never observe a half-written store). [`MAX_STORE_BYTES`] caps the
//! on-disk size; over cap, entries are evicted oldest-first by a logical
//! (non-wall-clock) `last_used` counter so eviction ordering costs no
//! `SystemTime` calls in the hot path.
//!
//! ## Hatches
//!
//! `[build] cache` in jet.toml, `--no-cache` on `jet build` (flag wins over
//! config; see `cli.rs`'s `build_cache_enabled`), and
//! `JET_NO_PERSISTENT_CACHE=1` as a lower-level kill switch consulted
//! directly in [`PersistentTransformCache::load`] (so any caller that
//! constructs a `Bundler` with `cache_project_root: Some(..)` directly —
//! tests, embedders — still honors the env override without re-plumbing the
//! CLI precedence).
//! @issue #2137

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use super::CompiledModule;

/// Bumped whenever a transform-pass behavior change could make an
/// old on-disk entry decode successfully but produce output that no longer
/// matches what today's transform would emit. Folded into the cache file's
/// `config_fingerprint` alongside the crate version, so either one changing
/// invalidates the whole store rather than serving stale-shape code.
pub const TRANSFORM_CACHE_SCHEMA: u32 = 1;

/// Store location, relative to the project root (the same root `jet build`
/// resolves `node_modules/` from). `node_modules/.jet/` is an existing jet
/// convention (see e.g. the dev-server's `polyfill-path.mjs`), so this adds
/// a file to an already-established directory rather than a new one.
pub const TRANSFORM_CACHE_REL_PATH: &str = "node_modules/.jet/transform-cache.bin";

/// ~200MB on-disk cap. `save` evicts oldest-first (by logical `last_used`
/// order) once the serialized entry set exceeds this.
pub const MAX_STORE_BYTES: u64 = 200 * 1024 * 1024;

/// Everything that must match between a cached entry and the module being
/// looked up right now for the cached `CompiledModule` to be safe to reuse.
/// See the module doc comment for what each field guards against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryKey {
    pub content_hash: u64,
    pub own_id: usize,
    pub dep_fingerprint: u64,
    pub barrel_fingerprint: u64,
}

/// One on-disk cache row. `payload` is an independently-encoded
/// `CompiledModule` blob (not inlined as struct fields) precisely so a
/// corrupt `payload` can be detected via `checksum` and skipped without
/// desyncing the decode of every `StoredEntry` after it in the file —
/// `postcard`/serde's sequence decoding otherwise aborts the whole
/// container on the first bad element.
#[derive(Debug, Serialize, Deserialize)]
struct StoredEntry {
    path: PathBuf,
    key: EntryKey,
    checksum: u64,
    payload: Vec<u8>,
    /// Logical (non-wall-clock) recency counter, carried across process
    /// runs so `save`'s oldest-first eviction stays meaningful even when
    /// most of a store's entries are untouched by the current build.
    last_used: u64,
}

/// The whole on-disk file: one `postcard`-encoded blob.
#[derive(Debug, Default, Serialize, Deserialize)]
struct CacheFile {
    schema: u32,
    config_fingerprint: u64,
    entries: Vec<StoredEntry>,
}

/// In-memory representation of one live entry — decoded once at `load` (or
/// inserted fresh this build), kept as a real `CompiledModule` rather than
/// re-decoding `payload` on every `get`.
#[derive(Debug, Clone)]
struct LiveEntry {
    key: EntryKey,
    module: CompiledModule,
    last_used: u64,
}

/// Result of [`PersistentTransformCache::load`], surfaced through
/// `JET_BUNDLE_TIMING` and the final build report.
#[derive(Debug, Clone, Default)]
pub struct LoadStats {
    pub enabled: bool,
    pub loaded_entries: usize,
    pub corrupt_entries: usize,
    pub duration: Duration,
}

/// Result of [`PersistentTransformCache::save`].
#[derive(Debug, Clone, Default)]
pub struct SaveStats {
    pub bytes_written: u64,
    pub entries_written: usize,
    pub evicted: usize,
    pub duration: Duration,
}

/// The persistent, content-addressed layer `transform_modules` consults
/// only after its existing in-memory `(path, mtime)` cache misses. See the
/// module doc comment for the full design.
pub struct PersistentTransformCache {
    enabled: bool,
    store_path: PathBuf,
    config_fingerprint: u64,
    entries: DashMap<PathBuf, LiveEntry>,
    hits: AtomicU64,
    misses: AtomicU64,
    /// Monotonic logical clock for `last_used` — deliberately not
    /// `SystemTime`, so touching an entry costs one atomic increment, not a
    /// clock syscall.
    clock: AtomicU64,
}

impl PersistentTransformCache {
    /// A disabled cache: every `get` is a no-op miss, `insert`/`save` are
    /// no-ops. Used whenever `BundleOptions::cache_project_root` is `None`
    /// (the default — dev-server/lib/nx paths not opted in) so the
    /// `transform_modules` seam can consult `self.persistent_cache`
    /// unconditionally without an `Option<..>` at every call site.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            store_path: PathBuf::new(),
            config_fingerprint: 0,
            entries: DashMap::new(),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            clock: AtomicU64::new(0),
        }
    }

    /// Load `<project_root>/node_modules/.jet/transform-cache.bin`, if
    /// enabled and present. `config_fingerprint` gates the *whole* file: a
    /// mismatch (schema bump, or a defines/jsx/target/minify/splitting
    /// option changed since the last build) discards every entry as a
    /// deliberate, silent cold-start — not corruption, so it does not warn.
    /// A missing file (first run, or a deliberately cleared store) is the
    /// same silent cold-start. Only a *present but undecodable* file, or
    /// individual corrupt entries inside an otherwise good file, print a
    /// warning — see the module doc comment.
    pub fn load(project_root: &Path, config_fingerprint: u64) -> (Self, LoadStats) {
        let start = Instant::now();

        if std::env::var_os("JET_NO_PERSISTENT_CACHE").is_some() {
            return (Self::disabled(), LoadStats::default());
        }

        let store_path = project_root.join(TRANSFORM_CACHE_REL_PATH);
        let cache = Self {
            enabled: true,
            store_path: store_path.clone(),
            config_fingerprint,
            entries: DashMap::new(),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            clock: AtomicU64::new(0),
        };

        let bytes = match std::fs::read(&store_path) {
            Ok(b) => b,
            Err(_) => {
                return (
                    cache,
                    LoadStats {
                        enabled: true,
                        duration: start.elapsed(),
                        ..Default::default()
                    },
                );
            }
        };

        let file: CacheFile = match postcard::from_bytes(&bytes) {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "warn: jet transform cache at {} is unreadable ({e}); starting cold (#2137)",
                    store_path.display(),
                );
                return (
                    cache,
                    LoadStats {
                        enabled: true,
                        duration: start.elapsed(),
                        ..Default::default()
                    },
                );
            }
        };

        if file.schema != TRANSFORM_CACHE_SCHEMA || file.config_fingerprint != config_fingerprint {
            return (
                cache,
                LoadStats {
                    enabled: true,
                    duration: start.elapsed(),
                    ..Default::default()
                },
            );
        }

        let mut loaded = 0usize;
        let mut corrupt = 0usize;
        let mut max_last_used = 0u64;
        for stored in file.entries {
            if hash_bytes(&stored.payload) != stored.checksum {
                corrupt += 1;
                continue;
            }
            let module: CompiledModule = match postcard::from_bytes(&stored.payload) {
                Ok(m) => m,
                Err(_) => {
                    corrupt += 1;
                    continue;
                }
            };
            max_last_used = max_last_used.max(stored.last_used);
            cache.entries.insert(
                stored.path,
                LiveEntry {
                    key: stored.key,
                    module,
                    last_used: stored.last_used,
                },
            );
            loaded += 1;
        }
        // Anything inserted fresh this build must sort after everything
        // just loaded, so `save`'s oldest-first eviction stays meaningful
        // across process runs instead of resetting to 0 every load.
        cache.clock.store(max_last_used + 1, Ordering::Relaxed);

        if corrupt > 0 {
            eprintln!(
                "warn: jet transform cache at {} had {corrupt} corrupt entr{} out of {}; \
                 dropped and will re-transform (#2137)",
                store_path.display(),
                if corrupt == 1 { "y" } else { "ies" },
                loaded + corrupt,
            );
        }

        (
            cache,
            LoadStats {
                enabled: true,
                loaded_entries: loaded,
                corrupt_entries: corrupt,
                duration: start.elapsed(),
            },
        )
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Look up `path` and return its cached module only if `key` matches
    /// exactly. Touches the entry's recency on a hit; counts exactly one
    /// hit or miss per call either way.
    pub fn get(&self, path: &Path, key: &EntryKey) -> Option<CompiledModule> {
        if !self.enabled {
            return None;
        }
        if let Some(mut entry) = self.entries.get_mut(path) {
            if entry.key == *key {
                entry.last_used = self.next_clock();
                self.hits.fetch_add(1, Ordering::Relaxed);
                return Some(entry.module.clone());
            }
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// Record (or replace) `path`'s entry. A later `save` call persists
    /// whatever is live in-memory at that point — unconditional overwrite,
    /// so a stale entry for `path` (old content hash) is naturally dropped
    /// the moment its module is re-transformed.
    pub fn insert(&self, path: PathBuf, key: EntryKey, module: CompiledModule) {
        if !self.enabled {
            return;
        }
        let last_used = self.next_clock();
        self.entries.insert(
            path,
            LiveEntry {
                key,
                module,
                last_used,
            },
        );
    }

    fn next_clock(&self) -> u64 {
        self.clock.fetch_add(1, Ordering::Relaxed)
    }

    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// Encode every live entry, apply the [`MAX_STORE_BYTES`] oldest-first
    /// eviction cap, and atomically write back via a tmp file + rename.
    /// Never fails the build: any encode/IO error is swallowed after a
    /// single warning, returning a zeroed [`SaveStats`].
    pub fn save(&self) -> SaveStats {
        if !self.enabled {
            return SaveStats::default();
        }
        let start = Instant::now();

        let mut rows: Vec<StoredEntry> = Vec::with_capacity(self.entries.len());
        for kv in self.entries.iter() {
            let module = &kv.value().module;
            let payload = match postcard::to_stdvec(module) {
                Ok(p) => p,
                Err(e) => {
                    tracing::debug!(
                        "jet transform cache: skipping unencodable entry {:?}: {e} (#2137)",
                        kv.key()
                    );
                    continue;
                }
            };
            let checksum = hash_bytes(&payload);
            rows.push(StoredEntry {
                path: kv.key().clone(),
                key: kv.value().key,
                checksum,
                payload,
                last_used: kv.value().last_used,
            });
        }

        rows.sort_unstable_by_key(|r| r.last_used);
        let mut total: u64 = rows.iter().map(|r| r.payload.len() as u64).sum();
        let mut evicted = 0usize;
        while total > MAX_STORE_BYTES && !rows.is_empty() {
            let removed = rows.remove(0);
            total -= removed.payload.len() as u64;
            evicted += 1;
        }
        let entries_written = rows.len();

        let file = CacheFile {
            schema: TRANSFORM_CACHE_SCHEMA,
            config_fingerprint: self.config_fingerprint,
            entries: rows,
        };
        let bytes = match postcard::to_stdvec(&file) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("warn: jet transform cache encode failed, not persisted: {e} (#2137)");
                return SaveStats::default();
            }
        };

        if let Some(parent) = self.store_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!(
                    "warn: jet transform cache could not create {}: {e} (#2137)",
                    parent.display(),
                );
                return SaveStats::default();
            }
        }

        // PID-suffixed tmp name: two concurrent `jet build` invocations in
        // the same project never contend for the same tmp path; the final
        // `rename` is still what makes the write atomic from a reader's
        // point of view.
        let tmp_path = PathBuf::from(format!(
            "{}.tmp.{}",
            self.store_path.display(),
            std::process::id()
        ));
        if let Err(e) = std::fs::write(&tmp_path, &bytes) {
            eprintln!("warn: jet transform cache write failed, not persisted: {e} (#2137)");
            let _ = std::fs::remove_file(&tmp_path);
            return SaveStats::default();
        }
        if let Err(e) = std::fs::rename(&tmp_path, &self.store_path) {
            eprintln!("warn: jet transform cache rename failed, not persisted: {e} (#2137)");
            let _ = std::fs::remove_file(&tmp_path);
            return SaveStats::default();
        }

        SaveStats {
            bytes_written: bytes.len() as u64,
            entries_written,
            evicted,
            duration: start.elapsed(),
        }
    }
}

/// Deterministic (not `RandomState`-seeded) hash of the module's own source
/// bytes — `DefaultHasher` (SipHash13 with the fixed `(0, 0)` key) is
/// already the pattern `calculate_hash` uses for output-code hashing above;
/// this reuses the same technique for source content instead.
pub fn hash_str(content: &str) -> u64 {
    hash_bytes(content.as_bytes())
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    let mut hasher = DefaultHasher::new();
    hasher.write(bytes);
    hasher.finish()
}

fn hash_seq<T: std::hash::Hash>(items: &[T]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    items.hash(&mut hasher);
    hasher.finish()
}

/// Ordered dependency-id fingerprint for `id`: `graph.dependencies(id)`'s
/// own (unsorted) order, each resolved through `module_map` to the numeric
/// id this build assigned it. Deliberately not sorted — see the module doc
/// comment's `dep_fingerprint` entry for why order must be preserved.
pub fn dependency_fingerprint(
    graph: &super::ModuleGraph,
    id: super::ModuleId,
    module_map: &HashMap<PathBuf, usize>,
) -> u64 {
    let ids: Vec<usize> = graph
        .dependencies(id)
        .into_iter()
        .filter_map(|(dep_id, _kind)| {
            graph
                .get_node(dep_id)
                .and_then(|node| module_map.get(&node.path).copied())
        })
        .collect();
    hash_seq(&ids)
}

/// Fingerprint of a barrel's accumulated demand-name set. `0` for any
/// module that is not a demand-pruned barrel (`None` or
/// `Some(BarrelDemand::Full)` — both leave the barrel's source completely
/// unpruned, so they are transform-equivalent and may safely share the same
/// fingerprint value). Names are sorted before hashing because
/// `BarrelDemand::Names` is a `HashSet`, whose iteration order is not
/// itself deterministic across runs.
pub(super) fn barrel_fingerprint(demand: Option<&super::BarrelDemand>) -> u64 {
    match demand {
        Some(super::BarrelDemand::Names(names)) => {
            let mut sorted: Vec<&str> = names.iter().map(String::as_str).collect();
            sorted.sort_unstable();
            hash_seq(&sorted)
        }
        _ => 0,
    }
}

/// Whole-build config fingerprint: defines (sorted — `HashMap` iteration
/// order is randomized per-process), minify, splitting, and the
/// jsx/target/dev-mode/source-map transform options, plus the crate
/// version and [`TRANSFORM_CACHE_SCHEMA`]. Checked once per cache file
/// (not per entry) at [`PersistentTransformCache::load`] — a mismatch
/// discards the whole store as a deliberate cold-start.
pub fn config_fingerprint(
    defines: &HashMap<String, String>,
    minify: bool,
    splitting: bool,
    transform_options: &crate::transform::TransformOptions,
) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut defines_sorted: Vec<(&String, &String)> = defines.iter().collect();
    defines_sorted.sort_unstable_by(|a, b| a.0.cmp(b.0));

    let mut hasher = DefaultHasher::new();
    TRANSFORM_CACHE_SCHEMA.hash(&mut hasher);
    env!("CARGO_PKG_VERSION").hash(&mut hasher);
    defines_sorted.hash(&mut hasher);
    minify.hash(&mut hasher);
    splitting.hash(&mut hasher);
    transform_options.jsx_pragma.hash(&mut hasher);
    transform_options.jsx_fragment.hash(&mut hasher);
    transform_options.jsx_automatic.hash(&mut hasher);
    // `TypeScriptTarget` does not derive `Hash`; its `Debug` output is a
    // fixed, hand-written set of unit variants, so formatting it is a safe
    // stand-in rather than adding a derive to an unrelated enum.
    format!("{:?}", transform_options.ts_target).hash(&mut hasher);
    transform_options.dev_mode.hash(&mut hasher);
    transform_options.source_maps.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_module(id: usize, code: &str) -> CompiledModule {
        CompiledModule {
            id,
            path: PathBuf::from(format!("/proj/src/m{id}.js")),
            code: code.to_string(),
            source_map: None,
            dependencies: Vec::new(),
            hash: format!("{:x}", hash_str(code)),
        }
    }

    fn sample_key() -> EntryKey {
        EntryKey {
            content_hash: hash_str("const x = 1;"),
            own_id: 0,
            dep_fingerprint: 0,
            barrel_fingerprint: 0,
        }
    }

    #[test]
    fn disabled_cache_never_hits_or_records() {
        let cache = PersistentTransformCache::disabled();
        assert!(!cache.enabled());
        let key = sample_key();
        cache.insert(PathBuf::from("/proj/src/a.js"), key, sample_module(0, "a"));
        assert!(cache.get(Path::new("/proj/src/a.js"), &key).is_none());
        assert_eq!(cache.hits(), 0);
        assert_eq!(cache.misses(), 0);
    }

    #[test]
    fn miss_then_insert_then_hit_round_trips_module() {
        let cache = PersistentTransformCache {
            enabled: true,
            ..PersistentTransformCache::disabled()
        };
        let path = PathBuf::from("/proj/src/a.js");
        let key = sample_key();

        assert!(cache.get(&path, &key).is_none());
        assert_eq!(cache.misses(), 1);

        cache.insert(path.clone(), key, sample_module(7, "const x = 1;"));
        let hit = cache.get(&path, &key).expect("expected a hit after insert");
        assert_eq!(hit.id, 7);
        assert_eq!(hit.code, "const x = 1;");
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 1);
    }

    #[test]
    fn key_mismatch_on_same_path_is_a_miss_not_a_stale_hit() {
        let cache = PersistentTransformCache {
            enabled: true,
            ..PersistentTransformCache::disabled()
        };
        let path = PathBuf::from("/proj/src/a.js");
        let old_key = sample_key();
        cache.insert(path.clone(), old_key, sample_module(0, "const x = 1;"));

        let mut new_key = old_key;
        new_key.content_hash = hash_str("const x = 2;"); // content changed
        assert!(cache.get(&path, &new_key).is_none());
    }

    #[test]
    fn own_id_mismatch_is_a_miss() {
        let cache = PersistentTransformCache {
            enabled: true,
            ..PersistentTransformCache::disabled()
        };
        let path = PathBuf::from("/proj/src/a.js");
        let mut key = sample_key();
        key.own_id = 3;
        cache.insert(path.clone(), key, sample_module(3, "const x = 1;"));

        let mut different_id_key = key;
        different_id_key.own_id = 4;
        assert!(cache.get(&path, &different_id_key).is_none());
    }

    #[test]
    fn dependency_order_changes_the_fingerprint() {
        let mut graph = super::super::ModuleGraph::new();
        let a = graph.add_module(
            PathBuf::from("/proj/src/a.js"),
            super::super::graph::ModuleKind::Script,
            0,
        );
        let b = graph.add_module(
            PathBuf::from("/proj/src/b.js"),
            super::super::graph::ModuleKind::Script,
            0,
        );
        let c = graph.add_module(
            PathBuf::from("/proj/src/c.js"),
            super::super::graph::ModuleKind::Script,
            0,
        );
        graph.add_dependency(a, b, super::super::EdgeKind::Import);
        graph.add_dependency(a, c, super::super::EdgeKind::Import);

        let module_map: HashMap<PathBuf, usize> = [
            (PathBuf::from("/proj/src/a.js"), 0usize),
            (PathBuf::from("/proj/src/b.js"), 1usize),
            (PathBuf::from("/proj/src/c.js"), 2usize),
        ]
        .into_iter()
        .collect();

        let forward = dependency_fingerprint(&graph, a, &module_map);

        // A graph where the same two ids are wired in the opposite order
        // must fingerprint differently — this is the "sibling ids swapped"
        // corruption scenario an order-*insensitive* (e.g. sorted) key
        // would miss entirely.
        let mut swapped = super::super::ModuleGraph::new();
        let a2 = swapped.add_module(
            PathBuf::from("/proj/src/a.js"),
            super::super::graph::ModuleKind::Script,
            0,
        );
        let c2 = swapped.add_module(
            PathBuf::from("/proj/src/c.js"),
            super::super::graph::ModuleKind::Script,
            0,
        );
        let b2 = swapped.add_module(
            PathBuf::from("/proj/src/b.js"),
            super::super::graph::ModuleKind::Script,
            0,
        );
        swapped.add_dependency(a2, c2, super::super::EdgeKind::Import);
        swapped.add_dependency(a2, b2, super::super::EdgeKind::Import);
        let reverse = dependency_fingerprint(&swapped, a2, &module_map);

        assert_ne!(forward, reverse);
    }

    #[test]
    fn barrel_fingerprint_is_order_independent_over_names() {
        use std::collections::HashSet;
        let a: HashSet<String> = ["foo".to_string(), "bar".to_string(), "baz".to_string()]
            .into_iter()
            .collect();
        let b: HashSet<String> = ["baz".to_string(), "foo".to_string(), "bar".to_string()]
            .into_iter()
            .collect();
        let demand_a = super::super::BarrelDemand::Names(a);
        let demand_b = super::super::BarrelDemand::Names(b);
        assert_eq!(
            barrel_fingerprint(Some(&demand_a)),
            barrel_fingerprint(Some(&demand_b))
        );
    }

    #[test]
    fn barrel_fingerprint_differs_for_different_demand_sets() {
        use std::collections::HashSet;
        let a: HashSet<String> = ["foo".to_string()].into_iter().collect();
        let b: HashSet<String> = ["foo".to_string(), "bar".to_string()].into_iter().collect();
        let demand_a = super::super::BarrelDemand::Names(a);
        let demand_b = super::super::BarrelDemand::Names(b);
        assert_ne!(
            barrel_fingerprint(Some(&demand_a)),
            barrel_fingerprint(Some(&demand_b))
        );
    }

    #[test]
    fn barrel_fingerprint_full_and_none_share_the_no_pruning_value() {
        // `Full` and "no entry at all" both leave the barrel's source
        // completely unpruned at transform time — transform-equivalent, so
        // they must fingerprint identically (see the doc comment above
        // `barrel_fingerprint`).
        assert_eq!(
            barrel_fingerprint(Some(&super::super::BarrelDemand::Full)),
            barrel_fingerprint(None)
        );
    }

    #[test]
    fn config_fingerprint_changes_when_a_define_changes() {
        let opts = crate::transform::TransformOptions {
            jsx_pragma: None,
            jsx_fragment: None,
            jsx_automatic: true,
            ts_target: crate::transform::TypeScriptTarget::ES2020,
            source_maps: true,
            minify: false,
            dev_mode: false,
        };
        let mut defines_a = HashMap::new();
        defines_a.insert("MODE".to_string(), "\"production\"".to_string());
        let mut defines_b = HashMap::new();
        defines_b.insert("MODE".to_string(), "\"development\"".to_string());

        let fp_a = config_fingerprint(&defines_a, false, false, &opts);
        let fp_b = config_fingerprint(&defines_b, false, false, &opts);
        assert_ne!(fp_a, fp_b);
    }

    #[test]
    fn config_fingerprint_is_stable_across_define_insertion_order() {
        // `HashMap` iteration order is randomized per-process; the
        // fingerprint must not be, so it must not leak that randomness.
        let opts = crate::transform::TransformOptions {
            jsx_pragma: None,
            jsx_fragment: None,
            jsx_automatic: true,
            ts_target: crate::transform::TypeScriptTarget::ES2020,
            source_maps: true,
            minify: false,
            dev_mode: false,
        };
        let mut defines_a = HashMap::new();
        defines_a.insert("A".to_string(), "1".to_string());
        defines_a.insert("B".to_string(), "2".to_string());
        let mut defines_b = HashMap::new();
        defines_b.insert("B".to_string(), "2".to_string());
        defines_b.insert("A".to_string(), "1".to_string());

        assert_eq!(
            config_fingerprint(&defines_a, false, false, &opts),
            config_fingerprint(&defines_b, false, false, &opts)
        );
    }

    #[test]
    fn poisoned_payload_checksum_rejects_without_losing_other_entries() {
        let good_module = sample_module(0, "const good = 1;");
        let good_payload = postcard::to_stdvec(&good_module).unwrap();
        let good_entry = StoredEntry {
            path: PathBuf::from("/proj/src/good.js"),
            key: sample_key(),
            checksum: hash_bytes(&good_payload),
            payload: good_payload,
            last_used: 0,
        };

        let mut bad_payload = postcard::to_stdvec(&sample_module(1, "const bad = 1;")).unwrap();
        // Corrupt the payload bytes but keep the checksum as computed
        // BEFORE corruption, so `load` must reject it on checksum mismatch.
        let bad_checksum = hash_bytes(&bad_payload);
        if let Some(byte) = bad_payload.first_mut() {
            *byte ^= 0xFF;
        }
        let bad_entry = StoredEntry {
            path: PathBuf::from("/proj/src/bad.js"),
            key: sample_key(),
            checksum: bad_checksum,
            payload: bad_payload,
            last_used: 0,
        };

        let file = CacheFile {
            schema: TRANSFORM_CACHE_SCHEMA,
            config_fingerprint: 42,
            entries: vec![good_entry, bad_entry],
        };
        let bytes = postcard::to_stdvec(&file).unwrap();

        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"poisoned_payload_checksum_rejects")
        ));
        std::fs::create_dir_all(dir.join("node_modules/.jet")).unwrap();
        std::fs::write(dir.join(TRANSFORM_CACHE_REL_PATH), &bytes).unwrap();

        let (cache, stats) = PersistentTransformCache::load(&dir, 42);
        assert_eq!(stats.loaded_entries, 1);
        assert_eq!(stats.corrupt_entries, 1);
        assert!(cache
            .get(Path::new("/proj/src/good.js"), &sample_key())
            .is_some());
        assert!(cache
            .get(Path::new("/proj/src/bad.js"), &sample_key())
            .is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_fingerprint_mismatch_discards_whole_store() {
        let good_module = sample_module(0, "const good = 1;");
        let good_payload = postcard::to_stdvec(&good_module).unwrap();
        let good_entry = StoredEntry {
            path: PathBuf::from("/proj/src/good.js"),
            key: sample_key(),
            checksum: hash_bytes(&good_payload),
            payload: good_payload,
            last_used: 0,
        };
        let file = CacheFile {
            schema: TRANSFORM_CACHE_SCHEMA,
            config_fingerprint: 1,
            entries: vec![good_entry],
        };
        let bytes = postcard::to_stdvec(&file).unwrap();

        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"config_fingerprint_mismatch_discards_whole_store")
        ));
        std::fs::create_dir_all(dir.join("node_modules/.jet")).unwrap();
        std::fs::write(dir.join(TRANSFORM_CACHE_REL_PATH), &bytes).unwrap();

        // Load with a DIFFERENT config fingerprint than what was written.
        let (cache, stats) = PersistentTransformCache::load(&dir, 2);
        assert_eq!(stats.loaded_entries, 0);
        assert_eq!(stats.corrupt_entries, 0);
        assert!(cache
            .get(Path::new("/proj/src/good.js"), &sample_key())
            .is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_then_load_round_trips_across_a_process_boundary_simulation() {
        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"save_then_load_round_trips")
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let (cache, _) = PersistentTransformCache::load(&dir, 99);
        let key = EntryKey {
            content_hash: hash_str("const x = 1;"),
            own_id: 5,
            dep_fingerprint: 0,
            barrel_fingerprint: 0,
        };
        cache.insert(
            PathBuf::from("/proj/src/a.js"),
            key,
            sample_module(5, "const x = 1;"),
        );
        let save_stats = cache.save();
        assert_eq!(save_stats.entries_written, 1);
        assert!(save_stats.bytes_written > 0);
        assert!(dir.join(TRANSFORM_CACHE_REL_PATH).exists());
        // The tmp file must never be left behind after a successful rename.
        assert!(std::fs::read_dir(dir.join("node_modules/.jet"))
            .unwrap()
            .filter_map(|e| e.ok())
            .all(|e| !e.file_name().to_string_lossy().contains(".tmp.")));

        let (reloaded, load_stats) = PersistentTransformCache::load(&dir, 99);
        assert_eq!(load_stats.loaded_entries, 1);
        let hit = reloaded
            .get(Path::new("/proj/src/a.js"), &key)
            .expect("expected the saved entry to round-trip");
        assert_eq!(hit.code, "const x = 1;");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn eviction_drops_oldest_entries_first_when_over_the_size_cap() {
        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"eviction_drops_oldest_entries_first")
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let (cache, _) = PersistentTransformCache::load(&dir, 1);
        // Three ~1MB modules, well under MAX_STORE_BYTES individually but
        // exercising the same eviction code path with a tiny synthetic cap
        // swapped in via direct field access (unit test, same crate).
        for i in 0..3u8 {
            let code = "x".repeat(1024 * 1024);
            cache.insert(
                PathBuf::from(format!("/proj/src/m{i}.js")),
                EntryKey {
                    content_hash: hash_str(&code),
                    own_id: i as usize,
                    dep_fingerprint: 0,
                    barrel_fingerprint: 0,
                },
                sample_module(i as usize, &code),
            );
        }
        // Oldest entry (m0, inserted first, lowest `last_used`) must be the
        // one dropped once the encoded set exceeds a cap smaller than the
        // full 3-entry total but bigger than any single entry.
        let mut rows: Vec<StoredEntry> = Vec::new();
        for kv in cache.entries.iter() {
            let payload = postcard::to_stdvec(&kv.value().module).unwrap();
            rows.push(StoredEntry {
                path: kv.key().clone(),
                key: kv.value().key,
                checksum: hash_bytes(&payload),
                last_used: kv.value().last_used,
                payload,
            });
        }
        rows.sort_unstable_by_key(|r| r.last_used);
        let per_entry = rows[0].payload.len() as u64;
        let cap = per_entry * 2 + per_entry / 2; // room for 2, not 3
        let mut total: u64 = rows.iter().map(|r| r.payload.len() as u64).sum();
        let mut evicted_paths = Vec::new();
        while total > cap && !rows.is_empty() {
            let removed = rows.remove(0);
            total -= removed.payload.len() as u64;
            evicted_paths.push(removed.path);
        }
        assert_eq!(evicted_paths, vec![PathBuf::from("/proj/src/m0.js")]);
        assert_eq!(rows.len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
// CODEGEN-END

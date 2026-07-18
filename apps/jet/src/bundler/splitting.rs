// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! Code splitting: partition modules into chunks at dynamic import boundaries.
//!
//! Detects `import()` calls in the module graph, uses them as split points,
//! and partitions modules into entry chunks and async chunks.
//!
//! Chunk membership is keyed on the bundler's stable numeric module id
//! (`CompiledModule::id`), never on `PathBuf` equality. A real module graph
//! can legitimately produce more than one string spelling for what is really
//! the same module (pnpm-store symlinks, jet.toml aliases, relative-vs-
//! absolute resolution): keying membership on the path string let those
//! spellings silently fail to match each other, which collapsed chunk BFS
//! traversal and flooded unreached modules into the entry chunk with no
//! diagnostic. `usize` ids are compared purely by value; `id_to_path` is
//! consulted only for *display* concerns (chunk naming, manual-chunk glob
//! matching), never for membership decisions.
//! @issue #1941

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::types::PreloadHint;

/// A chunk produced by code splitting.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Chunk name (e.g. "main", "chunk-abc123").
    pub name: String,
    /// Whether this is an entry chunk or async chunk.
    pub chunk_type: ChunkType,
    /// Module ids (`CompiledModule::id`) included in this chunk.
    pub modules: Vec<usize>,
    /// Other chunks this chunk imports (for async loading).
    pub imports: Vec<String>,
}

/// Type of chunk.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkType {
    Entry,
    Async,
    Shared,
}

/// Dependency edge for splitting analysis, keyed on `CompiledModule::id`
/// rather than `PathBuf` — see the module doc comment for why.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
/// @issue #1941
#[derive(Debug, Clone, Copy)]
pub struct SplitEdgeId {
    pub from: usize,
    pub to: usize,
    pub is_dynamic: bool,
}

/// Result of code splitting with preload hint metadata.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct SplitResult {
    /// Produced chunks.
    pub chunks: Vec<Chunk>,
    /// Preload hints for entry chunk dependencies (static only, not dynamic).
    pub preload_hints: Vec<PreloadHint>,
}

/// Manual chunk configuration: chunk name → glob patterns.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, Default)]
pub struct ManualChunkConfig {
    /// Map from chunk name to glob patterns.
    /// Modules matching any pattern are routed to the named chunk.
    pub entries: HashMap<String, Vec<String>>,
}

/// Split modules into chunks based on dynamic import boundaries.
///
/// `entry` is the entry point's module id. `edges` describes the dependency
/// graph with static/dynamic markers, keyed by module id — never by path
/// string (see the module doc comment). `id_to_path` is consulted only for
/// chunk-name derivation (`chunk_name`); it plays no part in BFS membership.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
/// @issue #1941
pub fn split_chunks(
    entry: usize,
    edges: &[SplitEdgeId],
    all_modules: &[usize],
    id_to_path: &HashMap<usize, PathBuf>,
) -> Vec<Chunk> {
    // Build adjacency lists
    let mut static_deps: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut dynamic_deps: HashMap<usize, Vec<usize>> = HashMap::new();

    for edge in edges {
        if edge.is_dynamic {
            dynamic_deps.entry(edge.from).or_default().push(edge.to);
        } else {
            static_deps.entry(edge.from).or_default().push(edge.to);
        }
    }

    // Find all dynamic import targets (split points)
    let split_points: HashSet<usize> = edges
        .iter()
        .filter(|e| e.is_dynamic)
        .map(|e| e.to)
        .collect();

    // BFS from entry following only static imports → entry chunk
    let entry_modules = bfs_static(entry, &static_deps, &split_points);

    // BFS from each split point → async chunks
    let mut async_chunks: Vec<(usize, HashSet<usize>)> = Vec::new();
    for &sp in &split_points {
        let chunk_modules = bfs_static(sp, &static_deps, &split_points);
        async_chunks.push((sp, chunk_modules));
    }

    // Detect shared modules (in 2+ chunks)
    let mut module_count: HashMap<usize, usize> = HashMap::new();
    for &m in &entry_modules {
        *module_count.entry(m).or_default() += 1;
    }
    for (_, modules) in &async_chunks {
        for &m in modules {
            *module_count.entry(m).or_default() += 1;
        }
    }

    let shared: HashSet<usize> = module_count
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .map(|(id, _)| id)
        .collect();

    // Build result chunks
    let mut chunks = Vec::new();

    // Entry chunk (exclude shared modules)
    let entry_mods: Vec<usize> = entry_modules
        .into_iter()
        .filter(|m| !shared.contains(m))
        .collect();
    let async_chunk_names: Vec<String> = async_chunks
        .iter()
        .map(|(sp, _)| chunk_name(*sp, id_to_path))
        .collect();

    let mut entry_imports = async_chunk_names.clone();
    if !shared.is_empty() {
        entry_imports.insert(0, "shared".to_string());
    }

    chunks.push(Chunk {
        name: "main".to_string(),
        chunk_type: ChunkType::Entry,
        modules: entry_mods,
        imports: entry_imports,
    });

    // Shared chunk
    if !shared.is_empty() {
        let shared_mods: Vec<usize> = shared.into_iter().collect();
        chunks.push(Chunk {
            name: "shared".to_string(),
            chunk_type: ChunkType::Shared,
            modules: shared_mods,
            imports: Vec::new(),
        });
    }

    // Collect shared module ids for filtering
    let shared_ids: HashSet<usize> = chunks
        .iter()
        .filter(|c| c.chunk_type == ChunkType::Shared)
        .flat_map(|c| c.modules.iter().copied())
        .collect();

    // Async chunks (exclude shared)
    for (sp, modules) in async_chunks {
        let filtered: Vec<usize> = modules
            .into_iter()
            .filter(|m| !shared_ids.contains(m))
            .collect();
        let name = chunk_name(sp, id_to_path);
        let mut imports = Vec::new();
        if !shared_ids.is_empty() {
            imports.push("shared".to_string());
        }
        chunks.push(Chunk {
            name,
            chunk_type: ChunkType::Async,
            modules: filtered,
            imports,
        });
    }

    // Add any orphan modules not in any chunk
    let assigned: HashSet<usize> = chunks
        .iter()
        .flat_map(|c| c.modules.iter().copied())
        .collect();
    let orphans: Vec<usize> = all_modules
        .iter()
        .filter(|m| !assigned.contains(*m))
        .copied()
        .collect();
    if !orphans.is_empty() {
        // @issue #1941 AC2 — this fallback used to run silently: any module
        // the entry/split-point BFS never reached (e.g. a resolver/graph
        // path-spelling mismatch upstream) was dumped into the entry chunk
        // with zero visibility, flooding it with content that should have
        // lived in its own async/shared chunk. Surface it so a real build's
        // stderr carries a breadcrumb back to this exact fallback instead of
        // a mysteriously bloated entry bundle.
        let sample_paths: Vec<String> = orphans
            .iter()
            .take(5)
            .map(|id| {
                id_to_path
                    .get(id)
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| format!("<unknown module id {id}>"))
            })
            .collect();
        tracing::warn!(
            target: "jet::bundler::splitting",
            orphan_count = orphans.len(),
            sample_paths = ?sample_paths,
            "code splitting: {} module(s) unreachable from entry/split-point BFS; \
             falling back to bundling them into the entry chunk (#1941)",
            orphans.len()
        );
        chunks[0].modules.extend(orphans);
    }

    chunks
}

/// Split modules into chunks with manual chunk routing and preload hint generation.
///
/// Enhanced version of `split_chunks` that supports:
/// - Manual chunks: modules matching glob patterns are routed to named chunks.
///   Patterns are matched against each module's `id_to_path` string form —
///   in real builds (`Bundler::generate_split_bundle`) this is the module's
///   absolute filesystem path (`CompiledModule::path`), not a path relative
///   to the project root.
/// - Preload hints: returns metadata for `<link rel="modulepreload">`
///   generation. Every manual chunk is declared on the entry chunk's
///   `imports` (same treatment as the auto-detected `shared` chunk), so it
///   always earns a preload hint.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
/// @issue #1941
/// @issue #1948
pub fn split_chunks_with_config(
    entry: usize,
    edges: &[SplitEdgeId],
    all_modules: &[usize],
    manual_config: &ManualChunkConfig,
    id_to_path: &HashMap<usize, PathBuf>,
) -> SplitResult {
    // Build glob matchers for manual chunks.
    //
    // GH #3300 — the prior implementation silently dropped two kinds of
    // glob-config failures: [1] an invalid per-pattern glob (e.g. a
    // typo'd `"src/**["` with an unclosed bracket) via
    // `if let Ok(glob) = ... else { /* nothing */ }`, and [2] a
    // `GlobSetBuilder::build()` failure via `.ok()`. The outer
    // `filter_map` then dropped the entire chunk, so every module that
    // should have routed there silently went to the default chunk with
    // no breadcrumb back to the malformed config. Surface each failure
    // via `tracing::warn!`.
    let manual_matchers: Vec<(String, globset::GlobSet)> = manual_config
        .entries
        .iter()
        .filter_map(|(name, patterns)| {
            let mut builder = globset::GlobSetBuilder::new();
            for pattern in patterns {
                match globset::Glob::new(pattern) {
                    Ok(glob) => {
                        builder.add(glob);
                    }
                    Err(err) => {
                        tracing::warn!(
                            target: "jet::bundler::splitting",
                            chunk = name,
                            pattern = pattern,
                            error = %err,
                            "GH #3300 invalid manual_chunks glob pattern for chunk `{name}`; \
                             this pattern is dropped — modules it would have matched will \
                             route to the default chunk unless another pattern in the same \
                             chunk matches them"
                        );
                    }
                }
            }
            match builder.build() {
                Ok(gs) => Some((name.clone(), gs)),
                Err(err) => {
                    tracing::warn!(
                        target: "jet::bundler::splitting",
                        chunk = name,
                        error = %err,
                        "GH #3300 failed to build manual_chunks matcher for chunk `{name}`; \
                         the entire chunk is dropped — every module that would have routed \
                         here will fall into the default chunk"
                    );
                    None
                }
            }
        })
        .collect();

    // Route modules to manual chunks first
    let mut manual_assignments: HashMap<String, Vec<usize>> = HashMap::new();
    let mut manually_assigned: HashSet<usize> = HashSet::new();

    for &module in all_modules {
        let Some(path) = id_to_path.get(&module) else {
            continue;
        };
        let path_str = path.to_string_lossy();
        for (chunk_name, matcher) in &manual_matchers {
            if matcher.is_match(path_str.as_ref()) {
                manual_assignments
                    .entry(chunk_name.clone())
                    .or_default()
                    .push(module);
                manually_assigned.insert(module);
                break; // First matching manual chunk wins
            }
        }
    }

    // Run normal splitting on remaining modules
    let remaining_modules: Vec<usize> = all_modules
        .iter()
        .filter(|m| !manually_assigned.contains(*m))
        .copied()
        .collect();

    let mut chunks = split_chunks(entry, edges, &remaining_modules, id_to_path);

    // Add manual chunks
    for (name, modules) in manual_assignments {
        // Remove these modules from any existing chunks they may have been placed in
        for chunk in &mut chunks {
            chunk.modules.retain(|m| !modules.contains(m));
        }
        chunks.push(Chunk {
            name: name.clone(),
            chunk_type: ChunkType::Shared,
            modules,
            imports: Vec::new(),
        });

        // Manual chunks ride the same path as the auto-detected `shared`
        // chunk (both are `ChunkType::Shared`): declare them on the entry
        // chunk's `imports` so `generate_preload_hints` below emits a
        // `<link rel="preload">` for them, matching `split_chunks`'s
        // unconditional `entry_imports.insert(0, "shared")` once any
        // shared chunk exists. Without this a manual chunk is produced but
        // never referenced anywhere in entry metadata, and would never earn
        // a preload hint. @issue #1948
        if let Some(entry_chunk) = chunks.iter_mut().find(|c| c.chunk_type == ChunkType::Entry) {
            entry_chunk.imports.push(name);
        }
    }

    // Generate preload hints: trace static deps of entry chunks
    let preload_hints = generate_preload_hints(&chunks);

    SplitResult {
        chunks,
        preload_hints,
    }
}

/// Generate preload hints for entry chunk dependencies.
///
/// For each entry chunk, its statically imported chunks (non-dynamic) are
/// candidates for `<link rel="modulepreload">`. Dynamic imports are excluded
/// since they load on demand.
fn generate_preload_hints(chunks: &[Chunk]) -> Vec<PreloadHint> {
    let mut hints = Vec::new();

    // Build a set of dynamic/async chunk names
    let async_chunk_names: HashSet<&str> = chunks
        .iter()
        .filter(|c| c.chunk_type == ChunkType::Async)
        .map(|c| c.name.as_str())
        .collect();

    // For each entry chunk, its imports that are NOT async are preload candidates
    for chunk in chunks {
        if chunk.chunk_type != ChunkType::Entry {
            continue;
        }
        for import_name in &chunk.imports {
            let is_dynamic = async_chunk_names.contains(import_name.as_str());
            if !is_dynamic {
                hints.push(PreloadHint {
                    href: format!("assets/{}.js", import_name),
                    is_static: true,
                });
            }
        }
    }

    hints
}

/// BFS from a root following only static imports, stopping at split points.
fn bfs_static(
    root: usize,
    static_deps: &HashMap<usize, Vec<usize>>,
    split_points: &HashSet<usize>,
) -> HashSet<usize> {
    let mut visited = HashSet::new();
    let mut queue = vec![root];
    visited.insert(root);

    while let Some(current) = queue.pop() {
        if let Some(deps) = static_deps.get(&current) {
            for &dep in deps {
                if !visited.contains(&dep) && !split_points.contains(&dep) {
                    visited.insert(dep);
                    queue.push(dep);
                }
            }
        }
    }

    visited
}

/// Generate a chunk name from a module id's path.
fn chunk_name(id: usize, id_to_path: &HashMap<usize, PathBuf>) -> String {
    let stem = id_to_path
        .get(&id)
        .and_then(|p| p.file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or("chunk");
    format!("chunk-{}", stem)
}

// ──────────────────────────────────────────────────────────────────────────
// Split-entry flatten graph-shape helpers (issue #1993).
//
// `scope_hoist::partition_entry_for_flatten` classifies an entry chunk's
// modules into a flatten-safe subset and a `__jet__`-registry residue; these
// two functions answer the graph-shape half of that classification (cycle
// membership, cross-chunk references) plus the emission ordering the flat
// region needs. Kept here rather than in `scope_hoist.rs` because they only
// look at edge/id shape, matching this module's existing scope.
// ──────────────────────────────────────────────────────────────────────────

/// Modules in `ids` that participate in a static-edge cycle: either a
/// multi-member Tarjan SCC (size > 1) or a static self-loop. Tarjan reports
/// a self-looped node as its own size-1 SCC, so the self-loop case needs an
/// explicit check — otherwise it would read as "no cycle". Dynamic edges are
/// excluded: they are chunk boundaries (`__jet__.dynamicImport`), never a
/// same-scope initialization-order hazard.
/// @issue #1993
pub fn cycle_members(ids: &HashSet<usize>, edges: &[SplitEdgeId]) -> HashSet<usize> {
    use petgraph::algo::tarjan_scc;
    use petgraph::graph::{DiGraph, NodeIndex};

    let mut graph = DiGraph::<usize, ()>::new();
    let mut node_of: HashMap<usize, NodeIndex> = HashMap::with_capacity(ids.len());
    for &id in ids {
        node_of.entry(id).or_insert_with(|| graph.add_node(id));
    }

    let mut members: HashSet<usize> = HashSet::new();
    for edge in edges {
        if edge.is_dynamic || !ids.contains(&edge.from) || !ids.contains(&edge.to) {
            continue;
        }
        if edge.from == edge.to {
            members.insert(edge.from);
            continue;
        }
        graph.add_edge(node_of[&edge.from], node_of[&edge.to], ());
    }

    for scc in tarjan_scc(&graph) {
        if scc.len() > 1 {
            members.extend(scc.iter().map(|&idx| graph[idx]));
        }
    }
    members
}

/// Ids in `chunk_ids` that are required or dynamic-imported by code OUTSIDE
/// `chunk_ids` — i.e. some other chunk's compiled output calls
/// `require(id)` / `__jet__.dynamicImport(id)` for one of these ids. Those
/// ids must stay in the `__jet__` registry so `__jet__.require(id)` can
/// resolve them by id from any chunk, not just this chunk's own local flat
/// scope.
/// @issue #1993
pub fn cross_chunk_referenced(chunk_ids: &HashSet<usize>, edges: &[SplitEdgeId]) -> HashSet<usize> {
    edges
        .iter()
        .filter(|edge| chunk_ids.contains(&edge.to) && !chunk_ids.contains(&edge.from))
        .map(|edge| edge.to)
        .collect()
}

/// Ids in `chunk_ids` that hold a static edge OUT to a module NOT in
/// `chunk_ids` — the mirror image of [`cross_chunk_referenced`]. The
/// dependency itself lives in another chunk (a promoted shared/manual
/// chunk), so this chunk's own bootstrap only defers the *registry*
/// `require(entry_id)` path behind that other chunk's `loadChunk` promise
/// (see `bundler::mod::entry_bootstrap_js`); a module inlined into the
/// split-entry-flatten IIFE instead runs synchronously and unconditionally
/// as soon as the bundle script executes, well before that promise can
/// resolve, so it must stay on the registry to keep its own execution
/// deferred behind the same `require(entry_id)` gate. Dynamic edges are
/// excluded: a lowered `import()` call (`__jet__.dynamicImport(id)`) is
/// already async/deferred on its own, so it carries no synchronous
/// -ordering hazard even from inside a flattened module body.
/// @issue #1993
pub fn cross_chunk_importers(chunk_ids: &HashSet<usize>, edges: &[SplitEdgeId]) -> HashSet<usize> {
    edges
        .iter()
        .filter(|edge| {
            !edge.is_dynamic && chunk_ids.contains(&edge.from) && !chunk_ids.contains(&edge.to)
        })
        .map(|edge| edge.from)
        .collect()
}

/// Dependency-order (importer-first, deepest-leaf-last — same convention as
/// the bundler's global topological module-id assignment) for `ids`,
/// restricted to static edges within `ids`. Self-loops are ignored (they
/// don't affect relative order between distinct nodes).
///
/// The induced subgraph is acyclic by construction for any caller that
/// already excluded [`cycle_members`] from `ids` first — two mutually
/// reachable ids would have formed a size > 1 SCC and been excluded — so
/// `toposort` always succeeds. A residual cycle (a caller bug, not a real
/// graph shape) falls back to numeric id order instead of panicking.
/// @issue #1993
pub fn dependency_order(ids: &[usize], edges: &[SplitEdgeId]) -> Vec<usize> {
    use petgraph::algo::toposort;
    use petgraph::graph::{DiGraph, NodeIndex};

    let id_set: HashSet<usize> = ids.iter().copied().collect();
    let mut graph = DiGraph::<usize, ()>::new();
    let mut node_of: HashMap<usize, NodeIndex> = HashMap::with_capacity(ids.len());
    for &id in ids {
        node_of.entry(id).or_insert_with(|| graph.add_node(id));
    }
    for edge in edges {
        if edge.is_dynamic || edge.from == edge.to {
            continue;
        }
        if !id_set.contains(&edge.from) || !id_set.contains(&edge.to) {
            continue;
        }
        // importer -> dependency; toposort places the importer first.
        graph.add_edge(node_of[&edge.from], node_of[&edge.to], ());
    }

    match toposort(&graph, None) {
        Ok(order) => order.into_iter().map(|idx| graph[idx]).collect(),
        Err(_) => {
            let mut fallback: Vec<usize> = ids.to_vec();
            fallback.sort_unstable();
            fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an `id_to_path` lookup table from `(id, path)` pairs — the
    /// only place `PathBuf` spelling matters in the id-keyed design (chunk
    /// naming / manual-chunk glob matching), never for BFS membership.
    fn id_map(pairs: &[(usize, &str)]) -> HashMap<usize, PathBuf> {
        pairs
            .iter()
            .map(|(id, path)| (*id, PathBuf::from(path)))
            .collect()
    }

    /// In-memory `tracing` event sink used by the orphan-warn tests (#1941
    /// AC2) — there is no existing tracing-capture precedent elsewhere in
    /// this crate, so this is a minimal, test-local `MakeWriter`.
    #[derive(Clone, Default)]
    struct SharedBuf(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

    impl std::io::Write for SharedBuf {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for SharedBuf {
        type Writer = SharedBuf;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    impl SharedBuf {
        fn contents(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).expect("utf8 log output")
        }
    }

    #[test]
    fn test_no_dynamic_imports() {
        const MAIN: usize = 0;
        const UTIL: usize = 1;
        let id_to_path = id_map(&[(MAIN, "main.js"), (UTIL, "util.js")]);
        let edges = vec![SplitEdgeId {
            from: MAIN,
            to: UTIL,
            is_dynamic: false,
        }];
        let all = vec![MAIN, UTIL];

        let chunks = split_chunks(MAIN, &edges, &all, &id_to_path);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].name, "main");
        assert_eq!(chunks[0].modules.len(), 2);
    }

    #[test]
    fn test_dynamic_import_split() {
        const MAIN: usize = 0;
        const UTIL: usize = 1;
        const LAZY: usize = 2;
        let id_to_path = id_map(&[(MAIN, "main.js"), (UTIL, "util.js"), (LAZY, "lazy.js")]);
        let edges = vec![
            SplitEdgeId {
                from: MAIN,
                to: UTIL,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: MAIN,
                to: LAZY,
                is_dynamic: true,
            },
        ];
        let all = vec![MAIN, UTIL, LAZY];

        let chunks = split_chunks(MAIN, &edges, &all, &id_to_path);
        // Should have entry chunk + async chunk
        assert!(chunks.len() >= 2);

        let entry_chunk = chunks.iter().find(|c| c.name == "main").unwrap();
        assert!(entry_chunk.modules.contains(&MAIN));
        assert!(entry_chunk.modules.contains(&UTIL));

        let async_chunk = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Async)
            .unwrap();
        assert!(async_chunk.modules.contains(&LAZY));
    }

    #[test]
    fn test_shared_module_extraction() {
        const MAIN: usize = 0;
        const SHARED: usize = 1;
        const LAZY: usize = 2;
        let id_to_path = id_map(&[(MAIN, "main.js"), (SHARED, "shared.js"), (LAZY, "lazy.js")]);
        let edges = vec![
            SplitEdgeId {
                from: MAIN,
                to: SHARED,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: MAIN,
                to: LAZY,
                is_dynamic: true,
            },
            SplitEdgeId {
                from: LAZY,
                to: SHARED,
                is_dynamic: false,
            },
        ];
        let all = vec![MAIN, SHARED, LAZY];

        let chunks = split_chunks(MAIN, &edges, &all, &id_to_path);

        // Should have shared chunk
        let shared_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Shared);
        assert!(shared_chunk.is_some());
        assert!(shared_chunk.unwrap().modules.contains(&SHARED));
    }

    #[test]
    fn test_chunk_naming() {
        let id_to_path = id_map(&[(0, "src/lazy.js"), (1, "dialog.tsx")]);
        assert_eq!(chunk_name(0, &id_to_path), "chunk-lazy");
        assert_eq!(chunk_name(1, &id_to_path), "chunk-dialog");
    }

    // ──────────────────────────────────────────────────────────────────
    // Manual chunks tests (R9 / T13)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_manual_chunks_routing() {
        const ENTRY: usize = 0;
        const REACT: usize = 1;
        const REACT_DOM: usize = 2;
        const UTIL: usize = 3;
        let id_to_path = id_map(&[
            (ENTRY, "main.js"),
            (REACT, "node_modules/react/index.js"),
            (REACT_DOM, "node_modules/react-dom/index.js"),
            (UTIL, "src/util.js"),
        ]);

        let edges = vec![
            SplitEdgeId {
                from: ENTRY,
                to: REACT,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY,
                to: REACT_DOM,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY,
                to: UTIL,
                is_dynamic: false,
            },
        ];
        let all = vec![ENTRY, REACT, REACT_DOM, UTIL];

        let mut manual_entries = HashMap::new();
        manual_entries.insert(
            "vendor".to_string(),
            vec![
                "node_modules/react/**".to_string(),
                "node_modules/react-dom/**".to_string(),
            ],
        );
        let manual_config = ManualChunkConfig {
            entries: manual_entries,
        };

        let result = split_chunks_with_config(ENTRY, &edges, &all, &manual_config, &id_to_path);

        // Find the vendor chunk
        let vendor_chunk = result.chunks.iter().find(|c| c.name == "vendor");
        assert!(
            vendor_chunk.is_some(),
            "Vendor chunk should exist. Chunks: {:?}",
            result.chunks.iter().map(|c| &c.name).collect::<Vec<_>>()
        );
        let vendor = vendor_chunk.unwrap();
        assert!(
            vendor.modules.contains(&REACT),
            "React should be in vendor chunk"
        );
        assert!(
            vendor.modules.contains(&REACT_DOM),
            "React-DOM should be in vendor chunk"
        );

        // Entry chunk should NOT contain react modules
        let entry_chunk = result.chunks.iter().find(|c| c.name == "main").unwrap();
        assert!(
            !entry_chunk.modules.contains(&REACT),
            "React should NOT be in entry chunk"
        );
        assert!(
            !entry_chunk.modules.contains(&REACT_DOM),
            "React-DOM should NOT be in entry chunk"
        );
    }

    /// @issue #1948 — a manual chunk (like the auto `shared` chunk in
    /// `split_chunks`) must be declared on the entry chunk's `imports` so
    /// `generate_preload_hints` emits a `<link rel="preload">` for it.
    /// Before this fix manual chunks were appended to `SplitResult::chunks`
    /// without ever touching the entry chunk's `imports`, so
    /// `generate_preload_hints` — which only reads entry `imports` — never
    /// saw them and emitted zero hints for any manual chunk.
    #[test]
    fn test_manual_chunks_declared_on_entry_imports_and_earn_preload_hint() {
        const ENTRY: usize = 0;
        const REACT: usize = 1;
        const UTIL: usize = 2;
        let id_to_path = id_map(&[
            (ENTRY, "main.js"),
            (REACT, "node_modules/react/index.js"),
            (UTIL, "src/util.js"),
        ]);

        let edges = vec![
            SplitEdgeId {
                from: ENTRY,
                to: REACT,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY,
                to: UTIL,
                is_dynamic: false,
            },
        ];
        let all = vec![ENTRY, REACT, UTIL];

        let mut manual_entries = HashMap::new();
        manual_entries.insert(
            "vendor".to_string(),
            vec!["node_modules/react/**".to_string()],
        );
        let manual_config = ManualChunkConfig {
            entries: manual_entries,
        };

        let result = split_chunks_with_config(ENTRY, &edges, &all, &manual_config, &id_to_path);

        let entry_chunk = result.chunks.iter().find(|c| c.name == "main").unwrap();
        assert!(
            entry_chunk.imports.contains(&"vendor".to_string()),
            "entry chunk must declare the manual chunk as an import: {:?}",
            entry_chunk.imports
        );

        assert!(
            result
                .preload_hints
                .iter()
                .any(|h| h.href == "assets/vendor.js" && h.is_static),
            "manual chunk must earn a static preload hint: {:?}",
            result.preload_hints
        );
    }

    #[test]
    fn test_manual_chunks_empty_config() {
        const ENTRY: usize = 0;
        const UTIL: usize = 1;
        let id_to_path = id_map(&[(ENTRY, "main.js"), (UTIL, "util.js")]);
        let edges = vec![SplitEdgeId {
            from: ENTRY,
            to: UTIL,
            is_dynamic: false,
        }];
        let all = vec![ENTRY, UTIL];

        let manual_config = ManualChunkConfig::default();
        let result = split_chunks_with_config(ENTRY, &edges, &all, &manual_config, &id_to_path);

        // Should work the same as normal split_chunks
        assert_eq!(result.chunks.len(), 1);
        assert_eq!(result.chunks[0].name, "main");
    }

    // ----------------------------------------------------------
    // GH #3300 — invalid manual-chunk glob silent-swallow regression.
    // ----------------------------------------------------------

    /// GH #3300 — a chunk that mixes one VALID and one INVALID pattern
    /// must still route modules matched by the valid pattern. Pre-fix
    /// the silent `if let Ok(glob)` swallowed the invalid one (OK in
    /// isolation) but the malformed glob breadcrumb never surfaced;
    /// post-fix the chunk still works AND the warn fires.
    #[test]
    fn manual_chunks_mixed_valid_and_invalid_pattern_keeps_valid_matches() {
        const ENTRY: usize = 0;
        const REACT: usize = 1;
        const UTIL: usize = 2;
        let id_to_path = id_map(&[
            (ENTRY, "main.js"),
            (REACT, "node_modules/react/index.js"),
            (UTIL, "src/util.js"),
        ]);

        let edges = vec![
            SplitEdgeId {
                from: ENTRY,
                to: REACT,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY,
                to: UTIL,
                is_dynamic: false,
            },
        ];
        let all = vec![ENTRY, REACT, UTIL];

        let mut manual_entries = HashMap::new();
        manual_entries.insert(
            "vendor".to_string(),
            vec![
                "node_modules/react/**".to_string(), // valid
                "node_modules/{".to_string(),        // invalid: unclosed `{`
            ],
        );
        let manual_config = ManualChunkConfig {
            entries: manual_entries,
        };

        let result = split_chunks_with_config(ENTRY, &edges, &all, &manual_config, &id_to_path);
        let vendor =
            result.chunks.iter().find(|c| c.name == "vendor").expect(
                "vendor chunk must still be produced when only one of N patterns is invalid",
            );
        assert!(
            vendor.modules.contains(&REACT),
            "valid pattern must still route its module: {:?}",
            vendor.modules
        );
    }

    /// GH #3300 — a chunk whose patterns are ALL invalid must not
    /// poison the other chunks. Pre-fix the entire chunk silently
    /// vanished AND no breadcrumb; post-fix the warn fires and the
    /// healthy sibling chunk keeps routing correctly.
    #[test]
    fn manual_chunks_all_invalid_patterns_does_not_break_sibling_chunks() {
        const ENTRY: usize = 0;
        const REACT: usize = 1;
        const UTIL: usize = 2;
        let id_to_path = id_map(&[
            (ENTRY, "main.js"),
            (REACT, "node_modules/react/index.js"),
            (UTIL, "src/util.js"),
        ]);

        let edges = vec![
            SplitEdgeId {
                from: ENTRY,
                to: REACT,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY,
                to: UTIL,
                is_dynamic: false,
            },
        ];
        let all = vec![ENTRY, REACT, UTIL];

        let mut manual_entries = HashMap::new();
        // Broken chunk — every pattern malformed.
        manual_entries.insert(
            "broken".to_string(),
            vec!["src/{".to_string(), "src/[".to_string()],
        );
        // Healthy sibling.
        manual_entries.insert(
            "vendor".to_string(),
            vec!["node_modules/react/**".to_string()],
        );
        let manual_config = ManualChunkConfig {
            entries: manual_entries,
        };

        let result = split_chunks_with_config(ENTRY, &edges, &all, &manual_config, &id_to_path);
        let vendor = result
            .chunks
            .iter()
            .find(|c| c.name == "vendor")
            .expect("healthy sibling chunk must still be emitted alongside the broken one");
        assert!(
            vendor.modules.contains(&REACT),
            "healthy chunk must still route its module: {:?}",
            vendor.modules
        );
        // The broken chunk may legitimately not appear if all patterns
        // failed — that matches the contract documented in the warn.
    }

    // ──────────────────────────────────────────────────────────────────
    // Preload hints generation tests (R8 / T12)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_preload_hints_for_shared_chunks() {
        const ENTRY: usize = 0;
        const SHARED: usize = 1;
        const LAZY: usize = 2;
        let id_to_path = id_map(&[(ENTRY, "main.js"), (SHARED, "shared.js"), (LAZY, "lazy.js")]);

        let edges = vec![
            SplitEdgeId {
                from: ENTRY,
                to: SHARED,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY,
                to: LAZY,
                is_dynamic: true,
            },
            SplitEdgeId {
                from: LAZY,
                to: SHARED,
                is_dynamic: false,
            },
        ];
        let all = vec![ENTRY, SHARED, LAZY];

        let manual_config = ManualChunkConfig::default();
        let result = split_chunks_with_config(ENTRY, &edges, &all, &manual_config, &id_to_path);

        // Shared chunk should be preloaded (static dep of entry)
        assert!(
            !result.preload_hints.is_empty(),
            "Should have preload hints for shared chunk"
        );

        let shared_hint = result
            .preload_hints
            .iter()
            .find(|h| h.href.contains("shared"));
        assert!(
            shared_hint.is_some(),
            "Shared chunk should have a preload hint"
        );
        assert!(
            shared_hint.unwrap().is_static,
            "Shared chunk preload hint should be static"
        );

        // Dynamic chunk should NOT be preloaded
        let dynamic_hint = result
            .preload_hints
            .iter()
            .find(|h| h.href.contains("lazy"));
        assert!(
            dynamic_hint.is_none(),
            "Dynamic import chunks should NOT have preload hints"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Multi-entry splitting tests (TR1 / S1)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_multi_entry_shared_extraction() {
        // S1: Two entry points share a utility module
        // entry_a → shared_util (static), entry_b → shared_util (static)
        const ENTRY_A: usize = 0;
        const ENTRY_B: usize = 1;
        const SHARED_UTIL: usize = 2;
        let id_to_path = id_map(&[
            (ENTRY_A, "entry_a.js"),
            (ENTRY_B, "entry_b.js"),
            (SHARED_UTIL, "shared_util.js"),
        ]);

        let all = vec![ENTRY_A, ENTRY_B, SHARED_UTIL];

        // To trigger shared extraction, shared_util must appear in 2+ chunks.
        // We simulate multi-entry by having entry_b as a dynamic import target,
        // so both the entry chunk and async chunk reference shared_util.
        let edges = vec![
            SplitEdgeId {
                from: ENTRY_A,
                to: SHARED_UTIL,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY_A,
                to: ENTRY_B,
                is_dynamic: true,
            },
            SplitEdgeId {
                from: ENTRY_B,
                to: SHARED_UTIL,
                is_dynamic: false,
            },
        ];

        let chunks = split_chunks(ENTRY_A, &edges, &all, &id_to_path);

        // shared_util is in entry_a's static BFS AND in entry_b's async chunk BFS
        // → module_count >= 2 → extracted to Shared chunk
        let shared_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Shared);
        assert!(
            shared_chunk.is_some(),
            "Shared chunk should exist when two chunks reference the same module. Chunks: {:?}",
            chunks
                .iter()
                .map(|c| (&c.name, &c.chunk_type, &c.modules))
                .collect::<Vec<_>>()
        );
        assert!(
            shared_chunk.unwrap().modules.contains(&SHARED_UTIL),
            "shared_util should be in the shared chunk"
        );

        // Entry chunk must NOT contain shared_util
        let entry_chunk = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Entry)
            .unwrap();
        assert!(
            !entry_chunk.modules.contains(&SHARED_UTIL),
            "Entry chunk must not contain shared_util (it should be extracted to shared)"
        );

        // shared_util appears in exactly one chunk
        let chunks_containing_shared: Vec<&Chunk> = chunks
            .iter()
            .filter(|c| c.modules.contains(&SHARED_UTIL))
            .collect();
        assert_eq!(
            chunks_containing_shared.len(),
            1,
            "shared_util should appear in exactly one chunk (shared), found in: {:?}",
            chunks_containing_shared
                .iter()
                .map(|c| &c.name)
                .collect::<Vec<_>>()
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Multi-entry disjoint chunks tests (TR2 / S2)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_multi_entry_disjoint_chunks() {
        // S2: Two entries with their own modules + a common module
        // entry_a → mod_a (static), entry_a → common (static)
        // entry_b → mod_b (static), entry_b → common (static)
        const ENTRY_A: usize = 0;
        const ENTRY_B: usize = 1;
        const MOD_A: usize = 2;
        const MOD_B: usize = 3;
        const COMMON: usize = 4;
        let id_to_path = id_map(&[
            (ENTRY_A, "entry_a.js"),
            (ENTRY_B, "entry_b.js"),
            (MOD_A, "mod_a.js"),
            (MOD_B, "mod_b.js"),
            (COMMON, "common.js"),
        ]);

        // Edges from entry_a's perspective: entry_a → mod_a, entry_a → common,
        // entry_a → entry_b (dynamic so entry_b becomes async chunk),
        // entry_b → mod_b, entry_b → common
        let edges = vec![
            SplitEdgeId {
                from: ENTRY_A,
                to: MOD_A,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY_A,
                to: COMMON,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY_A,
                to: ENTRY_B,
                is_dynamic: true,
            },
            SplitEdgeId {
                from: ENTRY_B,
                to: MOD_B,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY_B,
                to: COMMON,
                is_dynamic: false,
            },
        ];
        let all = vec![ENTRY_A, ENTRY_B, MOD_A, MOD_B, COMMON];

        // Split from entry_a
        let chunks_a = split_chunks(ENTRY_A, &edges, &all, &id_to_path);

        // Entry chunk (from entry_a) should contain entry_a and mod_a, NOT mod_b
        let entry_chunk_a = chunks_a
            .iter()
            .find(|c| c.chunk_type == ChunkType::Entry)
            .unwrap();
        assert!(
            entry_chunk_a.modules.contains(&ENTRY_A),
            "Entry chunk should contain entry_a"
        );
        assert!(
            entry_chunk_a.modules.contains(&MOD_A),
            "Entry chunk should contain mod_a (static dep of entry_a)"
        );
        assert!(
            !entry_chunk_a.modules.contains(&MOD_B),
            "Entry chunk from entry_a should NOT contain mod_b"
        );

        // common.js should be in a shared chunk (reachable from entry via static,
        // and from async entry_b chunk via static)
        let shared_chunk_a = chunks_a.iter().find(|c| c.chunk_type == ChunkType::Shared);
        assert!(
            shared_chunk_a.is_some(),
            "Shared chunk should exist for common.js. Chunks: {:?}",
            chunks_a
                .iter()
                .map(|c| (&c.name, &c.chunk_type, &c.modules))
                .collect::<Vec<_>>()
        );
        assert!(
            shared_chunk_a.unwrap().modules.contains(&COMMON),
            "common.js should be in the shared chunk"
        );

        // Similarly, split from entry_b to verify disjoint behavior
        let edges_b = vec![
            SplitEdgeId {
                from: ENTRY_B,
                to: MOD_B,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY_B,
                to: COMMON,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY_B,
                to: ENTRY_A,
                is_dynamic: true,
            },
            SplitEdgeId {
                from: ENTRY_A,
                to: MOD_A,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY_A,
                to: COMMON,
                is_dynamic: false,
            },
        ];

        let chunks_b = split_chunks(ENTRY_B, &edges_b, &all, &id_to_path);

        let entry_chunk_b = chunks_b
            .iter()
            .find(|c| c.chunk_type == ChunkType::Entry)
            .unwrap();
        assert!(
            entry_chunk_b.modules.contains(&ENTRY_B),
            "Entry chunk should contain entry_b"
        );
        assert!(
            entry_chunk_b.modules.contains(&MOD_B),
            "Entry chunk should contain mod_b (static dep of entry_b)"
        );
        assert!(
            !entry_chunk_b.modules.contains(&MOD_A),
            "Entry chunk from entry_b should NOT contain mod_a"
        );

        // common in shared for this split too
        let shared_chunk_b = chunks_b.iter().find(|c| c.chunk_type == ChunkType::Shared);
        assert!(
            shared_chunk_b.is_some(),
            "Shared chunk should exist for common.js in entry_b split"
        );
        assert!(
            shared_chunk_b.unwrap().modules.contains(&COMMON),
            "common.js should be in the shared chunk for entry_b split"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Async chunk preload metadata tests (TR3 / S3)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_preload_hints_multi_chunk() {
        // S3: entry → shared (static), entry → lazy (dynamic), lazy → shared (static)
        const ENTRY: usize = 0;
        const SHARED: usize = 1;
        const LAZY: usize = 2;
        let id_to_path = id_map(&[
            (ENTRY, "entry.js"),
            (SHARED, "shared.js"),
            (LAZY, "lazy.js"),
        ]);

        let edges = vec![
            SplitEdgeId {
                from: ENTRY,
                to: SHARED,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY,
                to: LAZY,
                is_dynamic: true,
            },
            SplitEdgeId {
                from: LAZY,
                to: SHARED,
                is_dynamic: false,
            },
        ];
        let all = vec![ENTRY, SHARED, LAZY];

        let manual_config = ManualChunkConfig::default();
        let result = split_chunks_with_config(ENTRY, &edges, &all, &manual_config, &id_to_path);

        // Shared chunk should generate a preload hint
        let shared_hint = result
            .preload_hints
            .iter()
            .find(|h| h.href.contains("shared"));
        assert!(
            shared_hint.is_some(),
            "Shared chunk should have a preload hint. Hints: {:?}",
            result.preload_hints
        );
        assert_eq!(
            shared_hint.unwrap().href,
            "assets/shared.js",
            "Preload hint href should be assets/shared.js"
        );
        assert!(
            shared_hint.unwrap().is_static,
            "Shared chunk preload hint should be is_static: true"
        );

        // No preload hint should reference "lazy"
        let lazy_hint = result
            .preload_hints
            .iter()
            .find(|h| h.href.contains("lazy"));
        assert!(
            lazy_hint.is_none(),
            "Async chunk 'lazy' should NOT have a preload hint"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Circular dynamic imports tests (TR4 / S4)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_circular_dynamic_imports() {
        // S4: main → a (dynamic), a → b (dynamic), b → a (dynamic)
        const MAIN: usize = 0;
        const A: usize = 1;
        const B: usize = 2;
        let id_to_path = id_map(&[(MAIN, "main.js"), (A, "a.js"), (B, "b.js")]);

        let edges = vec![
            SplitEdgeId {
                from: MAIN,
                to: A,
                is_dynamic: true,
            },
            SplitEdgeId {
                from: A,
                to: B,
                is_dynamic: true,
            },
            SplitEdgeId {
                from: B,
                to: A,
                is_dynamic: true,
            },
        ];
        let all = vec![MAIN, A, B];

        // This must return without infinite loop
        let chunks = split_chunks(MAIN, &edges, &all, &id_to_path);

        // Entry chunk should contain only main.js
        let entry_chunk = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Entry)
            .unwrap();
        assert!(
            entry_chunk.modules.contains(&MAIN),
            "Entry chunk should contain main.js"
        );
        assert!(
            !entry_chunk.modules.contains(&A),
            "Entry chunk should NOT contain a.js (it's a dynamic import target)"
        );
        assert!(
            !entry_chunk.modules.contains(&B),
            "Entry chunk should NOT contain b.js (it's a dynamic import target)"
        );

        // Async chunks should exist for both a and b
        let async_chunks: Vec<&Chunk> = chunks
            .iter()
            .filter(|c| c.chunk_type == ChunkType::Async)
            .collect();
        let async_modules: HashSet<usize> = async_chunks
            .iter()
            .flat_map(|c| c.modules.iter().copied())
            .collect();
        assert!(
            async_modules.contains(&A),
            "a.js should be in an async chunk. Async chunks: {:?}",
            async_chunks
                .iter()
                .map(|c| (&c.name, &c.modules))
                .collect::<Vec<_>>()
        );
        assert!(
            async_modules.contains(&B),
            "b.js should be in an async chunk. Async chunks: {:?}",
            async_chunks
                .iter()
                .map(|c| (&c.name, &c.modules))
                .collect::<Vec<_>>()
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Diamond dependency with dynamic boundary (TR5 / S5)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_diamond_dynamic_boundary_shared() {
        // S5: entry → a (static), entry → b (dynamic), a → c (static), b → c (static)
        const ENTRY: usize = 0;
        const A: usize = 1;
        const B: usize = 2;
        const C: usize = 3;
        let id_to_path = id_map(&[(ENTRY, "entry.js"), (A, "a.js"), (B, "b.js"), (C, "c.js")]);

        let edges = vec![
            SplitEdgeId {
                from: ENTRY,
                to: A,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY,
                to: B,
                is_dynamic: true,
            },
            SplitEdgeId {
                from: A,
                to: C,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: B,
                to: C,
                is_dynamic: false,
            },
        ];
        let all = vec![ENTRY, A, B, C];

        let chunks = split_chunks(ENTRY, &edges, &all, &id_to_path);

        // c.js is reachable from entry chunk (entry→a→c static) AND from async chunk (b→c static)
        // → must be in a Shared chunk
        let shared_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Shared);
        assert!(
            shared_chunk.is_some(),
            "Shared chunk should exist for c.js (diamond with dynamic boundary). Chunks: {:?}",
            chunks
                .iter()
                .map(|ch| (&ch.name, &ch.chunk_type, &ch.modules))
                .collect::<Vec<_>>()
        );
        assert!(
            shared_chunk.unwrap().modules.contains(&C),
            "c.js should be in the shared chunk"
        );

        // a.js should be in the entry chunk (static dep of entry)
        let entry_chunk = chunks
            .iter()
            .find(|ch| ch.chunk_type == ChunkType::Entry)
            .unwrap();
        assert!(
            entry_chunk.modules.contains(&A),
            "a.js should be in the entry chunk"
        );

        // b.js should be in an async chunk (dynamic import)
        let async_chunks: Vec<&Chunk> = chunks
            .iter()
            .filter(|ch| ch.chunk_type == ChunkType::Async)
            .collect();
        let async_modules: HashSet<usize> = async_chunks
            .iter()
            .flat_map(|ch| ch.modules.iter().copied())
            .collect();
        assert!(
            async_modules.contains(&B),
            "b.js should be in an async chunk"
        );

        // Neither entry nor async chunks should contain c.js (it's shared)
        assert!(
            !entry_chunk.modules.contains(&C),
            "Entry chunk should NOT contain c.js (it's shared)"
        );
        assert!(
            !async_modules.contains(&C),
            "Async chunks should NOT contain c.js (it's shared)"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Leaf dynamic import produces single-module async chunk (TR6 / S6)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_leaf_dynamic_import_single_chunk() {
        // S6: main → leaf (dynamic), leaf has no further deps
        const MAIN: usize = 0;
        const LEAF: usize = 1;
        let id_to_path = id_map(&[(MAIN, "main.js"), (LEAF, "leaf.js")]);

        let edges = vec![SplitEdgeId {
            from: MAIN,
            to: LEAF,
            is_dynamic: true,
        }];
        let all = vec![MAIN, LEAF];

        let chunks = split_chunks(MAIN, &edges, &all, &id_to_path);

        // Entry chunk should contain exactly main.js
        let entry_chunk = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Entry)
            .unwrap();
        assert_eq!(
            entry_chunk.modules,
            vec![MAIN],
            "Entry chunk should contain exactly [main.js]"
        );

        // Async chunk should contain exactly leaf.js
        let async_chunk = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Async)
            .unwrap();
        assert_eq!(
            async_chunk.modules,
            vec![LEAF],
            "Async chunk should contain exactly [leaf.js]"
        );

        // No shared chunks needed
        let shared_count = chunks
            .iter()
            .filter(|c| c.chunk_type == ChunkType::Shared)
            .count();
        assert_eq!(
            shared_count, 0,
            "No shared chunks needed for a simple leaf dynamic import"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Id-keying regression coverage (#1941)
    // ──────────────────────────────────────────────────────────────────

    /// #1941 — proves the pre-fix collapse mode is now structurally
    /// impossible: two DIFFERENT module ids that happen to render the SAME
    /// path string (simulating a pnpm-store symlink or jet.toml alias
    /// collision upstream in the resolver) must still partition correctly,
    /// because BFS membership is decided purely by `usize` id, never by
    /// `id_to_path`'s string spelling.
    #[test]
    fn split_chunks_partitions_by_id_not_by_path_spelling() {
        const ENTRY: usize = 0;
        const STATIC_DEP: usize = 1;
        const SPLIT_POINT: usize = 2;
        // Deliberately identical rendered path for two different ids: this
        // is exactly the shape a real graph can produce when a pnpm-store
        // symlink hop or a jet.toml alias resolves two logically distinct
        // module ids to the same display path.
        let id_to_path = id_map(&[
            (STATIC_DEP, "shared/collide.js"),
            (SPLIT_POINT, "shared/collide.js"),
        ]);

        let edges = vec![
            SplitEdgeId {
                from: ENTRY,
                to: STATIC_DEP,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: ENTRY,
                to: SPLIT_POINT,
                is_dynamic: true,
            },
        ];
        let all = vec![ENTRY, STATIC_DEP, SPLIT_POINT];

        let chunks = split_chunks(ENTRY, &edges, &all, &id_to_path);

        let entry_chunk = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Entry)
            .expect("entry chunk must exist");
        let async_chunk = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Async)
            .expect("async chunk must exist for the dynamic import target");

        assert!(
            entry_chunk.modules.contains(&STATIC_DEP),
            "entry chunk must contain the static dep by id: {:?}",
            entry_chunk.modules
        );
        assert!(
            !entry_chunk.modules.contains(&SPLIT_POINT),
            "entry chunk must NOT contain the dynamic-import target, even though it \
             renders the same path string as the static dep: {:?}",
            entry_chunk.modules
        );
        assert!(
            async_chunk.modules.contains(&SPLIT_POINT),
            "async chunk must contain the split-point module by id: {:?}",
            async_chunk.modules
        );
        assert!(
            !async_chunk.modules.contains(&STATIC_DEP),
            "async chunk must NOT contain the static dep, even though it renders the \
             same path string as the split point: {:?}",
            async_chunk.modules
        );

        // Every module id is assigned to exactly one chunk — no duplication,
        // no drops, regardless of the colliding path spelling.
        let all_assigned: Vec<usize> = chunks.iter().flat_map(|c| c.modules.clone()).collect();
        let unique: HashSet<usize> = all_assigned.iter().copied().collect();
        assert_eq!(
            all_assigned.len(),
            all.len(),
            "every module must be assigned exactly once: {all_assigned:?}"
        );
        assert_eq!(
            unique.len(),
            all_assigned.len(),
            "no module id may be duplicated across chunks: {all_assigned:?}"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Orphan-fallback visibility (#1941 AC2)
    // ──────────────────────────────────────────────────────────────────

    /// #1941 AC2 — the orphan fallback (a module the BFS never reaches,
    /// silently folded into the entry chunk) used to be completely silent.
    /// Prove the warn fires with the orphan count + a sample path when it
    /// actually happens.
    #[test]
    fn split_chunks_warns_with_orphan_count_and_sample_paths_when_modules_are_unreachable() {
        const ENTRY: usize = 0;
        const REACHED: usize = 1;
        const ORPHAN: usize = 2;
        let id_to_path = id_map(&[
            (ENTRY, "entry.js"),
            (REACHED, "reached.js"),
            (ORPHAN, "unreachable/orphan.js"),
        ]);
        let edges = vec![SplitEdgeId {
            from: ENTRY,
            to: REACHED,
            is_dynamic: false,
        }];
        // ORPHAN has no edge to/from anything but is still a compiled
        // module the caller expects accounted for.
        let all = vec![ENTRY, REACHED, ORPHAN];

        let buf = SharedBuf::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(buf.clone())
            .with_ansi(false)
            .without_time()
            .finish();

        let chunks = tracing::subscriber::with_default(subscriber, || {
            split_chunks(ENTRY, &edges, &all, &id_to_path)
        });

        // Functional: the orphan still lands in the entry chunk (fallback
        // behavior is preserved — only its visibility changes).
        assert!(
            chunks[0].modules.contains(&ORPHAN),
            "orphan module must still be folded into the entry chunk: {:?}",
            chunks[0].modules
        );

        let output = buf.contents();
        assert!(
            output.contains("orphan_count=1"),
            "warn must carry the orphan count: {output}"
        );
        assert!(
            output.contains("unreachable/orphan.js"),
            "warn must carry a sample of the orphaned module's path: {output}"
        );
    }

    /// #1941 AC2 — no orphan, no warn: every already-passing "clean" graph
    /// test above must not gain a stderr breadcrumb from this change.
    #[test]
    fn split_chunks_stays_silent_when_no_modules_are_orphaned() {
        const ENTRY: usize = 0;
        const UTIL: usize = 1;
        let id_to_path = id_map(&[(ENTRY, "entry.js"), (UTIL, "util.js")]);
        let edges = vec![SplitEdgeId {
            from: ENTRY,
            to: UTIL,
            is_dynamic: false,
        }];
        let all = vec![ENTRY, UTIL];

        let buf = SharedBuf::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(buf.clone())
            .with_ansi(false)
            .without_time()
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            split_chunks(ENTRY, &edges, &all, &id_to_path);
        });

        let output = buf.contents();
        assert!(
            output.is_empty(),
            "a fully-connected graph must not emit any orphan warning: {output}"
        );
    }

    // ── #1993: cycle_members / cross_chunk_referenced / dependency_order ──

    #[test]
    fn cycle_members_finds_two_module_scc() {
        const A: usize = 0;
        const B: usize = 1;
        const C: usize = 2;
        let ids: HashSet<usize> = HashSet::from([A, B, C]);
        let edges = vec![
            SplitEdgeId {
                from: A,
                to: B,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: B,
                to: A,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: A,
                to: C,
                is_dynamic: false,
            },
        ];

        assert_eq!(cycle_members(&ids, &edges), HashSet::from([A, B]));
    }

    #[test]
    fn cycle_members_finds_self_loop() {
        const A: usize = 0;
        const B: usize = 1;
        let ids: HashSet<usize> = HashSet::from([A, B]);
        let edges = vec![SplitEdgeId {
            from: A,
            to: A,
            is_dynamic: false,
        }];

        assert_eq!(cycle_members(&ids, &edges), HashSet::from([A]));
    }

    #[test]
    fn cycle_members_ignores_dynamic_edges() {
        const A: usize = 0;
        const B: usize = 1;
        let ids: HashSet<usize> = HashSet::from([A, B]);
        // Would be a 2-cycle if both edges were static; a dynamic edge is a
        // chunk boundary, not a same-scope initialization-order hazard.
        let edges = vec![
            SplitEdgeId {
                from: A,
                to: B,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: B,
                to: A,
                is_dynamic: true,
            },
        ];

        assert!(cycle_members(&ids, &edges).is_empty());
    }

    #[test]
    fn cross_chunk_referenced_finds_external_edge() {
        const OUTSIDE: usize = 99;
        const A: usize = 0;
        const B: usize = 1;
        let chunk_ids: HashSet<usize> = HashSet::from([A, B]);
        let edges = vec![
            SplitEdgeId {
                from: OUTSIDE,
                to: A,
                is_dynamic: true,
            },
            SplitEdgeId {
                from: A,
                to: B,
                is_dynamic: false,
            },
        ];

        assert_eq!(
            cross_chunk_referenced(&chunk_ids, &edges),
            HashSet::from([A])
        );
    }

    #[test]
    fn cross_chunk_referenced_ignores_internal_edges() {
        const A: usize = 0;
        const B: usize = 1;
        let chunk_ids: HashSet<usize> = HashSet::from([A, B]);
        let edges = vec![SplitEdgeId {
            from: A,
            to: B,
            is_dynamic: false,
        }];

        assert!(cross_chunk_referenced(&chunk_ids, &edges).is_empty());
    }

    #[test]
    fn cross_chunk_importers_finds_outgoing_external_edge() {
        const OUTSIDE: usize = 99; // e.g. a promoted shared/manual chunk module
        const A: usize = 0;
        const B: usize = 1;
        let chunk_ids: HashSet<usize> = HashSet::from([A, B]);
        let edges = vec![
            SplitEdgeId {
                from: A,
                to: OUTSIDE,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: A,
                to: B,
                is_dynamic: false,
            },
        ];

        assert_eq!(
            cross_chunk_importers(&chunk_ids, &edges),
            HashSet::from([A])
        );
    }

    #[test]
    fn cross_chunk_importers_ignores_internal_edges() {
        const A: usize = 0;
        const B: usize = 1;
        let chunk_ids: HashSet<usize> = HashSet::from([A, B]);
        let edges = vec![SplitEdgeId {
            from: A,
            to: B,
            is_dynamic: false,
        }];

        assert!(cross_chunk_importers(&chunk_ids, &edges).is_empty());
    }

    #[test]
    fn cross_chunk_importers_ignores_dynamic_edges() {
        // A lowered `import()` call is already async/deferred on its own
        // (`__jet__.dynamicImport(id)`), so it carries no synchronous
        // -ordering hazard and must not force its source module to the
        // registry.
        const OUTSIDE: usize = 99;
        const A: usize = 0;
        let chunk_ids: HashSet<usize> = HashSet::from([A]);
        let edges = vec![SplitEdgeId {
            from: A,
            to: OUTSIDE,
            is_dynamic: true,
        }];

        assert!(cross_chunk_importers(&chunk_ids, &edges).is_empty());
    }

    #[test]
    fn dependency_order_places_importer_before_dependency() {
        const ENTRY: usize = 0;
        const UTIL: usize = 1;
        let ids = vec![UTIL, ENTRY]; // deliberately out of order
        let edges = vec![SplitEdgeId {
            from: ENTRY,
            to: UTIL,
            is_dynamic: false,
        }];

        assert_eq!(dependency_order(&ids, &edges), vec![ENTRY, UTIL]);
    }

    #[test]
    fn dependency_order_falls_back_to_sorted_ids_on_residual_cycle() {
        // A caller that fails to exclude a real cycle first (a caller bug,
        // not a real graph shape) must not panic; it degrades to numeric
        // id order instead.
        const A: usize = 3;
        const B: usize = 1;
        let ids = vec![A, B];
        let edges = vec![
            SplitEdgeId {
                from: A,
                to: B,
                is_dynamic: false,
            },
            SplitEdgeId {
                from: B,
                to: A,
                is_dynamic: false,
            },
        ];

        assert_eq!(dependency_order(&ids, &edges), vec![B, A]);
    }
}
// CODEGEN-END

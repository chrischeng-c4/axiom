// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! Code splitting: partition modules into chunks at dynamic import boundaries.
//!
//! Detects `import()` calls in the module graph, uses them as split points,
//! and partitions modules into entry chunks and async chunks.

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
    /// Module paths included in this chunk.
    pub modules: Vec<PathBuf>,
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

/// Dependency edge for splitting analysis.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct SplitEdge {
    pub from: PathBuf,
    pub to: PathBuf,
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
/// `entry` is the entry point path.
/// `edges` describes the dependency graph with static/dynamic markers.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn split_chunks(entry: &PathBuf, edges: &[SplitEdge], all_modules: &[PathBuf]) -> Vec<Chunk> {
    // Build adjacency lists
    let mut static_deps: HashMap<&PathBuf, Vec<&PathBuf>> = HashMap::new();
    let mut dynamic_deps: HashMap<&PathBuf, Vec<&PathBuf>> = HashMap::new();

    for edge in edges {
        if edge.is_dynamic {
            dynamic_deps.entry(&edge.from).or_default().push(&edge.to);
        } else {
            static_deps.entry(&edge.from).or_default().push(&edge.to);
        }
    }

    // Find all dynamic import targets (split points)
    let split_points: HashSet<&PathBuf> = edges
        .iter()
        .filter(|e| e.is_dynamic)
        .map(|e| &e.to)
        .collect();

    // BFS from entry following only static imports → entry chunk
    let entry_modules = bfs_static(entry, &static_deps, &split_points);

    // BFS from each split point → async chunks
    let mut async_chunks: Vec<(PathBuf, HashSet<PathBuf>)> = Vec::new();
    for &sp in &split_points {
        let chunk_modules = bfs_static(sp, &static_deps, &split_points);
        async_chunks.push((sp.clone(), chunk_modules));
    }

    // Detect shared modules (in 2+ chunks)
    let mut module_count: HashMap<PathBuf, usize> = HashMap::new();
    for m in &entry_modules {
        *module_count.entry(m.clone()).or_default() += 1;
    }
    for (_, modules) in &async_chunks {
        for m in modules {
            *module_count.entry(m.clone()).or_default() += 1;
        }
    }

    let shared: HashSet<PathBuf> = module_count
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .map(|(path, _)| path)
        .collect();

    // Build result chunks
    let mut chunks = Vec::new();

    // Entry chunk (exclude shared modules)
    let entry_mods: Vec<PathBuf> = entry_modules
        .into_iter()
        .filter(|m| !shared.contains(m))
        .collect();
    let async_chunk_names: Vec<String> =
        async_chunks.iter().map(|(sp, _)| chunk_name(sp)).collect();

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
        let shared_mods: Vec<PathBuf> = shared.into_iter().collect();
        chunks.push(Chunk {
            name: "shared".to_string(),
            chunk_type: ChunkType::Shared,
            modules: shared_mods,
            imports: Vec::new(),
        });
    }

    // Collect shared module paths for filtering
    let shared_paths: HashSet<PathBuf> = chunks
        .iter()
        .filter(|c| c.chunk_type == ChunkType::Shared)
        .flat_map(|c| c.modules.iter().cloned())
        .collect();

    // Async chunks (exclude shared)
    for (sp, modules) in async_chunks {
        let filtered: Vec<PathBuf> = modules
            .into_iter()
            .filter(|m| !shared_paths.contains(m))
            .collect();
        let name = chunk_name(&sp);
        let mut imports = Vec::new();
        if !shared_paths.is_empty() {
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
    let assigned: HashSet<PathBuf> = chunks
        .iter()
        .flat_map(|c| c.modules.iter().cloned())
        .collect();
    let orphans: Vec<PathBuf> = all_modules
        .iter()
        .filter(|m| !assigned.contains(*m))
        .cloned()
        .collect();
    if !orphans.is_empty() {
        chunks[0].modules.extend(orphans);
    }

    chunks
}

/// Split modules into chunks with manual chunk routing and preload hint generation.
///
/// Enhanced version of `split_chunks` that supports:
/// - Manual chunks: modules matching glob patterns are routed to named chunks
/// - Preload hints: returns metadata for `<link rel="modulepreload">` generation
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn split_chunks_with_config(
    entry: &PathBuf,
    edges: &[SplitEdge],
    all_modules: &[PathBuf],
    manual_config: &ManualChunkConfig,
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
    let mut manual_assignments: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let mut manually_assigned: HashSet<PathBuf> = HashSet::new();

    for module in all_modules {
        let path_str = module.to_string_lossy();
        for (chunk_name, matcher) in &manual_matchers {
            if matcher.is_match(path_str.as_ref()) {
                manual_assignments
                    .entry(chunk_name.clone())
                    .or_default()
                    .push(module.clone());
                manually_assigned.insert(module.clone());
                break; // First matching manual chunk wins
            }
        }
    }

    // Run normal splitting on remaining modules
    let remaining_modules: Vec<PathBuf> = all_modules
        .iter()
        .filter(|m| !manually_assigned.contains(*m))
        .cloned()
        .collect();

    let mut chunks = split_chunks(entry, edges, &remaining_modules);

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
    root: &PathBuf,
    static_deps: &HashMap<&PathBuf, Vec<&PathBuf>>,
    split_points: &HashSet<&PathBuf>,
) -> HashSet<PathBuf> {
    let mut visited = HashSet::new();
    let mut queue = vec![root];
    visited.insert(root.clone());

    while let Some(current) = queue.pop() {
        if let Some(deps) = static_deps.get(current) {
            for &dep in deps {
                if !visited.contains(dep) && !split_points.contains(dep) {
                    visited.insert(dep.clone());
                    queue.push(dep);
                }
            }
        }
    }

    visited
}

/// Generate a chunk name from a module path.
fn chunk_name(path: &PathBuf) -> String {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("chunk");
    format!("chunk-{}", stem)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_dynamic_imports() {
        let entry = PathBuf::from("main.js");
        let edges = vec![SplitEdge {
            from: PathBuf::from("main.js"),
            to: PathBuf::from("util.js"),
            is_dynamic: false,
        }];
        let all = vec![PathBuf::from("main.js"), PathBuf::from("util.js")];

        let chunks = split_chunks(&entry, &edges, &all);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].name, "main");
        assert_eq!(chunks[0].modules.len(), 2);
    }

    #[test]
    fn test_dynamic_import_split() {
        let entry = PathBuf::from("main.js");
        let edges = vec![
            SplitEdge {
                from: PathBuf::from("main.js"),
                to: PathBuf::from("util.js"),
                is_dynamic: false,
            },
            SplitEdge {
                from: PathBuf::from("main.js"),
                to: PathBuf::from("lazy.js"),
                is_dynamic: true,
            },
        ];
        let all = vec![
            PathBuf::from("main.js"),
            PathBuf::from("util.js"),
            PathBuf::from("lazy.js"),
        ];

        let chunks = split_chunks(&entry, &edges, &all);
        // Should have entry chunk + async chunk
        assert!(chunks.len() >= 2);

        let entry_chunk = chunks.iter().find(|c| c.name == "main").unwrap();
        assert!(entry_chunk.modules.contains(&PathBuf::from("main.js")));
        assert!(entry_chunk.modules.contains(&PathBuf::from("util.js")));

        let async_chunk = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Async)
            .unwrap();
        assert!(async_chunk.modules.contains(&PathBuf::from("lazy.js")));
    }

    #[test]
    fn test_shared_module_extraction() {
        let entry = PathBuf::from("main.js");
        let shared_mod = PathBuf::from("shared.js");
        let edges = vec![
            SplitEdge {
                from: PathBuf::from("main.js"),
                to: shared_mod.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: PathBuf::from("main.js"),
                to: PathBuf::from("lazy.js"),
                is_dynamic: true,
            },
            SplitEdge {
                from: PathBuf::from("lazy.js"),
                to: shared_mod.clone(),
                is_dynamic: false,
            },
        ];
        let all = vec![
            PathBuf::from("main.js"),
            shared_mod.clone(),
            PathBuf::from("lazy.js"),
        ];

        let chunks = split_chunks(&entry, &edges, &all);

        // Should have shared chunk
        let shared_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Shared);
        assert!(shared_chunk.is_some());
        assert!(shared_chunk.unwrap().modules.contains(&shared_mod));
    }

    #[test]
    fn test_chunk_naming() {
        assert_eq!(chunk_name(&PathBuf::from("src/lazy.js")), "chunk-lazy");
        assert_eq!(chunk_name(&PathBuf::from("dialog.tsx")), "chunk-dialog");
    }

    // ──────────────────────────────────────────────────────────────────
    // Manual chunks tests (R9 / T13)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_manual_chunks_routing() {
        let entry = PathBuf::from("main.js");
        let react_mod = PathBuf::from("node_modules/react/index.js");
        let react_dom_mod = PathBuf::from("node_modules/react-dom/index.js");
        let util_mod = PathBuf::from("src/util.js");

        let edges = vec![
            SplitEdge {
                from: entry.clone(),
                to: react_mod.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: entry.clone(),
                to: react_dom_mod.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: entry.clone(),
                to: util_mod.clone(),
                is_dynamic: false,
            },
        ];
        let all = vec![
            entry.clone(),
            react_mod.clone(),
            react_dom_mod.clone(),
            util_mod.clone(),
        ];

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

        let result = split_chunks_with_config(&entry, &edges, &all, &manual_config);

        // Find the vendor chunk
        let vendor_chunk = result.chunks.iter().find(|c| c.name == "vendor");
        assert!(
            vendor_chunk.is_some(),
            "Vendor chunk should exist. Chunks: {:?}",
            result.chunks.iter().map(|c| &c.name).collect::<Vec<_>>()
        );
        let vendor = vendor_chunk.unwrap();
        assert!(
            vendor.modules.contains(&react_mod),
            "React should be in vendor chunk"
        );
        assert!(
            vendor.modules.contains(&react_dom_mod),
            "React-DOM should be in vendor chunk"
        );

        // Entry chunk should NOT contain react modules
        let entry_chunk = result.chunks.iter().find(|c| c.name == "main").unwrap();
        assert!(
            !entry_chunk.modules.contains(&react_mod),
            "React should NOT be in entry chunk"
        );
        assert!(
            !entry_chunk.modules.contains(&react_dom_mod),
            "React-DOM should NOT be in entry chunk"
        );
    }

    #[test]
    fn test_manual_chunks_empty_config() {
        let entry = PathBuf::from("main.js");
        let edges = vec![SplitEdge {
            from: entry.clone(),
            to: PathBuf::from("util.js"),
            is_dynamic: false,
        }];
        let all = vec![entry.clone(), PathBuf::from("util.js")];

        let manual_config = ManualChunkConfig::default();
        let result = split_chunks_with_config(&entry, &edges, &all, &manual_config);

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
        let entry = PathBuf::from("main.js");
        let react_mod = PathBuf::from("node_modules/react/index.js");
        let util_mod = PathBuf::from("src/util.js");

        let edges = vec![
            SplitEdge {
                from: entry.clone(),
                to: react_mod.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: entry.clone(),
                to: util_mod.clone(),
                is_dynamic: false,
            },
        ];
        let all = vec![entry.clone(), react_mod.clone(), util_mod.clone()];

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

        let result = split_chunks_with_config(&entry, &edges, &all, &manual_config);
        let vendor =
            result.chunks.iter().find(|c| c.name == "vendor").expect(
                "vendor chunk must still be produced when only one of N patterns is invalid",
            );
        assert!(
            vendor.modules.contains(&react_mod),
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
        let entry = PathBuf::from("main.js");
        let react_mod = PathBuf::from("node_modules/react/index.js");
        let util_mod = PathBuf::from("src/util.js");

        let edges = vec![
            SplitEdge {
                from: entry.clone(),
                to: react_mod.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: entry.clone(),
                to: util_mod.clone(),
                is_dynamic: false,
            },
        ];
        let all = vec![entry.clone(), react_mod.clone(), util_mod.clone()];

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

        let result = split_chunks_with_config(&entry, &edges, &all, &manual_config);
        let vendor = result
            .chunks
            .iter()
            .find(|c| c.name == "vendor")
            .expect("healthy sibling chunk must still be emitted alongside the broken one");
        assert!(
            vendor.modules.contains(&react_mod),
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
        let entry = PathBuf::from("main.js");
        let shared_mod = PathBuf::from("shared.js");
        let lazy_mod = PathBuf::from("lazy.js");

        let edges = vec![
            SplitEdge {
                from: entry.clone(),
                to: shared_mod.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: entry.clone(),
                to: lazy_mod.clone(),
                is_dynamic: true,
            },
            SplitEdge {
                from: lazy_mod.clone(),
                to: shared_mod.clone(),
                is_dynamic: false,
            },
        ];
        let all = vec![entry.clone(), shared_mod.clone(), lazy_mod.clone()];

        let manual_config = ManualChunkConfig::default();
        let result = split_chunks_with_config(&entry, &edges, &all, &manual_config);

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
        let entry_a = PathBuf::from("entry_a.js");
        let entry_b = PathBuf::from("entry_b.js");
        let shared_util = PathBuf::from("shared_util.js");

        let all = vec![entry_a.clone(), entry_b.clone(), shared_util.clone()];

        // To trigger shared extraction, shared_util must appear in 2+ chunks.
        // We simulate multi-entry by having entry_b as a dynamic import target,
        // so both the entry chunk and async chunk reference shared_util.
        let edges = vec![
            SplitEdge {
                from: entry_a.clone(),
                to: shared_util.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: entry_a.clone(),
                to: entry_b.clone(),
                is_dynamic: true,
            },
            SplitEdge {
                from: entry_b.clone(),
                to: shared_util.clone(),
                is_dynamic: false,
            },
        ];

        let chunks = split_chunks(&entry_a, &edges, &all);

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
            shared_chunk.unwrap().modules.contains(&shared_util),
            "shared_util should be in the shared chunk"
        );

        // Entry chunk must NOT contain shared_util
        let entry_chunk = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Entry)
            .unwrap();
        assert!(
            !entry_chunk.modules.contains(&shared_util),
            "Entry chunk must not contain shared_util (it should be extracted to shared)"
        );

        // shared_util appears in exactly one chunk
        let chunks_containing_shared: Vec<&Chunk> = chunks
            .iter()
            .filter(|c| c.modules.contains(&shared_util))
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
        let entry_a = PathBuf::from("entry_a.js");
        let entry_b = PathBuf::from("entry_b.js");
        let mod_a = PathBuf::from("mod_a.js");
        let mod_b = PathBuf::from("mod_b.js");
        let common = PathBuf::from("common.js");

        // Edges from entry_a's perspective: entry_a → mod_a, entry_a → common,
        // entry_a → entry_b (dynamic so entry_b becomes async chunk),
        // entry_b → mod_b, entry_b → common
        let edges = vec![
            SplitEdge {
                from: entry_a.clone(),
                to: mod_a.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: entry_a.clone(),
                to: common.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: entry_a.clone(),
                to: entry_b.clone(),
                is_dynamic: true,
            },
            SplitEdge {
                from: entry_b.clone(),
                to: mod_b.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: entry_b.clone(),
                to: common.clone(),
                is_dynamic: false,
            },
        ];
        let all = vec![
            entry_a.clone(),
            entry_b.clone(),
            mod_a.clone(),
            mod_b.clone(),
            common.clone(),
        ];

        // Split from entry_a
        let chunks_a = split_chunks(&entry_a, &edges, &all);

        // Entry chunk (from entry_a) should contain entry_a and mod_a, NOT mod_b
        let entry_chunk_a = chunks_a
            .iter()
            .find(|c| c.chunk_type == ChunkType::Entry)
            .unwrap();
        assert!(
            entry_chunk_a.modules.contains(&entry_a),
            "Entry chunk should contain entry_a"
        );
        assert!(
            entry_chunk_a.modules.contains(&mod_a),
            "Entry chunk should contain mod_a (static dep of entry_a)"
        );
        assert!(
            !entry_chunk_a.modules.contains(&mod_b),
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
            shared_chunk_a.unwrap().modules.contains(&common),
            "common.js should be in the shared chunk"
        );

        // Similarly, split from entry_b to verify disjoint behavior
        let edges_b = vec![
            SplitEdge {
                from: entry_b.clone(),
                to: mod_b.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: entry_b.clone(),
                to: common.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: entry_b.clone(),
                to: entry_a.clone(),
                is_dynamic: true,
            },
            SplitEdge {
                from: entry_a.clone(),
                to: mod_a.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: entry_a.clone(),
                to: common.clone(),
                is_dynamic: false,
            },
        ];

        let chunks_b = split_chunks(&entry_b, &edges_b, &all);

        let entry_chunk_b = chunks_b
            .iter()
            .find(|c| c.chunk_type == ChunkType::Entry)
            .unwrap();
        assert!(
            entry_chunk_b.modules.contains(&entry_b),
            "Entry chunk should contain entry_b"
        );
        assert!(
            entry_chunk_b.modules.contains(&mod_b),
            "Entry chunk should contain mod_b (static dep of entry_b)"
        );
        assert!(
            !entry_chunk_b.modules.contains(&mod_a),
            "Entry chunk from entry_b should NOT contain mod_a"
        );

        // common in shared for this split too
        let shared_chunk_b = chunks_b.iter().find(|c| c.chunk_type == ChunkType::Shared);
        assert!(
            shared_chunk_b.is_some(),
            "Shared chunk should exist for common.js in entry_b split"
        );
        assert!(
            shared_chunk_b.unwrap().modules.contains(&common),
            "common.js should be in the shared chunk for entry_b split"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Async chunk preload metadata tests (TR3 / S3)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_preload_hints_multi_chunk() {
        // S3: entry → shared (static), entry → lazy (dynamic), lazy → shared (static)
        let entry = PathBuf::from("entry.js");
        let shared = PathBuf::from("shared.js");
        let lazy = PathBuf::from("lazy.js");

        let edges = vec![
            SplitEdge {
                from: entry.clone(),
                to: shared.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: entry.clone(),
                to: lazy.clone(),
                is_dynamic: true,
            },
            SplitEdge {
                from: lazy.clone(),
                to: shared.clone(),
                is_dynamic: false,
            },
        ];
        let all = vec![entry.clone(), shared.clone(), lazy.clone()];

        let manual_config = ManualChunkConfig::default();
        let result = split_chunks_with_config(&entry, &edges, &all, &manual_config);

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
        let main = PathBuf::from("main.js");
        let a = PathBuf::from("a.js");
        let b = PathBuf::from("b.js");

        let edges = vec![
            SplitEdge {
                from: main.clone(),
                to: a.clone(),
                is_dynamic: true,
            },
            SplitEdge {
                from: a.clone(),
                to: b.clone(),
                is_dynamic: true,
            },
            SplitEdge {
                from: b.clone(),
                to: a.clone(),
                is_dynamic: true,
            },
        ];
        let all = vec![main.clone(), a.clone(), b.clone()];

        // This must return without infinite loop
        let chunks = split_chunks(&main, &edges, &all);

        // Entry chunk should contain only main.js
        let entry_chunk = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Entry)
            .unwrap();
        assert!(
            entry_chunk.modules.contains(&main),
            "Entry chunk should contain main.js"
        );
        assert!(
            !entry_chunk.modules.contains(&a),
            "Entry chunk should NOT contain a.js (it's a dynamic import target)"
        );
        assert!(
            !entry_chunk.modules.contains(&b),
            "Entry chunk should NOT contain b.js (it's a dynamic import target)"
        );

        // Async chunks should exist for both a and b
        let async_chunks: Vec<&Chunk> = chunks
            .iter()
            .filter(|c| c.chunk_type == ChunkType::Async)
            .collect();
        let async_modules: HashSet<PathBuf> = async_chunks
            .iter()
            .flat_map(|c| c.modules.iter().cloned())
            .collect();
        assert!(
            async_modules.contains(&a),
            "a.js should be in an async chunk. Async chunks: {:?}",
            async_chunks
                .iter()
                .map(|c| (&c.name, &c.modules))
                .collect::<Vec<_>>()
        );
        assert!(
            async_modules.contains(&b),
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
        let entry = PathBuf::from("entry.js");
        let a = PathBuf::from("a.js");
        let b = PathBuf::from("b.js");
        let c = PathBuf::from("c.js");

        let edges = vec![
            SplitEdge {
                from: entry.clone(),
                to: a.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: entry.clone(),
                to: b.clone(),
                is_dynamic: true,
            },
            SplitEdge {
                from: a.clone(),
                to: c.clone(),
                is_dynamic: false,
            },
            SplitEdge {
                from: b.clone(),
                to: c.clone(),
                is_dynamic: false,
            },
        ];
        let all = vec![entry.clone(), a.clone(), b.clone(), c.clone()];

        let chunks = split_chunks(&entry, &edges, &all);

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
            shared_chunk.unwrap().modules.contains(&c),
            "c.js should be in the shared chunk"
        );

        // a.js should be in the entry chunk (static dep of entry)
        let entry_chunk = chunks
            .iter()
            .find(|ch| ch.chunk_type == ChunkType::Entry)
            .unwrap();
        assert!(
            entry_chunk.modules.contains(&a),
            "a.js should be in the entry chunk"
        );

        // b.js should be in an async chunk (dynamic import)
        let async_chunks: Vec<&Chunk> = chunks
            .iter()
            .filter(|ch| ch.chunk_type == ChunkType::Async)
            .collect();
        let async_modules: HashSet<PathBuf> = async_chunks
            .iter()
            .flat_map(|ch| ch.modules.iter().cloned())
            .collect();
        assert!(
            async_modules.contains(&b),
            "b.js should be in an async chunk"
        );

        // Neither entry nor async chunks should contain c.js (it's shared)
        assert!(
            !entry_chunk.modules.contains(&c),
            "Entry chunk should NOT contain c.js (it's shared)"
        );
        assert!(
            !async_modules.contains(&c),
            "Async chunks should NOT contain c.js (it's shared)"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Leaf dynamic import produces single-module async chunk (TR6 / S6)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_leaf_dynamic_import_single_chunk() {
        // S6: main → leaf (dynamic), leaf has no further deps
        let main = PathBuf::from("main.js");
        let leaf = PathBuf::from("leaf.js");

        let edges = vec![SplitEdge {
            from: main.clone(),
            to: leaf.clone(),
            is_dynamic: true,
        }];
        let all = vec![main.clone(), leaf.clone()];

        let chunks = split_chunks(&main, &edges, &all);

        // Entry chunk should contain exactly main.js
        let entry_chunk = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Entry)
            .unwrap();
        assert_eq!(
            entry_chunk.modules,
            vec![main.clone()],
            "Entry chunk should contain exactly [main.js]"
        );

        // Async chunk should contain exactly leaf.js
        let async_chunk = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Async)
            .unwrap();
        assert_eq!(
            async_chunk.modules,
            vec![leaf.clone()],
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
}
// CODEGEN-END

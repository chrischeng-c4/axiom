// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
//! HMR rebuild glue — owns an `IncrementalTransformer`, walks the
//! dev-server module graph's reverse-dep edges, and emits one
//! `hmr_rebuild` JSON line per file-change event on the existing
//! `tracing` channel.
//!
//! @spec `.aw/tech-design/projects/jet/logic/bundler-incremental-rebuild.md`
//!     §"Slice 4c — IncrementalRebuilder glue + log emit".
//! @issue #1250 — closes the R2/R5 wiring half: the primitives shipped
//!     in Slice 4a (`RebuildMetrics::log_line`,
//!     `bundler::graph::ModuleGraph::dependents_of`) and Slice 4b
//!     (`dev_server::module_graph::ModuleGraph::dependents_of`) are
//!     consumed here.
//!
//! The rebuilder is intentionally a thin assembly layer: no new
//! algorithms, no new traversal logic. It simply ties the cache
//! (`IncrementalTransformer`), the reverse-dep walker
//! (`ModuleGraph::dependents_of`), and the log-line emit
//! (`RebuildMetrics::log_line`) together. Watcher hookup is the
//! caller's job.

use anyhow::Result;
use std::path::Path;

use crate::transform::incremental::{IncrementalTransformer, RebuildMetrics};

use super::module_graph::ModuleGraph;

/// Outcome of one rebuild call. Returned to the caller AND emitted
/// as a `tracing::info!` line so dev-server log collectors can
/// ingest the metric without waiting for the caller's response.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebuildOutcome {
    /// URLs the caller should mark dirty in the dev server's
    /// cache. Includes the changed URL itself and every transitive
    /// importer (per `ModuleGraph::dependents_of`).
    pub invalidated: Vec<String>,
    /// The `RebuildMetrics::log_line` JSON string emitted on the
    /// `tracing` channel. Returned for ergonomic test assertions
    /// without pulling in `tracing-test`.
    pub log_line: String,
}

/// Wraps an `IncrementalTransformer` with the dev-server's
/// reverse-dep walker. One instance per dev-server session; not
/// `Sync`-safe — the dev server holds it behind its own lock.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub struct IncrementalRebuilder {
    transformer: IncrementalTransformer,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl IncrementalRebuilder {
    pub fn new(language: tree_sitter::Language) -> Result<Self> {
        Ok(Self {
            transformer: IncrementalTransformer::new(language)?,
        })
    }

    /// Drive one HMR rebuild from a watcher event.
    ///
    /// Steps:
    ///   1. Compute `dependents_of(changed_url)` — the full
    ///      transitive importer set the cache must drop.
    ///   2. Drop cache entries for the changed URL's filesystem
    ///      path and every dependent's path.
    ///   3. Re-run `transform_for_path` for the changed file
    ///      itself (callers that want eager re-transform of the
    ///      dependents do so via separate `rebuild_one` calls).
    ///   4. Emit one `hmr_rebuild` JSON line on `tracing::info!`
    ///      and return it in the outcome.
    ///
    /// `changed_url` is the dev-server URL (e.g. `/src/App.tsx`);
    /// `changed_file` is the absolute filesystem path used for the
    /// cache key. The two diverge in served-asset setups, which is
    /// why the dev-server graph stores both.
    ///
    /// The function is best-effort: if the changed file's
    /// extension is unsupported (`.css`, `.json`, …), the
    /// transform step is skipped, the metrics still increment,
    /// and the log line is still emitted with `wall_ms = 0`.
    pub fn rebuild(
        &mut self,
        changed_url: &str,
        changed_file: &Path,
        new_source: &str,
        graph: &ModuleGraph,
    ) -> Result<RebuildOutcome> {
        let dependents = graph.dependents_of(changed_url);

        // Drop cache for the changed file + every dependent's file.
        self.transformer
            .invalidate(&changed_file.display().to_string());
        for dep_url in &dependents {
            if let Some(node) = graph.get(dep_url) {
                self.transformer.invalidate(&node.file);
            }
        }

        let mut invalidated = Vec::with_capacity(1 + dependents.len());
        invalidated.push(changed_url.to_string());
        invalidated.extend(dependents);

        // Re-transform the changed file (best-effort — unsupported
        // extensions like `.css` short-circuit without erroring).
        let started = std::time::Instant::now();
        if Self::is_supported(changed_file) {
            // The cache was just invalidated for this path, so this
            // is a guaranteed miss; the resulting `transformed`
            // text is reseated in the cache for future hits.
            let _ = self
                .transformer
                .transform_for_path(changed_file, new_source, None)?;
        }
        let wall_ms = started.elapsed().as_millis() as u64;

        let metrics = self.transformer.metrics_snapshot();
        let log_line = metrics.log_line(changed_file, wall_ms);
        tracing::info!(target: "jet::hmr", "{}", log_line);

        Ok(RebuildOutcome {
            invalidated,
            log_line,
        })
    }

    /// Snapshot of the running metrics. Equivalent to
    /// `IncrementalTransformer::metrics_snapshot` — exposed at the
    /// rebuilder level so dev-server status endpoints don't need
    /// to reach into the inner transformer.
    pub fn metrics_snapshot(&self) -> RebuildMetrics {
        self.transformer.metrics_snapshot()
    }

    fn is_supported(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("ts") | Some("tsx") | Some("js") | Some("jsx") | Some("mjs") | Some("cjs")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn js_lang() -> tree_sitter::Language {
        tree_sitter_javascript::LANGUAGE.into()
    }

    fn make_rebuilder() -> IncrementalRebuilder {
        IncrementalRebuilder::new(js_lang()).unwrap()
    }

    fn graph_with_chain() -> ModuleGraph {
        // entry → barrel → leaf
        let mut g = ModuleGraph::new();
        g.add_module(
            "/src/entry.tsx",
            "/abs/src/entry.tsx",
            &["/src/barrel.ts".to_string()],
        );
        g.add_module(
            "/src/barrel.ts",
            "/abs/src/barrel.ts",
            &["/src/leaf.ts".to_string()],
        );
        g.add_module("/src/leaf.ts", "/abs/src/leaf.ts", &[]);
        g
    }

    #[test]
    fn rebuild_returns_self_plus_dependents_in_invalidated_set() {
        let mut r = make_rebuilder();
        let g = graph_with_chain();
        let out = r
            .rebuild(
                "/src/leaf.ts",
                Path::new("/abs/src/leaf.ts"),
                "export const a = 1;\n",
                &g,
            )
            .unwrap();
        let mut ordered = out.invalidated.clone();
        ordered.sort();
        assert_eq!(
            ordered,
            vec![
                "/src/barrel.ts".to_string(),
                "/src/entry.tsx".to_string(),
                "/src/leaf.ts".to_string(),
            ]
        );
    }

    #[test]
    fn rebuild_emits_spec_shape_log_line() {
        let mut r = make_rebuilder();
        let g = graph_with_chain();
        let out = r
            .rebuild(
                "/src/leaf.ts",
                Path::new("/abs/src/leaf.ts"),
                "export const a = 1;\n",
                &g,
            )
            .unwrap();
        // Must round-trip as JSON and contain all spec'd fields.
        let parsed: serde_json::Value = serde_json::from_str(&out.log_line).unwrap();
        assert_eq!(parsed["event"], "hmr_rebuild");
        assert_eq!(parsed["path"], "/abs/src/leaf.ts");
        assert!(parsed["hits"].is_u64());
        assert!(parsed["misses"].is_u64());
        assert!(parsed["bytes_reused"].is_u64());
        assert!(parsed["wall_ms"].is_u64());
    }

    #[test]
    fn rebuild_drops_cache_entry_for_changed_file() {
        // Prime the cache, then change the source. A fresh transform
        // for the new source must register as a miss (cache was
        // dropped before re-transform), and the metrics must reflect
        // exactly one miss for the priming + one miss for the
        // post-rebuild re-prime call.
        let mut r = make_rebuilder();
        let g = graph_with_chain();

        // Prime: first call is a miss.
        let _ = r
            .transformer
            .transform_for_path(Path::new("/abs/src/leaf.ts"), "export const a = 1;\n", None)
            .unwrap();
        assert_eq!(r.metrics_snapshot().misses, 1);

        // Rebuild with a new source — the rebuild itself runs a
        // re-transform internally (miss #2).
        let _ = r
            .rebuild(
                "/src/leaf.ts",
                Path::new("/abs/src/leaf.ts"),
                "export const a = 2;\n",
                &g,
            )
            .unwrap();
        assert_eq!(r.metrics_snapshot().misses, 2);

        // A subsequent call with the *new* source must be a hit
        // (the rebuild reseated it).
        let _ = r
            .transformer
            .transform_for_path(Path::new("/abs/src/leaf.ts"), "export const a = 2;\n", None)
            .unwrap();
        let m = r.metrics_snapshot();
        assert_eq!(m.misses, 2);
        assert_eq!(m.hits, 1);
    }

    #[test]
    fn rebuild_drops_cache_for_every_dependent_path() {
        // Prime caches for entry, barrel, leaf. Then rebuild leaf —
        // the dependent caches must all be dropped (so a subsequent
        // call on barrel.ts is a miss, not a stale hit).
        let mut r = make_rebuilder();
        let g = graph_with_chain();

        for (file, src) in [
            (
                "/abs/src/entry.tsx",
                "import { a } from './barrel';\nexport const e = a;\n",
            ),
            ("/abs/src/barrel.ts", "export { a } from './leaf';\n"),
            ("/abs/src/leaf.ts", "export const a = 1;\n"),
        ] {
            let _ = r
                .transformer
                .transform_for_path(Path::new(file), src, None)
                .unwrap();
        }
        assert_eq!(r.metrics_snapshot().misses, 3);
        assert_eq!(r.metrics_snapshot().hits, 0);

        // Hit a primed path — confirms the cache is live.
        let _ = r
            .transformer
            .transform_for_path(
                Path::new("/abs/src/barrel.ts"),
                "export { a } from './leaf';\n",
                None,
            )
            .unwrap();
        assert_eq!(r.metrics_snapshot().hits, 1);

        // Rebuild leaf — invalidates leaf, barrel, entry.
        let _ = r
            .rebuild(
                "/src/leaf.ts",
                Path::new("/abs/src/leaf.ts"),
                "export const a = 99;\n",
                &g,
            )
            .unwrap();

        // Same source on barrel must now miss (cache was dropped).
        let pre = r.metrics_snapshot();
        let _ = r
            .transformer
            .transform_for_path(
                Path::new("/abs/src/barrel.ts"),
                "export { a } from './leaf';\n",
                None,
            )
            .unwrap();
        let post = r.metrics_snapshot();
        assert_eq!(
            post.misses,
            pre.misses + 1,
            "barrel cache must have been dropped, got hit instead"
        );
    }

    #[test]
    fn rebuild_handles_unsupported_extension_without_transform() {
        // .css is not a transformer-supported extension; the rebuild
        // must still emit the log line and the invalidated set, but
        // the transform step is skipped (no error, no miss).
        let mut g = ModuleGraph::new();
        g.add_module("/src/styles.css", "/abs/src/styles.css", &[]);
        let mut r = make_rebuilder();
        let out = r
            .rebuild(
                "/src/styles.css",
                Path::new("/abs/src/styles.css"),
                "body{}",
                &g,
            )
            .unwrap();
        assert_eq!(out.invalidated, vec!["/src/styles.css".to_string()]);
        let parsed: serde_json::Value = serde_json::from_str(&out.log_line).unwrap();
        assert_eq!(parsed["event"], "hmr_rebuild");
        // No transform was run, so no miss on this call.
        assert_eq!(r.metrics_snapshot().misses, 0);
    }

    #[test]
    fn rebuild_unknown_url_invalidates_only_self() {
        // A path the graph has never seen yields no dependents —
        // invalidated set is just the URL itself.
        let mut r = make_rebuilder();
        let g = ModuleGraph::new();
        let out = r
            .rebuild(
                "/src/ghost.ts",
                Path::new("/abs/src/ghost.ts"),
                "export const x = 1;\n",
                &g,
            )
            .unwrap();
        assert_eq!(out.invalidated, vec!["/src/ghost.ts".to_string()]);
    }
}
// CODEGEN-END

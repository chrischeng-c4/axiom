// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
// CODEGEN-BEGIN
//! Incremental transformation using Tree-sitter's incremental parsing.
//!
//! @spec `.aw/tech-design/projects/jet/logic/bundler-incremental-rebuild.md`
//!     §"Slice 2 — content-hashed parse-tree cache".
//! @issue #1250 — Slice 2: replaces the prior `HashMap<String, Tree>`
//!     cache (parser-only, never wired up beyond a smoke test) with a
//!     `(path, sha256(source))`-keyed cache that stores the parsed
//!     tree + the transformed text side-by-side, plus private
//!     `RebuildMetrics` counters Slice 4 will surface on the
//!     dev-server log.
//!
//! Slice 3 (shipped) wires `transform_tree` to the existing
//! `transform_jsx` / `transform_tsx` / `transform_typescript` entry
//! points behind a closed `Ext` enum keyed off the file extension.
//! That swap is transparent to the cache layer ↑ — same-source hits
//! return the cached transformed text without re-running whichever
//! transformer produced it.

use anyhow::Result;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tree_sitter::{InputEdit, Parser, Tree};

use super::{jsx, transform_tsx, typescript, TransformOptions};

/// Cache identity for one transformed module. Both fields matter:
/// path keeps two modules with identical content but different
/// locations distinct (so a cross-path collision never silently
/// reuses a transform), and the content hash forces a miss whenever
/// the source bytes drift — including whitespace, BOM edits, or
/// encoding bumps the parser would otherwise pretend are no-ops.
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub path: PathBuf,
    pub content_hash: [u8; 32],
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
impl CacheKey {
    pub fn new(path: impl Into<PathBuf>, source: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        let digest = hasher.finalize();
        let mut content_hash = [0u8; 32];
        content_hash.copy_from_slice(&digest);
        Self {
            path: path.into(),
            content_hash,
        }
    }
}

/// Value side of the transform cache. Slice 2 stores the parsed tree
/// + the transformed text + the wall-clock the transform took. Slice
/// 4's metrics surface uses `last_transform_us` to report
/// `bytes_reused` without re-running the transform on every cache
/// hit.
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
#[derive(Debug, Clone)]
pub struct CachedEntry {
    pub tree: Tree,
    pub transformed: String,
    pub last_transform_us: u64,
}

/// Running counters surfaced on the dev-server log. Slice 2 made
/// the counters increment; Slice 4 (this slice) wires `log_line`
/// for the dev-server emit path. `metrics_snapshot()` returns a
/// `Copy` view so callers can read without holding a borrow on
/// the transformer.
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RebuildMetrics {
    pub hits: u64,
    pub misses: u64,
    /// Total bytes of cached `transformed` text returned without
    /// re-running the transform. Sums the size of each hit's
    /// returned string.
    pub bytes_reused: u64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
impl RebuildMetrics {
    /// Render one `hmr_rebuild` JSON line per the spec's R5 shape:
    /// `{"event":"hmr_rebuild","path":...,"hits":...,"misses":...,
    ///   "bytes_reused":...,"wall_ms":...}`.
    ///
    /// Hand-formatted (not `serde_json`) because the field set is
    /// closed and tiny; an extra serde dep on this hot path is
    /// not worth it. The key order is fixed so dev-server log
    /// collectors can ingest the line without a custom parser.
    ///
    /// @spec `.aw/tech-design/projects/jet/logic/bundler-incremental-rebuild.md`
    ///   §"Dev-server metric surface".
    /// @issue #1250 Slice 4 — R5.
    pub fn log_line(&self, path: &Path, wall_ms: u64) -> String {
        format!(
            r#"{{"event":"hmr_rebuild","path":{path},"hits":{hits},"misses":{misses},"bytes_reused":{bytes_reused},"wall_ms":{wall_ms}}}"#,
            path = json_string(&path.display().to_string()),
            hits = self.hits,
            misses = self.misses,
            bytes_reused = self.bytes_reused,
            wall_ms = wall_ms,
        )
    }
}

/// Minimal JSON string escaper for the metric log line. Escapes the
/// six characters JSON requires (`\`, `"`, control chars) and wraps
/// in double quotes. We do not call into `serde_json` for this one
/// field because it's the only string in the line.
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// File extensions the incremental transformer dispatches on.
/// Closed enum: a new language is a typed change, not a string-match
/// drift that silently routes to a passthrough default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ext {
    Tsx,
    Ts,
    Jsx,
    Js,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
impl Ext {
    fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|e| e.to_str())? {
            "tsx" => Some(Ext::Tsx),
            "ts" => Some(Ext::Ts),
            "jsx" => Some(Ext::Jsx),
            "js" | "mjs" | "cjs" => Some(Ext::Js),
            _ => None,
        }
    }
}

/// Incremental transformer that reuses previous parse trees.
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub struct IncrementalTransformer {
    parser: Parser,
    entries: HashMap<CacheKey, CachedEntry>,
    metrics: RebuildMetrics,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
impl IncrementalTransformer {
    pub fn new(language: tree_sitter::Language) -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&language)?;
        Ok(Self {
            parser,
            entries: HashMap::new(),
            metrics: RebuildMetrics::default(),
        })
    }

    /// Transform with incremental parsing.
    ///
    /// Cache hit: same path + same `sha256(new_source)` → returns
    /// the cached `transformed` text and bumps `metrics.hits`.
    ///
    /// Cache miss: parse (incrementally if `edit` is provided and a
    /// prior tree for `file_path` exists), run `transform_tree`,
    /// store the resulting `CachedEntry`, bump `metrics.misses`.
    pub fn transform_incremental(
        &mut self,
        file_path: &str,
        new_source: &str,
        edit: Option<InputEdit>,
    ) -> Result<String> {
        self.transform_for_path(Path::new(file_path), new_source, edit)
    }

    /// Transform keyed off the file path's extension. Public Slice-3
    /// surface; the extension drives dispatch into the existing
    /// `transform::{jsx,transform_tsx,typescript}` entry points.
    pub fn transform_for_path(
        &mut self,
        path: &Path,
        new_source: &str,
        edit: Option<InputEdit>,
    ) -> Result<String> {
        let ext = Ext::from_path(path)
            .ok_or_else(|| anyhow::anyhow!("unsupported extension: {}", path.display()))?;
        let key = CacheKey::new(path.to_path_buf(), new_source);
        if let Some(entry) = self.entries.get(&key) {
            self.metrics.hits += 1;
            self.metrics.bytes_reused += entry.transformed.len() as u64;
            return Ok(entry.transformed.clone());
        }

        let old_tree = self.find_prior_tree_for(path);
        let tree = if let (Some(old_tree), Some(edit)) = (old_tree, edit) {
            let mut updated_tree = old_tree.clone();
            updated_tree.edit(&edit);
            self.parser
                .parse(new_source, Some(&updated_tree))
                .ok_or_else(|| anyhow::anyhow!("Parse failed"))?
        } else {
            self.parser
                .parse(new_source, None)
                .ok_or_else(|| anyhow::anyhow!("Parse failed"))?
        };

        let started = std::time::Instant::now();
        let transformed = transform_tree(new_source, &tree, ext)?;
        let last_transform_us = started.elapsed().as_micros() as u64;

        self.entries.insert(
            key,
            CachedEntry {
                tree,
                transformed: transformed.clone(),
                last_transform_us,
            },
        );
        self.metrics.misses += 1;
        Ok(transformed)
    }

    /// Drop every cached entry whose `key.path` matches the given
    /// file path, regardless of content hash. Dev-server callers
    /// invoke this on file deletion / rename. No-op if the path is
    /// absent.
    pub fn invalidate(&mut self, file_path: &str) {
        let target: &Path = Path::new(file_path);
        self.entries.retain(|key, _| key.path != target);
    }

    /// Snapshot of the running metrics. Slice 4 reads this from the
    /// dev-server log emit path; Slice 2 callers use it for testing.
    pub fn metrics_snapshot(&self) -> RebuildMetrics {
        self.metrics
    }

    /// Pick any cached tree for the given path. The hash component
    /// of the key intentionally varies, so for incremental parsing
    /// any prior tree on the same path is a valid base — the parser
    /// applies the `InputEdit` then reconciles against `new_source`.
    fn find_prior_tree_for(&self, path: &Path) -> Option<&Tree> {
        self.entries
            .iter()
            .find(|(key, _)| key.path == path)
            .map(|(_, entry)| &entry.tree)
    }
}

/// Dispatch into the existing transform entry points by extension.
/// The cache layer above short-circuits same-source hits, so the
/// inner re-parse these transformers do today is only paid on a
/// miss — and is the same cost the bundler would pay anyway.
fn transform_tree(source: &str, _tree: &Tree, ext: Ext) -> Result<String> {
    let options = TransformOptions::default();
    let result = match ext {
        Ext::Tsx => transform_tsx::transform_tsx(source, &options)?,
        Ext::Ts => typescript::transform_typescript(source, &options)?,
        Ext::Jsx => jsx::transform_jsx(source, &options)?,
        Ext::Js => return Ok(source.to_string()),
    };
    Ok(result.code)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn js_lang() -> tree_sitter::Language {
        tree_sitter_javascript::LANGUAGE.into()
    }

    #[test]
    fn cache_key_same_input_is_equal() {
        let a = CacheKey::new("a.js", "const x = 1;");
        let b = CacheKey::new("a.js", "const x = 1;");
        assert_eq!(a, b);
    }

    #[test]
    fn cache_key_different_content_diverges() {
        let a = CacheKey::new("a.js", "const x = 1;");
        let b = CacheKey::new("a.js", "const x = 2;");
        assert_ne!(a, b);
    }

    #[test]
    fn cache_key_different_path_diverges() {
        let a = CacheKey::new("a.js", "const x = 1;");
        let b = CacheKey::new("b.js", "const x = 1;");
        assert_ne!(a, b);
    }

    #[test]
    fn same_path_same_source_is_hit() {
        let mut t = IncrementalTransformer::new(js_lang()).unwrap();
        t.transform_incremental("a.js", "const x = 1;", None)
            .unwrap();
        t.transform_incremental("a.js", "const x = 1;", None)
            .unwrap();
        let m = t.metrics_snapshot();
        assert_eq!(m.misses, 1);
        assert_eq!(m.hits, 1);
    }

    #[test]
    fn same_path_different_source_is_miss() {
        let mut t = IncrementalTransformer::new(js_lang()).unwrap();
        t.transform_incremental("a.js", "const x = 1;", None)
            .unwrap();
        t.transform_incremental("a.js", "const x = 2;", None)
            .unwrap();
        let m = t.metrics_snapshot();
        assert_eq!(m.misses, 2);
        assert_eq!(m.hits, 0);
    }

    #[test]
    fn different_path_same_source_is_miss() {
        let mut t = IncrementalTransformer::new(js_lang()).unwrap();
        t.transform_incremental("a.js", "const x = 1;", None)
            .unwrap();
        t.transform_incremental("b.js", "const x = 1;", None)
            .unwrap();
        let m = t.metrics_snapshot();
        assert_eq!(m.misses, 2);
        assert_eq!(m.hits, 0);
    }

    #[test]
    fn invalidate_drops_every_hash_for_path() {
        let mut t = IncrementalTransformer::new(js_lang()).unwrap();
        t.transform_incremental("a.js", "const x = 1;", None)
            .unwrap();
        t.transform_incremental("a.js", "const x = 2;", None)
            .unwrap();
        t.transform_incremental("b.js", "const x = 1;", None)
            .unwrap();
        assert_eq!(t.entries.len(), 3);

        t.invalidate("a.js");
        assert_eq!(t.entries.len(), 1);
        assert!(t.entries.keys().all(|k| k.path != Path::new("a.js")));
    }

    #[test]
    fn invalidate_missing_path_is_noop() {
        let mut t = IncrementalTransformer::new(js_lang()).unwrap();
        t.transform_incremental("a.js", "const x = 1;", None)
            .unwrap();
        t.invalidate("nonexistent.js");
        assert_eq!(t.entries.len(), 1);
    }

    #[test]
    fn metrics_default_is_zero() {
        let t = IncrementalTransformer::new(js_lang()).unwrap();
        assert_eq!(t.metrics_snapshot(), RebuildMetrics::default());
    }

    #[test]
    fn ext_from_path_is_closed() {
        assert_eq!(Ext::from_path(Path::new("a.tsx")), Some(Ext::Tsx));
        assert_eq!(Ext::from_path(Path::new("a.ts")), Some(Ext::Ts));
        assert_eq!(Ext::from_path(Path::new("a.jsx")), Some(Ext::Jsx));
        assert_eq!(Ext::from_path(Path::new("a.js")), Some(Ext::Js));
        assert_eq!(Ext::from_path(Path::new("a.mjs")), Some(Ext::Js));
        assert_eq!(Ext::from_path(Path::new("a.cjs")), Some(Ext::Js));
        assert_eq!(Ext::from_path(Path::new("a.css")), None);
        assert_eq!(Ext::from_path(Path::new("README")), None);
    }

    #[test]
    fn unsupported_extension_errors() {
        let mut t = IncrementalTransformer::new(js_lang()).unwrap();
        let err = t
            .transform_for_path(Path::new("a.css"), "body{}", None)
            .unwrap_err();
        assert!(err.to_string().contains("unsupported extension"));
    }

    #[test]
    fn js_passthrough_is_identity() {
        let mut t = IncrementalTransformer::new(js_lang()).unwrap();
        let src = "const x = 1;\nexport default x;\n";
        let out = t.transform_for_path(Path::new("a.js"), src, None).unwrap();
        assert_eq!(out, src);
    }

    #[test]
    fn ts_strips_type_annotations_on_miss_and_caches_on_hit() {
        let mut t = IncrementalTransformer::new(js_lang()).unwrap();
        let src = "const x: number = 1;\nexport default x;\n";
        let first = t.transform_for_path(Path::new("a.ts"), src, None).unwrap();
        // The TS transform must remove the `: number` annotation.
        assert!(
            !first.contains(": number"),
            "ts transform should strip type annotation, got: {first:?}"
        );
        let second = t.transform_for_path(Path::new("a.ts"), src, None).unwrap();
        assert_eq!(first, second, "cache hit must be byte-identical");
        let m = t.metrics_snapshot();
        assert_eq!(m.misses, 1);
        assert_eq!(m.hits, 1);
        assert_eq!(m.bytes_reused, first.len() as u64);
    }

    #[test]
    fn tsx_dispatches_through_jsx_transform() {
        let mut t = IncrementalTransformer::new(js_lang()).unwrap();
        let src = "const App = (): JSX.Element => <div>hi</div>;\n";
        let out = t
            .transform_for_path(Path::new("App.tsx"), src, None)
            .unwrap();
        // The combined tsx pipeline must rewrite `<div>` into a
        // function call (jsx automatic runtime by default) and
        // erase the `: JSX.Element` annotation.
        assert!(
            !out.contains("<div>"),
            "tsx transform should rewrite jsx, got: {out:?}"
        );
        assert!(
            !out.contains(": JSX.Element"),
            "tsx transform should strip return type, got: {out:?}"
        );
    }

    #[test]
    fn metrics_log_line_matches_spec_shape() {
        let m = RebuildMetrics {
            hits: 482,
            misses: 18,
            bytes_reused: 1_532_418,
        };
        let line = m.log_line(Path::new("src/components/Header.tsx"), 47);
        assert_eq!(
            line,
            r#"{"event":"hmr_rebuild","path":"src/components/Header.tsx","hits":482,"misses":18,"bytes_reused":1532418,"wall_ms":47}"#
        );
    }

    #[test]
    fn metrics_log_line_escapes_path() {
        // A path with a backslash + quote must be escaped so the line
        // remains valid JSON. (Windows paths or weird filenames.)
        let m = RebuildMetrics::default();
        let line = m.log_line(Path::new("a\\b\"c.tsx"), 0);
        assert!(
            line.contains(r#""path":"a\\b\"c.tsx""#),
            "path field must escape both backslash and quote: {line:?}"
        );
        // The JSON must round-trip.
        let parsed: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(
            parsed["path"],
            serde_json::Value::String("a\\b\"c.tsx".to_string())
        );
    }

    #[test]
    fn jsx_dispatches_through_jsx_transform() {
        let mut t = IncrementalTransformer::new(js_lang()).unwrap();
        let src = "const App = () => <div>hi</div>;\n";
        let out = t
            .transform_for_path(Path::new("App.jsx"), src, None)
            .unwrap();
        assert!(
            !out.contains("<div>"),
            "jsx transform should rewrite jsx, got: {out:?}"
        );
    }
}
// CODEGEN-END

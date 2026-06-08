---
id: projects-jet-logic-bundler-incremental-rebuild-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Bundler Incremental Rebuild for HMR Invalidation

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/bundler-incremental-rebuild.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# Bundler Incremental Rebuild for HMR Invalidation

### Overview

Spec for the incremental-rebuild path that turns jet's existing
HMR detection (closed `enhancement-jet-dev-javascript-module-hmr-hot-module-replaceme`)
into a sub-100 ms targeted rebuild. Today's dev-mode flow detects
the change and then re-parses the **full** dependency graph;
`crates/jet/src/transform/incremental.rs` already exists as a
skeleton (parser cache + a `TODO: integrate with jsx/typescript
transform logic`), but the wiring through the bundler graph and
the dev server is not in place. This spec pins the architecture
that lets a single-file change in a 500-module graph rebuild in
under 100 ms (R6).

@spec #1250 — `jet: bundler — incremental rebuild for HMR invalidation`.

### Slice plan

Five slices. Each one ships a mergeable artifact and unblocks the
next; the suite is intentionally cache-first (Slice 2) before
graph-first (Slice 3) so the cheapest cache wins land before any
graph-traversal work.

- **Slice 1 (this doc) — architecture spec.** Pins cache key
  format, invalidation graph contract, jsx/ts integration
  surface, dev-server metric shape, and benchmark harness
  layout. No production code.
- **Slice 2 (shipped) — content-hashed parse-tree cache (R1, R3).**
  Replaces the current `HashMap<String, Tree>` cache in
  `transform/incremental.rs` with `entries: HashMap<CacheKey,
  CachedEntry>` keyed on `(path, sha256(source_bytes))`.
  Same-hash hits skip the parser entirely; different-hash misses
  re-parse and overwrite the entry. No bundler-graph changes
  yet — the cache is local to the transformer. Adds the
  `RebuildMetrics { hits, misses, bytes_reused }` struct that
  Slice 4 surfaces; counters increment in this slice but stay
  private until then via `metrics_snapshot()` returning a
  `Copy` snapshot. `invalidate(path)` retains its public
  signature but now drops every entry whose `key.path == path`
  regardless of hash. Eight new unit tests pin the four
  hit/miss polarities (same-path-same-content = hit,
  same-path-different-content = miss, different-path-same-content
  = miss, different-path-different-content = miss) plus
  invalidate semantics + the metrics counter increments.
- **Slice 3 (shipped) — `transform_tree` jsx/ts wiring (R4).**
  Replaced the empty-string stub in `transform_tree` with a
  call into `transform::transform_tsx::transform_tsx` (when the
  file is `.tsx`), `transform::jsx::transform_jsx` (when
  `.jsx`), and `transform::typescript::transform_typescript`
  (when `.ts`); plain `.js` / `.mjs` / `.cjs` pass through
  unchanged. The public entry point is
  `IncrementalTransformer::transform_for_path(path,
  new_source, edit) -> Result<String>` — `path`'s extension
  drives the dispatch via a closed enum (`Ext::{Tsx, Ts, Jsx,
  Js}`) so adding a new language is a typed change, not a
  string-match drift. `transform_incremental(file_path: &str,
  …)` retained as a thin `&str → &Path` adapter so the existing
  Slice-2 tests keep their call shape. Six new unit tests pin
  the dispatch matrix: `Ext::from_path` closure, unsupported
  extension errors, `.js` identity passthrough, `.ts` type
  stripping with cache-hit byte-identity + metric increments,
  and `.tsx` / `.jsx` jsx-rewrite dispatch.

  Deviation from the original draft: the spec named
  `transform::jsx::transform` / `transform::typescript::strip_types`,
  but those are not the actual public entry points in this
  workspace — the real surface is `transform_jsx` /
  `transform_tsx` / `transform_typescript` (each returning
  `TransformResult { code, source_map }`). Slice 3 calls those
  and unwraps `.code`. The internal re-parse those functions
  do is not yet hoisted onto the cached `tree_sitter::Tree` —
  that's a follow-up optimisation, not a Slice 3 deliverable.
- **Slice 4a (shipped) — invalidation-graph + metric-line
  primitives (R2 + R5 foundation).** Adds
  `ModuleGraph::dependents_of(&Path) -> Vec<PathBuf>` — a
  BFS walk over reverse `Import` / `CssImport` /
  `WasmImport` edges that **stops at `DynamicImport`
  boundaries** (every `import()` is already an HMR root in
  the runtime client). Returns dependents in BFS order,
  excluding the input path; an unknown path yields an empty
  vec. Adds `RebuildMetrics::log_line(&Path, wall_ms) ->
  String` rendering the spec's `hmr_rebuild` JSON shape with
  hand-rolled string escaping (no `serde_json` on the hot
  path; round-trip-tested). Seven new graph tests pin the
  reverse-walker contract: simple chain, self-exclusion,
  unknown path, barrel re-export cascade, dynamic-import
  boundary stop, diamond dedup, CSS / WASM static edges.
  Two new metrics tests pin the line shape (matches the
  spec example byte-for-byte) and path escaping
  (`json_string` survives a `\\\"` filename).
- **Slice 4b (shipped) — dev-server reverse-dep walker.**
  Adds `dev_server::module_graph::ModuleGraph::dependents_of(url)
  -> Vec<String>` — BFS over reverse `importers` edges, full
  transitive set, excludes the input. Distinct from the existing
  `find_hmr_boundary` (which stops at the first accepting
  boundary): this returns the FULL invalidation set the
  bundler-side cache layer must drop. Five new tests pin
  unknown-url empty, self-exclusion, transitive chain,
  diamond dedup, and the barrel-cascade case. The dev-server
  graph carries no edge kinds (all imports are static), so the
  dynamic-import stop rule lives only on the bundler-side
  `bundler::graph::ModuleGraph::dependents_of` (Slice 4a).
- **Slice 4c (shipped) — `IncrementalRebuilder` glue + log
  emit.** New `dev_server::incremental_rebuilder` module with
  `IncrementalRebuilder { transformer: IncrementalTransformer }`
  + `RebuildOutcome { invalidated: Vec<String>, log_line:
  String }`. `rebuild(changed_url, changed_file, new_source,
  &graph)`:
  1. Calls `graph.dependents_of(changed_url)` → transitive
     importer set.
  2. Drops the cache entry for `changed_file` AND each
     dependent's filesystem path (looked up via
     `graph.get(dep_url).file`).
  3. Re-runs `transform_for_path` on the changed file (skipped
     for unsupported extensions like `.css` so the rebuilder is
     watcher-grade — non-script file events still increment the
     log without erroring).
  4. Emits one `tracing::info!(target: "jet::hmr", "{json}")`
     line, returns the same JSON in `RebuildOutcome.log_line`
     for ergonomic test assertions (no `tracing-test` dep).

  Six new tests pin: invalidated set = self + transitive
  importers; log line round-trips as JSON with the spec's
  fields; cache is dropped for the changed file (subsequent
  call on the same source is a hit — proves reseating); cache
  is dropped for every dependent (subsequent call on a
  primed dependent is a miss — proves the fan-out
  invalidation); unsupported `.css` extension skips the
  transform but still emits the line; unknown URL invalidates
  only itself (no graph entry → no dependents).

  Watcher hookup (`watcher.rs` → `IncrementalRebuilder`) is
  intentionally NOT in this slice — the rebuilder is a
  drop-in for a future watcher-event handler. Plumbing it
  through the existing `dev_server::watcher` channel touches
  more surface than this tick budgets.
- **Slice 5 (shipped) — R6 wall-clock gate.** Lives at
  `crates/jet/tests/incremental_rebuild_bench.rs` as a regular
  integration test (not under `benches/`) so the gate fires on
  every `cargo test` run without a separate `cargo bench`
  invocation. Criterion is intentionally NOT pulled in: the
  gate is a hard ceiling assertion, and adding a heavy
  regression-tracking dep just to print prettier numbers is
  not worth the build cost on a critical-path crate.

  Three test cases:
    1. `cold_full_graph_baseline` (informational only) —
       transforms all 500 modules cold, records per-module
       wall-clock as the upper bound the warm path beats.
       Measured: ~84 ms / 500 modules ≈ 0.2 ms/module.
    2. `single_leaf_change_warm_cache_under_100ms` (**R6 gate**)
       — pre-primes the cache, changes one leaf, asserts
       wall-clock < 100 ms AND the invalidated set includes
       both the leaf's barrel and the entry. Measured: 0 ms
       (sub-millisecond — well under the ceiling).
    3. `barrel_cascade_under_2x_leaf_baseline` (relative gate)
       — measures barrel-cascade rebuild against the leaf
       baseline measured in the same run; asserts within 2×
       (with a 4 ms floor so sub-ms timing isn't flaky).

  The 500-module graph is built deterministically by
  `build_500_module_graph()`: 1 entry → 25 barrels × 19 leaves
  + 1 dropped leaf to land at exactly 500. No fixture files
  on disk; all sources are synthesised in-process.

Slice 5 closes R6's acceptance criterion.

### Cache key format

```rust
// crates/jet/src/transform/incremental.rs (Slice 2)

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Absolute path. Module identity at the dev-server layer.
    /// Two modules with the same content but different paths are
    /// distinct entries — we never want a cross-path content
    /// collision to silently reuse a transform.
    pub path: PathBuf,
    /// blake3 of the raw source bytes. Catches any byte-level
    /// change including whitespace, encoding bumps, or BOM
    /// edits the parser would otherwise pretend are no-ops.
    pub content_hash: [u8; 32],
}
```

`sha2::Sha256` is the workspace-pinned hasher (already used by
`crates/jet/src/wasm_build/manifest.rs`'s `jet_config_hash`,
which formats as `sha256:<hex>`). Re-using the same primitive
avoids dragging a second hash crate into the workspace; raw
`[u8; 32]` digest bytes are kept on the cache key for `Hash + Eq`
without the hex stringification cost on every lookup.

The cache is `HashMap<CacheKey, CachedEntry>`:

```rust
pub struct CachedEntry {
    pub tree: tree_sitter::Tree,
    pub transformed: String,
    /// Wall-clock the cached transform took to produce. The
    /// metrics surface uses this to report `bytes_reused`
    /// without re-running the transform.
    pub last_transform_us: u64,
}
```

Hit / miss logic:

1. Compute `key = CacheKey { path, content_hash }`.
2. If `cache.get(&key)` is `Some(entry)`, return
   `entry.transformed.clone()` and bump `metrics.hits`.
3. Else parse + transform, insert into cache, bump
   `metrics.misses` + `metrics.bytes_reused = 0`.

`invalidate(path)` retains the existing API but removes **every**
entry whose `key.path == path` regardless of hash. Callers from
the dev server use this on file deletion / rename.

### Invalidation graph contract

The dev server needs to know which modules to invalidate when
`./y.ts` changes. The contract is the existing module graph's
**reverse dependency** edges:

```rust
// crates/jet/src/bundler/graph.rs (Slice 4 — public surface)

impl ModuleGraph {
    /// Modules that import (directly or transitively) the
    /// given path. Walk is BFS over reverse Import / CssImport /
    /// WasmImport edges; DynamicImport edges stop the walk
    /// (dynamic boundaries are HMR boundaries — invalidating
    /// past them is what causes today's full-graph re-parse
    /// pathology).
    pub fn dependents_of(&self, path: &Path) -> Vec<PathBuf>;
}
```

DynamicImport edges are NOT walked because every `import()`
boundary is already an HMR root in the runtime client; pushing
invalidation past it just discards work the user's browser will
re-fetch lazily. This is the high-leverage rule that takes the
500-module-graph re-parse from "1.5 s" to "well under 100 ms"
for the typical leaf-module change.

### jsx / ts integration surface

`transform_tree` is replaced with a typed dispatcher:

```rust
// crates/jet/src/transform/incremental.rs (Slice 3)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ext { Tsx, Ts, Jsx, Js }

impl Ext {
    fn from_path(path: &Path) -> Option<Self> { /* match suffix */ }
}

impl IncrementalTransformer {
    pub fn transform_for_path(
        &mut self,
        path: &Path,
        new_source: &str,
        edit: Option<InputEdit>,
    ) -> Result<String> {
        let ext = Ext::from_path(path)
            .ok_or_else(|| anyhow!("unsupported extension: {}", path.display()))?;
        let tree = self.parse_with_cache(path, new_source, edit)?;
        match ext {
            Ext::Tsx | Ext::Jsx => crate::transform::jsx::transform(new_source, &tree),
            Ext::Ts => crate::transform::typescript::strip_types(new_source, &tree),
            Ext::Js => Ok(new_source.to_string()),
        }
    }
}
```

The `Ext` enum is closed (no `Other`) so a new file extension
fails compilation rather than silently routing to a default.
`transform::jsx::transform` and `transform::typescript::strip_types`
are the **existing** transform entry points — Slice 3 calls them
in the dependent direction; it does not duplicate or refactor
them.

### Dev-server metric surface

Per R5, every rebuild emits one log line on the dev server's
existing `tracing` channel. JSON shape (so dev-server log
collectors can ingest it without a custom parser):

```json
{
  "event": "hmr_rebuild",
  "path": "src/components/Header.tsx",
  "hits": 482,
  "misses": 18,
  "bytes_reused": 1532418,
  "wall_ms": 47
}
```

`hits` + `misses` are running counters since dev-server start;
`bytes_reused` is the total bytes of cached `transformed` output
delivered without re-running the transform; `wall_ms` is the
wall-clock the rebuild took. `wall_ms < 100` is the R6
acceptance criterion the benchmark harness gates against.

### Benchmark harness layout

```
crates/jet/benches/
  incremental_hmr.rs        # criterion bench; the only file in this slice
```

Three benchmark functions, each a `criterion_group!` member:

1. `cold_full_graph` — first build of a 500-module graph; no
   cache hits. Establishes the upper-bound that incremental
   must beat.
2. `single_leaf_change_warm_cache` — the headline case.
   Pre-builds the graph, then changes one leaf module's source
   and measures `transform_for_path` + the dev server's
   reverse-dep walk + cache invalidation pass. **R6 ceiling:
   100 ms.**
3. `barrel_cascade` — change a 3-deep barrel re-export's leaf;
   measures the cascade through `dependents_of`. Different
   shape from #2; the assertion is "barrel cascade is no more
   than 2× the leaf-change baseline" (not a hard ms ceiling —
   the cascade size is fixture-dependent).

The 500-module graph is built by a deterministic generator
(seed pinned in the bench source) so the wall-clock numbers are
comparable across machines that share the same CPU class. A
realistic Conductor-FE / mini-react fixture is **out of scope**
for this slice — the synthetic graph is the regression gate; the
realistic fixture lands as a follow-up enhancement once the
synthetic gate is green.

### Invariants

1. **Cache hit MUST be byte-identical to a fresh transform.** If
   the cached `transformed` differs from re-running the
   transform on the same source, the cache is broken and a hit
   is a correctness regression. Slice 2 ships a debug-build
   assertion (`#[cfg(debug_assertions)]`) that re-runs the
   transform on every hit and panics on mismatch. Production
   builds skip this; the assertion's purpose is to catch
   pipeline drift in CI.
2. **Cache key MUST be content + path, never path alone.** Two
   files with the same path across rebuilds with different
   content must miss; two distinct paths with the same content
   must not collide. Slice 2's tests pin both polarities.
3. **DynamicImport edges MUST NOT be walked by `dependents_of`.**
   Walking them re-introduces the 500-module re-parse
   pathology. Slice 4's tests assert a fixture with a
   DynamicImport edge stops the reverse walk at the boundary.

### Out of scope

- **Cross-session disk-persistent cache.** Issue body §"Out of
  Scope" calls this out; this spec respects that. A separate
  follow-up enhancement (similar to Vite's `depsOptimizer`)
  files the on-disk cache once the in-memory path is stable.
- **Incremental AOT / production builds.** Different invariant
  set (deterministic output, content-hashed asset names);
  separate spec.
- **`tsconfig` / `jet.config` change handling.** Both fall back
  to a full rebuild for now (issue body's explicit call-out).
  Future tickets file the targeted-invalidation paths.

### Cross-references

- `crates/jet/src/transform/incremental.rs` — module under
  rewrite. Today: 95 lines, parser-cache only, transform_tree
  is a TODO.
- `crates/jet/src/transform/jsx.rs`,
  `crates/jet/src/transform/typescript.rs` — transform entry
  points Slice 3 calls.
- `crates/jet/src/bundler/graph.rs` — module dependency graph.
  Slice 4 adds `dependents_of`; the existing
  `topological_sort` / `find_cycle_from` neighbours the API.
- `crates/jet/src/dev_server/hmr.rs` — HMR detection that
  triggers rebuild today; Slice 4 wires the new path through it.
- Closed precedent:
  `enhancement-jet-dev-javascript-module-hmr-hot-module-replaceme`.
- Feeds: `epic-module-federation-config-container-manifest-shared`
  (#1121) — MFE container manifest needs stable, targeted
  invalidation; this spec's `dependents_of` is the primitive
  the federation work calls.

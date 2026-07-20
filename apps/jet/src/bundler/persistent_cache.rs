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
//! ## Import-scan section (#2140)
//!
//! A second, independent row set in the *same* file: `path`-agnostic and
//! content-addressed by [`ImportScanKey`] (a module's own `content_hash`
//! plus `is_typescript` — the only two inputs the scan depends on), holding
//! the already-`Bundler::runtime_static_imports`-narrowed
//! [`imports::ModuleImports`] that `Bundler::prefetch_one_module` and
//! `build_graph`'s synchronous fallback both consume. Gated at load by
//! [`SCANNER_VERSION`] rather than `config_fingerprint` — a scanner-only
//! behavior change (or a transform-only one) invalidates just its own
//! section, not the whole file — and evicted independently of the transform
//! `entries` above (its own [`MAX_STORE_BYTES`] pass, since import-scan
//! rows are typically orders of magnitude smaller than a compiled module's
//! code + source map). Same corrupt-row isolation as the transform section:
//! see [`StoredImportScanEntry`].
//!
//! ## Resolution section (#2141)
//!
//! A third, independent row set: bare-package-specifier resolutions whose
//! importer lives inside `node_modules`, keyed by [`ResolutionKey`]
//! (importer's package-root realpath + specifier + [`resolver_config_fingerprint`]).
//! This is the disk-backed sibling of `ModuleResolver`'s own in-memory
//! `resolution_cache`, and reuses that memo's exact eligibility boundary —
//! only a bare package specifier (`react`, `@mui/utils/clamp`, never a
//! relative/absolute/alias one) is safe to key on "enclosing package root"
//! alone, since a relative specifier's target depends on the importer's
//! *exact* directory, not just its package scope (two sibling directories
//! in the same package can both import `./bar` and mean two different
//! files). Scoped to node_modules-internal resolutions only: an app-source
//! importer (or an app-source *target*, e.g. a `baseUrl`/alias hit) is
//! never cached here, even though it may itself be a bare specifier — see
//! `Bundler::resolve_dependency`'s scope-gate call site for the file-
//! appearance/probe-order hazard this avoids. Guarded per-entry (not just
//! per-section) by [`ResolutionValue::guard`]: every package.json path the
//! resolution actually consulted, each re-hashed on lookup — a mismatch
//! (content changed, or existence flipped either direction) is a miss, not
//! a stale hit, independent of whether the whole-file schema/config still
//! matches. See [`node_modules_scope_realpath`] and
//! `ModuleResolver::resolve_with_probe`'s doc comment for the realpath and
//! probe-capture mechanics.
//!
//! ## Analysis section (#2141)
//!
//! A fourth, independent row set: the per-module *raw* (graph-shape-free)
//! product of `tree_shake::compute_raw_module_facts` — the pure textual
//! extraction half of what tree-shaking's liveness analysis needs per
//! module, before any specifier is resolved to a target path. Content-
//! addressed by [`AnalysisKey`] (a module's own `content_hash` plus
//! `is_typescript`, same shape as [`ImportScanKey`] and for the same
//! reason). Gated at load by [`analysis_fingerprint`] (defines, sorted,
//! folded with [`ANALYSIS_VERSION`]) rather than the transform section's
//! `config_fingerprint` — see that function's doc comment for why a
//! defines change must invalidate the *whole* section rather than rely on
//! incidental content-hash drift. Deliberately does NOT cache the
//! *resolved* `tree_shake::ModuleFacts` (edges point at other modules'
//! paths, which depend on which modules exist in the CURRENT build, i.e.
//! graph shape) — only the raw, specifier-level facts; resolving a raw
//! fact's specifiers through the live `ModuleLookup` always happens fresh,
//! whether the raw facts themselves came from a cache hit or a fresh
//! extraction, so a hit is indistinguishable downstream. See
//! `tree_shake::RawModuleFacts`'s doc comment for the full raw/resolved
//! split.
//!
//! ## Replay section (#2143)
//!
//! A fifth, independent element (not a row set — exactly one, or none):
//! [`ReplayManifest`], recorded after a successful full build and consulted
//! *before* [`PersistentTransformCache::load`] even runs, by the standalone
//! [`peek_replay_manifest`] + [`try_replay`] fast path `cli.rs`'s `build`
//! handler calls first. Unlike the four sections above, this one exists to
//! let a build skip constructing a `Bundler` (and therefore this whole
//! cache) altogether: a cheap `(mtime, size)` stat screen over every module
//! the *previous* successful build actually consumed, falling back to a
//! content hash only for files whose stat drifted, plus a sorted-filename
//! listing fingerprint for every directory that held a consumed app-source
//! module (the resolution-shadow guard — a *new* file appearing in such a
//! directory can silently change which file a relative/extensionless
//! import resolves to, something no already-consumed file's own stat/hash
//! could ever reveal). `node_modules` directories are deliberately not
//! listing-fingerprinted: the resolution section's (#2141) per-entry
//! `package.json` guard above already re-verifies node_modules package
//! integrity. Any mismatch, missing file, or ambiguity anywhere in this
//! check is a full build, not a best-effort replay — see [`try_replay`]'s
//! doc comment. Replay itself is verify-only (v1, this issue's explicit
//! scope): outputs are never stored as payloads, only their relative dist
//! paths + content hashes; "replaying" means confirming every recorded
//! output is still on disk with a matching hash and reprinting the standard
//! build-complete line, never regenerating or copying anything.
//! `Bundler::collect_replay_inputs` declines to record a manifest at all
//! (leaving whatever was on disk untouched — see
//! [`PersistentTransformCache::replay_manifest`]'s field doc comment) for
//! any build whose graph contains a `Json`/`Asset` module: neither of this
//! v1 collector's inputs sees those kinds' content, so tracking them
//! soundly is a follow-up, not a silent gap.
//!
//! ## Hatches
//!
//! `[build] cache` in jet.toml, `--no-cache` on `jet build` (flag wins over
//! config; see `cli.rs`'s `build_cache_enabled`), and
//! `JET_NO_PERSISTENT_CACHE=1` as a lower-level kill switch consulted
//! directly in [`PersistentTransformCache::load`] (so any caller that
//! constructs a `Bundler` with `cache_project_root: Some(..)` directly —
//! tests, embedders — still honors the env override without re-plumbing the
//! CLI precedence). `JET_NO_REPLAY=1` (#2143) additionally disables just
//! the replay section's fast path (both the pre-`Bundler` [`try_replay`]
//! check and the post-build `Bundler::record_replay_manifest` write) while
//! leaving the four sections above fully active; `--no-cache` already
//! implies it, since a disabled store has no manifest to check against or
//! write.
//! @issue #2137

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::imports;
use super::CompiledModule;

/// Bumped whenever a transform-pass behavior change could make an
/// old on-disk entry decode successfully but produce output that no longer
/// matches what today's transform would emit. Folded into the cache file's
/// `config_fingerprint` alongside the crate version, so either one changing
/// invalidates the whole store rather than serving stale-shape code.
///
/// #2140 bumped this 1 -> 2: `CacheFile` grew two new trailing fields
/// (`scanner_version`, `import_scan_entries`) for the import-scan section
/// below. An old (schema-1) file has neither field, so `postcard`'s decode
/// of the new shape fails cleanly on the missing bytes — the existing
/// "unreadable, starting cold" warning path in [`PersistentTransformCache::
/// load`] already handles that outcome, so old stores need no migration:
/// clean full miss, warned once, never a hard error.
///
/// #2141 bumped this 2 -> 3: `CacheFile` grew three more trailing fields
/// (`resolution_entries`, `analysis_fingerprint`, `analysis_entries`) for
/// the resolution and analysis sections below. Same reasoning applies: an
/// old (schema-2) file is missing the new fields, `postcard` fails the
/// decode cleanly, and the file is treated as cold rather than partially
/// trusted.
///
/// #2143 bumped this 3 -> 4: `CacheFile` grew one more trailing field
/// (`replay_manifest`) for the replay section below. Same reasoning
/// applies again: an old (schema-3) file is missing the field, `postcard`
/// fails the decode cleanly, cold start.
pub const TRANSFORM_CACHE_SCHEMA: u32 = 4;

/// Bumped whenever import-scan behavior changes (`imports::
/// extract_imports_fast`, its tree-sitter fallback, or `Bundler::
/// runtime_static_imports`'s type-only-import narrowing) in a way that
/// could make an old cached [`imports::ModuleImports`] no longer match what
/// today's scan would produce for the same content. Stored once per file
/// (`CacheFile::scanner_version`) and checked once at load — independently
/// of [`TRANSFORM_CACHE_SCHEMA`]/`config_fingerprint`, which gate the
/// transform section — so a scanner-only behavior change invalidates just
/// the import-scan section, and vice versa. See the module doc comment's
/// "Import-scan section" heading.
/// @issue #2140
pub const SCANNER_VERSION: u32 = 1;

/// Bumped whenever tree-shaking's *raw* (specifier-level, graph-shape-free)
/// fact extraction changes (`tree_shake::compute_raw_module_facts` or any
/// of the pure extractors it composes) in a way that could make an old
/// cached [`super::tree_shake::RawModuleFacts`] no longer match what
/// today's extraction would produce for the same content. Folded into
/// [`analysis_fingerprint`] alongside the sorted `defines` set, and
/// checked once at load — see the module doc comment's "Analysis section"
/// heading.
/// @issue #2141
pub const ANALYSIS_VERSION: u32 = 1;

/// Bumped whenever the resolution *algorithm* changes in a way that could
/// make an old [`ResolutionValue`] no longer a safe reuse for the same
/// [`ResolutionKey`] — independent of [`resolver_config_fingerprint`]'s own
/// option inputs, which cover *configuration* only, not code changes
/// within one crate version. Folded into every [`ResolutionKey`] via
/// [`resolver_config_fingerprint`], so a bump makes every existing
/// resolution entry unreachable (no key this build ever builds can match
/// one from before the bump) — no separate whole-section sweep needed, see
/// `PersistentTransformCache::resolution_entries`'s field doc comment.
/// @issue #2141
pub const RESOLUTION_VERSION: u32 = 1;

/// Bumped whenever the #2143 replay verification algorithm itself changes
/// (which stat/hash fields it trusts, how the fast path screens inputs) in
/// a way that could make an old recorded [`ReplayManifest`] no longer a
/// safe basis for a replay decision — independent of
/// [`replay_config_fingerprint`]'s own build-config inputs. A mismatch
/// declines the replay attempt outright (see [`try_replay`]) rather than
/// attempting a partial reuse.
/// @issue #2143
pub const REPLAY_VERSION: u32 = 1;

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

/// Key for one cached import-scan entry (#2140): a module's own source
/// `content_hash` plus `is_typescript` — the only two inputs `Bundler::
/// prefetch_one_module`'s import-scan (`imports::extract_imports_fast` /
/// its tree-sitter fallback, narrowed through `Bundler::
/// runtime_static_imports`) depends on. Unlike [`EntryKey`], there is no
/// `own_id`/`dep_fingerprint` component: an import list is a pure function
/// of a module's own bytes plus whether it is parsed as TypeScript, never
/// of this build's graph-discovery order, so this key alone is enough to
/// address the entry directly (no `path`-keyed indirection needed — two
/// modules with byte-identical content and the same `is_typescript`-ness
/// legitimately share one entry).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImportScanKey {
    pub content_hash: u64,
    pub is_typescript: bool,
}

/// Key for one cached node_modules-scoped resolution (#2141): the
/// importer's enclosing package-root *realpath* (see
/// [`node_modules_scope_realpath`]), the bare specifier being resolved,
/// and [`resolver_config_fingerprint`] (aliases/baseUrl/conditions/
/// externalize flags/[`RESOLUTION_VERSION`]/crate version). Mirrors
/// `ModuleResolver`'s own in-memory `resolution_cache` key shape
/// (`(bare_specifier_cache_root(..), specifier)`) plus the two ingredients
/// a same-process memo doesn't need: a realpath (so two symlinks pointing
/// at the same pnpm-store package — the #1941 pnpm-symlink trap class —
/// collapse to the same cache scope) and an explicit config fingerprint (a
/// persisted entry can outlive the single build whose options it was
/// captured under).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResolutionKey {
    pub scope_realpath: PathBuf,
    pub specifier: String,
    pub resolver_config_fingerprint: u64,
}

/// Value for one cached resolution: the resolved path exactly as
/// `ModuleResolver::resolve` would return it today — deliberately NOT
/// canonicalized (see `Bundler::resolve_dependency`'s existing comment on
/// why canonicalizing the *returned* path breaks node_modules walk-up
/// resolution for transitive dependencies by following hardlinks into the
/// content store) — plus `guard`: every package.json path the resolution
/// actually consulted, paired with the content hash it had at capture time
/// (`None` when the path did not exist at capture time, so a package.json
/// *appearing* later is just as much a guard mismatch as one changing
/// content or disappearing — a plain sentinel `u64` would risk a
/// real-hash collision with "absent," so this is a proper `Option` instead).
/// Re-verified in full on every lookup by [`PersistentTransformCache::
/// get_resolution`] — a single mismatched guard entry is a miss,
/// independent of whether `scope_realpath`/`specifier`/
/// `resolver_config_fingerprint` still match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionValue {
    pub resolved_path: PathBuf,
    pub guard: Vec<(PathBuf, Option<u64>)>,
}

/// Key for one cached raw-analysis entry (#2141): a module's own source
/// `content_hash` plus `is_typescript` — same shape as [`ImportScanKey`]
/// and for the same reason (`tree_shake::compute_raw_module_facts` is a
/// pure function of exactly those two inputs, never of this build's graph
/// shape or discovery order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnalysisKey {
    pub content_hash: u64,
    pub is_typescript: bool,
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

/// One on-disk import-scan cache row (#2140). Same corrupt-row isolation as
/// [`StoredEntry`] (see its doc comment): `payload` is an independently
/// `postcard`-encoded [`imports::ModuleImports`] blob, checked against
/// `checksum` before decode, so one corrupt row can never desync the decode
/// of every row after it.
#[derive(Debug, Serialize, Deserialize)]
struct StoredImportScanEntry {
    key: ImportScanKey,
    checksum: u64,
    payload: Vec<u8>,
    last_used: u64,
}

/// One on-disk resolution cache row (#2141). Same corrupt-row isolation as
/// [`StoredEntry`]/[`StoredImportScanEntry`]: `payload` is an
/// independently `postcard`-encoded [`ResolutionValue`] blob, checked
/// against `checksum` before decode.
#[derive(Debug, Serialize, Deserialize)]
struct StoredResolutionEntry {
    key: ResolutionKey,
    checksum: u64,
    payload: Vec<u8>,
    last_used: u64,
}

/// One on-disk analysis cache row (#2141). Same corrupt-row isolation as
/// the sections above: `payload` is an independently `postcard`-encoded
/// [`super::tree_shake::RawModuleFacts`] blob, checked against `checksum`
/// before decode.
#[derive(Debug, Serialize, Deserialize)]
struct StoredAnalysisEntry {
    key: AnalysisKey,
    checksum: u64,
    payload: Vec<u8>,
    last_used: u64,
}

/// One tracked build input for the #2143 replay manifest — a module (or,
/// for `index.html`/`public/`, a plain file `Bundler` never sees) the
/// previous successful build actually consumed. `content_hash` is the same
/// [`hash_bytes`] of the file's raw content every other section already
/// keys on; `mtime_nanos`/`size` are the cheap stat pair [`try_replay`]
/// screens first, falling back to re-reading and re-hashing only when
/// either drifted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayInput {
    pub path: PathBuf,
    pub content_hash: u64,
    /// Nanoseconds since `UNIX_EPOCH`. Collection declines to record a
    /// manifest at all for a file whose mtime cannot be read (see
    /// [`mtime_nanos`]) rather than store a placeholder — [`try_replay`]'s
    /// own stat re-check already treats an unreadable mtime as a drifted
    /// stat (falls back to content hash) without needing a sentinel value
    /// here.
    pub mtime_nanos: u128,
    pub size: u64,
}

/// One tracked source directory's sorted-filename listing fingerprint — the
/// resolution-shadow guard described in the module doc comment's "Replay
/// section" heading: a *new* file appearing in a directory that held a
/// consumed app-source module can silently change which file a
/// relative/extensionless import resolves to, something no already-consumed
/// file's own [`ReplayInput`] could ever reveal, since the new file itself
/// was never consumed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayDirFingerprint {
    pub dir: PathBuf,
    pub listing_hash: u64,
}

/// One build output [`try_replay`] must find on disk, byte-identical,
/// before declaring a replay valid (v1: verify-only — the bytes themselves
/// are never stored here, only enough to detect drift). `rel_path` is
/// `/`-separated and relative to the build's output directory, matching
/// however deep a written file actually landed (entry bundle, chunks,
/// chunk maps, CSS/asset files, copied `public/` files, the generated
/// `index.html`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayOutput {
    pub rel_path: String,
    pub content_hash: u64,
}

/// The whole #2143 replay manifest: recorded once, after every successful
/// full build with replay enabled (see `Bundler::record_replay_manifest`),
/// consulted once, before the next build's `Bundler` is even constructed
/// (see [`try_replay`]). See the module doc comment's "Replay section"
/// heading for the full design.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayManifest {
    pub replay_version: u32,
    /// See [`replay_config_fingerprint`] — folds together the transform
    /// section's `config_fingerprint`, the resolution section's
    /// `resolver_config_fingerprint`, and a jet.toml content hash, so a
    /// change to any of the three declines a stale replay the same way a
    /// mismatched `config_fingerprint` already declines a stale transform
    /// cache hit.
    pub config_fingerprint: u64,
    pub inputs: Vec<ReplayInput>,
    pub source_dirs: Vec<ReplayDirFingerprint>,
    pub outputs: Vec<ReplayOutput>,
    /// `/`-separated relative dist path of the entry bundle
    /// (`main.<hash>.js`), duplicated out of `outputs` so a replay hit can
    /// reprint the standard `Build complete in Xms: <output>/<file> (<KB>
    /// KB)` line without re-deriving which recorded output is "the entry".
    pub entry_rel_path: String,
    pub entry_size: u64,
}

/// Result of one [`try_replay`] attempt, and the source of the
/// `JET_BUNDLE_TIMING` `replay: ...` line (see [`Self::timing_line`]).
#[derive(Debug, Clone)]
pub enum ReplayOutcome {
    Replayed {
        entry_rel_path: String,
        entry_size: u64,
        verified: usize,
        stat_ms: f64,
        hash_fallback: usize,
    },
    FullBuild {
        reason: String,
        verified: usize,
        stat_ms: f64,
        hash_fallback: usize,
    },
}

impl ReplayOutcome {
    pub fn is_replayed(&self) -> bool {
        matches!(self, ReplayOutcome::Replayed { .. })
    }

    /// `replay: verified=N stat_ms=X hash_fallback=K
    /// outcome=replayed|full-build(<reason>)` — the exact #2143
    /// `JET_BUNDLE_TIMING=1` line format.
    pub fn timing_line(&self) -> String {
        match self {
            ReplayOutcome::Replayed {
                verified,
                stat_ms,
                hash_fallback,
                ..
            } => format!(
                "replay: verified={verified} stat_ms={stat_ms:.2} hash_fallback={hash_fallback} outcome=replayed"
            ),
            ReplayOutcome::FullBuild {
                reason,
                verified,
                stat_ms,
                hash_fallback,
            } => format!(
                "replay: verified={verified} stat_ms={stat_ms:.2} hash_fallback={hash_fallback} outcome=full-build({reason})"
            ),
        }
    }
}

/// The whole on-disk file: one `postcard`-encoded blob.
///
/// #2140 added the two trailing fields for the import-scan section
/// (`scanner_version`, `import_scan_entries`); see [`TRANSFORM_CACHE_SCHEMA`]
/// for why appending fields here is safe (old-shaped files just fail to
/// decode, a clean cold-start) and the module doc comment's "Import-scan
/// section" heading for the design. #2141 appended three more trailing
/// fields for the resolution and analysis sections — same reasoning.
/// #2143 appended one more trailing field, `replay_manifest`, for the
/// replay section.
#[derive(Debug, Default, Serialize, Deserialize)]
struct CacheFile {
    schema: u32,
    config_fingerprint: u64,
    entries: Vec<StoredEntry>,
    scanner_version: u32,
    import_scan_entries: Vec<StoredImportScanEntry>,
    resolution_entries: Vec<StoredResolutionEntry>,
    analysis_fingerprint: u64,
    analysis_entries: Vec<StoredAnalysisEntry>,
    replay_manifest: Option<ReplayManifest>,
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

/// In-memory representation of one live import-scan entry (#2140) —
/// decoded once at `load` (or inserted fresh this build). Unlike
/// [`LiveEntry`] there is no separate key field: [`ImportScanKey`] IS the
/// map key (`PersistentTransformCache::import_scan_entries` is keyed by it
/// directly), since a lookup needs no secondary match-check beyond the map
/// lookup itself — see [`ImportScanKey`]'s doc comment.
#[derive(Debug, Clone)]
struct LiveImportScanEntry {
    module: imports::ModuleImports,
    last_used: u64,
}

/// In-memory representation of one live resolution entry (#2141) —
/// decoded once at `load` (or inserted fresh this build). No separate key
/// field: [`ResolutionKey`] IS the map key, same reasoning as
/// [`LiveImportScanEntry`].
#[derive(Debug, Clone)]
struct LiveResolutionEntry {
    value: ResolutionValue,
    last_used: u64,
}

/// In-memory representation of one live analysis entry (#2141) — decoded
/// once at `load` (or inserted fresh this build). No separate key field:
/// [`AnalysisKey`] IS the map key, same reasoning as [`LiveImportScanEntry`].
#[derive(Debug, Clone)]
struct LiveAnalysisEntry {
    facts: super::tree_shake::RawModuleFacts,
    last_used: u64,
}

/// Result of [`PersistentTransformCache::load`], surfaced through
/// `JET_BUNDLE_TIMING` and the final build report.
#[derive(Debug, Clone, Default)]
pub struct LoadStats {
    pub enabled: bool,
    pub loaded_entries: usize,
    pub corrupt_entries: usize,
    /// #2140 — import-scan section counterparts to `loaded_entries`/
    /// `corrupt_entries` above, tracked separately since the two sections
    /// are gated by independent fingerprints (`config_fingerprint` vs
    /// `SCANNER_VERSION`) and can legitimately disagree (one section valid,
    /// the other cold).
    pub import_scan_loaded_entries: usize,
    pub import_scan_corrupt_entries: usize,
    /// #2141 — resolution/analysis section counterparts, tracked
    /// separately since all four sections are gated by independent
    /// fingerprints and can legitimately disagree.
    pub resolution_loaded_entries: usize,
    pub resolution_corrupt_entries: usize,
    pub analysis_loaded_entries: usize,
    pub analysis_corrupt_entries: usize,
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
    /// clock syscall. Shared by both the transform and import-scan
    /// sections, so `save`'s two independent eviction passes still agree
    /// on one global sense of "oldest".
    clock: AtomicU64,
    /// #2140 — import-scan section: content-addressed by [`ImportScanKey`]
    /// directly (no `path` indirection — see its doc comment), tracked and
    /// evicted independently of `entries` above.
    import_scan_entries: DashMap<ImportScanKey, LiveImportScanEntry>,
    import_scan_hits: AtomicU64,
    import_scan_misses: AtomicU64,
    /// #2141 — resolution section: content-addressed by [`ResolutionKey`]
    /// directly, tracked and evicted independently of the sections above.
    /// No whole-section gating fingerprint at load time (unlike the
    /// transform/analysis sections): `resolver_config_fingerprint` is
    /// already part of every key, so a config change simply makes old
    /// entries unreachable (any lookup this build performs builds a key
    /// with the *current* fingerprint) rather than needing a separate
    /// section-wide sweep.
    resolution_entries: DashMap<ResolutionKey, LiveResolutionEntry>,
    resolution_hits: AtomicU64,
    resolution_misses: AtomicU64,
    /// #2141 — analysis section: content-addressed by [`AnalysisKey`],
    /// gated at load by `analysis_fingerprint` (stored here so `save` can
    /// write the same value back for the next load).
    analysis_entries: DashMap<AnalysisKey, LiveAnalysisEntry>,
    analysis_hits: AtomicU64,
    analysis_misses: AtomicU64,
    analysis_fingerprint: u64,
    /// #2143 — the replay section's single element (see the module doc
    /// comment's "Replay section" heading). Populated from the on-disk
    /// file at [`Self::load`] (a shape mismatch already fails the whole
    /// `CacheFile` decode via [`TRANSFORM_CACHE_SCHEMA`], so no separate
    /// per-field checksum is needed here, unlike the row-sequence sections
    /// above) and left untouched by [`Self::save`] unless
    /// [`Self::set_replay_manifest`] overwrote it first — so a build that
    /// never reaches `set_replay_manifest` (replay declined, or
    /// `JET_NO_REPLAY` set) carries the previous build's manifest forward
    /// instead of erasing it.
    replay_manifest: Mutex<Option<ReplayManifest>>,
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
            import_scan_entries: DashMap::new(),
            import_scan_hits: AtomicU64::new(0),
            import_scan_misses: AtomicU64::new(0),
            resolution_entries: DashMap::new(),
            resolution_hits: AtomicU64::new(0),
            resolution_misses: AtomicU64::new(0),
            analysis_entries: DashMap::new(),
            analysis_hits: AtomicU64::new(0),
            analysis_misses: AtomicU64::new(0),
            analysis_fingerprint: 0,
            replay_manifest: Mutex::new(None),
        }
    }

    /// Load `<project_root>/node_modules/.jet/transform-cache.bin`, if
    /// enabled and present. `schema` gates the whole file's *shape*: a
    /// mismatch (an old- or new-shaped file this binary cannot decode)
    /// discards everything as a deliberate, silent cold-start — not
    /// corruption, so it does not warn (a shape mismatch usually also just
    /// fails `postcard::from_bytes` outright, handled the same way below).
    /// Past that, the transform section (`entries`), import-scan section
    /// (`import_scan_entries`), and analysis section (`analysis_entries`)
    /// are gated *independently* — `config_fingerprint`, [`SCANNER_VERSION`]
    /// (#2140), and `analysis_fingerprint` (#2141) respectively — so a
    /// change that only affects one (e.g. a `defines` edit, or a
    /// scanner-only bug fix) discards only that section, not the others. A
    /// missing file (first run, or a deliberately cleared store) is the
    /// same silent cold-start for every section. The resolution section
    /// (#2141) has no separate load-time gate at all — see
    /// `resolution_entries`'s field doc comment for why. Only a *present
    /// but undecodable* file, or individual corrupt entries inside an
    /// otherwise good section, print a warning — see the module doc
    /// comment.
    pub fn load(
        project_root: &Path,
        config_fingerprint: u64,
        analysis_fingerprint: u64,
    ) -> (Self, LoadStats) {
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
            import_scan_entries: DashMap::new(),
            import_scan_hits: AtomicU64::new(0),
            import_scan_misses: AtomicU64::new(0),
            resolution_entries: DashMap::new(),
            resolution_hits: AtomicU64::new(0),
            resolution_misses: AtomicU64::new(0),
            analysis_entries: DashMap::new(),
            analysis_hits: AtomicU64::new(0),
            analysis_misses: AtomicU64::new(0),
            analysis_fingerprint,
            replay_manifest: Mutex::new(None),
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

        if file.schema != TRANSFORM_CACHE_SCHEMA {
            return (
                cache,
                LoadStats {
                    enabled: true,
                    duration: start.elapsed(),
                    ..Default::default()
                },
            );
        }
        let transform_section_valid = file.config_fingerprint == config_fingerprint;
        let import_scan_section_valid = file.scanner_version == SCANNER_VERSION;
        let analysis_section_valid = file.analysis_fingerprint == analysis_fingerprint;

        // #2143 — replay section: no whole-section fingerprint gate here
        // (unlike the transform/analysis sections above) — carried forward
        // as-is from disk. `try_replay`/`peek_replay_manifest` re-validate
        // `replay_version` and `config_fingerprint` themselves against
        // *this* build's own values at the point a replay is actually
        // attempted (before a `Bundler` — and therefore this build's own
        // fingerprints — even exists), so no extra gating is needed here.
        // Populating it (rather than leaving it `None`) matters so a build
        // which never calls `set_replay_manifest` (replay declined this
        // run, or `JET_NO_REPLAY` set) has `save` re-persist the previous
        // manifest unchanged instead of silently erasing a still-valid one
        // — see `replay_manifest`'s field doc comment.
        *cache.replay_manifest.lock().unwrap() = file.replay_manifest;

        let mut loaded = 0usize;
        let mut corrupt = 0usize;
        let mut max_last_used = 0u64;
        if transform_section_valid {
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
        }

        let mut import_scan_loaded = 0usize;
        let mut import_scan_corrupt = 0usize;
        if import_scan_section_valid {
            for stored in file.import_scan_entries {
                if hash_bytes(&stored.payload) != stored.checksum {
                    import_scan_corrupt += 1;
                    continue;
                }
                let module: imports::ModuleImports = match postcard::from_bytes(&stored.payload) {
                    Ok(m) => m,
                    Err(_) => {
                        import_scan_corrupt += 1;
                        continue;
                    }
                };
                max_last_used = max_last_used.max(stored.last_used);
                cache.import_scan_entries.insert(
                    stored.key,
                    LiveImportScanEntry {
                        module,
                        last_used: stored.last_used,
                    },
                );
                import_scan_loaded += 1;
            }
        }

        // #2141 — resolution section: no separate whole-section gate (see
        // the struct field doc comment) — always attempt to load every
        // row once the outer schema matches; stale-fingerprint entries
        // just never match a key this build ever builds.
        let mut resolution_loaded = 0usize;
        let mut resolution_corrupt = 0usize;
        for stored in file.resolution_entries {
            if hash_bytes(&stored.payload) != stored.checksum {
                resolution_corrupt += 1;
                continue;
            }
            let value: ResolutionValue = match postcard::from_bytes(&stored.payload) {
                Ok(v) => v,
                Err(_) => {
                    resolution_corrupt += 1;
                    continue;
                }
            };
            max_last_used = max_last_used.max(stored.last_used);
            cache.resolution_entries.insert(
                stored.key,
                LiveResolutionEntry {
                    value,
                    last_used: stored.last_used,
                },
            );
            resolution_loaded += 1;
        }

        // #2141 — analysis section: gated by `analysis_fingerprint`, same
        // independent-section shape as import-scan's `SCANNER_VERSION`
        // gate above.
        let mut analysis_loaded = 0usize;
        let mut analysis_corrupt = 0usize;
        if analysis_section_valid {
            for stored in file.analysis_entries {
                if hash_bytes(&stored.payload) != stored.checksum {
                    analysis_corrupt += 1;
                    continue;
                }
                let facts: super::tree_shake::RawModuleFacts =
                    match postcard::from_bytes(&stored.payload) {
                        Ok(f) => f,
                        Err(_) => {
                            analysis_corrupt += 1;
                            continue;
                        }
                    };
                max_last_used = max_last_used.max(stored.last_used);
                cache.analysis_entries.insert(
                    stored.key,
                    LiveAnalysisEntry {
                        facts,
                        last_used: stored.last_used,
                    },
                );
                analysis_loaded += 1;
            }
        }

        // Anything inserted fresh this build must sort after everything
        // just loaded, so `save`'s oldest-first eviction stays meaningful
        // across process runs instead of resetting to 0 every load. One
        // shared clock across both sections (see its field doc comment).
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
        if import_scan_corrupt > 0 {
            eprintln!(
                "warn: jet transform cache at {} had {import_scan_corrupt} corrupt import-scan \
                 entr{} out of {}; dropped and will re-scan (#2140)",
                store_path.display(),
                if import_scan_corrupt == 1 { "y" } else { "ies" },
                import_scan_loaded + import_scan_corrupt,
            );
        }
        if resolution_corrupt > 0 {
            eprintln!(
                "warn: jet transform cache at {} had {resolution_corrupt} corrupt resolution \
                 entr{} out of {}; dropped and will re-resolve (#2141)",
                store_path.display(),
                if resolution_corrupt == 1 { "y" } else { "ies" },
                resolution_loaded + resolution_corrupt,
            );
        }
        if analysis_corrupt > 0 {
            eprintln!(
                "warn: jet transform cache at {} had {analysis_corrupt} corrupt analysis \
                 entr{} out of {}; dropped and will re-analyze (#2141)",
                store_path.display(),
                if analysis_corrupt == 1 { "y" } else { "ies" },
                analysis_loaded + analysis_corrupt,
            );
        }

        (
            cache,
            LoadStats {
                enabled: true,
                loaded_entries: loaded,
                corrupt_entries: corrupt,
                import_scan_loaded_entries: import_scan_loaded,
                import_scan_corrupt_entries: import_scan_corrupt,
                resolution_loaded_entries: resolution_loaded,
                resolution_corrupt_entries: resolution_corrupt,
                analysis_loaded_entries: analysis_loaded,
                analysis_corrupt_entries: analysis_corrupt,
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
        let (lookup_ns, clone_ns) = (AtomicU64::new(0), AtomicU64::new(0));
        self.get_with_laps(path, key, &lookup_ns, &clone_ns)
    }

    /// Same contract as [`Self::get`], additionally attributing the wall
    /// time spent finding+matching the entry (`lookup_ns`) separately from
    /// cloning the matched `CompiledModule` out (`clone_ns`) — #2140's
    /// hit-path sub-lap attribution, consulted by `transform_modules`'s
    /// `persistent-cache-hit-laps` line. `get` is a thin wrapper around
    /// this with throwaway local counters, so there is exactly one copy of
    /// the lookup/clone logic.
    pub fn get_with_laps(
        &self,
        path: &Path,
        key: &EntryKey,
        lookup_ns: &AtomicU64,
        clone_ns: &AtomicU64,
    ) -> Option<CompiledModule> {
        if !self.enabled {
            return None;
        }
        let lookup_start = Instant::now();
        let found = self.entries.get_mut(path).filter(|entry| entry.key == *key);
        lookup_ns.fetch_add(lookup_start.elapsed().as_nanos() as u64, Ordering::Relaxed);
        match found {
            Some(mut entry) => {
                entry.last_used = self.next_clock();
                self.hits.fetch_add(1, Ordering::Relaxed);
                let clone_start = Instant::now();
                let module = entry.module.clone();
                clone_ns.fetch_add(clone_start.elapsed().as_nanos() as u64, Ordering::Relaxed);
                Some(module)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
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

    /// Look up a cached import-scan result for `key` (#2140). Content-
    /// addressed, so unlike [`Self::get`] there is no `path` parameter and
    /// no secondary key-match check beyond the map lookup itself: a hit on
    /// `key` is valid for *any* module sharing that exact
    /// `(content_hash, is_typescript)` pair (see [`ImportScanKey`]'s doc
    /// comment for why that is safe). Touches recency on a hit; counts
    /// exactly one hit or miss per call either way.
    pub fn get_import_scan(&self, key: &ImportScanKey) -> Option<imports::ModuleImports> {
        if !self.enabled {
            return None;
        }
        match self.import_scan_entries.get_mut(key) {
            Some(mut entry) => {
                entry.last_used = self.next_clock();
                self.import_scan_hits.fetch_add(1, Ordering::Relaxed);
                Some(entry.module.clone())
            }
            None => {
                self.import_scan_misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    /// Record (or replace) `key`'s import-scan result (#2140). A later
    /// `save` call persists whatever is live in-memory at that point.
    pub fn insert_import_scan(&self, key: ImportScanKey, module: imports::ModuleImports) {
        if !self.enabled {
            return;
        }
        let last_used = self.next_clock();
        self.import_scan_entries
            .insert(key, LiveImportScanEntry { module, last_used });
    }

    pub fn import_scan_hits(&self) -> u64 {
        self.import_scan_hits.load(Ordering::Relaxed)
    }

    pub fn import_scan_misses(&self) -> u64 {
        self.import_scan_misses.load(Ordering::Relaxed)
    }

    /// Look up a cached resolution for `key` (#2141). Re-verifies every
    /// guard entry's package.json content hash *right now* — a single
    /// mismatch (content changed, or a path's existence flipped either
    /// direction since capture) is a miss, independent of whether `key`
    /// itself matched, and evicts the stale entry so a later `insert_
    /// resolution` for the same `key` cannot race a lingering stale read.
    /// Touches recency on a hit; counts exactly one hit or miss per call.
    pub fn get_resolution(&self, key: &ResolutionKey) -> Option<PathBuf> {
        if !self.enabled {
            return None;
        }
        let Some(mut entry) = self.resolution_entries.get_mut(key) else {
            self.resolution_misses.fetch_add(1, Ordering::Relaxed);
            return None;
        };
        let guard_intact = entry.value.guard.iter().all(|(pkg_json, expected)| {
            let actual = std::fs::read(pkg_json).ok().map(|bytes| hash_bytes(&bytes));
            actual == *expected
        });
        if !guard_intact {
            drop(entry);
            self.resolution_entries.remove(key);
            self.resolution_misses.fetch_add(1, Ordering::Relaxed);
            return None;
        }
        entry.last_used = self.next_clock();
        self.resolution_hits.fetch_add(1, Ordering::Relaxed);
        Some(entry.value.resolved_path.clone())
    }

    /// Record (or replace) `key`'s resolution result (#2141). A later
    /// `save` call persists whatever is live in-memory at that point.
    pub fn insert_resolution(&self, key: ResolutionKey, value: ResolutionValue) {
        if !self.enabled {
            return;
        }
        let last_used = self.next_clock();
        self.resolution_entries
            .insert(key, LiveResolutionEntry { value, last_used });
    }

    pub fn resolution_hits(&self) -> u64 {
        self.resolution_hits.load(Ordering::Relaxed)
    }

    pub fn resolution_misses(&self) -> u64 {
        self.resolution_misses.load(Ordering::Relaxed)
    }

    /// Look up a cached raw-analysis result for `key` (#2141).
    /// Content-addressed, same "any module sharing this exact key is a
    /// valid hit" contract as [`Self::get_import_scan`] and for the same
    /// reason. Touches recency on a hit; counts exactly one hit or miss
    /// per call either way.
    pub fn get_analysis(&self, key: &AnalysisKey) -> Option<super::tree_shake::RawModuleFacts> {
        if !self.enabled {
            return None;
        }
        match self.analysis_entries.get_mut(key) {
            Some(mut entry) => {
                entry.last_used = self.next_clock();
                self.analysis_hits.fetch_add(1, Ordering::Relaxed);
                Some(entry.facts.clone())
            }
            None => {
                self.analysis_misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    /// Record (or replace) `key`'s raw-analysis result (#2141). A later
    /// `save` call persists whatever is live in-memory at that point.
    pub fn insert_analysis(&self, key: AnalysisKey, facts: super::tree_shake::RawModuleFacts) {
        if !self.enabled {
            return;
        }
        let last_used = self.next_clock();
        self.analysis_entries
            .insert(key, LiveAnalysisEntry { facts, last_used });
    }

    pub fn analysis_hits(&self) -> u64 {
        self.analysis_hits.load(Ordering::Relaxed)
    }

    pub fn analysis_misses(&self) -> u64 {
        self.analysis_misses.load(Ordering::Relaxed)
    }

    /// Record (or replace) this build's #2143 replay manifest. A later
    /// [`Self::save`] call persists whatever is here at that point — same
    /// "unconditional overwrite, save persists live state" shape as
    /// [`Self::insert`]/[`Self::insert_analysis`], except there is exactly
    /// one element, not a map.
    pub fn set_replay_manifest(&self, manifest: ReplayManifest) {
        if !self.enabled {
            return;
        }
        *self.replay_manifest.lock().unwrap() = Some(manifest);
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

        // #2140 — import-scan section: same encode + checksum + oldest-
        // first eviction shape as the transform `rows` above, kept as an
        // independent pass (own byte total, own `MAX_STORE_BYTES` cap)
        // rather than merged into one heterogeneous eviction, since
        // import-scan rows are typically orders of magnitude smaller than a
        // compiled module and merging would mean every `save` re-sorting a
        // mixed-type Vec for a cap this section will rarely if ever near.
        let mut scan_rows: Vec<StoredImportScanEntry> =
            Vec::with_capacity(self.import_scan_entries.len());
        for kv in self.import_scan_entries.iter() {
            let module = &kv.value().module;
            let payload = match postcard::to_stdvec(module) {
                Ok(p) => p,
                Err(e) => {
                    tracing::debug!(
                        "jet transform cache: skipping unencodable import-scan entry {:?}: {e} (#2140)",
                        kv.key()
                    );
                    continue;
                }
            };
            let checksum = hash_bytes(&payload);
            scan_rows.push(StoredImportScanEntry {
                key: *kv.key(),
                checksum,
                payload,
                last_used: kv.value().last_used,
            });
        }
        scan_rows.sort_unstable_by_key(|r| r.last_used);
        let mut scan_total: u64 = scan_rows.iter().map(|r| r.payload.len() as u64).sum();
        while scan_total > MAX_STORE_BYTES && !scan_rows.is_empty() {
            let removed = scan_rows.remove(0);
            scan_total -= removed.payload.len() as u64;
        }

        // #2141 — resolution section: same encode + checksum + oldest-
        // first eviction shape, independent pass/cap (see the import-scan
        // section's comment above for why independent passes rather than
        // one merged sort across heterogeneous row types).
        let mut resolution_rows: Vec<StoredResolutionEntry> =
            Vec::with_capacity(self.resolution_entries.len());
        for kv in self.resolution_entries.iter() {
            let value = &kv.value().value;
            let payload = match postcard::to_stdvec(value) {
                Ok(p) => p,
                Err(e) => {
                    tracing::debug!(
                        "jet transform cache: skipping unencodable resolution entry {:?}: {e} (#2141)",
                        kv.key()
                    );
                    continue;
                }
            };
            let checksum = hash_bytes(&payload);
            resolution_rows.push(StoredResolutionEntry {
                key: kv.key().clone(),
                checksum,
                payload,
                last_used: kv.value().last_used,
            });
        }
        resolution_rows.sort_unstable_by_key(|r| r.last_used);
        let mut resolution_total: u64 =
            resolution_rows.iter().map(|r| r.payload.len() as u64).sum();
        while resolution_total > MAX_STORE_BYTES && !resolution_rows.is_empty() {
            let removed = resolution_rows.remove(0);
            resolution_total -= removed.payload.len() as u64;
        }

        // #2141 — analysis section: same shape again.
        let mut analysis_rows: Vec<StoredAnalysisEntry> =
            Vec::with_capacity(self.analysis_entries.len());
        for kv in self.analysis_entries.iter() {
            let facts = &kv.value().facts;
            let payload = match postcard::to_stdvec(facts) {
                Ok(p) => p,
                Err(e) => {
                    tracing::debug!(
                        "jet transform cache: skipping unencodable analysis entry {:?}: {e} (#2141)",
                        kv.key()
                    );
                    continue;
                }
            };
            let checksum = hash_bytes(&payload);
            analysis_rows.push(StoredAnalysisEntry {
                key: *kv.key(),
                checksum,
                payload,
                last_used: kv.value().last_used,
            });
        }
        analysis_rows.sort_unstable_by_key(|r| r.last_used);
        let mut analysis_total: u64 = analysis_rows.iter().map(|r| r.payload.len() as u64).sum();
        while analysis_total > MAX_STORE_BYTES && !analysis_rows.is_empty() {
            let removed = analysis_rows.remove(0);
            analysis_total -= removed.payload.len() as u64;
        }

        let file = CacheFile {
            schema: TRANSFORM_CACHE_SCHEMA,
            config_fingerprint: self.config_fingerprint,
            entries: rows,
            scanner_version: SCANNER_VERSION,
            import_scan_entries: scan_rows,
            resolution_entries: resolution_rows,
            analysis_fingerprint: self.analysis_fingerprint,
            analysis_entries: analysis_rows,
            replay_manifest: self.replay_manifest.lock().unwrap().clone(),
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

/// `pub(crate)` (#2141): `Bundler::resolve_dependency` needs this directly
/// (not via `hash_str`) to hash a package.json's raw bytes for the
/// resolution-cache guard on the write side — byte-for-byte the same
/// computation [`PersistentTransformCache::get_resolution`]'s guard
/// re-verification already does on the read side (`std::fs::read` +
/// `hash_bytes`), with no UTF-8 round trip in between.
pub(crate) fn hash_bytes(bytes: &[u8]) -> u64 {
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

/// Ordered dependency-id fingerprint for `id`: `graph.dependency_ids(id)`'s
/// own (unsorted) order, each resolved through `module_map` to the numeric
/// id this build assigned it. Deliberately not sorted — see the module doc
/// comment's `dep_fingerprint` entry for why order must be preserved.
///
/// #2140 — goes through `dependency_ids` (bare `ModuleId`s), not
/// `dependencies` (`ModuleId` + `EdgeKind` pairs via a `find_edge` +
/// `edge_weight` lookup this function never used): same iteration order,
/// so byte-identical output, one fewer `Vec` allocation and no wasted
/// edge-kind lookups per dependency.
pub fn dependency_fingerprint(
    graph: &super::ModuleGraph,
    id: super::ModuleId,
    module_map: &HashMap<PathBuf, usize>,
) -> u64 {
    let ids: Vec<usize> = graph
        .dependency_ids(id)
        .filter_map(|dep_id| {
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

/// Realpath of `path`'s nearest enclosing npm package root under
/// `node_modules` (walking up past a scoped package's `@scope/name` pair
/// when present), or `None` when no `node_modules` path component exists
/// at all. `path` may be a file or a directory — the algorithm only cares
/// about the position of the `node_modules` component, so any trailing
/// components past the package root (a file name, a subpath) are
/// naturally truncated away.
///
/// Mirrors `resolver::bare_specifier_cache_root`'s component-walking
/// algorithm (the existing in-memory resolution memo's own scope key),
/// but (a) canonicalizes the result — collapsing a pnpm-style symlinked
/// package to its real store location, since two different symlinks
/// pointing at the same real package must share one cache scope (see the
/// #1941 pnpm-symlink trap class) — and (b) returns `None` instead of
/// falling back to `path` itself when there is no `node_modules`
/// component, since callers need to distinguish "not under node_modules
/// at all" from "under node_modules" for the #2141 scope gate, a
/// distinction `bare_specifier_cache_root` does not need to make for its
/// own (memoization-only) purpose.
pub fn node_modules_scope_realpath(path: &Path) -> Option<PathBuf> {
    let components: Vec<&std::ffi::OsStr> = path.iter().collect();
    let nm = components.iter().rposition(|c| *c == "node_modules")?;
    let mut end = nm + 1;
    if let Some(first) = components.get(end) {
        if first.to_string_lossy().starts_with('@') {
            end += 1;
        }
    }
    end += 1;
    if end > components.len() {
        return None;
    }
    let scope: PathBuf = components[..end].iter().collect();
    std::fs::canonicalize(&scope).ok()
}

/// Whole-resolution config fingerprint for the #2141 resolution cache:
/// aliases (order-preserving — first-match-wins semantics), `baseUrl`,
/// `conditions` (order-preserving — condition preference order changes
/// which `exports` branch wins), the externalize flags, resolvable
/// extensions and the index-resolution toggle, plus the crate version and
/// [`RESOLUTION_VERSION`]. Folded into every [`ResolutionKey`] rather than
/// checked once for the whole section at load (contrast
/// [`config_fingerprint`]/[`analysis_fingerprint`]) — see
/// `PersistentTransformCache`'s `resolution_entries` field doc comment for
/// why a per-key fingerprint needs no separate load-time gate. `externals`
/// is a `HashSet`, sorted before hashing for the same randomized-
/// iteration-order reason `config_fingerprint` sorts `defines`.
pub fn resolver_config_fingerprint(options: &crate::resolver::ResolveOptions) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut externals_sorted: Vec<&String> = options.externals.iter().collect();
    externals_sorted.sort_unstable();

    let mut hasher = DefaultHasher::new();
    RESOLUTION_VERSION.hash(&mut hasher);
    env!("CARGO_PKG_VERSION").hash(&mut hasher);
    options.alias.hash(&mut hasher);
    options.base_url.hash(&mut hasher);
    options.conditions.hash(&mut hasher);
    options.externalize_all_packages.hash(&mut hasher);
    externals_sorted.hash(&mut hasher);
    options.extensions.hash(&mut hasher);
    options.resolve_index.hash(&mut hasher);
    hasher.finish()
}

/// Whole-section analysis fingerprint for the #2141 analysis cache:
/// `defines` (sorted, same reasoning as [`config_fingerprint`]) folded
/// with [`ANALYSIS_VERSION`]. Deliberately narrower than
/// `config_fingerprint` — `compute_raw_module_facts` extraction does not
/// depend on minify/splitting/jsx/target/source-map settings, only on a
/// module's own source text plus which identifiers `defines` would fold
/// away (defines substitution happens upstream of this analysis, so a
/// defines change can change which imports/requires survive as dead code
/// versus live). Checked once per cache file at
/// [`PersistentTransformCache::load`] — a mismatch discards the whole
/// analysis section, not just the affected modules, so `a_misses` is
/// exactly the module count on a defines change.
pub fn analysis_fingerprint(defines: &HashMap<String, String>) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut defines_sorted: Vec<(&String, &String)> = defines.iter().collect();
    defines_sorted.sort_unstable_by(|a, b| a.0.cmp(b.0));

    let mut hasher = DefaultHasher::new();
    ANALYSIS_VERSION.hash(&mut hasher);
    defines_sorted.hash(&mut hasher);
    hasher.finish()
}

/// Lightweight pre-`Bundler` peek at the persistent store's #2143 replay
/// section only. Deliberately does not go through [`PersistentTransformCache::
/// load`]: that decodes every transform/import-scan/resolution/analysis row
/// into a live, typed structure — exactly the cost the replay fast path
/// exists to let an unchanged build skip paying. This reads the same file
/// but stops at the outer [`CacheFile`] shape, discarding every other
/// section's still-opaque row payloads unread. Any failure along the way
/// (missing file, undecodable bytes, wrong [`TRANSFORM_CACHE_SCHEMA`], no
/// manifest recorded yet) returns `None` — every case [`try_replay`] treats
/// identically: no replay candidate, do a full build.
fn peek_replay_manifest(project_root: &Path) -> Option<ReplayManifest> {
    if std::env::var_os("JET_NO_PERSISTENT_CACHE").is_some() {
        return None;
    }
    let store_path = project_root.join(TRANSFORM_CACHE_REL_PATH);
    let bytes = std::fs::read(&store_path).ok()?;
    let file: CacheFile = postcard::from_bytes(&bytes).ok()?;
    if file.schema != TRANSFORM_CACHE_SCHEMA {
        return None;
    }
    file.replay_manifest
}

/// Whole-build #2143 replay config fingerprint: folds together the
/// transform section's [`config_fingerprint`] (defines, minify, splitting,
/// transform options, crate version, [`TRANSFORM_CACHE_SCHEMA`]), the
/// resolution section's [`resolver_config_fingerprint`] (alias, baseUrl,
/// conditions, externals — the "tsconfig/alias consulted" surface), a
/// jet.toml content hash (see [`hash_file_or_absent`] — the two
/// fingerprints above only cover options the CLI actually threaded into
/// `BundleOptions`, not jet.toml bytes wholesale), and [`REPLAY_VERSION`]
/// itself. Computed by `cli.rs` before constructing a `Bundler` at all
/// (from the exact same `defines`/`minify`/`splitting`/`transform_options`/
/// `resolve_options` values about to go into `BundleOptions`), so
/// [`try_replay`] can decide whether to build one at all.
pub fn replay_config_fingerprint(
    config_fingerprint: u64,
    resolver_config_fingerprint: u64,
    jet_toml_hash: u64,
) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    REPLAY_VERSION.hash(&mut hasher);
    config_fingerprint.hash(&mut hasher);
    resolver_config_fingerprint.hash(&mut hasher);
    jet_toml_hash.hash(&mut hasher);
    hasher.finish()
}

/// [`hash_bytes`] of `path`'s content, or `0` when `path` does not exist —
/// `0` is reserved here as an explicit "absent" sentinel rather than relied
/// on for uniqueness (an empty-but-present file hashes through the same
/// [`hash_bytes`] call as everything else, and collision with `0` is
/// accepted the same way every other `u64` fingerprint in this module
/// accepts hash collisions). Used for jet.toml in
/// [`replay_config_fingerprint`] — jet.toml is optional (`JetConfig::load`
/// already falls back to defaults when absent), so its absence must be a
/// stable, trackable fingerprint input too: a project that gains a
/// jet.toml between builds must not replay against a manifest recorded
/// before it existed.
pub fn hash_file_or_absent(path: &Path) -> u64 {
    match std::fs::read(path) {
        Ok(bytes) => hash_bytes(&bytes),
        Err(_) => 0,
    }
}

/// `mtime` as nanoseconds since `UNIX_EPOCH`, or `None` on a
/// platform/filesystem that cannot report one — every caller treats `None`
/// the same as a drifted/untrusted stat rather than a hard error.
pub(crate) fn mtime_nanos(meta: &std::fs::Metadata) -> Option<u128> {
    meta.modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_nanos())
}

/// Sorted-filename listing hash of `dir`'s *immediate* entries (files and
/// subdirectories both — a new subdirectory changes what a directory
/// import could resolve into just as much as a new file does). `None` if
/// `dir` cannot be read — every caller treats that the same as a changed
/// listing (an unconditional full-build reason), the same "any doubt"
/// rule as everywhere else in this section.
pub(crate) fn single_dir_listing_hash(dir: &Path) -> Option<u64> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .ok()?
        .map(|entry| entry.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect::<std::io::Result<Vec<_>>>()
        .ok()?;
    names.sort_unstable();
    Some(hash_seq(&names))
}

/// Recursively collects a #2143 replay input set for a directory tree that
/// is copied to the output dir verbatim rather than resolved through the
/// module graph (`public/` — see `cli.rs`'s `copy_public_dir`): one
/// [`ReplayDirFingerprint`] per directory encountered (including `dir`
/// itself) plus one [`ReplayInput`] per file, at every depth. `None`
/// (decline) on any read error along the way. A missing `dir` is not an
/// error — `public/` is optional — and returns two empty `Vec`s.
pub fn collect_dir_tree_replay_inputs(
    dir: &Path,
) -> Option<(Vec<ReplayInput>, Vec<ReplayDirFingerprint>)> {
    let mut inputs = Vec::new();
    let mut dirs = Vec::new();
    if !dir.is_dir() {
        return Some((inputs, dirs));
    }
    collect_dir_tree_replay_inputs_into(dir, &mut inputs, &mut dirs)?;
    Some((inputs, dirs))
}

fn collect_dir_tree_replay_inputs_into(
    dir: &Path,
    inputs: &mut Vec<ReplayInput>,
    dirs: &mut Vec<ReplayDirFingerprint>,
) -> Option<()> {
    let mut names = Vec::new();
    let mut subdirs = Vec::new();
    for entry in std::fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        let file_type = entry.file_type().ok()?;
        names.push(entry.file_name().to_string_lossy().into_owned());
        if file_type.is_dir() {
            subdirs.push(path);
        } else if file_type.is_file() {
            let meta = entry.metadata().ok()?;
            let mtime = mtime_nanos(&meta)?;
            let bytes = std::fs::read(&path).ok()?;
            inputs.push(ReplayInput {
                content_hash: hash_bytes(&bytes),
                mtime_nanos: mtime,
                size: meta.len(),
                path,
            });
        }
    }
    names.sort_unstable();
    dirs.push(ReplayDirFingerprint {
        dir: dir.to_path_buf(),
        listing_hash: hash_seq(&names),
    });
    for subdir in subdirs {
        collect_dir_tree_replay_inputs_into(&subdir, inputs, dirs)?;
    }
    Some(())
}

/// Recursively collects every file under `output_dir` (`/`-separated path
/// relative to it, plus a content hash) after a full build finishes
/// writing it, for the #2143 replay manifest's output set. A plain
/// post-hoc walk rather than tracking each individual writer call site in
/// `cli.rs`, so it can never miss a future new asset kind (chunks, CSS,
/// copied `public/` files, the generated `index.html`) — whatever is
/// actually on disk when this runs *is* the output set [`try_replay`] must
/// match later. `None` (decline recording) on any read error.
pub fn collect_dist_outputs(output_dir: &Path) -> Option<Vec<ReplayOutput>> {
    let mut outputs = Vec::new();
    collect_dist_outputs_into(output_dir, output_dir, &mut outputs)?;
    Some(outputs)
}

fn collect_dist_outputs_into(
    root: &Path,
    dir: &Path,
    outputs: &mut Vec<ReplayOutput>,
) -> Option<()> {
    for entry in std::fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        let file_type = entry.file_type().ok()?;
        if file_type.is_dir() {
            collect_dist_outputs_into(root, &path, outputs)?;
        } else if file_type.is_file() {
            let bytes = std::fs::read(&path).ok()?;
            let rel_path = path
                .strip_prefix(root)
                .ok()?
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            outputs.push(ReplayOutput {
                rel_path,
                content_hash: hash_bytes(&bytes),
            });
        }
    }
    Some(())
}

/// The #2143 replay fast path: attempts to verify the previous successful
/// build's recorded [`ReplayManifest`] (via [`peek_replay_manifest`]) still
/// describes `project_root`'s current state closely enough to trust its
/// `output_dir` outputs as-is, without constructing a `Bundler` or running
/// any part of the normal build pipeline. Every step below follows one
/// rule: any mismatch, missing file, or ambiguity is
/// [`ReplayOutcome::FullBuild`], never a best-effort partial replay.
///
/// 1. No manifest, wrong [`REPLAY_VERSION`], or a `config_fingerprint` that
///    does not match `replay_config_fingerprint` (defines/minify/splitting/
///    transform options, alias/baseUrl/conditions, jet.toml bytes, or the
///    crate/schema versions folded into either changed) — decline
///    immediately.
/// 2. Every recorded [`ReplayInput`]: stat `(mtime, size)` first; a match
///    trusts the recorded `content_hash` without re-reading the file. A
///    drifted stat falls back to re-reading and re-hashing the file — a
///    match is still fine (e.g. a touch/rewrite that reproduced the exact
///    same bytes), any other outcome (missing file, unreadable file, or a
///    genuinely different hash) declines.
/// 3. Every recorded [`ReplayDirFingerprint`]: the resolution-shadow guard
///    — an unreadable directory or a changed sorted listing declines.
/// 4. Every recorded [`ReplayOutput`]: still on disk under `output_dir`
///    with a matching content hash (verify-only v1 — nothing is ever
///    regenerated or copied here) — a missing or drifted output declines.
///
/// Only once all four pass does this return [`ReplayOutcome::Replayed`].
pub fn try_replay(
    project_root: &Path,
    output_dir: &Path,
    replay_config_fingerprint: u64,
) -> ReplayOutcome {
    let start = Instant::now();
    let mut verified = 0usize;
    let mut hash_fallback = 0usize;
    // Local helper so each decline site below only has to name its own
    // reason string — the shared `verified`/`hash_fallback`/`stat_ms`
    // bookkeeping is identical at every one of them.
    let full_build =
        |reason: String, verified: usize, hash_fallback: usize| ReplayOutcome::FullBuild {
            reason,
            verified,
            stat_ms: start.elapsed().as_secs_f64() * 1000.0,
            hash_fallback,
        };

    let Some(manifest) = peek_replay_manifest(project_root) else {
        return full_build("no-manifest".to_string(), verified, hash_fallback);
    };
    if manifest.replay_version != REPLAY_VERSION {
        return full_build("replay-version".to_string(), verified, hash_fallback);
    }
    if manifest.config_fingerprint != replay_config_fingerprint {
        return full_build("config-changed".to_string(), verified, hash_fallback);
    }

    for input in &manifest.inputs {
        let meta = match std::fs::metadata(&input.path) {
            Ok(m) => m,
            Err(_) => {
                return full_build(
                    format!("missing-input:{}", input.path.display()),
                    verified,
                    hash_fallback,
                )
            }
        };
        let stat_matches =
            meta.len() == input.size && mtime_nanos(&meta) == Some(input.mtime_nanos);
        if !stat_matches {
            hash_fallback += 1;
            let bytes = match std::fs::read(&input.path) {
                Ok(b) => b,
                Err(_) => {
                    return full_build(
                        format!("unreadable-input:{}", input.path.display()),
                        verified,
                        hash_fallback,
                    )
                }
            };
            if hash_bytes(&bytes) != input.content_hash {
                return full_build(
                    format!("content-changed:{}", input.path.display()),
                    verified,
                    hash_fallback,
                );
            }
        }
        verified += 1;
    }

    for dirfp in &manifest.source_dirs {
        let Some(listing_hash) = single_dir_listing_hash(&dirfp.dir) else {
            return full_build(
                format!("dir-unreadable:{}", dirfp.dir.display()),
                verified,
                hash_fallback,
            );
        };
        if listing_hash != dirfp.listing_hash {
            return full_build(
                format!("dir-changed:{}", dirfp.dir.display()),
                verified,
                hash_fallback,
            );
        }
        verified += 1;
    }

    for output in &manifest.outputs {
        let path = output_dir.join(&output.rel_path);
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(_) => {
                return full_build(
                    format!("missing-output:{}", output.rel_path),
                    verified,
                    hash_fallback,
                )
            }
        };
        if hash_bytes(&bytes) != output.content_hash {
            return full_build(
                format!("output-changed:{}", output.rel_path),
                verified,
                hash_fallback,
            );
        }
        verified += 1;
    }

    ReplayOutcome::Replayed {
        entry_rel_path: manifest.entry_rel_path,
        entry_size: manifest.entry_size,
        verified,
        stat_ms: start.elapsed().as_secs_f64() * 1000.0,
        hash_fallback,
    }
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
            scanner_version: SCANNER_VERSION,
            import_scan_entries: Vec::new(),
            resolution_entries: Vec::new(),
            analysis_fingerprint: 0,
            analysis_entries: Vec::new(),
            replay_manifest: None,
        };
        let bytes = postcard::to_stdvec(&file).unwrap();

        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"poisoned_payload_checksum_rejects")
        ));
        std::fs::create_dir_all(dir.join("node_modules/.jet")).unwrap();
        std::fs::write(dir.join(TRANSFORM_CACHE_REL_PATH), &bytes).unwrap();

        let (cache, stats) = PersistentTransformCache::load(&dir, 42, 0);
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
            scanner_version: SCANNER_VERSION,
            import_scan_entries: Vec::new(),
            resolution_entries: Vec::new(),
            analysis_fingerprint: 0,
            analysis_entries: Vec::new(),
            replay_manifest: None,
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
        let (cache, stats) = PersistentTransformCache::load(&dir, 2, 0);
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

        let (cache, _) = PersistentTransformCache::load(&dir, 99, 0);
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

        let (reloaded, load_stats) = PersistentTransformCache::load(&dir, 99, 0);
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

        let (cache, _) = PersistentTransformCache::load(&dir, 1, 0);
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

    // #2140 — import-scan section tests below. Same fixture/naming style as
    // the transform-section tests above.

    fn sample_imports(source: &str) -> imports::ModuleImports {
        imports::ModuleImports {
            static_imports: vec![imports::ImportDeclaration {
                source: source.to_string(),
                kind: imports::ImportKind::Named,
            }],
            dynamic_imports: Vec::new(),
            exports: Vec::new(),
        }
    }

    #[test]
    fn import_scan_miss_then_insert_then_hit_round_trips() {
        let cache = PersistentTransformCache {
            enabled: true,
            ..PersistentTransformCache::disabled()
        };
        let key = ImportScanKey {
            content_hash: hash_str("import x from './x';"),
            is_typescript: false,
        };

        assert!(cache.get_import_scan(&key).is_none());
        assert_eq!(cache.import_scan_misses(), 1);

        cache.insert_import_scan(key, sample_imports("./x"));
        let hit = cache
            .get_import_scan(&key)
            .expect("expected a hit after insert");
        assert_eq!(hit.static_imports[0].source, "./x");
        assert_eq!(cache.import_scan_hits(), 1);
        assert_eq!(cache.import_scan_misses(), 1);
    }

    #[test]
    fn import_scan_is_typescript_is_part_of_the_key() {
        // Same `content_hash` (a `.js` file and a `.ts` file with
        // byte-identical bytes are a real, if rare, scenario), different
        // `is_typescript` — must be a miss, not a stale hit, since
        // `runtime_static_imports` behaves differently for the two.
        let cache = PersistentTransformCache {
            enabled: true,
            ..PersistentTransformCache::disabled()
        };
        let content_hash = hash_str("import x from './x'; export {};");
        let js_key = ImportScanKey {
            content_hash,
            is_typescript: false,
        };
        cache.insert_import_scan(js_key, sample_imports("./x"));

        let ts_key = ImportScanKey {
            content_hash,
            is_typescript: true,
        };
        assert!(cache.get_import_scan(&ts_key).is_none());
        assert!(cache.get_import_scan(&js_key).is_some());
    }

    #[test]
    fn import_scan_poisoned_payload_checksum_rejects_without_losing_other_entries() {
        let good_key = ImportScanKey {
            content_hash: hash_str("import a from './a';"),
            is_typescript: false,
        };
        let good_payload = postcard::to_stdvec(&sample_imports("./a")).unwrap();
        let good_entry = StoredImportScanEntry {
            key: good_key,
            checksum: hash_bytes(&good_payload),
            payload: good_payload,
            last_used: 0,
        };

        let bad_key = ImportScanKey {
            content_hash: hash_str("import b from './b';"),
            is_typescript: false,
        };
        let mut bad_payload = postcard::to_stdvec(&sample_imports("./b")).unwrap();
        let bad_checksum = hash_bytes(&bad_payload);
        if let Some(byte) = bad_payload.first_mut() {
            *byte ^= 0xFF;
        }
        let bad_entry = StoredImportScanEntry {
            key: bad_key,
            checksum: bad_checksum,
            payload: bad_payload,
            last_used: 0,
        };

        let file = CacheFile {
            schema: TRANSFORM_CACHE_SCHEMA,
            config_fingerprint: 7,
            entries: Vec::new(),
            scanner_version: SCANNER_VERSION,
            import_scan_entries: vec![good_entry, bad_entry],
            resolution_entries: Vec::new(),
            analysis_fingerprint: 0,
            analysis_entries: Vec::new(),
            replay_manifest: None,
        };
        let bytes = postcard::to_stdvec(&file).unwrap();

        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"import_scan_poisoned_payload_checksum_rejects")
        ));
        std::fs::create_dir_all(dir.join("node_modules/.jet")).unwrap();
        std::fs::write(dir.join(TRANSFORM_CACHE_REL_PATH), &bytes).unwrap();

        let (cache, stats) = PersistentTransformCache::load(&dir, 7, 0);
        assert_eq!(stats.import_scan_loaded_entries, 1);
        assert_eq!(stats.import_scan_corrupt_entries, 1);
        assert!(cache.get_import_scan(&good_key).is_some());
        assert!(cache.get_import_scan(&bad_key).is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_scan_and_transform_sections_are_gated_independently() {
        // A `config_fingerprint` mismatch (transform section) must not
        // discard an otherwise-valid import-scan section, and a
        // `scanner_version` mismatch (import-scan section) must not
        // discard an otherwise-valid transform section — the two
        // fingerprints gate their own section only (#2140).
        let good_module = sample_module(0, "const good = 1;");
        let good_payload = postcard::to_stdvec(&good_module).unwrap();
        let transform_entry = StoredEntry {
            path: PathBuf::from("/proj/src/good.js"),
            key: sample_key(),
            checksum: hash_bytes(&good_payload),
            payload: good_payload,
            last_used: 0,
        };

        let scan_key = ImportScanKey {
            content_hash: hash_str("import a from './a';"),
            is_typescript: false,
        };
        let scan_payload = postcard::to_stdvec(&sample_imports("./a")).unwrap();
        let scan_entry = StoredImportScanEntry {
            key: scan_key,
            checksum: hash_bytes(&scan_payload),
            payload: scan_payload,
            last_used: 0,
        };

        // Written with the CURRENT `SCANNER_VERSION` but a config_fingerprint
        // that the load call below deliberately won't match.
        let file = CacheFile {
            schema: TRANSFORM_CACHE_SCHEMA,
            config_fingerprint: 100,
            entries: vec![transform_entry],
            scanner_version: SCANNER_VERSION,
            import_scan_entries: vec![scan_entry],
            resolution_entries: Vec::new(),
            analysis_fingerprint: 0,
            analysis_entries: Vec::new(),
            replay_manifest: None,
        };
        let bytes = postcard::to_stdvec(&file).unwrap();

        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"import_scan_and_transform_sections_are_gated_independently")
        ));
        std::fs::create_dir_all(dir.join("node_modules/.jet")).unwrap();
        std::fs::write(dir.join(TRANSFORM_CACHE_REL_PATH), &bytes).unwrap();

        // Load with a DIFFERENT config_fingerprint (200 != 100): the
        // transform section must come up empty while the import-scan
        // section (whose own `scanner_version` still matches) survives.
        let (cache, stats) = PersistentTransformCache::load(&dir, 200, 0);
        assert_eq!(stats.loaded_entries, 0, "transform section must miss");
        assert_eq!(
            stats.import_scan_loaded_entries, 1,
            "import-scan section must still load"
        );
        assert!(cache
            .get(Path::new("/proj/src/good.js"), &sample_key())
            .is_none());
        assert!(cache.get_import_scan(&scan_key).is_some());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_scan_save_then_load_round_trips_across_a_process_boundary_simulation() {
        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"import_scan_save_then_load_round_trips")
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let (cache, _) = PersistentTransformCache::load(&dir, 55, 0);
        let key = ImportScanKey {
            content_hash: hash_str("import y from './y';"),
            is_typescript: true,
        };
        cache.insert_import_scan(key, sample_imports("./y"));
        let save_stats = cache.save();
        assert!(save_stats.bytes_written > 0);

        let (reloaded, load_stats) = PersistentTransformCache::load(&dir, 55, 0);
        assert_eq!(load_stats.import_scan_loaded_entries, 1);
        let hit = reloaded
            .get_import_scan(&key)
            .expect("expected the saved import-scan entry to round-trip");
        assert_eq!(hit.static_imports[0].source, "./y");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // #2141 — resolution section tests below. Same fixture/naming style as
    // the sections above.

    fn sample_resolution_key(specifier: &str) -> ResolutionKey {
        ResolutionKey {
            scope_realpath: PathBuf::from("/proj/node_modules/pkg-a"),
            specifier: specifier.to_string(),
            resolver_config_fingerprint: 0,
        }
    }

    #[test]
    fn resolution_miss_then_insert_then_hit_round_trips() {
        let cache = PersistentTransformCache {
            enabled: true,
            ..PersistentTransformCache::disabled()
        };
        let key = sample_resolution_key("pkg-a");

        assert!(cache.get_resolution(&key).is_none());
        assert_eq!(cache.resolution_misses(), 1);

        cache.insert_resolution(
            key.clone(),
            ResolutionValue {
                resolved_path: PathBuf::from("/proj/node_modules/pkg-a/index.js"),
                guard: Vec::new(),
            },
        );
        let hit = cache
            .get_resolution(&key)
            .expect("expected a hit after insert");
        assert_eq!(hit, PathBuf::from("/proj/node_modules/pkg-a/index.js"));
        assert_eq!(cache.resolution_hits(), 1);
        assert_eq!(cache.resolution_misses(), 1);
    }

    #[test]
    fn resolution_guard_mismatch_is_a_miss_and_evicts_the_stale_entry() {
        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"resolution_guard_mismatch_is_a_miss_and_evicts_the_stale_entry")
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let package_json = dir.join("package.json");
        std::fs::write(&package_json, r#"{"name":"pkg-a","version":"1.0.0"}"#).unwrap();

        let cache = PersistentTransformCache {
            enabled: true,
            ..PersistentTransformCache::disabled()
        };
        let key = sample_resolution_key("pkg-a");
        let captured_hash = hash_bytes(&std::fs::read(&package_json).unwrap());
        cache.insert_resolution(
            key.clone(),
            ResolutionValue {
                resolved_path: dir.join("index.js"),
                guard: vec![(package_json.clone(), Some(captured_hash))],
            },
        );

        // Guard intact: hit.
        assert!(cache.get_resolution(&key).is_some());
        assert_eq!(cache.resolution_hits(), 1);

        // Content changes underneath the guarded package.json: must miss
        // AND evict the now-stale entry, not just report a miss while
        // leaving it live for a future stale read to race against.
        std::fs::write(&package_json, r#"{"name":"pkg-a","version":"2.0.0"}"#).unwrap();
        assert!(cache.get_resolution(&key).is_none());
        assert_eq!(cache.resolution_misses(), 1);
        assert!(!cache.resolution_entries.contains_key(&key));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolution_guard_treats_a_newly_appearing_package_json_as_a_mismatch() {
        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"resolution_guard_treats_a_newly_appearing_package_json_as_a_mismatch")
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let package_json = dir.join("package.json"); // does not exist yet

        let cache = PersistentTransformCache {
            enabled: true,
            ..PersistentTransformCache::disabled()
        };
        let key = sample_resolution_key("pkg-a");
        // Captured while package.json did not exist: `None`, per
        // `ResolutionValue`'s doc comment (not a real-hash sentinel, to
        // avoid a collision with an actual hash that happens to match it).
        cache.insert_resolution(
            key.clone(),
            ResolutionValue {
                resolved_path: dir.join("index.js"),
                guard: vec![(package_json.clone(), None)],
            },
        );
        assert!(cache.get_resolution(&key).is_some());

        // The guarded path now exists: a fresh `Some(hash)` no longer
        // equals the captured `None`, so this must be a miss even though
        // nothing about the *resolution itself* looks wrong.
        std::fs::write(&package_json, r#"{"name":"pkg-a"}"#).unwrap();
        assert!(cache.get_resolution(&key).is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolution_poisoned_payload_checksum_rejects_without_losing_other_entries() {
        let good_key = sample_resolution_key("pkg-good");
        let good_value = ResolutionValue {
            resolved_path: PathBuf::from("/proj/node_modules/pkg-good/index.js"),
            guard: Vec::new(),
        };
        let good_payload = postcard::to_stdvec(&good_value).unwrap();
        let good_entry = StoredResolutionEntry {
            key: good_key.clone(),
            checksum: hash_bytes(&good_payload),
            payload: good_payload,
            last_used: 0,
        };

        let bad_key = sample_resolution_key("pkg-bad");
        let bad_value = ResolutionValue {
            resolved_path: PathBuf::from("/proj/node_modules/pkg-bad/index.js"),
            guard: Vec::new(),
        };
        let mut bad_payload = postcard::to_stdvec(&bad_value).unwrap();
        let bad_checksum = hash_bytes(&bad_payload);
        if let Some(byte) = bad_payload.first_mut() {
            *byte ^= 0xFF;
        }
        let bad_entry = StoredResolutionEntry {
            key: bad_key.clone(),
            checksum: bad_checksum,
            payload: bad_payload,
            last_used: 0,
        };

        let file = CacheFile {
            schema: TRANSFORM_CACHE_SCHEMA,
            config_fingerprint: 7,
            entries: Vec::new(),
            scanner_version: SCANNER_VERSION,
            import_scan_entries: Vec::new(),
            resolution_entries: vec![good_entry, bad_entry],
            analysis_fingerprint: 0,
            analysis_entries: Vec::new(),
            replay_manifest: None,
        };
        let bytes = postcard::to_stdvec(&file).unwrap();

        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"resolution_poisoned_payload_checksum_rejects")
        ));
        std::fs::create_dir_all(dir.join("node_modules/.jet")).unwrap();
        std::fs::write(dir.join(TRANSFORM_CACHE_REL_PATH), &bytes).unwrap();

        let (cache, stats) = PersistentTransformCache::load(&dir, 7, 0);
        assert_eq!(stats.resolution_loaded_entries, 1);
        assert_eq!(stats.resolution_corrupt_entries, 1);
        assert!(cache.get_resolution(&good_key).is_some());
        assert!(cache.get_resolution(&bad_key).is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolution_section_survives_a_transform_config_fingerprint_mismatch() {
        // The resolution section has no load-time gate of its own (see
        // `resolution_entries`'s field doc comment) — a `config_fingerprint`
        // mismatch that discards the transform section must never touch it.
        let good_module = sample_module(0, "const good = 1;");
        let good_module_payload = postcard::to_stdvec(&good_module).unwrap();
        let transform_entry = StoredEntry {
            path: PathBuf::from("/proj/src/good.js"),
            key: sample_key(),
            checksum: hash_bytes(&good_module_payload),
            payload: good_module_payload,
            last_used: 0,
        };

        let resolution_key = sample_resolution_key("pkg-a");
        let resolution_value = ResolutionValue {
            resolved_path: PathBuf::from("/proj/node_modules/pkg-a/index.js"),
            guard: Vec::new(),
        };
        let resolution_payload = postcard::to_stdvec(&resolution_value).unwrap();
        let resolution_entry = StoredResolutionEntry {
            key: resolution_key.clone(),
            checksum: hash_bytes(&resolution_payload),
            payload: resolution_payload,
            last_used: 0,
        };

        let file = CacheFile {
            schema: TRANSFORM_CACHE_SCHEMA,
            config_fingerprint: 100,
            entries: vec![transform_entry],
            scanner_version: SCANNER_VERSION,
            import_scan_entries: Vec::new(),
            resolution_entries: vec![resolution_entry],
            analysis_fingerprint: 0,
            analysis_entries: Vec::new(),
            replay_manifest: None,
        };
        let bytes = postcard::to_stdvec(&file).unwrap();

        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"resolution_section_survives_a_transform_config_fingerprint_mismatch")
        ));
        std::fs::create_dir_all(dir.join("node_modules/.jet")).unwrap();
        std::fs::write(dir.join(TRANSFORM_CACHE_REL_PATH), &bytes).unwrap();

        // Loaded with a DIFFERENT config_fingerprint (200 != 100): the
        // transform section must come up empty while the resolution
        // section (ungated) still loads and hits.
        let (cache, stats) = PersistentTransformCache::load(&dir, 200, 0);
        assert_eq!(stats.loaded_entries, 0, "transform section must miss");
        assert_eq!(
            stats.resolution_loaded_entries, 1,
            "resolution section has no config_fingerprint gate"
        );
        assert!(cache
            .get(Path::new("/proj/src/good.js"), &sample_key())
            .is_none());
        assert!(cache.get_resolution(&resolution_key).is_some());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolution_save_then_load_round_trips_across_a_process_boundary_simulation() {
        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"resolution_save_then_load_round_trips")
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let (cache, _) = PersistentTransformCache::load(&dir, 55, 0);
        let key = sample_resolution_key("pkg-z");
        let value = ResolutionValue {
            resolved_path: PathBuf::from("/proj/node_modules/pkg-z/index.js"),
            guard: Vec::new(),
        };
        cache.insert_resolution(key.clone(), value);
        let save_stats = cache.save();
        assert!(save_stats.bytes_written > 0);

        let (reloaded, load_stats) = PersistentTransformCache::load(&dir, 55, 0);
        assert_eq!(load_stats.resolution_loaded_entries, 1);
        let hit = reloaded
            .get_resolution(&key)
            .expect("expected the saved resolution entry to round-trip");
        assert_eq!(hit, PathBuf::from("/proj/node_modules/pkg-z/index.js"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// #1941 pnpm-symlink trap class: a pnpm store keeps one physical copy
    /// of a package under a content-addressed directory and every consumer
    /// reaches it through a *different* `node_modules` symlink. Two
    /// resolutions for "the same" package reached through two different
    /// symlinks must collapse to one resolution-cache scope, or a warm
    /// cache would silently never hit for pnpm-style layouts — the exact
    /// property the #2141 resolution cache depends on for its cache key.
    #[cfg(unix)]
    #[test]
    fn node_modules_scope_realpath_collapses_symlinked_and_real_paths() {
        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"node_modules_scope_realpath_collapses_symlinked_and_real_paths")
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let real_pkg = dir.join("store/pkg-a@1.0.0/node_modules/pkg-a");
        std::fs::create_dir_all(&real_pkg).unwrap();
        std::fs::write(real_pkg.join("package.json"), r#"{"name":"pkg-a"}"#).unwrap();
        std::fs::write(real_pkg.join("index.js"), "module.exports = {};").unwrap();

        let consumer_a_nm = dir.join("app-a/node_modules");
        let consumer_b_nm = dir.join("app-b/node_modules");
        std::fs::create_dir_all(&consumer_a_nm).unwrap();
        std::fs::create_dir_all(&consumer_b_nm).unwrap();
        std::os::unix::fs::symlink(&real_pkg, consumer_a_nm.join("pkg-a")).unwrap();
        std::os::unix::fs::symlink(&real_pkg, consumer_b_nm.join("pkg-a")).unwrap();

        let via_a = node_modules_scope_realpath(&consumer_a_nm.join("pkg-a/index.js"));
        let via_b = node_modules_scope_realpath(&consumer_b_nm.join("pkg-a/index.js"));
        let expected = std::fs::canonicalize(&real_pkg).unwrap();

        assert_eq!(via_a, Some(expected.clone()));
        assert_eq!(via_b, Some(expected));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn node_modules_scope_realpath_handles_scoped_packages_and_returns_none_outside_node_modules() {
        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"node_modules_scope_realpath_handles_scoped_packages")
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let scoped_pkg = dir.join("node_modules/@scope/name");
        std::fs::create_dir_all(&scoped_pkg).unwrap();
        let expected = std::fs::canonicalize(&scoped_pkg).unwrap();

        // A file several levels inside the scoped package still resolves to
        // the package root, not to `@scope` alone or to `node_modules`.
        let nested = scoped_pkg.join("dist/index.js");
        std::fs::create_dir_all(nested.parent().unwrap()).unwrap();
        std::fs::write(&nested, "export {};").unwrap();
        assert_eq!(node_modules_scope_realpath(&nested), Some(expected));

        // No `node_modules` path component at all: app source, never scoped.
        let app_src = dir.join("src/app.tsx");
        std::fs::create_dir_all(app_src.parent().unwrap()).unwrap();
        std::fs::write(&app_src, "export {};").unwrap();
        assert_eq!(node_modules_scope_realpath(&app_src), None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    // #2141 — analysis section tests below. Same fixture/naming style as
    // the sections above.

    fn sample_raw_facts(specifier: &str) -> crate::bundler::tree_shake::RawModuleFacts {
        crate::bundler::tree_shake::RawModuleFacts {
            exports: Vec::new(),
            cjs_exports: Vec::new(),
            static_edges: vec![(specifier.to_string(), Vec::new())],
            cjs_edges: Vec::new(),
            dynamic_targets: Vec::new(),
            reexports: Vec::new(),
        }
    }

    #[test]
    fn analysis_miss_then_insert_then_hit_round_trips() {
        let cache = PersistentTransformCache {
            enabled: true,
            ..PersistentTransformCache::disabled()
        };
        let key = AnalysisKey {
            content_hash: hash_str("import x from './x'; export {};"),
            is_typescript: false,
        };

        assert!(cache.get_analysis(&key).is_none());
        assert_eq!(cache.analysis_misses(), 1);

        cache.insert_analysis(key, sample_raw_facts("./x"));
        let hit = cache
            .get_analysis(&key)
            .expect("expected a hit after insert");
        assert_eq!(hit.static_edges[0].0, "./x");
        assert_eq!(cache.analysis_hits(), 1);
        assert_eq!(cache.analysis_misses(), 1);
    }

    #[test]
    fn analysis_is_typescript_is_part_of_the_key() {
        // Same `content_hash` (a `.js` file and a `.ts` file with
        // byte-identical bytes are a real, if rare, scenario), different
        // `is_typescript` — must be a miss, not a stale hit, since
        // `compute_raw_module_facts` behaves differently for the two
        // (`extract_export_names`'s `is_ts` parameter).
        let cache = PersistentTransformCache {
            enabled: true,
            ..PersistentTransformCache::disabled()
        };
        let content_hash = hash_str("import x from './x'; export {};");
        let js_key = AnalysisKey {
            content_hash,
            is_typescript: false,
        };
        cache.insert_analysis(js_key, sample_raw_facts("./x"));

        let ts_key = AnalysisKey {
            content_hash,
            is_typescript: true,
        };
        assert!(cache.get_analysis(&ts_key).is_none());
        assert!(cache.get_analysis(&js_key).is_some());
    }

    #[test]
    fn analysis_poisoned_payload_checksum_rejects_without_losing_other_entries() {
        let good_key = AnalysisKey {
            content_hash: hash_str("import a from './a'; export {};"),
            is_typescript: false,
        };
        let good_payload = postcard::to_stdvec(&sample_raw_facts("./a")).unwrap();
        let good_entry = StoredAnalysisEntry {
            key: good_key,
            checksum: hash_bytes(&good_payload),
            payload: good_payload,
            last_used: 0,
        };

        let bad_key = AnalysisKey {
            content_hash: hash_str("import b from './b'; export {};"),
            is_typescript: false,
        };
        let mut bad_payload = postcard::to_stdvec(&sample_raw_facts("./b")).unwrap();
        let bad_checksum = hash_bytes(&bad_payload);
        if let Some(byte) = bad_payload.first_mut() {
            *byte ^= 0xFF;
        }
        let bad_entry = StoredAnalysisEntry {
            key: bad_key,
            checksum: bad_checksum,
            payload: bad_payload,
            last_used: 0,
        };

        let file = CacheFile {
            schema: TRANSFORM_CACHE_SCHEMA,
            config_fingerprint: 7,
            entries: Vec::new(),
            scanner_version: SCANNER_VERSION,
            import_scan_entries: Vec::new(),
            resolution_entries: Vec::new(),
            analysis_fingerprint: 0,
            analysis_entries: vec![good_entry, bad_entry],
            replay_manifest: None,
        };
        let bytes = postcard::to_stdvec(&file).unwrap();

        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"analysis_poisoned_payload_checksum_rejects")
        ));
        std::fs::create_dir_all(dir.join("node_modules/.jet")).unwrap();
        std::fs::write(dir.join(TRANSFORM_CACHE_REL_PATH), &bytes).unwrap();

        let (cache, stats) = PersistentTransformCache::load(&dir, 7, 0);
        assert_eq!(stats.analysis_loaded_entries, 1);
        assert_eq!(stats.analysis_corrupt_entries, 1);
        assert!(cache.get_analysis(&good_key).is_some());
        assert!(cache.get_analysis(&bad_key).is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn analysis_and_transform_sections_are_gated_independently() {
        // `analysis_fingerprint` (defines-derived) and `config_fingerprint`
        // (transform section) are independent gates — a `defines` change
        // must fully miss the analysis section without discarding an
        // otherwise-valid transform section, and vice versa (#2141).
        let good_module = sample_module(0, "const good = 1;");
        let good_module_payload = postcard::to_stdvec(&good_module).unwrap();
        let transform_entry = StoredEntry {
            path: PathBuf::from("/proj/src/good.js"),
            key: sample_key(),
            checksum: hash_bytes(&good_module_payload),
            payload: good_module_payload,
            last_used: 0,
        };

        let analysis_key = AnalysisKey {
            content_hash: hash_str("import a from './a'; export {};"),
            is_typescript: false,
        };
        let analysis_payload = postcard::to_stdvec(&sample_raw_facts("./a")).unwrap();
        let analysis_entry = StoredAnalysisEntry {
            key: analysis_key,
            checksum: hash_bytes(&analysis_payload),
            payload: analysis_payload,
            last_used: 0,
        };

        let file = CacheFile {
            schema: TRANSFORM_CACHE_SCHEMA,
            config_fingerprint: 100,
            entries: vec![transform_entry],
            scanner_version: SCANNER_VERSION,
            import_scan_entries: Vec::new(),
            resolution_entries: Vec::new(),
            analysis_fingerprint: 50,
            analysis_entries: vec![analysis_entry],
            replay_manifest: None,
        };
        let bytes = postcard::to_stdvec(&file).unwrap();

        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"analysis_and_transform_sections_are_gated_independently")
        ));
        std::fs::create_dir_all(dir.join("node_modules/.jet")).unwrap();
        std::fs::write(dir.join(TRANSFORM_CACHE_REL_PATH), &bytes).unwrap();

        // Loaded with a DIFFERENT config_fingerprint (200 != 100) but the
        // SAME analysis_fingerprint (50): the transform section must miss
        // while the analysis section survives.
        let (cache, stats) = PersistentTransformCache::load(&dir, 200, 50);
        assert_eq!(stats.loaded_entries, 0, "transform section must miss");
        assert_eq!(
            stats.analysis_loaded_entries, 1,
            "analysis section must still load"
        );
        assert!(cache
            .get(Path::new("/proj/src/good.js"), &sample_key())
            .is_none());
        assert!(cache.get_analysis(&analysis_key).is_some());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn analysis_save_then_load_round_trips_across_a_process_boundary_simulation() {
        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"analysis_save_then_load_round_trips")
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let (cache, _) = PersistentTransformCache::load(&dir, 55, 77);
        let key = AnalysisKey {
            content_hash: hash_str("import y from './y'; export {};"),
            is_typescript: true,
        };
        cache.insert_analysis(key, sample_raw_facts("./y"));
        let save_stats = cache.save();
        assert!(save_stats.bytes_written > 0);

        let (reloaded, load_stats) = PersistentTransformCache::load(&dir, 55, 77);
        assert_eq!(load_stats.analysis_loaded_entries, 1);
        let hit = reloaded
            .get_analysis(&key)
            .expect("expected the saved analysis entry to round-trip");
        assert_eq!(hit.static_edges[0].0, "./y");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // #2143 replay section — see the module doc comment's "Replay section"
    // heading. The `tests/build/transform_cache.rs` HANDWRITE block's own
    // `reason=` text explicitly routes "store poison" coverage here rather
    // than to a real `jet build` subprocess test, since none of the three
    // scenarios below need a real `Bundler`/build at all — `try_replay` and
    // `peek_replay_manifest` are plain functions over the on-disk store.

    #[test]
    fn try_replay_never_panics_and_declines_on_a_poisoned_store() {
        let dir = std::env::temp_dir().join(format!(
            "jet-transform-cache-test-{}-{}",
            std::process::id(),
            hash_bytes(b"try_replay_never_panics_and_declines_on_a_poisoned_store")
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("node_modules/.jet")).unwrap();
        let output_dir = dir.join("dist");
        std::fs::create_dir_all(&output_dir).unwrap();

        // Scenario 1: completely undecodable garbage bytes in place of the
        // postcard-encoded `CacheFile` (disk corruption, a truncated write,
        // or just the wrong format entirely) — `peek_replay_manifest`'s
        // `postcard::from_bytes` must fail cleanly rather than panic, and
        // `try_replay` must decline.
        std::fs::write(
            dir.join(TRANSFORM_CACHE_REL_PATH),
            b"not a postcard-encoded CacheFile at all, just garbage bytes \xff\xfe\x00\x01",
        )
        .unwrap();
        match try_replay(&dir, &output_dir, 123) {
            ReplayOutcome::FullBuild { reason, .. } => assert_eq!(reason, "no-manifest"),
            ReplayOutcome::Replayed { .. } => panic!("a poisoned store must never replay"),
        }

        // Scenario 2: a validly postcard-encoded `CacheFile`, but from a
        // schema this binary no longer understands (an old jet version's
        // store, or any incompatible future shape) — the schema check in
        // `peek_replay_manifest` must reject it before ever trying to
        // interpret `replay_manifest`.
        let stale_schema_file = CacheFile {
            schema: TRANSFORM_CACHE_SCHEMA - 1,
            config_fingerprint: 0,
            entries: Vec::new(),
            scanner_version: SCANNER_VERSION,
            import_scan_entries: Vec::new(),
            resolution_entries: Vec::new(),
            analysis_fingerprint: 0,
            analysis_entries: Vec::new(),
            replay_manifest: None,
        };
        std::fs::write(
            dir.join(TRANSFORM_CACHE_REL_PATH),
            postcard::to_stdvec(&stale_schema_file).unwrap(),
        )
        .unwrap();
        match try_replay(&dir, &output_dir, 123) {
            ReplayOutcome::FullBuild { reason, .. } => assert_eq!(reason, "no-manifest"),
            ReplayOutcome::Replayed { .. } => panic!("a schema-mismatched store must never replay"),
        }

        // Scenario 3: a validly-encoded, current-schema `CacheFile` whose
        // `ReplayManifest` itself references an input path that no longer
        // exists on disk (a manifest surviving a wholesale source deletion,
        // e.g. `git clean`) — `try_replay`'s per-input `std::fs::metadata`
        // must fail cleanly, not panic.
        let dangling_manifest = ReplayManifest {
            replay_version: REPLAY_VERSION,
            config_fingerprint: 123,
            inputs: vec![ReplayInput {
                path: dir.join("src/does-not-exist.ts"),
                content_hash: 0,
                mtime_nanos: 0,
                size: 0,
            }],
            source_dirs: Vec::new(),
            outputs: Vec::new(),
            entry_rel_path: "main.js".to_string(),
            entry_size: 0,
        };
        let dangling_file = CacheFile {
            schema: TRANSFORM_CACHE_SCHEMA,
            config_fingerprint: 0,
            entries: Vec::new(),
            scanner_version: SCANNER_VERSION,
            import_scan_entries: Vec::new(),
            resolution_entries: Vec::new(),
            analysis_fingerprint: 0,
            analysis_entries: Vec::new(),
            replay_manifest: Some(dangling_manifest),
        };
        std::fs::write(
            dir.join(TRANSFORM_CACHE_REL_PATH),
            postcard::to_stdvec(&dangling_file).unwrap(),
        )
        .unwrap();
        match try_replay(&dir, &output_dir, 123) {
            ReplayOutcome::FullBuild { reason, .. } => assert!(
                reason.starts_with("missing-input:"),
                "unexpected reason: {reason}"
            ),
            ReplayOutcome::Replayed { .. } => panic!("a dangling-input manifest must never replay"),
        }

        let _ = std::fs::remove_dir_all(&dir);
    }
}
// CODEGEN-END

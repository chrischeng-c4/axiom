// HANDWRITE-BEGIN gap="missing-generator:unit-test:81533d65" tracker="pending-tracker" reason="Integration tests: WI #2137's persistent, content-addressed transform cache (node_modules/.jet/transform-cache.bin) makes a cache-hit rebuild of an unchanged real-node_modules fixture produce byte-identical dist/ output to a cache-deleted (cold) build with zero cache misses on the warm run; editing exactly one leaf module's JSX text content (no import/structural change) between two builds in the same project directory misses only that module's cache entry (every other module's dep_fingerprint-keyed entry still hits) and produces output byte-identical to a from-scratch build of the same edited source; and changing a --define between two builds invalidates the entire store via the config fingerprint (zero hits, every module re-transformed) while still producing correct output. Poisoned/corrupt on-disk entries are covered at the unit level in src/bundler/persistent_cache.rs's own #[cfg(test)] mod tests, not repeated here."
//! Persistent, content-addressed transform cache correctness gates
//! (#2137, beat-vite epic #1990).
//!
//! Drives the real `jet build` CLI path as a subprocess (not an in-process
//! `Bundler` call) against `tests/fixtures/dom-production-build/mui-visual/`
//! — a real-node_modules fixture (symlinks into `~/.jet-store`, no network
//! needed) — because the persistent cache only matters *across separate
//! process invocations*: an in-process `Bundler`'s `CompilationCache`
//! DashMap starts empty every time regardless, so a single-process test
//! would never exercise the disk-backed path this file covers.
//!
//! Every build passes `--sourcemap none` so absolute tempdir paths never
//! leak into compared output. Every rebuild into an already-built directory
//! first removes `dist/` directly (`clean_dist_dir`) rather than relying on
//! `jet build --empty-out-dir`: that flag is parsed in `src/cli.rs` but
//! `crate::build_clean::empty_out_dir` is only ever invoked from the
//! `--wasm` build path (`src/wasm_build/mod.rs`), not from the regular
//! (non-wasm) `jet build` handler these tests exercise, so today it silently
//! leaves stale content-hashed files behind on a second build into the same
//! `dist/`. Pre-existing and unrelated to #2137; out of scope here.
//!
//! #2140 extends the determinism and stale-guard tests below with the same
//! assertions against the store's independent import-scan section (its own
//! `[bundle-timing] import-scan: ... i_hits=N i_misses=M` line) rather than
//! adding new top-level tests, per the same reuse-not-duplicate-harness
//! intent as the rest of this file. Its poisoned-entry coverage is likewise
//! unit-level only (`persistent_cache.rs`'s `import_scan_*` tests).
//!
//! #2141 extends the determinism, stale-guard, and config-change tests
//! below with the same reconciliation assertions against the store's two
//! remaining sections: node_modules-scoped resolution and per-module
//! raw-facts analysis (`r_hits`/`r_misses`/`a_hits`/`a_misses`, all on the
//! same `[bundle-timing] cache: ...` line) — plus one new top-level test
//! for the resolution section's own per-entry guard fingerprint (editing a
//! package.json a real resolution consulted), which has no analogue in the
//! pre-#2141 sections since only resolution entries carry a per-entry
//! (rather than whole-section) validity check. Poisoned-entry and
//! section-independence coverage for both new sections is unit-level only
//! (`persistent_cache.rs`'s `resolution_*`/`analysis_*` tests), same
//! reuse-not-duplicate-harness intent as #2140.

use anyhow::{anyhow, Context, Result};
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture_source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("dom-production-build")
        .join("mui-visual")
}

/// Recursive copy that preserves symlinks instead of dereferencing them.
///
/// jet's local package store (`~/.jet-store/<pkg>@<version>`) is referenced
/// from a fixture's `node_modules/` via symlinks that occur at *any*
/// nesting depth (e.g. `node_modules/clsx` is a direct top-level symlink,
/// but `node_modules/@mui/material` is a symlink one level inside a real
/// `@mui` scope directory) — every directory level must be checked, not
/// just the top. A naive `read_dir` walk using `fs::copy` on everything
/// would instead dereference-and-materialize each symlink's target tree,
/// multiplying a small fixture into hundreds of MB.
fn copy_dir_all_preserving_symlinks(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src).with_context(|| format!("read {}", src.display()))? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            let target = fs::read_link(&src_path)
                .with_context(|| format!("read_link {}", src_path.display()))?;
            std::os::unix::fs::symlink(&target, &dst_path).with_context(|| {
                format!("symlink {} -> {}", dst_path.display(), target.display())
            })?;
        } else if file_type.is_dir() {
            copy_dir_all_preserving_symlinks(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).with_context(|| {
                format!("copy {} -> {}", src_path.display(), dst_path.display())
            })?;
        }
    }
    Ok(())
}

fn copy_fixture(dst: &Path) -> Result<()> {
    copy_dir_all_preserving_symlinks(&fixture_source(), dst)
}

/// Remove `fixture/dist` if present. See the module doc comment: this
/// stands in for `jet build --empty-out-dir`, which does not actually
/// clear the output directory on the (non-wasm) path these tests exercise.
/// A no-op (nothing to remove) on a fixture's first build.
fn clean_dist_dir(fixture: &Path) {
    let _ = fs::remove_dir_all(fixture.join("dist"));
}

/// Replaces a top-level `node_modules/<pkg>` symlink (jet's local package
/// store convention — see `copy_dir_all_preserving_symlinks`) with a real,
/// physically-copied directory holding the exact same content (read-only
/// off the symlink's target). #2141's resolution-guard test needs to edit
/// one real, resolved package's `package.json` in place; doing that
/// directly through the fixture's symlink would write through to
/// `~/.jet-store`, the same machine-wide content-addressed store every
/// other project (and concurrent test run) on this host resolves against.
/// Materializing first scopes the mutation to this one tempdir copy.
fn materialize_node_modules_package(fixture: &Path, pkg: &str) -> Result<()> {
    let link_path = fixture.join("node_modules").join(pkg);
    let target =
        fs::read_link(&link_path).with_context(|| format!("read_link {}", link_path.display()))?;
    fs::remove_file(&link_path)
        .with_context(|| format!("remove symlink {}", link_path.display()))?;
    copy_dir_all_preserving_symlinks(&target, &link_path).with_context(|| {
        format!(
            "materialize {} from {}",
            link_path.display(),
            target.display()
        )
    })
}

fn run_jet<I, S>(fixture: &Path, args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_jet"))
        .args(args)
        .current_dir(fixture)
        // Every assertion in this file reads the `[bundle-timing] cache: ...`
        // hit/miss line, so every invocation needs it on.
        .env("JET_BUNDLE_TIMING", "1")
        .output()
        .context("run jet command")
}

/// Same as `run_jet`, but with `JET_NO_REPLAY=1` set — the #2143 replay
/// hatch that disables both the pre-build fast-path check and the
/// post-build manifest write (see the "Hatches" heading in
/// `persistent_cache.rs`'s module doc comment). Used to produce a
/// forced-full-build baseline to diff a replayed build's `dist/` output
/// against (AC2).
fn run_jet_no_replay<I, S>(fixture: &Path, args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_jet"))
        .args(args)
        .current_dir(fixture)
        .env("JET_BUNDLE_TIMING", "1")
        .env("JET_NO_REPLAY", "1")
        .output()
        .context("run jet command (JET_NO_REPLAY=1)")
}

fn require_success(output: Output, phase: &str) -> Result<Output> {
    if output.status.success() {
        return Ok(output);
    }
    Err(anyhow!(
        "{phase} failed\nstatus={}\nstdout={}\nstderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

#[derive(Debug, Clone, Copy)]
struct CacheTiming {
    hits: u64,
    misses: u64,
    /// #2141 — node_modules-scoped resolution section counters, same
    /// `[bundle-timing] cache: ...` line as `hits`/`misses` above.
    r_hits: u64,
    r_misses: u64,
    /// #2141 — per-module raw-facts analysis section counters, same line.
    a_hits: u64,
    a_misses: u64,
}

/// Parse the `[bundle-timing] cache: hits=N misses=M loaded_in=Xms
/// saved_in=Yms bytes=Z r_hits=A r_misses=B a_hits=C a_misses=D` line `jet
/// build` emits to stderr under `JET_BUNDLE_TIMING=1` (see `Bundler::bundle`
/// in `src/bundler/mod.rs`).
fn parse_cache_timing_line(stderr: &str) -> Option<CacheTiming> {
    let line = stderr
        .lines()
        .find(|line| line.starts_with("[bundle-timing] cache:"))?;
    let mut hits = None;
    let mut misses = None;
    let mut r_hits = None;
    let mut r_misses = None;
    let mut a_hits = None;
    let mut a_misses = None;
    for field in line.split_whitespace() {
        if let Some(v) = field.strip_prefix("hits=") {
            hits = v.parse::<u64>().ok();
        } else if let Some(v) = field.strip_prefix("misses=") {
            misses = v.parse::<u64>().ok();
        } else if let Some(v) = field.strip_prefix("r_hits=") {
            r_hits = v.parse::<u64>().ok();
        } else if let Some(v) = field.strip_prefix("r_misses=") {
            r_misses = v.parse::<u64>().ok();
        } else if let Some(v) = field.strip_prefix("a_hits=") {
            a_hits = v.parse::<u64>().ok();
        } else if let Some(v) = field.strip_prefix("a_misses=") {
            a_misses = v.parse::<u64>().ok();
        }
    }
    Some(CacheTiming {
        hits: hits?,
        misses: misses?,
        r_hits: r_hits?,
        r_misses: r_misses?,
        a_hits: a_hits?,
        a_misses: a_misses?,
    })
}

#[derive(Debug, Clone, Copy)]
struct ImportScanTiming {
    fast: u64,
    fallback: u64,
    hits: u64,
    misses: u64,
}

/// Parse the `[bundle-timing] import-scan: fast=X fallback=Y i_hits=N
/// i_misses=M` line `jet build` emits to stderr under
/// `JET_BUNDLE_TIMING=1` (#2140; see `Bundler::build_graph` in
/// `src/bundler/mod.rs`). `i_hits`/`i_misses` count persistent import-scan
/// cache lookups (a section of the same `transform-cache.bin` store the
/// `[bundle-timing] cache: ...` line above reports on, gated independently
/// via `SCANNER_VERSION` rather than `config_fingerprint`); `fast`/
/// `fallback` count which raw scan strategy ran for modules that missed.
fn parse_import_scan_timing_line(stderr: &str) -> Option<ImportScanTiming> {
    let line = stderr
        .lines()
        .find(|line| line.starts_with("[bundle-timing] import-scan:"))?;
    let mut fast = None;
    let mut fallback = None;
    let mut hits = None;
    let mut misses = None;
    for field in line.split_whitespace() {
        if let Some(v) = field.strip_prefix("fast=") {
            fast = v.parse::<u64>().ok();
        } else if let Some(v) = field.strip_prefix("fallback=") {
            fallback = v.parse::<u64>().ok();
        } else if let Some(v) = field.strip_prefix("i_hits=") {
            hits = v.parse::<u64>().ok();
        } else if let Some(v) = field.strip_prefix("i_misses=") {
            misses = v.parse::<u64>().ok();
        }
    }
    Some(ImportScanTiming {
        fast: fast?,
        fallback: fallback?,
        hits: hits?,
        misses: misses?,
    })
}

#[derive(Debug, Clone)]
struct ReplayTiming {
    verified: usize,
    stat_ms: f64,
    hash_fallback: usize,
    outcome: String,
}

/// Parse the `[bundle-timing] replay: verified=N stat_ms=X hash_fallback=K
/// outcome=replayed|full-build(<reason>)` line `jet build` emits to stderr
/// under `JET_BUNDLE_TIMING=1` (#2143; see `ReplayOutcome::timing_line` in
/// `src/bundler/persistent_cache.rs`). `outcome` is parsed via
/// `split("outcome=")` rather than `split_whitespace()`, unlike the other
/// fields: a `full-build` reason can itself embed a filesystem path
/// (`missing-input:<path>`, `dir-changed:<path>`, ...), and a path is not
/// guaranteed to be whitespace-free.
fn parse_replay_timing_line(stderr: &str) -> Option<ReplayTiming> {
    let line = stderr
        .lines()
        .find(|line| line.starts_with("[bundle-timing] replay:"))?;
    let mut verified = None;
    let mut stat_ms = None;
    let mut hash_fallback = None;
    for field in line.split_whitespace() {
        if let Some(v) = field.strip_prefix("verified=") {
            verified = v.parse::<usize>().ok();
        } else if let Some(v) = field.strip_prefix("stat_ms=") {
            stat_ms = v.parse::<f64>().ok();
        } else if let Some(v) = field.strip_prefix("hash_fallback=") {
            hash_fallback = v.parse::<usize>().ok();
        }
    }
    let outcome = line.split("outcome=").nth(1)?.trim().to_string();
    Some(ReplayTiming {
        verified: verified?,
        stat_ms: stat_ms?,
        hash_fallback: hash_fallback?,
        outcome,
    })
}

fn content_hash(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

/// Filename -> content hash of every file directly under `dist/` (the
/// fixture's build output is flat). Used to assert byte-identical output
/// across separate builds/tempdirs without comparing absolute paths.
fn dist_signature(fixture: &Path) -> Result<BTreeMap<String, u64>> {
    let dist = fixture.join("dist");
    let mut signature = BTreeMap::new();
    for entry in fs::read_dir(&dist).with_context(|| format!("read {}", dist.display()))? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry
            .file_name()
            .to_str()
            .ok_or_else(|| anyhow!("non-utf8 dist filename"))?
            .to_string();
        let bytes =
            fs::read(entry.path()).with_context(|| format!("read {}", entry.path().display()))?;
        signature.insert(name, content_hash(&bytes));
    }
    Ok(signature)
}

/// Mutates the fixture's MUI heading text in place — a pure JSX text-node
/// edit with no import/structural change, so it only invalidates
/// `MuiVisualFixture.tsx`'s own `content_hash`; every other module's
/// dependency graph shape (and therefore `dep_fingerprint`) is untouched.
fn edit_fixture_heading_text(fixture: &Path) -> Result<()> {
    let path = fixture.join("src/MuiVisualFixture.tsx");
    let original = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let needle = "MUI visual table fixture";
    let occurrences = original.matches(needle).count();
    if occurrences != 1 {
        return Err(anyhow!(
            "expected exactly one occurrence of {needle:?} in {}, found {occurrences}",
            path.display()
        ));
    }
    let edited = original.replacen(needle, "MUI visual table fixture (edited)", 1);
    fs::write(&path, edited).with_context(|| format!("write {}", path.display()))
}

/// Bumps a file's mtime forward (content byte-for-byte unchanged) via
/// `File::set_modified` rather than relying on real wall-clock time to
/// elapse between two fast test builds — a filesystem's mtime write and
/// this process's next syscall can land in the same tick on a fast disk, so
/// real time deltas alone are not a reliable enough signal. #2143's
/// `try_replay` stat screen keys on `(mtime, size)`; size is unchanged here
/// (same bytes rewritten first), so only mtime actually drifts.
fn bump_mtime(path: &Path) -> Result<()> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    fs::write(path, &bytes).with_context(|| format!("rewrite {}", path.display()))?;
    let file = fs::OpenOptions::new()
        .write(true)
        .open(path)
        .with_context(|| format!("open {} to bump mtime", path.display()))?;
    let new_time = std::time::SystemTime::now() + std::time::Duration::from_secs(60 * 60 * 24);
    file.set_modified(new_time)
        .with_context(|| format!("set_modified {}", path.display()))
}

const BUILD_ARGS: [&str; 3] = ["build", "--sourcemap", "none"];

/// AC1 — determinism A/B: a cache-deleted (cold) build and a subsequent
/// cache-hit (warm) rebuild of the exact same, unchanged sources must
/// produce byte-identical `dist/` output (entry content hash + chunk set),
/// and the warm rebuild must hit every module the cold build transformed.
#[test]
fn persistent_cache_determinism_cold_vs_warm_rebuild() -> Result<()> {
    let temp = tempfile::tempdir().context("temp fixture")?;
    let fixture = temp.path().join("mui-visual");
    copy_fixture(&fixture).context("copy mui-visual fixture")?;

    clean_dist_dir(&fixture);
    let cold = require_success(run_jet(&fixture, BUILD_ARGS)?, "cold build")?;
    let cold_stderr = String::from_utf8_lossy(&cold.stderr).into_owned();
    let cold_timing = parse_cache_timing_line(&cold_stderr).ok_or_else(|| {
        anyhow!("cold build did not print a cache timing line\nstderr={cold_stderr}")
    })?;
    assert_eq!(
        cold_timing.hits, 0,
        "cold build (no prior store) must have zero cache hits; stderr={cold_stderr}"
    );
    assert!(
        cold_timing.misses > 1,
        "expected the mui-visual fixture to transform more than one module; stderr={cold_stderr}"
    );
    assert!(
        fixture
            .join("node_modules/.jet/transform-cache.bin")
            .exists(),
        "cold build did not write the persistent transform cache store"
    );
    // #2141: a cold store has no resolution/analysis entries loaded from
    // disk, but both sections double as intra-run memoization (the
    // resolution key is scoped to the *importing package*, not the
    // importing file — see `node_modules_scope_realpath` — so dozens of
    // sibling files inside one node_modules package that import the same
    // bare specifier share a single key and hit each other within the same
    // build; analysis can likewise see a handful of intra-run hits from
    // modules with byte-identical content). So this checks the
    // miss/at-least-one relationship rather than asserting hits == 0 —
    // same reasoning as the import-scan cold-run comment below.
    assert!(
        cold_timing.r_misses > 0,
        "expected the mui-visual fixture to resolve at least one \
         node_modules-scoped bare specifier; stderr={cold_stderr}"
    );
    assert!(
        cold_timing.a_misses > 0,
        "expected the mui-visual fixture to analyze at least one module; \
         stderr={cold_stderr}"
    );
    // #2140: every fresh import scan (fast-path or tree-sitter fallback) is
    // exactly one import-scan cache miss; holds on any run regardless of
    // how many modules happen to share byte-identical content (which can
    // produce a handful of intra-run cache *hits* even on an otherwise
    // cold store — e.g. generated barrel re-export stubs — so this checks
    // the miss/scan relationship rather than asserting hits == 0).
    let cold_import_scan = parse_import_scan_timing_line(&cold_stderr).ok_or_else(|| {
        anyhow!("cold build did not print an import-scan timing line\nstderr={cold_stderr}")
    })?;
    assert_eq!(
        cold_import_scan.misses,
        cold_import_scan.fast + cold_import_scan.fallback,
        "cold build: every import-scan cache miss must correspond to exactly \
         one fast-path or fallback scan; stderr={cold_stderr}"
    );
    let cold_signature = dist_signature(&fixture).context("hash cold dist output")?;

    clean_dist_dir(&fixture);
    let warm = require_success(run_jet(&fixture, BUILD_ARGS)?, "warm rebuild")?;
    let warm_stderr = String::from_utf8_lossy(&warm.stderr).into_owned();
    let warm_timing = parse_cache_timing_line(&warm_stderr).ok_or_else(|| {
        anyhow!("warm rebuild did not print a cache timing line\nstderr={warm_stderr}")
    })?;
    assert_eq!(
        warm_timing.misses, 0,
        "warm rebuild (unchanged sources) must have zero cache misses; stderr={warm_stderr}"
    );
    assert_eq!(
        warm_timing.hits, cold_timing.misses,
        "warm rebuild must hit exactly the modules the cold build transformed; \
         cold misses={} warm hits={}\nstderr={warm_stderr}",
        cold_timing.misses, warm_timing.hits
    );
    // #2141: an unchanged rebuild must have zero resolution/analysis
    // misses, and must hit every resolution/analysis attempt the cold build
    // made — hits + misses, not just misses (same intra-run-reuse reasoning
    // as the cold-run comment above and the import-scan hits comparison
    // below) — the AC1 "r_hits/a_hits at 100%" numbers.
    assert_eq!(
        warm_timing.r_misses, 0,
        "warm rebuild (unchanged sources) must have zero resolution cache \
         misses; stderr={warm_stderr}"
    );
    assert_eq!(
        warm_timing.r_hits,
        cold_timing.r_hits + cold_timing.r_misses,
        "warm rebuild must resolution-cache-hit every bare specifier \
         resolution the cold build made (fresh resolutions + intra-run \
         dedup hits); cold r_hits={} r_misses={} warm r_hits={}\n\
         stderr={warm_stderr}",
        cold_timing.r_hits,
        cold_timing.r_misses,
        warm_timing.r_hits
    );
    assert_eq!(
        warm_timing.a_misses, 0,
        "warm rebuild (unchanged sources) must have zero analysis cache \
         misses; stderr={warm_stderr}"
    );
    assert_eq!(
        warm_timing.a_hits,
        cold_timing.a_hits + cold_timing.a_misses,
        "warm rebuild must analysis-cache-hit every module analysis the \
         cold build made (fresh analyses + intra-run dedup hits); \
         cold a_hits={} a_misses={} warm a_hits={}\nstderr={warm_stderr}",
        cold_timing.a_hits,
        cold_timing.a_misses,
        warm_timing.a_hits
    );
    // #2140: an unchanged rebuild must have zero import-scan misses, and
    // must hit every content signature the cold build either scanned fresh
    // or already deduped in-memory (hits + misses, not just misses — see
    // the cold-run comment above on same-content intra-run hits).
    let warm_import_scan = parse_import_scan_timing_line(&warm_stderr).ok_or_else(|| {
        anyhow!("warm rebuild did not print an import-scan timing line\nstderr={warm_stderr}")
    })?;
    assert_eq!(
        warm_import_scan.misses, 0,
        "warm rebuild (unchanged sources) must have zero import-scan cache \
         misses; stderr={warm_stderr}"
    );
    assert_eq!(
        warm_import_scan.hits,
        cold_import_scan.hits + cold_import_scan.misses,
        "warm rebuild must import-scan-cache-hit every content signature the \
         cold build produced (fresh scans + intra-run dedup hits); \
         cold hits={} misses={} warm hits={}\nstderr={warm_stderr}",
        cold_import_scan.hits,
        cold_import_scan.misses,
        warm_import_scan.hits
    );
    let warm_signature = dist_signature(&fixture).context("hash warm dist output")?;

    assert_eq!(
        cold_signature, warm_signature,
        "a cache-hit rebuild must produce byte-identical dist/ output to the \
         cache-deleted (cold) build"
    );

    eprintln!(
        "[transform_cache] determinism: cold hits={} misses={}; warm hits={} misses={}; \
         import-scan cold hits={} misses={}; warm hits={} misses={}; \
         resolution cold r_hits={} r_misses={}; warm r_hits={} r_misses={}; \
         analysis cold a_hits={} a_misses={}; warm a_hits={} a_misses={}; dist files={}",
        cold_timing.hits,
        cold_timing.misses,
        warm_timing.hits,
        warm_timing.misses,
        cold_import_scan.hits,
        cold_import_scan.misses,
        warm_import_scan.hits,
        warm_import_scan.misses,
        cold_timing.r_hits,
        cold_timing.r_misses,
        warm_timing.r_hits,
        warm_timing.r_misses,
        cold_timing.a_hits,
        cold_timing.a_misses,
        warm_timing.a_hits,
        warm_timing.a_misses,
        warm_signature.len(),
    );
    Ok(())
}

/// AC3 — stale guard: editing exactly one leaf module's text content
/// between two builds in the same project directory must miss only that
/// module's cache entry (every other module still hits), and the resulting
/// output must equal a from-scratch build of the same edited source,
/// byte-for-byte.
#[test]
fn persistent_cache_stale_guard_only_edited_module_misses() -> Result<()> {
    // Copy A: build once (populate the store), edit one leaf module's text
    // content only, rebuild.
    let temp_a = tempfile::tempdir().context("temp fixture A")?;
    let fixture_a = temp_a.path().join("mui-visual");
    copy_fixture(&fixture_a).context("copy mui-visual fixture A")?;

    clean_dist_dir(&fixture_a);
    let baseline = require_success(run_jet(&fixture_a, BUILD_ARGS)?, "baseline build (A)")?;
    let baseline_timing = parse_cache_timing_line(&String::from_utf8_lossy(&baseline.stderr))
        .ok_or_else(|| anyhow!("baseline build (A) did not print a cache timing line"))?;
    let total_modules = baseline_timing.hits + baseline_timing.misses;
    assert!(
        total_modules > 1,
        "expected the mui-visual fixture to transform more than one module; \
         got {total_modules}"
    );
    // #2140: the import-scan section's own module pool (scripts scanned for
    // imports) need not equal the transform section's pool count exactly
    // (e.g. a module can be import-scanned but later pruned before
    // transform), so this tracks its own total independently.
    let baseline_import_scan = parse_import_scan_timing_line(&String::from_utf8_lossy(
        &baseline.stderr,
    ))
    .ok_or_else(|| anyhow!("baseline build (A) did not print an import-scan timing line"))?;
    let total_import_scanned = baseline_import_scan.hits + baseline_import_scan.misses;
    assert!(
        total_import_scanned > 1,
        "expected the mui-visual fixture to import-scan more than one module; \
         got {total_import_scanned}"
    );

    edit_fixture_heading_text(&fixture_a).context("edit fixture A")?;

    clean_dist_dir(&fixture_a);
    let edited = require_success(run_jet(&fixture_a, BUILD_ARGS)?, "edited rebuild (A)")?;
    let edited_stderr = String::from_utf8_lossy(&edited.stderr).into_owned();
    let edited_timing = parse_cache_timing_line(&edited_stderr).ok_or_else(|| {
        anyhow!("edited rebuild (A) did not print a cache timing line\nstderr={edited_stderr}")
    })?;
    assert_eq!(
        edited_timing.misses, 1,
        "editing exactly one leaf module's text content must miss exactly that one \
         module's cache entry; stderr={edited_stderr}"
    );
    assert_eq!(
        edited_timing.hits,
        total_modules - 1,
        "every module except the edited one must still hit the persistent cache; \
         total_modules={total_modules}\nstderr={edited_stderr}"
    );
    // #2140: a pure JSX text-node edit changes only the edited module's own
    // content_hash, so exactly one import-scan entry (keyed on content_hash
    // + is_typescript, no path/dep component) must miss; every other
    // module's import-scan entry is untouched and must still hit.
    let edited_import_scan = parse_import_scan_timing_line(&edited_stderr).ok_or_else(|| {
        anyhow!(
            "edited rebuild (A) did not print an import-scan timing line\nstderr={edited_stderr}"
        )
    })?;
    assert_eq!(
        edited_import_scan.misses, 1,
        "editing exactly one leaf module's text content must miss exactly that \
         one module's import-scan cache entry; stderr={edited_stderr}"
    );
    assert_eq!(
        edited_import_scan.hits,
        total_import_scanned - 1,
        "every module except the edited one must still hit the persistent \
         import-scan cache; total_import_scanned={total_import_scanned}\n\
         stderr={edited_stderr}"
    );
    let edited_signature = dist_signature(&fixture_a).context("hash edited (A) dist output")?;

    // Copy B: fresh tempdir, same edit applied *before* the only build ->
    // a from-scratch baseline for the edited source with no cache
    // involvement (misses == total_modules).
    let temp_b = tempfile::tempdir().context("temp fixture B")?;
    let fixture_b = temp_b.path().join("mui-visual");
    copy_fixture(&fixture_b).context("copy mui-visual fixture B")?;
    edit_fixture_heading_text(&fixture_b).context("edit fixture B")?;

    clean_dist_dir(&fixture_b);
    let from_scratch = require_success(run_jet(&fixture_b, BUILD_ARGS)?, "from-scratch build (B)")?;
    let from_scratch_stderr = String::from_utf8_lossy(&from_scratch.stderr).into_owned();
    let from_scratch_timing = parse_cache_timing_line(&from_scratch_stderr).ok_or_else(|| {
        anyhow!(
            "from-scratch build (B) did not print a cache timing line\n\
             stderr={from_scratch_stderr}"
        )
    })?;
    assert_eq!(
        from_scratch_timing.hits, 0,
        "fresh tempdir must have no prior store to hit; stderr={from_scratch_stderr}"
    );
    assert_eq!(
        from_scratch_timing.misses, total_modules,
        "from-scratch build of the edited source must transform every module \
         (same module count as the unedited baseline); stderr={from_scratch_stderr}"
    );
    // #2140: same miss/scan invariant as the cold-build case above (not
    // hits == 0 — a fresh tempdir can still see a handful of intra-run
    // import-scan hits from same-content modules).
    let from_scratch_import_scan =
        parse_import_scan_timing_line(&from_scratch_stderr).ok_or_else(|| {
            anyhow!(
                "from-scratch build (B) did not print an import-scan timing line\n\
                 stderr={from_scratch_stderr}"
            )
        })?;
    assert_eq!(
        from_scratch_import_scan.misses,
        from_scratch_import_scan.fast + from_scratch_import_scan.fallback,
        "from-scratch build: every import-scan cache miss must correspond to \
         exactly one fast-path or fallback scan; stderr={from_scratch_stderr}"
    );
    let from_scratch_signature =
        dist_signature(&fixture_b).context("hash from-scratch (B) dist output")?;

    assert_eq!(
        edited_signature, from_scratch_signature,
        "a stale-guarded incremental rebuild (only the edited module misses) must \
         produce byte-identical output to a from-scratch build of the same edited source"
    );

    eprintln!(
        "[transform_cache] stale guard: total_modules={total_modules} edited misses={} \
         edited hits={} from-scratch misses={}; import-scan \
         total_import_scanned={total_import_scanned} edited misses={} edited hits={} \
         from-scratch misses={}",
        edited_timing.misses,
        edited_timing.hits,
        from_scratch_timing.misses,
        edited_import_scan.misses,
        edited_import_scan.hits,
        from_scratch_import_scan.misses,
    );
    Ok(())
}

/// AC4 — config change: changing a `--define` between two builds in the
/// same directory must invalidate the entire store via the config
/// fingerprint (a full miss, not a per-module comparison), while still
/// producing correct output.
#[test]
fn persistent_cache_config_change_forces_full_miss() -> Result<()> {
    let temp = tempfile::tempdir().context("temp fixture")?;
    let fixture = temp.path().join("mui-visual");
    copy_fixture(&fixture).context("copy mui-visual fixture")?;

    clean_dist_dir(&fixture);
    let first = require_success(
        run_jet(
            &fixture,
            [
                "build",
                "--sourcemap",
                "none",
                "--define",
                "CACHE_PROBE=\"a\"",
            ],
        )?,
        "build with --define CACHE_PROBE=a",
    )?;
    let first_timing = parse_cache_timing_line(&String::from_utf8_lossy(&first.stderr))
        .ok_or_else(|| anyhow!("first build did not print a cache timing line"))?;
    let total_modules = first_timing.hits + first_timing.misses;
    assert!(
        total_modules > 1,
        "expected the mui-visual fixture to transform more than one module; \
         got {total_modules}"
    );
    // #2141: same "own pool, tracked independently" reasoning as the
    // import-scan section (see `persistent_cache_stale_guard_only_edited_
    // module_misses`'s comment) — the analysis section's module count need
    // not equal the transform section's exactly.
    let total_analyzed = first_timing.a_hits + first_timing.a_misses;
    assert!(
        total_analyzed > 1,
        "expected the mui-visual fixture to analyze more than one module; \
         got {total_analyzed}"
    );

    clean_dist_dir(&fixture);
    let second = require_success(
        run_jet(
            &fixture,
            [
                "build",
                "--sourcemap",
                "none",
                "--define",
                "CACHE_PROBE=\"b\"",
            ],
        )?,
        "build with --define CACHE_PROBE=b",
    )?;
    let second_stderr = String::from_utf8_lossy(&second.stderr).into_owned();
    let second_timing = parse_cache_timing_line(&second_stderr).ok_or_else(|| {
        anyhow!("second build did not print a cache timing line\nstderr={second_stderr}")
    })?;

    assert_eq!(
        second_timing.hits, 0,
        "changing a --define must invalidate the entire store via the config \
         fingerprint (no per-module comparison should ever hit); stderr={second_stderr}"
    );
    assert_eq!(
        second_timing.misses, total_modules,
        "changing a --define must re-transform every module; stderr={second_stderr}"
    );
    assert!(
        fixture.join("dist/index.html").exists(),
        "build after a config change must still produce correct output (dist/index.html)"
    );
    // #2141: `analysis_fingerprint` is derived from `defines` (see its doc
    // comment in `persistent_cache.rs`), so a --define change must discard
    // every analysis entry *loaded from disk* — but, same as the cold-run
    // reasoning in `persistent_cache_determinism_cold_vs_warm_rebuild`, the
    // section also serves as intra-run memoization for modules that happen
    // to share byte-identical content, and defines don't change source
    // content or module-graph traversal order — so the second build must
    // reproduce *exactly* the first build's own intra-run collision count,
    // not zero: any hit count above that would mean a stale disk entry
    // leaked through the fingerprint gate.
    assert_eq!(
        second_timing.a_hits, first_timing.a_hits,
        "changing a --define must invalidate every disk-loaded analysis \
         entry, leaving only the first build's own intra-run collision \
         hits (first a_hits={}); stderr={second_stderr}",
        first_timing.a_hits
    );
    assert_eq!(
        second_timing.a_misses, first_timing.a_misses,
        "changing a --define must re-analyze every module the first build \
         analyzed fresh (first a_misses={}); stderr={second_stderr}",
        first_timing.a_misses
    );
    // #2141: `resolver_config_fingerprint` deliberately excludes `defines`
    // (aliases/baseUrl/conditions/externalize flags only — see its doc
    // comment) — the resolution section must be completely unaffected by
    // this change, the opposite reconciliation from the analysis section
    // just above, proving the two #2141 sections are gated independently
    // of each other end-to-end (not just at the unit level).
    assert_eq!(
        second_timing.r_misses, 0,
        "changing a --define must not invalidate the resolution section; \
         stderr={second_stderr}"
    );
    assert_eq!(
        second_timing.r_hits,
        first_timing.r_hits + first_timing.r_misses,
        "changing a --define must still resolution-cache-hit every bare \
         specifier resolution the first build made (fresh resolutions + \
         intra-run dedup hits); first r_hits={} r_misses={} \
         second r_hits={}\nstderr={second_stderr}",
        first_timing.r_hits,
        first_timing.r_misses,
        second_timing.r_hits
    );

    eprintln!(
        "[transform_cache] config change: total_modules={total_modules} \
         total_analyzed={total_analyzed} first hits={} misses={} r_hits={} r_misses={} \
         a_hits={} a_misses={}; second (changed define) hits={} misses={} r_hits={} \
         r_misses={} a_hits={} a_misses={}",
        first_timing.hits,
        first_timing.misses,
        first_timing.r_hits,
        first_timing.r_misses,
        first_timing.a_hits,
        first_timing.a_misses,
        second_timing.hits,
        second_timing.misses,
        second_timing.r_hits,
        second_timing.r_misses,
        second_timing.a_hits,
        second_timing.a_misses,
    );
    Ok(())
}

/// #2141 — resolution guard: editing the content of ONE package.json a
/// real node_modules-scoped resolution actually consulted must miss at
/// least that resolution (and any other resolution whose guard included
/// it), leave the transform/analysis sections untouched (no source module
/// content changed), and still produce byte-identical output to the
/// pre-edit baseline (the edit adds an unused field only). The
/// `resolution_guard_*` unit tests in `persistent_cache.rs` cover the
/// guard mechanism directly against synthetic key/value pairs; this proves
/// the same property end-to-end against a real symlinked node_modules
/// layout.
#[test]
fn persistent_cache_resolution_guard_package_json_change_forces_resolution_miss() -> Result<()> {
    let temp = tempfile::tempdir().context("temp fixture")?;
    let fixture = temp.path().join("mui-visual");
    copy_fixture(&fixture).context("copy mui-visual fixture")?;

    // `react` is imported directly (`main.tsx`, `MuiVisualFixture.tsx`) as
    // a bare top-level specifier, guaranteeing its package.json is
    // consulted by a real node_modules-scoped resolution. Materialized to
    // a private copy first so the guard-busting edit below can never write
    // through the symlink into the shared, machine-wide `~/.jet-store`.
    materialize_node_modules_package(&fixture, "react")
        .context("materialize node_modules/react")?;

    clean_dist_dir(&fixture);
    let baseline = require_success(run_jet(&fixture, BUILD_ARGS)?, "baseline build")?;
    let baseline_stderr = String::from_utf8_lossy(&baseline.stderr).into_owned();
    let baseline_timing = parse_cache_timing_line(&baseline_stderr).ok_or_else(|| {
        anyhow!("baseline build did not print a cache timing line\nstderr={baseline_stderr}")
    })?;
    assert!(
        baseline_timing.r_misses > 0,
        "expected the mui-visual fixture's node_modules imports to populate \
         the resolution cache; stderr={baseline_stderr}"
    );
    let baseline_signature = dist_signature(&fixture).context("hash baseline dist output")?;

    let package_json = fixture.join("node_modules/react/package.json");
    let original = fs::read_to_string(&package_json)
        .with_context(|| format!("read {}", package_json.display()))?;
    let trimmed = original.trim_end();
    assert!(
        trimmed.ends_with('}'),
        "expected react's package.json to end with a closing brace"
    );
    let mutated = format!(
        "{}{}",
        &trimmed[..trimmed.len() - 1],
        ",\"_jetCacheGuardTest\":true}\n"
    );
    fs::write(&package_json, mutated)
        .with_context(|| format!("write {}", package_json.display()))?;

    clean_dist_dir(&fixture);
    let rebuild = require_success(
        run_jet(&fixture, BUILD_ARGS)?,
        "rebuild after package.json edit",
    )?;
    let rebuild_stderr = String::from_utf8_lossy(&rebuild.stderr).into_owned();
    let rebuild_timing = parse_cache_timing_line(&rebuild_stderr).ok_or_else(|| {
        anyhow!(
            "rebuild after package.json edit did not print a cache timing line\n\
             stderr={rebuild_stderr}"
        )
    })?;
    assert!(
        rebuild_timing.r_misses > 0,
        "editing a consulted package.json must miss at least the \
         resolution(s) whose guard included it; stderr={rebuild_stderr}"
    );
    // The transform/analysis sections are untouched by this edit (no
    // source module content or defines changed) — only the resolution
    // section's guard should react.
    assert_eq!(
        rebuild_timing.misses, 0,
        "a package.json content change must not invalidate the transform \
         section; stderr={rebuild_stderr}"
    );
    assert_eq!(
        rebuild_timing.hits,
        baseline_timing.hits + baseline_timing.misses,
        "every module's transform-cache entry must still hit after a \
         package.json-only edit; stderr={rebuild_stderr}"
    );
    assert_eq!(
        rebuild_timing.a_misses, 0,
        "a package.json content change must not invalidate the analysis \
         section; stderr={rebuild_stderr}"
    );

    let rebuild_signature = dist_signature(&fixture).context("hash rebuilt dist output")?;
    assert_eq!(
        baseline_signature, rebuild_signature,
        "a resolution-guard-busted rebuild must still produce byte-identical \
         dist/ output to the original baseline (the package.json edit added \
         an unused field only, no behavioral change)"
    );
    assert!(
        fixture.join("dist/index.html").exists(),
        "rebuild after a package.json content change must still produce \
         correct output (dist/index.html)"
    );

    eprintln!(
        "[transform_cache] resolution guard: baseline r_hits={} r_misses={}; \
         rebuild r_hits={} r_misses={} misses={} a_misses={}",
        baseline_timing.r_hits,
        baseline_timing.r_misses,
        rebuild_timing.r_hits,
        rebuild_timing.r_misses,
        rebuild_timing.misses,
        rebuild_timing.a_misses,
    );
    Ok(())
}

/// #2143 — AC1/AC2 core positive path: an unchanged rebuild (same sources,
/// same `dist/` left in place, no `clean_dist_dir`) must be replayed
/// (skip the real build entirely) rather than a full build, must leave
/// `dist/` byte-identical to the build it verified against, and that
/// replayed `dist/` must itself be byte-identical to a `JET_NO_REPLAY=1`
/// forced full build of the exact same unchanged sources on a separate
/// tempdir (AC2).
#[test]
fn persistent_cache_replay_unchanged_rebuild_is_replayed_and_byte_identical() -> Result<()> {
    let temp = tempfile::tempdir().context("temp fixture")?;
    let fixture = temp.path().join("mui-visual");
    copy_fixture(&fixture).context("copy mui-visual fixture")?;

    clean_dist_dir(&fixture);
    let baseline = require_success(run_jet(&fixture, BUILD_ARGS)?, "baseline build")?;
    let baseline_stderr = String::from_utf8_lossy(&baseline.stderr).into_owned();
    let baseline_replay = parse_replay_timing_line(&baseline_stderr).ok_or_else(|| {
        anyhow!("baseline build did not print a replay timing line\nstderr={baseline_stderr}")
    })?;
    assert_eq!(
        baseline_replay.outcome, "full-build(no-manifest)",
        "the very first build (no prior store) must always fall back to a \
         full build; stderr={baseline_stderr}"
    );
    let baseline_signature = dist_signature(&fixture).context("hash baseline dist output")?;

    // Deliberately no `clean_dist_dir` here: a still-valid `dist/` left
    // untouched by the previous build is exactly what a correct replay
    // must recognize and leave alone.
    let replayed = require_success(run_jet(&fixture, BUILD_ARGS)?, "unchanged rebuild")?;
    let replayed_stdout = String::from_utf8_lossy(&replayed.stdout).into_owned();
    let replayed_stderr = String::from_utf8_lossy(&replayed.stderr).into_owned();
    let replayed_timing = parse_replay_timing_line(&replayed_stderr).ok_or_else(|| {
        anyhow!("unchanged rebuild did not print a replay timing line\nstderr={replayed_stderr}")
    })?;
    assert_eq!(
        replayed_timing.outcome, "replayed",
        "an unchanged rebuild must be replayed, not a full build; stderr={replayed_stderr}"
    );
    assert_eq!(
        replayed_timing.hash_fallback, 0,
        "an unchanged rebuild (no mtime drift on any tracked file) must \
         verify entirely off the stat screen, with no content-hash \
         fallback; stderr={replayed_stderr}"
    );
    assert!(
        replayed_stdout.contains("[replayed]"),
        "a replayed build's `Build complete in ...` line must carry the \
         `[replayed]` marker; stdout={replayed_stdout}"
    );
    let replayed_signature = dist_signature(&fixture).context("hash replayed dist output")?;
    assert_eq!(
        baseline_signature, replayed_signature,
        "a replayed build must leave dist/ byte-identical to the baseline \
         build it verified against"
    );

    // AC2: a fresh tempdir copy of the exact same unchanged sources, forced
    // through a full build via JET_NO_REPLAY=1, must produce byte-identical
    // dist/ output to the replayed run above.
    let temp_forced = tempfile::tempdir().context("temp fixture (forced full build)")?;
    let fixture_forced = temp_forced.path().join("mui-visual");
    copy_fixture(&fixture_forced).context("copy mui-visual fixture (forced full build)")?;
    clean_dist_dir(&fixture_forced);
    require_success(
        run_jet_no_replay(&fixture_forced, BUILD_ARGS)?,
        "forced full build (JET_NO_REPLAY=1)",
    )?;
    let forced_signature =
        dist_signature(&fixture_forced).context("hash forced-full-build dist output")?;
    assert_eq!(
        replayed_signature, forced_signature,
        "a replayed build's dist/ must be byte-identical to a \
         JET_NO_REPLAY=1 forced full build of the same unchanged sources \
         (AC2)"
    );

    eprintln!(
        "[transform_cache] replay unchanged: baseline outcome={} replayed \
         outcome={} verified={} stat_ms={:.2} hash_fallback={} dist files={}",
        baseline_replay.outcome,
        replayed_timing.outcome,
        replayed_timing.verified,
        replayed_timing.stat_ms,
        replayed_timing.hash_fallback,
        replayed_signature.len(),
    );
    Ok(())
}

/// #2143 — a bumped mtime with byte-for-byte unchanged content must still
/// replay: the cheap `(mtime, size)` stat screen drifts (forcing the
/// content-hash fallback for that one file), but the fallback hash must
/// still match, so the overall outcome is still a replay.
#[test]
fn persistent_cache_replay_mtime_bump_same_content_still_replays_via_hash_fallback() -> Result<()> {
    let temp = tempfile::tempdir().context("temp fixture")?;
    let fixture = temp.path().join("mui-visual");
    copy_fixture(&fixture).context("copy mui-visual fixture")?;

    clean_dist_dir(&fixture);
    require_success(run_jet(&fixture, BUILD_ARGS)?, "baseline build")?;
    let baseline_signature = dist_signature(&fixture).context("hash baseline dist output")?;

    bump_mtime(&fixture.join("src/MuiVisualFixture.tsx")).context("bump fixture mtime")?;

    // Deliberately no `clean_dist_dir`: same reasoning as the unchanged-
    // rebuild test above — a correct replay must leave dist/ alone.
    let rebuild = require_success(run_jet(&fixture, BUILD_ARGS)?, "mtime-bumped rebuild")?;
    let rebuild_stderr = String::from_utf8_lossy(&rebuild.stderr).into_owned();
    let rebuild_timing = parse_replay_timing_line(&rebuild_stderr).ok_or_else(|| {
        anyhow!(
            "mtime-bumped rebuild did not print a replay timing line\n\
             stderr={rebuild_stderr}"
        )
    })?;
    assert_eq!(
        rebuild_timing.outcome, "replayed",
        "a bumped mtime with byte-for-byte unchanged content must still \
         replay (the content-hash fallback must confirm the file is \
         actually unchanged); stderr={rebuild_stderr}"
    );
    assert!(
        rebuild_timing.hash_fallback >= 1,
        "a bumped mtime must trigger at least one content-hash fallback \
         (the stat screen alone must not have been trusted blindly); \
         stderr={rebuild_stderr}"
    );
    let rebuild_signature = dist_signature(&fixture).context("hash rebuilt dist output")?;
    assert_eq!(
        baseline_signature, rebuild_signature,
        "a replayed mtime-bump rebuild must leave dist/ byte-identical to \
         the baseline"
    );

    eprintln!(
        "[transform_cache] replay mtime bump: outcome={} verified={} hash_fallback={}",
        rebuild_timing.outcome, rebuild_timing.verified, rebuild_timing.hash_fallback,
    );
    Ok(())
}

/// #2143 — a single-byte (or any) content edit to a tracked input must
/// force a full build with a `content-changed:<path>` reason, distinct
/// from `persistent_cache_stale_guard_only_edited_module_misses` above
/// (which only checks the transform section's own hit/miss counters, never
/// the replay line this test asserts on directly).
#[test]
fn persistent_cache_replay_single_byte_edit_forces_full_build() -> Result<()> {
    let temp = tempfile::tempdir().context("temp fixture")?;
    let fixture = temp.path().join("mui-visual");
    copy_fixture(&fixture).context("copy mui-visual fixture")?;

    clean_dist_dir(&fixture);
    require_success(run_jet(&fixture, BUILD_ARGS)?, "baseline build")?;

    edit_fixture_heading_text(&fixture).context("edit fixture")?;

    clean_dist_dir(&fixture);
    let rebuild = require_success(run_jet(&fixture, BUILD_ARGS)?, "edited rebuild")?;
    let rebuild_stderr = String::from_utf8_lossy(&rebuild.stderr).into_owned();
    let rebuild_timing = parse_replay_timing_line(&rebuild_stderr).ok_or_else(|| {
        anyhow!("edited rebuild did not print a replay timing line\nstderr={rebuild_stderr}")
    })?;
    assert!(
        rebuild_timing
            .outcome
            .starts_with("full-build(content-changed:"),
        "a single content edit must force a full build with a \
         content-changed reason; stderr={rebuild_stderr}"
    );
    assert!(
        fixture.join("dist/index.html").exists(),
        "a declined-replay rebuild must still run the normal build path \
         and produce correct output"
    );

    eprintln!(
        "[transform_cache] replay single-byte edit: outcome={}",
        rebuild_timing.outcome,
    );
    Ok(())
}

/// #2143 — changing a `--define` must force a full build via the replay
/// config fingerprint's own `config-changed` reason, the same invalidation
/// `persistent_cache_config_change_forces_full_miss` above already proves
/// for the transform section specifically — this proves the replay
/// section reacts too, gated by `replay_config_fingerprint` rather than
/// `config_fingerprint` alone.
#[test]
fn persistent_cache_replay_defines_change_forces_full_build() -> Result<()> {
    let temp = tempfile::tempdir().context("temp fixture")?;
    let fixture = temp.path().join("mui-visual");
    copy_fixture(&fixture).context("copy mui-visual fixture")?;

    clean_dist_dir(&fixture);
    require_success(
        run_jet(
            &fixture,
            [
                "build",
                "--sourcemap",
                "none",
                "--define",
                "CACHE_PROBE=\"a\"",
            ],
        )?,
        "baseline build with --define CACHE_PROBE=a",
    )?;

    clean_dist_dir(&fixture);
    let rebuild = require_success(
        run_jet(
            &fixture,
            [
                "build",
                "--sourcemap",
                "none",
                "--define",
                "CACHE_PROBE=\"b\"",
            ],
        )?,
        "rebuild with --define CACHE_PROBE=b",
    )?;
    let rebuild_stderr = String::from_utf8_lossy(&rebuild.stderr).into_owned();
    let rebuild_timing = parse_replay_timing_line(&rebuild_stderr).ok_or_else(|| {
        anyhow!("rebuild did not print a replay timing line\nstderr={rebuild_stderr}")
    })?;
    assert_eq!(
        rebuild_timing.outcome, "full-build(config-changed)",
        "changing a --define must force a full build via the replay \
         config fingerprint; stderr={rebuild_stderr}"
    );
    assert!(
        fixture.join("dist/index.html").exists(),
        "a declined-replay rebuild must still run the normal build path \
         and produce correct output"
    );

    eprintln!(
        "[transform_cache] replay defines change: outcome={}",
        rebuild_timing.outcome,
    );
    Ok(())
}

/// #2143 — deleting exactly one previously recorded output (everything
/// else in `dist/` left in place) must force a full build with a
/// `missing-output:<rel_path>` reason, and that full build must correctly
/// re-emit the missing file with output byte-identical to the original
/// baseline.
#[test]
fn persistent_cache_replay_deleted_output_forces_full_build_and_reemits() -> Result<()> {
    let temp = tempfile::tempdir().context("temp fixture")?;
    let fixture = temp.path().join("mui-visual");
    copy_fixture(&fixture).context("copy mui-visual fixture")?;

    clean_dist_dir(&fixture);
    require_success(run_jet(&fixture, BUILD_ARGS)?, "baseline build")?;
    let baseline_signature = dist_signature(&fixture).context("hash baseline dist output")?;

    let index_html = fixture.join("dist/index.html");
    assert!(
        index_html.exists(),
        "expected dist/index.html after baseline build"
    );
    fs::remove_file(&index_html).with_context(|| format!("remove {}", index_html.display()))?;

    // Deliberately no `clean_dist_dir`: only ONE recorded output should be
    // missing (everything else in dist/ is untouched) — try_replay must
    // notice the gap and decline, then the ordinary (non-replay) build path
    // re-emits the missing file without needing dist/ cleared first.
    let rebuild = require_success(
        run_jet(&fixture, BUILD_ARGS)?,
        "rebuild after deleting an output",
    )?;
    let rebuild_stderr = String::from_utf8_lossy(&rebuild.stderr).into_owned();
    let rebuild_timing = parse_replay_timing_line(&rebuild_stderr).ok_or_else(|| {
        anyhow!(
            "rebuild after deleting an output did not print a replay \
             timing line\nstderr={rebuild_stderr}"
        )
    })?;
    assert!(
        rebuild_timing
            .outcome
            .starts_with("full-build(missing-output:"),
        "deleting one recorded output must force a full build with a \
         missing-output reason; stderr={rebuild_stderr}"
    );
    assert!(
        index_html.exists(),
        "a full build after a deleted output must re-emit dist/index.html"
    );
    let rebuild_signature = dist_signature(&fixture).context("hash re-emitted dist output")?;
    assert_eq!(
        baseline_signature, rebuild_signature,
        "the re-emitted output after a declined replay must be \
         byte-identical to the original baseline"
    );

    eprintln!(
        "[transform_cache] replay deleted output: outcome={}",
        rebuild_timing.outcome,
    );
    Ok(())
}

/// #2143 — resolution-shadow guard: `main.tsx` imports `./MuiVisualFixture`
/// extensionlessly, and jet's resolver tries `.ts` before `.tsx`
/// (`ResolveOptions::default()`'s `extensions` order) — so a NEW sibling
/// `MuiVisualFixture.ts` genuinely shadows the pre-existing `.tsx` file the
/// baseline build actually resolved and bundled. No already-consumed
/// file's own content hash changes (only the directory's *listing* does),
/// which is exactly the gap the source-dir listing fingerprint guard
/// exists to close: a full build must be forced, and it must actually
/// rebundle against the new file (proven both by the replay decline reason
/// and by the newly emitted entry no longer containing the original
/// fixture's distinctive heading text).
#[test]
fn persistent_cache_replay_resolution_shadow_guard_forces_full_build() -> Result<()> {
    let temp = tempfile::tempdir().context("temp fixture")?;
    let fixture = temp.path().join("mui-visual");
    copy_fixture(&fixture).context("copy mui-visual fixture")?;

    clean_dist_dir(&fixture);
    require_success(run_jet(&fixture, BUILD_ARGS)?, "baseline build")?;
    let baseline_signature = dist_signature(&fixture).context("hash baseline dist output")?;

    let shadow_path = fixture.join("src/MuiVisualFixture.ts");
    assert!(
        !shadow_path.exists(),
        "fixture must not already have a MuiVisualFixture.ts sibling, or \
         this test is not exercising a real shadow"
    );
    fs::write(
        &shadow_path,
        "export const MuiVisualFixture = () => null;\n",
    )
    .with_context(|| format!("write {}", shadow_path.display()))?;

    // Deliberately no `clean_dist_dir`: nothing already-consumed changed,
    // so this must exercise the decline path via the directory-listing
    // guard specifically, not any of the input/output checks.
    let rebuild = require_success(
        run_jet(&fixture, BUILD_ARGS)?,
        "rebuild after adding a shadow file",
    )?;
    let rebuild_stderr = String::from_utf8_lossy(&rebuild.stderr).into_owned();
    let rebuild_timing = parse_replay_timing_line(&rebuild_stderr).ok_or_else(|| {
        anyhow!(
            "rebuild after adding a shadow file did not print a replay \
             timing line\nstderr={rebuild_stderr}"
        )
    })?;
    assert!(
        rebuild_timing
            .outcome
            .starts_with("full-build(dir-changed:"),
        "a new same-basename, earlier-priority-extension sibling file must \
         force a full build via the resolution-shadow directory-listing \
         guard; stderr={rebuild_stderr}"
    );
    assert!(
        fixture.join("dist/index.html").exists(),
        "rebuild after a shadow file addition must still produce correct \
         output"
    );
    let rebuild_signature = dist_signature(&fixture).context("hash rebuilt dist output")?;
    assert_ne!(
        baseline_signature, rebuild_signature,
        "the shadow file must actually change bundled output, or this \
         test is not exercising a real shadow"
    );

    let new_entry_name = rebuild_signature
        .keys()
        .find(|name| {
            name.starts_with("main.")
                && name.ends_with(".js")
                && !baseline_signature.contains_key(*name)
        })
        .ok_or_else(|| {
            anyhow!(
                "expected the shadow-file rebuild to emit a new, \
                 differently-hashed entry bundle; baseline={baseline_signature:?} \
                 rebuild={rebuild_signature:?}"
            )
        })?;
    let new_entry_bytes = fs::read_to_string(fixture.join("dist").join(new_entry_name))
        .with_context(|| format!("read dist/{new_entry_name}"))?;
    assert!(
        !new_entry_bytes.contains("MUI visual table fixture"),
        "the shadow MuiVisualFixture.ts stub must have won resolution over \
         the original .tsx (its distinctive heading text must be absent \
         from the newly emitted entry bundle)"
    );

    eprintln!(
        "[transform_cache] replay resolution-shadow guard: outcome={}",
        rebuild_timing.outcome,
    );
    Ok(())
}
// HANDWRITE-END

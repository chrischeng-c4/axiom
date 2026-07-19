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
}

/// Parse the `[bundle-timing] cache: hits=N misses=M loaded_in=Xms
/// saved_in=Yms bytes=Z` line `jet build` emits to stderr under
/// `JET_BUNDLE_TIMING=1` (see `Bundler::bundle` in `src/bundler/mod.rs`).
fn parse_cache_timing_line(stderr: &str) -> Option<CacheTiming> {
    let line = stderr
        .lines()
        .find(|line| line.starts_with("[bundle-timing] cache:"))?;
    let mut hits = None;
    let mut misses = None;
    for field in line.split_whitespace() {
        if let Some(v) = field.strip_prefix("hits=") {
            hits = v.parse::<u64>().ok();
        } else if let Some(v) = field.strip_prefix("misses=") {
            misses = v.parse::<u64>().ok();
        }
    }
    Some(CacheTiming {
        hits: hits?,
        misses: misses?,
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
    let warm_signature = dist_signature(&fixture).context("hash warm dist output")?;

    assert_eq!(
        cold_signature, warm_signature,
        "a cache-hit rebuild must produce byte-identical dist/ output to the \
         cache-deleted (cold) build"
    );

    eprintln!(
        "[transform_cache] determinism: cold hits={} misses={}; warm hits={} misses={}; \
         dist files={}",
        cold_timing.hits,
        cold_timing.misses,
        warm_timing.hits,
        warm_timing.misses,
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
    let from_scratch_signature =
        dist_signature(&fixture_b).context("hash from-scratch (B) dist output")?;

    assert_eq!(
        edited_signature, from_scratch_signature,
        "a stale-guarded incremental rebuild (only the edited module misses) must \
         produce byte-identical output to a from-scratch build of the same edited source"
    );

    eprintln!(
        "[transform_cache] stale guard: total_modules={total_modules} edited misses={} \
         edited hits={} from-scratch misses={}",
        edited_timing.misses, edited_timing.hits, from_scratch_timing.misses,
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

    eprintln!(
        "[transform_cache] config change: total_modules={total_modules} \
         first hits={} misses={}; second (changed define) hits={} misses={}",
        first_timing.hits, first_timing.misses, second_timing.hits, second_timing.misses,
    );
    Ok(())
}
// HANDWRITE-END

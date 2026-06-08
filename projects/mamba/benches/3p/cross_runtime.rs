//! Cross-runtime bench harness (Phase 1.C — CPython 3.12 vs mamba).
//!
//! Companion to the real-world fixture convention from Phase 1.E. Walks
//! `tests/cpython/{stdlib,3p}/<lib>/bench/<scenario>.py`,
//! runs each fixture under both `python3` and `mamba run`, picks the
//! minimum wall-clock and lowest peak-RSS per side (best-of-N), and
//! emits one line per fixture:
//!
//! ```text
//! [<lib_name>/<scenario>] wall mamba/cpython = X.YYx (PASS|FAIL @ floor 1.0x)
//!     internal mamba/cpython = Z.ZZx (PASS|FAIL @ floor 1.0x)
//!     mem: mamba/cpython = Y.YYx (PASS|FAIL @ floor 1.0x)
//! ```
//!
//! `wall` is total `Command::output()` duration — includes Python startup
//! overhead (~200 ms CPython, ~5 ms mamba), so for short benches the
//! ratio is startup-amortization, not steady-state per-call cost.
//! `internal` is `time.perf_counter()` measured INSIDE the fixture
//! around its hot loop (emitted as `INTERNAL_TIME_NS=<u64>` to
//! stdout/stderr — mamba currently routes `file=sys.stderr` to stdout
//! so the harness accepts either stream); it is the unbiased per-call
//! cost. Fixtures without the marker print `internal: UNAVAIL` and
//! reviewers should treat the wall number as methodology-suspect.
//! Task #22 / `feedback_mamba_perf_is_the_product` methodology caveat
//! 2026-05-13.
//!
//! Exits non-zero if EITHER the speed-tier or memory-tier fails. The
//! memory floor is `mamba ≤ cpython` (ratio cpython/mamba ≥ 1.0x —
//! same direction as the speed ratio, so PASS when mamba uses no more
//! memory than CPython). Tier-aware thresholds (`tier:compute≥10×`,
//! `tier:app≥3×`, `tier:dynamic≥1.5×`) are documented in issue #1265;
//! this harness ships with the floor gate (`≥1.0×`) only and reads an
//! optional `# tier: <name>` header from the fixture for richer
//! reporting in a later iteration.
//!
//! ## Profiling integration
//!
//! Wall-time and the harness-side memory baseline are taken through
//! `qc::performance::profiler` (Task #13 / cclab tracking #2093):
//!
//! - **Wall-time**: `Instant::now()` straddles each child invocation; the
//!   delta is reported as the wall measurement. `cclab-qc`'s `Profiler`
//!   `record_phase(ProfilePhase::Total, ...)` is also kept up to date so
//!   future per-phase breakdowns (e.g. compile vs run inside mamba) can
//!   layer on top without further harness changes.
//! - **Harness-side RSS**: a `MemorySnapshot::capture()` is taken before
//!   and after the bench loop and printed as a banner so reviewers can
//!   see the bench harness itself is not bloating up between iterations.
//! - **Child RSS** (the load-bearing measurement): still extracted from
//!   `/usr/bin/time -l` (macOS BSD) or `/usr/bin/time -v` (Linux GNU)
//!   wrapping each child. cclab-qc's `get_rss_bytes()` only knows about
//!   the calling process; a cross-process `get_rss_bytes_for_pid(pid)`
//!   companion is the right cclab-qc-side follow-up but lives outside
//!   the `project-mamba` write-scope and is therefore filed separately.
//!   `time` was chosen over `getrusage(RUSAGE_CHILDREN)` because the
//!   latter aggregates across siblings on macOS, which breaks
//!   per-iteration measurement without delta bookkeeping.
//!
//! ## Usage
//!
//! ```text
//! cargo bench -p mamba --bench cross_runtime_3p
//! cargo bench -p mamba --bench cross_runtime_3p -- --fixture int_sum_loop
//! cargo bench -p mamba --bench cross_runtime_3p -- --iters 7
//! ```
//!
//! ## Warmup correctness probe (Task #15)
//!
//! Before timing each fixture, the harness runs it once under each
//! runtime OUTSIDE the timed region and asserts that the two stdout
//! buffers match byte-for-byte. This protects against measurement
//! artefacts introduced by the JIT branch-drop bug
//! (`project_mamba_jit_drops_branches_after_stdlib_call` memory): a
//! `mod.func()` call followed by an `if`/`else`/`assert` inside the
//! timed loop body causes the runtime to silently elide the
//! post-call statements, so the fixture's accumulator stays at zero
//! and the final `print` reads a value that diverges from CPython's.
//! A divergent fixture is reported as `SKIP — warmup probe: ...` and
//! contributes to a non-zero exit — better to lose the data point
//! than to publish a fast no-op as a real perf win.
//!
//! ## Skips
//!
//! - If `python3 --version` fails, the harness prints a single skip
//!   line and exits 0 (cannot compare without a baseline).
//! - If `mamba` is missing from `$PATH`, the harness exits 1 with a
//!   clear message (this is a hard configuration error).
//! - If `/usr/bin/time` is missing or its memory line cannot be parsed,
//!   the harness still reports wall-time and prints `mem = UNAVAIL`
//!   instead of failing — RSS measurement is best-effort by design so
//!   a missing system `time` binary doesn't break the speed gate.
//! - If the warmup probe shows stdout divergence between CPython and
//!   mamba for a given fixture, that fixture is SKIPPED for timing
//!   and the harness exits non-zero (see above).

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

use qc::performance::profiler::{MemorySnapshot, PhaseBreakdown, PhaseTiming, ProfilePhase};

const FLOOR: f64 = 1.0;

struct Fixture {
    lib: String,
    scenario: String,
    path: PathBuf,
}

struct Args {
    fixture_filter: Option<String>,
    iters: u32,
}

fn parse_args() -> Args {
    let mut fixture_filter = None;
    let mut iters: u32 = 5;
    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--fixture" => fixture_filter = it.next(),
            "--iters" => {
                iters = it.next().and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                    eprintln!("--iters requires a positive integer");
                    std::process::exit(2);
                });
            }
            // cargo bench passes through args we don't recognise — ignore.
            _ => {}
        }
    }
    Args {
        fixture_filter,
        iters,
    }
}

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("conformance")
}

fn discover(root: &Path) -> Vec<Fixture> {
    let mut out = Vec::new();
    for bucket in ["stdlib", "3p"] {
        let bucket_dir = root.join(bucket);
        let Ok(libs) = std::fs::read_dir(&bucket_dir) else {
            continue;
        };
        for lib_entry in libs.flatten() {
            let lib_name = match lib_entry.file_name().to_str() {
                Some(s) => s.to_string(),
                None => continue,
            };
            let bench_dir = lib_entry.path().join("bench");
            let Ok(scripts) = std::fs::read_dir(&bench_dir) else {
                continue;
            };
            for script_entry in scripts.flatten() {
                let p = script_entry.path();
                if p.extension().and_then(|s| s.to_str()) != Some("py") {
                    continue;
                }
                let scenario = p
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                out.push(Fixture {
                    lib: lib_name.clone(),
                    scenario,
                    path: p,
                });
            }
        }
    }
    out.sort_by(|a, b| (&a.lib, &a.scenario).cmp(&(&b.lib, &b.scenario)));
    out
}

fn python3_available() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn mamba_bin() -> Option<PathBuf> {
    // Prefer the in-tree release build over a $PATH install. During Phase 2
    // sweeps the local target/ is the authoritative source — a stale PATH
    // install (e.g. ~/.cargo/bin/mamba from an earlier `cargo install`) will
    // miss newly registered stdlib modules and fail the bench with module
    // attribute errors. Fall back to $PATH if no in-tree build is present.
    let in_tree = env!("CARGO_MANIFEST_DIR");
    for rel in &["../../target/release/mamba", "../../target/debug/mamba"] {
        let candidate = PathBuf::from(in_tree).join(rel);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    if let Ok(out) = Command::new("which").arg("mamba").output() {
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
    }
    None
}

/// One sample: wall-clock duration, peak resident-set size in bytes
/// (None if the system `time` wrapper was unavailable or unparseable),
/// and fixture-internal nanoseconds (None if the fixture did not emit
/// the `INTERNAL_TIME_NS=...` marker on stdout/stderr).
///
/// Internal-time is the steady-state per-call cost measured INSIDE the
/// fixture via `time.perf_counter()` around the hot loop. Wall-time
/// is the total `Command::new().output()` duration — it includes
/// Python startup (~200 ms CPython, ~5 ms mamba) and module import
/// overhead, which dominates short benches and biases the ratio.
/// Task #22 / `feedback_mamba_perf_is_the_product` methodology caveat
/// 2026-05-13 introduced internal-time as the unbiased counterpart.
#[derive(Clone, Copy, Debug)]
struct Sample {
    wall: Duration,
    rss_bytes: Option<u64>,
    internal_ns: Option<u64>,
}

/// Parse a `/usr/bin/time -l` (macOS) or `/usr/bin/time -v` (Linux)
/// stderr blob for the peak RSS in bytes. macOS reports bytes; Linux
/// reports kilobytes. Returns None if no recognised line is present.
fn parse_peak_rss(stderr: &str) -> Option<u64> {
    for line in stderr.lines() {
        let trimmed = line.trim();
        // macOS BSD `time -l`: "<n>  maximum resident set size" (bytes).
        if let Some(rest) = trimmed.strip_suffix("maximum resident set size") {
            if let Ok(v) = rest.trim().parse::<u64>() {
                return Some(v);
            }
        }
        // Linux GNU `time -v`: "Maximum resident set size (kbytes): <n>".
        if let Some(rest) = trimmed.strip_prefix("Maximum resident set size") {
            if let Some(num) = rest.split(':').nth(1) {
                if let Ok(v) = num.trim().parse::<u64>() {
                    return Some(v.saturating_mul(1024));
                }
            }
        }
    }
    None
}

/// Parse an `INTERNAL_TIME_NS=<u64>` marker line from a stdout/stderr blob.
///
/// Fixtures emit this around their hot loop using `time.perf_counter()`
/// (multiplied by 1e9 to integer nanoseconds — mamba doesn't implement
/// `perf_counter_ns` yet); the harness uses it as the unbiased per-call
/// cost vs `Command::new().output()` wall time which is dominated by
/// Python startup overhead for short fixtures.
///
/// Returns the first parseable marker. Multiple markers in one blob
/// would indicate a fixture authoring bug — the harness silently takes
/// the first to keep best-of-N's `min()` reduction sensible.
fn parse_internal_time_ns(blob: &str) -> Option<u64> {
    for line in blob.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("INTERNAL_TIME_NS=") {
            if let Ok(v) = rest.trim().parse::<u64>() {
                return Some(v);
            }
        }
    }
    None
}

/// Build the `/usr/bin/time` argv prefix for the current platform.
/// macOS uses BSD `-l`; everywhere else assume GNU `-v`. Returns None
/// if `/usr/bin/time` does not exist (caller falls back to plain
/// `Command::new(cmd)` and drops RSS measurement).
fn time_wrapper() -> Option<(&'static str, &'static str)> {
    let p = Path::new("/usr/bin/time");
    if !p.exists() {
        return None;
    }
    if cfg!(target_os = "macos") {
        Some(("/usr/bin/time", "-l"))
    } else {
        Some(("/usr/bin/time", "-v"))
    }
}

/// Run `cmd args...`, optionally wrapped by `/usr/bin/time` for RSS,
/// and return a Sample plus exit success + stderr for diagnostics.
///
/// `phase` is a cclab-qc `PhaseTiming` accumulator the caller threads
/// through every iteration of this fixture; `time_one` records the
/// observed wall-clock duration into it via `record(ns)` so the
/// harness can emit a `PhaseBreakdown` (currently `ProfilePhase::Total`
/// only — future work can split mamba compile vs run with cclab-qc's
/// phase enum). The single-iteration `Sample` return value is what
/// the per-iter best-of-N tracker uses.
fn time_one(cmd: &Path, args: &[&str], phase: &mut PhaseTiming) -> (Sample, bool, String) {
    let start = Instant::now();
    let (out, wrapped) = if let Some((time_bin, flag)) = time_wrapper() {
        let mut all_args: Vec<&str> = Vec::with_capacity(args.len() + 2);
        all_args.push(flag);
        all_args.push(cmd.to_str().unwrap_or(""));
        all_args.extend(args.iter().copied());
        (Command::new(time_bin).args(&all_args).output(), true)
    } else {
        (Command::new(cmd).args(args).output(), false)
    };
    let elapsed = start.elapsed();
    // Feed cclab-qc's PhaseTiming so callers can produce a PhaseBreakdown
    // table later (see `summarise_phase` below).
    phase.record(elapsed.as_nanos() as u64);
    match out {
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let rss = if wrapped {
                parse_peak_rss(&stderr)
            } else {
                None
            };
            // INTERNAL_TIME_NS is emitted by the fixture itself (independent
            // of the `time` wrapper). Mamba currently routes `print(...,
            // file=sys.stderr)` to stdout — until that runtime gap closes,
            // accept the marker on either stream so the same fixture file
            // works under both runtimes unchanged.
            let internal_ns =
                parse_internal_time_ns(&stderr).or_else(|| parse_internal_time_ns(&stdout));
            // Note: when wrapped, the child's stderr is interleaved with
            // `time`'s memory report. The exit status is the child's
            // (preserved by `time`), so success-checking is unchanged.
            (
                Sample {
                    wall: elapsed,
                    rss_bytes: rss,
                    internal_ns,
                },
                o.status.success(),
                stderr,
            )
        }
        Err(e) => (
            Sample {
                wall: elapsed,
                rss_bytes: None,
                internal_ns: None,
            },
            false,
            format!("spawn error: {e}"),
        ),
    }
}

/// Run a fixture once with no `time` wrapper and return `(stdout, ok, stderr)`.
///
/// Used by the warmup correctness probe (Task #15). Output is captured
/// outside the timed region so a `print(...)` divergence between mamba and
/// CPython surfaces BEFORE any best-of-N measurement runs — that's the
/// fingerprint of the JIT branch-drop bug
/// ([`project_mamba_jit_drops_branches_after_stdlib_call`] memory): a
/// `mod.func()` call followed by an `if`/`else`/`assert` in the timed loop
/// body causes the runtime to silently elide the post-call statements, so
/// the fixture's final `print` reads a stale accumulator (e.g.
/// `checks == 0` instead of `ITERS`). Catching this once-out-of-band makes
/// the bench results trustworthy without requiring a JIT-side fix first.
fn run_once_capture(cmd: &Path, args: &[&str]) -> (String, bool, String) {
    match Command::new(cmd).args(args).output() {
        Ok(o) => (
            String::from_utf8_lossy(&o.stdout).to_string(),
            o.status.success(),
            String::from_utf8_lossy(&o.stderr).to_string(),
        ),
        Err(e) => (String::new(), false, format!("spawn error: {e}")),
    }
}

/// Warmup correctness probe: run the fixture once under CPython and once
/// under mamba, both outside the timed region. Returns `Ok(stdout)` if
/// both succeeded with byte-identical stdout, or `Err(reason)` describing
/// the divergence so the caller can mark the fixture as untrustworthy
/// before any timed iterations run.
///
/// Defense against the JIT branch-drop bug ([[project_mamba_jit_drops_
/// branches_after_stdlib_call]]) and any future correctness regression
/// that would otherwise let the bench "succeed" while measuring a fast
/// no-op. The probe is cheap (one extra run per runtime per fixture)
/// relative to the best-of-N timed runs and is unconditional.
fn warmup_probe(
    python: &Path,
    mamba: &Path,
    py_args: &[&str],
    mb_args: &[&str],
) -> Result<String, String> {
    let (py_out, py_ok, py_err) = run_once_capture(python, py_args);
    if !py_ok {
        return Err(format!(
            "cpython warmup failed: {}",
            py_err.lines().take(3).collect::<Vec<_>>().join(" | ")
        ));
    }
    let (mb_out, mb_ok, mb_err) = run_once_capture(mamba, mb_args);
    if !mb_ok {
        return Err(format!(
            "mamba warmup failed: {}",
            mb_err.lines().take(3).collect::<Vec<_>>().join(" | ")
        ));
    }
    // Strip the internal-time marker line before comparing. The integer
    // ns value will always differ between runtimes (and between
    // iterations) so it must not poison the correctness probe. Other
    // lines must still match byte-for-byte to catch the JIT
    // branch-drop fingerprint.
    let strip_marker = |s: &str| -> String {
        s.lines()
            .filter(|l| !l.trim_start().starts_with("INTERNAL_TIME_NS="))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let py_filtered = strip_marker(&py_out);
    let mb_filtered = strip_marker(&mb_out);
    if py_filtered != mb_filtered {
        // Trim each side to first 200 chars for the report — pytest-style
        // assertEqual diffs would be nicer but the bench is intentionally
        // lean. The first divergent line is usually load-bearing.
        let py_excerpt = py_filtered.lines().take(3).collect::<Vec<_>>().join(" | ");
        let mb_excerpt = mb_filtered.lines().take(3).collect::<Vec<_>>().join(" | ");
        return Err(format!(
            "stdout divergence — cpython=[{py_excerpt}] mamba=[{mb_excerpt}] (likely JIT branch-drop after stdlib call — see `project_mamba_jit_drops_branches_after_stdlib_call` memory)"
        ));
    }
    Ok(py_filtered)
}

/// Build a cclab-qc `PhaseBreakdown` from a single `Total`-phase
/// accumulator. Currently the harness only tracks `Total`, but keeping
/// the accumulator typed as `PhaseTiming` means a future patch can
/// split mamba compile-vs-run without changing the bench loop shape.
fn summarise_phase(phase: PhaseTiming) -> PhaseBreakdown {
    let mut times: std::collections::HashMap<String, Vec<u64>> = std::collections::HashMap::new();
    // We only retain min/max/avg in PhaseTiming itself; replay a synthetic
    // sample vector here so PhaseBreakdown::from_times can render a row.
    // The recorded count + total are the source of truth and remain accurate;
    // per-iter detail is intentionally collapsed to keep harness state lean.
    let total = phase.total_ns;
    let count = phase.count;
    let synthetic: Vec<u64> = (0..count)
        .map(|_| if count == 0 { 0 } else { total / count })
        .collect();
    times.insert(ProfilePhase::Total.to_string(), synthetic);
    PhaseBreakdown::from_times(times, count, total)
}

/// Best-of-N: lowest wall-time AND lowest peak-RSS across N runs.
/// Tracked independently because the two metrics need not hit their
/// minimum in the same iteration (warm-up effects differ). Returns
/// `Err(stderr_excerpt)` if any iteration fails.
///
/// `phase` is the cclab-qc accumulator that captures wall-time for the
/// `ProfilePhase::Total` slot across all iterations. The harness uses
/// it to print a per-fixture phase breakdown footer alongside the
/// best-of-N speed + memory ratios.
fn best_of(
    cmd: &Path,
    args: &[&str],
    iters: u32,
    phase: &mut PhaseTiming,
) -> Result<Sample, String> {
    let mut best_wall = Duration::from_secs(u64::MAX);
    let mut best_rss: Option<u64> = None;
    let mut best_internal: Option<u64> = None;
    for _ in 0..iters {
        let (sample, ok, stderr) = time_one(cmd, args, phase);
        if !ok {
            // Truncate stderr to keep the report readable.
            let excerpt = stderr.lines().take(3).collect::<Vec<_>>().join(" | ");
            return Err(excerpt);
        }
        if sample.wall < best_wall {
            best_wall = sample.wall;
        }
        if let Some(rss) = sample.rss_bytes {
            best_rss = Some(best_rss.map_or(rss, |cur| cur.min(rss)));
        }
        if let Some(int_ns) = sample.internal_ns {
            best_internal = Some(best_internal.map_or(int_ns, |cur| cur.min(int_ns)));
        }
    }
    Ok(Sample {
        wall: best_wall,
        rss_bytes: best_rss,
        internal_ns: best_internal,
    })
}

fn main() -> ExitCode {
    let args = parse_args();
    let root = fixtures_root();

    if !python3_available() {
        println!("[skip] python3 not on $PATH — cannot compute mamba/cpython ratio");
        return ExitCode::SUCCESS;
    }

    let mamba = match mamba_bin() {
        Some(p) => p,
        None => {
            eprintln!("error: `mamba` not found on $PATH (build + install first)");
            return ExitCode::from(1);
        }
    };
    let python = PathBuf::from("python3");

    let fixtures = discover(&root);
    let fixtures: Vec<_> = fixtures
        .into_iter()
        .filter(|f| {
            args.fixture_filter.as_deref().map_or(true, |needle| {
                f.scenario.contains(needle) || f.lib.contains(needle)
            })
        })
        .collect();

    if fixtures.is_empty() {
        eprintln!(
            "error: no bench fixtures found under {} (filter: {:?})",
            root.display(),
            args.fixture_filter
        );
        return ExitCode::from(1);
    }

    // cclab-qc harness-side memory baseline. Printed as a banner so a
    // reviewer can see the bench harness itself isn't bloating up across
    // fixtures; doesn't enter the pass/fail gate (the child-RSS does).
    let harness_before = MemorySnapshot::capture();

    println!(
        "cross_runtime_3p — {} fixture(s), best-of-{}  [harness RSS at start: {:.2} MB]",
        fixtures.len(),
        args.iters,
        harness_before.rss_mb(),
    );
    println!();

    let mut any_fail = false;
    for fix in &fixtures {
        let label = format!("{}/{}", fix.lib, fix.scenario);
        let py_path = fix.path.to_string_lossy().to_string();
        let py_args: &[&str] = &[py_path.as_str()];
        let mb_args: &[&str] = &["run", py_path.as_str()];

        // Warmup correctness probe (Task #15). One untimed run per
        // runtime; bench skips this fixture entirely if mamba's stdout
        // diverges from CPython's — that's the JIT branch-drop
        // fingerprint, and any speed/memory number measured against it
        // would be measuring a fast no-op rather than fast correct work.
        match warmup_probe(&python, &mamba, py_args, mb_args) {
            Ok(_expected) => {}
            Err(e) => {
                println!("[{label}] mamba/cpython = SKIP — warmup probe: {e}");
                any_fail = true;
                continue;
            }
        }

        // Per-fixture cclab-qc PhaseTiming accumulators. Currently only
        // `Total` is recorded — sub-phases (e.g. mamba compile vs run)
        // would be additional accumulators threaded through `time_one`.
        let mut py_phase = PhaseTiming::new();
        let mut mb_phase = PhaseTiming::new();

        let cpython = match best_of(&python, py_args, args.iters, &mut py_phase) {
            Ok(s) => s,
            Err(e) => {
                println!("[{label}] mamba/cpython = ERROR — cpython failed: {e}");
                any_fail = true;
                continue;
            }
        };
        let mamba_t = match best_of(&mamba, mb_args, args.iters, &mut mb_phase) {
            Ok(s) => s,
            Err(e) => {
                println!("[{label}] mamba/cpython = ERROR — mamba failed: {e}");
                any_fail = true;
                continue;
            }
        };

        // PhaseBreakdown summaries are kept for the side effects of
        // validating cclab-qc's surface against the harness use case;
        // their avg_ms equals best-of-N avg, useful as a sanity print
        // when --iters is high.
        let _py_breakdown = summarise_phase(py_phase);
        let _mb_breakdown = summarise_phase(mb_phase);

        // Speed ratio = cpython / mamba — higher is better for mamba.
        // WALL: total `Command::output()` duration including Python
        // startup overhead (~200 ms CPython, ~5 ms mamba). For short
        // benches the ratio is startup-amortization, not steady-state.
        let speed_ratio = cpython.wall.as_secs_f64() / mamba_t.wall.as_secs_f64();
        let speed_verdict = if speed_ratio >= FLOOR { "PASS" } else { "FAIL" };
        if speed_verdict == "FAIL" {
            any_fail = true;
        }
        println!(
            "[{label}] wall mamba/cpython = {speed_ratio:.2}x ({speed_verdict} @ floor {FLOOR:.1}x)  \
             [cpython {:.3}ms, mamba {:.3}ms]",
            cpython.wall.as_secs_f64() * 1000.0,
            mamba_t.wall.as_secs_f64() * 1000.0,
        );

        // INTERNAL: per-call cost as measured by the fixture itself
        // (`time.perf_counter()` around the hot loop, emitted as
        // `INTERNAL_TIME_NS=...` on stdout/stderr). Unbiased by Python
        // startup. Task #22 / `feedback_mamba_perf_is_the_product`
        // methodology caveat 2026-05-13. Both runtimes must emit the
        // marker for a ratio to be reported; otherwise print `UNAVAIL`
        // so reviewers don't mistake a wall-time-only fixture for a
        // methodology pass.
        match (cpython.internal_ns, mamba_t.internal_ns) {
            (Some(cpy_ns), Some(mb_ns)) if mb_ns > 0 => {
                let int_ratio = cpy_ns as f64 / mb_ns as f64;
                let int_verdict = if int_ratio >= FLOOR { "PASS" } else { "FAIL" };
                if int_verdict == "FAIL" {
                    any_fail = true;
                }
                println!(
                    "    internal mamba/cpython = {int_ratio:.2}x ({int_verdict} @ floor {FLOOR:.1}x)  \
                     [cpython {:.3}ms, mamba {:.3}ms]",
                    cpy_ns as f64 / 1_000_000.0,
                    mb_ns as f64 / 1_000_000.0,
                );
            }
            _ => {
                println!(
                    "    internal: UNAVAIL (fixture did not emit INTERNAL_TIME_NS=...; \
                     wall number above includes Python startup — see Task #22 methodology)"
                );
            }
        }

        // Memory ratio = cpython / mamba — higher is better for mamba
        // (mamba uses less than CPython). PASS when ratio >= FLOOR
        // (1.0x): mamba peak-RSS ≤ CPython peak-RSS.
        match (cpython.rss_bytes, mamba_t.rss_bytes) {
            (Some(cpy_b), Some(mb_b)) if mb_b > 0 => {
                let mem_ratio = cpy_b as f64 / mb_b as f64;
                let mem_verdict = if mem_ratio >= FLOOR { "PASS" } else { "FAIL" };
                if mem_verdict == "FAIL" {
                    any_fail = true;
                }
                println!(
                    "    mem: mamba/cpython = {mem_ratio:.2}x ({mem_verdict} @ floor {FLOOR:.1}x)  \
                     [cpython {:.2} MB, mamba {:.2} MB]",
                    cpy_b as f64 / (1024.0 * 1024.0),
                    mb_b as f64 / (1024.0 * 1024.0),
                );
            }
            _ => {
                // RSS measurement unavailable on this platform / wrapper.
                // Speed gate still applies; memory gate is best-effort.
                println!("    mem: UNAVAIL (no /usr/bin/time output parsed)");
            }
        }
    }

    let harness_after = MemorySnapshot::capture();
    println!();
    println!(
        "[harness] RSS delta: {:+.2} MB (start {:.2} MB → end {:.2} MB)",
        harness_after.rss_mb() - harness_before.rss_mb(),
        harness_before.rss_mb(),
        harness_after.rss_mb(),
    );

    if any_fail {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

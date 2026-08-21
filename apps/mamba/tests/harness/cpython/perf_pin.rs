//! Consolidated perf-pin regression gate runner (#1265 Goal 2).
//!
//! Replaces 119 standalone `<lib>_perf_pin_<issue>.rs` binaries. Each pin is
//! described declaratively by a TOML file under
//! `tests/harness/cpython/config/perf/pins/`:
//!
//! ```toml
//! issue = 1447
//! lib   = "abc"
//! fixture = "tests/cpython/_regression/std-libs/abc/bench/get_cache_token_hot.py"
//! floor   = 1.0
//! mem_floor = 1.0      # applies to workload RSS after the fixed runtime floor
//! samples = 1            # 1 = single shot; N>=3 = median-of-N
//! prereq_imports = []    # e.g. ["aiofiles", "google.protobuf"]
//! timeout_secs = 120      # optional; per-pin override (default 120s, #964)
//! ```
//!
//! The runner is `#[ignore]`-equivalent by default: it lives in an integration
//! test binary registered with `harness = false`, so it does not run unless the
//! G3.1 selector explicitly opts in:
//!
//!     cargo test -p mamba --release --test perf_pin_runner -- perf_pin
//!
//! For each TOML entry it loads the CPython baseline from the local SQLite
//! database created by `tests/harness/cpython/tools/perf_baseline.py record`, then
//! spawns `mamba run <fixture>`, measures the child's CPU time externally
//! (getrusage / `/usr/bin/time`), and asserts the mamba/cpython CPU-time ratio
//! `<= floor` (D5.2: the harness owns measurement; fixtures stay pure — no
//! self-emitted timing marker). When the baseline is absent, the
//! runner falls back to live `python3 <fixture>` measurement unless
//! `MAMBA_REQUIRE_CPYTHON_PERF_BASELINE=1` is set.
//!
//! Each pin's emitted test name is `perf_pin::<lib>_<issue>` which lets the
//! `perf_pin` substring filter match every pin in one go.
//!
//! Host-affinity (#966 host column, #1981 enforcement): a baseline recorded
//! on a different host (or a legacy pre-#966 row with no recorded host) is
//! never graded — CPU/RSS ratios are not portable across machines. Such a
//! pin is skipped here exactly like a missing CPython interpreter, with a
//! pointer to `perf_baseline.py record --pin <toml>` to re-record locally.
//! The full per-pin verdict picture (including WHY a pin has no usable
//! baseline, for every pin in one run) is `perf_gate_report.rs`'s job; this
//! file stays the single-pin, panic-on-fail gate. Both share their
//! measurement/baseline primitives via `harness_common.rs`.

use std::path::{Path, PathBuf};
use std::time::Duration;

use datatest_stable::harness;

#[path = "harness_common.rs"]
mod common;
use common::{
    cpython_measurement_from_baseline, evaluate_mem_gate, fixture_sha256, load_same_host_baseline,
    mamba_bin, measure_n, python3_available, python3_can_import, NoBaselineReason, Pin,
    DEFAULT_PIN_TIMEOUT_SECS, MAMBA_FIXED_RUNTIME_RSS_FLOOR_BYTES,
};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn join_gate_failures(failures: &[String]) -> Option<String> {
    if failures.is_empty() {
        None
    } else {
        Some(failures.join("\n"))
    }
}

fn run_pin(toml_path: &Path) -> datatest_stable::Result<()> {
    let raw = std::fs::read_to_string(toml_path)?;
    let pin: Pin = toml::from_str(&raw)?;

    let fixture = manifest_dir().join(&pin.fixture);
    assert!(
        fixture.exists(),
        "#{} {} fixture missing: {}",
        pin.issue,
        pin.lib,
        fixture.display()
    );

    if !python3_available() {
        eprintln!(
            "python3 not available; skipping #{} {} perf gate (mamba-only \
             run is meaningless without the CPython baseline)",
            pin.issue, pin.lib
        );
        return Ok(());
    }

    // #1981: a baseline recorded on a different host (or with no recorded
    // host at all, a legacy pre-#966 row) is never used for ratio grading —
    // CPU/RSS ratios are not portable across machines. Such a pin is skipped
    // exactly like a missing CPython interpreter, rather than a live-python3
    // fallback being attempted against it (the fallback stays for the
    // "genuinely no baseline row yet" case, which has no portability
    // concern since both sides are measured live on this same host).
    let baseline = match load_same_host_baseline(toml_path) {
        Ok(baseline) => Some(baseline),
        Err(NoBaselineReason::CrossHost) => {
            eprintln!(
                "#{} {} CPython perf baseline was recorded on a different host \
                 (or is a legacy pre-host-tracking row with no recorded host); \
                 CPU/RSS ratios are not portable across machines, so this pin is \
                 skipped rather than graded against non-portable data. Re-record \
                 on this host with `python3 tests/harness/cpython/tools/perf_baseline.py \
                 record --pin {}`.",
                pin.issue,
                pin.lib,
                toml_path.display()
            );
            return Ok(());
        }
        Err(NoBaselineReason::Missing) => None,
    };
    if let Some(baseline) = &baseline {
        let actual_hash = fixture_sha256(&fixture)?;
        assert_eq!(
            baseline.fixture_sha256,
            actual_hash,
            "#{} {} CPython perf baseline is stale for {}. Re-run `python3 tests/harness/cpython/tools/perf_baseline.py record --pin {}`.",
            pin.issue,
            pin.lib,
            fixture.display(),
            toml_path.display()
        );
        eprintln!(
            "#{} {} CPython perf baseline: {} samples={} python={} captured_at={} \
             internal={} ns cpu={:?} rss={:?} host={:?}",
            pin.issue,
            pin.lib,
            baseline.pin_path,
            baseline.samples,
            baseline.python,
            baseline.captured_at_unix,
            baseline.internal_time_ns,
            baseline.cpu_time_ns,
            baseline.peak_rss_bytes,
            baseline.host,
        );
    } else {
        eprintln!(
            "#{} {} CPython perf baseline missing; falling back to live python3 measurement",
            pin.issue, pin.lib
        );
        for imp in &pin.prereq_imports {
            if !python3_can_import(imp) {
                eprintln!(
                    "python3 lacks `{imp}`; skipping #{} {} perf gate \
                     (CPython baseline unavailable on this host)",
                    pin.issue, pin.lib
                );
                return Ok(());
            }
        }
    }

    let fixture_str = fixture.to_str().expect("fixture path is not valid UTF-8");
    let mamba_bin_path = mamba_bin();
    let mamba_bin_str = mamba_bin_path
        .to_str()
        .expect("mamba binary path is not valid UTF-8");

    let samples = pin.samples.max(1);
    let timeout = Duration::from_secs(pin.timeout_secs.unwrap_or(DEFAULT_PIN_TIMEOUT_SECS));
    let cpy = if let Some(baseline) = &baseline {
        cpython_measurement_from_baseline(baseline)
    } else {
        measure_n("python3", &[fixture_str], samples, timeout)
    };
    let mb = measure_n(mamba_bin_str, &["run", fixture_str], samples, timeout);

    let mode = if samples <= 1 {
        "single-shot".to_string()
    } else {
        format!("median-of-{samples}")
    };
    let mut gate_failures = Vec::new();

    // OPTIONAL peak-RSS gate. A pin without `mem_floor` behaves exactly as
    // before (no assertion). When present, assert mem_ratio = cpython_rss /
    // max(mamba_rss - fixed_runtime_floor, 0) >= mem_floor. This keeps the
    // per-pin `mem_floor` semantics intact while accounting for mamba's fixed
    // ship-profile runtime RSS floor (#1024) instead of weakening individual
    // pins. The CPython side comes from the SQLite baseline when present;
    // otherwise it is measured live as the compatibility fallback.
    if let Some(mem_floor) = pin.mem_floor {
        match (cpy.peak_rss_bytes, mb.peak_rss_bytes) {
            (Some(cpy_b), Some(mb_b)) if mb_b > 0 => {
                let evaluation = evaluate_mem_gate(cpy_b, mb_b);
                eprintln!(
                    "#{} {} mem gate: raw cpython/mamba peak-RSS ratio = {:.3}x; \
                     fixed-floor-adjusted ratio = {:.3}x using {} B fixed runtime RSS allowance \
                     (effective mamba workload RSS {} B; mamba total {} B vs cpython {} B)",
                    pin.issue,
                    pin.lib,
                    evaluation.raw_ratio,
                    evaluation.adjusted_ratio,
                    MAMBA_FIXED_RUNTIME_RSS_FLOOR_BYTES,
                    evaluation.effective_mamba_rss_bytes,
                    mb_b,
                    cpy_b
                );
                if evaluation.adjusted_ratio < mem_floor {
                    gate_failures.push(format!(
                        "#{} {} mem gate FAIL: fixed-floor-adjusted cpython/mamba workload-RSS ratio = \
                         {:.2}x below floor of {:.2}x after applying {} B fixed runtime RSS allowance \
                         (raw ratio {:.2}x; effective mamba workload RSS {} B; mamba total {} B vs cpython {} B)",
                        pin.issue,
                        pin.lib,
                        evaluation.adjusted_ratio,
                        mem_floor,
                        MAMBA_FIXED_RUNTIME_RSS_FLOOR_BYTES,
                        evaluation.raw_ratio,
                        evaluation.effective_mamba_rss_bytes,
                        mb_b,
                        cpy_b,
                    ));
                }
            }
            _ => {
                eprintln!(
                    "#{} {} mem gate skipped: peak-RSS measurement unavailable \
                     (cpython={:?}, mamba={:?}); mem_floor={:.2}x left unenforced",
                    pin.issue, pin.lib, cpy.peak_rss_bytes, mb.peak_rss_bytes, mem_floor
                );
            }
        }
    }

    // D5.2: the gate is the EXTERNAL CPU-time ratio (getrusage / /usr/bin/time),
    // not a fixture-emitted self-timing marker. Process-startup cost is
    // included; warmup/median (samples) damps it. See PRODUCTION-GATE.md D5.2.
    match (cpy.cpu_time_ns, mb.cpu_time_ns) {
        (Some(cpy_cpu), Some(mb_cpu)) if cpy_cpu > 0 => {
            let cpu_ratio = mb_cpu as f64 / cpy_cpu as f64;
            eprintln!(
                "#{} {} CPU gate ({mode}): mamba/cpython CPU-time ratio = {:.3}x \
                 (mamba {} ns vs cpython {} ns)",
                pin.issue, pin.lib, cpu_ratio, mb_cpu, cpy_cpu
            );
            if cpu_ratio > pin.floor {
                gate_failures.push(format!(
                    "#{} {} CPU gate FAIL: ratio = {:.2}x exceeds floor of {:.2}x \
                     (mamba {} ns vs cpython {} ns) [{mode}]",
                    pin.issue, pin.lib, cpu_ratio, pin.floor, mb_cpu, cpy_cpu,
                ));
            }
        }
        _ => {
            eprintln!(
                "#{} {} CPU gate skipped: CPU-time measurement unavailable \
                 (cpython={:?}, mamba={:?})",
                pin.issue, pin.lib, cpy.cpu_time_ns, mb.cpu_time_ns
            );
        }
    }
    if let Some(message) = join_gate_failures(&gate_failures) {
        panic!("{message}");
    }
    Ok(())
}

harness!(
    run_pin,
    "tests/harness/cpython/config/perf/pins",
    r"^.*\.toml$"
);

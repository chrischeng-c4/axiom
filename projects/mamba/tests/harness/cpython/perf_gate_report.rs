//! Full-enumeration perf-pin gate report (#1981).
//!
//! `perf_pin.rs` runs every pin as an *independent* `datatest_stable::Trial`,
//! so a single pin's panic never masks the others there — but it also never
//! produces one aggregate picture of the whole ~128-pin surface: there is no
//! single run that answers "how many of the perf pins currently pass, and
//! which ones don't, and why." This file is that aggregate: ONE `#[test]`
//! that walks every pin under `tests/harness/cpython/config/perf/pins/`,
//! classifies each into a verdict —
//!
//!   * `pass`           — CPU (and, if `mem_floor` is set, memory) ratio
//!                        measured and within floor. Ratios are reported even
//!                        on pass, so a healthy pin's numbers are visible too
//!                        (not just failures).
//!   * `ratio-fail`     — measured, but CPU or memory ratio exceeds its floor.
//!   * `fixture-error`  — the pin TOML, its fixture, or its baseline could not
//!                        be read/parsed/hashed, or the measurement itself
//!                        panicked (spawn failure, timeout, non-zero exit).
//!   * `no-baseline`    — no *usable* CPython comparison is available this
//!                        run: python3 itself is unavailable, the pin has no
//!                        baseline row and lacks a live-python3 prereq, or its
//!                        only baseline row is not from this host (see
//!                        host-affinity below) — never a ratio in this case.
//!
//! and NEVER stops early: one pin's panic is caught (`catch_unwind`) and
//! downgraded to a `fixture-error` row rather than aborting the run, so every
//! pin always gets a verdict row in the same run. The gate still fails (single
//! `panic!`) at the very end when any pin is non-pass, with per-class counts.
//!
//! A JSON sidecar is written every run (pass or fail) to
//! `tests/cpython/.cache/perf/last_gate.json`, mirroring the shape convention
//! of the conformance gate's `tests/cpython/.cache/conformance/last_gate.json`
//! (`runner.rs`): `schema_version` / `harness_kind` / `generated_at_unix_secs`
//! / `total` / `counts` / `non_pass_count` / `non_pass`, plus a `pins` array
//! that (unlike the conformance sidecar) carries EVERY pin, not just the
//! non-passing ones — the whole point of "full enumeration" is that a passing
//! pin's measured ratio stays visible too. `tests/cpython/.cache/` is
//! git-ignored, so the sidecar is machine-local like the SQLite baseline it
//! reads from.
//!
//! Host-affinity (#966 host column, #1981 enforcement): a baseline recorded
//! on a different host — or a legacy pre-#966 row with no recorded host at
//! all — is never used for ratio grading (see `harness_common.rs`'s
//! `load_same_host_baseline`); such a pin reports `no-baseline("cross-host")`
//! rather than a non-portable ratio. Re-record on this host with
//! `python3 tests/harness/cpython/tools/perf_baseline.py record`.
//!
//! This binary is opt-in (`test = false`, like `perf_pin.rs`):
//!
//!     cargo test -p mamba --release --test perf_gate_report -- --nocapture
//!
//! `--nocapture` is recommended so the per-pin lines stream as the gate runs
//! rather than being buffered until the end.
//!
//! Unlike `runner.rs`'s conformance gate (many independent `#[test]`-like
//! trials, where a filtered/partial invocation must not clobber the full-run
//! sidecar), this file has exactly one `#[test]` function: any invocation
//! that runs it at all runs the complete pin set, so no
//! `is_full_unfiltered_run()`-style guard is needed before writing the
//! sidecar.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[path = "harness_common.rs"]
mod common;
use common::{
    collect_files, cpython_measurement_from_baseline, evaluate_mem_gate, fixture_sha256,
    load_same_host_baseline, mamba_bin, measure_n, python3_available, python3_can_import,
    Measurement, NoBaselineReason, Pin, DEFAULT_PIN_TIMEOUT_SECS,
};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn pins_dir() -> PathBuf {
    manifest_dir().join("tests/harness/cpython/config/perf/pins")
}

fn sidecar_path() -> PathBuf {
    manifest_dir().join("tests/cpython/.cache/perf/last_gate.json")
}

fn rel_path_string(path: &Path) -> String {
    path.strip_prefix(manifest_dir())
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    Pass,
    RatioFail,
    FixtureError,
    NoBaseline,
}

impl Verdict {
    fn as_str(self) -> &'static str {
        match self {
            Verdict::Pass => "pass",
            Verdict::RatioFail => "ratio-fail",
            Verdict::FixtureError => "fixture-error",
            Verdict::NoBaseline => "no-baseline",
        }
    }
}

struct PinReport {
    /// Pin TOML path, relative to the crate manifest dir.
    path: String,
    issue: Option<u64>,
    lib: Option<String>,
    verdict: Verdict,
    detail: String,
}

impl PinReport {
    fn fixture_error(
        path: String,
        issue: Option<u64>,
        lib: Option<String>,
        detail: String,
    ) -> Self {
        PinReport {
            path,
            issue,
            lib,
            verdict: Verdict::FixtureError,
            detail,
        }
    }

    fn no_baseline(path: String, issue: Option<u64>, lib: Option<String>, detail: String) -> Self {
        PinReport {
            path,
            issue,
            lib,
            verdict: Verdict::NoBaseline,
            detail,
        }
    }
}

/// Grade a resolved (CPython, mamba) measurement pair against `pin`'s
/// floor/mem_floor, producing `pass` or `ratio-fail` with the measured
/// ratios always in `detail` (visible whether or not the pin passes — the
/// "unmask the pin surface" point of this report).
fn grade(rel_path: String, pin: &Pin, cpy: Measurement, mb: Measurement) -> PinReport {
    let mut fail_reasons: Vec<String> = Vec::new();
    let mut detail_parts: Vec<String> = Vec::new();

    match (cpy.cpu_time_ns, mb.cpu_time_ns) {
        (Some(cpy_cpu), Some(mb_cpu)) if cpy_cpu > 0 => {
            let cpu_ratio = mb_cpu as f64 / cpy_cpu as f64;
            detail_parts.push(format!("cpu={cpu_ratio:.3}x(floor {:.2}x)", pin.floor));
            if cpu_ratio > pin.floor {
                fail_reasons.push(format!(
                    "CPU ratio {cpu_ratio:.2}x exceeds floor {:.2}x (mamba {mb_cpu} ns vs cpython {cpy_cpu} ns)",
                    pin.floor
                ));
            }
        }
        _ => detail_parts.push("cpu=unavailable".to_string()),
    }

    if let Some(mem_floor) = pin.mem_floor {
        match (cpy.peak_rss_bytes, mb.peak_rss_bytes) {
            (Some(cpy_b), Some(mb_b)) if mb_b > 0 => {
                let evaluation = evaluate_mem_gate(cpy_b, mb_b);
                detail_parts.push(format!(
                    "mem={:.3}x(floor {mem_floor:.2}x,raw {:.3}x)",
                    evaluation.adjusted_ratio, evaluation.raw_ratio
                ));
                if evaluation.adjusted_ratio < mem_floor {
                    fail_reasons.push(format!(
                        "mem ratio {:.2}x below floor {mem_floor:.2}x (raw {:.2}x; mamba {mb_b} B vs cpython {cpy_b} B)",
                        evaluation.adjusted_ratio, evaluation.raw_ratio
                    ));
                }
            }
            _ => detail_parts.push("mem=unavailable".to_string()),
        }
    }

    let detail = detail_parts.join(", ");
    if fail_reasons.is_empty() {
        PinReport {
            path: rel_path,
            issue: Some(pin.issue),
            lib: Some(pin.lib.clone()),
            verdict: Verdict::Pass,
            detail,
        }
    } else {
        PinReport {
            path: rel_path,
            issue: Some(pin.issue),
            lib: Some(pin.lib.clone()),
            verdict: Verdict::RatioFail,
            detail: format!("{detail} — {}", fail_reasons.join("; ")),
        }
    }
}

/// Read and parse one pin TOML. This step cannot panic (both failure modes
/// are ordinary `Result`s), so it stays outside the caller's `catch_unwind` —
/// which means a pin that panics later, during measurement, still gets to
/// keep its `issue`/`lib` identity in the resulting report (see
/// `perf_pins_full_gate_report`): only the genuinely risky suffix
/// ([`evaluate_pin`]) needs the panic safety net.
fn parse_pin(toml_path: &Path) -> Result<(String, Pin), PinReport> {
    let rel_path = rel_path_string(toml_path);
    let raw = std::fs::read_to_string(toml_path).map_err(|err| {
        PinReport::fixture_error(
            rel_path.clone(),
            None,
            None,
            format!("cannot read pin toml: {err}"),
        )
    })?;
    let pin: Pin = toml::from_str(&raw).map_err(|err| {
        PinReport::fixture_error(
            rel_path.clone(),
            None,
            None,
            format!("cannot parse pin toml: {err}"),
        )
    })?;
    Ok((rel_path, pin))
}

/// Evaluate one already-parsed pin into a [`PinReport`]. Never panics on a
/// STRUCTURAL problem (missing fixture, stale baseline hash) — those become
/// `fixture-error` rows directly. A panic from deep inside the external
/// measurement path (spawn failure, timeout, non-zero exit — see
/// `harness_common.rs::run_once_with_metrics`) is still possible; the caller
/// wraps this call in `catch_unwind` as the outer safety net so even that can
/// never mask the rest of the pins in one run. `toml_path` is passed through
/// separately from the already-parsed `pin` because `load_same_host_baseline`
/// re-reads the TOML by path (it is keyed on `toml_path`, not the in-memory
/// `Pin`).
fn evaluate_pin(rel_path: String, pin: Pin, toml_path: &Path) -> PinReport {
    let fixture = manifest_dir().join(&pin.fixture);
    if !fixture.exists() {
        return PinReport::fixture_error(
            rel_path,
            Some(pin.issue),
            Some(pin.lib.clone()),
            format!("fixture missing: {}", fixture.display()),
        );
    }
    let fixture_str = match fixture.to_str() {
        Some(s) => s.to_string(),
        None => {
            return PinReport::fixture_error(
                rel_path,
                Some(pin.issue),
                Some(pin.lib.clone()),
                "fixture path is not valid UTF-8".to_string(),
            )
        }
    };

    if !python3_available() {
        return PinReport::no_baseline(
            rel_path,
            Some(pin.issue),
            Some(pin.lib.clone()),
            "python3 unavailable (mamba-only run is meaningless without the CPython baseline)"
                .to_string(),
        );
    }

    let samples = pin.samples.max(1);
    let timeout = Duration::from_secs(pin.timeout_secs.unwrap_or(DEFAULT_PIN_TIMEOUT_SECS));

    // Resolve the CPython side: a same-host cached baseline, or — only when
    // no row exists at all, which has no cross-host portability concern — a
    // live python3 measurement taken in this same run. A row that exists but
    // isn't this host's is #1981's `no-baseline("cross-host")`: never a
    // ratio, never a live-fallback measurement either.
    let cpy = match load_same_host_baseline(toml_path) {
        Ok(baseline) => {
            let actual_hash = match fixture_sha256(&fixture) {
                Ok(hash) => hash,
                Err(err) => {
                    return PinReport::fixture_error(
                        rel_path,
                        Some(pin.issue),
                        Some(pin.lib.clone()),
                        format!("cannot hash fixture: {err}"),
                    )
                }
            };
            if baseline.fixture_sha256 != actual_hash {
                return PinReport::fixture_error(
                    rel_path,
                    Some(pin.issue),
                    Some(pin.lib.clone()),
                    "baseline stale: fixture sha256 changed since capture".to_string(),
                );
            }
            cpython_measurement_from_baseline(&baseline)
        }
        Err(NoBaselineReason::CrossHost) => {
            return PinReport::no_baseline(
                rel_path,
                Some(pin.issue),
                Some(pin.lib.clone()),
                "cross-host: baseline recorded on a different host (or a legacy row with no \
                 recorded host); ratios are not portable across machines"
                    .to_string(),
            );
        }
        Err(NoBaselineReason::Missing) => {
            if let Some(imp) = pin
                .prereq_imports
                .iter()
                .find(|imp| !python3_can_import(imp))
            {
                return PinReport::no_baseline(
                    rel_path,
                    Some(pin.issue),
                    Some(pin.lib.clone()),
                    format!("missing: no baseline row and python3 lacks prereq `{imp}`"),
                );
            }
            measure_n("python3", &[fixture_str.as_str()], samples, timeout)
        }
    };

    let mamba_bin_path = mamba_bin();
    let mamba_bin_str = match mamba_bin_path.to_str() {
        Some(s) => s,
        None => {
            return PinReport::fixture_error(
                rel_path,
                Some(pin.issue),
                Some(pin.lib.clone()),
                "mamba binary path is not valid UTF-8".to_string(),
            )
        }
    };
    let mb = measure_n(
        mamba_bin_str,
        &["run", fixture_str.as_str()],
        samples,
        timeout,
    );

    grade(rel_path, &pin, cpy, mb)
}

/// Extract a one-line message from a caught panic payload.
fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "panicked with a non-string payload".to_string()
    }
}

/// Mirrors `runner.rs`'s verdict-detail truncation: first line only, capped
/// at 200 chars, so a panic message that embeds a subprocess's full
/// stdout/stderr does not blow up a report line or the sidecar. Truncates on
/// `char` boundaries (never byte-slices) so it can never panic on non-ASCII
/// output.
fn truncate_detail(detail: &str) -> String {
    let first_line = detail.lines().next().unwrap_or("");
    let truncated: String = first_line.chars().take(200).collect();
    if truncated.chars().count() < first_line.chars().count() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn json_str_or_null(value: Option<&str>) -> String {
    match value {
        Some(s) => format!("\"{}\"", json_escape(s)),
        None => "null".to_string(),
    }
}

/// Write the full-enumeration sidecar (schema mirrors `runner.rs`'s
/// conformance `last_gate.json`; `pins` additionally carries every row, not
/// just non-pass, per #1981's "every pin gets a verdict row"). Called
/// unconditionally before the end-of-run `panic!` so the artifact is written
/// whether the run passes or fails.
fn write_sidecar(reports: &[PinReport]) {
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for report in reports {
        *counts.entry(report.verdict.as_str()).or_insert(0) += 1;
    }
    let non_pass: Vec<&PinReport> = reports
        .iter()
        .filter(|r| r.verdict != Verdict::Pass)
        .collect();
    let generated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut json = String::new();
    json.push_str("{\n");
    json.push_str("  \"schema_version\": 1,\n");
    json.push_str("  \"harness_kind\": \"perf\",\n");
    json.push_str(&format!("  \"generated_at_unix_secs\": {generated_at},\n"));
    json.push_str(&format!("  \"total\": {},\n", reports.len()));
    let counts_str = counts
        .iter()
        .map(|(k, v)| format!("\"{k}\": {v}"))
        .collect::<Vec<_>>()
        .join(", ");
    json.push_str(&format!("  \"counts\": {{{counts_str}}},\n"));
    json.push_str(&format!("  \"non_pass_count\": {},\n", non_pass.len()));

    json.push_str("  \"pins\": [\n");
    for (i, report) in reports.iter().enumerate() {
        json.push_str(&format!(
            "    {{\"path\": {}, \"issue\": {}, \"lib\": {}, \"verdict\": \"{}\", \"detail\": {}}}",
            json_str_or_null(Some(&report.path)),
            report
                .issue
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".to_string()),
            json_str_or_null(report.lib.as_deref()),
            report.verdict.as_str(),
            json_str_or_null(Some(&truncate_detail(&report.detail))),
        ));
        json.push_str(if i + 1 < reports.len() { ",\n" } else { "\n" });
    }
    json.push_str("  ],\n");

    json.push_str("  \"non_pass\": [\n");
    for (i, report) in non_pass.iter().enumerate() {
        json.push_str(&format!(
            "    {{\"path\": {}, \"verdict\": \"{}\", \"detail\": {}}}",
            json_str_or_null(Some(&report.path)),
            report.verdict.as_str(),
            json_str_or_null(Some(&truncate_detail(&report.detail))),
        ));
        json.push_str(if i + 1 < non_pass.len() { ",\n" } else { "\n" });
    }
    json.push_str("  ]\n");
    json.push_str("}\n");

    let path = sidecar_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(err) = std::fs::write(&path, json) {
        eprintln!(
            "warning: failed to write perf-pin gate sidecar to {}: {err}",
            path.display()
        );
    }
}

#[test]
fn perf_pins_full_gate_report() {
    let dir = pins_dir();
    let pins = collect_files(&dir, ".toml");
    assert!(
        !pins.is_empty(),
        "expected at least one perf pin under {}",
        dir.display()
    );

    let mut reports: Vec<PinReport> = Vec::with_capacity(pins.len());
    for toml_path in &pins {
        // Parse first, outside `catch_unwind` (parsing cannot panic — see
        // `parse_pin`), so a panic during the risky measurement step below
        // still has `issue`/`lib` available for its fallback `fixture-error`
        // row instead of losing that identity to the panic boundary.
        let report = match parse_pin(toml_path) {
            Err(report) => report,
            Ok((rel_path, pin)) => {
                let issue = Some(pin.issue);
                let lib = Some(pin.lib.clone());
                let path_for_panic = rel_path.clone();
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    evaluate_pin(rel_path, pin, toml_path)
                }))
                .unwrap_or_else(|payload| {
                    PinReport::fixture_error(
                        path_for_panic,
                        issue,
                        lib,
                        truncate_detail(&panic_message(payload)),
                    )
                })
            }
        };
        eprintln!(
            "[{}] {}{} — {}",
            report.verdict.as_str(),
            report.lib.as_deref().unwrap_or("?"),
            report
                .issue
                .map(|i| format!("#{i}"))
                .unwrap_or_else(|| String::from("#?")),
            report.detail
        );
        reports.push(report);
    }

    write_sidecar(&reports);

    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for report in &reports {
        *counts.entry(report.verdict.as_str()).or_insert(0) += 1;
    }
    let non_pass: Vec<&PinReport> = reports
        .iter()
        .filter(|r| r.verdict != Verdict::Pass)
        .collect();

    if !non_pass.is_empty() {
        let summary = non_pass
            .iter()
            .map(|r| format!("  [{}] {} — {}", r.verdict.as_str(), r.path, r.detail))
            .collect::<Vec<_>>()
            .join("\n");
        panic!(
            "{} of {} perf pins are non-pass; counts={counts:?}\n{summary}\nsidecar: {}",
            non_pass.len(),
            reports.len(),
            sidecar_path().display(),
        );
    }
}

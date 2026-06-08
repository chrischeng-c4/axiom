//! CPython 3.12 Lib/test conformance runner — folder-based contract
//! convention (#3729).
//!
//! Each seed's parent directory name encodes the outcome the runner must
//! observe. The runner walks every configured contract dir under
//! `tests/harness/cpython/config/seeds/`, executes each seed through
//! `mamba run`, classifies the outcome, and fails if any seed's actual
//! outcome diverges from the outcome its parent directory claims.
//!
//! ## Contract directories
//!
//!   - `pass/`         — must AssertionPass (printed
//!                       `MAMBA_ASSERTION_PASS` marker after every raw
//!                       `assert` executed; exit 0)
//!   - `spec/`         — must Fail today. Encodes full CPython 3.12
//!                       runtime contracts that mamba does NOT implement
//!                       yet. When mamba grows into the contract, the
//!                       seed will start passing and the runner reports
//!                       the drift — promotion is `git mv spec/<f>
//!                       pass/<f>`. Equivalent to `fail/` in raw
//!                       contract terms but carries the explicit
//!                       intent of being a growth-tracked spec.
//!   - `stub/`         — must Stub (mamba silently bypassed the entry
//!                       point, e.g. `unittest.main()`)
//!   - `fail/`         — must Fail. Known broken or unsupported syntax
//!                       with no current spec-driven growth path.
//!   - `import_pass/`  — must ImportPass (exit 0 with no proof of
//!                       assertion). Legacy outcome — new seeds should
//!                       land in `pass/` or `spec/`.
//!   - `timeout/`      — must Timeout (60s budget exceeded).
//!
//! ## Promotion / demotion
//!
//! Improvement: `git mv spec/<file>.py pass/<file>.py`. Drift gate then
//! records the new pinned outcome via the directory itself.
//!
//! Regression: `git mv pass/<file>.py fail/<file>.py` (or `stub/`).
//!
//! No TOML edits — the directory layout IS the contract.
//!
//! ## Outcome classification
//!
//!   - `AssertionPass` — exit 0, no stub marker, and at least one
//!                       proof-of-execution marker
//!                       (`MAMBA_ASSERTION_PASS` or
//!                       `[mamba-assertion-pass]`) present in
//!                       stdout / stderr.
//!   - `ImportPass`    — exit 0, no stub marker, no proof marker. The
//!                       module imported but no assertion is known to
//!                       have executed.
//!   - `Stub`          — exit 0 but a known stub marker is present
//!                       (e.g. `unittest.main() called`).
//!   - `Fail`          — non-zero exit (runtime / parse / type error,
//!                       or an `AssertionError` from a raw `assert`).
//!   - `Timeout`       — child took longer than 60s.
//!
//! ## Summary JSON
//!
//! The runner writes a machine-readable sidecar (path:
//! `$CARGO_TARGET_TMPDIR/cpython_lib_test_summary.json`, override via
//! `MAMBA_CPYTHON_LIB_TEST_SUMMARY_PATH`) carrying per-outcome counts
//! plus a `drift_entries` array. The schema is versioned
//! (`schema_version = 2`, `harness_kind = "runtime"`). #2546 split this
//! out from the parser-only `cpython_compat.rs` harness — CI scrapers
//! key on `harness_kind` so the two counts can never be silently summed.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn fixture_root() -> PathBuf {
    // Seed corpus lives under the harness config (config/seeds/), moved out of
    // tests/cpython/ in the reorg. The old tests/fixtures/cpython_lib_test path
    // never existed, so this gate read an empty root (total=0) before.
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/harness/cpython/config/seeds")
}

/// Ordered list of `(subdir, expected_outcome)` pairs the runner walks.
///
/// Order is stable so per-seed stderr is deterministic across runs.
/// Stems MUST be unique across contract dirs — duplicate stems are a
/// fatal error from `discover_seeds`.
///
/// `spec/` and `fail/` both demand `Outcome::Fail` — the distinction
/// is semantic (spec = full CPython contract for mamba to grow into;
/// fail = known broken with no growth path). A spec/ seed that starts
/// passing surfaces drift, prompting a `git mv` into pass/.
const CONTRACT_DIRS: &[(&str, Outcome)] = &[
    ("pass", Outcome::AssertionPass),
    ("spec", Outcome::Fail),
    ("stub", Outcome::Stub),
    ("fail", Outcome::Fail),
    ("import_pass", Outcome::ImportPass),
    ("timeout", Outcome::Timeout),
];

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Outcome {
    ImportPass,
    AssertionPass,
    Stub,
    Fail,
    Timeout,
}

impl Outcome {
    /// True iff this outcome counts as a passing CPython Lib/test for
    /// MVP coverage accounting. Per #2540: only AssertionPass counts —
    /// ImportPass and Stub explicitly do NOT.
    fn is_mvp_pass(self) -> bool {
        matches!(self, Outcome::AssertionPass)
    }
}

/// Per-seed wall-clock budget. Bumped to 60s 2026-05-16 because several
/// T1 / T2 seeds (e.g. `test_int`) flap around 30s depending on host
/// load. Genuine infinite loops still surface as Timeout.
const SEED_TIMEOUT: Duration = Duration::from_secs(60);

/// Known stub markers — when stderr contains any of these, the test
/// "passed" only because mamba silently bypassed the entry point.
const STUB_MARKERS: &[&str] = &["unittest.main() called", "is not implemented in Mamba"];

/// Positive proof-of-execution marker (#2691). Their presence escalates
/// `ImportPass` to `AssertionPass`. Inverting any `assert` still fails
/// the seed — Python exits non-zero on AssertionError before the marker
/// is printed.
const ASSERTION_PASS_MARKERS: &[&str] = &["MAMBA_ASSERTION_PASS", "[mamba-assertion-pass]"];

fn run_seed(path: &Path) -> Outcome {
    let output = Command::new(mamba_bin())
        .args(["run", path.to_str().expect("seed path is utf8")])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            let timeout = SEED_TIMEOUT;
            let start = std::time::Instant::now();
            loop {
                match child.try_wait()? {
                    Some(_status) => return child.wait_with_output(),
                    None => {
                        if start.elapsed() > timeout {
                            let _ = child.kill();
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::TimedOut,
                                "seed exceeded 60s budget",
                            ));
                        }
                        std::thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        });

    let output = match output {
        Ok(o) => o,
        Err(e) if e.kind() == std::io::ErrorKind::TimedOut => return Outcome::Timeout,
        Err(_) => return Outcome::Fail,
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    if !output.status.success() {
        return Outcome::Fail;
    }

    let combined: String = format!("{stderr}\n{stdout}");
    if STUB_MARKERS.iter().any(|m| combined.contains(m)) {
        return Outcome::Stub;
    }

    if ASSERTION_PASS_MARKERS.iter().any(|m| combined.contains(m)) {
        return Outcome::AssertionPass;
    }

    Outcome::ImportPass
}

/// A discovered seed: the contract dir it lives under, the outcome that
/// dir pins, and the path on disk.
#[derive(Debug, Clone)]
struct Entry {
    subdir: String,
    expected: Outcome,
    path: PathBuf,
}

/// Walk every contract dir under `fixture_root`, returning the flat list
/// of seeds with the outcome their parent dir pins.
///
/// `dirs` is parameterised so unit tests can drive the function with
/// synthetic temp directories.
///
/// Errors with `duplicate stem ...` if two different contract dirs both
/// contain `<stem>.py` — a seed lives in exactly one outcome bucket.
fn discover_seeds(fixture_root: &Path, dirs: &[(&str, Outcome)]) -> Result<Vec<Entry>, String> {
    let mut out: Vec<Entry> = Vec::new();
    let mut stem_to_dir: HashMap<String, String> = HashMap::new();
    for &(subdir, expected) in dirs {
        let dir = fixture_root.join(subdir);
        if !dir.exists() {
            continue;
        }
        let read = std::fs::read_dir(&dir)
            .map_err(|e| format!("contract dir {} unreadable: {e}", dir.display()))?;
        let mut paths: Vec<PathBuf> = read
            .filter_map(|r| r.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "py"))
            .collect();
        paths.sort();
        for path in paths {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if let Some(prev_dir) = stem_to_dir.get(&stem) {
                return Err(format!(
                    "duplicate stem `{stem}` in contract dirs \
                     [{prev_dir}, {subdir}] — each seed lives in \
                     exactly one outcome bucket",
                ));
            }
            stem_to_dir.insert(stem, subdir.to_string());
            out.push(Entry {
                subdir: subdir.to_string(),
                expected,
                path,
            });
        }
    }
    Ok(out)
}

#[test]
fn cpython_lib_test_folder_contracts() {
    let fixture_root = fixture_root();
    let entries = discover_seeds(&fixture_root, CONTRACT_DIRS)
        .unwrap_or_else(|e| panic!("seed discovery failed: {e}"));

    let mut import_pass = 0usize;
    let mut assertion_pass = 0usize;
    let mut stub = 0usize;
    let mut fail = 0usize;
    let mut timeout = 0usize;
    let mut mvp_pass = 0usize;

    let mut drift: Vec<String> = Vec::new();

    for entry in &entries {
        let name = entry
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?");
        let stem = entry
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let actual = run_seed(&entry.path);
        match actual {
            Outcome::ImportPass => import_pass += 1,
            Outcome::AssertionPass => assertion_pass += 1,
            Outcome::Stub => stub += 1,
            Outcome::Fail => fail += 1,
            Outcome::Timeout => timeout += 1,
        }
        if actual.is_mvp_pass() {
            mvp_pass += 1;
        }
        eprintln!(
            "[cpython_lib_test] {}/{:<40} expected={:?} actual={:?}",
            entry.subdir, name, entry.expected, actual,
        );

        if actual != entry.expected {
            let hint = if actual == Outcome::AssertionPass && entry.subdir == "spec" {
                format!(
                    " — promote: `git mv {sub}/{stem}.py pass/{stem}.py`",
                    sub = entry.subdir,
                    stem = stem,
                )
            } else {
                format!(
                    " — move {sub}/{stem}.py into the matching outcome dir",
                    sub = entry.subdir,
                    stem = stem,
                )
            };
            drift.push(format!(
                "  - {sub}/{stem}: expected {exp:?}, got {act:?}{hint}",
                sub = entry.subdir,
                stem = stem,
                exp = entry.expected,
                act = actual,
            ));
        }
    }

    let total = entries.len();
    let mvp_marker = if mvp_pass == 0 {
        "MVP-PENDING"
    } else {
        "MVP-OK"
    };
    eprintln!(
        "[cpython_lib_test] summary: harness=runtime {mvp_marker} \
         total={total} mvp_pass={mvp_pass} import_pass={import_pass} \
         assertion_pass={assertion_pass} stub={stub} fail={fail} \
         timeout={timeout}"
    );

    write_summary_json(SummarySnapshot {
        harness_kind: "runtime",
        total,
        mvp_pass,
        import_pass,
        assertion_pass,
        stub,
        fail,
        timeout,
        drift: &drift,
    });

    assert!(
        total > 0,
        "no seed files under any contract dir in {}",
        fixture_root.display(),
    );
    assert!(
        drift.is_empty(),
        "cpython_lib_test folder-contract drift ({} violation{}):\n{}",
        drift.len(),
        if drift.len() == 1 { "" } else { "s" },
        drift.join("\n"),
    );
}

/// #2544 MVP gate. Asserts at least one seed lives under `pass/` — the
/// "no AssertionPass seed exists" state is the "MVP not yet started"
/// signal and explicitly fails here.
///
/// Cheap: walks the contract dirs without executing any seed.
#[test]
fn cpython_lib_test_mvp_gate_requires_pass_dir_entry() {
    let entries = discover_seeds(&fixture_root(), CONTRACT_DIRS)
        .unwrap_or_else(|e| panic!("seed discovery failed: {e}"));
    let pass_count = entries.iter().filter(|e| e.subdir == "pass").count();
    assert!(
        pass_count > 0,
        "CPython Lib/test MVP gate: 0 seed files under pass/ — the MVP \
         pass count is therefore 0. Move at least one seed to pass/ to \
         unblock this gate.",
    );
}

/// Per-run counts written out as `summary_json_path()`.
struct SummarySnapshot<'a> {
    harness_kind: &'static str,
    total: usize,
    mvp_pass: usize,
    import_pass: usize,
    assertion_pass: usize,
    stub: usize,
    fail: usize,
    timeout: usize,
    drift: &'a [String],
}

/// Default sidecar path: `$CARGO_TARGET_TMPDIR/cpython_lib_test_summary.json`.
/// Override with `MAMBA_CPYTHON_LIB_TEST_SUMMARY_PATH`.
fn summary_json_path() -> PathBuf {
    if let Ok(p) = std::env::var("MAMBA_CPYTHON_LIB_TEST_SUMMARY_PATH") {
        return PathBuf::from(p);
    }
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cpython_lib_test_summary.json")
}

fn write_summary_json(s: SummarySnapshot<'_>) {
    write_summary_json_to(&summary_json_path(), s);
}

fn write_summary_json_to(path: &Path, s: SummarySnapshot<'_>) {
    let mvp_status = if s.mvp_pass == 0 { "PENDING" } else { "OK" };
    let mut body = String::with_capacity(512 + s.drift.len() * 80);
    body.push_str("{\n");
    body.push_str("  \"schema_version\": 2,\n");
    body.push_str(&format!("  \"harness_kind\": \"{}\",\n", s.harness_kind));
    body.push_str(&format!("  \"mvp_status\": \"{mvp_status}\",\n"));
    body.push_str(&format!("  \"total\": {},\n", s.total));
    body.push_str(&format!("  \"mvp_pass\": {},\n", s.mvp_pass));
    body.push_str(&format!("  \"import_pass\": {},\n", s.import_pass));
    body.push_str(&format!("  \"assertion_pass\": {},\n", s.assertion_pass));
    body.push_str(&format!("  \"stub\": {},\n", s.stub));
    body.push_str(&format!("  \"fail\": {},\n", s.fail));
    body.push_str(&format!("  \"timeout\": {},\n", s.timeout));
    body.push_str(&format!("  \"drift_count\": {},\n", s.drift.len()));
    body.push_str("  \"drift_entries\": [");
    for (i, entry) in s.drift.iter().enumerate() {
        if i > 0 {
            body.push_str(", ");
        }
        let trimmed = entry.trim_start_matches("  - ").trim();
        let mut escaped = String::with_capacity(trimmed.len() + 2);
        escaped.push('"');
        for c in trimmed.chars() {
            match c {
                '\\' => escaped.push_str("\\\\"),
                '"' => escaped.push_str("\\\""),
                '\n' => escaped.push_str("\\n"),
                _ => escaped.push(c),
            }
        }
        escaped.push('"');
        body.push_str(&escaped);
    }
    body.push_str("]\n");
    body.push_str("}\n");

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(path, &body) {
        eprintln!(
            "[cpython_lib_test] WARNING: could not write summary JSON to \
             {}: {e}",
            path.display(),
        );
    } else {
        eprintln!("[cpython_lib_test] summary_json: {}", path.display());
    }
}

#[test]
fn outcome_debug_uses_canonical_spelling() {
    assert_eq!(format!("{:?}", Outcome::ImportPass), "ImportPass");
    assert_eq!(format!("{:?}", Outcome::AssertionPass), "AssertionPass");
    assert_eq!(format!("{:?}", Outcome::Stub), "Stub");
    assert_eq!(format!("{:?}", Outcome::Fail), "Fail");
    assert_eq!(format!("{:?}", Outcome::Timeout), "Timeout");
}

#[test]
fn discover_seeds_rejects_duplicate_stems_across_contract_dirs() {
    let tmpdir =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cpython_lib_test_discover_duplicate_stem");
    let _ = std::fs::remove_dir_all(&tmpdir);
    std::fs::create_dir_all(tmpdir.join("pass")).expect("mk pass");
    std::fs::create_dir_all(tmpdir.join("fail")).expect("mk fail");
    std::fs::write(tmpdir.join("pass").join("test_foo.py"), b"# pass\n")
        .expect("write pass/test_foo.py");
    std::fs::write(tmpdir.join("fail").join("test_foo.py"), b"# fail\n")
        .expect("write fail/test_foo.py");

    let dirs: &[(&str, Outcome)] = &[("pass", Outcome::AssertionPass), ("fail", Outcome::Fail)];
    let err = discover_seeds(&tmpdir, dirs)
        .expect_err("expected discover_seeds to error on duplicate stem");
    assert!(
        err.contains("duplicate stem"),
        "error should mention 'duplicate stem'; got: {err}",
    );
    assert!(
        err.contains("test_foo"),
        "error should name the duplicate stem; got: {err}",
    );
    assert!(
        err.contains("pass") && err.contains("fail"),
        "error should name both colliding dirs; got: {err}",
    );
}

#[test]
fn discover_seeds_returns_unique_stems_with_expected_outcomes() {
    let tmpdir =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cpython_lib_test_discover_unique_stems");
    let _ = std::fs::remove_dir_all(&tmpdir);
    std::fs::create_dir_all(tmpdir.join("pass")).expect("mk pass");
    std::fs::create_dir_all(tmpdir.join("spec")).expect("mk spec");
    std::fs::write(tmpdir.join("pass").join("test_a.py"), b"# a\n").expect("a");
    std::fs::write(tmpdir.join("spec").join("test_b.py"), b"# b\n").expect("b");

    let dirs: &[(&str, Outcome)] = &[("pass", Outcome::AssertionPass), ("spec", Outcome::Fail)];
    let entries = discover_seeds(&tmpdir, dirs).expect("unique stems must discover ok");
    let mut got: Vec<(String, Outcome, String)> = entries
        .iter()
        .map(|e| {
            (
                e.subdir.clone(),
                e.expected,
                e.path.file_stem().unwrap().to_string_lossy().into_owned(),
            )
        })
        .collect();
    got.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        got,
        vec![
            (
                "pass".to_string(),
                Outcome::AssertionPass,
                "test_a".to_string()
            ),
            ("spec".to_string(), Outcome::Fail, "test_b".to_string()),
        ],
    );
}

#[test]
fn discover_seeds_skips_missing_contract_dirs() {
    let tmpdir =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cpython_lib_test_discover_missing_dir");
    let _ = std::fs::remove_dir_all(&tmpdir);
    std::fs::create_dir_all(tmpdir.join("pass")).expect("mk pass");
    std::fs::write(tmpdir.join("pass").join("test_p.py"), b"# p\n").expect("p");
    // `spec/` deliberately NOT created.

    let dirs: &[(&str, Outcome)] = &[("pass", Outcome::AssertionPass), ("spec", Outcome::Fail)];
    let entries =
        discover_seeds(&tmpdir, dirs).expect("missing contract dirs must be skipped, not error");
    assert_eq!(entries.len(), 1, "only the present dir should contribute");
    assert_eq!(entries[0].subdir, "pass");
}

#[test]
fn summary_json_schema_is_stable_and_parseable() {
    let tmpdir =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cpython_lib_test_summary_schema_smoke");
    std::fs::create_dir_all(&tmpdir).expect("mktmpdir");
    let out_path = tmpdir.join("summary.json");
    let drift_lines = [
        "  - spec/test_synth: expected Fail, got AssertionPass — promote: `git mv spec/test_synth.py pass/test_synth.py`".to_string(),
    ];
    write_summary_json_to(
        &out_path,
        SummarySnapshot {
            harness_kind: "runtime",
            total: 58,
            mvp_pass: 0,
            import_pass: 7,
            assertion_pass: 0,
            stub: 32,
            fail: 19,
            timeout: 0,
            drift: &drift_lines,
        },
    );

    let body = std::fs::read_to_string(&out_path).expect("summary written");
    for required_key in [
        "\"schema_version\": 2",
        "\"harness_kind\": \"runtime\"",
        "\"mvp_status\": \"PENDING\"",
        "\"total\": 58",
        "\"mvp_pass\": 0",
        "\"import_pass\": 7",
        "\"assertion_pass\": 0",
        "\"stub\": 32",
        "\"fail\": 19",
        "\"timeout\": 0",
        "\"drift_count\": 1",
        "\"drift_entries\":",
    ] {
        assert!(
            body.contains(required_key),
            "summary JSON missing required key/value `{required_key}`; got:\n{body}"
        );
    }
    assert!(
        body.contains("spec/test_synth: expected Fail"),
        "drift entry text missing or mis-escaped; got:\n{body}",
    );
}

#[test]
fn summary_json_mvp_status_flips_to_ok_when_assertion_pass_present() {
    let tmpdir =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cpython_lib_test_summary_mvp_ok_smoke");
    std::fs::create_dir_all(&tmpdir).expect("mktmpdir");
    let out_path = tmpdir.join("summary.json");
    write_summary_json_to(
        &out_path,
        SummarySnapshot {
            harness_kind: "runtime",
            total: 1,
            mvp_pass: 1,
            import_pass: 0,
            assertion_pass: 1,
            stub: 0,
            fail: 0,
            timeout: 0,
            drift: &[],
        },
    );

    let body = std::fs::read_to_string(&out_path).expect("summary written");
    assert!(
        body.contains("\"mvp_status\": \"OK\""),
        "mvp_status must flip to OK when assertion_pass >= 1; got:\n{body}",
    );
    assert!(body.contains("\"assertion_pass\": 1"));
    assert!(body.contains("\"drift_count\": 0"));
    assert!(body.contains("\"drift_entries\": []"));
}

#[test]
fn summary_json_carries_runtime_harness_kind_and_schema_v2() {
    let tmpdir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("cpython_lib_test_summary_harness_kind_smoke");
    std::fs::create_dir_all(&tmpdir).expect("mktmpdir");
    let out_path = tmpdir.join("summary.json");
    write_summary_json_to(
        &out_path,
        SummarySnapshot {
            harness_kind: "runtime",
            total: 0,
            mvp_pass: 0,
            import_pass: 0,
            assertion_pass: 0,
            stub: 0,
            fail: 0,
            timeout: 0,
            drift: &[],
        },
    );

    let body = std::fs::read_to_string(&out_path).expect("summary written");
    assert!(
        body.contains("\"schema_version\": 2"),
        "schema_version must be 2 after #2546 harness_kind addition; got:\n{body}",
    );
    assert!(
        body.contains("\"harness_kind\": \"runtime\""),
        "harness_kind must be \"runtime\" for the runtime harness summary; got:\n{body}",
    );
    let kind_pos = body
        .find("\"harness_kind\"")
        .expect("harness_kind key present");
    let mvp_pos = body.find("\"mvp_pass\"").expect("mvp_pass key present");
    assert!(
        kind_pos < mvp_pos,
        "harness_kind must precede mvp_pass in the summary JSON; got:\n{body}",
    );
}

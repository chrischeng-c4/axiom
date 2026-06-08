//! Real-world conformance fixture runner.
//!
//! Convention: see `tests/harness/cpython/conventions/REAL-WORLD-CONVENTION.md`.
//!
//! Walks every `tests/cpython/fixtures/{std-libs,3rd-libs}/*/real_world/*.py`
//! fixture and shells out twice per file:
//!   1. `python3 <file>`   — must exit 0 (skipped if python3 missing).
//!   2. `mamba <file>`     — must exit 0.
//!
//! Marked `#[ignore]` so the default `cargo test` run stays fast; opt in
//! with `cargo test -p mamba --test conformance_real_world -- --ignored`.
//!
//! #2550: fixtures are split into **required** and **optional** buckets
//! using the manifest at `projects/mamba/ecosystem_fixture_manifest.toml`
//! (which #2551 introduced). The MVP gate fails on a `required_fail`,
//! never on `optional_fail`. Both categories are reported on stderr so
//! a worker can still see the optional-bucket health without having it
//! gate MVP green.
//!
//! #2555: required fixtures further split by `expected_outcome` —
//! `pass` (must exit 0), `xfail` (expected to fail under mamba; never
//! gates MVP; unexpected pass surfaces as a graduate-to-pass warning),
//! or `skip` (structurally unrunnable, e.g. needs a C-extension mamba
//! can't load yet). pass / fail / xfail / xpass / skip counts are
//! reported separately; required_pass excludes xfail and skip.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// #2555 — per-fixture expected outcome from the manifest. Drives the
/// runner's pass / fail / xfail / skip bucketing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExpectedOutcome {
    /// Must exit 0 under both interpreters. Counts toward required pass.
    Pass,
    /// Expected to fail under mamba today. Failure does not gate MVP;
    /// unexpected pass surfaces as a warning ("graduate to pass").
    Xfail,
    /// Structurally unrunnable in the default gate (e.g. needs a
    /// C-extension mamba can't load yet). Never executed.
    Skip,
}

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("conformance")
}

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ecosystem_fixture_manifest.toml")
}

/// #2555 — load `relpath → (ExpectedOutcome, blocker)` from the manifest.
/// The shape is validated by the dedicated smoke gate
/// (`ecosystem_fixture_manifest_smoke.rs`) — here we read the fields we
/// need at runtime and treat an unreadable / unparseable manifest as
/// "no required fixtures" so a broken manifest cannot silently promote
/// a regression.
fn load_manifest_outcomes() -> HashMap<String, (ExpectedOutcome, Option<String>)> {
    let raw = match std::fs::read_to_string(manifest_path()) {
        Ok(s) => s,
        Err(_) => return HashMap::new(),
    };
    let doc: toml::Value = match raw.parse() {
        Ok(v) => v,
        Err(_) => return HashMap::new(),
    };

    let mut out = HashMap::new();
    if let Some(fixtures) = doc.get("fixtures").and_then(|v| v.as_table()) {
        for entry in fixtures.values() {
            let Some(table) = entry.as_table() else {
                continue;
            };
            let Some(relpath) = table.get("relpath").and_then(|v| v.as_str()) else {
                continue;
            };
            let outcome = match table.get("expected_outcome").and_then(|v| v.as_str()) {
                Some("pass") => ExpectedOutcome::Pass,
                Some("xfail") => ExpectedOutcome::Xfail,
                Some("skip") => ExpectedOutcome::Skip,
                _ => ExpectedOutcome::Pass, // smoke gate rejects others
            };
            let blocker = table
                .get("blocker")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            out.insert(relpath.to_string(), (outcome, blocker));
        }
    }
    out
}

/// Back-compat helper for the `#[ignore]`-d runner: just the relpath set.
fn load_required_relpaths() -> HashSet<String> {
    load_manifest_outcomes().into_keys().collect()
}

fn collect_real_world_scripts(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for bucket in ["std-libs", "3rd-libs"] {
        let bucket_dir = root.join(bucket);
        let Ok(libs) = std::fs::read_dir(&bucket_dir) else {
            continue;
        };
        for lib_entry in libs.flatten() {
            let real_world = lib_entry.path().join("real_world");
            let Ok(scripts) = std::fs::read_dir(&real_world) else {
                continue;
            };
            for script_entry in scripts.flatten() {
                let p = script_entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("py") {
                    out.push(p);
                }
            }
        }
    }
    out.sort();
    out
}

fn python3_available() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
#[ignore = "real-world conformance gate; opt in with --ignored"]
fn real_world_fixtures_pass_under_both_interpreters() {
    let root = fixtures_root();
    let scripts = collect_real_world_scripts(&root);
    assert!(
        !scripts.is_empty(),
        "no real-world fixtures found under {}",
        root.display()
    );

    let outcomes = load_manifest_outcomes();
    let has_python3 = python3_available();
    let mamba = mamba_bin();

    // #2550: required vs optional. Optional fixtures are anything not in
    // the manifest at all — failures are advisory.
    // #2555: required fixtures further split by expected_outcome:
    //   - pass  → must exit 0; failure fails MVP
    //   - xfail → failure is silent (expected); unexpected pass surfaces
    //     as a warning. Never gates MVP.
    //   - skip  → not executed at all; never gates MVP.
    let mut required_failures: Vec<String> = Vec::new();
    let mut optional_failures: Vec<String> = Vec::new();
    let mut xfail_observed: Vec<String> = Vec::new();
    let mut xpass_warnings: Vec<String> = Vec::new();
    let mut skip_observed: Vec<String> = Vec::new();
    let mut required_pass = 0usize;
    let mut optional_pass = 0usize;

    for script in &scripts {
        let rel = script.strip_prefix(&root).unwrap_or(script);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let manifest_entry = outcomes.get(&rel_str);
        let outcome = manifest_entry.map(|(o, _)| *o);
        let blocker = manifest_entry.and_then(|(_, b)| b.clone());
        let is_required = manifest_entry.is_some();

        // Skip path — never invoke the interpreters.
        if outcome == Some(ExpectedOutcome::Skip) {
            skip_observed.push(format!(
                "[skip] {} (blocker: {})",
                rel.display(),
                blocker.as_deref().unwrap_or("(none)"),
            ));
            continue;
        }

        let category = if is_required { "required" } else { "optional" };

        if has_python3 {
            let out = Command::new("python3")
                .arg(script)
                .output()
                .expect("failed to spawn python3");
            if !out.status.success() {
                let msg = format!(
                    "[{category}] python3 {}: exit {}\nstdout: {}\nstderr: {}",
                    rel.display(),
                    out.status,
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr),
                );
                if outcome == Some(ExpectedOutcome::Xfail) {
                    xfail_observed.push(format!(
                        "[xfail] {} (python3 failed; blocker: {})",
                        rel.display(),
                        blocker.as_deref().unwrap_or("(none)"),
                    ));
                } else if is_required {
                    required_failures.push(msg);
                } else {
                    optional_failures.push(msg);
                }
                continue;
            }
        }

        let out = Command::new(&mamba)
            .arg("run")
            .arg(script)
            .output()
            .expect("failed to spawn mamba");
        let mamba_passed = out.status.success();

        match outcome {
            Some(ExpectedOutcome::Xfail) => {
                if mamba_passed {
                    xpass_warnings.push(format!(
                        "[xpass] {} passed unexpectedly under mamba — \
                         consider graduating to expected_outcome=\"pass\" \
                         (blocker was: {})",
                        rel.display(),
                        blocker.as_deref().unwrap_or("(none)"),
                    ));
                } else {
                    xfail_observed.push(format!(
                        "[xfail] {} (blocker: {})",
                        rel.display(),
                        blocker.as_deref().unwrap_or("(none)"),
                    ));
                }
            }
            Some(ExpectedOutcome::Pass) => {
                if mamba_passed {
                    required_pass += 1;
                } else {
                    required_failures.push(format!(
                        "[{category}] mamba run {}: exit {}\nstdout: {}\nstderr: {}",
                        rel.display(),
                        out.status,
                        String::from_utf8_lossy(&out.stdout),
                        String::from_utf8_lossy(&out.stderr),
                    ));
                }
            }
            Some(ExpectedOutcome::Skip) => unreachable!("skip handled above"),
            None => {
                if mamba_passed {
                    optional_pass += 1;
                } else {
                    optional_failures.push(format!(
                        "[{category}] mamba run {}: exit {}\nstdout: {}\nstderr: {}",
                        rel.display(),
                        out.status,
                        String::from_utf8_lossy(&out.stdout),
                        String::from_utf8_lossy(&out.stderr),
                    ));
                }
            }
        }
    }

    let required_fail = required_failures.len();
    let optional_fail = optional_failures.len();
    let xfail_count = xfail_observed.len();
    let xpass_count = xpass_warnings.len();
    let skip_count = skip_observed.len();

    // #2555 acceptance: pass / fail / xfail / skip reported separately;
    // required_pass excludes xfail and skip by construction (Xfail and
    // Skip branches never increment `required_pass`).
    eprintln!(
        "[conformance_real_world] summary: required_pass={required_pass} \
         required_fail={required_fail} optional_pass={optional_pass} \
         optional_fail={optional_fail} xfail={xfail_count} \
         xpass={xpass_count} skip={skip_count}"
    );

    if !xfail_observed.is_empty() {
        eprintln!(
            "[conformance_real_world] {xfail_count} xfail fixture(s) \
             observed (do NOT gate MVP):\n{}",
            xfail_observed.join("\n"),
        );
    }
    if !xpass_warnings.is_empty() {
        eprintln!(
            "[conformance_real_world] {xpass_count} xfail fixture(s) \
             passed unexpectedly — graduate to expected_outcome=\"pass\":\n{}",
            xpass_warnings.join("\n"),
        );
    }
    if !skip_observed.is_empty() {
        eprintln!(
            "[conformance_real_world] {skip_count} skipped fixture(s) \
             (structurally unrunnable; do NOT gate MVP):\n{}",
            skip_observed.join("\n"),
        );
    }
    if optional_fail > 0 {
        eprintln!(
            "[conformance_real_world] {optional_fail} optional fixture \
             failure(s) (advisory, do NOT fail the MVP gate):\n{}",
            optional_failures.join("\n---\n"),
        );
    }

    assert!(
        required_fail == 0,
        "{required_fail} required real-world fixture failure(s) — \
         xfail / skip / optional failures are advisory and do not \
         appear here:\n{}",
        required_failures.join("\n---\n"),
    );
}

/// #2550 cheap unit gate. The runner above is `#[ignore]`-d so the
/// required/optional split is otherwise unexercised in the default
/// test run. This test re-checks two invariants that the runner
/// itself relies on:
///
///   1. The manifest is loadable and the required set is non-empty
///      (a broken or empty manifest would collapse every fixture into
///      "optional" and silently let regressions through).
///   2. Every fixture stem listed in the manifest exists under
///      `tests/cpython/`. The dedicated smoke gate in
///      `ecosystem_fixture_manifest_smoke.rs` already enforces this
///      from the other direction; doubling up here means the
///      `--ignored` runner cannot silently bucket-miss a required
///      fixture because the manifest had a typo'd relpath.
#[test]
fn real_world_runner_required_set_is_loadable_and_resolvable() {
    let required = load_required_relpaths();
    assert!(
        !required.is_empty(),
        "ecosystem_fixture_manifest.toml produced an empty required set \
         — the runner's required/optional bucket logic would silently \
         classify every fixture as optional, hiding regressions"
    );

    let root = fixtures_root();
    let mut missing = Vec::new();
    for relpath in &required {
        let abs = root.join(relpath);
        if !abs.exists() {
            missing.push(format!(
                "  - {relpath}: manifest lists a required fixture that \
                 does not exist on disk (resolved {})",
                abs.display(),
            ));
        }
    }
    assert!(
        missing.is_empty(),
        "{} required manifest entr{} reference missing files:\n{}",
        missing.len(),
        if missing.len() == 1 { "y" } else { "ies" },
        missing.join("\n"),
    );
}

/// #2555 cheap unit gate. The `--ignored` runner above is otherwise
/// unexercised in the default test run, so we double-check here that
/// the outcome loader behaves correctly:
///
///   1. Every manifest entry decodes to one of Pass / Xfail / Skip.
///   2. Non-pass entries always carry a non-empty blocker (this also
///      lets the runner echo the blocker into the failure report,
///      satisfying "summary names the blocker").
///   3. The `Pass` set is non-empty (sanity: a manifest of pure xfail
///      / skip would mean MVP green requires nothing, which is wrong).
#[test]
fn real_world_runner_outcomes_load_with_blockers() {
    let outcomes = load_manifest_outcomes();
    assert!(
        !outcomes.is_empty(),
        "manifest produced empty outcome map — runner has nothing to gate on"
    );

    let mut pass_count = 0usize;
    let mut violations: Vec<String> = Vec::new();
    for (relpath, (outcome, blocker)) in &outcomes {
        match outcome {
            ExpectedOutcome::Pass => {
                pass_count += 1;
                if blocker.is_some() {
                    violations.push(format!(
                        "  - {relpath}: outcome=pass must not carry a blocker"
                    ));
                }
            }
            ExpectedOutcome::Xfail | ExpectedOutcome::Skip => {
                if blocker.as_deref().unwrap_or("").is_empty() {
                    violations.push(format!(
                        "  - {relpath}: outcome={outcome:?} requires a \
                         non-empty blocker so the report can name the cause"
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} outcome/blocker violation(s):\n{}",
        violations.len(),
        violations.join("\n"),
    );
    assert!(
        pass_count > 0,
        "manifest has zero pass-required fixtures — MVP gate would be vacuous"
    );
}

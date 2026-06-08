// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-gate-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests — T1..T10 per issue #2144.

use std::path::{Path, PathBuf};

use chrono::{TimeZone, Utc};
use jet_parity_gate::gate::{
    run_gate, EXIT_BLOCKING_FAIL, EXIT_PASS, EXIT_SKIPPED, EXIT_SOFT_FAIL,
};
use jet_parity_gate::init::run_init;
use jet_parity_gate::manifest::GatingManifest;
use jet_parity_gate::result::{ChannelResult, DiffKind, Status};
use jet_parity_gate::waivers::Waivers;

fn repo_root() -> PathBuf {
    // crate dir is projects/jet/parity-gate/, repo root is three levels up.
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn default_manifest_path() -> PathBuf {
    repo_root().join("projects/jet/parity/data/parity-gating.toml")
}

fn default_waivers_path() -> PathBuf {
    repo_root().join("projects/jet/parity/data/waivers.toml")
}

fn sample_result(fixture: &str, channel: &str, status: Status) -> ChannelResult {
    ChannelResult {
        schema_version: 1,
        fixture_id: fixture.to_string(),
        channel: channel.to_string(),
        status,
        diff_kind: DiffKind::PixelL2,
        diff_value: 0.0,
        captured_at: Utc.with_ymd_and_hms(2026, 5, 16, 0, 0, 0).unwrap(),
        waived_by: None,
    }
}

// T1 — parse default parity-gating.toml.
#[test]
fn t1_parses_default_manifest() {
    let m = GatingManifest::parse(default_manifest_path()).expect("default manifest parses");
    assert_eq!(m.channels.len(), 5);
    assert!(m.channels.iter().any(|c| c == "pixel"));
    assert_eq!(m.tolerance.pixel_delta, 8);
    assert!((m.tolerance.a11y_diff_ratio - 0.05).abs() < 1e-9);
    assert_eq!(m.adapter.id, "mui-react-dom");
    assert!(!m.blocking);
    assert!(m.allow_waivers);
}

// T2 — manifest rejects unknown channel.
#[test]
fn t2_rejects_unknown_channel() {
    let toml = r#"
channels = ["pixel", "bogus-channel"]
blocking = false
allow_waivers = true
[tolerance]
pixel_delta = 8
a11y_diff_ratio = 0.05
[adapter]
id = "mui-react-dom"
"#;
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("m.toml");
    std::fs::write(&p, toml).unwrap();
    let err = GatingManifest::parse(&p).expect_err("should reject");
    let msg = format!("{err}");
    assert!(msg.contains("bogus-channel"), "msg={msg}");
}

// T3 — manifest rejects pixel_delta = 300.
#[test]
fn t3_rejects_pixel_delta_out_of_range() {
    let toml = r#"
channels = ["pixel"]
blocking = false
allow_waivers = true
[tolerance]
pixel_delta = 300
a11y_diff_ratio = 0.05
[adapter]
id = "mui-react-dom"
"#;
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("m.toml");
    std::fs::write(&p, toml).unwrap();
    let err = GatingManifest::parse(&p).expect_err("should reject");
    let msg = format!("{err}");
    assert!(msg.contains("pixel_delta"), "msg={msg}");
}

// T4 — empty waivers parses.
#[test]
fn t4_empty_waivers_parses() {
    let w = Waivers::parse(default_waivers_path()).expect("waivers parse");
    assert!(w.waivers.is_empty());
}

// T5 — expired waiver does not apply.
#[test]
fn t5_expired_waiver_does_not_apply() {
    let toml = r#"
[[waivers]]
fixture_id = "mui-button-default"
channel    = "pixel"
expires_on = "2025-01-01"
reason     = "expired"

[[waivers]]
fixture_id = "mui-button-default"
channel    = "ax-tree"
expires_on = "2099-01-01"
reason     = "live"
"#;
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("w.toml");
    std::fs::write(&p, toml).unwrap();
    let w = Waivers::parse(&p).unwrap();
    let now = Utc.with_ymd_and_hms(2026, 5, 16, 0, 0, 0).unwrap();
    assert!(w.applies_to("mui-button-default", "pixel", now).is_none());
    assert!(w.applies_to("mui-button-default", "ax-tree", now).is_some());
}

fn manifest_with(blocking: bool) -> GatingManifest {
    let toml = format!(
        r#"
channels = ["pixel", "ax-tree", "focus-order", "pointer-hit-map", "ime-trace"]
blocking = {blocking}
allow_waivers = true
[tolerance]
pixel_delta = 8
a11y_diff_ratio = 0.05
[adapter]
id = "mui-react-dom"
"#
    );
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("m.toml");
    std::fs::write(&p, toml).unwrap();
    GatingManifest::parse(&p).unwrap()
}

// T6 — gate over all-pass results returns exit 0.
#[test]
fn t6_all_pass_returns_zero() {
    let m = manifest_with(false);
    let results = vec![
        sample_result("f1", "pixel", Status::Pass),
        sample_result("f1", "ax-tree", Status::Pass),
    ];
    let now = Utc.with_ymd_and_hms(2026, 5, 16, 0, 0, 0).unwrap();
    let report = run_gate(&m, &results, &Waivers::default(), now);
    assert_eq!(report.exit_code, EXIT_PASS);
    assert_eq!(report.pass, 2);
}

// T7 — blocking + one fail → exit 1.
#[test]
fn t7_blocking_fail_returns_one() {
    let m = manifest_with(true);
    let results = vec![
        sample_result("f1", "pixel", Status::Pass),
        sample_result("f2", "pixel", Status::Fail),
    ];
    let now = Utc.with_ymd_and_hms(2026, 5, 16, 0, 0, 0).unwrap();
    let report = run_gate(&m, &results, &Waivers::default(), now);
    assert_eq!(report.exit_code, EXIT_BLOCKING_FAIL);
    assert_eq!(report.fail, 1);
}

// T8 — non-blocking + one fail → exit 2.
#[test]
fn t8_soft_fail_returns_two() {
    let m = manifest_with(false);
    let results = vec![sample_result("f1", "pixel", Status::Fail)];
    let now = Utc.with_ymd_and_hms(2026, 5, 16, 0, 0, 0).unwrap();
    let report = run_gate(&m, &results, &Waivers::default(), now);
    assert_eq!(report.exit_code, EXIT_SOFT_FAIL);
    assert_eq!(report.fail, 1);
}

// T9 — no results → exit 77.
#[test]
fn t9_no_results_returns_skipped() {
    let m = manifest_with(false);
    let now = Utc.with_ymd_and_hms(2026, 5, 16, 0, 0, 0).unwrap();
    let report = run_gate(&m, &[], &Waivers::default(), now);
    assert_eq!(report.exit_code, EXIT_SKIPPED);
    assert_eq!(report.total, 0);
}

// T10 — init writes three files; second run without --force errors.
#[test]
fn t10_init_scaffold() {
    let dir = tempfile::tempdir().unwrap();
    let r = run_init(dir.path(), false).expect("first run ok");
    assert_eq!(r.written.len(), 3);
    assert!(dir.path().join("parity-gating.toml").exists());
    assert!(dir.path().join("waivers.toml").exists());
    assert!(dir.path().join("docs/gating-manifest.md").exists());

    // second run without --force errors
    let err = run_init(dir.path(), false).expect_err("should refuse");
    let msg = format!("{err}");
    assert!(msg.contains("force"), "msg={msg}");

    // with --force, succeeds again
    run_init(dir.path(), true).expect("force overwrite ok");
}

// Bonus — exercise ChannelResult::parse_dir round-trip so the read
// path is covered by the integration suite.
#[test]
fn parse_dir_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let row = sample_result("f1", "pixel", Status::Pass);
    let p = dir.path().join("f1.pixel.channel-result.json");
    std::fs::write(&p, serde_json::to_string(&row).unwrap()).unwrap();
    let rows = ChannelResult::parse_dir(dir.path()).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].fixture_id, "f1");
}
// CODEGEN-END

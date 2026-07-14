// HANDWRITE-BEGIN gap="missing-generator:unit-test:36f27606" tracker="#1597" reason="Verify the hermetic benchmark profile and provide an opt-in live smoke test."
use std::process::Command;

use serde_json::Value;

const SCRIPT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/benchmarks/pgbouncer-transaction-pooling/run.sh"
);
const RUNNER_SOURCE: &str = include_str!("../benchmarks/pgbouncer-transaction-pooling/run.sh");
const BENCHMARK_README: &str =
    include_str!("../benchmarks/pgbouncer-transaction-pooling/README.md");

fn command(args: &[&str]) -> std::process::Output {
    Command::new("bash")
        .arg(SCRIPT)
        .args(args)
        .output()
        .expect("benchmark script should start")
}

fn dry_run_profile() -> Value {
    let output = command(&["--dry-run"]);
    assert!(
        output.status.success(),
        "dry run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("dry run should emit JSON")
}

fn select_only_dry_run_profile() -> Value {
    let output = command(&["--dry-run", "--workload", "select-only"]);
    assert!(
        output.status.success(),
        "select-only dry run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("select-only dry run should emit JSON")
}

#[test]
fn dry_run_profile_stays_immutable_when_meter_is_requested() {
    let ordinary = dry_run_profile();
    let output = command(&["--dry-run", "--meter-bin", "/definitely/not/a/meter-binary"]);
    assert!(
        output.status.success(),
        "meter dry run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let with_meter: Value =
        serde_json::from_slice(&output.stdout).expect("meter dry run should emit JSON");
    assert_eq!(
        with_meter, ordinary,
        "meter must not mutate the fixed profile"
    );
}

#[test]
fn dry_run_profile_declares_equal_transaction_pooling_inputs() {
    let profile = dry_run_profile();

    assert_eq!(profile["schema"], "pgpool.pgbouncer-baseline.v2");
    assert_eq!(profile["profile"]["protocol"], "simple");
    assert_eq!(profile["profile"]["pool_mode"], "transaction");
    assert_eq!(profile["profile"]["backend_connection_cap"], 16);
    assert_eq!(profile["profile"]["clients"], 64);
    assert_eq!(profile["profile"]["jobs"], 4);
    assert_eq!(profile["profile"]["duration_seconds"], 30);
    assert_eq!(profile["profile"]["paired_trials"], 2);
    assert_eq!(
        profile["profile"]["orders"],
        serde_json::json!(["pgbouncer-first", "pgpool-first"])
    );
    assert_eq!(
        profile["profile"]["max_pair_ratio_relative_spread"],
        serde_json::json!(0.20)
    );
    assert_eq!(profile["profile"]["pool_acquire_timeout_ms"], 60_000);
    assert_eq!(profile["targets"]["pgbouncer"]["pool_mode"], "transaction");
    assert_eq!(profile["targets"]["pgpool"]["pool_mode"], "transaction");
    assert_eq!(
        profile["targets"]["pgbouncer"]["reset_between_owners"],
        "DISCARD ALL"
    );
    assert_eq!(
        profile["targets"]["pgpool"]["reset_between_owners"],
        "DISCARD ALL"
    );
    assert_eq!(
        profile["targets"]["pgbouncer"]["reset_on_return_to_idle"],
        true
    );
    assert_eq!(
        profile["targets"]["pgpool"]["reset_on_return_to_idle"],
        true
    );
    assert_eq!(
        profile["targets"]["pgbouncer"]["backend_connection_cap"],
        profile["targets"]["pgpool"]["backend_connection_cap"]
    );
    assert_eq!(
        profile["targets"]["pgpool"]["pool_acquire_timeout_ms"],
        profile["profile"]["pool_acquire_timeout_ms"]
    );
}

#[test]
fn select_only_profile_keeps_pooler_inputs_equal_and_explicit() {
    let tpcb = dry_run_profile();
    let select_only = select_only_dry_run_profile();

    assert_eq!(select_only["schema"], tpcb["schema"]);
    assert_eq!(select_only["profile"]["workload"], "pgbench-select-only");
    assert_eq!(tpcb["profile"]["workload"], "pgbench-tpcb");
    for field in [
        "protocol",
        "pool_mode",
        "backend_connection_cap",
        "clients",
        "jobs",
        "duration_seconds",
        "paired_trials",
        "orders",
        "max_pair_ratio_relative_spread",
        "scale",
        "pool_acquire_timeout_ms",
    ] {
        assert_eq!(
            select_only["profile"][field], tpcb["profile"][field],
            "select-only must preserve the fixed pooler input {field}"
        );
    }
    assert_eq!(select_only["targets"], tpcb["targets"]);
}

#[test]
fn ordinary_peer_profile_counterbalances_order_and_documents_peer_verdict_rules() {
    let profile = dry_run_profile();

    assert_eq!(profile["profile"]["paired_trials"], 2);
    assert!(RUNNER_SOURCE.contains("start_pgpool\n\nPGBOUNCER_FIRST_LOG"));
    assert!(RUNNER_SOURCE.contains("PGBOUNCER_FIRST_LOG"));
    assert!(RUNNER_SOURCE.contains("PGPOOL_FIRST_LOG"));
    assert!(RUNNER_SOURCE.contains("pair_winner"));
    assert!(RUNNER_SOURCE.contains("UNANIMOUS_DIRECTION"));
    assert!(RUNNER_SOURCE.contains("PGPOOL_WIN_ELIGIBLE"));
    assert!(RUNNER_SOURCE.contains("comparison_valid"));
    assert!(RUNNER_SOURCE.contains("pgpool_win_eligible"));
    assert!(BENCHMARK_README.contains("counterbalanced paired trials"));
    assert!(BENCHMARK_README.contains("both clean pairs favor PgBouncer"));
    assert!(BENCHMARK_README.contains("pgpool_win_eligible: false"));
}

#[test]
fn pgbouncer_config_forces_discard_all_on_every_transaction_release() {
    assert!(
        RUNNER_SOURCE.contains("server_reset_query = DISCARD ALL"),
        "PgBouncer must configure DISCARD ALL as its reset query"
    );
    assert!(
        RUNNER_SOURCE.contains("server_reset_query_always = 1"),
        "PgBouncer must run its configured reset query on every transaction-pool return"
    );
}

#[test]
fn runner_is_syntax_valid_and_dry_run_is_hermetic() {
    let syntax = Command::new("bash")
        .args(["-n", SCRIPT])
        .output()
        .expect("bash should validate the runner syntax");
    assert!(
        syntax.status.success(),
        "syntax check failed: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let profile = dry_run_profile();
    assert_eq!(profile["profile"]["workload"], "pgbench-tpcb");
}

#[test]
fn live_transaction_pooling_baseline_emits_comparable_metrics_when_enabled() {
    if std::env::var_os("PGPOOL_RUN_PGBOUNCER_BENCH").is_none() {
        eprintln!(
            "skipping live PgBouncer baseline; run with PGPOOL_RUN_PGBOUNCER_BENCH=1 cargo test -p pgpool --test pgbouncer_benchmark"
        );
        return;
    }

    let pgpool_bin = env!("CARGO_BIN_EXE_pgpool");
    let output = command(&["--pgpool-bin", pgpool_bin]);
    assert!(
        output.status.success(),
        "live baseline failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let result: Value =
        serde_json::from_slice(&output.stdout).expect("live baseline should emit JSON");
    assert_eq!(result["schema"], "pgpool.pgbouncer-baseline.v2");
    assert_eq!(result["trials"].as_array().map(Vec::len), Some(2));
    assert_eq!(result["trials"][0]["order"], "pgbouncer-first");
    assert_eq!(result["trials"][1]["order"], "pgpool-first");
    assert!(result["trials"][0]["winner_by_tps"].is_string());
    assert!(result["trials"][1]["winner_by_tps"].is_string());
    assert!(
        result["targets"]["pgbouncer"]["tps"]
            .as_f64()
            .unwrap_or_default()
            > 0.0
    );
    assert!(
        result["targets"]["pgpool"]["tps"]
            .as_f64()
            .unwrap_or_default()
            > 0.0
    );
    assert!(
        result["targets"]["pgbouncer"]["latency_average_ms"]
            .as_f64()
            .unwrap_or_default()
            > 0.0
    );
    assert!(
        result["targets"]["pgpool"]["latency_average_ms"]
            .as_f64()
            .unwrap_or_default()
            > 0.0
    );
    assert!(result["ratios"]["pgpool_over_pgbouncer_tps"]
        .as_f64()
        .is_some());
    assert!(result["comparison_valid"].is_boolean());
    assert!(result["pgpool_win_eligible"].is_boolean());
}
// HANDWRITE-END

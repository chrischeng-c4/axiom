// HANDWRITE-BEGIN gap="missing-generator:unit-test:36f27606" tracker="#1597" reason="Verify the hermetic benchmark profile and provide an opt-in live smoke test."
use std::process::Command;

use serde_json::Value;

const SCRIPT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/benchmarks/pgbouncer-transaction-pooling/run.sh"
);

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

#[test]
fn dry_run_profile_declares_equal_transaction_pooling_inputs() {
    let profile = dry_run_profile();

    assert_eq!(profile["schema"], "pgpool.pgbouncer-baseline.v1");
    assert_eq!(profile["profile"]["protocol"], "simple");
    assert_eq!(profile["profile"]["pool_mode"], "transaction");
    assert_eq!(profile["profile"]["backend_connection_cap"], 16);
    assert_eq!(profile["profile"]["clients"], 64);
    assert_eq!(profile["profile"]["jobs"], 4);
    assert_eq!(profile["profile"]["duration_seconds"], 30);
    assert_eq!(profile["profile"]["pool_acquire_timeout_ms"], 60_000);
    assert_eq!(profile["targets"]["pgbouncer"]["pool_mode"], "transaction");
    assert_eq!(profile["targets"]["pgpool"]["pool_mode"], "transaction");
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
    assert_eq!(result["schema"], "pgpool.pgbouncer-baseline.v1");
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
}
// HANDWRITE-END

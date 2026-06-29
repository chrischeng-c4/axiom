// HANDWRITE-BEGIN gap="missing-generator:unit-test:c5f394f6" tracker="pending-tracker" reason="Smoke-test the sorted_page_deep bench CLI and output fields."
// @spec projects/lumen/tech-design/logic/gate-the-filter-sort-deep-page-chain-bench-cell-pg-competitive-p.md#unit-test
use std::process::Command;

#[test]
fn sorted_page_deep_bench_cli_reports_latency_fields() {
    let bin = env!("CARGO_BIN_EXE_lumen-bench");
    let output = Command::new(bin)
        .args([
            "run",
            "--types",
            "sorted_page_deep",
            "--documents",
            "1000",
            "--page-size",
            "50",
            "--queries",
            "10",
        ])
        .output()
        .expect("run lumen-bench");

    assert!(
        output.status.success(),
        "lumen-bench failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    assert!(stdout.contains("cell=sorted_page_deep"), "{stdout}");
    assert!(stdout.contains("p50_us="), "{stdout}");
    assert!(stdout.contains("p99_us="), "{stdout}");
    assert!(stdout.contains("status=pass"), "{stdout}");
}
// HANDWRITE-END

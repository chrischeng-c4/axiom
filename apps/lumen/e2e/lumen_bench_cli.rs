// CODEGEN-BEGIN
//! ## Contracts inherited from the retired EC shells
//!
//! This sentence was the whole of the `// Contract:` comment in an AW-EC shell under
//! `apps/lumen/e2e/`, which ran `cargo test -p lumen --test lumen_bench_cli` in a
//! subprocess and asserted the child's exit status. `cargo test -p lumen` already runs
//! this target directly, so the shell added a second, nested run and nothing else. It
//! was deleted on 2026-08-20 with the EC machinery it belonged to, and the sentence is
//! the only thing it held that nothing else did. The line below is prefixed with the EC
//! id the shell was filed under.
//!
//! - `lumen-claim-competitor-performance-depth-invariant` — The Lumen-only deep-page
//!   and filter/sort perf gates stay depth-invariant against the retained calibrated
//!   floors without rerunning peer databases by default.

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
// CODEGEN-END

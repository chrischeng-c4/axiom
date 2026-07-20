// HANDWRITE-BEGIN gap="missing-generator:unit-test:defer-cli-interface-efficiency" tracker="#2213" reason="Release-mode offline CLI and exact codegen latency oracle."
use std::process::Command;
use std::time::{Duration, Instant};

const EXPECTED_TS_FILES: [&str; 5] = [
    "client.ts",
    "hooks.ts",
    "index.ts",
    "runtime.ts",
    "types.ts",
];

fn defer() -> Command {
    Command::new(env!("CARGO_BIN_EXE_defer"))
}

fn run_checked(args: &[&str]) -> (Duration, usize) {
    let started = Instant::now();
    let output = defer().args(args).output().unwrap();
    let elapsed = started.elapsed();
    assert!(
        output.status.success(),
        "defer {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.stdout.is_empty(),
        "defer {} emitted empty stdout",
        args.join(" ")
    );
    (elapsed, output.stdout.len())
}

fn generate_checked() -> (Duration, usize) {
    let out = tempfile::tempdir().unwrap();
    let started = Instant::now();
    let output = defer()
        .args([
            "spec",
            "gen",
            "--lang",
            "ts",
            "--out",
            out.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let elapsed = started.elapsed();
    assert!(
        output.status.success(),
        "spec gen failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut files = out
        .path()
        .read_dir()
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    files.sort();
    assert_eq!(files, EXPECTED_TS_FILES);
    let client = std::fs::read_to_string(out.path().join("client.ts")).unwrap();
    for symbol in [
        "createDeferClient",
        "taskCreate(data",
        "taskStatus(data",
        "taskCancel(data",
    ] {
        assert!(client.contains(symbol), "generated client missing {symbol}");
    }
    let generated_bytes = EXPECTED_TS_FILES
        .iter()
        .map(|name| std::fs::metadata(out.path().join(name)).unwrap().len() as usize)
        .sum::<usize>();
    assert!(generated_bytes > 0, "generated TypeScript is empty");
    (elapsed, generated_bytes + output.stdout.len())
}

fn percentile_us(sorted: &[u128], percentile: usize) -> u128 {
    let rank = (sorted.len() * percentile).div_ceil(100).max(1);
    sorted[rank - 1]
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="Own the release-mode non-zero operation count and hard median/p99 CLI efficiency oracle.">
#[test]
#[ignore = "release-mode CLI efficiency gate"]
fn offline_cli_and_codegen_stay_within_latency_ceiling() {
    for args in [
        &["--help"][..],
        &["llm", "--topic", "outline"][..],
        &["spec", "--format", "openapi"][..],
    ] {
        run_checked(args);
    }
    generate_checked();

    let mut elapsed_us = Vec::with_capacity(20);
    let mut output_bytes = 0usize;
    for _ in 0..5 {
        for args in [
            &["--help"][..],
            &["llm", "--topic", "outline"][..],
            &["spec", "--format", "openapi"][..],
        ] {
            let (elapsed, bytes) = run_checked(args);
            elapsed_us.push(elapsed.as_micros());
            output_bytes += bytes;
        }
        let (elapsed, bytes) = generate_checked();
        elapsed_us.push(elapsed.as_micros());
        output_bytes += bytes;
    }

    elapsed_us.sort_unstable();
    let median = percentile_us(&elapsed_us, 50);
    let p95 = percentile_us(&elapsed_us, 95);
    let p99 = percentile_us(&elapsed_us, 99);
    println!(
        "defer_cli_efficiency operations={} output_bytes={} median_ms={:.3} p95_ms={:.3} p99_ms={:.3} errors=0",
        elapsed_us.len(),
        output_bytes,
        median as f64 / 1_000.0,
        p95 as f64 / 1_000.0,
        p99 as f64 / 1_000.0
    );
    assert_eq!(elapsed_us.len(), 20);
    assert!(output_bytes > 0);
    assert!(
        median <= 250_000,
        "median {:.3}ms exceeds 250ms",
        median as f64 / 1_000.0
    );
    assert!(
        p99 <= 750_000,
        "p99 {:.3}ms exceeds 750ms",
        p99 as f64 / 1_000.0
    );
}
// </HANDWRITE>
// HANDWRITE-END

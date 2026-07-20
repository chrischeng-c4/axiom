// HANDWRITE-BEGIN gap="missing-generator:unit-test:defer-cli-interface-stability" tracker="#2213" reason="Repeated offline CLI determinism, cleanup, and FD plateau oracle."
use std::collections::BTreeMap;
use std::process::Command;
use std::time::{Duration, Instant};

const ROUNDS: usize = 64;
const CODEGEN_ROUNDS: usize = 16;
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

fn run_checked(args: &[&str]) -> Vec<u8> {
    let output = defer().args(args).output().unwrap();
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
    output.stdout
}

fn generate_snapshot() -> BTreeMap<String, Vec<u8>> {
    let out = tempfile::tempdir().unwrap();
    let path = out.path().to_path_buf();
    let output = defer()
        .args([
            "spec",
            "gen",
            "--lang",
            "ts",
            "--out",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "spec gen failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut files = path
        .read_dir()
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    files.sort();
    assert_eq!(files, EXPECTED_TS_FILES);
    let snapshot = EXPECTED_TS_FILES
        .iter()
        .map(|name| ((*name).to_string(), std::fs::read(path.join(name)).unwrap()))
        .collect();
    drop(out);
    assert!(!path.exists(), "temporary codegen directory leaked");
    snapshot
}

fn fd_count() -> Option<usize> {
    ["/proc/self/fd", "/dev/fd"]
        .into_iter()
        .find_map(|path| std::fs::read_dir(path).ok().map(|entries| entries.count()))
}

#[test]
#[ignore = "release-mode repeated CLI stability gate"]
fn offline_cli_is_deterministic_and_resource_bounded() {
    let commands: &[&[&str]] = &[
        &["--help"],
        &["llm", "--topic", "outline"],
        &["spec", "--format", "openapi"],
        &["dockerfile", "render", "--variant", "source"],
    ];
    let baselines = commands
        .iter()
        .map(|args| run_checked(args))
        .collect::<Vec<_>>();
    let codegen_baseline = generate_snapshot();
    let fd_before = fd_count();
    let started = Instant::now();

    let mut operations = 0usize;
    for _ in 0..ROUNDS {
        for (args, baseline) in commands.iter().zip(&baselines) {
            assert_eq!(run_checked(args), *baseline, "nondeterministic CLI stdout");
            operations += 1;
        }
    }
    for _ in 0..CODEGEN_ROUNDS {
        assert_eq!(generate_snapshot(), codegen_baseline);
    }

    let elapsed = started.elapsed();
    let fd_after = fd_count();
    println!(
        "defer_cli_stability rounds={} operations={} codegen_rounds={} elapsed_ms={} fd_before={:?} fd_after={:?} errors=0",
        ROUNDS,
        operations,
        CODEGEN_ROUNDS,
        elapsed.as_millis(),
        fd_before,
        fd_after
    );
    assert_eq!(operations, ROUNDS * commands.len());
    assert!(elapsed <= Duration::from_secs(60));
    if let (Some(before), Some(after)) = (fd_before, fd_after) {
        assert!(
            after <= before + 8,
            "file descriptor growth exceeded 8: {before} -> {after}"
        );
    }
}
// HANDWRITE-END

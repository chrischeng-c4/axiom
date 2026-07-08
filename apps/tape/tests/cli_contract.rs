// SPEC-MANAGED: apps/tape/tech-design/semantic/source/apps-tape-tests-cli-contract-rs.md#unit-test
// <HANDWRITE gap="missing-generator:test:tape-bootstrap" tracker="#768" reason="Initial binary smoke tests for the first Tape service slice.">
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn tape_bin() -> &'static str {
    env!("CARGO_BIN_EXE_tape")
}

#[test]
fn help_ships_standard_and_replay_commands() {
    let output = Command::new(tape_bin())
        .arg("--help")
        .output()
        .expect("run tape --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for needle in [
        "append",
        "replay",
        "checkpoint",
        "spec",
        "llm",
        "upgrade",
        "issue",
    ] {
        assert!(stdout.contains(needle), "help should contain {needle}");
    }
}

#[test]
fn spec_routes_list_topic_contract() {
    let output = Command::new(tape_bin())
        .args(["spec", "--format", "routes"])
        .output()
        .expect("run tape spec");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("/topics/{topic}/append"));
    assert!(stdout.contains("/topics/{topic}/replay"));
    assert!(!stdout.contains("/v1/"));
    assert!(stdout.contains("/checkpoint"));
    assert!(stdout.contains("/healthz"));
    assert!(stdout.contains("/readyz"));
    assert!(stdout.contains("/metrics"));
    assert!(stdout.contains("/openapi.json"));
    assert!(stdout.contains("/docs"));
}

#[test]
fn append_replay_checkpoint_roundtrip() {
    let store = std::env::temp_dir().join(format!(
        "tape-cli-contract-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let append = Command::new(tape_bin())
        .args([
            "append",
            "orders",
            "--payload",
            r#"{"id":"o1"}"#,
            "--timestamp-ms",
            "100",
            "--store",
            store.to_str().unwrap(),
        ])
        .output()
        .expect("run tape append");
    assert!(
        append.status.success(),
        "{}",
        String::from_utf8_lossy(&append.stderr)
    );
    let stdout = String::from_utf8_lossy(&append.stdout);
    assert!(stdout.contains("\"offset\": 0"));
    assert!(stdout.contains("next: tape replay orders --from-offset 0"));

    let replay = Command::new(tape_bin())
        .args([
            "replay",
            "orders",
            "--from-offset",
            "0",
            "--store",
            store.to_str().unwrap(),
        ])
        .output()
        .expect("run tape replay");
    assert!(
        replay.status.success(),
        "{}",
        String::from_utf8_lossy(&replay.stderr)
    );
    let stdout = String::from_utf8_lossy(&replay.stdout);
    assert!(stdout.contains("\"id\": \"o1\""));
    assert!(stdout.contains("next: done"));

    let put = Command::new(tape_bin())
        .args([
            "checkpoint",
            "put",
            "orders",
            "worker-a",
            "--offset",
            "1",
            "--store",
            store.to_str().unwrap(),
        ])
        .output()
        .expect("run checkpoint put");
    assert!(
        put.status.success(),
        "{}",
        String::from_utf8_lossy(&put.stderr)
    );

    let get = Command::new(tape_bin())
        .args([
            "checkpoint",
            "get",
            "orders",
            "worker-a",
            "--store",
            store.to_str().unwrap(),
        ])
        .output()
        .expect("run checkpoint get");
    assert!(
        get.status.success(),
        "{}",
        String::from_utf8_lossy(&get.stderr)
    );
    let stdout = String::from_utf8_lossy(&get.stdout);
    assert!(stdout.contains("\"consumer\": \"worker-a\""));
    assert!(stdout.contains("\"offset\": 1"));
    assert!(stdout.contains("next: done"));

    let _ = std::fs::remove_file(store);
}
// </HANDWRITE>

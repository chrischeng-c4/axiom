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
fn serve_exposes_shared_structured_log_configuration() {
    let output = Command::new(tape_bin())
        .args(["serve", "--help"])
        .output()
        .expect("run tape serve --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--log-format"));
    assert!(stdout.contains("pretty"));
    assert!(stdout.contains("json"));
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
    assert!(stdout.contains("/topics/{topic}/replay/stream"));
    assert!(!stdout.contains("/v1/"));
    assert!(stdout.contains("/checkpoint"));
    assert!(stdout.contains("/healthz"));
    assert!(stdout.contains("/readyz"));
    assert!(stdout.contains("/metrics"));
    assert!(stdout.contains("/openapi.json"));
    assert!(stdout.contains("/docs"));
}

/// #2482: `/topics/{topic}/retention` is served with both GET and PUT
/// (`retention_get`/`retention_put` in `src/server.rs`), so the published
/// route inventory must list both methods, not only the PUT.
#[test]
fn spec_routes_list_retention_get_and_put_methods() {
    let output = Command::new(tape_bin())
        .args(["spec", "--format", "routes"])
        .output()
        .expect("run tape spec routes");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let doc: serde_json::Value = serde_json::from_str(&stdout).expect("routes json parses");
    let methods: Vec<&str> = doc["routes"]
        .as_array()
        .expect("routes is a JSON array")
        .iter()
        .filter(|route| route["path"] == "/topics/{topic}/retention")
        .map(|route| route["method"].as_str().expect("method is a string"))
        .collect();
    assert!(
        methods.contains(&"GET"),
        "route inventory must publish GET /topics/{{topic}}/retention, got {methods:?}"
    );
    assert!(
        methods.contains(&"PUT"),
        "route inventory must publish PUT /topics/{{topic}}/retention, got {methods:?}"
    );
}

#[test]
fn subscription_cli_surface() {
    let help = Command::new(tape_bin())
        .arg("--help")
        .output()
        .expect("run tape --help");
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("subscription"));

    let create_help = Command::new(tape_bin())
        .args(["subscription", "create", "--help"])
        .output()
        .expect("run tape subscription create --help");
    assert!(create_help.status.success());
    let stdout = String::from_utf8_lossy(&create_help.stdout);
    assert!(stdout.contains("--store"));
    assert!(!stdout.contains("--push"));
    assert!(!stdout.contains("--pull"));

    let invalid = Command::new(tape_bin())
        .args([
            "subscription",
            "create",
            "orders",
            "worker-a",
            "--push",
            "https://hooks.example.invalid/events",
        ])
        .output()
        .expect("run invalid subscription create");
    assert!(!invalid.status.success());
}

#[test]
fn subscription_resource_roundtrip() {
    let store = std::env::temp_dir().join(format!(
        "tape-subscription-contract-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let store_arg = store.to_str().unwrap();

    let create = Command::new(tape_bin())
        .args([
            "subscription",
            "create",
            "orders",
            "worker-a",
            "--store",
            store_arg,
        ])
        .output()
        .expect("create pull subscription");
    assert!(
        create.status.success(),
        "{}",
        String::from_utf8_lossy(&create.stderr)
    );
    assert!(String::from_utf8_lossy(&create.stdout).contains("\"name\": \"worker-a\""));

    let list = Command::new(tape_bin())
        .args(["subscription", "list", "orders", "--store", store_arg])
        .output()
        .expect("list subscriptions");
    assert!(list.status.success());
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert!(list_stdout.contains("worker-a"));

    let show = Command::new(tape_bin())
        .args([
            "subscription",
            "show",
            "orders",
            "worker-a",
            "--store",
            store_arg,
        ])
        .output()
        .expect("show pull subscription");
    assert!(show.status.success());
    assert!(String::from_utf8_lossy(&show.stdout).contains("\"checkpoint\": null"));

    let delete = Command::new(tape_bin())
        .args([
            "subscription",
            "delete",
            "orders",
            "worker-a",
            "--store",
            store_arg,
        ])
        .output()
        .expect("delete pull subscription");
    assert!(delete.status.success());

    let remaining = Command::new(tape_bin())
        .args(["subscription", "list", "orders", "--store", store_arg])
        .output()
        .expect("list remaining subscriptions");
    assert!(remaining.status.success());
    let remaining_stdout = String::from_utf8_lossy(&remaining.stdout);
    assert!(!remaining_stdout.contains("worker-a"));
    assert!(remaining_stdout.contains("\"subscriptions\": []"));

    let _ = std::fs::remove_file(store);
}

#[test]
fn subscription_spec_inventory() {
    let routes = Command::new(tape_bin())
        .args(["spec", "--format", "routes"])
        .output()
        .expect("run tape spec routes");
    assert!(routes.status.success());
    let routes_stdout = String::from_utf8_lossy(&routes.stdout);
    assert!(routes_stdout.contains("/topics/{topic}/subscriptions"));
    assert!(routes_stdout.contains("/topics/{topic}/subscriptions/{subscription}"));

    let openapi = Command::new(tape_bin())
        .args(["spec", "--format", "openapi"])
        .output()
        .expect("run tape spec openapi");
    assert!(openapi.status.success());
    let openapi_stdout = String::from_utf8_lossy(&openapi.stdout);
    assert!(openapi_stdout.contains("SubscriptionCreateRequest"));
    assert!(!openapi_stdout.contains("DeliveryConfig"));

    let schema = Command::new(tape_bin())
        .args(["spec", "--format", "json-schema"])
        .output()
        .expect("run tape spec json-schema");
    assert!(schema.status.success());
    let schema_stdout = String::from_utf8_lossy(&schema.stdout);
    assert!(schema_stdout.contains("SubscriptionListResponse"));
    assert!(!schema_stdout.contains("\"mode\""));
}

#[test]
fn pull_subscription_cli_roundtrip() {
    let store = std::env::temp_dir().join(format!(
        "tape-pull-subscription-contract-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let store_arg = store.to_str().unwrap();

    for payload in [r#"{"id":"o1"}"#, r#"{"id":"o2"}"#] {
        let append = Command::new(tape_bin())
            .args([
                "append",
                "orders",
                "--payload",
                payload,
                "--store",
                store_arg,
            ])
            .output()
            .expect("append pull fixture");
        assert!(append.status.success());
    }
    let create = Command::new(tape_bin())
        .args([
            "subscription",
            "create",
            "orders",
            "worker-a",
            "--store",
            store_arg,
        ])
        .output()
        .expect("create pull subscription");
    assert!(create.status.success());

    let pull = Command::new(tape_bin())
        .args([
            "subscription",
            "pull",
            "orders",
            "worker-a",
            "--limit",
            "1",
            "--store",
            store_arg,
        ])
        .output()
        .expect("pull first window");
    assert!(
        pull.status.success(),
        "{}",
        String::from_utf8_lossy(&pull.stderr)
    );
    let pull_stdout = String::from_utf8_lossy(&pull.stdout);
    assert!(pull_stdout.contains("\"cursor\": 0"));
    assert!(pull_stdout.contains("\"next_offset\": 1"));
    assert!(pull_stdout.contains("\"id\": \"o1\""));
    assert!(pull_stdout.contains("next: tape subscription ack orders worker-a --offset 1"));

    let ack = Command::new(tape_bin())
        .args([
            "subscription",
            "ack",
            "orders",
            "worker-a",
            "--offset",
            "1",
            "--store",
            store_arg,
        ])
        .output()
        .expect("ack first window");
    assert!(
        ack.status.success(),
        "{}",
        String::from_utf8_lossy(&ack.stderr)
    );

    let next_pull = Command::new(tape_bin())
        .args([
            "subscription",
            "pull",
            "orders",
            "worker-a",
            "--limit",
            "1",
            "--store",
            store_arg,
        ])
        .output()
        .expect("pull second window");
    assert!(next_pull.status.success());
    let next_stdout = String::from_utf8_lossy(&next_pull.stdout);
    assert!(next_stdout.contains("\"cursor\": 1"));
    assert!(next_stdout.contains("\"id\": \"o2\""));

    let _ = std::fs::remove_file(store);
}

#[test]
fn pull_subscription_spec_inventory() {
    let routes = Command::new(tape_bin())
        .args(["spec", "--format", "routes"])
        .output()
        .expect("run tape spec routes");
    assert!(routes.status.success());
    let routes_stdout = String::from_utf8_lossy(&routes.stdout);
    assert!(routes_stdout.contains("/topics/{topic}/subscriptions/{subscription}/pull"));
    assert!(routes_stdout.contains("/topics/{topic}/subscriptions/{subscription}/ack"));

    let openapi = Command::new(tape_bin())
        .args(["spec", "--format", "openapi"])
        .output()
        .expect("run tape spec openapi");
    assert!(openapi.status.success());
    let stdout = String::from_utf8_lossy(&openapi.stdout);
    assert!(stdout.contains("PullSubscriptionBatch"));
    assert!(stdout.contains("PullSubscriptionAckRequest"));
    assert!(stdout.contains("\"maximum\": 1000"));
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
